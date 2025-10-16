// Expert Witness Management Service - Feature #9
// Expert database, qualifications, rates, and scheduling

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertWitness {
    pub id: String,
    pub name: String,
    pub credentials: Vec<String>,
    pub specialties: Vec<String>,
    pub hourly_rate: f64,
    pub cv_path: Option<String>,
    pub availability: Vec<AvailabilitySlot>,
    pub past_cases: Vec<PastCase>,
    pub rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilitySlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastCase {
    pub case_name: String,
    pub year: u32,
    pub outcome: String,
}

pub struct ExpertWitnessService {
    db: SqlitePool,
}

impl ExpertWitnessService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn search_experts(&self, specialty: &str) -> Result<Vec<ExpertWitness>> {
        Ok(vec![])
    }

    pub async fn book_expert(&self, expert_id: &str, date: DateTime<Utc>) -> Result<()> {
        Ok(())
    }
}
