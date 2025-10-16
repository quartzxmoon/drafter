// Case Management Service - Manages clients, matters, and automated document generation

use crate::domain::case_management::*;
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

pub struct CaseManagementService {
    db_pool: Pool<Sqlite>,
}

impl CaseManagementService {
    pub fn new(db_pool: Pool<Sqlite>) -> Self {
        Self { db_pool }
    }

    // ========================================================================
    // Client Management
    // ========================================================================

    #[instrument(skip(self, request))]
    pub async fn create_client(&self, request: CreateClientRequest) -> Result<Client> {
        info!("Creating new client: {} {}", request.first_name, request.last_name);

        let client = Client {
            id: Uuid::new_v4().to_string(),
            first_name: request.first_name.clone(),
            last_name: request.last_name.clone(),
            email: request.email.clone(),
            phone: request.phone.clone(),
            address: request.address.clone(),
            city: request.city.clone(),
            state: request.state.clone().or(Some("PA".to_string())),
            zip_code: request.zip_code.clone(),
            date_of_birth: None,
            ssn_encrypted: None,
            notes: request.notes.clone(),
            client_type: request.client_type.clone(),
            business_name: request.business_name.clone(),
            contact_person: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: ClientStatus::Active,
        };

        sqlx::query!(
            r#"
            INSERT INTO clients (
                id, first_name, last_name, email, phone, address, city, state, zip_code,
                notes, client_type, business_name, created_at, updated_at, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            client.id,
            client.first_name,
            client.last_name,
            client.email,
            client.phone,
            client.address,
            client.city,
            client.state,
            client.zip_code,
            client.notes,
            serde_json::to_string(&client.client_type)?,
            client.business_name,
            client.created_at.to_rfc3339(),
            client.updated_at.to_rfc3339(),
            serde_json::to_string(&client.status)?
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create client")?;

        info!("Client created successfully: {}", client.id);
        Ok(client)
    }

    #[instrument(skip(self))]
    pub async fn get_client(&self, client_id: &str) -> Result<Client> {
        debug!("Fetching client: {}", client_id);

        let row = sqlx::query!(
            r#"SELECT * FROM clients WHERE id = ?"#,
            client_id
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Client not found")?;

        Ok(Client {
            id: row.id,
            first_name: row.first_name,
            last_name: row.last_name,
            email: row.email,
            phone: row.phone,
            address: row.address,
            city: row.city,
            state: row.state,
            zip_code: row.zip_code,
            date_of_birth: None, // TODO: Parse from row
            ssn_encrypted: row.ssn_encrypted,
            notes: row.notes,
            client_type: serde_json::from_str(&row.client_type.unwrap_or("\"individual\"".to_string()))?,
            business_name: row.business_name,
            contact_person: row.contact_person,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            status: serde_json::from_str(&row.status.unwrap_or("\"active\"".to_string()))?,
        })
    }

    #[instrument(skip(self))]
    pub async fn list_clients(&self, status: Option<ClientStatus>) -> Result<Vec<Client>> {
        debug!("Listing clients");

        let query = if let Some(status) = status {
            let status_str = serde_json::to_string(&status)?;
            sqlx::query!(
                r#"SELECT * FROM clients WHERE status = ? ORDER BY last_name, first_name"#,
                status_str
            )
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query!(r#"SELECT * FROM clients ORDER BY last_name, first_name"#)
                .fetch_all(&self.db_pool)
                .await?
        };

        let clients: Vec<Client> = query
            .into_iter()
            .filter_map(|row| {
                Some(Client {
                    id: row.id,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    email: row.email,
                    phone: row.phone,
                    address: row.address,
                    city: row.city,
                    state: row.state,
                    zip_code: row.zip_code,
                    date_of_birth: None,
                    ssn_encrypted: row.ssn_encrypted,
                    notes: row.notes,
                    client_type: serde_json::from_str(&row.client_type.unwrap_or("\"individual\"".to_string())).ok()?,
                    business_name: row.business_name,
                    contact_person: row.contact_person,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))?,
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))?,
                    status: serde_json::from_str(&row.status.unwrap_or("\"active\"".to_string())).ok()?,
                })
            })
            .collect();

        info!("Found {} clients", clients.len());
        Ok(clients)
    }

    // ========================================================================
    // Matter Management
    // ========================================================================

