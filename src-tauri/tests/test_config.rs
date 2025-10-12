// Test configuration and utilities

use std::sync::Once;
use tracing_subscriber;

static INIT: Once = Once::new();

/// Initialize logging for tests
pub fn init_test_logging() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_test_writer()
            .init();
    });
}

/// Test configuration for database
pub struct TestConfig {
    pub database_url: String,
    pub test_data_dir: String,
}

impl TestConfig {
    pub fn new() -> Self {
        Self {
            database_url: ":memory:".to_string(), // Use in-memory SQLite for tests
            test_data_dir: "/tmp/pa_edocket_test".to_string(),
        }
    }
}

/// Test utilities for creating mock data
pub mod mock_data {
    use crate::domain::*;
    use chrono::Utc;

    pub fn create_mock_search_result(id: &str, docket_number: &str) -> SearchResult {
        SearchResult {
            id: id.to_string(),
            docket_number: docket_number.to_string(),
            case_title: format!("Commonwealth v. Test Case {}", id),
            court: "Philadelphia County Court of Common Pleas".to_string(),
            filing_date: Some("2023-01-15".to_string()),
            status: "Active".to_string(),
            case_type: "Criminal".to_string(),
            parties: vec![
                "Commonwealth of Pennsylvania".to_string(),
                format!("Test Defendant {}", id),
            ],
            last_activity: Some("2023-12-01".to_string()),
            judge: Some("Hon. Test Judge".to_string()),
        }
    }

    pub fn create_mock_docket(id: &str, docket_number: &str) -> Docket {
        Docket {
            id: id.to_string(),
            docket_number: docket_number.to_string(),
            court: "Philadelphia County Court of Common Pleas".to_string(),
            case_type: "Criminal".to_string(),
            status: "Active".to_string(),
            filing_date: Some("2023-01-15".to_string()),
            parties: vec![
                Party {
                    id: format!("party-1-{}", id),
                    name: "Commonwealth of Pennsylvania".to_string(),
                    party_type: "Plaintiff".to_string(),
                    role: "Prosecutor".to_string(),
                    address: None,
                    attorney: None,
                    status: "Active".to_string(),
                },
                Party {
                    id: format!("party-2-{}", id),
                    name: format!("Test Defendant {}", id),
                    party_type: "Defendant".to_string(),
                    role: "Defendant".to_string(),
                    address: None,
                    attorney: Some(Attorney {
                        id: format!("atty-{}", id),
                        name: "Test Attorney, Esq.".to_string(),
                        bar_number: Some("PA12345".to_string()),
                        firm: Some("Test Law Firm".to_string()),
                        address: None,
                        phone: None,
                        email: None,
                    }),
                    status: "Active".to_string(),
                },
            ],
            charges: vec![
                Charge {
                    id: format!("charge-{}", id),
                    statute: "18 Pa.C.S. § 3502".to_string(),
                    description: "Burglary".to_string(),
                    grade: "F2".to_string(),
                    disposition: Some("Guilty Plea".to_string()),
                    disposition_date: Some("2023-06-15".to_string()),
                    sentence: Some("2-4 years imprisonment".to_string()),
                    sentence_date: Some("2023-07-01".to_string()),
                }
            ],
            events: vec![
                Event {
                    id: format!("event-{}", id),
                    date: "2023-01-15".to_string(),
                    time: Some("09:00".to_string()),
                    description: "Case Filed".to_string(),
                    event_type: "Filing".to_string(),
                    location: Some("Clerk's Office".to_string()),
                    judge: None,
                    result: None,
                }
            ],
            filings: vec![
                Filing {
                    id: format!("filing-{}", id),
                    filing_date: "2023-01-15".to_string(),
                    document_type: "Information".to_string(),
                    description: "Criminal Information".to_string(),
                    filed_by: "District Attorney's Office".to_string(),
                    status: "Filed".to_string(),
                    pages: Some(3),
                    attachments: vec![],
                }
            ],
            financials: vec![
                Financial {
                    id: format!("financial-{}", id),
                    transaction_type: "Fine".to_string(),
                    amount: 500.00,
                    description: "Court costs and fines".to_string(),
                    date: "2023-07-01".to_string(),
                    status: "Assessed".to_string(),
                    balance: Some(500.00),
                }
            ],
            attachments: vec![],
            judge: Some("Hon. Test Judge".to_string()),
            division: Some("Criminal Division".to_string()),
            last_updated: Some(Utc::now().to_rfc3339()),
            source: "UJS Portal".to_string(),
        }
    }

