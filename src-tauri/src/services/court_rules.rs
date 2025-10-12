// Court rules service for PA eDocket Desktop

use crate::domain::*;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, instrument, warn};

#[derive(Debug, Serialize, Deserialize)]
struct CourtsConfig {
    courts: HashMap<String, CourtConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CourtConfig {
    name: String,
    level: String,
    jurisdiction: String,
    formatting: FormattingRules,
    efiling: Option<EFilingConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FormattingRules {
    margins: Margins,
    font: FontSettings,
    caption: CaptionRules,
    signature: SignatureRules,
    service_certificate: bool,
    page_limits: HashMap<String, u32>,
    #[serde(default)]
    table_of_contents: bool,
    #[serde(default)]
    table_of_authorities: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Margins {
    top: String,
    bottom: String,
    left: String,
    right: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FontSettings {
    family: String,
    size: String,
    line_spacing: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CaptionRules {
    format: String,
    include_docket: bool,
    include_court: bool,
    include_county: bool,
    #[serde(default)]
    include_judge: bool,
    #[serde(default)]
    include_division: bool,
    #[serde(default)]
    include_lower_court: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SignatureRules {
    attorney_name: bool,
    attorney_id: bool,
    firm_name: bool,
    address: bool,
    phone: bool,
    email: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct EFilingConfig {
    enabled: bool,
    provider: Option<String>,
    endpoint: Option<String>,
}

pub struct CourtRulesService {
    courts_config: Option<CourtsConfig>,
}

impl CourtRulesService {
    pub fn new() -> Self {
        Self {
            courts_config: None,
        }
    }

    pub async fn load_config(&mut self, config_path: &Path) -> Result<()> {
        info!("Loading courts configuration from: {:?}", config_path);

        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read courts config file: {:?}", config_path))?;

        let config: CourtsConfig = serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse courts.yaml configuration")?;

        debug!("Loaded configuration for {} courts", config.courts.len());
        self.courts_config = Some(config);

        Ok(())
    }

    #[instrument(skip(self, court_id))]
    pub async fn get_court_rules(&self, court_id: &str) -> Result<CourtRules> {
        info!("Loading court rules for: {}", court_id);

        let config = self.courts_config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Courts configuration not loaded"))?;

        let court_config = config.courts.get(court_id)
            .ok_or_else(|| anyhow::anyhow!("Court not found: {}", court_id))?;

        // Convert config to domain model
        let court_rules = CourtRules {
            court_id: court_id.to_string(),
            court_name: court_config.name.clone(),
            level: court_config.level.clone(),
            jurisdiction: court_config.jurisdiction.clone(),

            // Formatting rules
            margins: DocumentMargins {
                top: self.parse_measurement(&court_config.formatting.margins.top)?,
                bottom: self.parse_measurement(&court_config.formatting.margins.bottom)?,
                left: self.parse_measurement(&court_config.formatting.margins.left)?,
                right: self.parse_measurement(&court_config.formatting.margins.right)?,
            },

            font: DocumentFont {
                family: court_config.formatting.font.family.clone(),
                size: self.parse_font_size(&court_config.formatting.font.size)?,
                line_spacing: self.parse_line_spacing(&court_config.formatting.font.line_spacing)?,
            },

            caption_format: court_config.formatting.caption.format.clone(),
            requires_service_certificate: court_config.formatting.service_certificate,
            requires_table_of_contents: court_config.formatting.table_of_contents,
            requires_table_of_authorities: court_config.formatting.table_of_authorities,

            page_limits: court_config.formatting.page_limits.clone(),

            // E-filing configuration
            efiling_enabled: court_config.efiling.as_ref().map(|e| e.enabled).unwrap_or(false),
            efiling_provider: court_config.efiling.as_ref().and_then(|e| e.provider.clone()),
            efiling_endpoint: court_config.efiling.as_ref().and_then(|e| e.endpoint.clone()),
        };

        Ok(court_rules)
    }

    #[instrument(skip(self, court_rules, document_type, content))]
    pub async fn validate_document_format(&self, court_rules: &CourtRules, document_type: &str, content: &str) -> Result<Vec<String>> {
        info!("Validating document format for {}", document_type);

        let mut violations = Vec::new();

        // Check page limits
        if let Some(&limit) = court_rules.page_limits.get(document_type) {
            let page_count = self.estimate_page_count(content, &court_rules.font)?;
            if page_count > limit {
                violations.push(format!(
                    "Document exceeds page limit: {} pages (limit: {})",
                    page_count, limit
                ));
            }
        }

        // Check required sections
        if court_rules.requires_table_of_contents && !self.has_table_of_contents(content) {
            violations.push("Document must include a table of contents".to_string());
        }

        if court_rules.requires_table_of_authorities && !self.has_table_of_authorities(content) {
            violations.push("Document must include a table of authorities".to_string());
        }

        if court_rules.requires_service_certificate && !self.has_service_certificate(content) {
            violations.push("Document must include a certificate of service".to_string());
        }

        // Check caption format
        if !self.has_proper_caption(content, &court_rules.caption_format) {
            violations.push(format!(
                "Document must include proper caption in {} format",
                court_rules.caption_format
            ));
        }

        // Check signature block
        if !self.has_signature_block(content) {
            violations.push("Document must include attorney signature block".to_string());
        }

        Ok(violations)
    }

    #[instrument(skip(self, court_rules, content))]
    pub async fn apply_formatting(&self, court_rules: &CourtRules, content: &str) -> Result<String> {
        info!("Applying court formatting rules for {}", court_rules.court_name);

        let mut formatted_content = content.to_string();

        // Apply font formatting
        formatted_content = self.apply_font_formatting(&formatted_content, &court_rules.font)?;

        // Apply margin formatting
        formatted_content = self.apply_margin_formatting(&formatted_content, &court_rules.margins)?;

        // Ensure proper caption
        formatted_content = self.ensure_caption(&formatted_content, &court_rules.caption_format)?;

        // Add required sections if missing
        if court_rules.requires_table_of_contents && !self.has_table_of_contents(&formatted_content) {
            formatted_content = self.add_table_of_contents(&formatted_content)?;
        }

        if court_rules.requires_table_of_authorities && !self.has_table_of_authorities(&formatted_content) {
            formatted_content = self.add_table_of_authorities(&formatted_content)?;
        }

        if court_rules.requires_service_certificate && !self.has_service_certificate(&formatted_content) {
            formatted_content = self.add_service_certificate(&formatted_content)?;
        }

        // Ensure proper signature block
        formatted_content = self.ensure_signature_block(&formatted_content)?;

        Ok(formatted_content)
    }

    // Helper methods
    fn parse_measurement(&self, measurement: &str) -> Result<f32> {
        let re = Regex::new(r"^(\d+\.?\d*)(in|pt|cm|mm)$")?;
        if let Some(caps) = re.captures(measurement) {
            let value: f32 = caps[1].parse()?;
            let unit = &caps[2];

            // Convert to points (standard unit)
            let points = match unit {
                "in" => value * 72.0,
                "pt" => value,
                "cm" => value * 28.35,
                "mm" => value * 2.835,
                _ => return Err(anyhow::anyhow!("Unknown unit: {}", unit)),
            };

            Ok(points)
        } else {
            Err(anyhow::anyhow!("Invalid measurement format: {}", measurement))
        }
    }

    fn parse_font_size(&self, size: &str) -> Result<f32> {
        if size.ends_with("pt") {
            let value = size.trim_end_matches("pt").parse::<f32>()?;
            Ok(value)
        } else {
            Err(anyhow::anyhow!("Font size must be in points: {}", size))
        }
    }

    fn parse_line_spacing(&self, spacing: &str) -> Result<f32> {
        match spacing {
            "single" => Ok(1.0),
            "double" => Ok(2.0),
            "1.5" => Ok(1.5),
            _ => {
                if let Ok(value) = spacing.parse::<f32>() {
                    Ok(value)
                } else {
                    Err(anyhow::anyhow!("Invalid line spacing: {}", spacing))
                }
            }
        }
    }

    fn estimate_page_count(&self, content: &str, font: &DocumentFont) -> Result<u32> {
        // Rough estimation based on character count and font size
        let chars_per_line = (72.0 * 6.5 / font.size) as usize; // Assuming 6.5" line width
        let lines_per_page = (72.0 * 9.0 / (font.size * font.line_spacing)) as usize; // Assuming 9" text height

        let total_chars = content.len();
        let total_lines = (total_chars / chars_per_line) + 1;
        let pages = (total_lines / lines_per_page) + 1;

        Ok(pages as u32)
    }

    fn has_table_of_contents(&self, content: &str) -> bool {
        let toc_patterns = [
            r"(?i)table\s+of\s+contents",
            r"(?i)contents",
            r"(?i)index",
        ];

        toc_patterns.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(content)).unwrap_or(false)
        })
    }

    fn has_table_of_authorities(&self, content: &str) -> bool {
        let toa_patterns = [
            r"(?i)table\s+of\s+authorities",
            r"(?i)authorities\s+cited",
            r"(?i)cases\s+cited",
        ];

        toa_patterns.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(content)).unwrap_or(false)
        })
    }

    fn has_service_certificate(&self, content: &str) -> bool {
        let cert_patterns = [
            r"(?i)certificate\s+of\s+service",
            r"(?i)proof\s+of\s+service",
            r"(?i)I\s+hereby\s+certify",
        ];

        cert_patterns.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(content)).unwrap_or(false)
        })
    }

    fn has_proper_caption(&self, content: &str, format: &str) -> bool {
        match format {
            "standard_pa" => {
                let caption_pattern = r"(?i)(in\s+the\s+court\s+of\s+common\s+pleas|magisterial\s+district\s+court)";
                Regex::new(caption_pattern).map(|re| re.is_match(content)).unwrap_or(false)
            },
            "appellate_pa" => {
                let caption_pattern = r"(?i)(superior\s+court\s+of\s+pennsylvania|supreme\s+court\s+of\s+pennsylvania)";
                Regex::new(caption_pattern).map(|re| re.is_match(content)).unwrap_or(false)
            },
            _ => true, // Unknown format, assume valid
        }
    }

    fn has_signature_block(&self, content: &str) -> bool {
        let sig_patterns = [
            r"(?i)respectfully\s+submitted",
            r"(?i)attorney\s+for",
            r"(?i)pa\s+attorney\s+id",
            r"(?i)bar\s+id",
        ];

        sig_patterns.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(content)).unwrap_or(false)
        })
    }

    // Formatting application methods (simplified implementations)
    fn apply_font_formatting(&self, content: &str, font: &DocumentFont) -> Result<String> {
        // In a real implementation, this would apply RTF/HTML/LaTeX formatting
        // For now, we'll add formatting comments
        let formatted = format!(
            "<!-- Font: {} {}pt, Line Spacing: {} -->\n{}",
            font.family, font.size, font.line_spacing, content
        );
        Ok(formatted)
    }

    fn apply_margin_formatting(&self, content: &str, margins: &DocumentMargins) -> Result<String> {
        // In a real implementation, this would set page margins
        let formatted = format!(
            "<!-- Margins: Top: {}pt, Bottom: {}pt, Left: {}pt, Right: {}pt -->\n{}",
            margins.top, margins.bottom, margins.left, margins.right, content
        );
        Ok(formatted)
    }

    fn ensure_caption(&self, content: &str, format: &str) -> Result<String> {
        if self.has_proper_caption(content, format) {
            Ok(content.to_string())
        } else {
            // Add a placeholder caption
            let caption = match format {
                "standard_pa" => "IN THE COURT OF COMMON PLEAS\n[COUNTY], PENNSYLVANIA\n\n",
                "appellate_pa" => "IN THE SUPERIOR COURT OF PENNSYLVANIA\n\n",
                _ => "IN THE [COURT NAME]\n\n",
            };
            Ok(format!("{}{}", caption, content))
        }
    }

    fn add_table_of_contents(&self, content: &str) -> Result<String> {
        let toc = "\nTABLE OF CONTENTS\n\nI. Introduction ............................ 1\nII. Statement of Facts ..................... 2\nIII. Argument .............................. 3\nIV. Conclusion ............................. 5\n\n";
        Ok(format!("{}{}", toc, content))
    }

    fn add_table_of_authorities(&self, content: &str) -> Result<String> {
        let toa = "\nTABLE OF AUTHORITIES\n\nCASES\n\n[Cases will be automatically generated from citations]\n\nSTATUTES\n\n[Statutes will be automatically generated from citations]\n\n";
        Ok(format!("{}{}", toa, content))
    }

    fn add_service_certificate(&self, content: &str) -> Result<String> {
        let cert = "\n\nCERTIFICATE OF SERVICE\n\nI hereby certify that a true and correct copy of the foregoing document was served upon the parties listed below in the manner indicated:\n\n[ ] By hand delivery\n[ ] By first-class mail, postage prepaid\n[ ] By electronic filing\n[ ] By facsimile transmission\n\nDate: _______________\n\n_________________________\n[Attorney Name]\nAttorney for [Party]\nPA Attorney ID No. [Number]\n";
        Ok(format!("{}{}", content, cert))
    }

    fn ensure_signature_block(&self, content: &str) -> Result<String> {
        if self.has_signature_block(content) {
            Ok(content.to_string())
        } else {
            let signature = "\n\nRespectfully submitted,\n\n_________________________\n[Attorney Name]\nAttorney for [Party]\nPA Attorney ID No. [Number]\n[Firm Name]\n[Address]\n[City, State ZIP]\n[Phone]\n[Email]\n";
            Ok(format!("{}{}", content, signature))
        }
    }
}
