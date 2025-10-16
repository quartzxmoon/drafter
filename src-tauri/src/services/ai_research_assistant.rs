// AI Legal Research Assistant
// Intelligent legal research with natural language queries and AI-powered analysis

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchQuery {
    pub id: String,
    pub query: String,
    pub context: Option<String>,
    pub jurisdiction: Option<String>,
    pub practice_area: Option<String>,
    pub date_range: Option<DateRange>,
    pub search_scope: SearchScope,
    pub result_limit: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchScope {
    CaseLaw,
    Statutes,
    Regulations,
    SecondaryAuthority,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchResult {
    pub query_id: String,
    pub results: Vec<LegalDocument>,
    pub analysis: AIAnalysis,
    pub recommendations: Vec<Recommendation>,
    pub related_queries: Vec<String>,
    pub confidence_score: f32,
    pub processing_time_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalDocument {
    pub id: String,
    pub title: String,
    pub citation: String,
    pub court: Option<String>,
    pub jurisdiction: String,
    pub date: Option<DateTime<Utc>>,
    pub relevance_score: f32,
    pub key_passages: Vec<KeyPassage>,
    pub document_type: DocumentType,
    pub summary: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPassage {
    pub text: String,
    pub relevance_score: f32,
    pub context: String,
    pub page_number: Option<u32>,
    pub paragraph_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DocumentType {
    CaseOpinion,
    Statute,
    Regulation,
    LawReview,
    TreatiseSection,
    PracticeGuide,
    FormBook,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIAnalysis {
    pub summary: String,
    pub key_legal_principles: Vec<LegalPrinciple>,
    pub conflicting_authorities: Vec<ConflictingAuthority>,
    pub trends: Vec<LegalTrend>,
    pub gaps_in_law: Vec<String>,
    pub strategic_considerations: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalPrinciple {
    pub principle: String,
    pub supporting_cases: Vec<String>,
    pub jurisdiction_scope: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictingAuthority {
    pub issue: String,
    pub majority_view: AuthorityView,
    pub minority_view: AuthorityView,
    pub trend_direction: TrendDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorityView {
    pub position: String,
    pub supporting_cases: Vec<String>,
    pub jurisdictions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrendDirection {
    TowardMajority,
    TowardMinority,
    Stable,
    Unclear,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalTrend {
    pub description: String,
    pub time_period: String,
    pub jurisdictions_affected: Vec<String>,
    pub impact_level: ImpactLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation: String,
    pub reasoning: String,
    pub priority: Priority,
    pub estimated_effort: String,
    pub potential_impact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchSession {
    pub id: String,
    pub queries: Vec<ResearchQuery>,
    pub results: Vec<ResearchResult>,
    pub notes: Vec<ResearchNote>,
    pub bookmarks: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchNote {
    pub id: String,
    pub content: String,
    pub document_id: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AILegalResearchAssistant {
    client: Client,
    openai_api_key: Option<String>,
    anthropic_api_key: Option<String>,
    search_providers: Vec<Box<dyn SearchProvider>>,
    knowledge_base: KnowledgeBase,
}

#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, scope: &SearchScope, limit: u32) -> Result<Vec<LegalDocument>>;
    fn get_name(&self) -> &str;
}

pub struct KnowledgeBase {
    legal_concepts: HashMap<String, LegalConcept>,
    jurisdiction_rules: HashMap<String, JurisdictionRules>,
    practice_areas: HashMap<String, PracticeArea>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalConcept {
    pub name: String,
    pub definition: String,
    pub related_concepts: Vec<String>,
    pub key_cases: Vec<String>,
    pub jurisdictions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JurisdictionRules {
    pub jurisdiction: String,
    pub court_hierarchy: Vec<String>,
    pub citation_format: String,
    pub procedural_rules: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PracticeArea {
    pub name: String,
    pub key_statutes: Vec<String>,
    pub leading_cases: Vec<String>,
    pub common_issues: Vec<String>,
    pub research_strategies: Vec<String>,
}

impl AILegalResearchAssistant {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            openai_api_key: None,
            anthropic_api_key: None,
            search_providers: vec![],
            knowledge_base: KnowledgeBase::new(),
        }
    }

    pub fn set_openai_key(&mut self, api_key: String) {
        self.openai_api_key = Some(api_key);
    }

    pub fn set_anthropic_key(&mut self, api_key: String) {
        self.anthropic_api_key = Some(api_key);
    }

    pub fn add_search_provider(&mut self, provider: Box<dyn SearchProvider>) {
        self.search_providers.push(provider);
    }

    pub async fn research(&self, query: ResearchQuery) -> Result<ResearchResult> {
        let start_time = std::time::Instant::now();
        
        info!("Starting AI research for query: {}", query.query);

        // Step 1: Analyze and expand the query
        let expanded_query = self.analyze_and_expand_query(&query).await?;
        
        // Step 2: Search across all providers
        let mut all_documents = Vec::new();
        for provider in &self.search_providers {
            match provider.search(&expanded_query, &query.search_scope, query.result_limit).await {
                Ok(mut docs) => all_documents.append(&mut docs),
                Err(e) => warn!("Search provider {} failed: {}", provider.get_name(), e),
            }
        }

        // Step 3: Rank and filter results
        let ranked_documents = self.rank_documents(&all_documents, &query).await?;

        // Step 4: Generate AI analysis
        let analysis = self.generate_analysis(&ranked_documents, &query).await?;

        // Step 5: Generate recommendations
        let recommendations = self.generate_recommendations(&ranked_documents, &analysis, &query).await?;

        // Step 6: Generate related queries
        let related_queries = self.generate_related_queries(&query, &analysis).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        let result = ResearchResult {
            query_id: query.id.clone(),
            results: ranked_documents,
            analysis,
            recommendations,
            related_queries,
            confidence_score: self.calculate_confidence_score(&all_documents),
            processing_time_ms: processing_time,
            created_at: Utc::now(),
        };

        info!("AI research completed in {}ms", processing_time);
        Ok(result)
    }

    async fn analyze_and_expand_query(&self, query: &ResearchQuery) -> Result<String> {
        if let Some(api_key) = &self.openai_api_key {
            let prompt = format!(
                "Analyze this legal research query and expand it with relevant legal terms, synonyms, and related concepts:\n\nQuery: {}\nContext: {}\nJurisdiction: {}\nPractice Area: {}\n\nProvide an expanded search query that will capture all relevant legal documents:",
                query.query,
                query.context.as_deref().unwrap_or("None"),
                query.jurisdiction.as_deref().unwrap_or("Any"),
                query.practice_area.as_deref().unwrap_or("General")
            );

            let response = self.call_openai_api(&prompt, 150).await?;
            Ok(response.trim().to_string())
        } else {
            // Fallback: basic query expansion using knowledge base
            Ok(self.expand_query_with_knowledge_base(&query.query))
        }
    }

    fn expand_query_with_knowledge_base(&self, query: &str) -> String {
        let mut expanded = query.to_string();
        
        // Add related legal terms from knowledge base
        for (concept_name, concept) in &self.knowledge_base.legal_concepts {
            if query.to_lowercase().contains(&concept_name.to_lowercase()) {
                for related in &concept.related_concepts {
                    if !expanded.contains(related) {
                        expanded.push_str(&format!(" OR {}", related));
                    }
                }
            }
        }

        expanded
    }

    async fn rank_documents(&self, documents: &[LegalDocument], query: &ResearchQuery) -> Result<Vec<LegalDocument>> {
        let mut ranked = documents.to_vec();
        
        // Sort by relevance score (highest first)
        ranked.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply additional ranking factors
        for doc in &mut ranked {
            let mut score = doc.relevance_score;
            
            // Boost recent documents
            if let Some(doc_date) = doc.date {
                let age_days = (Utc::now() - doc_date).num_days();
                if age_days < 365 {
                    score *= 1.1; // 10% boost for documents less than 1 year old
                }
            }
            
            // Boost documents from preferred jurisdiction
            if let Some(jurisdiction) = &query.jurisdiction {
                if doc.jurisdiction.to_lowercase().contains(&jurisdiction.to_lowercase()) {
                    score *= 1.2; // 20% boost for matching jurisdiction
                }
            }
            
            // Boost higher court decisions
            if let Some(court) = &doc.court {
                if court.to_lowercase().contains("supreme") {
                    score *= 1.3; // 30% boost for Supreme Court decisions
                } else if court.to_lowercase().contains("circuit") || court.to_lowercase().contains("appellate") {
                    score *= 1.1; // 10% boost for appellate decisions
                }
            }
            
            doc.relevance_score = score;
        }
        
        // Re-sort after applying boosts
        ranked.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        ranked.truncate(query.result_limit as usize);
        
        Ok(ranked)
    }

    async fn generate_analysis(&self, documents: &[LegalDocument], query: &ResearchQuery) -> Result<AIAnalysis> {
        if let Some(api_key) = &self.openai_api_key {
            let documents_summary = documents.iter()
                .take(10) // Limit to top 10 for analysis
                .map(|doc| format!("{}: {}", doc.citation, doc.summary))
                .collect::<Vec<_>>()
                .join("\n\n");

            let prompt = format!(
                "Analyze these legal documents in response to the research query: '{}'\n\nDocuments:\n{}\n\nProvide a comprehensive legal analysis including:\n1. Summary of key findings\n2. Main legal principles\n3. Any conflicting authorities\n4. Legal trends\n5. Gaps in the law\n6. Strategic considerations\n\nFormat as JSON with the following structure:\n{{\n  \"summary\": \"...\",\n  \"key_legal_principles\": [...],\n  \"conflicting_authorities\": [...],\n  \"trends\": [...],\n  \"gaps_in_law\": [...],\n  \"strategic_considerations\": [...],\n  \"next_steps\": [...]\n}}",
                query.query,
                documents_summary
            );

            let response = self.call_openai_api(&prompt, 1000).await?;
            
            // Parse JSON response
            match serde_json::from_str::<AIAnalysis>(&response) {
                Ok(analysis) => Ok(analysis),
                Err(_) => {
                    // Fallback to basic analysis if JSON parsing fails
                    Ok(self.generate_basic_analysis(documents, query))
                }
            }
        } else {
            Ok(self.generate_basic_analysis(documents, query))
        }
    }

    fn generate_basic_analysis(&self, documents: &[LegalDocument], _query: &ResearchQuery) -> AIAnalysis {
        let summary = if documents.is_empty() {
            "No relevant documents found for this query.".to_string()
        } else {
            format!("Found {} relevant documents. The most relevant cases include: {}", 
                documents.len(),
                documents.iter().take(3).map(|d| &d.citation).collect::<Vec<_>>().join(", ")
            )
        };

        AIAnalysis {
            summary,
            key_legal_principles: vec![],
            conflicting_authorities: vec![],
            trends: vec![],
            gaps_in_law: vec![],
            strategic_considerations: vec![],
            next_steps: vec![
                "Review the most relevant cases in detail".to_string(),
                "Check for more recent developments".to_string(),
                "Consider jurisdiction-specific variations".to_string(),
            ],
        }
    }

    async fn generate_recommendations(&self, documents: &[LegalDocument], analysis: &AIAnalysis, query: &ResearchQuery) -> Result<Vec<Recommendation>> {
        let mut recommendations = vec![
            Recommendation {
                recommendation: "Review the top-ranked cases for binding precedent".to_string(),
                reasoning: "These cases are most relevant to your query and may provide controlling authority".to_string(),
                priority: Priority::High,
                estimated_effort: "2-3 hours".to_string(),
                potential_impact: "High - may determine case outcome".to_string(),
            }
        ];

        if documents.len() > 10 {
            recommendations.push(Recommendation {
                recommendation: "Narrow your search criteria".to_string(),
                reasoning: "Large number of results suggests the query could be more specific".to_string(),
                priority: Priority::Medium,
                estimated_effort: "30 minutes".to_string(),
                potential_impact: "Medium - improves research efficiency".to_string(),
            });
        }

        if !analysis.conflicting_authorities.is_empty() {
            recommendations.push(Recommendation {
                recommendation: "Analyze circuit split or conflicting authorities".to_string(),
                reasoning: "Conflicting authorities identified that may affect case strategy".to_string(),
                priority: Priority::High,
                estimated_effort: "1-2 hours".to_string(),
                potential_impact: "High - may affect jurisdiction choice or arguments".to_string(),
            });
        }

        Ok(recommendations)
    }

    async fn generate_related_queries(&self, query: &ResearchQuery, analysis: &AIAnalysis) -> Result<Vec<String>> {
        let mut related = vec![
            format!("{} recent developments", query.query),
            format!("{} circuit split", query.query),
            format!("{} state law variations", query.query),
        ];

        // Add queries based on analysis
        for principle in &analysis.key_legal_principles {
            related.push(principle.principle.clone());
        }

        for trend in &analysis.trends {
            related.push(format!("{} trend analysis", trend.description));
        }

        related.truncate(10); // Limit to 10 related queries
        Ok(related)
    }

    async fn call_openai_api(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let api_key = self.openai_api_key.as_ref()
            .ok_or_else(|| anyhow!("OpenAI API key not set"))?;

        let request_body = serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert legal research assistant. Provide accurate, comprehensive legal analysis based on the documents and queries provided."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": max_tokens,
            "temperature": 0.3
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI response format"))?;

        Ok(content.to_string())
    }

    fn calculate_confidence_score(&self, documents: &[LegalDocument]) -> f32 {
        if documents.is_empty() {
            return 0.0;
        }

        let avg_relevance: f32 = documents.iter()
            .map(|d| d.relevance_score)
            .sum::<f32>() / documents.len() as f32;

        // Adjust confidence based on number of results
        let result_factor = if documents.len() < 5 {
            0.8 // Lower confidence with few results
        } else if documents.len() > 50 {
            0.9 // Slightly lower confidence with too many results
        } else {
            1.0
        };

        (avg_relevance * result_factor).min(1.0)
    }

    pub async fn create_research_session(&self) -> ResearchSession {
        ResearchSession {
            id: uuid::Uuid::new_v4().to_string(),
            queries: vec![],
            results: vec![],
            notes: vec![],
            bookmarks: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_note_to_session(&self, session: &mut ResearchSession, content: String, document_id: Option<String>, tags: Vec<String>) {
        let note = ResearchNote {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            document_id,
            tags,
            created_at: Utc::now(),
        };
        session.notes.push(note);
        session.updated_at = Utc::now();
    }

    pub fn bookmark_document(&self, session: &mut ResearchSession, document_id: String) {
        if !session.bookmarks.contains(&document_id) {
            session.bookmarks.push(document_id);
            session.updated_at = Utc::now();
        }
    }
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            legal_concepts: HashMap::new(),
            jurisdiction_rules: HashMap::new(),
            practice_areas: HashMap::new(),
        }
    }

    pub fn load_default_knowledge(&mut self) {
        // Load common legal concepts
        self.legal_concepts.insert("contract".to_string(), LegalConcept {
            name: "Contract".to_string(),
            definition: "A legally binding agreement between parties".to_string(),
            related_concepts: vec!["agreement".to_string(), "consideration".to_string(), "offer".to_string(), "acceptance".to_string()],
            key_cases: vec!["Carlill v. Carbolic Smoke Ball Co.".to_string()],
            jurisdictions: vec!["All".to_string()],
        });

        // Add more concepts as needed...
    }
}
