// Harvard Caselaw Access Project API Provider
// Free access to 6.7 million pages of U.S. case law
// API Docs: https://api.case.law/v1/

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use reqwest::Client;
use chrono::{DateTime, Utc, NaiveDate};

use crate::domain::*;
use crate::providers::rate_limiter::RateLimiter;

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCase {
    pub id: u64,
    pub url: String,
    pub name: String,
    pub name_abbreviation: String,
    pub decision_date: String,
    pub docket_number: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub citations: Vec<HarvardCitation>,
    pub volume: Option<HarvardVolume>,
    pub reporter: Option<HarvardReporter>,
    pub court: HarvardCourt,
    pub jurisdiction: HarvardJurisdiction,
    pub cites_to: Vec<HarvardCite>,
    pub frontend_url: String,
    pub frontend_pdf_url: Option<String>,
    pub preview: Vec<String>,
    pub analysis: HarvardAnalysis,
    pub last_updated: Option<String>,
    pub provenance: Option<String>,
    pub casebody: Option<HarvardCasebody>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCitation {
    #[serde(rename = "type")]
    pub citation_type: String,
    pub cite: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardVolume {
    pub url: String,
    pub volume_number: String,
    pub barcode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardReporter {
    pub id: u64,
    pub url: String,
    pub full_name: String,
    pub short_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCourt {
    pub id: u64,
    pub url: String,
    pub name: String,
    pub name_abbreviation: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardJurisdiction {
    pub id: u64,
    pub url: String,
    pub name: String,
    pub name_long: String,
    pub whitelisted: bool,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCite {
    pub cite: String,
    pub category: String,
    pub case_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardAnalysis {
    pub cardinality: u32,
    pub char_count: u32,
    pub ocr_confidence: Option<f32>,
    pub pagerank: Option<HarvardPagerank>,
    pub sha256: String,
    pub simhash: String,
    pub word_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardPagerank {
    pub percentile: f32,
    pub raw: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCasebody {
    pub status: String,
    pub data: Option<HarvardCasebodyData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCasebodyData {
    pub html: Option<String>,
    pub text: Option<String>,
    pub xml: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardSearchResponse {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<HarvardCase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardCourtResponse {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<HarvardCourt>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarvardJurisdictionResponse {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<HarvardJurisdiction>,
}

pub struct HarvardCaselawProvider {
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
    api_key: Option<String>,
}

impl HarvardCaselawProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(10, std::time::Duration::from_secs(1)), // 10 req/sec
            base_url: "https://api.case.law/v1".to_string(),
            api_key,
        }
    }

    /// Search cases with comprehensive filters
    pub async fn search_cases(
        &self,
        query: Option<&str>,
        jurisdiction: Option<&str>,
        court: Option<&str>,
        reporter: Option<&str>,
        decision_date_min: Option<&str>,
        decision_date_max: Option<&str>,
        docket_number: Option<&str>,
        cite: Option<&str>,
        full_case: bool,
        page_size: Option<u32>,
    ) -> Result<HarvardSearchResponse> {
        self.rate_limiter.wait().await;

        let mut params: Vec<(&str, String)> = vec![];

        if let Some(q) = query {
            params.push(("search", q.to_string()));
        }

        if let Some(j) = jurisdiction {
            params.push(("jurisdiction", j.to_string()));
        }

        if let Some(c) = court {
            params.push(("court", c.to_string()));
        }

        if let Some(r) = reporter {
            params.push(("reporter", r.to_string()));
        }

        if let Some(min) = decision_date_min {
            params.push(("decision_date_min", min.to_string()));
        }

        if let Some(max) = decision_date_max {
            params.push(("decision_date_max", max.to_string()));
        }

        if let Some(dn) = docket_number {
            params.push(("docket_number", dn.to_string()));
        }

        if let Some(c) = cite {
            params.push(("cite", c.to_string()));
        }

        if full_case {
            params.push(("full_case", "true".to_string()));
        }

        if let Some(size) = page_size {
            params.push(("page_size", size.to_string()));
        }

        let mut request = self.client
            .get(&format!("{}/cases/", self.base_url))
            .query(&params)
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Token {}", key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Harvard Caselaw API error: {}", response.status()));
        }

        let search_response: HarvardSearchResponse = response.json().await?;
        info!("Harvard Caselaw search returned {} results", search_response.count);

        Ok(search_response)
    }

    /// Get a specific case by ID with full casebody
    pub async fn get_case(&self, case_id: u64) -> Result<HarvardCase> {
        self.rate_limiter.wait().await;

        let mut request = self.client
            .get(&format!("{}/cases/{}/", self.base_url, case_id))
            .query(&[("full_case", "true")])
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Token {}", key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Harvard Caselaw API error: {}", response.status()));
        }

        let case: HarvardCase = response.json().await?;
        info!("Retrieved Harvard case: {}", case.name);

        Ok(case)
    }

    /// Get case by citation (e.g., "410 U.S. 113" for Roe v. Wade)
    pub async fn get_case_by_citation(&self, citation: &str) -> Result<Option<HarvardCase>> {
        let results = self.search_cases(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(citation),
            true,
            Some(1),
        ).await?;

        Ok(results.results.into_iter().next())
    }

    /// Search Pennsylvania cases
    pub async fn search_pennsylvania_cases(
        &self,
        query: &str,
        court: Option<&str>,
        decision_date_min: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<HarvardSearchResponse> {
        self.search_cases(
            Some(query),
            Some("pa"),
            court,
            None,
            decision_date_min,
            None,
            None,
            None,
            true,
            page_size,
        ).await
    }

    /// Get all Pennsylvania courts
    pub async fn get_pennsylvania_courts(&self) -> Result<Vec<HarvardCourt>> {
        self.rate_limiter.wait().await;

        let mut request = self.client
            .get(&format!("{}/courts/", self.base_url))
            .query(&[("jurisdiction", "pa")])
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Token {}", key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Harvard Caselaw API error: {}", response.status()));
        }

        let court_response: HarvardCourtResponse = response.json().await?;
        Ok(court_response.results)
    }

    /// Get all available jurisdictions
    pub async fn get_jurisdictions(&self) -> Result<Vec<HarvardJurisdiction>> {
        self.rate_limiter.wait().await;

        let mut request = self.client
            .get(&format!("{}/jurisdictions/", self.base_url))
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Token {}", key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Harvard Caselaw API error: {}", response.status()));
        }

        let jurisdiction_response: HarvardJurisdictionResponse = response.json().await?;
        Ok(jurisdiction_response.results)
    }

    /// Bulk download cases for a jurisdiction and date range
    pub async fn bulk_download_jurisdiction(
        &self,
        jurisdiction: &str,
        decision_date_min: &str,
        decision_date_max: &str,
        batch_size: u32,
    ) -> Result<Vec<HarvardCase>> {
        let mut all_cases = Vec::new();
        let mut page = 1;

        loop {
            self.rate_limiter.wait().await;

            let mut request = self.client
                .get(&format!("{}/cases/", self.base_url))
                .query(&[
                    ("jurisdiction", jurisdiction),
                    ("decision_date_min", decision_date_min),
                    ("decision_date_max", decision_date_max),
                    ("full_case", "true"),
                    ("page_size", &batch_size.to_string()),
                    ("page", &page.to_string()),
                ])
                .header("User-Agent", "PA-eDocket-Desktop/1.0");

            if let Some(key) = &self.api_key {
                request = request.header("Authorization", format!("Token {}", key));
            }

            let response = request.send().await?;

            if !response.status().is_success() {
                return Err(anyhow!("Harvard Caselaw API error: {}", response.status()));
            }

            let search_response: HarvardSearchResponse = response.json().await?;

            if search_response.results.is_empty() {
                break;
            }

            all_cases.extend(search_response.results);

            if search_response.next.is_none() {
                break;
            }

            page += 1;
            info!("Downloaded page {} ({} cases so far)", page, all_cases.len());
        }

        info!("Bulk download complete: {} total cases", all_cases.len());
        Ok(all_cases)
    }

    /// Extract full text from case
    pub fn get_case_text(&self, case: &HarvardCase) -> Option<String> {
        case.casebody.as_ref()
            .and_then(|cb| cb.data.as_ref())
            .and_then(|data| data.text.clone())
    }

    /// Extract HTML from case
    pub fn get_case_html(&self, case: &HarvardCase) -> Option<String> {
        case.casebody.as_ref()
            .and_then(|cb| cb.data.as_ref())
            .and_then(|data| data.html.clone())
    }

    /// Get Bluebook citation from Harvard case
    pub fn get_bluebook_citation(&self, case: &HarvardCase) -> String {
        if let Some(official) = case.citations.iter().find(|c| c.citation_type == "official") {
            format!("{}, {} ({})",
                case.name_abbreviation,
                official.cite,
                case.decision_date.split('-').next().unwrap_or("")
            )
        } else if let Some(first_cite) = case.citations.first() {
            format!("{}, {} ({})",
                case.name_abbreviation,
                first_cite.cite,
                case.decision_date.split('-').next().unwrap_or("")
            )
        } else {
            format!("{} ({})",
                case.name_abbreviation,
                case.decision_date.split('-').next().unwrap_or("")
            )
        }
    }

    /// Find cited cases (shepardize)
    pub fn get_cited_cases(&self, case: &HarvardCase) -> Vec<String> {
        case.cites_to.iter()
            .map(|cite| cite.cite.clone())
            .collect()
    }

    /// Get case importance score based on PageRank
    pub fn get_importance_score(&self, case: &HarvardCase) -> Option<f32> {
        case.analysis.pagerank.as_ref().map(|pr| pr.percentile)
    }

    /// Convert Harvard case to domain Citation
    pub fn convert_to_citation(&self, case: &HarvardCase) -> Citation {
        Citation {
            id: case.id.to_string(),
            citation_type: CitationType::Case,
            case_name: Some(case.name.clone()),
            reporter: case.reporter.as_ref().map(|r| r.full_name.clone()),
            volume: case.volume.as_ref().map(|v| v.volume_number.clone()),
            page: case.first_page.clone(),
            year: case.decision_date.split('-').next().map(|s| s.to_string()),
            court: Some(case.court.name.clone()),
            docket_number: case.docket_number.clone(),
            full_citation: self.get_bluebook_citation(case),
            url: Some(case.frontend_url.clone()),
            weight: self.get_importance_score(case).map(|s| s as f64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harvard_search_pa() {
        let provider = HarvardCaselawProvider::new(None);
        let result = provider.search_pennsylvania_cases(
            "contract",
            None,
            Some("2020-01-01"),
            Some(5),
        ).await;

        match result {
            Ok(response) => {
                assert!(response.results.len() <= 5);
                println!("Found {} PA cases", response.results.len());

                for case in &response.results {
                    println!("  - {} ({})", case.name, case.decision_date);
                }
            }
            Err(e) => {
                println!("Search failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_case_by_citation() {
        let provider = HarvardCaselawProvider::new(None);

        // Roe v. Wade
        let result = provider.get_case_by_citation("410 U.S. 113").await;

        match result {
            Ok(Some(case)) => {
                println!("Found: {}", case.name);
                assert!(case.name.contains("Roe"));
            }
            Ok(None) => println!("Case not found"),
            Err(e) => println!("Error: {}", e),
        }
    }
}
