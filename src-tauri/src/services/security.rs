// Security and credentials service for PA eDocket Desktop

use anyhow::{Context, Result};
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub id: String,
    pub service: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    pub enforce_https: bool,
    pub require_certificate_validation: bool,
    pub session_timeout_minutes: u32,
    pub max_failed_attempts: u32,
    pub password_min_length: u32,
    pub require_mfa: bool,
}

impl Default for SecureConfig {
    fn default() -> Self {
        Self {
            enforce_https: true,
            require_certificate_validation: true,
            session_timeout_minutes: 30,
            max_failed_attempts: 3,
            password_min_length: 8,
            require_mfa: false,
        }
    }
}

pub struct SecurityService {
    app_name: String,
    config: SecureConfig,
    session_tokens: HashMap<String, SessionInfo>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    user_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
    metadata: HashMap<String, String>,
}

impl SecurityService {
    pub fn new(app_name: String) -> Self {
        Self {
            app_name,
            config: SecureConfig::default(),
            session_tokens: HashMap::new(),
        }
    }

    pub fn with_config(app_name: String, config: SecureConfig) -> Self {
        Self {
            app_name,
            config,
            session_tokens: HashMap::new(),
        }
    }

    // Credential Management
    #[instrument(skip(self, password))]
    pub async fn store_credential(
        &self,
        service: &str,
        username: &str,
        password: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        info!("Storing credential for service: {}, username: {}", service, username);

        let credential_id = Uuid::new_v4().to_string();
        let keyring_service = format!("{}:{}", self.app_name, service);
        let keyring_username = format!("{}:{}", username, credential_id);

        // Store password in OS keychain
        let entry = Entry::new(&keyring_service, &keyring_username)
            .context("Failed to create keyring entry")?;
        
        entry.set_password(password)
            .context("Failed to store password in keychain")?;

        // Store metadata separately (encrypted)
        let credential_entry = CredentialEntry {
            id: credential_id.clone(),
            service: service.to_string(),
            username: username.to_string(),
            created_at: chrono::Utc::now(),
            last_used: None,
            metadata: metadata.unwrap_or_default(),
        };

        let metadata_json = serde_json::to_string(&credential_entry)?;
        let metadata_key = format!("{}:metadata", keyring_username);
        let metadata_entry = Entry::new(&keyring_service, &metadata_key)
            .context("Failed to create metadata keyring entry")?;
        
        metadata_entry.set_password(&metadata_json)
            .context("Failed to store metadata in keychain")?;

        info!("Credential stored successfully with ID: {}", credential_id);
        Ok(credential_id)
    }

    #[instrument(skip(self))]
    pub async fn retrieve_credential(&mut self, credential_id: &str) -> Result<(String, String)> {
        debug!("Retrieving credential: {}", credential_id);

        // Find the credential by searching through stored entries
        let (service, username) = self.find_credential_info(credential_id).await?;
        
        let keyring_service = format!("{}:{}", self.app_name, service);
        let keyring_username = format!("{}:{}", username, credential_id);

        let entry = Entry::new(&keyring_service, &keyring_username)
            .context("Failed to create keyring entry")?;
        
        let password = entry.get_password()
            .context("Failed to retrieve password from keychain")?;

        // Update last used timestamp
        self.update_credential_last_used(credential_id).await?;

        debug!("Credential retrieved successfully");
        Ok((username, password))
    }

    #[instrument(skip(self))]
    pub async fn list_credentials(&self) -> Result<Vec<CredentialEntry>> {
        info!("Listing stored credentials");
        
        // This is a simplified implementation
        // In a real implementation, you would need to enumerate keychain entries
        // For now, return empty list as keyring crate doesn't provide enumeration
        warn!("Credential enumeration not fully implemented");
        Ok(vec![])
    }

