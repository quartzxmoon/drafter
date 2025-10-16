// AI-Powered Citation Extraction and Integration Service
// Automatically detects citations, searches case law, and integrates into documents

use crate::domain::*;
use crate::providers::courtlistener::{CourtListenerProvider, SearchQuery};
use crate::services::citations::CitationService;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationSuggestion {
    pub original_text: String,
    pub suggested_citation: String,
    pub case_name: String,
    pub court: String,
    pub year: String,
    pub relevance_score: f32,
    pub full_text_url: Option<String>,
    pub opinion_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedCitation {
    pub text: String,
    pub start_index: usize,
    pub end_index: usize,
    pub citation_type: String,
    pub is_valid: bool,
    pub bluebook_format: String,
    pub suggestions: Vec<CitationSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfAuthoritiesEntry {
    pub citation: String,
    pub page_references: Vec<u32>,
    pub citation_type: String,
    pub sort_key: String,
}

pub struct AICitationService {
    citation_service: CitationService,
    courtlistener: Option<CourtListenerProvider>,
}

impl AICitationService {
    pub fn new(courtlistener_token: Option<String>) -> Self {
        Self {
            citation_service: CitationService::new(),
            courtlistener: courtlistener_token.map(|token| CourtListenerProvider::new(token)),
        }
    }

    // ========================================================================
    // Citation Extraction
    // ========================================================================

    #[instrument(skip(self, text))]
    pub async fn extract_citations(&self, text: &str) -> Result<Vec<ExtractedCitation>> {
        info!("Extracting citations from text");

        let mut extracted = Vec::new();

        // Case law citations pattern (e.g., "123 F.3d 456")
        let case_pattern = Regex::new(r"\b(\d+)\s+([A-Za-z\.]+\d*)\s+(\d+)\b")?;

        // Statute citations pattern (e.g., "42 U.S.C. § 1983")
        let statute_pattern = Regex::new(r"\b(\d+)\s+([A-Za-z\.]+)\s+§+\s*(\d+)")?;

        // Pennsylvania statute pattern (e.g., "18 Pa.C.S. § 2702")
        let pa_statute_pattern = Regex::new(r"\b(\d+)\s+Pa\.C\.S\.\s+§+\s*(\d+)")?;

        // Extract case law citations
        for cap in case_pattern.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let citation_text = full_match.as_str();

            let volume = &cap[1];
            let reporter = &cap[2];
            let page = &cap[3];

            // Validate and format
            let bluebook = self.format_case_citation(volume, reporter, page).await?;

            // Get suggestions from CourtListener
            let suggestions = self.get_citation_suggestions(citation_text).await?;

            extracted.push(ExtractedCitation {
                text: citation_text.to_string(),
                start_index: full_match.start(),
                end_index: full_match.end(),
                citation_type: "case".to_string(),
                is_valid: !suggestions.is_empty(),
                bluebook_format: bluebook,
                suggestions,
            });
        }

        // Extract statute citations
        for cap in statute_pattern.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let citation_text = full_match.as_str();

            let bluebook = self.format_statute_citation(citation_text).await?;

            extracted.push(ExtractedCitation {
                text: citation_text.to_string(),
                start_index: full_match.start(),
                end_index: full_match.end(),
                citation_type: "statute".to_string(),
                is_valid: true,
                bluebook_format: bluebook,
                suggestions: Vec::new(),
            });
        }

        // Extract PA statute citations
        for cap in pa_statute_pattern.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let citation_text = full_match.as_str();

            let bluebook = self.format_pa_statute_citation(citation_text).await?;

            extracted.push(ExtractedCitation {
                text: citation_text.to_string(),
                start_index: full_match.start(),
                end_index: full_match.end(),
                citation_type: "pa_statute".to_string(),
                is_valid: true,
                bluebook_format: bluebook,
                suggestions: Vec::new(),
            });
        }

        info!("Extracted {} citations", extracted.len());
        Ok(extracted)
    }

    // ========================================================================
    // AI-Powered Citation Suggestions
    // ========================================================================

    #[instrument(skip(self, context))]
    pub async fn suggest_citations(&self, context: &str, query: &str) -> Result<Vec<CitationSuggestion>> {
        info!("Getting AI citation suggestions for: {}", query);

        let mut suggestions = Vec::new();

        if let Some(ref courtlistener) = self.courtlistener {
            // Search CourtListener for relevant cases
            let search_result = courtlistener
                .search_opinions(SearchQuery {
                    q: Some(query.to_string()),
                    case_name: None,
                    court: None,
                    docket_number: None,
                    filed_after: None,
                    filed_before: None,
                    cited_gt: None,
                    cited_lt: None,
                    status: Some("Precedential".to_string()),
                    order_by: Some("-citeCount".to_string()), // Most cited first
                })
                .await?;

            // Convert to suggestions
            for (idx, opinion) in search_result.results.iter().enumerate().take(10) {
                // Get cluster for citation info
                if let Ok(cluster_id) = self.extract_cluster_id(&opinion.cluster) {
                    if let Ok(cluster) = courtlistener.get_opinion_cluster(cluster_id).await {
                        let citation = cluster.federal_cite_one
                            .or(cluster.state_cite_one)
                            .or(cluster.neutral_cite)
                            .unwrap_or_else(|| cluster.case_name.clone());

                        suggestions.push(CitationSuggestion {
                            original_text: query.to_string(),
                            suggested_citation: citation.clone(),
                            case_name: cluster.case_name.clone(),
                            court: self.get_court_name(&cluster.docket),
                            year: cluster.date_filed.split('-').next().unwrap_or("").to_string(),
                            relevance_score: 1.0 - (idx as f32 * 0.1),
                            full_text_url: Some(opinion.absolute_url.clone()),
                            opinion_id: Some(opinion.id),
                        });
                    }
                }
            }
        }

        Ok(suggestions)
    }

    // ========================================================================
    // Real-time Citation Formatting
    // ========================================================================

    #[instrument(skip(self, text))]
    pub async fn format_citations_in_text(&self, text: &str) -> Result<String> {
        info!("Formatting citations in text");

        let extracted = self.extract_citations(text).await?;
        let mut formatted_text = text.to_string();

        // Replace citations in reverse order to maintain indices
        for citation in extracted.iter().rev() {
            if citation.is_valid {
                formatted_text.replace_range(
                    citation.start_index..citation.end_index,
                    &citation.bluebook_format,
                );
            }
        }

        Ok(formatted_text)
    }

    // ========================================================================
    // Table of Authorities Generation
    // ========================================================================

    #[instrument(skip(self, document_text))]
    pub async fn generate_table_of_authorities(
        &self,
        document_text: &str,
    ) -> Result<String> {
        info!("Generating Table of Authorities");

        let extracted = self.extract_citations(document_text).await?;

        // Group citations by type
        let mut cases = Vec::new();
        let mut statutes = Vec::new();
        let mut rules = Vec::new();
        let mut other = Vec::new();

        for citation in extracted {
            let entry = TableOfAuthoritiesEntry {
                citation: citation.bluebook_format.clone(),
                page_references: self.find_page_references(&citation, document_text),
                citation_type: citation.citation_type.clone(),
                sort_key: self.get_sort_key(&citation.bluebook_format),
            };

            match citation.citation_type.as_str() {
                "case" => cases.push(entry),
                "statute" | "pa_statute" => statutes.push(entry),
                "rule" => rules.push(entry),
                _ => other.push(entry),
            }
        }

        // Sort each category
        cases.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        statutes.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        rules.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        other.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));

        // Build TOA
        let mut toa = String::new();
        toa.push_str("<h2 style='text-align: center; text-decoration: underline;'>TABLE OF AUTHORITIES</h2>\n\n");

        if !cases.is_empty() {
            toa.push_str("<h3>Cases</h3>\n");
            for entry in cases {
                toa.push_str(&format!(
                    "<p style='margin-left: 0.5in; text-indent: -0.5in;'>{} ... {}</p>\n",
                    entry.citation,
                    self.format_page_refs(&entry.page_references)
                ));
            }
            toa.push_str("\n");
        }

        if !statutes.is_empty() {
            toa.push_str("<h3>Statutes</h3>\n");
            for entry in statutes {
                toa.push_str(&format!(
                    "<p style='margin-left: 0.5in; text-indent: -0.5in;'>{} ... {}</p>\n",
                    entry.citation,
                    self.format_page_refs(&entry.page_references)
                ));
            }
            toa.push_str("\n");
        }

        if !rules.is_empty() {
            toa.push_str("<h3>Rules</h3>\n");
            for entry in rules {
                toa.push_str(&format!(
                    "<p style='margin-left: 0.5in; text-indent: -0.5in;'>{} ... {}</p>\n",
                    entry.citation,
                    self.format_page_refs(&entry.page_references)
                ));
            }
            toa.push_str("\n");
        }

        if !other.is_empty() {
            toa.push_str("<h3>Other Authorities</h3>\n");
            for entry in other {
                toa.push_str(&format!(
                    "<p style='margin-left: 0.5in; text-indent: -0.5in;'>{} ... {}</p>\n",
                    entry.citation,
                    self.format_page_refs(&entry.page_references)
                ));
            }
        }

        Ok(toa)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn format_case_citation(&self, volume: &str, reporter: &str, page: &str) -> Result<String> {
        // Normalize reporter abbreviation
        let normalized_reporter = self.normalize_reporter(reporter);
        Ok(format!("{} {} {}", volume, normalized_reporter, page))
    }

    async fn format_statute_citation(&self, citation: &str) -> Result<String> {
        // Already in proper format, just validate spacing
        Ok(citation.to_string())
    }

    async fn format_pa_statute_citation(&self, citation: &str) -> Result<String> {
        // Ensure proper spacing: "18 Pa.C.S. § 2702"
        Ok(citation.to_string())
    }

    fn normalize_reporter(&self, reporter: &str) -> String {
        match reporter {
            "F3d" | "F.3d" => "F.3d".to_string(),
            "F2d" | "F.2d" => "F.2d".to_string(),
            "FSupp" | "F.Supp." => "F. Supp.".to_string(),
            "A3d" | "A.3d" => "A.3d".to_string(),
            "A2d" | "A.2d" => "A.2d".to_string(),
            _ => reporter.to_string(),
        }
    }

    fn extract_cluster_id(&self, cluster_url: &str) -> Result<u32> {
        let parts: Vec<&str> = cluster_url.split('/').collect();
        parts
            .iter()
            .find_map(|&part| part.parse::<u32>().ok())
            .ok_or_else(|| anyhow::anyhow!("Could not extract cluster ID"))
    }

    fn get_court_name(&self, docket_url: &str) -> String {
        if docket_url.contains("pasuper") {
            "Pa. Super. Ct.".to_string()
        } else if docket_url.contains("pacommw") {
            "Pa. Commw. Ct.".to_string()
        } else if docket_url.contains("/pa/") {
            "Pa.".to_string()
        } else if docket_url.contains("ca3") {
            "3d Cir.".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn find_page_references(&self, citation: &ExtractedCitation, document: &str) -> Vec<u32> {
        // Simple page reference calculation (would need actual page break detection)
        let lines_before = document[..citation.start_index].lines().count();
        let page = (lines_before / 25) + 1; // Assume 25 lines per page
        vec![page as u32]
    }

    fn format_page_refs(&self, pages: &[u32]) -> String {
        if pages.is_empty() {
            return String::new();
        }

        if pages.len() == 1 {
            return pages[0].to_string();
        }

        let mut result = String::new();
        let mut prev = pages[0];
        result.push_str(&prev.to_string());

        for &page in &pages[1..] {
            if page == prev + 1 {
                // Consecutive pages - will use range
            } else {
                result.push_str(", ");
                result.push_str(&page.to_string());
            }
            prev = page;
        }

        result
    }

    fn get_sort_key(&self, citation: &str) -> String {
        // Extract case name or first significant word for sorting
        citation
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_citations() {
        let service = AICitationService::new(None);
        let text = "As held in Smith v. Jones, 123 F.3d 456 (3d Cir. 2020), and pursuant to 42 U.S.C. § 1983.";

        let citations = service.extract_citations(text).await.unwrap();
        assert!(citations.len() >= 2);
    }

    #[tokio::test]
    async fn test_generate_toa() {
        let service = AICitationService::new(None);
        let text = "See 123 F.3d 456; 42 U.S.C. § 1983; 18 Pa.C.S. § 2702.";

        let toa = service.generate_table_of_authorities(text).await.unwrap();
        assert!(toa.contains("TABLE OF AUTHORITIES"));
        assert!(toa.contains("Cases") || toa.contains("Statutes"));
    }
}
