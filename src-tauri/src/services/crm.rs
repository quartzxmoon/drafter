// Client Intake & CRM Service - Feature #12
// Lead tracking, intake forms, client database, pipeline management

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub source: LeadSource,
    pub status: LeadStatus,
    pub practice_area: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeadSource {
    Website,
    Referral,
    Advertisement,
    SocialMedia,
    Walk_in,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeadStatus {
    New,
    Contacted,
    Qualified,
    Retained,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeForm {
    pub id: String,
    pub lead_id: String,
    pub fields: std::collections::HashMap<String, String>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct CRMService {
    db: SqlitePool,
}

impl CRMService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_lead(&self, name: &str, email: &str) -> Result<Lead> {
        Ok(Lead {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            email: email.to_string(),
            phone: String::new(),
            source: LeadSource::Website,
            status: LeadStatus::New,
            practice_area: String::new(),
            notes: String::new(),
            created_at: Utc::now(),
        })
    }

    pub async fn convert_to_client(&self, lead_id: &str) -> Result<String> {
        Ok(Uuid::new_v4().to_string())
    }
}
