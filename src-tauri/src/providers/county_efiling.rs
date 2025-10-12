// County E-filing Provider
// Config-driven integration with county-specific e-filing systems

use crate::domain::*;
use crate::providers::{client::ProviderClient, EFilingProvider, ProviderConfig, ProviderError, ProviderResult};
use async_trait::async_trait;
use base64;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

pub struct CountyEFilingProvider {
    client: ProviderClient,
    config: ProviderConfig,
    county_configs: HashMap<String, CountyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyConfig {
    pub name: String,
    pub base_url: String,
    pub auth_type: AuthType,
    pub endpoints: HashMap<String, String>,
    pub capabilities: CountyCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyCapabilities {
    pub document_types: Vec<String>,
    pub max_file_size: u64,
    pub allowed_formats: Vec<String>,
    pub requires_cover_sheet: bool,
    pub supports_electronic_service: bool,
    pub fee_calculation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
        scope: Option<String>,
    },
    Basic {
        username: String,
        password: String
    },
    ApiKey {
        key: String,
        header_name: String,
    },
    Session {
        login_url: String,
        username_field: String,
        password_field: String,
    },
}

// County-specific request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct CountyAuthRequest {
    username: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grant_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountyAuthResponse {
    #[serde(alias = "access_token")]
    token: String,
    #[serde(alias = "refresh_token")]
    refresh_token: Option<String>,
    #[serde(alias = "expires_in")]
    expires_in: Option<u64>,
    #[serde(alias = "token_type")]
    token_type: Option<String>,
    #[serde(alias = "session_id")]
    session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountySubmissionRequest {
    case_number: Option<String>,
    document_type: String,
    filing_party: String,
    documents: Vec<CountyDocument>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountyDocument {
    filename: String,
    content_type: String,
    content: String, // base64 encoded
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountySubmissionResponse {
    submission_id: String,
    confirmation_number: Option<String>,
    status: String,
    receipt_url: Option<String>,
    messages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CountyStatusResponse {
    submission_id: String,
    status: String,
    last_updated: DateTime<Utc>,
    receipt_url: Option<String>,
    messages: Vec<String>,
}

impl CountyEFilingProvider {
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let client = ProviderClient::new(config.clone())?;
        let mut county_configs = HashMap::new();

        // Initialize Philadelphia County configuration
        let philadelphia_config = CountyConfig {
            name: "Philadelphia County".to_string(),
            base_url: "https://efiling.courts.phila.gov".to_string(),
            auth_type: AuthType::Session {
                login_url: "/api/auth/login".to_string(),
                username_field: "username".to_string(),
                password_field: "password".to_string(),
            },
            endpoints: {
                let mut endpoints = HashMap::new();
                endpoints.insert("capabilities".to_string(), "/api/capabilities".to_string());
                endpoints.insert("submit".to_string(), "/api/filing/submit".to_string());
                endpoints.insert("status".to_string(), "/api/filing/{id}/status".to_string());
                endpoints.insert("refresh".to_string(), "/api/auth/refresh".to_string());
                endpoints
            },
            capabilities: CountyCapabilities {
                document_types: vec![
                    "Motion".to_string(),
                    "Brief".to_string(),
                    "Petition".to_string(),
                    "Answer".to_string(),
                    "Complaint".to_string(),
                    "Order".to_string(),
                ],
                max_file_size: 25 * 1024 * 1024, // 25MB
                allowed_formats: vec!["pdf".to_string(), "doc".to_string(), "docx".to_string()],
                requires_cover_sheet: true,
                supports_electronic_service: true,
                fee_calculation: true,
            },
        };
        county_configs.insert("philadelphia".to_string(), philadelphia_config);

        // Initialize Allegheny County configuration
        let allegheny_config = CountyConfig {
            name: "Allegheny County".to_string(),
            base_url: "https://efiling.alleghenycourts.us".to_string(),
            auth_type: AuthType::OAuth2 {
                client_id: "allegheny_efiling".to_string(),
                client_secret: "".to_string(), // Will be provided via credentials
                token_url: "/oauth/token".to_string(),
                scope: Some("filing:submit filing:status".to_string()),
            },
            endpoints: {
                let mut endpoints = HashMap::new();
                endpoints.insert("capabilities".to_string(), "/api/v1/capabilities".to_string());
                endpoints.insert("submit".to_string(), "/api/v1/filings".to_string());
                endpoints.insert("status".to_string(), "/api/v1/filings/{id}".to_string());
                endpoints.insert("refresh".to_string(), "/oauth/token".to_string());
                endpoints
            },
            capabilities: CountyCapabilities {
                document_types: vec![
                    "Civil Motion".to_string(),
                    "Civil Brief".to_string(),
                    "Civil Petition".to_string(),
                    "Civil Answer".to_string(),
                    "Civil Complaint".to_string(),
                    "Criminal Motion".to_string(),
                    "Criminal Brief".to_string(),
                ],
                max_file_size: 50 * 1024 * 1024, // 50MB
                allowed_formats: vec!["pdf".to_string()],
                requires_cover_sheet: false,
                supports_electronic_service: true,
                fee_calculation: false,
            },
        };
        county_configs.insert("allegheny".to_string(), allegheny_config);

        Ok(Self {
            client,
            config,
            county_configs,
        })
    }

    pub fn add_county_config(&mut self, county_id: String, config: CountyConfig) {
        self.county_configs.insert(county_id, config);
    }

    fn get_county_from_court_id(&self, court_id: &str) -> Option<&CountyConfig> {
        // Extract county from court ID (e.g., "philadelphia-common-pleas" -> "philadelphia")
        let county_name = court_id.split('-').next()?;
        self.county_configs.get(county_name)
    }

    #[instrument(skip(self, county_config, credentials))]
    async fn authenticate_county(&self, county_config: &CountyConfig, credentials: HashMap<String, String>) -> ProviderResult<CountyAuthResponse> {
        match &county_config.auth_type {
            AuthType::OAuth2 { client_id, token_url, scope, .. } => {
                let client_secret = credentials.get("client_secret")
                    .ok_or_else(|| ProviderError::AuthenticationFailed("Client secret required for OAuth2".to_string()))?;

                let mut body = HashMap::new();
                body.insert("grant_type", "client_credentials");
                body.insert("client_id", client_id);
                body.insert("client_secret", client_secret);
                if let Some(scope) = scope {
                    body.insert("scope", scope);
                }

                let url = format!("{}{}", county_config.base_url, token_url);
                let response: CountyAuthResponse = self.client.post_json(&url, &body).await?;
                Ok(response)
            },
            AuthType::Session { login_url, username_field, password_field } => {
                let username = credentials.get("username")
                    .ok_or_else(|| ProviderError::AuthenticationFailed("Username required".to_string()))?;
                let password = credentials.get("password")
                    .ok_or_else(|| ProviderError::AuthenticationFailed("Password required".to_string()))?;

                let mut body = HashMap::new();
                body.insert(username_field, username);
                body.insert(password_field, password);

                let url = format!("{}{}", county_config.base_url, login_url);
                let response: CountyAuthResponse = self.client.post_json(&url, &body).await?;
                Ok(response)
            },
            AuthType::Basic { .. } => {
                // For basic auth, we don't need a separate auth call
                Ok(CountyAuthResponse {
                    token: "basic_auth".to_string(),
                    refresh_token: None,
                    expires_in: None,
                    token_type: Some("Basic".to_string()),
                    session_id: None,
                })
            },
            AuthType::ApiKey { .. } => {
                // For API key auth, we don't need a separate auth call
                Ok(CountyAuthResponse {
                    token: "api_key".to_string(),
                    refresh_token: None,
                    expires_in: None,
                    token_type: Some("ApiKey".to_string()),
                    session_id: None,
                })
            },
        }
    }
}

#[async_trait]
impl EFilingProvider for CountyEFilingProvider {
    #[instrument(skip(self, court_id))]
    async fn get_capabilities(&self, court_id: &str) -> Result<Vec<EFilingCapability>, ProviderError> {
        info!("Fetching county e-filing capabilities for: {}", court_id);

        let county_config = self.get_county_from_court_id(court_id)
            .ok_or_else(|| ProviderError::Configuration(format!("Unknown court: {}", court_id)))?;

        let capability = EFilingCapability {
            court_id: court_id.to_string(),
            enabled: true,
            provider: format!("county-{}", county_config.name.to_lowercase().replace(' ', "-")),
            document_types: county_config.capabilities.document_types.clone(),
            max_file_size: county_config.capabilities.max_file_size,
            allowed_formats: county_config.capabilities.allowed_formats.clone(),
            requires_cover_sheet: county_config.capabilities.requires_cover_sheet,
            supports_electronic_service: county_config.capabilities.supports_electronic_service,
            fee_calculation: county_config.capabilities.fee_calculation,
        };

        Ok(vec![capability])
    }

    #[instrument(skip(self, credentials))]
    async fn authenticate(&self, credentials: HashMap<String, String>) -> Result<EFilingSession, ProviderError> {
        info!("Authenticating with county e-filing system");

        let court_id = credentials.get("court_id")
            .ok_or_else(|| ProviderError::AuthenticationFailed("Court ID required".to_string()))?;

        let county_config = self.get_county_from_court_id(court_id)
            .ok_or_else(|| ProviderError::Configuration(format!("Unknown court: {}", court_id)))?;

        let auth_response = self.authenticate_county(county_config, credentials.clone()).await?;

        let expires_at = if let Some(expires_in) = auth_response.expires_in {
            Utc::now() + chrono::Duration::seconds(expires_in as i64)
        } else {
            Utc::now() + chrono::Duration::hours(24) // Default 24 hour expiry
        };

        let session = EFilingSession {
            id: Uuid::new_v4(),
            court_id: court_id.clone(),
            provider: format!("county-{}", county_config.name.to_lowercase().replace(' ', "-")),
            token: auth_response.token,
            refresh_token: auth_response.refresh_token,
            expires_at,
            user_id: credentials.get("username").cloned(),
            permissions: vec!["filing:submit".to_string(), "filing:status".to_string()],
        };

        info!("County authentication successful for court: {}", court_id);
        Ok(session)
    }

    #[instrument(skip(self, submission))]
    async fn submit_filing(&self, submission: &EFilingSubmission) -> Result<String, ProviderError> {
        info!("Submitting filing to county system: {}", submission.id);

        // Extract court ID from submission metadata or docket ID
        let court_id = submission.metadata.get("court_id")
            .and_then(|v| v.as_str())
            .or_else(|| submission.docket_id.as_deref())
            .ok_or_else(|| ProviderError::Configuration("Court ID required for county filing".to_string()))?;

        let county_config = self.get_county_from_court_id(court_id)
            .ok_or_else(|| ProviderError::Configuration(format!("Unknown court: {}", court_id)))?;

        // Upload documents
        let mut documents = Vec::new();
        for file_path in &submission.files {
            let content = tokio::fs::read(file_path).await.map_err(|e| {
                ProviderError::Configuration(format!("Failed to read file {}: {}", file_path, e))
            })?;

            let encoded_content = base64::encode(&content);
            let content_type = match std::path::Path::new(file_path)
                .extension()
                .and_then(|ext| ext.to_str())
            {
                Some("pdf") => "application/pdf",
                Some("doc") => "application/msword",
                Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                _ => "application/octet-stream",
            };

            let filename = std::path::Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("document")
                .to_string();

            documents.push(CountyDocument {
                filename: filename.clone(),
                content_type: content_type.to_string(),
                content: encoded_content,
                description: filename,
            });
        }

        let filing_request = CountySubmissionRequest {
            case_number: submission.docket_id.clone(),
            document_type: submission.document_type.clone(),
            filing_party: submission.metadata.get("filing_party")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            documents,
            metadata: submission.metadata.clone(),
        };

        let submit_endpoint = county_config.endpoints.get("submit")
            .ok_or_else(|| ProviderError::Configuration("Submit endpoint not configured".to_string()))?;

        let url = format!("{}{}", county_config.base_url, submit_endpoint);
        let response: CountySubmissionResponse = self.client.post_json(&url, &filing_request).await?;

        info!("County filing submitted successfully: {}", response.submission_id);
        Ok(response.submission_id)
    }

    #[instrument(skip(self, submission_id))]
    async fn get_status(&self, submission_id: &str) -> Result<EFilingSubmission, ProviderError> {
        info!("Checking county filing status: {}", submission_id);

        // For status checking, we need to determine which county this submission belongs to
        // This would typically be stored in the database or passed as context
        // For now, we'll try Philadelphia first, then Allegheny

        let mut last_error = None;

        for (county_name, county_config) in &self.county_configs {
            let status_endpoint = county_config.endpoints.get("status")
                .ok_or_else(|| ProviderError::Configuration("Status endpoint not configured".to_string()))?;

            let url = format!("{}{}", county_config.base_url, status_endpoint.replace("{id}", submission_id));

            match self.client.get_json::<CountyStatusResponse>(&url).await {
                Ok(response) => {
                    let status = match response.status.as_str() {
                        "pending" | "submitted" => SubmissionStatus::Pending,
                        "accepted" | "filed" => SubmissionStatus::Accepted,
                        "rejected" | "denied" => SubmissionStatus::Rejected,
                        "error" | "failed" => SubmissionStatus::Error,
                        _ => SubmissionStatus::Pending,
                    };

                    let submission = EFilingSubmission {
                        id: Uuid::parse_str(submission_id).unwrap_or_else(|_| Uuid::new_v4()),
                        session_id: Uuid::new_v4(),
                        docket_id: None,
                        document_type: "unknown".to_string(),
                        files: vec![],
                        metadata: HashMap::new(),
                        status,
                        submission_id: Some(response.submission_id),
                        receipt_path: None,
                        error_message: if response.messages.is_empty() {
                            None
                        } else {
                            Some(response.messages.join("; "))
                        },
                        submitted_at: Some(Utc::now()),
                        processed_at: Some(response.last_updated),
                    };

                    return Ok(submission);
                },
                Err(e) => {
                    warn!("Failed to check status with {}: {}", county_name, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ProviderError::ServiceUnavailable("No county found for submission".to_string())))
    }

    #[instrument(skip(self, session))]
    async fn refresh_token(&self, session: &EFilingSession) -> Result<EFilingSession, ProviderError> {
        info!("Refreshing county e-filing token: {}", session.id);

        let county_config = self.get_county_from_court_id(&session.court_id)
            .ok_or_else(|| ProviderError::Configuration(format!("Unknown court: {}", session.court_id)))?;

        match &county_config.auth_type {
            AuthType::OAuth2 { token_url, .. } => {
                let refresh_token = session.refresh_token.as_ref()
                    .ok_or_else(|| ProviderError::AuthenticationFailed("No refresh token available".to_string()))?;

                let mut body = HashMap::new();
                body.insert("grant_type", "refresh_token");
                body.insert("refresh_token", refresh_token);

                let url = format!("{}{}", county_config.base_url, token_url);
                let response: CountyAuthResponse = self.client.post_json(&url, &body).await?;

                let expires_at = if let Some(expires_in) = response.expires_in {
                    Utc::now() + chrono::Duration::seconds(expires_in as i64)
                } else {
                    Utc::now() + chrono::Duration::hours(24)
                };

                let mut new_session = session.clone();
                new_session.token = response.token;
                new_session.refresh_token = response.refresh_token;
                new_session.expires_at = expires_at;

                Ok(new_session)
            },
            _ => {
                // For other auth types, return the existing session
                Ok(session.clone())
            }
        }
    }
}
