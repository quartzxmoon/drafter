// AI-Powered Document Assembly & Template Engine
// Smart templates with conditional logic, AI clause suggestions, and natural language editing

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use regex::Regex;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub category: TemplateCategory,
    pub description: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
    pub conditional_blocks: Vec<ConditionalBlock>,
    pub clauses: Vec<ClauseLibrary>,
    pub version: u32,
    pub is_public: bool,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: u32,
    pub rating: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Pleading,
    Motion,
    Brief,
    Contract,
    Agreement,
    Letter,
    Discovery,
    Corporate,
    RealEstate,
    FamilyLaw,
    Criminal,
    Immigration,
    Bankruptcy,
    IntellectualProperty,
    Employment,
    Personal,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: VariableType,
    pub label: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub validation: Option<ValidationRule>,
    pub help_text: Option<String>,
    pub auto_populate: Option<AutoPopulateRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    Text,
    Number,
    Date,
    Boolean,
    Select,
    MultiSelect,
    Email,
    Phone,
    Address,
    Currency,
    Paragraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    Regex,
    Length,
    Range,
    Email,
    Phone,
    Date,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPopulateRule {
    pub source: AutoPopulateSource,
    pub field_path: String,
    pub transformation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AutoPopulateSource {
    Matter,
    Client,
    User,
    Court,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBlock {
    pub id: String,
    pub condition: String,
    pub content: String,
    pub else_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseLibrary {
    pub id: String,
    pub name: String,
    pub category: String,
    pub text: String,
    pub jurisdiction: Option<String>,
    pub practice_area: Option<String>,
    pub tags: Vec<String>,
    pub ai_relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyRequest {
    pub template_id: String,
    pub matter_id: Option<String>,
    pub variables: HashMap<String, String>,
    pub auto_populate: bool,
    pub ai_enhancement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledDocument {
    pub id: String,
    pub template_id: String,
    pub matter_id: Option<String>,
    pub content: String,
    pub variables_used: HashMap<String, String>,
    pub ai_suggestions: Vec<AISuggestion>,
    pub assembled_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestion {
    pub suggestion_type: SuggestionType,
    pub content: String,
    pub relevance: f32,
    pub position: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    Clause,
    Language,
    Citation,
    Formatting,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageEdit {
    pub instruction: String,
    pub context: String,
}

pub struct DocumentAssemblyService {
    db: SqlitePool,
}

impl DocumentAssemblyService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new template
    pub async fn create_template(
        &self,
        name: &str,
        category: TemplateCategory,
        description: &str,
        content: &str,
        author: &str,
        is_public: bool,
    ) -> Result<Template> {
        let template = Template {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            category,
            description: description.to_string(),
            content: content.to_string(),
            variables: self.extract_variables(content)?,
            conditional_blocks: self.extract_conditional_blocks(content)?,
            clauses: Vec::new(),
            version: 1,
            is_public,
            author: author.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            rating: 0.0,
        };

        // Save to database
        self.save_template(&template).await?;

        info!("Created template: {} ({})", template.name, template.id);
        Ok(template)
    }

    /// Extract variables from template content
    fn extract_variables(&self, content: &str) -> Result<Vec<TemplateVariable>> {
        let mut variables = Vec::new();
        let re = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}")?;

        for cap in re.captures_iter(content) {
            let var_name = cap[1].to_string();

            // Check if already added
            if !variables.iter().any(|v: &TemplateVariable| v.name == var_name) {
                variables.push(TemplateVariable {
                    name: var_name.clone(),
                    var_type: VariableType::Text,
                    label: var_name.replace('_', " ").to_uppercase(),
                    default_value: None,
                    required: true,
                    validation: None,
                    help_text: None,
                    auto_populate: None,
                });
            }
        }

        Ok(variables)
    }

    /// Extract conditional blocks from template
    fn extract_conditional_blocks(&self, content: &str) -> Result<Vec<ConditionalBlock>> {
        let mut blocks = Vec::new();
        let re = Regex::new(r"\{\{#if\s+([^}]+)\}\}(.*?)\{\{/if\}\}")?;

        for cap in re.captures_iter(content) {
            blocks.push(ConditionalBlock {
                id: uuid::Uuid::new_v4().to_string(),
                condition: cap[1].to_string(),
                content: cap[2].to_string(),
                else_content: None,
            });
        }

        Ok(blocks)
    }

    /// Assemble document from template
    pub async fn assemble_document(
        &self,
        request: AssemblyRequest,
    ) -> Result<AssembledDocument> {
        // Load template
        let template = self.get_template(&request.template_id).await?;

        // Auto-populate variables if requested
        let mut variables = request.variables.clone();
        if request.auto_populate {
            if let Some(matter_id) = &request.matter_id {
                variables = self.auto_populate_from_matter(matter_id, &template, variables).await?;
            }
        }

        // Validate all required variables
        self.validate_variables(&template, &variables)?;

        // Render template
        let mut content = template.content.clone();

        // Replace variables
        for (key, value) in &variables {
            let pattern = format!("{{{{{}}}}}", key);
            content = content.replace(&pattern, value);
        }

        // Process conditional blocks
        content = self.process_conditionals(&content, &variables)?;

        // AI enhancement if requested
        let ai_suggestions = if request.ai_enhancement {
            self.generate_ai_suggestions(&content, &template, &variables).await?
        } else {
            Vec::new()
        };

        let assembled = AssembledDocument {
            id: uuid::Uuid::new_v4().to_string(),
            template_id: request.template_id.clone(),
            matter_id: request.matter_id.clone(),
            content,
            variables_used: variables,
            ai_suggestions,
            assembled_at: Utc::now(),
        };

        // Update template usage count
        self.increment_template_usage(&request.template_id).await?;

        info!("Assembled document from template: {}", request.template_id);
        Ok(assembled)
    }

    /// Auto-populate variables from matter data
    async fn auto_populate_from_matter(
        &self,
        matter_id: &str,
        template: &Template,
        mut variables: HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        // Fetch matter data
        let matter = sqlx::query!(
            r#"
            SELECT
                m.title,
                m.matter_number,
                m.docket_number,
                m.court_name,
                c.name as client_name,
                c.email as client_email,
                c.phone as client_phone
            FROM matters m
            LEFT JOIN clients c ON c.id = m.client_id
            WHERE m.id = ?
            "#,
            matter_id
        )
        .fetch_one(&self.db)
        .await?;

        // Map common variables
        variables.entry("matter_title".to_string()).or_insert(matter.title);
        variables.entry("matter_number".to_string()).or_insert(matter.matter_number);

        if let Some(docket) = matter.docket_number {
            variables.entry("docket_number".to_string()).or_insert(docket);
        }

        if let Some(court) = matter.court_name {
            variables.entry("court_name".to_string()).or_insert(court);
        }

        if let Some(client) = matter.client_name {
            variables.entry("client_name".to_string()).or_insert(client);
        }

        if let Some(email) = matter.client_email {
            variables.entry("client_email".to_string()).or_insert(email);
        }

        if let Some(phone) = matter.client_phone {
            variables.entry("client_phone".to_string()).or_insert(phone);
        }

        // Current date
        variables.entry("current_date".to_string()).or_insert(
            Utc::now().format("%B %d, %Y").to_string()
        );

        Ok(variables)
    }

    /// Validate required variables are present
    fn validate_variables(
        &self,
        template: &Template,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        for var in &template.variables {
            if var.required && !variables.contains_key(&var.name) {
                anyhow::bail!("Missing required variable: {}", var.name);
            }

            // Validate variable value if present
            if let Some(value) = variables.get(&var.name) {
                if let Some(validation) = &var.validation {
                    self.validate_value(value, validation)?;
                }
            }
        }

        Ok(())
    }

    /// Validate a single value against validation rules
    fn validate_value(&self, value: &str, validation: &ValidationRule) -> Result<()> {
        match validation.rule_type {
            ValidationType::Regex => {
                if let Some(pattern) = &validation.pattern {
                    let re = Regex::new(pattern)?;
                    if !re.is_match(value) {
                        anyhow::bail!("{}", validation.error_message);
                    }
                }
            }
            ValidationType::Length => {
                let len = value.len();
                if let Some(min) = validation.min_length {
                    if len < min {
                        anyhow::bail!("{}", validation.error_message);
                    }
                }
                if let Some(max) = validation.max_length {
                    if len > max {
                        anyhow::bail!("{}", validation.error_message);
                    }
                }
            }
            ValidationType::Email => {
                let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?;
                if !email_re.is_match(value) {
                    anyhow::bail!("{}", validation.error_message);
                }
            }
            ValidationType::Phone => {
                let phone_re = Regex::new(r"^\d{3}-?\d{3}-?\d{4}$")?;
                if !phone_re.is_match(value) {
                    anyhow::bail!("{}", validation.error_message);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Process conditional blocks
    fn process_conditionals(
        &self,
        content: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let mut result = content.to_string();
        let re = Regex::new(r"\{\{#if\s+([a-zA-Z_][a-zA-Z0-9_]*)\}\}(.*?)\{\{/if\}\}")?;

        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            let block_content = &cap[2];
            let full_match = &cap[0];

            // Evaluate condition
            let should_include = if let Some(value) = variables.get(var_name) {
                !value.is_empty() && value != "false" && value != "0"
            } else {
                false
            };

            result = result.replace(
                full_match,
                if should_include { block_content } else { "" }
            );
        }

        Ok(result)
    }

    /// Generate AI suggestions for document enhancement
    async fn generate_ai_suggestions(
        &self,
        content: &str,
        template: &Template,
        variables: &HashMap<String, String>,
    ) -> Result<Vec<AISuggestion>> {
        let mut suggestions = Vec::new();

        // Suggest relevant clauses based on template category
        let clauses = self.get_relevant_clauses(&template.category, content).await?;
        for clause in clauses {
            suggestions.push(AISuggestion {
                suggestion_type: SuggestionType::Clause,
                content: clause.text.clone(),
                relevance: clause.ai_relevance_score,
                position: 0,
                reason: format!("Recommended {} clause", clause.category),
            });
        }

        // Check for missing standard clauses
        if matches!(template.category, TemplateCategory::Contract | TemplateCategory::Agreement) {
            if !content.contains("governing law") && !content.contains("Governing Law") {
                suggestions.push(AISuggestion {
                    suggestion_type: SuggestionType::Clause,
                    content: "GOVERNING LAW: This Agreement shall be governed by and construed in accordance with the laws of the Commonwealth of Pennsylvania.".to_string(),
                    relevance: 0.95,
                    position: content.len(),
                    reason: "Missing governing law clause (standard for contracts)".to_string(),
                });
            }

            if !content.contains("force majeure") && !content.contains("Force Majeure") {
                suggestions.push(AISuggestion {
                    suggestion_type: SuggestionType::Clause,
                    content: "FORCE MAJEURE: Neither party shall be liable for any failure or delay in performance due to circumstances beyond its reasonable control.".to_string(),
                    relevance: 0.85,
                    position: content.len(),
                    reason: "Consider adding force majeure clause".to_string(),
                });
            }
        }

        Ok(suggestions)
    }

    /// Get relevant clauses from library
    async fn get_relevant_clauses(
        &self,
        category: &TemplateCategory,
        _content: &str,
    ) -> Result<Vec<ClauseLibrary>> {
        let category_str = serde_json::to_string(category)?;

        let records = sqlx::query!(
            r#"
            SELECT id, name, category, text, jurisdiction, practice_area, tags, ai_relevance_score
            FROM clause_library
            WHERE template_category = ?
            ORDER BY ai_relevance_score DESC
            LIMIT 5
            "#,
            category_str
        )
        .fetch_all(&self.db)
        .await?;

        let clauses = records.into_iter().map(|r| ClauseLibrary {
            id: r.id,
            name: r.name,
            category: r.category,
            text: r.text,
            jurisdiction: r.jurisdiction,
            practice_area: r.practice_area,
            tags: serde_json::from_str(&r.tags).unwrap_or_default(),
            ai_relevance_score: r.ai_relevance_score as f32,
        }).collect();

        Ok(clauses)
    }

    /// Natural language template editing
    pub async fn edit_with_natural_language(
        &self,
        template_id: &str,
        edit: NaturalLanguageEdit,
    ) -> Result<Template> {
        let mut template = self.get_template(template_id).await?;

        // Parse natural language instruction
        let instruction_lower = edit.instruction.to_lowercase();

        if instruction_lower.contains("add") && instruction_lower.contains("clause") {
            // AI would parse this properly, but for now we'll use simple rules
            if instruction_lower.contains("force majeure") {
                template.content.push_str("\n\nFORCE MAJEURE: Neither party shall be liable for any failure or delay in performance due to circumstances beyond its reasonable control, including but not limited to acts of God, war, terrorism, pandemic, government action, or natural disasters.");
            } else if instruction_lower.contains("confidentiality") {
                template.content.push_str("\n\nCONFIDENTIALITY: The parties agree to maintain the confidentiality of all proprietary information disclosed during the term of this agreement.");
            }
        } else if instruction_lower.contains("remove") {
            // Parse what to remove
            if let Some(keyword) = instruction_lower.split("remove").nth(1) {
                let keyword = keyword.trim();
                // Remove paragraphs containing keyword
                let paragraphs: Vec<&str> = template.content.split("\n\n").collect();
                let filtered: Vec<&str> = paragraphs
                    .into_iter()
                    .filter(|p| !p.to_lowercase().contains(keyword))
                    .collect();
                template.content = filtered.join("\n\n");
            }
        }

        // Update version and timestamp
        template.version += 1;
        template.updated_at = Utc::now();

        // Save updated template
        self.save_template(&template).await?;

        info!("Edited template {} with NL: {}", template_id, edit.instruction);
        Ok(template)
    }

    /// Get template by ID
    async fn get_template(&self, template_id: &str) -> Result<Template> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, name, category, description, content, variables,
                conditional_blocks, version, is_public, author,
                created_at, updated_at, usage_count, rating
            FROM templates
            WHERE id = ?
            "#,
            template_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(Template {
            id: record.id,
            name: record.name,
            category: serde_json::from_str(&record.category)?,
            description: record.description,
            content: record.content,
            variables: serde_json::from_str(&record.variables)?,
            conditional_blocks: serde_json::from_str(&record.conditional_blocks)?,
            clauses: Vec::new(),
            version: record.version as u32,
            is_public: record.is_public,
            author: record.author,
            created_at: DateTime::parse_from_rfc3339(&record.created_at)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&record.updated_at)?.with_timezone(&Utc),
            usage_count: record.usage_count as u32,
            rating: record.rating as f32,
        })
    }

    /// Save template to database
    async fn save_template(&self, template: &Template) -> Result<()> {
        let category_json = serde_json::to_string(&template.category)?;
        let variables_json = serde_json::to_string(&template.variables)?;
        let conditionals_json = serde_json::to_string(&template.conditional_blocks)?;

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO templates (
                id, name, category, description, content, variables,
                conditional_blocks, version, is_public, author,
                created_at, updated_at, usage_count, rating
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            template.id,
            template.name,
            category_json,
            template.description,
            template.content,
            variables_json,
            conditionals_json,
            template.version,
            template.is_public,
            template.author,
            template.created_at,
            template.updated_at,
            template.usage_count,
            template.rating
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Increment template usage count
    async fn increment_template_usage(&self, template_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE templates
            SET usage_count = usage_count + 1
            WHERE id = ?
            "#,
            template_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// List templates by category
    pub async fn list_templates(&self, category: Option<TemplateCategory>) -> Result<Vec<Template>> {
        let records = if let Some(cat) = category {
            let cat_json = serde_json::to_string(&cat)?;
            sqlx::query!(
                r#"
                SELECT
                    id, name, category, description, content, variables,
                    conditional_blocks, version, is_public, author,
                    created_at, updated_at, usage_count, rating
                FROM templates
                WHERE category = ?
                ORDER BY usage_count DESC, rating DESC
                "#,
                cat_json
            )
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT
                    id, name, category, description, content, variables,
                    conditional_blocks, version, is_public, author,
                    created_at, updated_at, usage_count, rating
                FROM templates
                ORDER BY usage_count DESC, rating DESC
                "#
            )
            .fetch_all(&self.db)
            .await?
        };

        let templates = records.into_iter().map(|r| Template {
            id: r.id,
            name: r.name,
            category: serde_json::from_str(&r.category).unwrap_or(TemplateCategory::Custom),
            description: r.description,
            content: r.content,
            variables: serde_json::from_str(&r.variables).unwrap_or_default(),
            conditional_blocks: serde_json::from_str(&r.conditional_blocks).unwrap_or_default(),
            clauses: Vec::new(),
            version: r.version as u32,
            is_public: r.is_public,
            author: r.author,
            created_at: DateTime::parse_from_rfc3339(&r.created_at).ok().map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(Utc::now),
            updated_at: DateTime::parse_from_rfc3339(&r.updated_at).ok().map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(Utc::now),
            usage_count: r.usage_count as u32,
            rating: r.rating as f32,
        }).collect();

        Ok(templates)
    }
}
