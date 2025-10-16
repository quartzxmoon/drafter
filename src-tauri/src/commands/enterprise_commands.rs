// Enterprise Feature Commands - Tauri IPC handlers for all 33 features
// Provides frontend access to settlement calculator, AI automation, bulk data ingestion, and all enterprise features

use tauri::State;
use crate::services::*;
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

// ============================================================================
// FLAGSHIP FEATURE: Settlement Calculator & Demand Generator
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculateSettlementRequest {
    pub matter_id: String,
    pub case_type: settlement_calculator::CaseType,
    pub economic_damages: settlement_calculator::EconomicDamages,
    pub injury_details: Option<settlement_calculator::PersonalInjuryDetails>,
    pub liability_percentage: f64,
    pub jurisdiction: String,
    pub plaintiff_name: String,
    pub defendant_name: String,
}

#[tauri::command]
pub async fn cmd_calculate_settlement(
    request: CalculateSettlementRequest,
    db: State<'_, SqlitePool>,
) -> Result<settlement_calculator::SettlementCalculation, String> {
    let service = settlement_calculator::SettlementCalculatorService::new(db.inner().clone());

    service
        .calculate_settlement(
            &request.matter_id,
            request.case_type,
            request.economic_damages,
            request.injury_details,
            request.liability_percentage,
            &request.jurisdiction,
        )
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateDemandLetterRequest {
    pub settlement_calculation: settlement_calculator::SettlementCalculation,
    pub recipient_name: String,
    pub recipient_address: String,
    pub case_facts: String,
    pub liability_description: String,
    pub damages_description: String,
}

#[tauri::command]
pub async fn cmd_generate_demand_letter(
    request: GenerateDemandLetterRequest,
    db: State<'_, SqlitePool>,
) -> Result<settlement_calculator::DemandLetter, String> {
    let service = settlement_calculator::SettlementCalculatorService::new(db.inner().clone());

    service
        .generate_demand_letter(
            &request.settlement_calculation,
            &request.recipient_name,
            &request.case_facts,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_analyze_settlement_offer(
    settlement_calc_id: String,
    offer_amount: f64,
    offer_terms: String,
    db: State<'_, SqlitePool>,
) -> Result<settlement_calculator::OfferAnalysis, String> {
    let service = settlement_calculator::SettlementCalculatorService::new(db.inner().clone());

    service
        .analyze_offer(&settlement_calc_id, offer_amount, &offer_terms)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// CRITICAL FEATURE: Bulk Data Ingestion
// ============================================================================

#[tauri::command]
pub async fn cmd_start_bulk_ingestion_courtlistener(
    db: State<'_, SqlitePool>,
) -> Result<bulk_data_ingestion::BulkIngestionJob, String> {
    let service = bulk_data_ingestion::BulkDataIngestionService::new(db.inner().clone());

    service
        .ingest_courtlistener_bulk()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_start_bulk_ingestion_govinfo(
    db: State<'_, SqlitePool>,
) -> Result<bulk_data_ingestion::BulkIngestionJob, String> {
    let service = bulk_data_ingestion::BulkDataIngestionService::new(db.inner().clone());

    service
        .ingest_govinfo_bulk()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_start_bulk_ingestion_harvard(
    db: State<'_, SqlitePool>,
) -> Result<bulk_data_ingestion::BulkIngestionJob, String> {
    let service = bulk_data_ingestion::BulkDataIngestionService::new(db.inner().clone());

    service
        .ingest_harvard_caselaw_bulk()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_ingestion_status(
    job_id: String,
    db: State<'_, SqlitePool>,
) -> Result<bulk_data_ingestion::BulkIngestionJob, String> {
    let service = bulk_data_ingestion::BulkDataIngestionService::new(db.inner().clone());

    service
        .get_job_status(&job_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_search_ingested_cases(
    query: String,
    filters: Option<bulk_data_ingestion::SearchFilters>,
    limit: Option<usize>,
    db: State<'_, SqlitePool>,
) -> Result<Vec<bulk_data_ingestion::CaseLawDocument>, String> {
    let service = bulk_data_ingestion::BulkDataIngestionService::new(db.inner().clone());

    service
        .search_cases(&query, filters, limit)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// GAME CHANGER: AI Automation Suite
// ============================================================================

#[tauri::command]
pub async fn cmd_automate_case_lifecycle(
    matter_id: String,
    db: State<'_, SqlitePool>,
) -> Result<ai_automation_suite::AutomatedCaseWorkflow, String> {
    let service = ai_automation_suite::AIAutomationService::new(db.inner().clone());

    service
        .automate_case_lifecycle(&matter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_automate_client_management(
    client_id: String,
    db: State<'_, SqlitePool>,
) -> Result<ai_automation_suite::AutomatedClientManagement, String> {
    let service = ai_automation_suite::AIAutomationService::new(db.inner().clone());

    service
        .automate_client_management(&client_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_automate_team_management(
    firm_id: String,
    db: State<'_, SqlitePool>,
) -> Result<ai_automation_suite::AutomatedTeamManagement, String> {
    let service = ai_automation_suite::AIAutomationService::new(db.inner().clone());

    service
        .automate_team_management(&firm_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_predict_case_outcome(
    matter_id: String,
    db: State<'_, SqlitePool>,
) -> Result<ai_automation_suite::CasePrediction, String> {
    let service = ai_automation_suite::AIAutomationService::new(db.inner().clone());

    service
        .predict_case_outcome(&matter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_optimize_firm_workflow(
    firm_id: String,
    db: State<'_, SqlitePool>,
) -> Result<ai_automation_suite::WorkflowOptimization, String> {
    let service = ai_automation_suite::AIAutomationService::new(db.inner().clone());

    service
        .optimize_workflow(&firm_id)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Tier 1 Features: Document Assembly, Conflict Checking, Time Tracking, Billing
// ============================================================================

#[tauri::command]
pub async fn cmd_assemble_document(
    template_id: String,
    matter_id: String,
    variable_data: serde_json::Value,
    db: State<'_, SqlitePool>,
) -> Result<document_assembly::AssembledDocument, String> {
    let service = document_assembly::DocumentAssemblyService::new(db.inner().clone());

    service
        .assemble_from_template(&template_id, &matter_id, variable_data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_run_conflict_check(
    client_name: String,
    matter_description: String,
    opposing_parties: Vec<String>,
    db: State<'_, SqlitePool>,
) -> Result<conflict_checking::ConflictCheckReport, String> {
    let service = conflict_checking::ConflictCheckingService::new(db.inner().clone());

    service
        .run_conflict_check(&client_name, &matter_description, opposing_parties)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_start_time_entry(
    matter_id: String,
    attorney_id: String,
    description: String,
    db: State<'_, SqlitePool>,
) -> Result<time_tracking::TimeEntry, String> {
    let service = time_tracking::TimeTrackingService::new(db.inner().clone());

    service
        .start_timer(&matter_id, &attorney_id, &description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_stop_time_entry(
    entry_id: String,
    db: State<'_, SqlitePool>,
) -> Result<time_tracking::TimeEntry, String> {
    let service = time_tracking::TimeTrackingService::new(db.inner().clone());

    service
        .stop_timer(&entry_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_generate_invoice(
    matter_id: String,
    billing_period_start: String,
    billing_period_end: String,
    db: State<'_, SqlitePool>,
) -> Result<billing::Invoice, String> {
    let service = billing::BillingService::new(db.inner().clone());

    service
        .generate_invoice(&matter_id, &billing_period_start, &billing_period_end)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_process_payment(
    invoice_id: String,
    amount: f64,
    payment_method: billing::PaymentMethod,
    db: State<'_, SqlitePool>,
) -> Result<billing::Payment, String> {
    let service = billing::BillingService::new(db.inner().clone());

    service
        .process_payment(&invoice_id, amount, payment_method)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Tier 1 Features: Email, Contract Review, Legal Research
// ============================================================================

#[tauri::command]
pub async fn cmd_sync_emails(
    account_id: String,
    db: State<'_, SqlitePool>,
) -> Result<email_integration::EmailSyncResult, String> {
    let service = email_integration::EmailIntegrationService::new(db.inner().clone());

    service
        .sync_emails(&account_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_link_email_to_matter(
    email_id: String,
    matter_id: String,
    db: State<'_, SqlitePool>,
) -> Result<(), String> {
    let service = email_integration::EmailIntegrationService::new(db.inner().clone());

    service
        .link_to_matter(&email_id, &matter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_review_contract(
    document_path: String,
    contract_type: contract_review::ContractType,
    db: State<'_, SqlitePool>,
) -> Result<contract_review::ContractAnalysis, String> {
    let service = contract_review::ContractReviewService::new(db.inner().clone());

    service
        .analyze_contract(&document_path, contract_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_research_legal_issue(
    query: String,
    jurisdiction: String,
    practice_area: String,
    db: State<'_, SqlitePool>,
) -> Result<legal_research::ResearchResults, String> {
    let service = legal_research::LegalResearchService::new(db.inner().clone());

    service
        .research(&query, &jurisdiction, &practice_area)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Tier 2 Features: Discovery, Expert Witness, Court Filing, CRM
// ============================================================================

#[tauri::command]
pub async fn cmd_create_discovery_request(
    matter_id: String,
    request_type: discovery::DiscoveryType,
    db: State<'_, SqlitePool>,
) -> Result<discovery::DiscoveryRequest, String> {
    let service = discovery::DiscoveryService::new(db.inner().clone());

    service
        .create_discovery_request(&matter_id, request_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_generate_privilege_log(
    matter_id: String,
    db: State<'_, SqlitePool>,
) -> Result<discovery::PrivilegeLog, String> {
    let service = discovery::DiscoveryService::new(db.inner().clone());

    service
        .generate_privilege_log(&matter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_search_expert_witnesses(
    specialty: String,
    db: State<'_, SqlitePool>,
) -> Result<Vec<expert_witness::ExpertWitness>, String> {
    let service = expert_witness::ExpertWitnessService::new(db.inner().clone());

    service
        .search_experts(&specialty)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_submit_court_filing(
    filing: court_filing::EFiling,
    db: State<'_, SqlitePool>,
) -> Result<String, String> {
    let service = court_filing::CourtFilingService::new(db.inner().clone());

    service
        .submit_filing(&filing)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_create_lead(
    name: String,
    email: String,
    db: State<'_, SqlitePool>,
) -> Result<crm::Lead, String> {
    let service = crm::CRMService::new(db.inner().clone());

    service
        .create_lead(&name, &email)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_convert_lead_to_client(
    lead_id: String,
    db: State<'_, SqlitePool>,
) -> Result<String, String> {
    let service = crm::CRMService::new(db.inner().clone());

    service
        .convert_to_client(&lead_id)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Additional Enterprise Features
// ============================================================================

#[tauri::command]
pub async fn cmd_transcribe_audio(
    audio_path: String,
    language: Option<String>,
    db: State<'_, SqlitePool>,
) -> Result<speech_to_text::Transcription, String> {
    let service = speech_to_text::SpeechToTextService::new(db.inner().clone());

    service
        .transcribe_file(&audio_path, language)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_run_analytics_report(
    report_type: analytics::ReportType,
    date_range: analytics::DateRange,
    db: State<'_, SqlitePool>,
) -> Result<analytics::AnalyticsReport, String> {
    let service = analytics::AnalyticsService::new(db.inner().clone());

    service
        .generate_report(report_type, date_range)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_check_iolta_compliance(
    trust_account_id: String,
    db: State<'_, SqlitePool>,
) -> Result<compliance::IOLTAComplianceReport, String> {
    let service = compliance::ComplianceService::new(db.inner().clone());

    service
        .check_iolta_compliance(&trust_account_id)
        .await
        .map_err(|e| e.to_string())
}
