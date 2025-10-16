// Discovery Management Service - Feature #10
// Document requests, interrogatories, production tracking, privilege logs

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRequest {
    pub id: String,
    pub matter_id: String,
    pub request_type: DiscoveryType,
    pub from_party: String,
    pub to_party: String,
    pub requests: Vec<DiscoveryItem>,
    pub due_date: DateTime<Utc>,
    pub status: DiscoveryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryType {
    DocumentRequest,
    Interrogatories,
    RequestForAdmission,
    Deposition,
    SubpoenaDucesTecum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryStatus {
    Pending,
    Responded,
    Objected,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryItem {
    pub number: u32,
    pub text: String,
    pub response: Option<String>,
    pub objection: Option<String>,
    pub documents_produced: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeLog {
    pub id: String,
    pub matter_id: String,
    pub entries: Vec<PrivilegeLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeLogEntry {
    pub document_id: String,
    pub date: DateTime<Utc>,
    pub author: String,
    pub recipient: String,
    pub description: String,
    pub privilege_type: PrivilegeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivilegeType {
    AttorneyClient,
    WorkProduct,
    AttorneyClientWorkProduct,
}

pub struct DiscoveryService {
    db: SqlitePool,
}

impl DiscoveryService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_discovery_request(
        &self,
        matter_id: &str,
        request_type: DiscoveryType,
    ) -> Result<DiscoveryRequest> {
        Ok(DiscoveryRequest {
            id: Uuid::new_v4().to_string(),
            matter_id: matter_id.to_string(),
            request_type,
            from_party: "Plaintiff".to_string(),
            to_party: "Defendant".to_string(),
            requests: vec![],
            due_date: Utc::now() + chrono::Duration::days(30),
            status: DiscoveryStatus::Pending,
        })
    }

    pub async fn generate_privilege_log(&self, matter_id: &str) -> Result<PrivilegeLog> {
        Ok(PrivilegeLog {
            id: Uuid::new_v4().to_string(),
            matter_id: matter_id.to_string(),
            entries: vec![],
        })
    }
}
