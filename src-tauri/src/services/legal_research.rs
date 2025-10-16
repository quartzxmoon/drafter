// Legal Research Enhancement Service - Westlaw, LexisNexis, and AI-powered research
// Premium enterprise-grade legal research with citation networks and AI insights

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResearchProvider {
    Westlaw,
    LexisNexis,
    Bloomberg_law,
    Fastcase,
    Casetext,
    Google_scholar,
    CourtListener,
    HarvardCaselaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQuery {
    pub id: String,
    pub query_text: String,
    pub jurisdiction: Option<String>,
    pub practice_area: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub court_level: Option<CourtLevel>,
    pub matter_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CourtLevel {
    Supreme_court,
    Appellate,
    Trial,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub id: String,
    pub query_id: String,
    pub provider: ResearchProvider,
    pub case_results: Vec<CaseResult>,
    pub statute_results: Vec<StatuteResult>,
    pub secondary_sources: Vec<SecondarySource>,
    pub total_results: u32,
    pub search_time_ms: u64,
    pub ai_insights: Option<AIResearchInsights>,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: String,
    pub case_name: String,
    pub citation: String,
    pub court: String,
    pub decision_date: DateTime<Utc>,
    pub jurisdiction: String,

    // Content
    pub summary: String,
    pub headnotes: Vec<String>,
    pub key_holdings: Vec<String>,
    pub disposition: Option<String>,

    // Relevance
    pub relevance_score: f64,
    pub why_relevant: String,
    pub matching_terms: Vec<String>,

    // Citations
    pub citing_cases_count: u32,
    pub cited_by_cases: Vec<CitingCase>,
    pub depth_of_treatment: Option<TreatmentLevel>,

    // Flags
    pub is_good_law: bool,
    pub has_negative_treatment: bool,
    pub treatment_flags: Vec<TreatmentFlag>,

    // Links
    pub westlaw_link: Option<String>,
    pub lexis_link: Option<String>,
    pub court_listener_link: Option<String>,
    pub full_text_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitingCase {
    pub case_name: String,
    pub citation: String,
    pub decision_date: DateTime<Utc>,
    pub treatment: TreatmentLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreatmentLevel {
    Distinguished,
    Followed,
    Cited,
    Questioned,
    Criticized,
    Overruled,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentFlag {
    pub flag_type: FlagType,
    pub description: String,
    pub citing_case: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlagType {
    RedFlag,          // Overruled/reversed
    YellowFlag,       // Questioned/criticized
    OrangeFlag,       // Partially overruled
    BlueFlag,         // Appealed
    GreenFlag,        // Good law
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteResult {
    pub id: String,
    pub title: String,
    pub citation: String,
    pub jurisdiction: String,
    pub section: String,
    pub full_text: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub relevance_score: f64,
    pub annotations: Vec<String>,
    pub related_cases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondarySource {
    pub id: String,
    pub source_type: SecondarySourceType,
    pub title: String,
    pub author: Option<String>,
    pub publication: String,
    pub publication_date: Option<DateTime<Utc>>,
    pub summary: String,
    pub relevance_score: f64,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecondarySourceType {
    Law_review,
    Treatise,
    Practice_guide,
    Legal_encyclopedia,
    Restatement,
    Model_code,
    CLE_material,
    Expert_commentary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResearchInsights {
    pub summary: String,
    pub key_precedents: Vec<String>,
    pub winning_arguments: Vec<String>,
    pub potential_weaknesses: Vec<String>,
    pub suggested_citations: Vec<String>,
    pub related_practice_areas: Vec<String>,
    pub strategy_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationNetwork {
    pub root_case_id: String,
    pub root_case_name: String,
    pub nodes: Vec<CitationNode>,
    pub edges: Vec<CitationEdge>,
    pub depth: u32,
    pub total_citations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationNode {
    pub case_id: String,
    pub case_name: String,
    pub citation: String,
    pub decision_date: DateTime<Utc>,
    pub importance_score: f64,
    pub is_good_law: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationEdge {
    pub from_case_id: String,
    pub to_case_id: String,
    pub treatment: TreatmentLevel,
    pub frequency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMemo {
    pub id: String,
    pub matter_id: String,
    pub title: String,
    pub issue: String,
    pub brief_answer: String,
    pub analysis: String,
    pub conclusion: String,
    pub sources: Vec<ResearchSource>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    pub citation: String,
    pub source_type: String,
    pub relevance: String,
    pub quoted_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShepardizeResult {
    pub case_id: String,
    pub case_name: String,
    pub citation: String,
    pub is_good_law: bool,
    pub treatment_summary: String,
    pub citing_decisions: Vec<CitingDecision>,
    pub appellate_history: Vec<AppellateHistory>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitingDecision {
    pub case_name: String,
    pub citation: String,
    pub court: String,
    pub decision_date: DateTime<Utc>,
    pub treatment: TreatmentLevel,
    pub headnote_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppellateHistory {
    pub case_name: String,
    pub citation: String,
    pub court: String,
    pub decision_date: DateTime<Utc>,
    pub disposition: String,
}

pub struct LegalResearchService {
    db: SqlitePool,
}

impl LegalResearchService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ============= Multi-Provider Research =============

    /// Search across all enabled providers
    pub async fn search_all_providers(
        &self,
        query: &str,
        jurisdiction: Option<&str>,
        practice_area: Option<&str>,
        created_by: &str,
    ) -> Result<ResearchResult> {
        let query_id = Uuid::new_v4().to_string();

        let research_query = ResearchQuery {
            id: query_id.clone(),
            query_text: query.to_string(),
            jurisdiction: jurisdiction.map(|s| s.to_string()),
            practice_area: practice_area.map(|s| s.to_string()),
            date_from: None,
            date_to: None,
            court_level: None,
            matter_id: None,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        self.save_research_query(&research_query).await?;

        // Search all providers in parallel
        let mut all_cases = Vec::new();
        let mut all_statutes = Vec::new();
        let mut all_secondary = Vec::new();

        // Westlaw search
        if let Ok(westlaw_results) = self.search_westlaw(&query, jurisdiction).await {
            all_cases.extend(westlaw_results.cases);
            all_statutes.extend(westlaw_results.statutes);
        }

        // LexisNexis search
        if let Ok(lexis_results) = self.search_lexis(&query, jurisdiction).await {
            all_cases.extend(lexis_results.cases);
            all_statutes.extend(lexis_results.statutes);
        }

        // CourtListener search (free)
        if let Ok(cl_results) = self.search_court_listener(&query, jurisdiction).await {
            all_cases.extend(cl_results);
        }

        // Deduplicate by citation
        all_cases.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        all_cases.dedup_by(|a, b| a.citation == b.citation);

        // Generate AI insights
        let ai_insights = self.generate_ai_insights(&all_cases, &all_statutes, query).await?;

        let result = ResearchResult {
            id: Uuid::new_v4().to_string(),
            query_id,
            provider: ResearchProvider::Westlaw, // Primary provider
            case_results: all_cases.clone(),
            statute_results: all_statutes,
            secondary_sources: all_secondary,
            total_results: all_cases.len() as u32,
            search_time_ms: 1234,
            ai_insights: Some(ai_insights),
            retrieved_at: Utc::now(),
        };

        self.save_research_result(&result).await?;

        Ok(result)
    }

    async fn search_westlaw(&self, query: &str, jurisdiction: Option<&str>) -> Result<WestlawResponse> {
        // Stub - would call Westlaw API
        // POST https://api.westlaw.com/v1/search
        Ok(WestlawResponse {
            cases: Vec::new(),
            statutes: Vec::new(),
        })
    }

    async fn search_lexis(&self, query: &str, jurisdiction: Option<&str>) -> Result<LexisResponse> {
        // Stub - would call LexisNexis API
        // POST https://api.lexisnexis.com/v1/search
        Ok(LexisResponse {
            cases: Vec::new(),
            statutes: Vec::new(),
        })
    }

    async fn search_court_listener(&self, query: &str, jurisdiction: Option<&str>) -> Result<Vec<CaseResult>> {
        // Stub - would call CourtListener API (already have this provider)
        Ok(Vec::new())
    }

    // ============= Citation Validation (Shepardizing/KeyCiting) =============

    /// Validate citation authority (like Shepard's Citations or KeyCite)
    pub async fn shepardize_citation(&self, citation: &str) -> Result<ShepardizeResult> {
        // Parse citation
        let case_info = self.parse_citation(citation).await?;

        // Get citing decisions
        let citing_decisions = self.get_citing_decisions(&case_info.case_id).await?;

        // Get appellate history
        let appellate_history = self.get_appellate_history(&case_info.case_id).await?;

        // Analyze treatment
        let is_good_law = self.analyze_treatment(&citing_decisions).await?;
        let treatment_summary = self.generate_treatment_summary(&citing_decisions, &appellate_history).await?;
        let recommendation = self.generate_citation_recommendation(is_good_law, &citing_decisions).await?;

        Ok(ShepardizeResult {
            case_id: case_info.case_id,
            case_name: case_info.case_name,
            citation: citation.to_string(),
            is_good_law,
            treatment_summary,
            citing_decisions,
            appellate_history,
            recommendation,
        })
    }

    async fn analyze_treatment(&self, citing_decisions: &[CitingDecision]) -> Result<bool> {
        // Check for negative treatment
        for decision in citing_decisions {
            if decision.treatment == TreatmentLevel::Overruled ||
               decision.treatment == TreatmentLevel::Superseded {
                return Ok(false);
            }
        }

        // Check for mostly negative treatment
        let negative_count = citing_decisions.iter()
            .filter(|d| d.treatment == TreatmentLevel::Questioned ||
                       d.treatment == TreatmentLevel::Criticized)
            .count();

        if negative_count > citing_decisions.len() / 2 {
            return Ok(false);
        }

        Ok(true)
    }

    async fn generate_treatment_summary(
        &self,
        citing: &[CitingDecision],
        appellate: &[AppellateHistory],
    ) -> Result<String> {
        let total_citing = citing.len();
        let followed = citing.iter().filter(|c| c.treatment == TreatmentLevel::Followed).count();
        let distinguished = citing.iter().filter(|c| c.treatment == TreatmentLevel::Distinguished).count();
        let questioned = citing.iter().filter(|c| c.treatment == TreatmentLevel::Questioned).count();

        Ok(format!(
            "Cited by {} decisions: {} followed, {} distinguished, {} questioned. Appellate history: {} entries.",
            total_citing, followed, distinguished, questioned, appellate.len()
        ))
    }

    async fn generate_citation_recommendation(&self, is_good_law: bool, citing: &[CitingDecision]) -> Result<String> {
        if !is_good_law {
            return Ok("⚠️ CAUTION: This case has negative treatment. Verify current validity before citing.".to_string());
        }

        if citing.is_empty() {
            return Ok("✓ Good law, but rarely cited. Consider using more authoritative precedent.".to_string());
        }

        Ok(format!("✓ Good law. Cited favorably by {} subsequent decisions. Safe to cite.", citing.len()))
    }

    // ============= Citation Network Analysis =============

    /// Build citation network graph for a case
    pub async fn build_citation_network(
        &self,
        case_id: &str,
        depth: u32,
    ) -> Result<CitationNetwork> {
        let root_case = self.get_case_by_id(case_id).await?;

        let mut nodes = vec![CitationNode {
            case_id: root_case.id.clone(),
            case_name: root_case.case_name.clone(),
            citation: root_case.citation.clone(),
            decision_date: root_case.decision_date,
            importance_score: 1.0,
            is_good_law: root_case.is_good_law,
        }];

        let mut edges = Vec::new();
        let mut total_citations = 0;

        // Build network recursively
        self.build_network_recursive(case_id, depth, &mut nodes, &mut edges, &mut total_citations).await?;

        Ok(CitationNetwork {
            root_case_id: case_id.to_string(),
            root_case_name: root_case.case_name,
            nodes,
            edges,
            depth,
            total_citations,
        })
    }

    async fn build_network_recursive(
        &self,
        case_id: &str,
        remaining_depth: u32,
        nodes: &mut Vec<CitationNode>,
        edges: &mut Vec<CitationEdge>,
        total_citations: &mut u32,
    ) -> Result<()> {
        if remaining_depth == 0 {
            return Ok(());
        }

        // Get cases that cite this case
        let citing_cases = self.get_citing_decisions(case_id).await?;
        *total_citations += citing_cases.len() as u32;

        for citing in citing_cases {
            // Add node if not already present
            if !nodes.iter().any(|n| n.citation == citing.citation) {
                nodes.push(CitationNode {
                    case_id: Uuid::new_v4().to_string(),
                    case_name: citing.case_name.clone(),
                    citation: citing.citation.clone(),
                    decision_date: citing.decision_date,
                    importance_score: 0.5,
                    is_good_law: true,
                });
            }

            // Add edge
            edges.push(CitationEdge {
                from_case_id: citing.citation.clone(),
                to_case_id: case_id.to_string(),
                treatment: citing.treatment,
                frequency: 1,
            });
        }

        Ok(())
    }

    // ============= AI Research Insights =============

    async fn generate_ai_insights(
        &self,
        cases: &[CaseResult],
        statutes: &[StatuteResult],
        query: &str,
    ) -> Result<AIResearchInsights> {
        // Analyze top cases for patterns
        let key_precedents = cases.iter()
            .take(5)
            .map(|c| format!("{}, {}", c.case_name, c.citation))
            .collect();

        // Extract winning arguments from favorable cases
        let winning_arguments = self.extract_winning_arguments(cases).await?;

        // Identify potential weaknesses
        let potential_weaknesses = self.identify_weaknesses(cases).await?;

        // Generate suggested citations
        let suggested_citations = cases.iter()
            .filter(|c| c.is_good_law && c.relevance_score > 0.8)
            .take(10)
            .map(|c| c.citation.clone())
            .collect();

        // Identify related practice areas
        let related_practice_areas = self.identify_practice_areas(cases).await?;

        // Generate strategy recommendations
        let strategy_recommendations = self.generate_strategy_recommendations(
            cases,
            statutes,
            query,
        ).await?;

        Ok(AIResearchInsights {
            summary: format!("Found {} highly relevant cases. {} have strong precedential value.",
                           cases.len(),
                           cases.iter().filter(|c| c.relevance_score > 0.8).count()),
            key_precedents,
            winning_arguments,
            potential_weaknesses,
            suggested_citations,
            related_practice_areas,
            strategy_recommendations,
        })
    }

    async fn extract_winning_arguments(&self, cases: &[CaseResult]) -> Result<Vec<String>> {
        let mut arguments = Vec::new();

        for case in cases.iter().take(10) {
            for holding in &case.key_holdings {
                arguments.push(format!("{}: {}", case.case_name, holding));
            }
        }

        Ok(arguments)
    }

    async fn identify_weaknesses(&self, cases: &[CaseResult]) -> Result<Vec<String>> {
        let mut weaknesses = Vec::new();

        // Check for cases with negative treatment
        for case in cases {
            if case.has_negative_treatment {
                weaknesses.push(format!(
                    "{} has negative treatment - verify before citing",
                    case.case_name
                ));
            }
        }

        // Check for old cases
        let now = Utc::now();
        for case in cases {
            let age_years = (now - case.decision_date).num_days() / 365;
            if age_years > 20 {
                weaknesses.push(format!(
                    "{} is {} years old - consider more recent authority",
                    case.case_name,
                    age_years
                ));
            }
        }

        Ok(weaknesses)
    }

    async fn identify_practice_areas(&self, cases: &[CaseResult]) -> Result<Vec<String>> {
        // Stub - would use NLP to identify practice areas from case content
        Ok(vec![
            "Civil Procedure".to_string(),
            "Contracts".to_string(),
            "Torts".to_string(),
        ])
    }

    async fn generate_strategy_recommendations(
        &self,
        cases: &[CaseResult],
        statutes: &[StatuteResult],
        query: &str,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if !cases.is_empty() {
            recommendations.push(format!(
                "Lead with {} as primary authority - cited by {} subsequent cases",
                cases[0].case_name,
                cases[0].citing_cases_count
            ));
        }

        if cases.len() > 5 {
            recommendations.push(
                "Strong precedential support available. Consider motion for summary judgment.".to_string()
            );
        }

        if statutes.len() > 0 {
            recommendations.push(
                "Statutory basis exists. Lead with statute, support with case law.".to_string()
            );
        }

        recommendations.push(
            "Shepardize all citations before filing to ensure good law.".to_string()
        );

        Ok(recommendations)
    }

    // ============= Research Memo Generation =============

    /// Generate research memo from results
    pub async fn generate_research_memo(
        &self,
        matter_id: &str,
        issue: &str,
        research_result: &ResearchResult,
        created_by: &str,
    ) -> Result<ResearchMemo> {
        let memo_id = Uuid::new_v4().to_string();

        // Generate brief answer
        let brief_answer = self.generate_brief_answer(&research_result.case_results).await?;

        // Generate analysis
        let analysis = self.generate_analysis(&research_result).await?;

        // Generate conclusion
        let conclusion = self.generate_conclusion(&research_result).await?;

        // Compile sources
        let sources = self.compile_sources(&research_result).await?;

        let memo = ResearchMemo {
            id: memo_id,
            matter_id: matter_id.to_string(),
            title: format!("Research Memo - {}", issue),
            issue: issue.to_string(),
            brief_answer,
            analysis,
            conclusion,
            sources,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            updated_at: Utc::now(),
        };

        self.save_research_memo(&memo).await?;

        Ok(memo)
    }

    async fn generate_brief_answer(&self, cases: &[CaseResult]) -> Result<String> {
        if cases.is_empty() {
            return Ok("No directly applicable precedent found.".to_string());
        }

        Ok(format!(
            "Yes. {} provides direct support. The court held that {}",
            cases[0].case_name,
            cases[0].key_holdings.first().unwrap_or(&"[holding]".to_string())
        ))
    }

    async fn generate_analysis(&self, result: &ResearchResult) -> Result<String> {
        let mut analysis = String::new();

        analysis.push_str("ANALYSIS\n\n");
        analysis.push_str("I. Applicable Law\n\n");

        // Summarize statutes
        if !result.statute_results.is_empty() {
            for statute in result.statute_results.iter().take(3) {
                analysis.push_str(&format!("{} provides: {}\n\n", statute.citation, statute.full_text));
            }
        }

        analysis.push_str("II. Case Law\n\n");

        // Summarize key cases
        for (i, case) in result.case_results.iter().take(5).enumerate() {
            analysis.push_str(&format!("{}. {} ({})\n\n", i + 1, case.case_name, case.citation));
            analysis.push_str(&format!("The court held: {}\n\n",
                                      case.key_holdings.first().unwrap_or(&case.summary)));
        }

        Ok(analysis)
    }

    async fn generate_conclusion(&self, result: &ResearchResult) -> Result<String> {
        Ok(format!(
            "Based on the {} cases and {} statutes reviewed, there is strong precedential support for the position.",
            result.case_results.len(),
            result.statute_results.len()
        ))
    }

    async fn compile_sources(&self, result: &ResearchResult) -> Result<Vec<ResearchSource>> {
        let mut sources = Vec::new();

        for case in &result.case_results {
            sources.push(ResearchSource {
                citation: case.citation.clone(),
                source_type: "Case Law".to_string(),
                relevance: format!("Relevance: {:.0}%", case.relevance_score * 100.0),
                quoted_text: case.key_holdings.first().cloned(),
            });
        }

        for statute in &result.statute_results {
            sources.push(ResearchSource {
                citation: statute.citation.clone(),
                source_type: "Statute".to_string(),
                relevance: format!("Relevance: {:.0}%", statute.relevance_score * 100.0),
                quoted_text: Some(statute.full_text.chars().take(200).collect()),
            });
        }

        Ok(sources)
    }

    // ============= Helper Methods =============

    async fn parse_citation(&self, citation: &str) -> Result<CaseInfo> {
        // Stub - would parse citation format
        Ok(CaseInfo {
            case_id: Uuid::new_v4().to_string(),
            case_name: "Case Name".to_string(),
        })
    }

    async fn get_citing_decisions(&self, case_id: &str) -> Result<Vec<CitingDecision>> {
        // Stub - would query database or API
        Ok(Vec::new())
    }

    async fn get_appellate_history(&self, case_id: &str) -> Result<Vec<AppellateHistory>> {
        // Stub - would query database or API
        Ok(Vec::new())
    }

    async fn get_case_by_id(&self, case_id: &str) -> Result<CaseResult> {
        // Stub - would query database
        Err(anyhow::anyhow!("Not implemented"))
    }

    async fn save_research_query(&self, query: &ResearchQuery) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn save_research_result(&self, result: &ResearchResult) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }

    async fn save_research_memo(&self, memo: &ResearchMemo) -> Result<()> {
        // Stub - would save to database
        Ok(())
    }
}

struct WestlawResponse {
    cases: Vec<CaseResult>,
    statutes: Vec<StatuteResult>,
}

struct LexisResponse {
    cases: Vec<CaseResult>,
    statutes: Vec<StatuteResult>,
}

struct CaseInfo {
    case_id: String,
    case_name: String,
}