    #[instrument(skip(self, request))]
    pub async fn create_matter(&self, request: CreateMatterRequest) -> Result<Matter> {
        info!("Creating new matter for client: {}", request.client_id);

        // Verify client exists
        self.get_client(&request.client_id).await?;

        // Generate unique matter number
        let matter_number = self.generate_matter_number(&request.matter_type).await?;

        let matter = Matter {
            id: Uuid::new_v4().to_string(),
            client_id: request.client_id.clone(),
            matter_number,
            title: request.title.clone(),
            description: request.description.clone(),
            matter_type: request.matter_type.clone(),
            case_type: request.case_type.clone(),
            court_level: request.court_level.clone(),
            court_name: request.court_name.clone(),
            county: request.county.clone(),
            docket_number: None,
            judge_name: None,
            opposing_party: request.opposing_party.clone(),
            opposing_counsel: None,
            opposing_counsel_firm: None,
            opposing_counsel_email: None,
            opposing_counsel_phone: None,
            filing_date: None,
            status: MatterStatus::Active,
            outcome: None,
            settlement_amount: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };

        sqlx::query!(
            r#"
            INSERT INTO matters (
                id, client_id, matter_number, title, description, matter_type, case_type,
                court_level, court_name, county, opposing_party, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            matter.id,
            matter.client_id,
            matter.matter_number,
            matter.title,
            matter.description,
            serde_json::to_string(&matter.matter_type)?,
            matter.case_type,
            matter.court_level,
            matter.court_name,
            matter.county,
            matter.opposing_party,
            serde_json::to_string(&matter.status)?,
            matter.created_at.to_rfc3339(),
            matter.updated_at.to_rfc3339()
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to create matter")?;

        info!("Matter created successfully: {}", matter.id);
        Ok(matter)
    }

    #[instrument(skip(self))]
    pub async fn get_matter(&self, matter_id: &str) -> Result<Matter> {
        debug!("Fetching matter: {}", matter_id);

        let row = sqlx::query!(
            r#"SELECT * FROM matters WHERE id = ?"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Matter not found")?;

        Ok(self.row_to_matter(row)?)
    }

    #[instrument(skip(self))]
    pub async fn get_matter_summary(&self, matter_id: &str) -> Result<MatterSummary> {
        debug!("Fetching matter summary: {}", matter_id);

        let matter = self.get_matter(matter_id).await?;
        let client = self.get_client(&matter.client_id).await?;

        // Get counts and statistics
        let events_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM case_events WHERE matter_id = ?"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0) as i32;

        let documents_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM case_documents WHERE matter_id = ?"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0) as i32;

        let tasks_pending = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM tasks WHERE matter_id = ? AND status != 'completed'"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0) as i32;

        let total_time = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(hours), 0) as total FROM time_entries WHERE matter_id = ?"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0.0) as f32;

        let total_expenses = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(amount), 0) as total FROM expenses WHERE matter_id = ?"#,
            matter_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0.0) as f32;

        // Get next deadline
        let next_deadline = self.get_next_deadline(matter_id).await.ok();

        Ok(MatterSummary {
            matter,
            client,
            events_count,
            documents_count,
            tasks_pending,
            next_deadline,
            total_time,
            total_expenses,
        })
    }

