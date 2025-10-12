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

// Import command handlers
use crate::services::commands::*;

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
