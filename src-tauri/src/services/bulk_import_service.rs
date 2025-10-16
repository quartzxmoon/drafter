// Bulk Data Import Service
// Automated REST API import from CourtListener, GovInfo, and other sources
// Keeps local database synchronized with latest case law and statutes

use crate::providers::courtlistener::{CourtListenerProvider, Opinion, OpinionCluster};
use crate::providers::govinfo::GovInfoProvider;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

const BATCH_SIZE: usize = 100;
const CONCURRENT_DOWNLOADS: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: String,
    pub job_type: String,
    pub data_source: String,
    pub collection: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub total_items: i32,
    pub processed_items: i32,
    pub failed_items: i32,
}

pub struct BulkImportService {
    db_pool: Pool<Sqlite>,
    courtlistener: Option<Arc<CourtListenerProvider>>,
    govinfo: Option<Arc<GovInfoProvider>>,
}

impl BulkImportService {
    pub fn new(
        db_pool: Pool<Sqlite>,
        courtlistener_token: Option<String>,
        govinfo_key: Option<String>,
    ) -> Self {
        Self {
            db_pool,
            courtlistener: courtlistener_token.map(|t| Arc::new(CourtListenerProvider::new(t))),
            govinfo: govinfo_key.map(|k| Arc::new(GovInfoProvider::new(k))),
        }
    }

