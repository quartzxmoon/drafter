// Integration tests for PA eDocket Desktop

use anyhow::Result;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;

// Import the modules we want to test
use pa_edocket_desktop::services::database::DatabaseService;
use pa_edocket_desktop::services::automation::{AutomationService, JobsConfig};
use pa_edocket_desktop::domain::*;

#[tokio::test]
async fn test_database_integration() -> Result<()> {
    // Create a temporary database for testing
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}", db_path.display());
    
    // Initialize database service
    let db_service = Arc::new(DatabaseService::new(&database_url).await?);
    
    // Test health check
    db_service.health_check().await?;
    
    // Test search cache functionality
    let search_results = vec![
        SearchResult {
            id: "result-1".to_string(),
            docket_number: "CP-51-CR-0001234-2023".to_string(),
            case_title: "Commonwealth v. Doe".to_string(),
            court: "Philadelphia County Court of Common Pleas".to_string(),
            filing_date: Some("2023-01-15".to_string()),
            status: "Active".to_string(),
            case_type: "Criminal".to_string(),
            parties: vec!["Commonwealth of Pennsylvania".to_string(), "John Doe".to_string()],
            last_activity: Some("2023-12-01".to_string()),
            judge: Some("Hon. Mary Johnson".to_string()),
        }
    ];
    
    let query_hash = "test_query_hash";
    let cache_id = db_service.cache_search_results(query_hash, &search_results, 24).await?;
    assert!(!cache_id.is_empty());
    
    // Retrieve cached results
    let cached_results = db_service.get_cached_search_results(query_hash).await?;
    assert!(cached_results.is_some());
    let cached = cached_results.unwrap();
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].docket_number, "CP-51-CR-0001234-2023");
    
    // Test docket cache functionality
    let docket = Docket {
        id: "docket-1".to_string(),
        docket_number: "CP-51-CR-0001234-2023".to_string(),
        court: "Philadelphia County Court of Common Pleas".to_string(),
        case_type: "Criminal".to_string(),
        status: "Active".to_string(),
        filing_date: Some("2023-01-15".to_string()),
        parties: vec![],
        charges: vec![],
        events: vec![],
        filings: vec![],
        financials: vec![],
        attachments: vec![],
        judge: Some("Hon. Mary Johnson".to_string()),
        division: Some("Criminal Division".to_string()),
        last_updated: Some("2023-12-01T10:00:00Z".to_string()),
        source: "UJS Portal".to_string(),
    };
    
    let docket_cache_id = db_service.cache_docket(&docket).await?;
    assert!(!docket_cache_id.is_empty());
    
    // Retrieve cached docket
    let cached_docket = db_service.get_cached_docket(&docket.docket_number, &docket.court).await?;
    assert!(cached_docket.is_some());
    let cached = cached_docket.unwrap();
    assert_eq!(cached.docket_number, "CP-51-CR-0001234-2023");
    
    // Test watchlist functionality
    let watchlist_id = db_service.create_watchlist("Test Watchlist", Some("Test description")).await?;
    assert!(!watchlist_id.is_empty());
    
    let watchlists = db_service.get_watchlists().await?;
    assert_eq!(watchlists.len(), 1);
    assert_eq!(watchlists[0].name, "Test Watchlist");
    
    // Add item to watchlist
    let item_id = db_service.add_to_watchlist(
        &watchlist_id,
        "CP-51-CR-0001234-2023",
        "Philadelphia County Court of Common Pleas",
        Some("Important case to monitor")
    ).await?;
    assert!(!item_id.is_empty());
    
    let watchlist_items = db_service.get_watchlist_items(&watchlist_id).await?;
    assert_eq!(watchlist_items.len(), 1);
    assert_eq!(watchlist_items[0].docket_number, "CP-51-CR-0001234-2023");
    
    // Test export history
    let export_id = db_service.record_export(
        "PDF",
        "/tmp/test_export.pdf",
        Some(&serde_json::json!({"files": 1, "size": 1024}))
    ).await?;
    assert!(!export_id.is_empty());
    
    let export_history = db_service.get_export_history(Some(10)).await?;
    assert_eq!(export_history.len(), 1);
    assert_eq!(export_history[0].export_type, "PDF");
    
    // Test settings
    db_service.set_setting("test_key", "test_value").await?;
    let setting_value = db_service.get_setting("test_key").await?;
    assert_eq!(setting_value, Some("test_value".to_string()));
    
    let all_settings = db_service.get_all_settings().await?;
    assert!(all_settings.contains_key("test_key"));
    
    // Test database stats
    let stats = db_service.get_database_stats().await?;
    assert!(stats.contains_key("search_cache_count"));
    assert!(stats.contains_key("docket_cache_count"));
    assert!(stats.contains_key("watchlists_count"));
    
    // Test cleanup
    let cleaned_count = db_service.cleanup_expired_cache().await?;
    // Should be 0 since we just created the cache entries
    assert_eq!(cleaned_count, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_automation_integration() -> Result<()> {
    // Create a temporary database for testing
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}", db_path.display());
    
    let db_service = Arc::new(DatabaseService::new(&database_url).await?);
    
    // Create a minimal jobs config for testing
    let jobs_config = JobsConfig {
        jobs: std::collections::HashMap::new(),
        global: pa_edocket_desktop::services::automation::GlobalJobConfig {
            execution: pa_edocket_desktop::services::automation::ExecutionConfig {
                max_concurrent_jobs: 2,
                job_timeout_minutes: 5,
                cleanup_completed_jobs_hours: 1,
                cleanup_failed_jobs_hours: 24,
            },
            logging: pa_edocket_desktop::services::automation::LoggingConfig {
                level: "info".to_string(),
                log_job_start: true,
                log_job_completion: true,
                log_job_errors: true,
                structured_logging: true,
            },
            error_handling: pa_edocket_desktop::services::automation::ErrorHandlingConfig {
                max_consecutive_failures: 3,
                disable_job_on_failure: true,
                notification_on_failure: true,
                retry_exponential_backoff: true,
            },
            performance: pa_edocket_desktop::services::automation::PerformanceConfig {
                thread_pool_size: 2,
                queue_size: 10,
                batch_processing: true,
                rate_limiting: true,
            },
        },
        queues: std::collections::HashMap::new(),
        notifications: pa_edocket_desktop::services::automation::NotificationConfig {
            enabled: false,
            channels: vec![],
        },
        monitoring: pa_edocket_desktop::services::automation::MonitoringConfig {
            enabled: false,
            metrics: pa_edocket_desktop::services::automation::MetricsConfig {
                job_execution_time: true,
                job_success_rate: true,
                queue_size: true,
                system_resources: true,
            },
            retention: pa_edocket_desktop::services::automation::RetentionConfig {
                metrics_retention_days: 7,
                logs_retention_days: 3,
            },
            alerts: pa_edocket_desktop::services::automation::AlertsConfig {
                enabled: false,
                thresholds: std::collections::HashMap::new(),
            },
        },
    };
    
    // Initialize automation service
    let automation_service = AutomationService::new(jobs_config, db_service.clone()).await?;
    
    // Test that we can start the automation service
    // Note: In a real test, we might want to test actual job execution
    // but for now we'll just test initialization
    
    // Test getting running jobs (should be empty initially)
    let running_jobs = automation_service.get_running_jobs().await;
    assert_eq!(running_jobs.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_provider_integration() -> Result<()> {
    // Test that we can create provider instances
    use pa_edocket_desktop::providers::ujs_portal::UJSPortalProvider;
    use pa_edocket_desktop::providers::client::ProviderClient;
    
    let client = ProviderClient::new("https://ujsportal.pacourts.us".to_string(), None)?;
    let provider = UJSPortalProvider::new(client);
    
    // Test basic provider functionality (without making actual HTTP requests)
    // In a real integration test, you might want to test against a test server
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_integration() -> Result<()> {
    use pa_edocket_desktop::config::{ConfigManager, AppConfig};
    use std::path::PathBuf;
    
    // Create a temporary directory for config files
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().to_path_buf();
    
    let mut config_manager = ConfigManager::new(config_dir);
    
    // Test loading default configuration
    let config = config_manager.load_config().await?;
    assert_eq!(config.global.app_name, "PA eDocket Desktop");
    
    // Test saving and reloading configuration
    config_manager.save_config(config).await?;
    let reloaded_config = config_manager.reload_config().await?;
    assert_eq!(reloaded_config.global.app_name, "PA eDocket Desktop");
    
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow() -> Result<()> {
    // This test simulates a complete workflow:
    // 1. Initialize services
    // 2. Perform a search (cached)
    // 3. Retrieve a docket (cached)
    // 4. Add to watchlist
    // 5. Generate a citation
    // 6. Export data
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite://{}", db_path.display());
    
    let db_service = Arc::new(DatabaseService::new(&database_url).await?);
    
    // Step 1: Cache a search result
    let search_results = vec![
        SearchResult {
            id: "result-1".to_string(),
            docket_number: "CP-51-CR-0001234-2023".to_string(),
            case_title: "Commonwealth v. Doe".to_string(),
            court: "Philadelphia County Court of Common Pleas".to_string(),
            filing_date: Some("2023-01-15".to_string()),
            status: "Active".to_string(),
            case_type: "Criminal".to_string(),
            parties: vec!["Commonwealth of Pennsylvania".to_string(), "John Doe".to_string()],
            last_activity: Some("2023-12-01".to_string()),
            judge: Some("Hon. Mary Johnson".to_string()),
        }
    ];
    
    db_service.cache_search_results("test_query", &search_results, 24).await?;
    
    // Step 2: Cache a docket
    let docket = Docket {
        id: "docket-1".to_string(),
        docket_number: "CP-51-CR-0001234-2023".to_string(),
        court: "Philadelphia County Court of Common Pleas".to_string(),
        case_type: "Criminal".to_string(),
        status: "Active".to_string(),
        filing_date: Some("2023-01-15".to_string()),
        parties: vec![],
        charges: vec![],
        events: vec![],
        filings: vec![],
        financials: vec![],
        attachments: vec![],
        judge: Some("Hon. Mary Johnson".to_string()),
        division: Some("Criminal Division".to_string()),
        last_updated: Some("2023-12-01T10:00:00Z".to_string()),
        source: "UJS Portal".to_string(),
    };
    
    db_service.cache_docket(&docket).await?;
    
    // Step 3: Create watchlist and add docket
    let watchlist_id = db_service.create_watchlist("E2E Test Watchlist", None).await?;
    db_service.add_to_watchlist(
        &watchlist_id,
        &docket.docket_number,
        &docket.court,
        Some("Added during E2E test")
    ).await?;
    
    // Step 4: Test citation generation
    use pa_edocket_desktop::services::citations::CitationService;
    let citation_service = CitationService::new();
    let citation = citation_service.generate_case_citation(&docket).await?;
    assert_eq!(citation.citation_type, CitationType::Case);
    
    // Step 5: Record an export
    db_service.record_export(
        "E2E_TEST",
        "/tmp/e2e_test_export.json",
        Some(&serde_json::json!({"test": true, "docket": docket.docket_number}))
    ).await?;
    
    // Verify the complete workflow
    let cached_search = db_service.get_cached_search_results("test_query").await?;
    assert!(cached_search.is_some());
    
    let cached_docket = db_service.get_cached_docket(&docket.docket_number, &docket.court).await?;
    assert!(cached_docket.is_some());
    
    let watchlist_items = db_service.get_watchlist_items(&watchlist_id).await?;
    assert_eq!(watchlist_items.len(), 1);
    
    let export_history = db_service.get_export_history(Some(1)).await?;
    assert_eq!(export_history.len(), 1);
    assert_eq!(export_history[0].export_type, "E2E_TEST");
    
    Ok(())
}
