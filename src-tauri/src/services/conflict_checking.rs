// Conflict of Interest Checking System
// Automated conflict detection for parties, attorneys, and related entities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheck {
    pub id: String,
    pub matter_id: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub checked_by: String,
    pub parties: Vec<ConflictParty>,
    pub conflicts_found: Vec<Conflict>,
    pub status: ConflictStatus,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictParty {
    pub name: String,
    pub party_type: PartyType,
    pub aliases: Vec<String>,
    pub related_entities: Vec<String>,
    pub ssn_last4: Option<String>,
    pub date_of_birth: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartyType {
    Client,
    OpposingParty,
    Witness,
    Attorney,
    Corporation,
    Government,
    ThirdParty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: String,
    pub conflicting_matter_id: String,
    pub conflicting_matter_name: String,
    pub conflicting_party: String,
    pub relationship: String,
    pub detected_at: DateTime<Utc>,
    pub requires_waiver: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    DirectAdverse,          // Directly opposing current client
    PositionalConflict,     // Same issue, opposite sides
    FormerClient,           // Former client, adverse interest
    FamilyRelationship,     // Family member conflict
    BusinessRelationship,   // Business partner/affiliate
    ConcurrentRepresentation, // Representing both parties
    PersonalInterest,       // Attorney has personal interest
    JointRepresentation,    // Co-clients with adverse interests
    GovernmentEmployee,     // Government ethics conflict
    CorporateAffiliate,     // Corporate family conflict
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    Critical,   // Cannot proceed without waiver
    High,       // Strong conflict, needs review
    Medium,     // Potential conflict, manageable
    Low,        // Minor concern, document only
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStatus {
    Pending,
    Cleared,
    ConflictDetected,
    WaiverRequired,
    WaiverObtained,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub resolution_type: ResolutionType,
    pub resolved_at: DateTime<Utc>,
    pub resolved_by: String,
    pub notes: String,
    pub waiver_obtained: bool,
    pub waiver_document_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    NoConflict,
    WaiverObtained,
    ChineseWall,
    WithdrawRepresentation,
    ClientConsent,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConflict {
    pub event_id: String,
    pub event_title: String,
    pub event_time: DateTime<Utc>,
    pub conflicting_event_id: String,
    pub conflicting_event_title: String,
    pub conflicting_time: DateTime<Utc>,
}

pub struct ConflictCheckingService {
    db: SqlitePool,
}

impl ConflictCheckingService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Perform comprehensive conflict check
    pub async fn perform_conflict_check(
        &self,
        parties: Vec<ConflictParty>,
        matter_id: Option<String>,
        checked_by: &str,
    ) -> Result<ConflictCheck> {
        info!("Performing conflict check for {} parties", parties.len());

        let mut conflicts = Vec::new();

        // Check each party
        for party in &parties {
            // Name-based conflicts
            conflicts.extend(self.check_name_conflicts(party).await?);

            // Entity relationship conflicts
            conflicts.extend(self.check_entity_conflicts(party).await?);

            // Former client conflicts
            if party.party_type == PartyType::OpposingParty {
                conflicts.extend(self.check_former_client_conflicts(party).await?);
            }

            // Family relationship conflicts
            conflicts.extend(self.check_family_conflicts(party).await?);

            // Corporate affiliate conflicts
            conflicts.extend(self.check_corporate_conflicts(party).await?);
        }

        // Check for concurrent representation
        conflicts.extend(self.check_concurrent_representation(&parties).await?);

        // Remove duplicates
        conflicts = self.deduplicate_conflicts(conflicts);

        // Sort by severity
        conflicts.sort_by(|a, b| b.severity.cmp(&a.severity));

        // Determine overall status
        let status = if conflicts.is_empty() {
            ConflictStatus::Cleared
        } else if conflicts.iter().any(|c| c.severity == ConflictSeverity::Critical) {
            ConflictStatus::WaiverRequired
        } else {
            ConflictStatus::ConflictDetected
        };

        let check = ConflictCheck {
            id: uuid::Uuid::new_v4().to_string(),
            matter_id,
            checked_at: Utc::now(),
            checked_by: checked_by.to_string(),
            parties,
            conflicts_found: conflicts,
            status,
            resolution: None,
        };

        // Save conflict check to database
        self.save_conflict_check(&check).await?;

        info!(
            "Conflict check complete: {} conflicts found",
            check.conflicts_found.len()
        );

        Ok(check)
    }

    /// Check for name-based conflicts
    async fn check_name_conflicts(&self, party: &ConflictParty) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        // Normalize name for comparison
        let normalized_name = self.normalize_name(&party.name);

        // Search in existing matters
        let records = sqlx::query!(
            r#"
            SELECT
                m.id as matter_id,
                m.title as matter_title,
                cp.name as party_name,
                cp.party_type,
                m.status
            FROM case_participants cp
            JOIN matters m ON m.id = cp.matter_id
            WHERE
                LOWER(cp.name) LIKE '%' || ? || '%'
                AND m.status IN ('active', 'pending')
            "#,
            normalized_name
        )
        .fetch_all(&self.db)
        .await?;

        for record in records {
            // Determine conflict type based on party types
            let conflict_type = if record.party_type == "client" && party.party_type == PartyType::OpposingParty {
                ConflictType::DirectAdverse
            } else if record.party_type == "opposing_party" && party.party_type == PartyType::Client {
                ConflictType::DirectAdverse
            } else {
                ConflictType::PositionalConflict
            };

            let severity = if matches!(conflict_type, ConflictType::DirectAdverse) {
                ConflictSeverity::Critical
            } else {
                ConflictSeverity::High
            };

            conflicts.push(Conflict {
                id: uuid::Uuid::new_v4().to_string(),
                conflict_type,
                severity,
                description: format!(
                    "Party '{}' appears in another active matter as {}",
                    party.name, record.party_type
                ),
                conflicting_matter_id: record.matter_id,
                conflicting_matter_name: record.matter_title,
                conflicting_party: record.party_name,
                relationship: "Same party in different matters".to_string(),
                detected_at: Utc::now(),
                requires_waiver: severity == ConflictSeverity::Critical,
            });
        }

        // Check aliases
        for alias in &party.aliases {
            let normalized_alias = self.normalize_name(alias);
            let alias_records = sqlx::query!(
                r#"
                SELECT
                    m.id as matter_id,
                    m.title as matter_title,
                    cp.name as party_name,
                    cp.party_type
                FROM case_participants cp
                JOIN matters m ON m.id = cp.matter_id
                WHERE
                    LOWER(cp.name) LIKE '%' || ? || '%'
                    AND m.status IN ('active', 'pending')
                "#,
                normalized_alias
            )
            .fetch_all(&self.db)
            .await?;

            for record in alias_records {
                conflicts.push(Conflict {
                    id: uuid::Uuid::new_v4().to_string(),
                    conflict_type: ConflictType::PositionalConflict,
                    severity: ConflictSeverity::High,
                    description: format!(
                        "Party alias '{}' matches existing party '{}'",
                        alias, record.party_name
                    ),
                    conflicting_matter_id: record.matter_id,
                    conflicting_matter_name: record.matter_title,
                    conflicting_party: record.party_name,
                    relationship: "Alias match".to_string(),
                    detected_at: Utc::now(),
                    requires_waiver: true,
                });
            }
        }

        Ok(conflicts)
    }

    /// Check for entity relationship conflicts
    async fn check_entity_conflicts(&self, party: &ConflictParty) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        for related_entity in &party.related_entities {
            let normalized_entity = self.normalize_name(related_entity);

            let records = sqlx::query!(
                r#"
                SELECT
                    m.id as matter_id,
                    m.title as matter_title,
                    cp.name as party_name
                FROM case_participants cp
                JOIN matters m ON m.id = cp.matter_id
                WHERE
                    LOWER(cp.name) LIKE '%' || ? || '%'
                    AND m.status IN ('active', 'pending')
                "#,
                normalized_entity
            )
            .fetch_all(&self.db)
            .await?;

            for record in records {
                conflicts.push(Conflict {
                    id: uuid::Uuid::new_v4().to_string(),
                    conflict_type: ConflictType::BusinessRelationship,
                    severity: ConflictSeverity::Medium,
                    description: format!(
                        "Related entity '{}' appears in another active matter",
                        related_entity
                    ),
                    conflicting_matter_id: record.matter_id,
                    conflicting_matter_name: record.matter_title,
                    conflicting_party: record.party_name,
                    relationship: "Related entity".to_string(),
                    detected_at: Utc::now(),
                    requires_waiver: false,
                });
            }
        }

        Ok(conflicts)
    }

    /// Check for former client conflicts
    async fn check_former_client_conflicts(&self, party: &ConflictParty) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        let normalized_name = self.normalize_name(&party.name);

        // Check if this party was ever a former client
        let records = sqlx::query!(
            r#"
            SELECT
                m.id as matter_id,
                m.title as matter_title,
                m.status,
                m.closed_at
            FROM matters m
            JOIN clients c ON c.id = m.client_id
            WHERE
                LOWER(c.name) LIKE '%' || ? || '%'
                AND m.status = 'closed'
            "#,
            normalized_name
        )
        .fetch_all(&self.db)
        .await?;

        for record in records {
            conflicts.push(Conflict {
                id: uuid::Uuid::new_v4().to_string(),
                conflict_type: ConflictType::FormerClient,
                severity: ConflictSeverity::High,
                description: format!(
                    "Opposing party '{}' is a former client in matter: {}",
                    party.name, record.matter_title
                ),
                conflicting_matter_id: record.matter_id,
                conflicting_matter_name: record.matter_title,
                conflicting_party: party.name.clone(),
                relationship: "Former client".to_string(),
                detected_at: Utc::now(),
                requires_waiver: true,
            });
        }

        Ok(conflicts)
    }

    /// Check for family relationship conflicts
    async fn check_family_conflicts(&self, party: &ConflictParty) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        // Extract last name
        if let Some(last_name) = party.name.split_whitespace().last() {
            // Check for same last name in different matters
            let records = sqlx::query!(
                r#"
                SELECT
                    m.id as matter_id,
                    m.title as matter_title,
                    cp.name as party_name,
                    cp.party_type
                FROM case_participants cp
                JOIN matters m ON m.id = cp.matter_id
                WHERE
                    LOWER(cp.name) LIKE '%' || ? || '%'
                    AND m.status IN ('active', 'pending')
                LIMIT 10
                "#,
                last_name.to_lowercase()
            )
            .fetch_all(&self.db)
            .await?;

            if records.len() > 1 {
                for record in records {
                    conflicts.push(Conflict {
                        id: uuid::Uuid::new_v4().to_string(),
                        conflict_type: ConflictType::FamilyRelationship,
                        severity: ConflictSeverity::Low,
                        description: format!(
                            "Possible family relationship: '{}' and '{}'",
                            party.name, record.party_name
                        ),
                        conflicting_matter_id: record.matter_id,
                        conflicting_matter_name: record.matter_title,
                        conflicting_party: record.party_name,
                        relationship: "Possible family member".to_string(),
                        detected_at: Utc::now(),
                        requires_waiver: false,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Check for corporate affiliate conflicts
    async fn check_corporate_conflicts(&self, party: &ConflictParty) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        if party.party_type == PartyType::Corporation {
            // Extract base company name (remove Inc, LLC, etc.)
            let base_name = self.extract_base_company_name(&party.name);

            let records = sqlx::query!(
                r#"
                SELECT
                    m.id as matter_id,
                    m.title as matter_title,
                    cp.name as party_name
                FROM case_participants cp
                JOIN matters m ON m.id = cp.matter_id
                WHERE
                    LOWER(cp.name) LIKE '%' || ? || '%'
                    AND m.status IN ('active', 'pending')
                "#,
                base_name
            )
            .fetch_all(&self.db)
            .await?;

            for record in records {
                if record.party_name != party.name {
                    conflicts.push(Conflict {
                        id: uuid::Uuid::new_v4().to_string(),
                        conflict_type: ConflictType::CorporateAffiliate,
                        severity: ConflictSeverity::Medium,
                        description: format!(
                            "Possible corporate affiliate: '{}' and '{}'",
                            party.name, record.party_name
                        ),
                        conflicting_matter_id: record.matter_id,
                        conflicting_matter_name: record.matter_title,
                        conflicting_party: record.party_name,
                        relationship: "Corporate affiliate".to_string(),
                        detected_at: Utc::now(),
                        requires_waiver: false,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Check for concurrent representation conflicts
    async fn check_concurrent_representation(&self, parties: &[ConflictParty]) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        // Count clients and opposing parties
        let client_count = parties.iter().filter(|p| p.party_type == PartyType::Client).count();
        let opposing_count = parties.iter().filter(|p| p.party_type == PartyType::OpposingParty).count();

        if client_count > 1 {
            // Multiple clients - check for adverse interests
            conflicts.push(Conflict {
                id: uuid::Uuid::new_v4().to_string(),
                conflict_type: ConflictType::JointRepresentation,
                severity: ConflictSeverity::High,
                description: "Multiple clients may have conflicting interests".to_string(),
                conflicting_matter_id: "new_matter".to_string(),
                conflicting_matter_name: "Current matter".to_string(),
                conflicting_party: "Multiple clients".to_string(),
                relationship: "Joint representation".to_string(),
                detected_at: Utc::now(),
                requires_waiver: true,
            });
        }

        Ok(conflicts)
    }

    /// Normalize name for comparison
    fn normalize_name(&self, name: &str) -> String {
        name.to_lowercase()
            .replace(".", "")
            .replace(",", "")
            .trim()
            .to_string()
    }

    /// Extract base company name
    fn extract_base_company_name(&self, name: &str) -> String {
        let suffixes = ["inc", "llc", "corp", "ltd", "co", "company", "corporation"];
        let mut base = name.to_lowercase();

        for suffix in &suffixes {
            base = base.replace(suffix, "");
        }

        base.trim().to_string()
    }

    /// Deduplicate conflicts
    fn deduplicate_conflicts(&self, conflicts: Vec<Conflict>) -> Vec<Conflict> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for conflict in conflicts {
            let key = format!(
                "{}:{}:{}",
                conflict.conflict_type as u8,
                conflict.conflicting_matter_id,
                conflict.conflicting_party
            );

            if seen.insert(key) {
                unique.push(conflict);
            }
        }

        unique
    }

    /// Save conflict check to database
    async fn save_conflict_check(&self, check: &ConflictCheck) -> Result<()> {
        let parties_json = serde_json::to_string(&check.parties)?;
        let conflicts_json = serde_json::to_string(&check.conflicts_found)?;
        let status_json = serde_json::to_string(&check.status)?;

        sqlx::query!(
            r#"
            INSERT INTO conflict_checks (
                id, matter_id, checked_at, checked_by, parties,
                conflicts_found, status, resolution
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            check.id,
            check.matter_id,
            check.checked_at,
            check.checked_by,
            parties_json,
            conflicts_json,
            status_json,
            check.resolution
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Check calendar conflicts
    pub async fn check_calendar_conflicts(
        &self,
        event_time: DateTime<Utc>,
        duration_minutes: i64,
    ) -> Result<Vec<CalendarConflict>> {
        let end_time = event_time + chrono::Duration::minutes(duration_minutes);

        let records = sqlx::query!(
            r#"
            SELECT id, title, start_time, end_time
            FROM calendar_events
            WHERE
                (start_time <= ? AND end_time >= ?)
                OR (start_time >= ? AND start_time < ?)
            "#,
            end_time,
            event_time,
            event_time,
            end_time
        )
        .fetch_all(&self.db)
        .await?;

        let conflicts = records.into_iter().map(|r| CalendarConflict {
            event_id: "new_event".to_string(),
            event_title: "New Event".to_string(),
            event_time,
            conflicting_event_id: r.id,
            conflicting_event_title: r.title,
            conflicting_time: DateTime::parse_from_rfc3339(&r.start_time).ok().map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(Utc::now),
        }).collect();

        Ok(conflicts)
    }
}
