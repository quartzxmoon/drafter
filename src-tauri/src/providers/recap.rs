// Free.Law RECAP Provider
// Integration with RECAP Archive for federal court documents

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};
use reqwest::Client;
use chrono::{DateTime, Utc};

use crate::domain::*;
use crate::providers::rate_limiter::RateLimiter;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapDocument {
    pub id: u64,
    pub docket_id: u64,
    pub document_number: Option<String>,
    pub attachment_number: Option<u32>,
    pub description: Option<String>,
    pub filepath_local: Option<String>,
    pub filepath_ia: Option<String>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub sha1: Option<String>,
    pub page_count: Option<u32>,
    pub file_size: Option<u64>,
    pub thumbnail: Option<String>,
    pub thumbnail_status: Option<String>,
    pub plain_text: Option<String>,
    pub ocr_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapDocket {
    pub id: u64,
    pub court_id: String,
    pub docket_number: String,
    pub case_name: String,
    pub case_name_short: Option<String>,
    pub slug: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub date_filed: Option<DateTime<Utc>>,
    pub date_terminated: Option<DateTime<Utc>>,
    pub assigned_to: Option<String>,
    pub referred_to: Option<String>,
    pub nature_of_suit: Option<String>,
    pub cause: Option<String>,
    pub jury_demand: Option<String>,
    pub jurisdiction_type: Option<String>,
    pub appellate_fee_status: Option<String>,
    pub appellate_case_type_information: Option<String>,
    pub mdl_status: Option<String>,
    pub filepath_local: Option<String>,
    pub filepath_ia: Option<String>,
    pub ia_upload_failure_count: Option<u32>,
    pub ia_needs_upload: Option<bool>,
    pub ia_date_first_change: Option<DateTime<Utc>>,
    pub view_count: u32,
    pub date_last_index: Option<DateTime<Utc>>,
    pub date_cert_granted: Option<DateTime<Utc>>,
    pub date_cert_denied: Option<DateTime<Utc>>,
    pub date_argued: Option<DateTime<Utc>>,
    pub date_reargued: Option<DateTime<Utc>>,
    pub date_reargument_denied: Option<DateTime<Utc>>,
    pub parties: Vec<RecapParty>,
    pub docket_entries: Vec<RecapDocketEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapParty {
    pub id: u64,
    pub name: String,
    pub extra_info: Option<String>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapDocketEntry {
    pub id: u64,
    pub date_filed: Option<DateTime<Utc>>,
    pub entry_number: Option<u32>,
    pub description: String,
    pub recap_documents: Vec<RecapDocument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapSearchResponse {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<RecapSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecapSearchResult {
    pub id: u64,
    pub court_id: String,
    pub docket_number: String,
    pub case_name: String,
    pub date_filed: Option<DateTime<Utc>>,
    pub absolute_url: String,
}

pub struct RecapProvider {
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
    api_token: Option<String>,
}

impl RecapProvider {
    pub fn new(api_token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(10, std::time::Duration::from_secs(1)), // 10 requests per second
            base_url: "https://www.courtlistener.com/api/rest/v3".to_string(),
            api_token,
        }
    }

    pub async fn search_dockets(&self, query: &str, court: Option<&str>, limit: Option<u32>) -> Result<RecapSearchResponse> {
        self.rate_limiter.wait().await;

        let mut params = vec![
            ("q", query),
            ("format", "json"),
        ];

        if let Some(court_id) = court {
            params.push(("court", court_id));
        }

        if let Some(limit_val) = limit {
            params.push(("page_size", &limit_val.to_string()));
        }

        let mut request = self.client
            .get(&format!("{}/dockets/", self.base_url))
            .query(&params)
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(token) = &self.api_token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("RECAP API error: {}", response.status()));
        }

        let search_response: RecapSearchResponse = response.json().await?;
        info!("RECAP search returned {} results", search_response.count);

        Ok(search_response)
    }

    pub async fn get_docket(&self, docket_id: u64) -> Result<RecapDocket> {
        self.rate_limiter.wait().await;

        let mut request = self.client
            .get(&format!("{}/dockets/{}/", self.base_url, docket_id))
            .query(&[("format", "json")])
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(token) = &self.api_token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("RECAP API error: {}", response.status()));
        }

        let docket: RecapDocket = response.json().await?;
        info!("Retrieved RECAP docket: {}", docket.case_name);

        Ok(docket)
    }

    pub async fn get_document(&self, document_id: u64) -> Result<RecapDocument> {
        self.rate_limiter.wait().await;

        let mut request = self.client
            .get(&format!("{}/recap-documents/{}/", self.base_url, document_id))
            .query(&[("format", "json")])
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(token) = &self.api_token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("RECAP API error: {}", response.status()));
        }

        let document: RecapDocument = response.json().await?;
        info!("Retrieved RECAP document: {:?}", document.description);

        Ok(document)
    }

    pub async fn download_document(&self, document: &RecapDocument, output_path: &str) -> Result<()> {
        if let Some(filepath_ia) = &document.filepath_ia {
            self.rate_limiter.wait().await;

            let response = self.client
                .get(filepath_ia)
                .header("User-Agent", "PA-eDocket-Desktop/1.0")
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to download document: {}", response.status()));
            }

            let bytes = response.bytes().await?;
            tokio::fs::write(output_path, bytes).await?;

            info!("Downloaded RECAP document to: {}", output_path);
            Ok(())
        } else {
            Err(anyhow!("Document has no download URL"))
        }
    }

    pub async fn search_documents(&self, query: &str, court: Option<&str>, limit: Option<u32>) -> Result<Vec<RecapDocument>> {
        self.rate_limiter.wait().await;

        let mut params = vec![
            ("q", query),
            ("format", "json"),
        ];

        if let Some(court_id) = court {
            params.push(("docket__court", court_id));
        }

        if let Some(limit_val) = limit {
            params.push(("page_size", &limit_val.to_string()));
        }

        let mut request = self.client
            .get(&format!("{}/recap-documents/", self.base_url))
            .query(&params)
            .header("User-Agent", "PA-eDocket-Desktop/1.0");

        if let Some(token) = &self.api_token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("RECAP API error: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct DocumentSearchResponse {
            results: Vec<RecapDocument>,
        }

        let search_response: DocumentSearchResponse = response.json().await?;
        info!("RECAP document search returned {} results", search_response.results.len());

        Ok(search_response.results)
    }

    pub fn convert_to_domain_docket(&self, recap_docket: &RecapDocket) -> Docket {
        Docket {
            id: recap_docket.id.to_string(),
            docket_number: recap_docket.docket_number.clone(),
            case_name: recap_docket.case_name.clone(),
            court: recap_docket.court_id.clone(),
            judge: recap_docket.assigned_to.clone(),
            date_filed: recap_docket.date_filed.map(|d| d.to_rfc3339()),
            status: if recap_docket.date_terminated.is_some() { "Terminated".to_string() } else { "Active".to_string() },
            nature_of_suit: recap_docket.nature_of_suit.clone(),
            cause_of_action: recap_docket.cause.clone(),
            parties: recap_docket.parties.iter().map(|p| Party {
                name: p.name.clone(),
                party_type: "Unknown".to_string(), // RECAP doesn't specify party type
                attorneys: vec![], // Would need separate API call
            }).collect(),
            entries: recap_docket.docket_entries.iter().map(|e| DocketEntry {
                entry_number: e.entry_number.unwrap_or(0),
                date_filed: e.date_filed.map(|d| d.to_rfc3339()).unwrap_or_default(),
                description: e.description.clone(),
                attachments: e.recap_documents.iter().map(|d| Attachment {
                    id: d.id.to_string(),
                    filename: d.description.clone().unwrap_or_default(),
                    description: d.description.clone(),
                    file_size: d.file_size,
                    page_count: d.page_count,
                    download_url: d.filepath_ia.clone(),
                }).collect(),
            }).collect(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recap_search() {
        let provider = RecapProvider::new(None);
        let result = provider.search_dockets("contract", Some("ca9"), Some(5)).await;
        
        match result {
            Ok(response) => {
                assert!(response.results.len() <= 5);
                println!("Found {} dockets", response.results.len());
            }
            Err(e) => {
                println!("Search failed: {}", e);
                // Don't fail test if API is unavailable
            }
        }
    }
}
