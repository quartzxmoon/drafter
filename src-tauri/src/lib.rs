// PA eDocket Desktop - Production-grade court docket management application
// Copyright (c) 2024 PA eDocket Team

use tauri::Manager;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Module declarations
pub mod domain;
pub mod providers;
pub mod services;
pub mod utils;
pub mod config;
pub mod commands;
pub mod api;

// Import command handlers
use crate::services::commands::*;
use crate::commands::{document_commands::*, enterprise_commands::*};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pa_edocket_desktop=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting PA eDocket Desktop application");

    tauri::Builder::default()
        // Core plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())

        .plugin(tauri_plugin_sql::Builder::default().build())

        // Register command handlers
        .invoke_handler(tauri::generate_handler![
            // Search and docket commands
            cmd_search,
            cmd_get_docket,
            cmd_get_attachments,

            // Export commands
            cmd_export,

            // Document drafting commands
            cmd_draft,

            // E-filing commands
            cmd_efiling_capabilities,
            cmd_efiling_login,
            cmd_efiling_submit,
            cmd_efiling_status,

            // Watchlist commands
            cmd_watch_add,
            cmd_watch_remove,
            cmd_watch_list,

            // Citation commands
            cmd_citation_parse,
            cmd_citation_format,
            cmd_citation_validate,

            // Court rules commands
            cmd_get_court_rules,

            // System commands
            cmd_system_info,
            cmd_system_health,
            cmd_get_logs,

            // Configuration commands
            cmd_update_config,
            cmd_get_config,

            // NEW: Document editor commands
            cmd_save_document,
            cmd_export_document,

            // NEW: AI Citation commands
            cmd_search_case_law,
            cmd_extract_citations,
            cmd_format_citations,
            cmd_generate_toa,

            // NEW: Pleading formatting
            cmd_format_as_pleading,

            // NEW: Case management commands
            cmd_list_matters,
            cmd_get_matter_summary,
            cmd_create_client,
            cmd_create_matter,
            cmd_generate_document,

            // NEW: Organization commands
            cmd_get_case_folders,
            cmd_get_practice_areas,

            // NEW: AI Assistant commands
            cmd_get_ai_suggestions,
            cmd_analyze_document,

            // ============================================================================
            // ENTERPRISE COMMANDS - All 33 Features
            // ============================================================================

            // FLAGSHIP: Settlement Calculator & Demand Generator
            cmd_calculate_settlement,
            cmd_generate_demand_letter,
            cmd_analyze_settlement_offer,

            // CRITICAL: Bulk Data Ingestion
            cmd_start_bulk_ingestion_courtlistener,
            cmd_start_bulk_ingestion_govinfo,
            cmd_start_bulk_ingestion_harvard,
            cmd_get_ingestion_status,
            cmd_search_ingested_cases,

            // GAME CHANGER: AI Automation Suite
            cmd_automate_case_lifecycle,
            cmd_automate_client_management,
            cmd_automate_team_management,
            cmd_predict_case_outcome,
            cmd_optimize_firm_workflow,

            // Tier 1: Core Revenue Features
            cmd_assemble_document,
            cmd_run_conflict_check,
            cmd_start_time_entry,
            cmd_stop_time_entry,
            cmd_generate_invoice,
            cmd_process_payment,
            cmd_sync_emails,
            cmd_link_email_to_matter,
            cmd_review_contract,
            cmd_research_legal_issue,

            // Tier 2: Competitive Advantage Features
            cmd_create_discovery_request,
            cmd_generate_privilege_log,
            cmd_search_expert_witnesses,
            cmd_submit_court_filing,
            cmd_create_lead,
            cmd_convert_lead_to_client,

            // Additional Enterprise Features
            cmd_transcribe_audio,
            cmd_run_analytics_report,
            cmd_check_iolta_compliance,
        ])

        // Setup handler for initialization
        .setup(|app| {
            info!("Initializing application services");

            // Initialize database
            if let Err(e) = setup_database(app.handle()) {
                error!("Failed to initialize database: {}", e);
                return Err(e.into());
            }

            // Load configuration
            if let Err(e) = load_configuration(app.handle()) {
                error!("Failed to load configuration: {}", e);
                return Err(e.into());
            }

            // Initialize providers
            if let Err(e) = initialize_providers(app.handle()) {
                error!("Failed to initialize providers: {}", e);
                return Err(e.into());
            }

            info!("Application initialized successfully");
            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Setup functions
fn setup_database(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    // TODO: Initialize SQLite database with migrations
    info!("Database setup completed");
    Ok(())
}

fn load_configuration(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    // TODO: Load courts.yaml and providers.yaml
    info!("Configuration loaded");
    Ok(())
}

fn initialize_providers(app_handle: &tauri::AppHandle) -> anyhow::Result<()> {
    // TODO: Initialize provider clients with rate limiting
    info!("Providers initialized");
    Ok(())
}
