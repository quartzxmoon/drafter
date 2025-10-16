// AI Legal Research Assistant
// Combines multiple data sources with AI-powered analysis

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use crate::providers::courtlistener::CourtListenerProvider;
use crate::providers::govinfo::GovInfoProvider;
use crate::providers::recap::RecapProvider;
use crate::providers::harvard_caselaw::HarvardCaselawProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQuery {
    pub query: String,
    pub jurisdiction: Option<String>,
    pub court: Option<String>,
    pub date_range: Option<DateRange>,
    pub document_types: Vec<DocumentType>,
    pub max_results: usize,
    pub include_citations: bool,
    pub include_related_cases: bool,
    pub include_statutes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Opinion,
    Statute,
    Regulation,
    Brief,
    Motion,
    Order,
    Docket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub id: String,
    pub query: String,
    pub results: Vec<ResearchItem>,
    pub citations: Vec<CitationResult>,
    pub related_cases: Vec<RelatedCase>,
    pub statutes: Vec<StatuteResult>,
    pub summary: ResearchSummary,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchItem {
    pub id: String,
    pub title: String,
    pub court: Option<String>,
    pub date: Option<String>,
    pub citation: Option<String>,
    pub excerpt: String,
    pub full_text: Option<String>,
    pub relevance_score: f32,
    pub source: DataSource,
    pub url: Option<String>,
    pub document_type: DocumentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    CourtListener,
    HarvardCaselaw,
    GovInfo,
    RECAP,
    LocalCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationResult {
    pub citation: String,
    pub case_name: String,
    pub year: Option<String>,
    pub court: Option<String>,
    pub cited_by_count: u32,
    pub importance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedCase {
    pub case_name: String,
    pub citation: String,
    pub relationship: RelationshipType,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    CitesDirect,
    CitedBy,
    Overruled,
    Followed,
    Distinguished,
    Similar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteResult {
    pub title: String,
    pub section: String,
    pub text: String,
    pub jurisdiction: String,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSummary {
    pub total_results: usize,
    pub sources_searched: Vec<DataSource>,
    pub key_findings: Vec<String>,
    pub suggested_citations: Vec<String>,
    pub research_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalMemo {
    pub title: String,
    pub question_presented: String,
    pub brief_answer: String,
    pub statement_of_facts: String,
    pub discussion: String,
    pub conclusion: String,
    pub citations: Vec<String>,
}

pub struct AILegalResearchService {
    db: SqlitePool,
    courtlistener: CourtListenerProvider,
    govinfo: GovInfoProvider,
    recap: RecapProvider,
    harvard: HarvardCaselawProvider,
}

impl AILegalResearchService {
    pub fn new(
        db: SqlitePool,
        courtlistener_token: Option<String>,
        govinfo_token: Option<String>,
        harvard_token: Option<String>,
    ) -> Self {
        Self {
            db: db.clone(),
            courtlistener: CourtListenerProvider::new(courtlistener_token.clone()),
            govinfo: GovInfoProvider::new(govinfo_token),
            recap: RecapProvider::new(courtlistener_token),
            harvard: HarvardCaselawProvider::new(harvard_token),
        }
    }

    /// Perform comprehensive legal research across all sources
    pub async fn research(&self, query: ResearchQuery) -> Result<ResearchResult> {
        let start_time = std::time::Instant::now();
        info!("Starting legal research for query: {}", query.query);

        let mut all_results = Vec::new();
        let mut sources_searched = Vec::new();

        // Search CourtListener
        if query.document_types.contains(&DocumentType::Opinion) {
            match self.search_courtlistener(&query).await {
                Ok(mut results) => {
                    info!("Found {} results from CourtListener", results.len());
                    all_results.append(&mut results);
                    sources_searched.push(DataSource::CourtListener);
                }
                Err(e) => {
                    warn!("CourtListener search failed: {}", e);
                }
            }
        }

        // Search Harvard Caselaw
        if query.document_types.contains(&DocumentType::Opinion) {
            match self.search_harvard(&query).await {
                Ok(mut results) => {
                    info!("Found {} results from Harvard Caselaw", results.len());
                    all_results.append(&mut results);
                    sources_searched.push(DataSource::HarvardCaselaw);
                }
                Err(e) => {
                    warn!("Harvard Caselaw search failed: {}", e);
                }
            }
        }

        // Search GovInfo for statutes
        if query.include_statutes {
            match self.search_govinfo(&query).await {
                Ok(mut results) => {
                    info!("Found {} results from GovInfo", results.len());
                    all_results.append(&mut results);
                    sources_searched.push(DataSource::GovInfo);
                }
                Err(e) => {
                    warn!("GovInfo search failed: {}", e);
                }
            }
        }

        // Search RECAP for dockets
        if query.document_types.contains(&DocumentType::Docket) {
            match self.search_recap(&query).await {
                Ok(mut results) => {
                    info!("Found {} results from RECAP", results.len());
                    all_results.append(&mut results);
                    sources_searched.push(DataSource::RECAP);
                }
                Err(e) => {
                    warn!("RECAP search failed: {}", e);
                }
            }
        }

        // Search local cache
        match self.search_local_cache(&query).await {
            Ok(mut results) => {
                info!("Found {} results from local cache", results.len());
                all_results.append(&mut results);
                if !results.is_empty() {
                    sources_searched.push(DataSource::LocalCache);
                }
            }
            Err(e) => {
                warn!("Local cache search failed: {}", e);
            }
        }

        // Rank and deduplicate results
        all_results = self.rank_results(all_results, &query);
        all_results.truncate(query.max_results);

        // Extract citations if requested
        let citations = if query.include_citations {
            self.extract_citations(&all_results).await?
        } else {
            Vec::new()
        };

        // Find related cases if requested
        let related_cases = if query.include_related_cases {
            self.find_related_cases(&all_results).await?
        } else {
            Vec::new()
        };

        // Find relevant statutes
        let statutes = if query.include_statutes {
            self.find_relevant_statutes(&query).await?
        } else {
            Vec::new()
        };

        // Generate summary
        let summary = self.generate_summary(
            &all_results,
            &citations,
            &sources_searched,
            start_time.elapsed().as_millis() as u64,
        );

        let result = ResearchResult {
            id: uuid::Uuid::new_v4().to_string(),
            query: query.query.clone(),
            results: all_results,
            citations,
            related_cases,
            statutes,
            summary,
            created_at: Utc::now(),
        };

        // Save research to database
        self.save_research(&result).await?;

        info!(
            "Research completed in {:?} with {} total results",
            start_time.elapsed(),
            result.results.len()
        );

        Ok(result)
    }

    async fn search_courtlistener(&self, query: &ResearchQuery) -> Result<Vec<ResearchItem>> {
        let mut results = Vec::new();

        let court = query.jurisdiction.as_deref();
        let search_results = self.courtlistener
            .search_opinions(&query.query, court, Some(50))
            .await?;

        for opinion in search_results.results.iter().take(20) {
            results.push(ResearchItem {
                id: opinion.id.to_string(),
                title: opinion.case_name.clone(),
                court: Some(opinion.court.clone()),
                date: Some(opinion.date_filed.clone()),
                citation: opinion.citation.first().cloned(),
                excerpt: opinion.snippet.clone().unwrap_or_default(),
                full_text: None,
                relevance_score: 0.8,
                source: DataSource::CourtListener,
                url: Some(format!("https://www.courtlistener.com{}", opinion.absolute_url)),
                document_type: DocumentType::Opinion,
            });
        }

        Ok(results)
    }

    async fn search_harvard(&self, query: &ResearchQuery) -> Result<Vec<ResearchItem>> {
        let mut results = Vec::new();

        let jurisdiction = query.jurisdiction.as_deref();
        let search_results = self.harvard
            .search_cases(
                Some(&query.query),
                jurisdiction,
                None,
                None,
                query.date_range.as_ref().map(|r| r.start.as_str()),
                query.date_range.as_ref().map(|r| r.end.as_str()),
                None,
                None,
                true,
                Some(50),
            )
            .await?;

        for case in search_results.results.iter().take(20) {
            let excerpt = case.preview.join(" ");
            let relevance = self.harvard.get_importance_score(&case).unwrap_or(0.5);

            results.push(ResearchItem {
                id: case.id.to_string(),
                title: case.name.clone(),
                court: Some(case.court.name.clone()),
                date: Some(case.decision_date.clone()),
                citation: case.citations.first().map(|c| c.cite.clone()),
                excerpt: excerpt[..excerpt.len().min(500)].to_string(),
                full_text: self.harvard.get_case_text(&case),
                relevance_score: relevance,
                source: DataSource::HarvardCaselaw,
                url: Some(case.frontend_url.clone()),
                document_type: DocumentType::Opinion,
            });
        }

        Ok(results)
    }

    async fn search_govinfo(&self, query: &ResearchQuery) -> Result<Vec<ResearchItem>> {
        let mut results = Vec::new();

        // Search US Code
        let usc_results = self.govinfo
            .search_collection("USCODE", &query.query, Some(20))
            .await?;

        for result in usc_results.results.iter().take(10) {
            results.push(ResearchItem {
                id: result.package_id.clone(),
                title: result.title.clone(),
                court: None,
                date: Some(result.date_issued.clone()),
                citation: Some(result.package_id.clone()),
                excerpt: result.summary.clone().unwrap_or_default(),
                full_text: None,
                relevance_score: 0.7,
                source: DataSource::GovInfo,
                url: Some(result.package_link.clone()),
                document_type: DocumentType::Statute,
            });
        }

        Ok(results)
    }

    async fn search_recap(&self, query: &ResearchQuery) -> Result<Vec<ResearchItem>> {
        let mut results = Vec::new();

        let court = query.court.as_deref();
        let dockets = self.recap
            .search_dockets(&query.query, court, Some(20))
            .await?;

        for docket in dockets {
            results.push(ResearchItem {
                id: docket.id.to_string(),
                title: docket.case_name.clone(),
                court: Some(docket.court_id.clone()),
                date: docket.date_filed.map(|d| d.to_rfc3339()),
                citation: Some(docket.docket_number.clone()),
                excerpt: docket.case_name_short.unwrap_or_default(),
                full_text: None,
                relevance_score: 0.6,
                source: DataSource::RECAP,
                url: Some(format!("https://www.courtlistener.com/docket/{}/", docket.id)),
                document_type: DocumentType::Docket,
            });
        }

        Ok(results)
    }

    async fn search_local_cache(&self, query: &ResearchQuery) -> Result<Vec<ResearchItem>> {
        let results = sqlx::query!(
            r#"
            SELECT
                id,
                case_name,
                citation,
                court,
                decision_date,
                plain_text,
                source
            FROM case_law
            WHERE case_law MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
            query.query,
            query.max_results
        )
        .fetch_all(&self.db)
        .await?;

        let items = results
            .into_iter()
            .map(|row| {
                let excerpt = row.plain_text
                    .as_ref()
                    .map(|text| text.chars().take(500).collect::<String>())
                    .unwrap_or_default();

                ResearchItem {
                    id: row.id,
                    title: row.case_name,
                    court: Some(row.court),
                    date: Some(row.decision_date),
                    citation: Some(row.citation),
                    excerpt,
                    full_text: row.plain_text,
                    relevance_score: 0.9, // Local cache is highly relevant
                    source: DataSource::LocalCache,
                    url: None,
                    document_type: DocumentType::Opinion,
                }
            })
            .collect();

        Ok(items)
    }

    fn rank_results(&self, mut results: Vec<ResearchItem>, query: &ResearchQuery) -> Vec<ResearchItem> {
        // Boost scores for matching jurisdiction
        if let Some(jurisdiction) = &query.jurisdiction {
            for result in &mut results {
                if let Some(court) = &result.court {
                    if court.to_lowercase().contains(&jurisdiction.to_lowercase()) {
                        result.relevance_score *= 1.5;
                    }
                }
            }
        }

        // Boost recent cases
        for result in &mut results {
            if let Some(date_str) = &result.date {
                if date_str.starts_with("202") {
                    result.relevance_score *= 1.2;
                }
            }
        }

        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Deduplicate by title
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.title.clone()));

        results
    }

    async fn extract_citations(&self, results: &[ResearchItem]) -> Result<Vec<CitationResult>> {
        let mut citations = Vec::new();

        for result in results {
            if let Some(citation) = &result.citation {
                citations.push(CitationResult {
                    citation: citation.clone(),
                    case_name: result.title.clone(),
                    year: result.date.as_ref().and_then(|d| d.split('-').next().map(|s| s.to_string())),
                    court: result.court.clone(),
                    cited_by_count: 0, // Would need to query for this
                    importance_score: result.relevance_score,
                });
            }
        }

        Ok(citations)
    }

    async fn find_related_cases(&self, results: &[ResearchItem]) -> Result<Vec<RelatedCase>> {
        let mut related = Vec::new();

        // For now, return similar cases from results
        for (i, result) in results.iter().enumerate() {
            if i < 5 {
                if let Some(citation) = &result.citation {
                    related.push(RelatedCase {
                        case_name: result.title.clone(),
                        citation: citation.clone(),
                        relationship: RelationshipType::Similar,
                        relevance: result.relevance_score * 0.8,
                    });
                }
            }
        }

        Ok(related)
    }

    async fn find_relevant_statutes(&self, query: &ResearchQuery) -> Result<Vec<StatuteResult>> {
        let govinfo_results = self.search_govinfo(query).await?;

        let statutes = govinfo_results
            .into_iter()
            .filter(|r| matches!(r.document_type, DocumentType::Statute))
            .map(|r| StatuteResult {
                title: r.title.clone(),
                section: r.citation.unwrap_or_default(),
                text: r.excerpt.clone(),
                jurisdiction: "Federal".to_string(),
                last_updated: r.date,
            })
            .collect();

        Ok(statutes)
    }

    fn generate_summary(
        &self,
        results: &[ResearchItem],
        citations: &[CitationResult],
        sources: &[DataSource],
        time_ms: u64,
    ) -> ResearchSummary {
        let key_findings = results
            .iter()
            .take(5)
            .map(|r| format!("{} - {}", r.title, r.court.as_ref().unwrap_or(&"Unknown Court".to_string())))
            .collect();

        let suggested_citations = citations
            .iter()
            .take(10)
            .map(|c| c.citation.clone())
            .collect();

        ResearchSummary {
            total_results: results.len(),
            sources_searched: sources.to_vec(),
            key_findings,
            suggested_citations,
            research_time_ms: time_ms,
        }
    }

    async fn save_research(&self, result: &ResearchResult) -> Result<()> {
        let results_json = serde_json::to_string(&result.results)?;
        let citations_json = serde_json::to_string(&result.citations)?;

        sqlx::query!(
            r#"
            INSERT INTO research_history (
                id,
                query,
                results,
                citations,
                total_results,
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
            result.id,
            result.query,
            results_json,
            citations_json,
            result.summary.total_results,
            result.created_at
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Generate a legal memorandum from research results
    pub async fn generate_memo(&self, research: &ResearchResult, facts: &str) -> Result<LegalMemo> {
        // Extract key citations
        let citations: Vec<String> = research.citations
            .iter()
            .map(|c| format!("{}, {}", c.case_name, c.citation))
            .collect();

        // Build discussion from top results
        let mut discussion = String::new();
        for (i, result) in research.results.iter().take(5).enumerate() {
            discussion.push_str(&format!("\n{}. {}\n\n", i + 1, result.title));
            discussion.push_str(&result.excerpt);
            discussion.push_str("\n\n");
        }

        Ok(LegalMemo {
            title: format!("Legal Memorandum: {}", research.query),
            question_presented: format!("Whether {}?", research.query),
            brief_answer: "Based on the research, the analysis suggests...".to_string(),
            statement_of_facts: facts.to_string(),
            discussion,
            conclusion: "For the foregoing reasons...".to_string(),
            citations,
        })
    }
}
