// Core domain models for PA eDocket Desktop
// Production-ready Rust structs with serde serialization

pub mod case_management;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CourtLevel {
    #[serde(rename = "MDJ")]
    Mdj,
    #[serde(rename = "CP")]
    Cp,
    #[serde(rename = "APP")]
    App,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartyRole {
    Plaintiff,
    Defendant,
    Appellant,
    Appellee,
    Petitioner,
    Respondent,
    Intervenor,
    #[serde(rename = "Third Party Defendant")]
    ThirdPartyDefendant,
    #[serde(rename = "Cross Defendant")]
    CrossDefendant,
    #[serde(rename = "Cross Plaintiff")]
    CrossPlaintiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChargeGrade {
    F1, F2, F3,
    M1, M2, M3,
    S, V,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaseStatus {
    Active,
    Closed,
    Pending,
    Disposed,
    Appealed,
    Transferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Filing,
    Hearing,
    Order,
    Motion,
    Trial,
    Sentencing,
    Appeal,
    Settlement,
    Dismissal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinancialType {
    Fine,
    Cost,
    Restitution,
    Fee,
    Bail,
    Bond,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchParams {
    pub term: Option<String>,
    pub court: Option<CourtLevel>,
    pub county: Option<String>,
    pub from: Option<String>, // ISO date string
    pub to: Option<String>,   // ISO date string
    pub docket: Option<String>,
    pub otn: Option<String>,
    pub sid: Option<String>,
    #[validate(range(min = 1, max = 1000))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub caption: String,
    pub court: CourtLevel,
    pub county: String,
    pub filed: String,
    pub status: CaseStatus,
    pub last_updated: Option<String>,
    pub docket_number: Option<String>,
    pub otn: Option<String>,
    pub sid: Option<String>,
    pub judge: Option<String>,
    pub courtroom: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Party {
    pub id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub role: PartyRole,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub attorney: Option<String>,
    pub attorney_id: Option<String>,
    pub attorney_phone: Option<String>,
    #[validate(email)]
    pub attorney_email: Option<String>,
    pub date_added: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Charge {
    pub sequence: Option<u32>,
    pub id: Option<Uuid>,
    #[validate(length(min = 1, max = 100))]
    pub statute: String,
    pub grade: Option<ChargeGrade>,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    pub disposition: Option<String>,
    pub disposition_date: Option<DateTime<Utc>>,
    pub sentence: Option<String>,
    pub plea: Option<String>,
    pub verdict: Option<String>,
    pub counts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Event {
    pub description: Option<String>,
    pub time: Option<String>,
    pub id: Option<Uuid>,
    pub event_type: EventType,
    pub when: DateTime<Utc>,
    pub location: Option<String>,
    pub courtroom: Option<String>,
    pub judge: Option<String>,
    pub notes: Option<String>,
    pub result: Option<String>,
    pub next_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Filing {
    pub document_url: Option<String>,
    pub status: Option<String>,
    pub id: Option<Uuid>,
    pub date: DateTime<Utc>,
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub by: Option<String>,
    #[validate(url)]
    pub doc_url: Option<String>,
    pub doc_type: Option<String>,
    pub pages: Option<u32>,
    pub size: Option<u64>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Financial {
    pub id: Option<Uuid>,
    pub financial_type: FinancialType,
    #[validate(range(min = 0.0))]
    pub amount: f64,
    pub balance: f64,
    pub description: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_date: Option<DateTime<Utc>>,
    pub paid_amount: Option<f64>,
    pub payment_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Attachment {
    pub id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(url)]
    pub url: String,
    pub attachment_type: Option<String>,
    pub size: Option<u64>,
    pub hash: Option<String>,
    pub upload_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Docket {
    pub id: String,
    #[validate(length(min = 1, max = 500))]
    pub caption: String,
    pub status: CaseStatus,
    pub court: CourtLevel,
    #[validate(length(min = 1, max = 100))]
    pub county: String,
    pub filed: DateTime<Utc>,
    pub docket_number: Option<String>,
    pub otn: Option<String>,
    pub sid: Option<String>,
    pub judge: Option<String>,
    pub courtroom: Option<String>,
    pub division: Option<String>,
    
    // Related data
    #[validate(nested)]
    pub parties: Vec<Party>,
    #[validate(nested)]
    pub charges: Vec<Charge>,
    #[validate(nested)]
    pub events: Vec<Event>,
    #[validate(nested)]
    pub filings: Vec<Filing>,
    #[validate(nested)]
    pub financials: Vec<Financial>,
    #[validate(nested)]
    pub attachments: Option<Vec<Attachment>>,
    
    // Metadata
    pub last_updated: Option<DateTime<Utc>>,
    #[validate(url)]
    pub source_url: Option<String>,
    pub fetched_at: Option<DateTime<Utc>>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    #[serde(rename = "PDF")]
    Pdf,
    #[serde(rename = "DOCX")]
    Docx,
    #[serde(rename = "BOTH")]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DraftJob {
    pub id: Option<Uuid>,
    #[validate(length(min = 1, max = 100))]
    pub court_id: String,
    #[validate(length(min = 1, max = 100))]
    pub template_id: String,
    #[validate(length(min = 1))]
    pub dockets: Vec<String>,
    pub variables: HashMap<String, serde_json::Value>,
    pub output: OutputFormat,
    
    // Optional metadata
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub status: Option<JobStatus>,
    pub result_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFilingCapability {
    pub court_id: String,
    pub enabled: bool,
    pub provider: String,
    pub document_types: Vec<String>,
    pub max_file_size: u64,
    pub allowed_formats: Vec<String>,
    pub requires_cover_sheet: bool,
    pub supports_electronic_service: bool,
    pub fee_calculation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFilingSession {
    pub id: Uuid,
    pub court_id: String,
    pub provider: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubmissionStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "submitted")]
    Submitted,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFilingSubmission {
    pub id: Uuid,
    pub session_id: Uuid,
    pub docket_id: Option<String>,
    pub document_type: String,
    pub files: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub status: SubmissionStatus,
    pub submission_id: Option<String>,
    pub receipt_path: Option<String>,
    pub error_message: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportType {
    #[serde(rename = "JSON")]
    Json,
    #[serde(rename = "CSV")]
    Csv,
    #[serde(rename = "PDF")]
    Pdf,
    #[serde(rename = "ZIP")]
    Zip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportSource {
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "docket")]
    Docket,
    #[serde(rename = "draft")]
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub hash: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub id: Uuid,
    pub export_type: ExportType,
    pub source: ExportSource,
    pub query: Option<SearchParams>,
    pub docket_id: Option<String>,
    pub job_id: Option<Uuid>,
    
    // Files included
    pub files: Vec<ExportFile>,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub source_url: Option<String>,
    pub total_size: u64,
    pub checksum: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: Uuid,
    pub docket_id: String,
    pub caption: String,
    pub court: CourtLevel,
    pub county: String,
    pub added_at: DateTime<Utc>,
    pub last_checked: Option<DateTime<Utc>>,
    pub last_changed: Option<DateTime<Utc>>,
    pub notify_on_change: bool,
    pub check_interval: u32, // Minutes
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CitationType {
    #[serde(rename = "case")]
    Case,
    #[serde(rename = "statute")]
    Statute,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "constitution")]
    Constitution,
    #[serde(rename = "regulation")]
    Regulation,
    #[serde(rename = "book")]
    Book,
    #[serde(rename = "article")]
    Article,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Citation {
    pub id: Option<Uuid>,
    pub citation_type: CitationType,
    #[validate(length(min = 1, max = 1000))]
    pub full_citation: String,
    pub short_form: Option<String>,
    pub pin_cite: Option<String>,
    pub parenthetical: Option<String>,
    pub signal: Option<String>,

    // Parsed components
    pub title: Option<String>,
    pub reporter: Option<String>,
    pub volume: Option<String>,
    pub page: Option<String>,
    pub year: Option<String>,
    pub court: Option<String>,
    pub jurisdiction: Option<String>,

    // Validation
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtMargins {
    pub top: String,
    pub bottom: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtFont {
    pub family: String,
    pub size: String,
    pub line_spacing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtCaption {
    pub format: String,
    pub include_docket: bool,
    pub include_court: bool,
    pub include_county: bool,
    pub include_judge: bool,
    pub include_division: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtSignature {
    pub attorney_name: bool,
    pub attorney_id: bool,
    pub firm_name: bool,
    pub address: bool,
    pub phone: bool,
    pub email: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtRules {
    pub court_id: String,
    pub margins: CourtMargins,
    pub font: CourtFont,
    pub caption: CourtCaption,
    pub signature: CourtSignature,
    pub service_certificate: bool,
    pub table_of_contents: Option<bool>,
    pub table_of_authorities: Option<bool>,
    pub page_limits: HashMap<String, u32>,
}