    pub fn create_mock_citation(id: &str, citation_type: CitationType) -> Citation {
        match citation_type {
            CitationType::Case => Citation {
                id: id.to_string(),
                citation_type: CitationType::Case,
                case_name: Some(format!("Commonwealth v. Test Case {}", id)),
                volume: Some("123".to_string()),
                reporter: Some("Pa.".to_string()),
                page: Some("456".to_string()),
                year: Some("2023".to_string()),
                ..Default::default()
            },
            CitationType::Statute => Citation {
                id: id.to_string(),
                citation_type: CitationType::Statute,
                title: Some("18".to_string()),
                code: Some("Pa.C.S.".to_string()),
                section: Some("3502".to_string()),
                ..Default::default()
            },
            CitationType::Rule => Citation {
                id: id.to_string(),
                citation_type: CitationType::Rule,
                rule_set: Some("Pa.R.Crim.P.".to_string()),
                rule_number: Some("600".to_string()),
                ..Default::default()
            },
            CitationType::Filing => Citation {
                id: id.to_string(),
                citation_type: CitationType::Filing,
                case_name: Some(format!("Commonwealth v. Test Case {}", id)),
                docket_number: Some(format!("CP-51-CR-{}-2023", id)),
                document_type: Some("Motion".to_string()),
                filing_date: Some("2023-01-20".to_string()),
                ..Default::default()
            },
        }
    }

    pub fn create_mock_job_config(job_id: &str) -> crate::services::automation::JobConfig {
        crate::services::automation::JobConfig {
            name: format!("Test Job {}", job_id),
            description: format!("Test job description for {}", job_id),
            enabled: true,
            schedule: "0 */6 * * *".to_string(),
            job_type: "recurring".to_string(),
            priority: "medium".to_string(),
            timeout_minutes: 30,
            retry: crate::services::automation::RetryConfig {
                max_attempts: 3,
                backoff_multiplier: 2.0,
                initial_delay_seconds: 60,
            },
            config: serde_json::json!({
                "test_param": format!("value_{}", job_id),
                "batch_size": 10
            }),
        }
    }
}

/// Test assertions and helpers
pub mod assertions {
    use crate::domain::*;

    pub fn assert_search_result_valid(result: &SearchResult) {
        assert!(!result.id.is_empty());
        assert!(!result.docket_number.is_empty());
        assert!(!result.case_title.is_empty());
        assert!(!result.court.is_empty());
        assert!(!result.status.is_empty());
        assert!(!result.case_type.is_empty());
    }

    pub fn assert_docket_valid(docket: &Docket) {
        assert!(!docket.id.is_empty());
        assert!(!docket.docket_number.is_empty());
        assert!(!docket.court.is_empty());
        assert!(!docket.case_type.is_empty());
        assert!(!docket.status.is_empty());
        assert!(!docket.source.is_empty());
    }

    pub fn assert_citation_valid(citation: &Citation) {
        assert!(!citation.id.is_empty());
        
        match citation.citation_type {
            CitationType::Case => {
                assert!(citation.case_name.is_some());
                assert!(citation.volume.is_some() || citation.docket_number.is_some());
            },
            CitationType::Statute => {
                assert!(citation.title.is_some());
                assert!(citation.code.is_some());
                assert!(citation.section.is_some());
            },
            CitationType::Rule => {
                assert!(citation.rule_set.is_some());
                assert!(citation.rule_number.is_some());
            },
            CitationType::Filing => {
                assert!(citation.docket_number.is_some());
                assert!(citation.document_type.is_some());
            },
        }
    }

    pub fn assert_watchlist_valid(watchlist: &crate::services::database::Watchlist) {
        assert!(!watchlist.id.is_empty());
        assert!(!watchlist.name.is_empty());
    }

    pub fn assert_export_record_valid(record: &crate::services::database::ExportRecord) {
        assert!(!record.id.is_empty());
        assert!(!record.export_type.is_empty());
        assert!(!record.file_path.is_empty());
    }
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    pub struct PerformanceTimer {
        start: Instant,
        name: String,
    }

    impl PerformanceTimer {
        pub fn new(name: &str) -> Self {
            Self {
                start: Instant::now(),
                name: name.to_string(),
            }
        }

        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }

        pub fn assert_under(&self, max_duration: Duration) {
            let elapsed = self.elapsed();
            assert!(
                elapsed <= max_duration,
                "{} took {:?}, expected under {:?}",
                self.name,
                elapsed,
                max_duration
            );
        }
    }

    pub fn assert_performance<F, R>(name: &str, max_duration: Duration, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = PerformanceTimer::new(name);
        let result = f();
        timer.assert_under(max_duration);
        result
    }

    pub async fn assert_async_performance<F, Fut, R>(
        name: &str,
        max_duration: Duration,
        f: F,
    ) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let timer = PerformanceTimer::new(name);
        let result = f().await;
        timer.assert_under(max_duration);
        result
    }
}

/// Database testing utilities
pub mod database {
    use anyhow::Result;
    use std::sync::Arc;
    use tempfile::TempDir;
    use crate::services::database::DatabaseService;

    pub async fn create_test_database() -> Result<(Arc<DatabaseService>, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite://{}", db_path.display());
        
        let db_service = Arc::new(DatabaseService::new(&database_url).await?);
        
        Ok((db_service, temp_dir))
    }

    pub async fn create_in_memory_database() -> Result<Arc<DatabaseService>> {
        let database_url = "sqlite::memory:".to_string();
        let db_service = Arc::new(DatabaseService::new(&database_url).await?);
        Ok(db_service)
    }
}
