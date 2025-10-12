// Provider layer for PA eDocket Desktop
// Handles integration with external court data sources

pub mod ujs_portal;
pub mod pacfile;
pub mod county_efiling;
pub mod ctrack;
pub mod rate_limiter;
pub mod client;

// Common provider traits and types
use crate::domain::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait SearchProvider {
    async fn search(&self, params: &SearchParams) -> Result<Vec<SearchResult>, ProviderError>;
    async fn get_docket(&self, id: &str) -> Result<Docket, ProviderError>;
    async fn get_attachments(&self, docket_id: &str) -> Result<Vec<Attachment>, ProviderError>;
}

#[async_trait]
pub trait EFilingProvider {
    async fn get_capabilities(&self, court_id: &str) -> Result<Vec<EFilingCapability>, ProviderError>;
    async fn authenticate(&self, credentials: HashMap<String, String>) -> Result<EFilingSession, ProviderError>;
    async fn submit_filing(&self, submission: &EFilingSubmission) -> Result<String, ProviderError>;
    async fn get_status(&self, submission_id: &str) -> Result<EFilingSubmission, ProviderError>;
    async fn refresh_token(&self, session: &EFilingSession) -> Result<EFilingSession, ProviderError>;
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub base_url: String,
    pub rate_limit: RateLimitConfig,
    pub retry: RetryConfig,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_multiplier: f64,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

// Provider error types
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Rate limit exceeded")]
    RateLimited,
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Parsing error: {0}")]
    Parsing(String),
}

pub type ProviderResult<T> = Result<T, ProviderError>;
