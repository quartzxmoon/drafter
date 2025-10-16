// Email Integration Service - Gmail and Outlook integration with matter linking
// Supports OAuth2 authentication, email syncing, and automatic case file organization

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailProvider {
    Gmail,
    Outlook,
    Exchange,
    IMAP,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailStatus {
    Unread,
    Read,
    Archived,
    Deleted,
    Draft,
    Sent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: String,
    pub provider: EmailProvider,
    pub email_address: String,
    pub display_name: String,

    // OAuth credentials
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: DateTime<Utc>,

    // Sync settings
    pub is_active: bool,
    pub sync_enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_from_date: Option<DateTime<Utc>>,

    // Settings
    pub auto_file_emails: bool,
    pub auto_link_to_matters: bool,
    pub signature: Option<String>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub thread_id: Option<String>,

    // Email headers
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub reply_to: Option<EmailAddress>,

    // Content
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub snippet: Option<String>,

    // Metadata
    pub date: DateTime<Utc>,
    pub status: EmailStatus,
    pub is_important: bool,
    pub has_attachments: bool,
    pub labels: Vec<String>,

    // Matter linking
    pub matter_id: Option<String>,
    pub matter_name: Option<String>,
    pub is_client_communication: bool,
    pub confidence_score: Option<f64>,  // For auto-linking confidence

    // Attachments
    pub attachments: Vec<EmailAttachment>,

    // Sync
    pub synced_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub content_id: Option<String>,
    pub provider_attachment_id: String,
    pub is_inline: bool,
    pub downloaded: bool,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailThread {
    pub id: String,
    pub account_id: String,
    pub provider_thread_id: String,
    pub subject: String,
    pub participants: Vec<EmailAddress>,
    pub message_count: u32,
    pub messages: Vec<Email>,
    pub matter_id: Option<String>,
    pub first_message_date: DateTime<Utc>,
    pub last_message_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDraft {
    pub id: String,
    pub account_id: String,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body_html: String,
    pub attachments: Vec<EmailAttachment>,
    pub matter_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub category: EmailTemplateCategory,
    pub subject: String,
    pub body_html: String,
    pub variables: Vec<String>,
    pub attachments: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailTemplateCategory {
    ClientCommunication,
    CourtCorrespondence,
    OpposingCounsel,
    Internal,
    StatusUpdate,
    Reminder,
    Invoice,
    Welcome,
    FollowUp,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRule {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub is_active: bool,

    // Conditions
    pub from_contains: Option<String>,
    pub to_contains: Option<String>,
    pub subject_contains: Option<String>,
    pub body_contains: Option<String>,
    pub has_attachments: Option<bool>,

    // Actions
    pub link_to_matter_id: Option<String>,
    pub add_labels: Vec<String>,
    pub mark_as_important: Option<bool>,
    pub auto_file: Option<bool>,
    pub forward_to: Option<String>,

    // Stats
    pub matches_count: u32,
    pub last_matched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSearchQuery {
    pub account_id: Option<String>,
    pub query: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub matter_id: Option<String>,
    pub has_attachments: Option<bool>,
    pub status: Option<EmailStatus>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub labels: Option<Vec<String>>,
    pub is_important: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub struct EmailIntegrationService {
    db: SqlitePool,
}

impl EmailIntegrationService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ============= Account Management =============

    /// Connect Gmail account via OAuth2
    pub async fn connect_gmail_account(
        &self,
        email_address: &str,
        display_name: &str,
        access_token: &str,
        refresh_token: &str,
        expires_in_seconds: i64,
    ) -> Result<EmailAccount> {
        let account_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let token_expires_at = now + chrono::Duration::seconds(expires_in_seconds);

        let account = EmailAccount {
            id: account_id,
            provider: EmailProvider::Gmail,
            email_address: email_address.to_string(),
            display_name: display_name.to_string(),
            access_token: access_token.to_string(),
            refresh_token: Some(refresh_token.to_string()),
            token_expires_at,
            is_active: true,
            sync_enabled: true,
            last_sync_at: None,
            sync_from_date: Some(now - chrono::Duration::days(30)), // Sync last 30 days by default
            auto_file_emails: true,
            auto_link_to_matters: true,
            signature: None,
            created_at: now,
            updated_at: now,
        };

        self.save_email_account(&account).await?;

        Ok(account)
    }

    /// Connect Outlook account via OAuth2
    pub async fn connect_outlook_account(
        &self,
        email_address: &str,
        display_name: &str,
        access_token: &str,
        refresh_token: &str,
        expires_in_seconds: i64,
    ) -> Result<EmailAccount> {
        let account_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let token_expires_at = now + chrono::Duration::seconds(expires_in_seconds);

        let account = EmailAccount {
            id: account_id,
            provider: EmailProvider::Outlook,
            email_address: email_address.to_string(),
            display_name: display_name.to_string(),
            access_token: access_token.to_string(),
            refresh_token: Some(refresh_token.to_string()),
            token_expires_at,
            is_active: true,
            sync_enabled: true,
            last_sync_at: None,
            sync_from_date: Some(now - chrono::Duration::days(30)),
            auto_file_emails: true,
            auto_link_to_matters: true,
            signature: None,
            created_at: now,
            updated_at: now,
        };

        self.save_email_account(&account).await?;

        Ok(account)
    }

    /// Refresh access token
    pub async fn refresh_access_token(&self, account_id: &str) -> Result<EmailAccount> {
        let mut account = self.get_email_account(account_id).await?;

        // Check if token is expired or will expire soon
        let now = Utc::now();
        if account.token_expires_at > now + chrono::Duration::minutes(5) {
            return Ok(account); // Token still valid
        }

        // Refresh token based on provider
        match account.provider {
            EmailProvider::Gmail => {
                let (new_access_token, expires_in) = self.refresh_gmail_token(&account).await?;
                account.access_token = new_access_token;
                account.token_expires_at = now + chrono::Duration::seconds(expires_in);
            }
            EmailProvider::Outlook => {
                let (new_access_token, expires_in) = self.refresh_outlook_token(&account).await?;
                account.access_token = new_access_token;
                account.token_expires_at = now + chrono::Duration::seconds(expires_in);
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported provider for token refresh"));
            }
        }

        account.updated_at = now;
        self.save_email_account(&account).await?;

        Ok(account)
    }

    async fn refresh_gmail_token(&self, account: &EmailAccount) -> Result<(String, i64)> {
        // Stub - would call Google OAuth2 token refresh endpoint
        Ok(("new_access_token".to_string(), 3600))
    }

    async fn refresh_outlook_token(&self, account: &EmailAccount) -> Result<(String, i64)> {
        // Stub - would call Microsoft OAuth2 token refresh endpoint
        Ok(("new_access_token".to_string(), 3600))
    }

    /// Disconnect email account
    pub async fn disconnect_account(&self, account_id: &str) -> Result<()> {
        let mut account = self.get_email_account(account_id).await?;
        account.is_active = false;
        account.sync_enabled = false;
        account.updated_at = Utc::now();

        self.save_email_account(&account).await?;

        Ok(())
    }

    // ============= Email Syncing =============

    /// Sync emails from provider
    pub async fn sync_emails(&self, account_id: &str) -> Result<u32> {
        let mut account = self.get_email_account(account_id).await?;

        if !account.sync_enabled {
            return Ok(0);
        }

        // Refresh token if needed
        account = self.refresh_access_token(&account.id).await?;

        let sync_count = match account.provider {
            EmailProvider::Gmail => self.sync_gmail_emails(&account).await?,
            EmailProvider::Outlook => self.sync_outlook_emails(&account).await?,
            _ => 0,
        };

        // Update last sync time
        account.last_sync_at = Some(Utc::now());
        account.updated_at = Utc::now();
        self.save_email_account(&account).await?;

        Ok(sync_count)
    }

    async fn sync_gmail_emails(&self, account: &EmailAccount) -> Result<u32> {
        // Stub - would call Gmail API to fetch messages
        // GET https://gmail.googleapis.com/gmail/v1/users/me/messages

        let mock_emails = vec![];

        for email in mock_emails {
            self.save_email(&email).await?;

            // Auto-link to matters if enabled
            if account.auto_link_to_matters {
                self.auto_link_email_to_matter(&email).await?;
            }

            // Apply email rules
            self.apply_email_rules(&email).await?;
        }

        Ok(mock_emails.len() as u32)
    }

    async fn sync_outlook_emails(&self, account: &EmailAccount) -> Result<u32> {
        // Stub - would call Microsoft Graph API to fetch messages
        // GET https://graph.microsoft.com/v1.0/me/messages

        Ok(0)
    }

    /// Download email attachment
    pub async fn download_attachment(
        &self,
        email_id: &str,
        attachment_id: &str,
        local_path: &str,
    ) -> Result<EmailAttachment> {
        let email = self.get_email(email_id).await?;
        let account = self.get_email_account(&email.account_id).await?;

        let mut attachment = email.attachments.iter()
            .find(|a| a.id == attachment_id)
            .ok_or_else(|| anyhow::anyhow!("Attachment not found"))?
            .clone();

        if attachment.downloaded {
            return Ok(attachment);
        }

        // Download based on provider
        match account.provider {
            EmailProvider::Gmail => {
                self.download_gmail_attachment(&account, &email, &attachment, local_path).await?;
            }
            EmailProvider::Outlook => {
                self.download_outlook_attachment(&account, &email, &attachment, local_path).await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported provider for attachment download"));
            }
        }

        attachment.downloaded = true;
        attachment.local_path = Some(local_path.to_string());

        // Update email with downloaded attachment
        // Would update in database

        Ok(attachment)
    }

    async fn download_gmail_attachment(
        &self,
        account: &EmailAccount,
        email: &Email,
        attachment: &EmailAttachment,
        local_path: &str,
    ) -> Result<()> {
        // Stub - would call Gmail API
        // GET https://gmail.googleapis.com/gmail/v1/users/me/messages/{messageId}/attachments/{attachmentId}
        Ok(())
    }

    async fn download_outlook_attachment(
        &self,
        account: &EmailAccount,
        email: &Email,
        attachment: &EmailAttachment,
        local_path: &str,
    ) -> Result<()> {
        // Stub - would call Microsoft Graph API
        // GET https://graph.microsoft.com/v1.0/me/messages/{messageId}/attachments/{attachmentId}/$value
        Ok(())
    }

    // ============= Matter Linking =============

    /// Automatically link email to matter based on content analysis
    pub async fn auto_link_email_to_matter(&self, email: &Email) -> Result<Option<String>> {
        // Extract potential matter references from email
        let candidates = self.find_matter_candidates(email).await?;

        if candidates.is_empty() {
            return Ok(None);
        }

        // Score each candidate
        let mut scored_candidates: Vec<(String, f64)> = Vec::new();

        for matter_id in candidates {
            let score = self.calculate_linking_score(email, &matter_id).await?;
            scored_candidates.push((matter_id, score));
        }

        // Sort by score descending
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Use top candidate if confidence is high enough
        if let Some((matter_id, score)) = scored_candidates.first() {
            if *score > 0.7 {
                self.link_email_to_matter(&email.id, matter_id, Some(*score)).await?;
                return Ok(Some(matter_id.clone()));
            }
        }

        Ok(None)
    }

    async fn find_matter_candidates(&self, email: &Email) -> Result<Vec<String>> {
        let mut candidates = Vec::new();

        // Check sender/recipients against matter parties
        let addresses: Vec<String> = std::iter::once(&email.from)
            .chain(email.to.iter())
            .chain(email.cc.iter())
            .map(|addr| addr.address.clone())
            .collect();

        for address in addresses {
            let matters = self.find_matters_by_email(&address).await?;
            candidates.extend(matters);
        }

        // Check subject line for docket numbers or case names
        if let Some(matter_id) = self.extract_matter_from_subject(&email.subject).await? {
            candidates.push(matter_id);
        }

        // Check body for case references
        if let Some(body) = &email.body_text {
            if let Some(matter_id) = self.extract_matter_from_body(body).await? {
                candidates.push(matter_id);
            }
        }

        // Deduplicate
        candidates.sort();
        candidates.dedup();

        Ok(candidates)
    }

    async fn calculate_linking_score(&self, email: &Email, matter_id: &str) -> Result<f64> {
        let mut score = 0.0;

        // Sender matches matter party (0.4 weight)
        if self.email_matches_matter_party(&email.from.address, matter_id).await? {
            score += 0.4;
        }

        // Any recipient matches matter party (0.3 weight)
        for addr in email.to.iter().chain(email.cc.iter()) {
            if self.email_matches_matter_party(&addr.address, matter_id).await? {
                score += 0.3;
                break;
            }
        }

        // Subject contains matter reference (0.2 weight)
        if self.subject_references_matter(&email.subject, matter_id).await? {
            score += 0.2;
        }

        // Body contains matter reference (0.1 weight)
        if let Some(body) = &email.body_text {
            if self.body_references_matter(body, matter_id).await? {
                score += 0.1;
            }
        }

        Ok(score)
    }

    /// Manually link email to matter
    pub async fn link_email_to_matter(
        &self,
        email_id: &str,
        matter_id: &str,
        confidence: Option<f64>,
    ) -> Result<()> {
        let mut email = self.get_email(email_id).await?;
        let matter_name = self.get_matter_name(matter_id).await?;

        email.matter_id = Some(matter_id.to_string());
        email.matter_name = Some(matter_name);
        email.confidence_score = confidence;

        self.save_email(&email).await?;

        Ok(())
    }

    /// Unlink email from matter
    pub async fn unlink_email_from_matter(&self, email_id: &str) -> Result<()> {
        let mut email = self.get_email(email_id).await?;

        email.matter_id = None;
        email.matter_name = None;
        email.confidence_score = None;

        self.save_email(&email).await?;

        Ok(())
    }

    // ============= Email Rules =============

    /// Create email rule
    pub async fn create_email_rule(&self, rule: EmailRule) -> Result<EmailRule> {
        self.save_email_rule(&rule).await?;
        Ok(rule)
    }

    /// Apply email rules to email
    async fn apply_email_rules(&self, email: &Email) -> Result<()> {
        let rules = self.get_active_rules_for_account(&email.account_id).await?;

        for rule in rules {
            if self.rule_matches_email(&rule, email) {
                self.execute_rule_actions(&rule, email).await?;

                // Update rule match count
                self.increment_rule_matches(&rule.id).await?;
            }
        }

        Ok(())
    }

    fn rule_matches_email(&self, rule: &EmailRule, email: &Email) -> bool {
        if let Some(from_filter) = &rule.from_contains {
            if !email.from.address.to_lowercase().contains(&from_filter.to_lowercase()) {
                return false;
            }
        }

        if let Some(subject_filter) = &rule.subject_contains {
            if !email.subject.to_lowercase().contains(&subject_filter.to_lowercase()) {
                return false;
            }
        }

        if let Some(has_attachments) = rule.has_attachments {
            if email.has_attachments != has_attachments {
                return false;
            }
        }

        true
    }

    async fn execute_rule_actions(&self, rule: &EmailRule, email: &Email) -> Result<()> {
        // Link to matter
        if let Some(matter_id) = &rule.link_to_matter_id {
            self.link_email_to_matter(&email.id, matter_id, Some(1.0)).await?;
        }

        // Add labels
        if !rule.add_labels.is_empty() {
            // Would update email labels
        }

        // Mark as important
        if let Some(true) = rule.mark_as_important {
            // Would update email importance
        }

        Ok(())
    }

    // ============= Sending Emails =============

    /// Create draft email
    pub async fn create_draft(
        &self,
        account_id: &str,
        to: Vec<EmailAddress>,
        subject: &str,
        body_html: &str,
        matter_id: Option<String>,
    ) -> Result<EmailDraft> {
        let draft_id = Uuid::new_v4().to_string();

        let draft = EmailDraft {
            id: draft_id,
            account_id: account_id.to_string(),
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: subject.to_string(),
            body_html: body_html.to_string(),
            attachments: Vec::new(),
            matter_id,
            in_reply_to: None,
            references: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.save_draft(&draft).await?;

        Ok(draft)
    }

    /// Send email
    pub async fn send_email(&self, draft_id: &str) -> Result<Email> {
        let draft = self.get_draft(draft_id).await?;
        let account = self.get_email_account(&draft.account_id).await?;

        // Send based on provider
        let provider_message_id = match account.provider {
            EmailProvider::Gmail => self.send_gmail_email(&account, &draft).await?,
            EmailProvider::Outlook => self.send_outlook_email(&account, &draft).await?,
            _ => {
                return Err(anyhow::anyhow!("Unsupported provider for sending"));
            }
        };

        // Create email record
        let email = Email {
            id: Uuid::new_v4().to_string(),
            account_id: draft.account_id.clone(),
            provider_message_id,
            thread_id: None,
            from: EmailAddress {
                name: Some(account.display_name.clone()),
                address: account.email_address.clone(),
            },
            to: draft.to.clone(),
            cc: draft.cc.clone(),
            bcc: draft.bcc.clone(),
            reply_to: None,
            subject: draft.subject.clone(),
            body_text: None,
            body_html: Some(draft.body_html.clone()),
            snippet: None,
            date: Utc::now(),
            status: EmailStatus::Sent,
            is_important: false,
            has_attachments: !draft.attachments.is_empty(),
            labels: Vec::new(),
            matter_id: draft.matter_id.clone(),
            matter_name: None,
            is_client_communication: false,
            confidence_score: None,
            attachments: draft.attachments.clone(),
            synced_at: Utc::now(),
            is_deleted: false,
        };

        self.save_email(&email).await?;

        // Delete draft
        self.delete_draft(draft_id).await?;

        Ok(email)
    }

    async fn send_gmail_email(&self, account: &EmailAccount, draft: &EmailDraft) -> Result<String> {
        // Stub - would call Gmail API
        // POST https://gmail.googleapis.com/gmail/v1/users/me/messages/send
        Ok(format!("gmail_{}", Uuid::new_v4()))
    }

    async fn send_outlook_email(&self, account: &EmailAccount, draft: &EmailDraft) -> Result<String> {
        // Stub - would call Microsoft Graph API
        // POST https://graph.microsoft.com/v1.0/me/sendMail
        Ok(format!("outlook_{}", Uuid::new_v4()))
    }

    // ============= Email Templates =============

    /// Create email template
    pub async fn create_template(&self, template: EmailTemplate) -> Result<EmailTemplate> {
        self.save_template(&template).await?;
        Ok(template)
    }

    /// Apply template to draft
    pub async fn apply_template_to_draft(
        &self,
        draft_id: &str,
        template_id: &str,
        variables: HashMap<String, String>,
    ) -> Result<EmailDraft> {
        let mut draft = self.get_draft(draft_id).await?;
        let template = self.get_template(template_id).await?;

        // Replace variables in subject
        let mut subject = template.subject.clone();
        for (key, value) in &variables {
            subject = subject.replace(&format!("{{{{{}}}}}", key), value);
        }
        draft.subject = subject;

        // Replace variables in body
        let mut body = template.body_html.clone();
        for (key, value) in &variables {
            body = body.replace(&format!("{{{{{}}}}}", key), value);
        }
        draft.body_html = body;

        draft.updated_at = Utc::now();

        self.save_draft(&draft).await?;

        Ok(draft)
    }

    // ============= Search =============

    /// Search emails
    pub async fn search_emails(&self, query: EmailSearchQuery) -> Result<Vec<Email>> {
        // Stub - would query database with filters
        Ok(Vec::new())
    }

    // ============= Helper Methods =============

    async fn save_email_account(&self, account: &EmailAccount) -> Result<()> {
        let provider_str = format!("{:?}", account.provider);

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO email_accounts
            (id, provider, email_address, display_name, access_token, refresh_token,
             token_expires_at, is_active, sync_enabled, last_sync_at, sync_from_date,
             auto_file_emails, auto_link_to_matters, signature, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            account.id,
            provider_str,
            account.email_address,
            account.display_name,
            account.access_token,
            account.refresh_token,
            account.token_expires_at,
            account.is_active,
            account.sync_enabled,
            account.last_sync_at,
            account.sync_from_date,
            account.auto_file_emails,
            account.auto_link_to_matters,
            account.signature,
            account.created_at,
            account.updated_at
        )
        .execute(&self.db)
        .await
        .context("Failed to save email account")?;

        Ok(())
    }

    async fn get_email_account(&self, account_id: &str) -> Result<EmailAccount> {
        // Stub - would query database
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn save_email(&self, email: &Email) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn get_email(&self, email_id: &str) -> Result<Email> {
        // Stub - would query database
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn save_email_rule(&self, rule: &EmailRule) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn get_active_rules_for_account(&self, account_id: &str) -> Result<Vec<EmailRule>> {
        // Stub - would query database
        Ok(Vec::new())
    }

    async fn increment_rule_matches(&self, rule_id: &str) -> Result<()> {
        // Stub - would update database
        Ok(())
    }

    async fn save_draft(&self, draft: &EmailDraft) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn get_draft(&self, draft_id: &str) -> Result<EmailDraft> {
        // Stub - would query database
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn delete_draft(&self, draft_id: &str) -> Result<()> {
        // Stub - would delete from database
        Ok(())
    }

    async fn save_template(&self, template: &EmailTemplate) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn get_template(&self, template_id: &str) -> Result<EmailTemplate> {
        // Stub - would query database
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn find_matters_by_email(&self, email_address: &str) -> Result<Vec<String>> {
        // Stub - would query matters and parties
        Ok(Vec::new())
    }

    async fn extract_matter_from_subject(&self, subject: &str) -> Result<Option<String>> {
        // Stub - would parse subject for case references
        Ok(None)
    }

    async fn extract_matter_from_body(&self, body: &str) -> Result<Option<String>> {
        // Stub - would parse body for case references
        Ok(None)
    }

    async fn email_matches_matter_party(&self, email: &str, matter_id: &str) -> Result<bool> {
        // Stub - would check if email matches any party in matter
        Ok(false)
    }

    async fn subject_references_matter(&self, subject: &str, matter_id: &str) -> Result<bool> {
        // Stub - would check if subject references matter
        Ok(false)
    }

    async fn body_references_matter(&self, body: &str, matter_id: &str) -> Result<bool> {
        // Stub - would check if body references matter
        Ok(false)
    }

    async fn get_matter_name(&self, matter_id: &str) -> Result<String> {
        // Stub - would query matters table
        Ok(format!("Matter {}", matter_id))
    }
}