    #[instrument(skip(self))]
    pub async fn delete_credential(&self, credential_id: &str) -> Result<()> {
        info!("Deleting credential: {}", credential_id);

        let (service, username) = self.find_credential_info(credential_id).await?;
        
        let keyring_service = format!("{}:{}", self.app_name, service);
        let keyring_username = format!("{}:{}", username, credential_id);

        // Delete password
        let entry = Entry::new(&keyring_service, &keyring_username)
            .context("Failed to create keyring entry")?;
        
        entry.delete_password()
            .context("Failed to delete password from keychain")?;

        // Delete metadata
        let metadata_key = format!("{}:metadata", keyring_username);
        let metadata_entry = Entry::new(&keyring_service, &metadata_key)
            .context("Failed to create metadata keyring entry")?;
        
        metadata_entry.delete_password()
            .context("Failed to delete metadata from keychain")?;

        info!("Credential deleted successfully");
        Ok(())
    }

    // Session Management
    #[instrument(skip(self))]
    pub async fn create_session(&mut self, user_id: &str, metadata: Option<HashMap<String, String>>) -> Result<String> {
        info!("Creating session for user: {}", user_id);

        let session_token = self.generate_session_token();
        let session_info = SessionInfo {
            user_id: user_id.to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: metadata.unwrap_or_default(),
        };

        self.session_tokens.insert(session_token.clone(), session_info);
        
        info!("Session created with token: {}", &session_token[..8]);
        Ok(session_token)
    }

    #[instrument(skip(self))]
    pub async fn validate_session(&mut self, session_token: &str) -> Result<bool> {
        debug!("Validating session token: {}", &session_token[..8]);

        if let Some(session_info) = self.session_tokens.get_mut(session_token) {
            let now = chrono::Utc::now();
            let session_age = now.signed_duration_since(session_info.created_at);
            
            if session_age.num_minutes() > self.config.session_timeout_minutes as i64 {
                warn!("Session expired for token: {}", &session_token[..8]);
                self.session_tokens.remove(session_token);
                return Ok(false);
            }

            // Update last activity
            session_info.last_activity = now;
            debug!("Session validated successfully");
            Ok(true)
        } else {
            warn!("Invalid session token: {}", &session_token[..8]);
            Ok(false)
        }
    }

    #[instrument(skip(self))]
    pub async fn invalidate_session(&mut self, session_token: &str) -> Result<()> {
        info!("Invalidating session token: {}", &session_token[..8]);
        self.session_tokens.remove(session_token);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn cleanup_expired_sessions(&mut self) -> Result<usize> {
        debug!("Cleaning up expired sessions");

        let now = chrono::Utc::now();
        let timeout_minutes = self.config.session_timeout_minutes as i64;
        
        let expired_tokens: Vec<String> = self.session_tokens
            .iter()
            .filter(|(_, session)| {
                now.signed_duration_since(session.created_at).num_minutes() > timeout_minutes
            })
            .map(|(token, _)| token.clone())
            .collect();

        let count = expired_tokens.len();
        for token in expired_tokens {
            self.session_tokens.remove(&token);
        }

        if count > 0 {
            info!("Cleaned up {} expired sessions", count);
        }
        
        Ok(count)
    }

    // Security Utilities
    pub fn hash_password(&self, password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_salt(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < self.config.password_min_length as usize {
            return Err(anyhow::anyhow!(
                "Password must be at least {} characters long",
                self.config.password_min_length
            ));
        }

        // Add more password strength checks as needed
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if !has_upper || !has_lower || !has_digit || !has_special {
            return Err(anyhow::anyhow!(
                "Password must contain uppercase, lowercase, digit, and special characters"
            ));
        }

        Ok(())
    }

    pub fn validate_url_security(&self, url: &str) -> Result<()> {
        if self.config.enforce_https && !url.starts_with("https://") {
            return Err(anyhow::anyhow!("HTTPS is required for all connections"));
        }
        Ok(())
    }

    // Private helper methods
    fn generate_session_token(&self) -> String {
        format!("{}-{}", Uuid::new_v4(), chrono::Utc::now().timestamp())
    }

    async fn find_credential_info(&self, credential_id: &str) -> Result<(String, String)> {
        // This is a simplified implementation
        // In a real implementation, you would search through keychain entries
        // For now, return a placeholder
        Err(anyhow::anyhow!("Credential lookup not fully implemented"))
    }

    async fn update_credential_last_used(&self, credential_id: &str) -> Result<()> {
        // Update the metadata with last used timestamp
        // This would require retrieving, updating, and storing the metadata
        debug!("Updating last used timestamp for credential: {}", credential_id);
        Ok(())
    }
}
