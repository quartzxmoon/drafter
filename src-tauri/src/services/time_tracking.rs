// Time Tracking Service - Automatic time tracking and billing integration
// Supports timer-based tracking, manual entry, automatic detection, and billing rate management

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeEntryStatus {
    Running,
    Paused,
    Stopped,
    Submitted,
    Approved,
    Billed,
    Written_off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeEntryType {
    Timer,           // Started from timer
    Manual,          // Manually entered
    Automatic,       // Auto-detected from activity
    Imported,        // Imported from external system
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillableStatus {
    Billable,
    NonBillable,
    NoCharge,        // Billable but not charged to client
    WriteOff,        // Written off after the fact
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityType {
    Research,
    Drafting,
    Review,
    Email,
    Phone,
    Meeting,
    CourtAppearance,
    Travel,
    ClientConsultation,
    CaseManagement,
    Discovery,
    Negotiation,
    Administrative,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub matter_id: String,
    pub attorney_id: String,
    pub attorney_name: String,

    // Time tracking
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i64>,
    pub billable_minutes: Option<i64>,

    // Entry details
    pub activity_type: ActivityType,
    pub description: String,
    pub notes: Option<String>,

    // Status
    pub status: TimeEntryStatus,
    pub entry_type: TimeEntryType,
    pub billable_status: BillableStatus,

    // Billing
    pub hourly_rate: Option<f64>,
    pub amount: Option<f64>,
    pub discount_percent: Option<f64>,
    pub discount_amount: Option<f64>,
    pub final_amount: Option<f64>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
    pub billed_at: Option<DateTime<Utc>>,
    pub invoice_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub id: String,
    pub time_entry_id: String,
    pub matter_id: String,
    pub attorney_id: String,
    pub started_at: DateTime<Utc>,
    pub paused_at: Option<DateTime<Utc>>,
    pub total_pause_duration_minutes: i64,
    pub is_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRate {
    pub id: String,
    pub attorney_id: String,
    pub activity_type: Option<ActivityType>,
    pub matter_id: Option<String>,
    pub client_id: Option<String>,
    pub rate_type: RateType,
    pub hourly_rate: f64,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RateType {
    Standard,        // Standard hourly rate
    Discounted,      // Discounted rate for specific client/matter
    Flat,            // Flat fee for activity type
    Contingency,     // Contingency fee arrangement
    Pro_bono,        // Pro bono work (no charge)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeReport {
    pub report_type: TimeReportType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub filters: TimeReportFilters,

    // Summary statistics
    pub total_hours: f64,
    pub billable_hours: f64,
    pub non_billable_hours: f64,
    pub total_amount: f64,

    // Breakdown
    pub by_attorney: Vec<AttorneyTimeSummary>,
    pub by_matter: Vec<MatterTimeSummary>,
    pub by_activity: Vec<ActivityTimeSummary>,
    pub by_client: Vec<ClientTimeSummary>,

    pub entries: Vec<TimeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeReportType {
    Summary,
    Detailed,
    ByAttorney,
    ByMatter,
    ByClient,
    ByActivity,
    Unbilled,
    Realization,      // Billed vs collected analysis
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeReportFilters {
    pub attorney_ids: Option<Vec<String>>,
    pub matter_ids: Option<Vec<String>>,
    pub client_ids: Option<Vec<String>>,
    pub activity_types: Option<Vec<ActivityType>>,
    pub billable_status: Option<Vec<BillableStatus>>,
    pub status: Option<Vec<TimeEntryStatus>>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttorneyTimeSummary {
    pub attorney_id: String,
    pub attorney_name: String,
    pub total_hours: f64,
    pub billable_hours: f64,
    pub total_amount: f64,
    pub entries_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatterTimeSummary {
    pub matter_id: String,
    pub matter_name: String,
    pub client_name: String,
    pub total_hours: f64,
    pub billable_hours: f64,
    pub total_amount: f64,
    pub entries_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTimeSummary {
    pub activity_type: ActivityType,
    pub total_hours: f64,
    pub billable_hours: f64,
    pub total_amount: f64,
    pub entries_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTimeSummary {
    pub client_id: String,
    pub client_name: String,
    pub total_hours: f64,
    pub billable_hours: f64,
    pub total_amount: f64,
    pub matters_count: u32,
    pub entries_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticTimeDetection {
    pub enabled: bool,
    pub detect_document_editing: bool,
    pub detect_email_activity: bool,
    pub detect_research_activity: bool,
    pub min_activity_duration_minutes: i64,
    pub idle_timeout_minutes: i64,
}

pub struct TimeTrackingService {
    db: SqlitePool,
    active_timers: HashMap<String, Timer>, // attorney_id -> Timer
}

impl TimeTrackingService {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            active_timers: HashMap::new(),
        }
    }

    // ============= Timer Management =============

    /// Start a new timer for time tracking
    pub async fn start_timer(
        &mut self,
        matter_id: &str,
        attorney_id: &str,
        activity_type: ActivityType,
        description: &str,
    ) -> Result<Timer> {
        // Check if attorney already has a running timer
        if let Some(existing) = self.active_timers.get(attorney_id) {
            if existing.is_running {
                return Err(anyhow::anyhow!(
                    "Attorney {} already has a running timer for matter {}",
                    attorney_id,
                    existing.matter_id
                ));
            }
        }

        let timer_id = Uuid::new_v4().to_string();
        let entry_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create time entry
        let time_entry = TimeEntry {
            id: entry_id.clone(),
            matter_id: matter_id.to_string(),
            attorney_id: attorney_id.to_string(),
            attorney_name: self.get_attorney_name(attorney_id).await?,
            start_time: now,
            end_time: None,
            duration_minutes: None,
            billable_minutes: None,
            activity_type,
            description: description.to_string(),
            notes: None,
            status: TimeEntryStatus::Running,
            entry_type: TimeEntryType::Timer,
            billable_status: BillableStatus::Billable,
            hourly_rate: self.get_billing_rate(attorney_id, matter_id, &activity_type).await?,
            amount: None,
            discount_percent: None,
            discount_amount: None,
            final_amount: None,
            created_at: now,
            updated_at: now,
            submitted_at: None,
            approved_at: None,
            approved_by: None,
            billed_at: None,
            invoice_id: None,
        };

        // Save time entry
        self.save_time_entry(&time_entry).await?;

        // Create timer
        let timer = Timer {
            id: timer_id,
            time_entry_id: entry_id,
            matter_id: matter_id.to_string(),
            attorney_id: attorney_id.to_string(),
            started_at: now,
            paused_at: None,
            total_pause_duration_minutes: 0,
            is_running: true,
        };

        // Save timer to database
        sqlx::query!(
            r#"
            INSERT INTO timers (id, time_entry_id, matter_id, attorney_id, started_at,
                                total_pause_duration_minutes, is_running)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            timer.id,
            timer.time_entry_id,
            timer.matter_id,
            timer.attorney_id,
            timer.started_at,
            timer.total_pause_duration_minutes,
            timer.is_running
        )
        .execute(&self.db)
        .await
        .context("Failed to save timer")?;

        // Store in active timers
        self.active_timers.insert(attorney_id.to_string(), timer.clone());

        Ok(timer)
    }

    /// Pause a running timer
    pub async fn pause_timer(&mut self, attorney_id: &str) -> Result<Timer> {
        let timer = self.active_timers.get_mut(attorney_id)
            .ok_or_else(|| anyhow::anyhow!("No active timer for attorney {}", attorney_id))?;

        if !timer.is_running {
            return Err(anyhow::anyhow!("Timer is not running"));
        }

        let now = Utc::now();
        timer.paused_at = Some(now);
        timer.is_running = false;

        // Update in database
        sqlx::query!(
            r#"
            UPDATE timers
            SET paused_at = ?, is_running = ?
            WHERE id = ?
            "#,
            now,
            false,
            timer.id
        )
        .execute(&self.db)
        .await
        .context("Failed to pause timer")?;

        Ok(timer.clone())
    }

    /// Resume a paused timer
    pub async fn resume_timer(&mut self, attorney_id: &str) -> Result<Timer> {
        let timer = self.active_timers.get_mut(attorney_id)
            .ok_or_else(|| anyhow::anyhow!("No active timer for attorney {}", attorney_id))?;

        if timer.is_running {
            return Err(anyhow::anyhow!("Timer is already running"));
        }

        let now = Utc::now();

        // Calculate pause duration
        if let Some(paused_at) = timer.paused_at {
            let pause_duration = now.signed_duration_since(paused_at);
            timer.total_pause_duration_minutes += pause_duration.num_minutes();
        }

        timer.paused_at = None;
        timer.is_running = true;

        // Update in database
        sqlx::query!(
            r#"
            UPDATE timers
            SET paused_at = NULL, is_running = ?, total_pause_duration_minutes = ?
            WHERE id = ?
            "#,
            true,
            timer.total_pause_duration_minutes,
            timer.id
        )
        .execute(&self.db)
        .await
        .context("Failed to resume timer")?;

        Ok(timer.clone())
    }

    /// Stop a timer and finalize the time entry
    pub async fn stop_timer(
        &mut self,
        attorney_id: &str,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<TimeEntry> {
        let timer = self.active_timers.remove(attorney_id)
            .ok_or_else(|| anyhow::anyhow!("No active timer for attorney {}", attorney_id))?;

        let now = Utc::now();

        // Calculate total duration
        let total_duration = now.signed_duration_since(timer.started_at);
        let duration_minutes = total_duration.num_minutes() - timer.total_pause_duration_minutes;

        // Get the time entry
        let mut time_entry = self.get_time_entry(&timer.time_entry_id).await?;

        // Update time entry
        time_entry.end_time = Some(now);
        time_entry.duration_minutes = Some(duration_minutes);
        time_entry.billable_minutes = Some(duration_minutes); // Default to full duration
        time_entry.status = TimeEntryStatus::Stopped;

        if let Some(desc) = description {
            time_entry.description = desc;
        }
        time_entry.notes = notes;

        // Calculate amount
        if let Some(rate) = time_entry.hourly_rate {
            let hours = duration_minutes as f64 / 60.0;
            time_entry.amount = Some(rate * hours);
            time_entry.final_amount = Some(rate * hours);
        }

        time_entry.updated_at = now;

        // Save updated time entry
        self.save_time_entry(&time_entry).await?;

        // Delete timer
        sqlx::query!(
            r#"
            DELETE FROM timers WHERE id = ?
            "#,
            timer.id
        )
        .execute(&self.db)
        .await
        .context("Failed to delete timer")?;

        Ok(time_entry)
    }

    /// Get active timer for attorney
    pub fn get_active_timer(&self, attorney_id: &str) -> Option<&Timer> {
        self.active_timers.get(attorney_id)
    }

    // ============= Time Entry Management =============

    /// Create a manual time entry
    pub async fn create_manual_entry(
        &self,
        matter_id: &str,
        attorney_id: &str,
        activity_type: ActivityType,
        description: &str,
        start_time: DateTime<Utc>,
        duration_minutes: i64,
        billable_status: BillableStatus,
        notes: Option<String>,
    ) -> Result<TimeEntry> {
        let entry_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let hourly_rate = self.get_billing_rate(attorney_id, matter_id, &activity_type).await?;

        let amount = if let Some(rate) = hourly_rate {
            let hours = duration_minutes as f64 / 60.0;
            Some(rate * hours)
        } else {
            None
        };

        let time_entry = TimeEntry {
            id: entry_id,
            matter_id: matter_id.to_string(),
            attorney_id: attorney_id.to_string(),
            attorney_name: self.get_attorney_name(attorney_id).await?,
            start_time,
            end_time: Some(start_time + Duration::minutes(duration_minutes)),
            duration_minutes: Some(duration_minutes),
            billable_minutes: Some(duration_minutes),
            activity_type,
            description: description.to_string(),
            notes,
            status: TimeEntryStatus::Stopped,
            entry_type: TimeEntryType::Manual,
            billable_status,
            hourly_rate,
            amount,
            discount_percent: None,
            discount_amount: None,
            final_amount: amount,
            created_at: now,
            updated_at: now,
            submitted_at: None,
            approved_at: None,
            approved_by: None,
            billed_at: None,
            invoice_id: None,
        };

        self.save_time_entry(&time_entry).await?;

        Ok(time_entry)
    }

    /// Update an existing time entry
    pub async fn update_time_entry(
        &self,
        entry_id: &str,
        description: Option<String>,
        duration_minutes: Option<i64>,
        billable_minutes: Option<i64>,
        billable_status: Option<BillableStatus>,
        notes: Option<String>,
    ) -> Result<TimeEntry> {
        let mut entry = self.get_time_entry(entry_id).await?;

        // Can only update entries that haven't been billed
        if entry.status == TimeEntryStatus::Billed {
            return Err(anyhow::anyhow!("Cannot update billed time entry"));
        }

        if let Some(desc) = description {
            entry.description = desc;
        }

        if let Some(duration) = duration_minutes {
            entry.duration_minutes = Some(duration);

            // Recalculate amount
            if let Some(rate) = entry.hourly_rate {
                let hours = duration as f64 / 60.0;
                entry.amount = Some(rate * hours);
                entry.final_amount = Some(rate * hours);
            }
        }

        if let Some(billable) = billable_minutes {
            entry.billable_minutes = Some(billable);
        }

        if let Some(status) = billable_status {
            entry.billable_status = status;
        }

        if notes.is_some() {
            entry.notes = notes;
        }

        entry.updated_at = Utc::now();

        self.save_time_entry(&entry).await?;

        Ok(entry)
    }

    /// Delete a time entry
    pub async fn delete_time_entry(&self, entry_id: &str) -> Result<()> {
        let entry = self.get_time_entry(entry_id).await?;

        // Can only delete entries that haven't been billed
        if entry.status == TimeEntryStatus::Billed {
            return Err(anyhow::anyhow!("Cannot delete billed time entry"));
        }

        sqlx::query!(
            r#"
            DELETE FROM time_entries WHERE id = ?
            "#,
            entry_id
        )
        .execute(&self.db)
        .await
        .context("Failed to delete time entry")?;

        Ok(())
    }

    /// Submit time entries for approval
    pub async fn submit_entries(&self, entry_ids: Vec<String>) -> Result<Vec<TimeEntry>> {
        let now = Utc::now();
        let mut updated_entries = Vec::new();

        for entry_id in entry_ids {
            let mut entry = self.get_time_entry(&entry_id).await?;

            if entry.status != TimeEntryStatus::Stopped {
                continue; // Skip non-stopped entries
            }

            entry.status = TimeEntryStatus::Submitted;
            entry.submitted_at = Some(now);
            entry.updated_at = now;

            self.save_time_entry(&entry).await?;
            updated_entries.push(entry);
        }

        Ok(updated_entries)
    }

    /// Approve time entries
    pub async fn approve_entries(
        &self,
        entry_ids: Vec<String>,
        approved_by: &str,
    ) -> Result<Vec<TimeEntry>> {
        let now = Utc::now();
        let mut updated_entries = Vec::new();

        for entry_id in entry_ids {
            let mut entry = self.get_time_entry(&entry_id).await?;

            if entry.status != TimeEntryStatus::Submitted {
                continue; // Skip non-submitted entries
            }

            entry.status = TimeEntryStatus::Approved;
            entry.approved_at = Some(now);
            entry.approved_by = Some(approved_by.to_string());
            entry.updated_at = now;

            self.save_time_entry(&entry).await?;
            updated_entries.push(entry);
        }

        Ok(updated_entries)
    }

    // ============= Billing Rate Management =============

    /// Get billing rate for attorney/matter/activity
    async fn get_billing_rate(
        &self,
        attorney_id: &str,
        matter_id: &str,
        activity_type: &ActivityType,
    ) -> Result<Option<f64>> {
        // Try to find most specific rate first

        // 1. Matter-specific + activity-specific rate
        if let Some(rate) = self.find_rate(Some(attorney_id), Some(matter_id), Some(activity_type), None).await? {
            return Ok(Some(rate.hourly_rate));
        }

        // 2. Matter-specific rate
        if let Some(rate) = self.find_rate(Some(attorney_id), Some(matter_id), None, None).await? {
            return Ok(Some(rate.hourly_rate));
        }

        // 3. Client-specific rate
        let client_id = self.get_client_id_for_matter(matter_id).await?;
        if let Some(client) = client_id {
            if let Some(rate) = self.find_rate(Some(attorney_id), None, None, Some(&client)).await? {
                return Ok(Some(rate.hourly_rate));
            }
        }

        // 4. Activity-specific default rate
        if let Some(rate) = self.find_rate(Some(attorney_id), None, Some(activity_type), None).await? {
            return Ok(Some(rate.hourly_rate));
        }

        // 5. Attorney default rate
        if let Some(rate) = self.find_rate(Some(attorney_id), None, None, None).await? {
            return Ok(Some(rate.hourly_rate));
        }

        Ok(None)
    }

    async fn find_rate(
        &self,
        attorney_id: Option<&str>,
        matter_id: Option<&str>,
        activity_type: Option<&ActivityType>,
        client_id: Option<&str>,
    ) -> Result<Option<BillingRate>> {
        // This is a simplified version - real implementation would use complex SQL query
        let activity_str = activity_type.map(|a| format!("{:?}", a));

        let result = sqlx::query_as!(
            BillingRate,
            r#"
            SELECT id, attorney_id, activity_type as "activity_type: _", matter_id, client_id,
                   rate_type as "rate_type: _", hourly_rate, effective_from, effective_to, is_default
            FROM billing_rates
            WHERE attorney_id = ?
              AND (matter_id = ? OR matter_id IS NULL)
              AND (activity_type = ? OR activity_type IS NULL)
              AND (client_id = ? OR client_id IS NULL)
              AND effective_from <= datetime('now')
              AND (effective_to IS NULL OR effective_to >= datetime('now'))
            ORDER BY
              CASE WHEN matter_id IS NOT NULL THEN 1 ELSE 2 END,
              CASE WHEN activity_type IS NOT NULL THEN 1 ELSE 2 END,
              CASE WHEN client_id IS NOT NULL THEN 1 ELSE 2 END
            LIMIT 1
            "#,
            attorney_id,
            matter_id,
            activity_str,
            client_id
        )
        .fetch_optional(&self.db)
        .await
        .context("Failed to query billing rate")?;

        Ok(result)
    }

    // ============= Reporting =============

    /// Generate time report
    pub async fn generate_report(
        &self,
        report_type: TimeReportType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        filters: TimeReportFilters,
    ) -> Result<TimeReport> {
        // Get all time entries in date range
        let entries = self.query_time_entries(start_date, end_date, &filters).await?;

        // Calculate totals
        let total_hours: f64 = entries.iter()
            .filter_map(|e| e.duration_minutes)
            .sum::<i64>() as f64 / 60.0;

        let billable_hours: f64 = entries.iter()
            .filter(|e| e.billable_status == BillableStatus::Billable)
            .filter_map(|e| e.billable_minutes)
            .sum::<i64>() as f64 / 60.0;

        let non_billable_hours = total_hours - billable_hours;

        let total_amount: f64 = entries.iter()
            .filter_map(|e| e.final_amount)
            .sum();

        // Generate breakdowns
        let by_attorney = self.generate_attorney_summary(&entries);
        let by_matter = self.generate_matter_summary(&entries).await?;
        let by_activity = self.generate_activity_summary(&entries);
        let by_client = self.generate_client_summary(&entries).await?;

        Ok(TimeReport {
            report_type,
            start_date,
            end_date,
            filters,
            total_hours,
            billable_hours,
            non_billable_hours,
            total_amount,
            by_attorney,
            by_matter,
            by_activity,
            by_client,
            entries,
        })
    }

    fn generate_attorney_summary(&self, entries: &[TimeEntry]) -> Vec<AttorneyTimeSummary> {
        let mut summaries: HashMap<String, AttorneyTimeSummary> = HashMap::new();

        for entry in entries {
            let summary = summaries.entry(entry.attorney_id.clone()).or_insert(AttorneyTimeSummary {
                attorney_id: entry.attorney_id.clone(),
                attorney_name: entry.attorney_name.clone(),
                total_hours: 0.0,
                billable_hours: 0.0,
                total_amount: 0.0,
                entries_count: 0,
            });

            if let Some(duration) = entry.duration_minutes {
                summary.total_hours += duration as f64 / 60.0;
            }

            if entry.billable_status == BillableStatus::Billable {
                if let Some(billable) = entry.billable_minutes {
                    summary.billable_hours += billable as f64 / 60.0;
                }
            }

            if let Some(amount) = entry.final_amount {
                summary.total_amount += amount;
            }

            summary.entries_count += 1;
        }

        summaries.into_values().collect()
    }

    async fn generate_matter_summary(&self, entries: &[TimeEntry]) -> Result<Vec<MatterTimeSummary>> {
        let mut summaries: HashMap<String, MatterTimeSummary> = HashMap::new();

        for entry in entries {
            if !summaries.contains_key(&entry.matter_id) {
                let matter_name = self.get_matter_name(&entry.matter_id).await?;
                let client_name = self.get_client_name_for_matter(&entry.matter_id).await?;

                summaries.insert(entry.matter_id.clone(), MatterTimeSummary {
                    matter_id: entry.matter_id.clone(),
                    matter_name,
                    client_name,
                    total_hours: 0.0,
                    billable_hours: 0.0,
                    total_amount: 0.0,
                    entries_count: 0,
                });
            }

            let summary = summaries.get_mut(&entry.matter_id).unwrap();

            if let Some(duration) = entry.duration_minutes {
                summary.total_hours += duration as f64 / 60.0;
            }

            if entry.billable_status == BillableStatus::Billable {
                if let Some(billable) = entry.billable_minutes {
                    summary.billable_hours += billable as f64 / 60.0;
                }
            }

            if let Some(amount) = entry.final_amount {
                summary.total_amount += amount;
            }

            summary.entries_count += 1;
        }

        Ok(summaries.into_values().collect())
    }

    fn generate_activity_summary(&self, entries: &[TimeEntry]) -> Vec<ActivityTimeSummary> {
        let mut summaries: HashMap<String, ActivityTimeSummary> = HashMap::new();

        for entry in entries {
            let key = format!("{:?}", entry.activity_type);
            let summary = summaries.entry(key).or_insert(ActivityTimeSummary {
                activity_type: entry.activity_type.clone(),
                total_hours: 0.0,
                billable_hours: 0.0,
                total_amount: 0.0,
                entries_count: 0,
            });

            if let Some(duration) = entry.duration_minutes {
                summary.total_hours += duration as f64 / 60.0;
            }

            if entry.billable_status == BillableStatus::Billable {
                if let Some(billable) = entry.billable_minutes {
                    summary.billable_hours += billable as f64 / 60.0;
                }
            }

            if let Some(amount) = entry.final_amount {
                summary.total_amount += amount;
            }

            summary.entries_count += 1;
        }

        summaries.into_values().collect()
    }

    async fn generate_client_summary(&self, entries: &[TimeEntry]) -> Result<Vec<ClientTimeSummary>> {
        // Implementation would group by client
        Ok(Vec::new())
    }

    // ============= Helper Methods =============

    async fn save_time_entry(&self, entry: &TimeEntry) -> Result<()> {
        let activity_type_str = format!("{:?}", entry.activity_type);
        let status_str = format!("{:?}", entry.status);
        let entry_type_str = format!("{:?}", entry.entry_type);
        let billable_status_str = format!("{:?}", entry.billable_status);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO time_entries
            (id, matter_id, attorney_id, attorney_name, start_time, end_time, duration_minutes,
             billable_minutes, activity_type, description, notes, status, entry_type,
             billable_status, hourly_rate, amount, discount_percent, discount_amount,
             final_amount, created_at, updated_at, submitted_at, approved_at, approved_by,
             billed_at, invoice_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            entry.id,
            entry.matter_id,
            entry.attorney_id,
            entry.attorney_name,
            entry.start_time,
            entry.end_time,
            entry.duration_minutes,
            entry.billable_minutes,
            activity_type_str,
            entry.description,
            entry.notes,
            status_str,
            entry_type_str,
            billable_status_str,
            entry.hourly_rate,
            entry.amount,
            entry.discount_percent,
            entry.discount_amount,
            entry.final_amount,
            entry.created_at,
            entry.updated_at,
            entry.submitted_at,
            entry.approved_at,
            entry.approved_by,
            entry.billed_at,
            entry.invoice_id
        )
        .execute(&self.db)
        .await
        .context("Failed to save time entry")?;

        Ok(())
    }

    async fn get_time_entry(&self, entry_id: &str) -> Result<TimeEntry> {
        let result = sqlx::query_as!(
            TimeEntry,
            r#"
            SELECT id, matter_id, attorney_id, attorney_name, start_time, end_time,
                   duration_minutes, billable_minutes,
                   activity_type as "activity_type: _",
                   description, notes,
                   status as "status: _",
                   entry_type as "entry_type: _",
                   billable_status as "billable_status: _",
                   hourly_rate, amount, discount_percent, discount_amount, final_amount,
                   created_at, updated_at, submitted_at, approved_at, approved_by,
                   billed_at, invoice_id
            FROM time_entries
            WHERE id = ?
            "#,
            entry_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to fetch time entry")?;

        Ok(result)
    }

    async fn query_time_entries(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        filters: &TimeReportFilters,
    ) -> Result<Vec<TimeEntry>> {
        // Simplified query - real implementation would apply all filters
        let results = sqlx::query_as!(
            TimeEntry,
            r#"
            SELECT id, matter_id, attorney_id, attorney_name, start_time, end_time,
                   duration_minutes, billable_minutes,
                   activity_type as "activity_type: _",
                   description, notes,
                   status as "status: _",
                   entry_type as "entry_type: _",
                   billable_status as "billable_status: _",
                   hourly_rate, amount, discount_percent, discount_amount, final_amount,
                   created_at, updated_at, submitted_at, approved_at, approved_by,
                   billed_at, invoice_id
            FROM time_entries
            WHERE start_time >= ? AND start_time <= ?
            ORDER BY start_time DESC
            "#,
            start_date,
            end_date
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to query time entries")?;

        Ok(results)
    }

    async fn get_attorney_name(&self, attorney_id: &str) -> Result<String> {
        // Stub - would query attorneys table
        Ok(format!("Attorney {}", attorney_id))
    }

    async fn get_matter_name(&self, matter_id: &str) -> Result<String> {
        // Stub - would query matters table
        Ok(format!("Matter {}", matter_id))
    }

    async fn get_client_id_for_matter(&self, matter_id: &str) -> Result<Option<String>> {
        // Stub - would query matters table
        Ok(None)
    }

    async fn get_client_name_for_matter(&self, matter_id: &str) -> Result<String> {
        // Stub - would query matters/clients tables
        Ok("Client Name".to_string())
    }
}
