// API module - REST API server for external integrations
// Provides comprehensive REST endpoints for all enterprise features

pub mod rest_api;

// Re-export main API server creation function
pub use rest_api::create_api_server;
