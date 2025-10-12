// Database service for PA eDocket Desktop

use crate::domain::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSearchResult {
    pub id: String,
    pub query_hash: String,
    pub results: Vec<SearchResult>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDocket {
    pub id: String,
    pub docket_number: String,
    pub court_id: String,
    pub data: Docket,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: String,
    pub watchlist_id: String,
    pub docket_number: String,
    pub court_id: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecord {
    pub id: String,
    pub export_type: String,
    pub file_path: String,
    pub manifest: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

pub struct DatabaseService {
    pool: Pool<Sqlite>,
}

impl DatabaseService {
    #[instrument]
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Initializing database connection to: {}", database_url);

        let pool = SqlitePool::connect(database_url).await
            .context("Failed to connect to SQLite database")?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await
            .context("Failed to run database migrations")?;

        info!("Database initialized successfully");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await
            .context("Database health check failed")?;
        debug!("Database health check passed");
        Ok(())
    }

    // Search Cache Methods
    #[instrument(skip(self, results))]
    pub async fn cache_search_results(
        &self,
        query_hash: &str,
        results: &[SearchResult],
        ttl_hours: i64,
    ) -> Result<String> {
        debug!("Caching search results for query hash: {}", query_hash);

        let id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(ttl_hours);
        let results_json = serde_json::to_string(results)
            .context("Failed to serialize search results")?;

        sqlx::query!(
            r#"
            INSERT INTO search_cache (id, query_hash, results, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
            id,
            query_hash,
            results_json,
            expires_at
        )
        .execute(&self.pool)
        .await
        .context("Failed to cache search results")?;

        debug!("Cached {} search results with ID: {}", results.len(), id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_cached_search_results(&self, query_hash: &str) -> Result<Option<Vec<SearchResult>>> {
        debug!("Retrieving cached search results for query hash: {}", query_hash);

        let row = sqlx::query!(
            r#"
            SELECT results FROM search_cache
            WHERE query_hash = ? AND expires_at > CURRENT_TIMESTAMP
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            query_hash
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query search cache")?;

        if let Some(row) = row {
            let results: Vec<SearchResult> = serde_json::from_str(&row.results)
                .context("Failed to deserialize cached search results")?;
            debug!("Found {} cached search results", results.len());
            Ok(Some(results))
        } else {
            debug!("No cached search results found for query hash: {}", query_hash);
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    pub async fn cleanup_expired_cache(&self) -> Result<usize> {
        debug!("Cleaning up expired cache entries");

        let result = sqlx::query!(
            "DELETE FROM search_cache WHERE expires_at <= CURRENT_TIMESTAMP"
        )
        .execute(&self.pool)
        .await
        .context("Failed to cleanup expired cache")?;

        let deleted_count = result.rows_affected() as usize;
        if deleted_count > 0 {
            info!("Cleaned up {} expired cache entries", deleted_count);
        }

        Ok(deleted_count)
    }

    // Docket Cache Methods
    #[instrument(skip(self, docket))]
    pub async fn cache_docket(&self, docket: &Docket) -> Result<String> {
        debug!("Caching docket: {}", docket.docket_number);

        let id = Uuid::new_v4().to_string();
        let docket_json = serde_json::to_string(docket)
            .context("Failed to serialize docket")?;

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO docket_cache (id, docket_number, court_id, data, last_updated)
            VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
            id,
            docket.docket_number,
            docket.court,
            docket_json
        )
        .execute(&self.pool)
        .await
        .context("Failed to cache docket")?;

        debug!("Cached docket with ID: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_cached_docket(&self, docket_number: &str, court_id: &str) -> Result<Option<Docket>> {
        debug!("Retrieving cached docket: {} from court: {}", docket_number, court_id);

        let row = sqlx::query!(
            r#"
            SELECT data FROM docket_cache
            WHERE docket_number = ? AND court_id = ?
            ORDER BY last_updated DESC
            LIMIT 1
            "#,
            docket_number,
            court_id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query docket cache")?;

        if let Some(row) = row {
            let docket: Docket = serde_json::from_str(&row.data)
                .context("Failed to deserialize cached docket")?;
            debug!("Found cached docket: {}", docket_number);
            Ok(Some(docket))
        } else {
            debug!("No cached docket found: {}", docket_number);
            Ok(None)
        }
    }

    // Watchlist Methods
    #[instrument(skip(self))]
    pub async fn create_watchlist(&self, name: &str, description: Option<&str>) -> Result<String> {
        info!("Creating watchlist: {}", name);

        let id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO watchlists (id, name, description)
            VALUES (?, ?, ?)
            "#,
            id,
            name,
            description
        )
        .execute(&self.pool)
        .await
        .context("Failed to create watchlist")?;

        info!("Created watchlist with ID: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_watchlists(&self) -> Result<Vec<Watchlist>> {
        debug!("Retrieving all watchlists");

        let rows = sqlx::query!(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM watchlists
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query watchlists")?;

        let watchlists = rows
            .into_iter()
            .map(|row| Watchlist {
                id: row.id,
                name: row.name,
                description: row.description,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
            })
            .collect();

        debug!("Retrieved {} watchlists", watchlists.len());
        Ok(watchlists)
    }

    #[instrument(skip(self))]
    pub async fn add_to_watchlist(
        &self,
        watchlist_id: &str,
        docket_number: &str,
        court_id: &str,
        notes: Option<&str>,
    ) -> Result<String> {
        debug!("Adding docket {} to watchlist {}", docket_number, watchlist_id);

        let id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO watchlist_items (id, watchlist_id, docket_number, court_id, notes)
            VALUES (?, ?, ?, ?, ?)
            "#,
            id,
            watchlist_id,
            docket_number,
            court_id,
            notes
        )
        .execute(&self.pool)
        .await
        .context("Failed to add item to watchlist")?;

        // Update watchlist timestamp
        sqlx::query!(
            "UPDATE watchlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            watchlist_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to update watchlist timestamp")?;

        debug!("Added watchlist item with ID: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_watchlist_items(&self, watchlist_id: &str) -> Result<Vec<WatchlistItem>> {
        debug!("Retrieving items for watchlist: {}", watchlist_id);

        let rows = sqlx::query!(
            r#"
            SELECT id, watchlist_id, docket_number, court_id, notes, created_at
            FROM watchlist_items
            WHERE watchlist_id = ?
            ORDER BY created_at DESC
            "#,
            watchlist_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query watchlist items")?;

        let items = rows
            .into_iter()
            .map(|row| WatchlistItem {
                id: row.id,
                watchlist_id: row.watchlist_id,
                docket_number: row.docket_number,
                court_id: row.court_id,
                notes: row.notes,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
            })
            .collect();

        debug!("Retrieved {} watchlist items", items.len());
        Ok(items)
    }

    #[instrument(skip(self))]
    pub async fn remove_from_watchlist(&self, item_id: &str) -> Result<()> {
        debug!("Removing watchlist item: {}", item_id);

        let result = sqlx::query!(
            "DELETE FROM watchlist_items WHERE id = ?",
            item_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to remove watchlist item")?;

        if result.rows_affected() == 0 {
            warn!("Watchlist item not found: {}", item_id);
        } else {
            debug!("Removed watchlist item: {}", item_id);
        }

        Ok(())
    }

    // Export History Methods
    #[instrument(skip(self, manifest))]
    pub async fn record_export(
        &self,
        export_type: &str,
        file_path: &str,
        manifest: Option<&serde_json::Value>,
    ) -> Result<String> {
        debug!("Recording export: {} to {}", export_type, file_path);

        let id = Uuid::new_v4().to_string();
        let manifest_json = manifest
            .map(|m| serde_json::to_string(m))
            .transpose()
            .context("Failed to serialize export manifest")?;

        sqlx::query!(
            r#"
            INSERT INTO export_history (id, export_type, file_path, manifest)
            VALUES (?, ?, ?, ?)
            "#,
            id,
            export_type,
            file_path,
            manifest_json
        )
        .execute(&self.pool)
        .await
        .context("Failed to record export")?;

        debug!("Recorded export with ID: {}", id);
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_export_history(&self, limit: Option<i64>) -> Result<Vec<ExportRecord>> {
        debug!("Retrieving export history");

        let limit = limit.unwrap_or(100);
        let rows = sqlx::query!(
            r#"
            SELECT id, export_type, file_path, manifest, created_at
            FROM export_history
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to query export history")?;

        let records = rows
            .into_iter()
            .map(|row| {
                let manifest = row.manifest
                    .as_ref()
                    .and_then(|m| serde_json::from_str(m).ok());

                ExportRecord {
                    id: row.id,
                    export_type: row.export_type,
                    file_path: row.file_path,
                    manifest,
                    created_at: DateTime::parse_from_rfc3339(&row.created_at)
                        .unwrap_or_default()
                        .with_timezone(&Utc),
                }
            })
            .collect();

        debug!("Retrieved {} export records", records.len());
        Ok(records)
    }

    // Settings Methods
    #[instrument(skip(self, value))]
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        debug!("Setting configuration: {} = {}", key, value);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO app_settings (key, value, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            "#,
            key,
            value
        )
        .execute(&self.pool)
        .await
        .context("Failed to set setting")?;

        debug!("Setting updated: {}", key);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        debug!("Getting setting: {}", key);

        let row = sqlx::query!(
            "SELECT value FROM app_settings WHERE key = ?",
            key
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get setting")?;

        Ok(row.map(|r| r.value))
    }

    #[instrument(skip(self))]
    pub async fn get_all_settings(&self) -> Result<HashMap<String, String>> {
        debug!("Getting all settings");

        let rows = sqlx::query!(
            "SELECT key, value FROM app_settings ORDER BY key"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get all settings")?;

        let settings = rows
            .into_iter()
            .map(|row| (row.key, row.value))
            .collect();

        debug!("Retrieved {} settings", settings.len());
        Ok(settings)
    }

    // Maintenance Methods
    #[instrument(skip(self))]
    pub async fn vacuum_database(&self) -> Result<()> {
        info!("Running database vacuum");

        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await
            .context("Failed to vacuum database")?;

        info!("Database vacuum completed");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_database_stats(&self) -> Result<HashMap<String, i64>> {
        debug!("Getting database statistics");

        let mut stats = HashMap::new();

        // Get table row counts
        let tables = ["search_cache", "docket_cache", "watchlists", "watchlist_items", "export_history", "app_settings"];

        for table in &tables {
            let count: i64 = sqlx::query(&format!("SELECT COUNT(*) as count FROM {}", table))
                .fetch_one(&self.pool)
                .await
                .context("Failed to get table count")?
                .get("count");

            stats.insert(format!("{}_count", table), count);
        }

        // Get database size
        let size: i64 = sqlx::query("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")
            .fetch_one(&self.pool)
            .await
            .context("Failed to get database size")?
            .get("size");

        stats.insert("database_size_bytes".to_string(), size);

        debug!("Retrieved database statistics");
        Ok(stats)
    }
}
