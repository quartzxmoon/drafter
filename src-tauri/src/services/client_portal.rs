// Client Portal Service
// Secure document sharing and collaboration platform for clients

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{info, warn, error};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPortalUser {
    pub id: String,
    pub client_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub password_hash: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub two_factor_enabled: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSession {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDocument {
    pub id: String,
    pub document_id: String,
    pub matter_id: String,
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub shared_by: String,
    pub shared_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub download_limit: Option<u32>,
    pub downloads_count: u32,
    pub requires_signature: bool,
    pub signature_status: Option<SignatureStatus>,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SignatureStatus {
    Pending,
    Signed,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    View,
    Download,
    Comment,
    Sign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComment {
    pub id: String,
    pub document_id: String,
    pub user_id: String,
    pub user_name: String,
    pub comment: String,
    pub page_number: Option<u32>,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage {
    pub id: String,
    pub matter_id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub to_user_name: String,
    pub subject: String,
    pub body: String,
    pub encrypted: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub attachments: Vec<MessageAttachment>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: String,
    pub filename: String,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalActivity {
    pub id: String,
    pub user_id: String,
    pub activity_type: ActivityType,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub description: String,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Login,
    Logout,
    DocumentViewed,
    DocumentDownloaded,
    DocumentSigned,
    MessageSent,
    MessageRead,
    CommentAdded,
    ProfileUpdated,
    PasswordChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDashboard {
    pub client_id: String,
    pub matters: Vec<MatterSummary>,
    pub recent_documents: Vec<SharedDocument>,
    pub unread_messages: u32,
    pub pending_signatures: u32,
    pub upcoming_deadlines: Vec<DeadlineSummary>,
    pub recent_activity: Vec<PortalActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatterSummary {
    pub id: String,
    pub title: String,
    pub matter_number: String,
    pub status: String,
    pub docket_number: Option<String>,
    pub court_name: Option<String>,
    pub next_deadline: Option<DateTime<Utc>>,
    pub document_count: u32,
    pub unread_messages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlineSummary {
    pub id: String,
    pub matter_id: String,
    pub title: String,
    pub deadline_date: DateTime<Utc>,
    pub days_remaining: i64,
    pub priority: String,
}

pub struct ClientPortalService {
    db: SqlitePool,
}

impl ClientPortalService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create new portal user for a client
    pub async fn create_portal_user(
        &self,
        client_id: &str,
        email: &str,
        first_name: &str,
        last_name: &str,
        phone: Option<&str>,
        password: &str,
    ) -> Result<ClientPortalUser> {
        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();

        let user = ClientPortalUser {
            id: uuid::Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            email: email.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            phone: phone.map(|p| p.to_string()),
            password_hash,
            is_active: true,
            email_verified: false,
            two_factor_enabled: false,
            last_login: None,
            created_at: Utc::now(),
        };

        // Save to database
        sqlx::query!(
            r#"
            INSERT INTO portal_users (
                id, client_id, email, first_name, last_name, phone,
                password_hash, is_active, email_verified, two_factor_enabled,
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            user.id,
            user.client_id,
            user.email,
            user.first_name,
            user.last_name,
            user.phone,
            user.password_hash,
            user.is_active,
            user.email_verified,
            user.two_factor_enabled,
            user.created_at
        )
        .execute(&self.db)
        .await?;

        info!("Created portal user: {} for client: {}", user.email, client_id);
        Ok(user)
    }

    /// Authenticate user and create session
    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<PortalSession> {
        // Fetch user
        let user_record = sqlx::query!(
            r#"
            SELECT id, client_id, password_hash, is_active
            FROM portal_users
            WHERE email = ?
            "#,
            email
        )
        .fetch_one(&self.db)
        .await
        .map_err(|_| anyhow!("Invalid credentials"))?;

        if !user_record.is_active {
            return Err(anyhow!("Account is disabled"));
        }

        // Verify password
        let parsed_hash = PasswordHash::new(&user_record.password_hash)
            .map_err(|e| anyhow!("Invalid password hash: {}", e))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Invalid credentials"))?;

        // Create session
        let session = PortalSession {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_record.id.clone(),
            token: uuid::Uuid::new_v4().to_string(),
            expires_at: Utc::now() + Duration::hours(24),
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            created_at: Utc::now(),
        };

        // Save session
        sqlx::query!(
            r#"
            INSERT INTO portal_sessions (
                id, user_id, token, expires_at, ip_address, user_agent, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            session.id,
            session.user_id,
            session.token,
            session.expires_at,
            session.ip_address,
            session.user_agent,
            session.created_at
        )
        .execute(&self.db)
        .await?;

        // Update last login
        sqlx::query!(
            r#"
            UPDATE portal_users
            SET last_login = ?
            WHERE id = ?
            "#,
            Utc::now(),
            user_record.id
        )
        .execute(&self.db)
        .await?;

        // Log activity
        self.log_activity(
            &user_record.id,
            ActivityType::Login,
            None,
            None,
            "User logged in",
            ip_address,
        ).await?;

        info!("User authenticated: {}", email);
        Ok(session)
    }

    /// Share document with client
    pub async fn share_document(
        &self,
        document_id: &str,
        matter_id: &str,
        client_id: &str,
        title: &str,
        description: Option<&str>,
        shared_by: &str,
        access_level: AccessLevel,
        expires_at: Option<DateTime<Utc>>,
        download_limit: Option<u32>,
        requires_signature: bool,
    ) -> Result<SharedDocument> {
        // Get document info
        let doc = sqlx::query!(
            r#"
            SELECT content, document_type
            FROM case_documents
            WHERE id = ?
            "#,
            document_id
        )
        .fetch_one(&self.db)
        .await?;

        let file_size = doc.content.len() as u64;
        let mime_type = "application/pdf".to_string(); // Simplified

        let shared_doc = SharedDocument {
            id: uuid::Uuid::new_v4().to_string(),
            document_id: document_id.to_string(),
            matter_id: matter_id.to_string(),
            client_id: client_id.to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            file_path: format!("shared/{}/{}", client_id, document_id),
            file_size,
            mime_type,
            shared_by: shared_by.to_string(),
            shared_at: Utc::now(),
            expires_at,
            download_limit,
            downloads_count: 0,
            requires_signature,
            signature_status: if requires_signature { Some(SignatureStatus::Pending) } else { None },
            access_level,
        };

        // Save to database
        sqlx::query!(
            r#"
            INSERT INTO shared_documents (
                id, document_id, matter_id, client_id, title, description,
                file_path, file_size, mime_type, shared_by, shared_at,
                expires_at, download_limit, downloads_count, requires_signature,
                signature_status, access_level
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            shared_doc.id,
            shared_doc.document_id,
            shared_doc.matter_id,
            shared_doc.client_id,
            shared_doc.title,
            shared_doc.description,
            shared_doc.file_path,
            shared_doc.file_size,
            shared_doc.mime_type,
            shared_doc.shared_by,
            shared_doc.shared_at,
            shared_doc.expires_at,
            shared_doc.download_limit,
            shared_doc.downloads_count,
            shared_doc.requires_signature,
            serde_json::to_string(&shared_doc.signature_status)?,
            serde_json::to_string(&shared_doc.access_level)?
        )
        .execute(&self.db)
        .await?;

        info!("Document shared: {} with client: {}", document_id, client_id);
        Ok(shared_doc)
    }

    /// Send secure message
    pub async fn send_message(
        &self,
        matter_id: &str,
        from_user_id: &str,
        from_user_name: &str,
        to_user_id: &str,
        to_user_name: &str,
        subject: &str,
        body: &str,
        attachments: Vec<MessageAttachment>,
    ) -> Result<SecureMessage> {
        let message = SecureMessage {
            id: uuid::Uuid::new_v4().to_string(),
            matter_id: matter_id.to_string(),
            from_user_id: from_user_id.to_string(),
            from_user_name: from_user_name.to_string(),
            to_user_id: to_user_id.to_string(),
            to_user_name: to_user_name.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            encrypted: true,
            read_at: None,
            attachments,
            created_at: Utc::now(),
        };

        // Save to database
        let attachments_json = serde_json::to_string(&message.attachments)?;

        sqlx::query!(
            r#"
            INSERT INTO portal_messages (
                id, matter_id, from_user_id, from_user_name, to_user_id,
                to_user_name, subject, body, encrypted, read_at,
                attachments, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            message.id,
            message.matter_id,
            message.from_user_id,
            message.from_user_name,
            message.to_user_id,
            message.to_user_name,
            message.subject,
            message.body,
            message.encrypted,
            message.read_at,
            attachments_json,
            message.created_at
        )
        .execute(&self.db)
        .await?;

        // Log activity
        self.log_activity(
            from_user_id,
            ActivityType::MessageSent,
            Some(&message.id),
            Some("message"),
            &format!("Sent message: {}", subject),
            None,
        ).await?;

        info!("Secure message sent from {} to {}", from_user_name, to_user_name);
        Ok(message)
    }

    /// Get client dashboard
    pub async fn get_dashboard(&self, client_id: &str) -> Result<ClientDashboard> {
        // Get matters
        let matters = self.get_client_matters(client_id).await?;

        // Get recent documents
        let recent_documents = self.get_shared_documents(client_id, Some(10)).await?;

        // Count unread messages
        let unread_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM portal_messages
            WHERE to_user_id IN (
                SELECT id FROM portal_users WHERE client_id = ?
            ) AND read_at IS NULL
            "#,
            client_id
        )
        .fetch_one(&self.db)
        .await?;

        // Count pending signatures
        let pending_sigs = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM shared_documents
            WHERE client_id = ? AND requires_signature = 1
            AND signature_status = ?
            "#,
            client_id,
            serde_json::to_string(&SignatureStatus::Pending)?
        )
        .fetch_one(&self.db)
        .await?;

        // Get upcoming deadlines
        let upcoming_deadlines = self.get_upcoming_deadlines(client_id).await?;

        // Get recent activity
        let recent_activity = self.get_recent_activity(client_id, 20).await?;

        Ok(ClientDashboard {
            client_id: client_id.to_string(),
            matters,
            recent_documents,
            unread_messages: unread_count.count as u32,
            pending_signatures: pending_sigs.count as u32,
            upcoming_deadlines,
            recent_activity,
        })
    }

    async fn get_client_matters(&self, client_id: &str) -> Result<Vec<MatterSummary>> {
        let records = sqlx::query!(
            r#"
            SELECT
                m.id,
                m.title,
                m.matter_number,
                m.status,
                m.docket_number,
                m.court_name,
                COUNT(DISTINCT d.id) as doc_count,
                COUNT(DISTINCT msg.id) FILTER (WHERE msg.read_at IS NULL) as unread_msg_count
            FROM matters m
            LEFT JOIN case_documents d ON d.matter_id = m.id
            LEFT JOIN portal_messages msg ON msg.matter_id = m.id
            WHERE m.client_id = ?
            GROUP BY m.id
            "#,
            client_id
        )
        .fetch_all(&self.db)
        .await?;

        let matters = records.into_iter().map(|r| MatterSummary {
            id: r.id,
            title: r.title,
            matter_number: r.matter_number,
            status: r.status,
            docket_number: r.docket_number,
            court_name: r.court_name,
            next_deadline: None, // Would query from deadlines table
            document_count: r.doc_count as u32,
            unread_messages: r.unread_msg_count as u32,
        }).collect();

        Ok(matters)
    }

    async fn get_shared_documents(&self, client_id: &str, limit: Option<u32>) -> Result<Vec<SharedDocument>> {
        let limit_clause = limit.unwrap_or(50);

        let records = sqlx::query!(
            r#"
            SELECT
                id, document_id, matter_id, client_id, title, description,
                file_path, file_size, mime_type, shared_by, shared_at,
                expires_at, download_limit, downloads_count, requires_signature,
                signature_status, access_level
            FROM shared_documents
            WHERE client_id = ?
            ORDER BY shared_at DESC
            LIMIT ?
            "#,
            client_id,
            limit_clause
        )
        .fetch_all(&self.db)
        .await?;

        let documents = records.into_iter().map(|r| SharedDocument {
            id: r.id,
            document_id: r.document_id,
            matter_id: r.matter_id,
            client_id: r.client_id,
            title: r.title,
            description: r.description,
            file_path: r.file_path,
            file_size: r.file_size as u64,
            mime_type: r.mime_type,
            shared_by: r.shared_by,
            shared_at: DateTime::parse_from_rfc3339(&r.shared_at).unwrap().with_timezone(&Utc),
            expires_at: r.expires_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            download_limit: r.download_limit.map(|l| l as u32),
            downloads_count: r.downloads_count as u32,
            requires_signature: r.requires_signature,
            signature_status: r.signature_status.and_then(|s| serde_json::from_str(&s).ok()),
            access_level: serde_json::from_str(&r.access_level).unwrap_or(AccessLevel::View),
        }).collect();

        Ok(documents)
    }

    async fn get_upcoming_deadlines(&self, client_id: &str) -> Result<Vec<DeadlineSummary>> {
        // Simplified - would query from tasks/deadlines table
        Ok(Vec::new())
    }

    async fn get_recent_activity(&self, client_id: &str, limit: u32) -> Result<Vec<PortalActivity>> {
        let records = sqlx::query!(
            r#"
            SELECT
                id, user_id, activity_type, resource_id, resource_type,
                description, ip_address, created_at
            FROM portal_activity
            WHERE user_id IN (
                SELECT id FROM portal_users WHERE client_id = ?
            )
            ORDER BY created_at DESC
            LIMIT ?
            "#,
            client_id,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        let activities = records.into_iter().map(|r| PortalActivity {
            id: r.id,
            user_id: r.user_id,
            activity_type: serde_json::from_str(&r.activity_type).unwrap_or(ActivityType::Login),
            resource_id: r.resource_id,
            resource_type: r.resource_type,
            description: r.description,
            ip_address: r.ip_address,
            created_at: DateTime::parse_from_rfc3339(&r.created_at).unwrap().with_timezone(&Utc),
        }).collect();

        Ok(activities)
    }

    async fn log_activity(
        &self,
        user_id: &str,
        activity_type: ActivityType,
        resource_id: Option<&str>,
        resource_type: Option<&str>,
        description: &str,
        ip_address: Option<String>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let activity_type_json = serde_json::to_string(&activity_type)?;

        sqlx::query!(
            r#"
            INSERT INTO portal_activity (
                id, user_id, activity_type, resource_id, resource_type,
                description, ip_address, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            user_id,
            activity_type_json,
            resource_id,
            resource_type,
            description,
            ip_address,
            Utc::now()
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
