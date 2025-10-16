// Calendar Sync Service
// Integration with Google Calendar, Outlook Calendar, and Apple Calendar

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub attendees: Vec<Attendee>,
    pub reminders: Vec<Reminder>,
    pub calendar_provider: CalendarProvider,
    pub external_id: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendee {
    pub email: String,
    pub name: Option<String>,
    pub response_status: ResponseStatus,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub method: ReminderMethod,
    pub minutes_before: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderMethod {
    Email,
    Popup,
    SMS,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalendarProvider {
    Google,
    Outlook,
    Apple,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Synced,
    Pending,
    Failed,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalDeadline {
    pub id: String,
    pub matter_id: String,
    pub docket_number: Option<String>,
    pub deadline_type: DeadlineType,
    pub deadline_date: DateTime<Utc>,
    pub calculated_from: Option<DateTime<Utc>>,
    pub jurisdiction: Option<String>,
    pub court_rules: Vec<String>,
    pub description: String,
    pub priority: Priority,
    pub auto_calculated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeadlineType {
    Filing,
    Response,
    Discovery,
    Trial,
    Appeal,
    Settlement,
    Statute OfLimitations,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

pub struct CalendarSyncService {
    client: Client,
    google_credentials: Option<GoogleCalendarCredentials>,
    outlook_credentials: Option<OutlookCalendarCredentials>,
}

#[derive(Clone)]
struct GoogleCalendarCredentials {
    access_token: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
}

#[derive(Clone)]
struct OutlookCalendarCredentials {
    access_token: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
    tenant_id: String,
}

impl CalendarSyncService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            google_credentials: None,
            outlook_credentials: None,
        }
    }

    pub fn with_google(mut self, credentials: GoogleCalendarCredentials) -> Self {
        self.google_credentials = Some(credentials);
        self
    }

    pub fn with_outlook(mut self, credentials: OutlookCalendarCredentials) -> Self {
        self.outlook_credentials = Some(credentials);
        self
    }

    /// Sync event to calendar provider
    pub async fn sync_event(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        match event.calendar_provider {
            CalendarProvider::Google => self.sync_to_google(event).await,
            CalendarProvider::Outlook => self.sync_to_outlook(event).await,
            CalendarProvider::Apple => self.sync_to_apple(event).await,
            CalendarProvider::Local => Ok(event),
        }
    }

    async fn sync_to_google(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        let creds = self.google_credentials.as_ref()
            .ok_or_else(|| anyhow!("Google Calendar not configured"))?;

        let google_event = serde_json::json!({
            "summary": event.title,
            "description": event.description,
            "location": event.location,
            "start": {
                "dateTime": event.start_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": event.end_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "attendees": event.attendees.iter().map(|a| serde_json::json!({
                "email": a.email,
                "displayName": a.name,
                "optional": a.optional
            })).collect::<Vec<_>>(),
            "reminders": {
                "useDefault": false,
                "overrides": event.reminders.iter().map(|r| serde_json::json!({
                    "method": match r.method {
                        ReminderMethod::Email => "email",
                        ReminderMethod::Popup => "popup",
                        ReminderMethod::SMS => "sms",
                    },
                    "minutes": r.minutes_before
                })).collect::<Vec<_>>()
            }
        });

        let url = "https://www.googleapis.com/calendar/v3/calendars/primary/events";

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .header("Content-Type", "application/json")
            .json(&google_event)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google Calendar API error: {}", error_text));
        }

        let created_event: serde_json::Value = response.json().await?;
        let external_id = created_event["id"].as_str()
            .ok_or_else(|| anyhow!("No event ID in response"))?;

        info!("Google Calendar event created: {}", external_id);

        Ok(CalendarEvent {
            external_id: Some(external_id.to_string()),
            sync_status: SyncStatus::Synced,
            ..event
        })
    }

    async fn sync_to_outlook(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        let creds = self.outlook_credentials.as_ref()
            .ok_or_else(|| anyhow!("Outlook Calendar not configured"))?;

        let outlook_event = serde_json::json!({
            "subject": event.title,
            "body": {
                "contentType": "HTML",
                "content": event.description.unwrap_or_default()
            },
            "start": {
                "dateTime": event.start_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": event.end_time.to_rfc3339(),
                "timeZone": "UTC"
            },
            "location": {
                "displayName": event.location.unwrap_or_default()
            },
            "attendees": event.attendees.iter().map(|a| serde_json::json!({
                "emailAddress": {
                    "address": a.email,
                    "name": a.name
                },
                "type": if a.optional { "optional" } else { "required" }
            })).collect::<Vec<_>>(),
            "isReminderOn": !event.reminders.is_empty(),
            "reminderMinutesBeforeStart": event.reminders.first()
                .map(|r| r.minutes_before)
                .unwrap_or(15)
        });

        let url = "https://graph.microsoft.com/v1.0/me/events";

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", creds.access_token))
            .header("Content-Type", "application/json")
            .json(&outlook_event)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Outlook Calendar API error: {}", error_text));
        }

        let created_event: serde_json::Value = response.json().await?;
        let external_id = created_event["id"].as_str()
            .ok_or_else(|| anyhow!("No event ID in response"))?;

        info!("Outlook Calendar event created: {}", external_id);

        Ok(CalendarEvent {
            external_id: Some(external_id.to_string()),
            sync_status: SyncStatus::Synced,
            ..event
        })
    }

    async fn sync_to_apple(&self, event: CalendarEvent) -> Result<CalendarEvent> {
        // Apple Calendar uses CalDAV protocol
        // For now, return as local event
        warn!("Apple Calendar sync not fully implemented, storing locally");
        Ok(CalendarEvent {
            sync_status: SyncStatus::Pending,
            ..event
        })
    }

    /// Calculate legal deadlines based on court rules
    pub fn calculate_legal_deadline(
        &self,
        event_date: DateTime<Utc>,
        days_to_add: i64,
        jurisdiction: &str,
        exclude_weekends: bool,
        exclude_court_holidays: bool,
    ) -> Result<DateTime<Utc>> {
        let mut deadline = event_date;
        let mut days_added = 0;

        // Pennsylvania court holidays (simplified list)
        let court_holidays = vec![
            "01-01", // New Year's Day
            "07-04", // Independence Day
            "12-25", // Christmas
        ];

        while days_added < days_to_add {
            deadline = deadline + Duration::days(1);

            // Skip weekends
            if exclude_weekends {
                let weekday = deadline.weekday();
                if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                    continue;
                }
            }

            // Skip court holidays
            if exclude_court_holidays {
                let date_str = deadline.format("%m-%d").to_string();
                if court_holidays.contains(&date_str.as_str()) {
                    continue;
                }
            }

            days_added += 1;
        }

        Ok(deadline)
    }

    /// Create calendar event from legal deadline
    pub fn deadline_to_calendar_event(&self, deadline: LegalDeadline) -> CalendarEvent {
        let title = format!("{} - {}", deadline.deadline_type_str(), deadline.docket_number.as_deref().unwrap_or("No Docket"));

        CalendarEvent {
            id: deadline.id.clone(),
            title,
            description: Some(deadline.description.clone()),
            location: deadline.jurisdiction.clone(),
            start_time: deadline.deadline_date - Duration::hours(1),
            end_time: deadline.deadline_date,
            all_day: true,
            attendees: Vec::new(),
            reminders: vec![
                Reminder {
                    method: ReminderMethod::Email,
                    minutes_before: 24 * 60, // 1 day before
                },
                Reminder {
                    method: ReminderMethod::Popup,
                    minutes_before: 60, // 1 hour before
                },
            ],
            calendar_provider: CalendarProvider::Local,
            external_id: None,
            sync_status: SyncStatus::Pending,
        }
    }

    /// Sync all pending legal deadlines to calendar
    pub async fn sync_all_deadlines(&self, deadlines: Vec<LegalDeadline>) -> Result<Vec<CalendarEvent>> {
        let mut synced_events = Vec::new();

        for deadline in deadlines {
            let event = self.deadline_to_calendar_event(deadline);
            match self.sync_event(event).await {
                Ok(synced_event) => {
                    synced_events.push(synced_event);
                }
                Err(e) => {
                    error!("Failed to sync deadline: {}", e);
                }
            }
        }

        Ok(synced_events)
    }
}

impl LegalDeadline {
    fn deadline_type_str(&self) -> &str {
        match self.deadline_type {
            DeadlineType::Filing => "Filing Deadline",
            DeadlineType::Response => "Response Deadline",
            DeadlineType::Discovery => "Discovery Deadline",
            DeadlineType::Trial => "Trial Date",
            DeadlineType::Appeal => "Appeal Deadline",
            DeadlineType::Settlement => "Settlement Conference",
            DeadlineType::StatuteOfLimitations => "Statute of Limitations",
            DeadlineType::Custom => "Deadline",
        }
    }
}

impl Default for CalendarSyncService {
    fn default() -> Self {
        Self::new()
    }
}
