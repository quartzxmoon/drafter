// Case Management Domain Models

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Client Management
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub ssn_encrypted: Option<String>,
    pub notes: Option<String>,
    pub client_type: ClientType,
    pub business_name: Option<String>,
    pub contact_person: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: ClientStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Individual,
    Business,
    Government,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientStatus {
    Active,
    Inactive,
    Archived,
}

// ============================================================================
// Matter/Case Management
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matter {
    pub id: String,
    pub client_id: String,
    pub matter_number: String,
    pub title: String,
    pub description: Option<String>,
    pub matter_type: MatterType,
    pub case_type: Option<String>,
    pub court_level: Option<String>,
    pub court_name: Option<String>,
    pub county: Option<String>,
    pub docket_number: Option<String>,
    pub judge_name: Option<String>,
    pub opposing_party: Option<String>,
    pub opposing_counsel: Option<String>,
    pub opposing_counsel_firm: Option<String>,
    pub opposing_counsel_email: Option<String>,
    pub opposing_counsel_phone: Option<String>,
    pub filing_date: Option<NaiveDate>,
    pub status: MatterStatus,
    pub outcome: Option<String>,
    pub settlement_amount: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatterType {
    Civil,
    Criminal,
    Family,
    Estate,
    RealEstate,
    Business,
    Employment,
    PersonalInjury,
    Immigration,
    Bankruptcy,
    IntellectualProperty,
    Administrative,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatterStatus {
    Active,
    Pending,
    Closed,
    Archived,
}

// ============================================================================
// Case Participants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseParticipant {
    pub id: String,
    pub matter_id: String,
    pub role: ParticipantRole,
    pub party_type: PartyType,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Plaintiff,
    Defendant,
    Witness,
    Expert,
    CoDefendant,
    CoPlaintiff,
    Intervenor,
    Victim,
    Guardian,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PartyType {
    Person,
    Organization,
}

// ============================================================================
// Case Events and Timeline
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvent {
    pub id: String,
    pub matter_id: String,
    pub event_type: EventType,
    pub title: String,
    pub description: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: Option<NaiveTime>,
    pub location: Option<String>,
    pub participants: Vec<String>,
    pub outcome: Option<String>,
    pub notes: Option<String>,
    pub reminder_set: bool,
    pub reminder_date: Option<DateTime<Utc>>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Filing,
    Hearing,
    Deadline,
    Conference,
    Trial,
    Deposition,
    Mediation,
    Arbitration,
    Discovery,
    MotionHearing,
    StatusConference,
    Settlement,
    Appeal,
    Sentencing,
    Other,
}

// ============================================================================
// Tasks and Deadlines
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub matter_id: Option<String>,
    pub assigned_to: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub due_date: Option<NaiveDate>,
    pub due_time: Option<NaiveTime>,
    pub status: TaskStatus,
    pub category: Option<TaskCategory>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Research,
    Drafting,
    Filing,
    ClientCommunication,
    CourtAppearance,
    Discovery,
    Investigation,
    Review,
    Other,
}

// ============================================================================
// Document Management
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseDocument {
    pub id: String,
    pub matter_id: String,
    pub document_type: DocumentType,
    pub title: String,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub version: i32,
    pub is_template: bool,
    pub filed_with_court: bool,
    pub filing_date: Option<NaiveDate>,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    // Pleadings
    Complaint,
    Answer,
    Petition,
    Response,
    Counterclaim,
    CrossClaim,
    ThirdPartyClaim,

    // Motions
    Motion,
    MotionToCompel,
    MotionToDismiss,
    MotionForSummaryJudgment,
    MotionInLimine,
    MotionForContinuance,
    MotionToSuppress,

    // Briefs and Memoranda
    Brief,
    Memorandum,
    MemorandumOfLaw,
    ReplyBrief,

    // Discovery
    Interrogatories,
    RequestForProduction,
    RequestForAdmissions,
    DiscoveryResponse,

    // Evidence
    Affidavit,
    Declaration,
    Exhibit,
    ExpertReport,

    // Orders and Judgments
    Order,
    Judgment,
    Decree,
    Stipulation,

    // Administrative
    NoticeOfAppearance,
    NoticeOfWithdrawal,
    CertificateOfService,
    ProofOfService,

    // Correspondence
    Letter,
    Email,
    Notice,

    // Table and Index
    TableOfAuthorities,
    TableOfContents,

    // Other
    Contract,
    Agreement,
    Settlement,
    Transcript,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub version: i32,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub changes_summary: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Notes and Journal
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNote {
    pub id: String,
    pub matter_id: String,
    pub note_type: NoteType,
    pub title: Option<String>,
    pub content: String,
    pub is_private: bool,
    pub tags: Vec<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteType {
    General,
    Research,
    Strategy,
    ClientCommunication,
    CourtCommunication,
    InternalMemo,
}

// ============================================================================
// Time and Billing
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub matter_id: String,
    pub task_id: Option<String>,
    pub attorney_id: Option<String>,
    pub entry_date: NaiveDate,
    pub hours: f32,
    pub rate: Option<f32>,
    pub description: String,
    pub billable: bool,
    pub billed: bool,
    pub invoice_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: String,
    pub matter_id: String,
    pub expense_date: NaiveDate,
    pub category: ExpenseCategory,
    pub amount: f32,
    pub description: String,
    pub receipt_path: Option<String>,
    pub billable: bool,
    pub billed: bool,
    pub invoice_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategory {
    FilingFee,
    ServiceFee,
    ExpertFee,
    Travel,
    Copying,
    Research,
    CourtReporter,
    Investigation,
    Other,
}

// ============================================================================
// Templates and Auto-generation
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: String,
    pub template_name: String,
    pub document_type: DocumentType,
    pub court_level: Option<String>,
    pub matter_types: Vec<MatterType>,
    pub description: Option<String>,
    pub template_content: String,
    pub variable_schema: serde_json::Value,
    pub auto_populate_rules: Option<serde_json::Value>,
    pub formatting_rules: Option<serde_json::Value>,
    pub file_path: Option<String>,
    pub is_public: bool,
    pub is_pro_se_friendly: bool,
    pub category: TemplateCategory,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Pleading,
    Motion,
    Discovery,
    Correspondence,
    Brief,
    Administrative,
    Settlement,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoGenerationRule {
    pub id: String,
    pub template_id: String,
    pub trigger_event: TriggerEvent,
    pub matter_type: Option<MatterType>,
    pub conditions: serde_json::Value,
    pub variable_mappings: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerEvent {
    CaseFiled,
    HearingScheduled,
    DeadlineApproaching,
    DiscoveryReceived,
    MotionFiled,
    OrderReceived,
    Manual,
}

// ============================================================================
// User Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: String,
    pub user_id: String,
    pub attorney_name: Option<String>,
    pub bar_number: Option<String>,
    pub firm_name: Option<String>,
    pub firm_address: Option<String>,
    pub firm_phone: Option<String>,
    pub firm_email: Option<String>,
    pub default_signature: Option<String>,
    pub letterhead_template: Option<String>,
    pub billing_rate: Option<f32>,
    pub timezone: String,
    pub date_format: String,
    pub pro_se_mode: bool,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub client_type: ClientType,
    pub business_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMatterRequest {
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub matter_type: MatterType,
    pub case_type: Option<String>,
    pub court_level: Option<String>,
    pub court_name: Option<String>,
    pub county: Option<String>,
    pub opposing_party: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentRequest {
    pub matter_id: String,
    pub template_id: String,
    pub document_type: DocumentType,
    pub title: String,
    pub auto_populate: bool,
    pub custom_variables: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentResponse {
    pub document_id: String,
    pub file_path: String,
    pub preview_html: String,
    pub warnings: Vec<String>,
    pub missing_data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatterSummary {
    pub matter: Matter,
    pub client: Client,
    pub events_count: i32,
    pub documents_count: i32,
    pub tasks_pending: i32,
    pub next_deadline: Option<CaseEvent>,
    pub total_time: f32,
    pub total_expenses: f32,
}
