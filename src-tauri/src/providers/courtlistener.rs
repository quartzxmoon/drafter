// CourtListener API Integration
// Provides access to millions of US court opinions and PACER documents

use crate::providers::rate_limiter::RateLimiter;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

const COURTLISTENER_BASE_URL: &str = "https://www.courtlistener.com/api/rest/v3";

pub struct CourtListenerProvider {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
    api_token: String,
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionSearchResult {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<Opinion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opinion {
    pub id: u32,
    pub absolute_url: String,
    pub cluster: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub author_str: Option<String>,
    pub per_curiam: bool,
    pub joined_by_str: Option<String>,
    pub type_: String,
    pub page_count: Option<u32>,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
    pub plain_text: Option<String>,
    pub html: Option<String>,
    pub html_lawbox: Option<String>,
    pub html_columbia: Option<String>,
    pub html_anon_2020: Option<String>,
    pub xml_harvard: Option<String>,
    pub html_with_citations: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionCluster {
    pub id: u32,
    pub absolute_url: String,
    pub panel: Vec<Judge>,
    pub non_participating_judges: Vec<Judge>,
    pub case_name: String,
    pub case_name_short: String,
    pub case_name_full: String,
    pub federal_cite_one: Option<String>,
    pub federal_cite_two: Option<String>,
    pub federal_cite_three: Option<String>,
    pub state_cite_one: Option<String>,
    pub state_cite_two: Option<String>,
    pub state_cite_three: Option<String>,
    pub state_cite_regional: Option<String>,
    pub specialty_cite_one: Option<String>,
    pub scotus_early_cite: Option<String>,
    pub lexis_cite: Option<String>,
    pub westlaw_cite: Option<String>,
    pub neutral_cite: Option<String>,
    pub scdb_id: Option<String>,
    pub scdb_decision_direction: Option<String>,
    pub scdb_votes_majority: Option<u32>,
    pub scdb_votes_minority: Option<u32>,
    pub date_filed: String,
    pub date_filed_is_approximate: bool,
    pub slug: String,
    pub citation_count: u32,
    pub precedential_status: String,
    pub date_blocked: Option<DateTime<Utc>>,
    pub blocked: bool,
    pub docket: String,
    pub sub_opinions: Vec<String>,
    pub source: String,
    pub procedural_history: Option<String>,
    pub attorneys: Option<String>,
    pub nature_of_suit: Option<String>,
    pub posture: Option<String>,
    pub syllabus: Option<String>,
    pub headnotes: Option<String>,
    pub summary: Option<String>,
    pub disposition: Option<String>,
    pub history: Option<String>,
    pub other_dates: Option<String>,
    pub cross_reference: Option<String>,
    pub correction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judge {
    pub id: u32,
    pub name_first: String,
    pub name_middle: Option<String>,
    pub name_last: String,
    pub name_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Docket {
    pub id: u32,
    pub absolute_url: String,
    pub court: String,
    pub audio_files: Vec<String>,
    pub clusters: Vec<String>,
    pub docket_number: String,
    pub docket_number_core: String,
    pub case_name: String,
    pub case_name_short: String,
    pub case_name_full: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub date_last_index: Option<DateTime<Utc>>,
    pub date_cert_granted: Option<String>,
    pub date_cert_denied: Option<String>,
    pub date_argued: Option<String>,
    pub date_reargued: Option<String>,
    pub date_reargument_denied: Option<String>,
    pub date_filed: Option<String>,
    pub date_terminated: Option<String>,
    pub date_last_filing: Option<String>,
    pub assigned_to_str: Option<String>,
    pub referred_to_str: Option<String>,
    pub court_id: String,
    pub pacer_case_id: Option<String>,
    pub cause: Option<String>,
    pub nature_of_suit: Option<String>,
    pub jury_demand: Option<String>,
    pub jurisdiction_type: Option<String>,
    pub appellate_fee_status: Option<String>,
    pub appellate_case_type_information: Option<String>,
    pub mdl_status: Option<String>,
    pub filepath_local: Option<String>,
    pub filepath_ia: Option<String>,
    pub filepath_ia_json: Option<String>,
    pub ia_upload_failure_count: Option<u32>,
    pub ia_needs_upload: Option<bool>,
    pub ia_date_first_change: Option<DateTime<Utc>>,
    pub view_count: u32,
    pub date_blocked: Option<DateTime<Utc>>,
    pub blocked: bool,
    pub appeal_from_str: Option<String>,
    pub assigned_to: Option<u32>,
    pub referred_to: Option<u32>,
    pub panel: Vec<u32>,
    pub tags: Vec<String>,
    pub html_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub case_name: Option<String>,
    pub court: Option<String>,
    pub docket_number: Option<String>,
    pub filed_after: Option<String>,
    pub filed_before: Option<String>,
    pub cited_gt: Option<u32>,
    pub cited_lt: Option<u32>,
    pub status: Option<String>,
    pub order_by: Option<String>,
}

impl CourtListenerProvider {
    pub fn new(api_token: String) -> Self {
        let client = Client::builder()
            .user_agent("PA-eDocket-Desktop/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = Arc::new(RateLimiter::new(5.0)); // 5 requests per second

        Self {
            client,
            rate_limiter,
            api_token,
        }
    }

    // ========================================================================
    // Search Operations
    // ========================================================================

    #[instrument(skip(self, query))]
    pub async fn search_opinions(&self, query: SearchQuery) -> Result<OpinionSearchResult> {
        info!("Searching CourtListener opinions");

        self.rate_limiter.wait().await;

        let mut url = format!("{}/search/", COURTLISTENER_BASE_URL);
        let mut params = vec![];

        if let Some(q) = &query.q {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(case_name) = &query.case_name {
            params.push(format!("case_name={}", urlencoding::encode(case_name)));
        }
        if let Some(court) = &query.court {
            params.push(format!("court={}", court));
        }
        if let Some(docket_number) = &query.docket_number {
            params.push(format!("docket_number={}", urlencoding::encode(docket_number)));
        }
        if let Some(filed_after) = &query.filed_after {
            params.push(format!("filed_after={}", filed_after));
        }
        if let Some(filed_before) = &query.filed_before {
            params.push(format!("filed_before={}", filed_before));
        }
        if let Some(status) = &query.status {
            params.push(format!("status={}", status));
        }
        if let Some(order_by) = &query.order_by {
            params.push(format!("order_by={}", order_by));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        debug!("CourtListener search URL: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .context("Failed to send request to CourtListener")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("CourtListener API error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("CourtListener API error: {}", status));
        }

        let result: OpinionSearchResult = response
            .json()
            .await
            .context("Failed to parse CourtListener response")?;

        info!("Found {} opinions", result.count);
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn get_opinion(&self, opinion_id: u32) -> Result<Opinion> {
        info!("Fetching opinion: {}", opinion_id);

        self.rate_limiter.wait().await;

        let url = format!("{}/opinions/{}/", COURTLISTENER_BASE_URL, opinion_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .context("Failed to fetch opinion")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch opinion: {}", response.status()));
        }

        let opinion: Opinion = response
            .json()
            .await
            .context("Failed to parse opinion")?;

        Ok(opinion)
    }

    #[instrument(skip(self))]
    pub async fn get_opinion_cluster(&self, cluster_id: u32) -> Result<OpinionCluster> {
        info!("Fetching opinion cluster: {}", cluster_id);

        self.rate_limiter.wait().await;

        let url = format!("{}/clusters/{}/", COURTLISTENER_BASE_URL, cluster_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .context("Failed to fetch opinion cluster")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch cluster: {}", response.status()));
        }

        let cluster: OpinionCluster = response
            .json()
            .await
            .context("Failed to parse cluster")?;

        Ok(cluster)
    }

    #[instrument(skip(self))]
    pub async fn get_docket(&self, docket_id: u32) -> Result<Docket> {
        info!("Fetching docket: {}", docket_id);

        self.rate_limiter.wait().await;

        let url = format!("{}/dockets/{}/", COURTLISTENER_BASE_URL, docket_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .context("Failed to fetch docket")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch docket: {}", response.status()));
        }

        let docket: Docket = response
            .json()
            .await
            .context("Failed to parse docket")?;

        Ok(docket)
    }

    // ========================================================================
    // Pennsylvania-specific searches
    // ========================================================================

    pub async fn search_pa_supreme_court(&self, query: &str) -> Result<OpinionSearchResult> {
        info!("Searching PA Supreme Court: {}", query);

        self.search_opinions(SearchQuery {
            q: Some(query.to_string()),
            court: Some("pa".to_string()), // PA Supreme Court
            case_name: None,
            docket_number: None,
            filed_after: None,
            filed_before: None,
            cited_gt: None,
            cited_lt: None,
            status: Some("Precedential".to_string()),
            order_by: Some("-date_filed".to_string()),
        })
        .await
    }

    pub async fn search_pa_superior_court(&self, query: &str) -> Result<OpinionSearchResult> {
        info!("Searching PA Superior Court: {}", query);

        self.search_opinions(SearchQuery {
            q: Some(query.to_string()),
            court: Some("pasuperct".to_string()), // PA Superior Court
            case_name: None,
            docket_number: None,
            filed_after: None,
            filed_before: None,
            cited_gt: None,
            cited_lt: None,
            status: Some("Precedential".to_string()),
            order_by: Some("-date_filed".to_string()),
        })
        .await
    }

    pub async fn search_pa_commonwealth_court(&self, query: &str) -> Result<OpinionSearchResult> {
        info!("Searching PA Commonwealth Court: {}", query);

        self.search_opinions(SearchQuery {
            q: Some(query.to_string()),
            court: Some("pacommwct".to_string()), // PA Commonwealth Court
            case_name: None,
            docket_number: None,
            filed_after: None,
            filed_before: None,
            cited_gt: None,
            cited_lt: None,
            status: Some("Precedential".to_string()),
            order_by: Some("-date_filed".to_string()),
        })
        .await
    }

    pub async fn search_third_circuit(&self, query: &str) -> Result<OpinionSearchResult> {
        info!("Searching Third Circuit: {}", query);

        self.search_opinions(SearchQuery {
            q: Some(query.to_string()),
            court: Some("ca3".to_string()), // Third Circuit (covers PA)
            case_name: None,
            docket_number: None,
            filed_after: None,
            filed_before: None,
            cited_gt: None,
            cited_lt: None,
            status: Some("Precedential".to_string()),
            order_by: Some("-date_filed".to_string()),
        })
        .await
    }

    // ========================================================================
    // Citation Extraction
    // ========================================================================

    pub async fn get_citations_for_opinion(&self, opinion_id: u32) -> Result<Vec<String>> {
        info!("Fetching citations for opinion: {}", opinion_id);

        let cluster = self.get_opinion_cluster(opinion_id).await?;

        let mut citations = Vec::new();

        if let Some(cite) = cluster.federal_cite_one {
            citations.push(cite);
        }
        if let Some(cite) = cluster.state_cite_one {
            citations.push(cite);
        }
        if let Some(cite) = cluster.neutral_cite {
            citations.push(cite);
        }
        if let Some(cite) = cluster.westlaw_cite {
            citations.push(cite);
        }
        if let Some(cite) = cluster.lexis_cite {
            citations.push(cite);
        }

        Ok(citations)
    }

    // ========================================================================
    // Bulk Data Operations
    // ========================================================================

    pub async fn bulk_download_recent(&self, court: &str, days: u32) -> Result<Vec<Opinion>> {
        info!("Bulk downloading recent opinions from {}", court);

        let date_after = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let date_str = date_after.format("%Y-%m-%d").to_string();

        let result = self.search_opinions(SearchQuery {
            q: None,
            case_name: None,
            court: Some(court.to_string()),
            docket_number: None,
            filed_after: Some(date_str),
            filed_before: None,
            cited_gt: None,
            cited_lt: None,
            status: Some("Precedential".to_string()),
            order_by: Some("-date_filed".to_string()),
        })
        .await?;

        Ok(result.results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API token
    async fn test_search_opinions() {
        let api_token = std::env::var("COURTLISTENER_API_TOKEN").expect("API token required");
        let provider = CourtListenerProvider::new(api_token);

        let result = provider
            .search_pa_supreme_court("contract")
            .await
            .expect("Search failed");

        assert!(result.count > 0);
    }
}
