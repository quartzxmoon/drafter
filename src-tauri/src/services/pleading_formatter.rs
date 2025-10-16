// Automated Pleading Paper Formatter
// Formats legal documents according to court-specific rules with line numbering,
// proper margins, captions, and all required elements

use crate::domain::case_management::*;
use crate::domain::*;
use crate::services::court_rules::CourtRulesService;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

pub struct PleadingFormatter {
    court_rules_service: CourtRulesService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PleadingFormat {
    // Page setup
    pub page_width: f32,      // in inches
    pub page_height: f32,     // in inches
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,

    // Line numbering
    pub line_numbering: bool,
    pub line_number_position: LineNumberPosition,
    pub line_number_spacing: u32,  // Every N lines
    pub line_number_start: u32,
    pub line_number_font_size: f32,

    // Typography
    pub font_family: String,
    pub font_size: f32,
    pub line_spacing: f32,  // 1.0 = single, 2.0 = double
    pub paragraph_spacing: f32,

    // Caption
    pub caption_format: CaptionFormat,
    pub caption_alignment: Alignment,
    pub caption_font_size: f32,
    pub caption_all_caps: bool,

    // Footer
    pub include_footer: bool,
    pub footer_attorney_info: bool,
    pub footer_page_numbers: bool,
    pub footer_cert_service: bool,

    // Court-specific
    pub court_name: String,
    pub county: String,
    pub term: Option<String>,
    pub document_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineNumberPosition {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptionFormat {
    Standard,      // Party v. Party format
    InRe,          // In re: Matter Name
    Commonwealth,  // Commonwealth v. Defendant (PA criminal)
    Petition,      // Petition of Petitioner
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedDocument {
    pub html: String,
    pub rtf: String,
    pub latex: Option<String>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub page_count: u32,
    pub line_count: u32,
    pub word_count: u32,
    pub character_count: u32,
    pub complies_with_rules: bool,
    pub warnings: Vec<String>,
}

impl PleadingFormatter {
    pub fn new() -> Self {
        Self {
            court_rules_service: CourtRulesService::new(),
        }
    }

    // ========================================================================
    // Main Formatting Functions
    // ========================================================================

    #[instrument(skip(self, content, matter))]
    pub async fn format_pleading(
        &self,
        content: &str,
        matter: &Matter,
        client: &Client,
        document_type: &DocumentType,
        court_rules: &CourtRules,
    ) -> Result<FormattedDocument> {
        info!("Formatting pleading for matter: {}", matter.id);

        // Get court-specific format settings
        let format = self.get_format_for_court(court_rules, document_type).await?;

        // Build caption
        let caption = self.build_caption(matter, client, document_type, &format).await?;

        // Format body content
        let body = self.format_body(content, &format).await?;

        // Add signature block
        let signature = self.build_signature_block(court_rules).await?;

        // Add certificate of service if required
        let cert_service = if court_rules.service_certificate {
            self.build_certificate_of_service().await?
        } else {
            String::new()
        };

        // Assemble complete document
        let html = self.assemble_html_document(&caption, &body, &signature, &cert_service, &format).await?;
        let rtf = self.convert_to_rtf(&html, &format).await?;

        // Calculate metadata
        let metadata = self.calculate_metadata(&html, &format, court_rules).await?;

        Ok(FormattedDocument {
            html,
            rtf,
            latex: None,
            metadata,
        })
    }

    // ========================================================================
    // Court-Specific Format Rules
    // ========================================================================

    async fn get_format_for_court(
        &self,
        court_rules: &CourtRules,
        document_type: &DocumentType,
    ) -> Result<PleadingFormat> {
        // Pennsylvania court defaults
        let mut format = PleadingFormat {
            page_width: 8.5,
            page_height: 11.0,
            margin_top: 1.0,
            margin_bottom: 1.0,
            margin_left: 1.5,   // Extra space for line numbers
            margin_right: 1.0,

            line_numbering: true,
            line_number_position: LineNumberPosition::Left,
            line_number_spacing: 1,
            line_number_start: 1,
            line_number_font_size: 10.0,

            font_family: court_rules.font.family.clone(),
            font_size: court_rules.font.size.parse().unwrap_or(12.0),
            line_spacing: court_rules.font.line_spacing.parse().unwrap_or(2.0),
            paragraph_spacing: 0.0,

            caption_format: CaptionFormat::Standard,
            caption_alignment: Alignment::Center,
            caption_font_size: 12.0,
            caption_all_caps: true,

            include_footer: true,
            footer_attorney_info: true,
            footer_page_numbers: true,
            footer_cert_service: court_rules.service_certificate,

            court_name: court_rules.court_id.clone(),
            county: String::new(),
            term: None,
            document_type: format!("{:?}", document_type),
        };

        // Court-specific adjustments
        if court_rules.court_id.contains("CP") {
            // Court of Common Pleas - requires line numbering
            format.line_numbering = true;
            format.line_number_spacing = 1;
        } else if court_rules.court_id.contains("MDJ") {
            // Magisterial District Justice - usually no line numbers
            format.line_numbering = false;
        } else if court_rules.court_id.contains("APP") {
            // Appellate courts
            format.line_numbering = false;
            format.line_spacing = 1.5;
        }

        // Document-type specific adjustments
        match document_type {
            DocumentType::Brief | DocumentType::Memorandum => {
                format.line_spacing = 1.5;
            }
            DocumentType::Affidavit | DocumentType::Declaration => {
                format.line_spacing = 1.0;
                format.line_numbering = false;
            }
            _ => {}
        }

        Ok(format)
    }

    // ========================================================================
    // Caption Building
    // ========================================================================

    async fn build_caption(
        &self,
        matter: &Matter,
        client: &Client,
        document_type: &DocumentType,
        format: &PleadingFormat,
    ) -> Result<String> {
        let mut caption = String::new();

        // Court name
        if let Some(court_name) = &matter.court_name {
            caption.push_str(&format!("<div class=\"court-name\">{}</div>\n", court_name.to_uppercase()));
        }

        // County
        if let Some(county) = &matter.county {
            caption.push_str(&format!("<div class=\"county\">{} COUNTY</div>\n", county.to_uppercase()));
        }

        caption.push_str("<div class=\"caption-box\">\n");

        // Party names based on format
        match format.caption_format {
            CaptionFormat::Standard => {
                let client_name = format!("{} {}", client.first_name, client.last_name);
                let opposing = matter.opposing_party.clone().unwrap_or("Unknown".to_string());

                caption.push_str(&format!(
                    "<div class=\"party plaintiff\">{}</div>\n",
                    if format.caption_all_caps { client_name.to_uppercase() } else { client_name }
                ));
                caption.push_str("<div class=\"versus\">v.</div>\n");
                caption.push_str(&format!(
                    "<div class=\"party defendant\">{}</div>\n",
                    if format.caption_all_caps { opposing.to_uppercase() } else { opposing }
                ));
            }
            CaptionFormat::Commonwealth => {
                caption.push_str("<div class=\"party\">");
                COMMONWEALTH OF PENNSYLVANIA</div>\n");
                caption.push_str("<div class=\"versus\">v.</div>\n");
                let defendant = matter.opposing_party.clone().unwrap_or("Unknown".to_string());
                caption.push_str(&format!(
                    "<div class=\"party defendant\">{}</div>\n",
                    if format.caption_all_caps { defendant.to_uppercase() } else { defendant }
                ));
            }
            CaptionFormat::InRe => {
                caption.push_str(&format!("<div class=\"in-re\">IN RE: {}</div>\n", matter.title.to_uppercase()));
            }
            CaptionFormat::Petition => {
                caption.push_str(&format!("<div class=\"petition\">PETITION OF {}</div>\n",
                    format!("{} {}", client.first_name, client.last_name).to_uppercase()));
            }
        }

        // Docket number
        if let Some(docket) = &matter.docket_number {
            caption.push_str(&format!("<div class=\"docket-number\">No. {}</div>\n", docket));
        }

        caption.push_str("</div>\n");

        // Document title
        caption.push_str(&format!(
            "<div class=\"document-title\">{}</div>\n",
            format!("{:?}", document_type).to_uppercase().replace("_", " ")
        ));

        Ok(caption)
    }

    // ========================================================================
    // Body Formatting
    // ========================================================================

    async fn format_body(&self, content: &str, format: &PleadingFormat) -> Result<String> {
        let mut html = String::new();
        let mut line_number = format.line_number_start;

        html.push_str("<div class=\"document-body\">\n");

        // Split content into paragraphs
        for paragraph in content.split("\n\n") {
            if paragraph.trim().is_empty() {
                continue;
            }

            html.push_str("<div class=\"paragraph\">\n");

            // Split into lines for line numbering
            let lines: Vec<&str> = paragraph.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                if format.line_numbering && line_number % format.line_number_spacing == 0 {
                    html.push_str(&format!(
                        "<div class=\"line\"><span class=\"line-number\">{}</span>{}</div>\n",
                        line_number, line
                    ));
                } else {
                    html.push_str(&format!("<div class=\"line\">{}</div>\n", line));
                }
                line_number += 1;
            }

            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");

        Ok(html)
    }

    // ========================================================================
    // Signature Block
    // ========================================================================

    async fn build_signature_block(&self, court_rules: &CourtRules) -> Result<String> {
        let mut signature = String::new();

        signature.push_str("<div class=\"signature-block\">\n");
        signature.push_str("<div class=\"respectfully-submitted\">Respectfully submitted,</div>\n");
        signature.push_str("<div class=\"signature-line\">_________________________________</div>\n");

        if court_rules.signature.attorney_name {
            signature.push_str("<div class=\"attorney-name\">[Attorney Name]</div>\n");
        }

        if court_rules.signature.attorney_id {
            signature.push_str("<div class=\"attorney-id\">[Attorney ID Number]</div>\n");
        }

        if court_rules.signature.firm_name {
            signature.push_str("<div class=\"firm-name\">[Firm Name]</div>\n");
        }

        if court_rules.signature.address {
            signature.push_str("<div class=\"address\">[Address]</div>\n");
            signature.push_str("<div class=\"address\">[City, State ZIP]</div>\n");
        }

        if court_rules.signature.phone {
            signature.push_str("<div class=\"phone\">[Phone Number]</div>\n");
        }

        if court_rules.signature.email {
            signature.push_str("<div class=\"email\">[Email Address]</div>\n");
        }

        signature.push_str("</div>\n");

        Ok(signature)
    }

    // ========================================================================
    // Certificate of Service
    // ========================================================================

    async fn build_certificate_of_service(&self) -> Result<String> {
        let mut cert = String::new();

        cert.push_str("<div class=\"certificate-of-service\">\n");
        cert.push_str("<h3>CERTIFICATE OF SERVICE</h3>\n");
        cert.push_str(&format!(
            "<p>I hereby certify that on {}, I served a true and correct copy of the foregoing document upon the following parties:</p>\n",
            Utc::now().format("%B %d, %Y")
        ));
        cert.push_str("<div class=\"service-list\">\n");
        cert.push_str("<p>[List of parties served]</p>\n");
        cert.push_str("</div>\n");
        cert.push_str("<div class=\"service-signature\">\n");
        cert.push_str("<div class=\"signature-line\">_________________________________</div>\n");
        cert.push_str("<div class=\"attorney-name\">[Attorney Name]</div>\n");
        cert.push_str("</div>\n");
        cert.push_str("</div>\n");

        Ok(cert)
    }

    // ========================================================================
    // Document Assembly
    // ========================================================================

    async fn assemble_html_document(
        &self,
        caption: &str,
        body: &str,
        signature: &str,
        cert_service: &str,
        format: &PleadingFormat,
    ) -> Result<String> {
        let css = self.generate_css(format).await?;

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Pleading Document</title>
    <style>{}</style>
</head>
<body>
    <div class="pleading-paper">
        <div class="caption-section">
            {}
        </div>
        <div class="body-section">
            {}
        </div>
        <div class="signature-section">
            {}
        </div>
        {}
    </div>
</body>
</html>"#,
            css, caption, body, signature, cert_service
        );

        Ok(html)
    }

    async fn generate_css(&self, format: &PleadingFormat) -> Result<String> {
        Ok(format!(
            r#"
@page {{
    size: {} {}in;
    margin: {}in {}in {}in {}in;
}}

body {{
    font-family: '{}', serif;
    font-size: {}pt;
    line-height: {};
    margin: 0;
    padding: 0;
}}

.pleading-paper {{
    width: 100%;
    max-width: {}in;
    margin: 0 auto;
}}

.caption-section {{
    text-align: center;
    margin-bottom: 2em;
    page-break-after: avoid;
}}

.court-name, .county {{
    font-weight: bold;
    margin: 0.5em 0;
}}

.caption-box {{
    border: 2px solid black;
    padding: 1em;
    margin: 1em auto;
    max-width: 80%;
}}

.party {{
    margin: 0.5em 0;
}}

.versus {{
    margin: 0.5em 0;
}}

.docket-number {{
    margin-top: 1em;
    font-weight: bold;
}}

.document-title {{
    font-weight: bold;
    text-decoration: underline;
    margin: 1em 0;
}}

.document-body {{
    text-align: justify;
    margin: 2em 0;
}}

.paragraph {{
    margin-bottom: {}em;
}}

.line {{
    position: relative;
}}

.line-number {{
    position: absolute;
    left: -{}in;
    width: 0.5in;
    text-align: right;
    font-size: {}pt;
    color: #666;
}}

.signature-block {{
    margin-top: 3em;
    margin-left: 50%;
}}

.respectfully-submitted {{
    margin-bottom: 2em;
}}

.signature-line {{
    margin: 2em 0 0.5em 0;
}}

.certificate-of-service {{
    margin-top: 3em;
    page-break-before: auto;
}}

.certificate-of-service h3 {{
    text-align: center;
    text-decoration: underline;
}}

@media print {{
    .line-number {{
        print-color-adjust: exact;
        -webkit-print-color-adjust: exact;
    }}
}}
"#,
            format.page_width,
            format.page_height,
            format.margin_top,
            format.margin_right,
            format.margin_bottom,
            format.margin_left,
            format.font_family,
            format.font_size,
            format.line_spacing,
            format.page_width,
            format.paragraph_spacing,
            format.margin_left - 0.5,
            format.line_number_font_size
        ))
    }

    // ========================================================================
    // Format Conversion
    // ========================================================================

    async fn convert_to_rtf(&self, html: &str, format: &PleadingFormat) -> Result<String> {
        // Basic HTML to RTF conversion
        // In production, use a proper HTML-to-RTF library
        let mut rtf = String::from(r"{\rtf1\ansi\deff0");

        rtf.push_str(&format!(r"{{\fonttbl{{\f0 {};}}}}", format.font_family));
        rtf.push_str(&format!(r"\f0\fs{}", (format.font_size * 2.0) as u32));

        // Simple conversion - strip HTML tags
        let text = html
            .replace("<div>", "")
            .replace("</div>", r"\par ")
            .replace("<p>", "")
            .replace("</p>", r"\par ")
            .replace("<br>", r"\line ")
            .replace("&nbsp;", " ");

        rtf.push_str(&text);
        rtf.push('}');

        Ok(rtf)
    }

    // ========================================================================
    // Metadata Calculation
    // ========================================================================

    async fn calculate_metadata(
        &self,
        html: &str,
        format: &PleadingFormat,
        court_rules: &CourtRules,
    ) -> Result<DocumentMetadata> {
        // Strip HTML tags for counting
        let text = html.replace(/<[^>]*>/g, "");

        let word_count = text.split_whitespace().count() as u32;
        let character_count = text.chars().count() as u32;
        let line_count = text.lines().count() as u32;

        // Estimate page count (very rough)
        let lines_per_page = ((format.page_height - format.margin_top - format.margin_bottom) / (format.font_size * format.line_spacing / 72.0)) as u32;
        let page_count = (line_count / lines_per_page) + 1;

        // Check compliance
        let mut warnings = Vec::new();
        let mut complies = true;

        // Check page limits if defined
        if let Some(&limit) = court_rules.page_limits.get(&format.document_type) {
            if page_count > limit {
                warnings.push(format!("Document exceeds page limit of {} pages", limit));
                complies = false;
            }
        }

        Ok(DocumentMetadata {
            page_count,
            line_count,
            word_count,
            character_count,
            complies_with_rules: complies,
            warnings,
        })
    }
}
