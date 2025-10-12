// PACFile E-filing Provider
// Production-ready integration with Pennsylvania's PACFile system

use crate::domain::*;
use crate::providers::{
    client::ProviderClient, EFilingProvider, ProviderConfig, ProviderError, ProviderResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct PacFileAuthRequest {
    username: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileAuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    token_type: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfa_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileCapabilityResponse {
    court_id: String,
    enabled: bool,
    document_types: Vec<String>,
    max_file_size: u64,
    allowed_formats: Vec<String>,
    requires_cover_sheet: bool,
    supports_electronic_service: bool,
    fee_calculation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileSubmissionRequest {
    court_id: String,
    case_id: Option<String>,
    document_type: String,
    filing_party: String,
    documents: Vec<PacFileDocument>,
    metadata: HashMap<String, serde_json::Value>,
    electronic_service: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileDocument {
    filename: String,
    content_type: String,
    content: String, // Base64 encoded
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileSubmissionResponse {
    submission_id: String,
    status: String,
    confirmation_number: String,
    filing_date: DateTime<Utc>,
    fees: Vec<PacFileFee>,
    receipt_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileFee {
    description: String,
    amount: f64,
    waived: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct PacFileStatusResponse {
    submission_id: String,
    status: String,
    last_updated: DateTime<Utc>,
    messages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmation_number: Option<String>,
}

pub struct PacFileProvider {
    client: ProviderClient,
    config: ProviderConfig,
}

impl PacFileProvider {
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let client = ProviderClient::new(config.clone())?;
        
        Ok(Self { client, config })
    }
    
    #[instrument(skip(self, refresh_token))]
    async fn refresh_access_token(&self, refresh_token: &str) -> ProviderResult<PacFileAuthResponse> {
        info!("Refreshing PACFile access token");
        
        let mut body = HashMap::new();
        body.insert("grant_type", "refresh_token");
        body.insert("refresh_token", refresh_token);
        
        let url = format!("{}/api/auth/refresh", self.config.base_url);
        let response: PacFileAuthResponse = self.client.post_json(&url, &body).await?;
        
        debug!("Token refresh successful, expires in {} seconds", response.expires_in);
        Ok(response)
    }
    
    #[instrument(skip(self, files))]
    async fn upload_documents(&self, files: &[String]) -> ProviderResult<Vec<PacFileDocument>> {
        let mut documents = Vec::new();
        
        for file_path in files {
            info!("Processing file: {}", file_path);
            
            // Read file content
            let content = tokio::fs::read(file_path).await.map_err(|e| {
                ProviderError::Configuration(format!("Failed to read file {}: {}", file_path, e))
            })?;
            
            // Encode as base64
            let encoded_content = base64::encode(&content);
            
            // Determine content type
            let content_type = match std::path::Path::new(file_path)
                .extension()
                .and_then(|ext| ext.to_str())
            {
                Some("pdf") => "application/pdf",
                Some("doc") => "application/msword",
                Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                Some("txt") => "text/plain",
                _ => "application/octet-stream",
            };
            
            let filename = std::path::Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("document")
                .to_string();
            
            let document = PacFileDocument {
                filename: filename.clone(),
                content_type: content_type.to_string(),
                content: encoded_content,
                description: filename,
            };
            
            documents.push(document);
        }
        
        Ok(documents)
    }
    
    #[instrument(skip(self, submission_id))]
    async fn download_receipt(&self, submission_id: &str, receipt_url: &str) -> ProviderResult<String> {
        info!("Downloading receipt for submission: {}", submission_id);
        
        let receipt_content = self.client.get_text(receipt_url).await?;
        
        // Save receipt to temporary file
        let receipt_path = format!("/tmp/pacfile_receipt_{}.pdf", submission_id);
        tokio::fs::write(&receipt_path, receipt_content).await.map_err(|e| {
            ProviderError::Configuration(format!("Failed to save receipt: {}", e))
        })?;
        
        Ok(receipt_path)
    }
}

#[async_trait]
impl EFilingProvider for PacFileProvider {
    #[instrument(skip(self, court_id))]
    async fn get_capabilities(&self, court_id: &str) -> Result<Vec<EFilingCapability>, ProviderError> {
        info!("Fetching PACFile capabilities for court: {}", court_id);
        
        let url = format!("{}/api/courts/{}/capabilities", self.config.base_url, court_id);
        let response: PacFileCapabilityResponse = self.client.get_json(&url).await?;
        
        let capability = EFilingCapability {
            court_id: response.court_id,
            enabled: response.enabled,
            provider: "pacfile".to_string(),
            document_types: response.document_types,
            max_file_size: response.max_file_size,
            allowed_formats: response.allowed_formats,
            requires_cover_sheet: response.requires_cover_sheet,
            supports_electronic_service: response.supports_electronic_service,
            fee_calculation: response.fee_calculation,
        };
        
        Ok(vec![capability])
    }
    
    #[instrument(skip(self, credentials))]
    async fn authenticate(&self, credentials: HashMap<String, String>) -> Result<EFilingSession, ProviderError> {
        info!("Authenticating with PACFile");
        
        let username = credentials
            .get("username")
            .ok_or_else(|| ProviderError::AuthenticationFailed("Username required".to_string()))?;
        let password = credentials
            .get("password")
            .ok_or_else(|| ProviderError::AuthenticationFailed("Password required".to_string()))?;
        let mfa_code = credentials.get("mfa_code").cloned();

        let auth_request = PacFileAuthRequest {
            username: username.clone(),
            password: password.clone(),
            mfa_code: mfa_code.clone(),
        };

        let url = format!("{}/api/auth/login", self.config.base_url);
        let response: PacFileAuthResponse = self.client.post_json(&url, &auth_request).await?;

        if response.mfa_required.unwrap_or(false) && mfa_code.is_none() {
            return Err(ProviderError::AuthenticationFailed(
                "MFA code required".to_string(),
            )
            .into());
        }
        
        let expires_at = Utc::now() + chrono::Duration::seconds(response.expires_in as i64);
        
        let session = EFilingSession {
            id: Uuid::new_v4(),
            court_id: "pacfile".to_string(),
            provider: "pacfile".to_string(),
            token: response.access_token,
            refresh_token: Some(response.refresh_token),
            expires_at,
            user_id: Some(username.clone()),
            permissions: response.scope.split(' ').map(|s| s.to_string()).collect(),
        };
        
        info!("PACFile authentication successful for user: {}", username);
        Ok(session)
    }
    
    #[instrument(skip(self, submission))]
    async fn submit_filing(&self, submission: &EFilingSubmission) -> Result<String, ProviderError> {
        info!("Submitting filing to PACFile: {}", submission.id);
        
        // Upload documents
        let documents = self.upload_documents(&submission.files).await?;
        
        // Prepare submission request
        let filing_request = PacFileSubmissionRequest {
            court_id: submission.docket_id.clone().unwrap_or_default(),
            case_id: submission.docket_id.clone(),
            document_type: submission.document_type.clone(),
            filing_party: submission
                .metadata
                .get("filing_party")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            documents,
            metadata: submission.metadata.clone(),
            electronic_service: submission
                .metadata
                .get("electronic_service")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        
        let url = format!("{}/api/filing/submit", self.config.base_url);
        let response: PacFileSubmissionResponse = self.client.post_json(&url, &filing_request).await?;
        
        info!(
            "Filing submitted successfully: {} (confirmation: {})",
            response.submission_id, response.confirmation_number
        );
        
        Ok(response.submission_id)
    }
    
    #[instrument(skip(self, submission_id))]
    async fn get_status(&self, submission_id: &str) -> Result<EFilingSubmission, ProviderError> {
        info!("Checking PACFile submission status: {}", submission_id);
        
        let url = format!("{}/api/filing/{}/status", self.config.base_url, submission_id);
        let response: PacFileStatusResponse = self.client.get_json(&url).await?;
        
        let status = match response.status.as_str() {
            "pending" => SubmissionStatus::Pending,
            "submitted" => SubmissionStatus::Submitted,
            "accepted" => SubmissionStatus::Accepted,
            "rejected" => SubmissionStatus::Rejected,
            "error" => SubmissionStatus::Error,
            _ => SubmissionStatus::Pending,
        };
        
        let receipt_path = if let Some(receipt_url) = &response.receipt_url {
            Some(self.download_receipt(submission_id, receipt_url).await?)
        } else {
            None
        };
        
        let submission = EFilingSubmission {
            id: Uuid::parse_str(submission_id).unwrap_or_else(|_| Uuid::new_v4()),
            session_id: Uuid::new_v4(), // TODO: Get from context
            docket_id: None,
            document_type: "unknown".to_string(),
            files: vec![],
            metadata: HashMap::new(),
            status,
            submission_id: Some(response.submission_id),
            receipt_path,
            error_message: if response.messages.is_empty() {
                None
            } else {
                Some(response.messages.join("; "))
            },
            submitted_at: Some(Utc::now()),
            processed_at: Some(response.last_updated),
        };
        
        Ok(submission)
    }
    
    #[instrument(skip(self, session))]
    async fn refresh_token(&self, session: &EFilingSession) -> Result<EFilingSession, ProviderError> {
        info!("Refreshing PACFile token for session: {}", session.id);
        
        let refresh_token = session
            .refresh_token
            .as_ref()
            .ok_or_else(|| ProviderError::AuthenticationFailed("No refresh token available".to_string()))?;
        
        let response = self.refresh_access_token(refresh_token).await?;
        
        let expires_at = Utc::now() + chrono::Duration::seconds(response.expires_in as i64);
        
        let mut new_session = session.clone();
        new_session.token = response.access_token;
        new_session.refresh_token = Some(response.refresh_token);
        new_session.expires_at = expires_at;
        
        info!("Token refresh successful for session: {}", session.id);
        Ok(new_session)
    }
}