    #[instrument(skip(self))]
    pub async fn list_matters(&self, client_id: Option<&str>, status: Option<MatterStatus>) -> Result<Vec<Matter>> {
        debug!("Listing matters");

        let query = match (client_id, status) {
            (Some(cid), Some(stat)) => {
                let status_str = serde_json::to_string(&stat)?;
                sqlx::query!(
                    r#"SELECT * FROM matters WHERE client_id = ? AND status = ? ORDER BY created_at DESC"#,
                    cid,
                    status_str
                )
                .fetch_all(&self.db_pool)
                .await?
            }
            (Some(cid), None) => {
                sqlx::query!(
                    r#"SELECT * FROM matters WHERE client_id = ? ORDER BY created_at DESC"#,
                    cid
                )
                .fetch_all(&self.db_pool)
                .await?
            }
            (None, Some(stat)) => {
                let status_str = serde_json::to_string(&stat)?;
                sqlx::query!(
                    r#"SELECT * FROM matters WHERE status = ? ORDER BY created_at DESC"#,
                    status_str
                )
                .fetch_all(&self.db_pool)
                .await?
            }
            (None, None) => {
                sqlx::query!(r#"SELECT * FROM matters ORDER BY created_at DESC"#)
                    .fetch_all(&self.db_pool)
                    .await?
            }
        };

        let matters: Vec<Matter> = query
            .into_iter()
            .filter_map(|row| self.row_to_matter(row).ok())
            .collect();

        info!("Found {} matters", matters.len());
        Ok(matters)
    }

    // ========================================================================
    // Automated Document Generation
    // ========================================================================

    #[instrument(skip(self, request))]
    pub async fn generate_document(&self, request: GenerateDocumentRequest) -> Result<GenerateDocumentResponse> {
        info!(
            "Generating document for matter: {} using template: {}",
            request.matter_id, request.template_id
        );

        // Get matter data
        let matter_summary = self.get_matter_summary(&request.matter_id).await?;

        // Get template
        let template = self.get_template(&request.template_id).await?;

        // Auto-populate variables from case data
        let variables = if request.auto_populate {
            self.auto_populate_variables(&matter_summary, &template).await?
        } else {
            HashMap::new()
        };

        // Merge with custom variables if provided
        let final_variables = if let Some(custom) = request.custom_variables {
            let mut merged = variables;
            if let serde_json::Value::Object(custom_map) = custom {
                for (key, value) in custom_map {
                    merged.insert(key, value);
                }
            }
            merged
        } else {
            variables
        };

        // Generate document content
        let (content, warnings, missing_data) = self.render_template(&template, &final_variables).await?;

        // Create document record
        let document_id = Uuid::new_v4().to_string();
        let file_name = format!("{}_{}.docx", request.title.replace(" ", "_"), document_id);
        let file_path = format!("documents/{}/{}", request.matter_id, file_name);

        // Save document
        sqlx::query!(
            r#"
            INSERT INTO case_documents (
                id, matter_id, document_type, title, file_path, version, is_template,
                filed_with_court, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, 1, 0, 0, ?, ?)
            "#,
            document_id,
            request.matter_id,
            serde_json::to_string(&request.document_type)?,
            request.title,
            file_path,
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339()
        )
        .execute(&self.db_pool)
        .await
        .context("Failed to save document record")?;

        info!("Document generated successfully: {}", document_id);

        Ok(GenerateDocumentResponse {
            document_id,
            file_path,
            preview_html: content,
            warnings,
            missing_data,
        })
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn generate_matter_number(&self, matter_type: &MatterType) -> Result<String> {
        let prefix = match matter_type {
            MatterType::Civil => "CIV",
            MatterType::Criminal => "CRIM",
            MatterType::Family => "FAM",
            MatterType::Estate => "EST",
            MatterType::RealEstate => "RE",
            MatterType::Business => "BUS",
            MatterType::Employment => "EMP",
            MatterType::PersonalInjury => "PI",
            MatterType::Immigration => "IMM",
            MatterType::Bankruptcy => "BK",
            MatterType::IntellectualProperty => "IP",
            MatterType::Administrative => "ADM",
            MatterType::Other => "OTH",
        };

        let year = Utc::now().format("%Y");
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM matters WHERE matter_number LIKE ?"#,
            format!("{}-{}-", prefix, year)
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(format!("{}-{}-{:04}", prefix, year, count + 1))
    }

    async fn get_template(&self, template_id: &str) -> Result<DocumentTemplate> {
        let row = sqlx::query!(
            r#"SELECT * FROM document_templates WHERE id = ?"#,
            template_id
        )
        .fetch_one(&self.db_pool)
        .await
        .context("Template not found")?;

        Ok(DocumentTemplate {
            id: row.id,
            template_name: row.template_name,
            document_type: serde_json::from_str(&row.document_type)?,
            court_level: row.court_level,
            matter_types: serde_json::from_str(&row.matter_types.unwrap_or("[]".to_string()))?,
            description: row.description,
            template_content: row.template_content,
            variable_schema: serde_json::from_str(&row.variable_schema)?,
            auto_populate_rules: row.auto_populate_rules.and_then(|s| serde_json::from_str(&s).ok()),
            formatting_rules: row.formatting_rules.and_then(|s| serde_json::from_str(&s).ok()),
            file_path: row.file_path,
            is_public: row.is_public != 0,
            is_pro_se_friendly: row.is_pro_se_friendly != 0,
            category: serde_json::from_str(&row.category)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        })
    }

    async fn auto_populate_variables(
        &self,
        matter_summary: &MatterSummary,
        template: &DocumentTemplate,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut variables = HashMap::new();

        // Client information
        variables.insert("client_name".to_string(), json!(format!("{} {}", matter_summary.client.first_name, matter_summary.client.last_name)));
        variables.insert("client_address".to_string(), json!(matter_summary.client.address.clone().unwrap_or_default()));
        variables.insert("client_phone".to_string(), json!(matter_summary.client.phone.clone().unwrap_or_default()));
        variables.insert("client_email".to_string(), json!(matter_summary.client.email.clone().unwrap_or_default()));

        // Matter information
        variables.insert("matter_title".to_string(), json!(matter_summary.matter.title.clone()));
        variables.insert("matter_number".to_string(), json!(matter_summary.matter.matter_number.clone()));
        variables.insert("docket_number".to_string(), json!(matter_summary.matter.docket_number.clone().unwrap_or_default()));
        variables.insert("court_name".to_string(), json!(matter_summary.matter.court_name.clone().unwrap_or_default()));
        variables.insert("county".to_string(), json!(matter_summary.matter.county.clone().unwrap_or_default()));
        variables.insert("judge_name".to_string(), json!(matter_summary.matter.judge_name.clone().unwrap_or_default()));
        variables.insert("opposing_party".to_string(), json!(matter_summary.matter.opposing_party.clone().unwrap_or_default()));

        // Current date
        variables.insert("current_date".to_string(), json!(Utc::now().format("%B %d, %Y").to_string()));

        Ok(variables)
    }

    async fn render_template(
        &self,
        template: &DocumentTemplate,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<(String, Vec<String>, Vec<String>)> {
        let mut content = template.template_content.clone();
        let mut warnings = Vec::new();
        let mut missing_data = Vec::new();

        // Replace variables in template
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            content = content.replace(&placeholder, &value_str);
        }

        // Check for unreplaced placeholders
        if content.contains("{{") {
            let placeholders: Vec<String> = content
                .split("{{")
                .skip(1)
                .filter_map(|s| s.split("}}").next())
                .map(|s| s.to_string())
                .collect();

            for placeholder in placeholders {
                if !variables.contains_key(&placeholder) {
                    missing_data.push(placeholder.clone());
                    warnings.push(format!("Missing data for: {}", placeholder));
                }
            }
        }

        Ok((content, warnings, missing_data))
    }

    async fn get_next_deadline(&self, matter_id: &str) -> Result<CaseEvent> {
        let now = Utc::now().to_rfc3339();
        let row = sqlx::query!(
            r#"SELECT * FROM case_events WHERE matter_id = ? AND event_date >= ? AND completed = 0 ORDER BY event_date ASC LIMIT 1"#,
            matter_id,
            now
        )
        .fetch_one(&self.db_pool)
        .await
        .context("No upcoming deadlines")?;

        // TODO: Convert row to CaseEvent
        Err(anyhow::anyhow!("Not implemented"))
    }

    fn row_to_matter(&self, row: sqlx::sqlite::SqliteRow) -> Result<Matter> {
        use sqlx::Row;

        Ok(Matter {
            id: row.try_get("id")?,
            client_id: row.try_get("client_id")?,
            matter_number: row.try_get("matter_number")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            matter_type: serde_json::from_str(&row.try_get::<String, _>("matter_type")?)?,
            case_type: row.try_get("case_type")?,
            court_level: row.try_get("court_level")?,
            court_name: row.try_get("court_name")?,
            county: row.try_get("county")?,
            docket_number: row.try_get("docket_number")?,
            judge_name: row.try_get("judge_name")?,
            opposing_party: row.try_get("opposing_party")?,
            opposing_counsel: row.try_get("opposing_counsel")?,
            opposing_counsel_firm: row.try_get("opposing_counsel_firm")?,
            opposing_counsel_email: row.try_get("opposing_counsel_email")?,
            opposing_counsel_phone: row.try_get("opposing_counsel_phone")?,
            filing_date: None, // TODO: Parse date
            status: serde_json::from_str(&row.try_get::<String, _>("status")?)?,
            outcome: row.try_get("outcome")?,
            settlement_amount: row.try_get("settlement_amount")?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            closed_at: None, // TODO: Parse date
        })
    }
}
