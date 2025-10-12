// Document drafting service for PA eDocket Desktop

use crate::domain::*;
use crate::services::court_rules::CourtRulesService;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

pub struct DraftingService {
    templates_dir: PathBuf,
    output_dir: PathBuf,
    court_rules_service: CourtRulesService,
    templates_cache: HashMap<String, DocumentTemplate>,
}

impl DraftingService {
    pub fn new(templates_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            templates_dir,
            output_dir,
            court_rules_service: CourtRulesService::new(),
            templates_cache: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self, courts_config_path: &Path) -> Result<()> {
        info!("Initializing drafting service");

        // Load court rules configuration
        self.court_rules_service.load_config(courts_config_path).await?;

        // Load all templates
        self.load_templates().await?;

        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

        info!("Drafting service initialized with {} templates", self.templates_cache.len());
        Ok(())
    }

    #[instrument(skip(self, job))]
    pub async fn draft_document(&self, job: &DraftJob) -> Result<DraftResult> {
        info!("Drafting document with template: {}", job.template_id);

        // Get template
        let template = self.get_template(&job.template_id).await?;

        // Get court rules if specified
        let court_rules = if let Some(court_id) = &job.court_id {
            Some(self.court_rules_service.get_court_rules(court_id).await?)
        } else {
            None
        };

        // Validate required variables
        let validation_errors = self.validate_variables(&template, &job.variables)?;
        if !validation_errors.is_empty() {
            return Ok(DraftResult {
                pdf_path: None,
                docx_path: None,
                manifest_path: String::new(),
                validation_errors,
                warnings: vec![],
            });
        }

        // Process template with variables
        let mut content = self.substitute_variables(&template.content, &job.variables)?;

        // Apply court-specific formatting if rules are available
        let mut warnings = Vec::new();
        if let Some(rules) = &court_rules {
            content = self.court_rules_service.apply_formatting(rules, &content).await?;

            // Validate against court rules
            let rule_violations = self.court_rules_service
                .validate_document_format(rules, &job.document_type, &content)
                .await?;
            warnings.extend(rule_violations);
        }

        // Generate output files
        let job_id = Uuid::new_v4();
        let base_filename = format!("{}_{}", job.template_id, job_id);

        // Generate DOCX
        let docx_path = self.generate_docx(&content, &base_filename, court_rules.as_ref()).await?;

        // Generate PDF
        let pdf_path = self.generate_pdf(&content, &base_filename, court_rules.as_ref()).await?;

        // Generate manifest
        let manifest_path = self.generate_manifest(&job, &template, &docx_path, &pdf_path, &warnings).await?;

        info!("Document drafted successfully: {}", base_filename);

        Ok(DraftResult {
            pdf_path: Some(pdf_path),
            docx_path: Some(docx_path),
            manifest_path,
            validation_errors: vec![],
            warnings,
        })
    }

    #[instrument(skip(self, template_id))]
    pub async fn get_template(&self, template_id: &str) -> Result<DocumentTemplate> {
        info!("Loading template: {}", template_id);

        if let Some(template) = self.templates_cache.get(template_id) {
            return Ok(template.clone());
        }

        Err(anyhow::anyhow!("Template not found: {}", template_id))
    }

    pub async fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        let templates: Vec<TemplateInfo> = self.templates_cache
            .values()
            .map(|t| TemplateInfo {
                id: t.id.clone(),
                name: t.name.clone(),
                category: t.category.clone(),
                description: t.description.clone(),
                court_types: t.court_types.clone(),
                document_type: t.document_type.clone(),
                variable_count: t.variables.len(),
            })
            .collect();