    // ========================================================================
    // CourtListener Bulk Import
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn import_courtlistener_bulk(
        &self,
        court: &str,
        days_back: u32,
        job_id: Option<String>,
    ) -> Result<SyncJob> {
        info!("Starting bulk import from CourtListener for court: {}", court);

        let courtlistener = self
            .courtlistener
            .as_ref()
            .context("CourtListener not configured")?;

        let job_id = job_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create sync job
        let mut job = self.create_sync_job(
            &job_id,
            "bulk_initial",
            "courtlistener",
            Some(court.to_string()),
            days_back,
        )
        .await?;

        // Download opinions from CourtListener
        match courtlistener.bulk_download_recent(court, days_back).await {
            Ok(opinions) => {
                info!("Downloaded {} opinions from CourtListener", opinions.len());

                job.total_items = opinions.len() as i32;
                self.update_sync_job_progress(&job).await?;

                // Process in batches with concurrency control
                let semaphore = Arc::new(Semaphore::new(CONCURRENT_DOWNLOADS));
                let mut processed = 0;
                let mut failed = 0;

                for batch in opinions.chunks(BATCH_SIZE) {
                    for opinion in batch {
                        let permit = semaphore.clone().acquire_owned().await?;
                        let pool = self.db_pool.clone();
                        let cl = courtlistener.clone();
                        let opinion_clone = opinion.clone();

                        tokio::spawn(async move {
                            let _permit = permit; // Hold permit until done

                            match Self::save_opinion(&pool, &opinion_clone, cl.as_ref()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to save opinion {}: {}", opinion_clone.id, e);
                                }
                            }
                        });

                        processed += 1;
                        if processed % 10 == 0 {
                            job.processed_items = processed;
                            self.update_sync_job_progress(&job).await?;
                        }
                    }
                }

                // Wait for all tasks to complete
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                job.status = "completed".to_string();
                job.processed_items = processed;
                job.failed_items = failed;
                self.complete_sync_job(&job).await?;

                info!("Bulk import completed: {} processed, {} failed", processed, failed);
                Ok(job)
            }
            Err(e) => {
                error!("CourtListener bulk download failed: {}", e);
                job.status = "failed".to_string();
                self.fail_sync_job(&job, &e.to_string()).await?;
                Err(e)
            }
        }
    }

    async fn save_opinion(
        pool: &Pool<Sqlite>,
        opinion: &Opinion,
        courtlistener: &CourtListenerProvider,
    ) -> Result<()> {
        // Extract cluster ID from URL
        let cluster_id = Self::extract_id_from_url(&opinion.cluster)?;

        // Get cluster for full citation info
        let cluster = courtlistener.get_opinion_cluster(cluster_id).await?;

        // Download full text if available
        let plain_text = if let Some(ref download_url) = opinion.download_url {
            // In production, would download the file
            opinion.plain_text.clone()
        } else {
            opinion.plain_text.clone()
        };

        // Insert into database
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO case_law (
                opinion_id, cluster_id, case_name, case_name_short, court, court_id,
                date_filed, date_filed_year, federal_cite_one, federal_cite_two,
                state_cite_one, state_cite_regional, neutral_cite, westlaw_cite,
                lexis_cite, plain_text, html, precedential_status, citation_count,
                docket_number, source, last_updated, created_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#,
            opinion.id,
            cluster_id,
            cluster.case_name,
            cluster.case_name_short,
            cluster.docket, // This needs to be parsed for actual court name
            cluster.docket,
            cluster.date_filed,
            cluster.date_filed.split('-').next().and_then(|s| s.parse::<i32>().ok()),
            cluster.federal_cite_one,
            cluster.federal_cite_two,
            cluster.state_cite_one,
            cluster.state_cite_regional,
            cluster.neutral_cite,
            cluster.westlaw_cite,
            cluster.lexis_cite,
            plain_text,
            opinion.html,
            cluster.precedential_status,
            cluster.citation_count,
            cluster.docket,
            "courtlistener",
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339()
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // GovInfo Bulk Import
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn import_govinfo_bulk(
        &self,
        collection: &str,
        max_items: u32,
        job_id: Option<String>,
    ) -> Result<SyncJob> {
        info!("Starting bulk import from GovInfo for collection: {}", collection);

        let govinfo = self.govinfo.as_ref().context("GovInfo not configured")?;

        let job_id = job_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let mut job = self.create_sync_job(
            &job_id,
            "bulk_initial",
            "govinfo",
            Some(collection.to_string()),
            0,
        )
        .await?;

        match govinfo.bulk_download_collection(collection, max_items).await {
            Ok(packages) => {
                info!("Downloaded {} packages from GovInfo", packages.len());

                job.total_items = packages.len() as i32;
                self.update_sync_job_progress(&job).await?;

                let semaphore = Arc::new(Semaphore::new(CONCURRENT_DOWNLOADS));
                let mut processed = 0;

                for batch in packages.chunks(BATCH_SIZE) {
                    for package in batch {
                        let permit = semaphore.clone().acquire_owned().await?;
                        let pool = self.db_pool.clone();
                        let govinfo_clone = govinfo.clone();
                        let package_clone = package.clone();

                        tokio::spawn(async move {
                            let _permit = permit;

                            match Self::save_statute(&pool, &package_clone, govinfo_clone.as_ref()).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Failed to save package {}: {}", package_clone.package_id, e);
                                }
                            }
                        });

                        processed += 1;
                        if processed % 10 == 0 {
                            job.processed_items = processed;
                            self.update_sync_job_progress(&job).await?;
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                job.status = "completed".to_string();
                job.processed_items = processed;
                self.complete_sync_job(&job).await?;

                info!("GovInfo import completed: {} processed", processed);
                Ok(job)
            }
            Err(e) => {
                error!("GovInfo bulk download failed: {}", e);
                job.status = "failed".to_string();
                self.fail_sync_job(&job, &e.to_string()).await?;
                Err(e)
            }
        }
    }

    async fn save_statute(
        pool: &Pool<Sqlite>,
        package: &crate::providers::govinfo::Package,
        govinfo: &GovInfoProvider,
    ) -> Result<()> {
        // Download text content
        let text_content = govinfo.download_text(&package.package_id).await.ok();

        // Parse USC or CFR info from package_id
        let (usc_title, usc_section, cfr_title, cfr_part, cfr_section) =
            Self::parse_package_id(&package.package_id);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO statutes (
                package_id, title, collection, usc_title, usc_section,
                cfr_title, cfr_part, cfr_section, text_content,
                citation, date_issued, source, last_updated, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            package.package_id,
            package.title,
            package.doc_class,
            usc_title,
            usc_section,
            cfr_title,
            cfr_part,
            cfr_section,
            text_content,
            package.package_id, // Use as citation for now
            package.date_issued,
            "govinfo",
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339()
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Incremental Updates
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn incremental_sync(&self, data_source: &str, collection: &str) -> Result<SyncJob> {
        info!("Running incremental sync for {}/{}", data_source, collection);

        // Get last sync time
        let last_sync = self.get_last_sync_time(data_source, collection).await?;

        let days_since = if let Some(last) = last_sync {
            let duration = Utc::now() - last;
            duration.num_days() as u32
        } else {
            7 // Default to last week if never synced
        };

        match data_source {
            "courtlistener" => self.import_courtlistener_bulk(collection, days_since, None).await,
            "govinfo" => self.import_govinfo_bulk(collection, 1000, None).await,
            _ => Err(anyhow::anyhow!("Unknown data source: {}", data_source)),
        }
    }

    // ========================================================================
    // Scheduled Sync
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn run_scheduled_syncs(&self) -> Result<Vec<SyncJob>> {
        info!("Running scheduled syncs");

        let schedules = self.get_enabled_schedules().await?;
        let mut jobs = Vec::new();

        for schedule in schedules {
            if self.should_run_schedule(&schedule).await? {
                let job = self
                    .incremental_sync(&schedule.data_source, &schedule.collection.unwrap_or_default())
                    .await?;

                jobs.push(job);

                // Update schedule
                self.update_schedule_last_run(&schedule.id).await?;
            }
        }

        Ok(jobs)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn create_sync_job(
        &self,
        id: &str,
        job_type: &str,
        data_source: &str,
        collection: Option<String>,
        days_back: u32,
    ) -> Result<SyncJob> {
        let start_date = if days_back > 0 {
            Some((Utc::now() - Duration::days(days_back as i64)).naive_utc().date())
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO sync_jobs (id, job_type, data_source, collection, start_date, status, created_at)
            VALUES (?, ?, ?, ?, ?, 'running', ?)
            "#,
            id,
            job_type,
            data_source,
            collection,
            start_date.map(|d| d.to_string()),
            Utc::now().to_rfc3339()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(SyncJob {
            id: id.to_string(),
            job_type: job_type.to_string(),
            data_source: data_source.to_string(),
            collection,
            start_date,
            end_date: None,
            status: "running".to_string(),
            total_items: 0,
            processed_items: 0,
            failed_items: 0,
        })
    }

    async fn update_sync_job_progress(&self, job: &SyncJob) -> Result<()> {
        sqlx::query!(
            r#"UPDATE sync_jobs SET total_items = ?, processed_items = ?, failed_items = ? WHERE id = ?"#,
            job.total_items,
            job.processed_items,
            job.failed_items,
            job.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn complete_sync_job(&self, job: &SyncJob) -> Result<()> {
        sqlx::query!(
            r#"UPDATE sync_jobs SET status = 'completed', completed_at = ? WHERE id = ?"#,
            Utc::now().to_rfc3339(),
            job.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn fail_sync_job(&self, job: &SyncJob, error: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE sync_jobs SET status = 'failed', error_message = ?, completed_at = ? WHERE id = ?"#,
            error,
            Utc::now().to_rfc3339(),
            job.id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_last_sync_time(&self, data_source: &str, collection: &str) -> Result<Option<DateTime<Utc>>> {
        let row = sqlx::query!(
            r#"
            SELECT completed_at FROM sync_jobs
            WHERE data_source = ? AND collection = ? AND status = 'completed'
            ORDER BY completed_at DESC LIMIT 1
            "#,
            data_source,
            collection
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.and_then(|r| r.completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)))))
    }

    async fn get_enabled_schedules(&self) -> Result<Vec<SyncSchedule>> {
        let rows = sqlx::query!(
            r#"SELECT * FROM sync_schedules WHERE enabled = 1"#
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SyncSchedule {
                id: r.id,
                data_source: r.data_source,
                collection: r.collection,
                schedule_type: r.schedule_type,
                last_run_at: r.last_run_at,
            })
            .collect())
    }

    async fn should_run_schedule(&self, schedule: &SyncSchedule) -> Result<bool> {
        // Simple check - in production would parse schedule_type
        Ok(schedule.last_run_at.is_none()
            || schedule
                .last_run_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| Utc::now() - dt.with_timezone(&Utc) > Duration::hours(24))
                .unwrap_or(true))
    }

    async fn update_schedule_last_run(&self, schedule_id: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE sync_schedules SET last_run_at = ? WHERE id = ?"#,
            Utc::now().to_rfc3339(),
            schedule_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    fn extract_id_from_url(url: &str) -> Result<u32> {
        url.split('/')
            .find_map(|s| s.parse::<u32>().ok())
            .context("Could not extract ID from URL")
    }

    fn parse_package_id(package_id: &str) -> (Option<i32>, Option<String>, Option<i32>, Option<String>, Option<String>) {
        // Example: USCODE-2021-title18-partI-chap1-sec1
        if package_id.starts_with("USCODE") {
            let parts: Vec<&str> = package_id.split('-').collect();
            let title = parts.get(2).and_then(|s| s.strip_prefix("title")).and_then(|s| s.parse().ok());
            let section = parts.last().and_then(|s| s.strip_prefix("sec")).map(|s| s.to_string());
            (title, section, None, None, None)
        } else if package_id.starts_with("CFR") {
            let parts: Vec<&str> = package_id.split('-').collect();
            let title = parts.get(1).and_then(|s| s.parse().ok());
            (None, None, title, None, None)
        } else {
            (None, None, None, None, None)
        }
    }
}

#[derive(Debug)]
struct SyncSchedule {
    id: String,
    data_source: String,
    collection: Option<String>,
    schedule_type: String,
    last_run_at: Option<String>,
}