        Ok(templates)
    }

    pub async fn get_template_variables(&self, template_id: &str) -> Result<Vec<TemplateVariable>> {
        let template = self.get_template(template_id).await?;
        Ok(template.variables)
    }

    // Helper methods
    async fn load_templates(&mut self) -> Result<()> {
        info!("Loading templates from: {:?}", self.templates_dir);

        if !self.templates_dir.exists() {
            warn!("Templates directory does not exist, creating default templates");
            self.create_default_templates().await?;
        }

        let entries = fs::read_dir(&self.templates_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_template_file(&path).await {
                    Ok(template) => {
                        self.templates_cache.insert(template.id.clone(), template);
                    },
                    Err(e) => {
                        warn!("Failed to load template {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_template_file(&self, path: &Path) -> Result<DocumentTemplate> {
        let content = fs::read_to_string(path)?;
        let template: TemplateConfig = serde_yaml::from_str(&content)?;

        Ok(DocumentTemplate {
            id: template.id,
            name: template.name,
            category: template.category,
            description: template.description,
            court_types: template.court_types,
            document_type: template.document_type,
            content: template.content,
            variables: template.variables,
        })
    }

    async fn create_default_templates(&self) -> Result<()> {
        fs::create_dir_all(&self.templates_dir)?;

        // Create a sample motion template
        let motion_template = TemplateConfig {
            id: "motion_basic".to_string(),
            name: "Basic Motion".to_string(),
            category: "Motions".to_string(),
            description: "A basic motion template for Pennsylvania courts".to_string(),
            court_types: vec!["cp".to_string(), "mdj".to_string()],
            document_type: "motion".to_string(),
            content: include_str!("../../templates/motion_basic.txt").to_string(),
            variables: vec![
                TemplateVariable {
                    name: "case_caption".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "Full case caption".to_string(),
                    options: None,
                    default_value: None,
                },
                TemplateVariable {
                    name: "docket_number".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "Court docket number".to_string(),
                    options: None,
                    default_value: None,
                },
                TemplateVariable {
                    name: "motion_title".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "Title of the motion".to_string(),
                    options: None,
                    default_value: Some("Motion for Summary Judgment".to_string()),
                },
                TemplateVariable {
                    name: "movant_name".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "Name of the moving party".to_string(),
                    options: None,
                    default_value: None,
                },
                TemplateVariable {
                    name: "attorney_name".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "Attorney name".to_string(),
                    options: None,
                    default_value: None,
                },
                TemplateVariable {
                    name: "attorney_id".to_string(),
                    var_type: "text".to_string(),
                    required: true,
                    description: "PA Attorney ID Number".to_string(),
                    options: None,
                    default_value: None,
                },
            ],
        };

        let template_path = self.templates_dir.join("motion_basic.yaml");
        let yaml_content = serde_yaml::to_string(&motion_template)?;
        fs::write(template_path, yaml_content)?;

        info!("Created default motion template");
        Ok(())
    }

    fn validate_variables(&self, template: &DocumentTemplate, variables: &HashMap<String, String>) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        for template_var in &template.variables {
            if template_var.required {
                if let Some(value) = variables.get(&template_var.name) {
                    if value.trim().is_empty() {
                        errors.push(format!("Required variable '{}' cannot be empty", template_var.name));
                    }
                } else {
                    errors.push(format!("Required variable '{}' is missing", template_var.name));
                }
            }

            // Validate against options if provided
            if let Some(options) = &template_var.options {
                if let Some(value) = variables.get(&template_var.name) {
                    if !options.contains(value) {
                        errors.push(format!(
                            "Variable '{}' value '{}' is not in allowed options: {:?}",
                            template_var.name, value, options
                        ));
                    }
                }
            }
        }

        Ok(errors)
    }

    fn substitute_variables(&self, content: &str, variables: &HashMap<String, String>) -> Result<String> {
        let mut result = content.to_string();

        // Replace {{variable_name}} patterns
        let var_regex = Regex::new(r"\{\{(\w+)\}\}")?;

        result = var_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            variables.get(var_name)
                .map(|v| v.as_str())
                .unwrap_or(&format!("{{{{MISSING: {}}}}}", var_name))
        }).to_string();

        // Replace conditional blocks {{#if variable}}...{{/if}}
        let if_regex = Regex::new(r"\{\{#if\s+(\w+)\}\}(.*?)\{\{/if\}\}")?;
        result = if_regex.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let content = &caps[2];

            if variables.get(var_name).map(|v| !v.is_empty()).unwrap_or(false) {
                content
            } else {
                ""
            }
        }).to_string();

        // Replace date placeholders
        let now = Utc::now();
        result = result.replace("{{current_date}}", &now.format("%B %d, %Y").to_string());
        result = result.replace("{{current_year}}", &now.format("%Y").to_string());

        Ok(result)
    }

    async fn generate_docx(&self, content: &str, base_filename: &str, court_rules: Option<&CourtRules>) -> Result<String> {
        let filename = format!("{}.docx", base_filename);
        let output_path = self.output_dir.join(&filename);

        // For now, save as RTF which can be opened by Word
        let rtf_content = self.convert_to_rtf(content, court_rules)?;
        fs::write(&output_path, rtf_content)?;

        Ok(output_path.to_string_lossy().to_string())
    }

    async fn generate_pdf(&self, content: &str, base_filename: &str, court_rules: Option<&CourtRules>) -> Result<String> {
        let filename = format!("{}.pdf", base_filename);
        let output_path = self.output_dir.join(&filename);

        // For now, save as HTML which can be converted to PDF
        let html_content = self.convert_to_html(content, court_rules)?;
        fs::write(&output_path.with_extension("html"), html_content)?;

        // In a real implementation, you would use a PDF generation library
        // For now, just return the HTML path
        Ok(output_path.with_extension("html").to_string_lossy().to_string())
    }

    async fn generate_manifest(&self, job: &DraftJob, template: &DocumentTemplate, docx_path: &str, pdf_path: &str, warnings: &[String]) -> Result<String> {
        let manifest = DraftManifest {
            job_id: Uuid::new_v4(),
            template_id: template.id.clone(),
            template_name: template.name.clone(),
            document_type: job.document_type.clone(),
            court_id: job.court_id.clone(),
            created_at: Utc::now(),
            variables: job.variables.clone(),
            output_files: vec![
                OutputFile {
                    path: docx_path.to_string(),
                    format: "docx".to_string(),
                    size: fs::metadata(docx_path).map(|m| m.len()).unwrap_or(0),
                },
                OutputFile {
                    path: pdf_path.to_string(),
                    format: "pdf".to_string(),
                    size: fs::metadata(pdf_path).map(|m| m.len()).unwrap_or(0),
                },
            ],
            warnings: warnings.to_vec(),
        };

        let manifest_filename = format!("manifest_{}.json", manifest.job_id);
        let manifest_path = self.output_dir.join(&manifest_filename);

        let json_content = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, json_content)?;

        Ok(manifest_path.to_string_lossy().to_string())
    }

    fn convert_to_rtf(&self, content: &str, court_rules: Option<&CourtRules>) -> Result<String> {
        let mut rtf = String::from(r"{\rtf1\ansi\deff0");

        // Add font table
        if let Some(rules) = court_rules {
            rtf.push_str(&format!(r"{{\fonttbl{{\f0 {};}}}}", rules.font.family));
            rtf.push_str(&format!(r"\f0\fs{}", (rules.font.size * 2.0) as u32)); // RTF uses half-points
        } else {
            rtf.push_str(r"{\fonttbl{\f0 Times New Roman;}}");
            rtf.push_str(r"\f0\fs24");
        }

        // Add margins if available
        if let Some(rules) = court_rules {
            rtf.push_str(&format!(r"\margl{}\margr{}\margt{}\margb{}",
                (rules.margins.left * 20.0) as u32,   // RTF uses twips (1/20 point)
                (rules.margins.right * 20.0) as u32,
                (rules.margins.top * 20.0) as u32,
                (rules.margins.bottom * 20.0) as u32
            ));
        }

        // Convert content
        let rtf_content = content
            .replace('\n', r"\par ")
            .replace('\t', r"\tab ");

        rtf.push_str(&rtf_content);
        rtf.push('}');

        Ok(rtf)
    }

    fn convert_to_html(&self, content: &str, court_rules: Option<&CourtRules>) -> Result<String> {
        let mut html = String::from("<!DOCTYPE html><html><head><meta charset='utf-8'><style>");

        // Add CSS styles based on court rules
        if let Some(rules) = court_rules {
            html.push_str(&format!(
                "body {{ font-family: '{}'; font-size: {}pt; line-height: {}; margin: {}pt {}pt {}pt {}pt; }}",
                rules.font.family,
                rules.font.size,
                rules.font.line_spacing,
                rules.margins.top,
                rules.margins.right,
                rules.margins.bottom,
                rules.margins.left
            ));
        } else {
            html.push_str("body { font-family: 'Times New Roman'; font-size: 12pt; line-height: 2.0; margin: 72pt; }");
        }

        html.push_str("</style></head><body>");

        // Convert content to HTML
        let html_content = content
            .replace('\n', "<br>")
            .replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;");

        html.push_str(&html_content);
        html.push_str("</body></html>");

        Ok(html)
    }
}

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftResult {
    pub pdf_path: Option<String>,
    pub docx_path: Option<String>,
    pub manifest_path: String,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub court_types: Vec<String>,
    pub document_type: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: String,
    pub required: bool,
    pub description: String,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub court_types: Vec<String>,
    pub document_type: String,
    pub variable_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateConfig {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub court_types: Vec<String>,
    pub document_type: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DraftManifest {
    pub job_id: Uuid,
    pub template_id: String,
    pub template_name: String,
    pub document_type: String,
    pub court_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub variables: HashMap<String, String>,
    pub output_files: Vec<OutputFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputFile {
    pub path: String,
    pub format: String,
    pub size: u64,
}
