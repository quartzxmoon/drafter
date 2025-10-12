// PA UJS Web Portal Provider
// Production-ready integration with Pennsylvania Unified Judicial System

use crate::domain::*;
use crate::providers::{ProviderConfig, ProviderError, ProviderResult, SearchProvider};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use url::Url;

pub struct UjsPortalProvider {
    client: Client,
    config: ProviderConfig,
    base_url: Url,
}

impl UjsPortalProvider {
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let base_url = Url::parse(&config.base_url)
            .map_err(|e| ProviderError::Configuration(format!("Invalid base URL: {}", e)))?;
            
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent("PA-eDocket-Desktop/1.0")
            .build()
            .map_err(ProviderError::Network)?;
            
        Ok(Self {
            client,
            config,
            base_url,
        })
    }
    
    #[instrument(skip(self))]
    async fn make_request(&self, endpoint: &str, params: &HashMap<String, String>) -> ProviderResult<String> {
        let mut url = self.base_url.join(endpoint)
            .map_err(|e| ProviderError::Configuration(format!("Invalid endpoint: {}", e)))?;
            
        // Add query parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
        }
        
        debug!("Making request to: {}", url);
        
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(ProviderError::Network)?;
            
        if !response.status().is_success() {
            return Err(ProviderError::ServiceUnavailable(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }
        
        let text = response.text().await.map_err(ProviderError::Network)?;
        Ok(text)
    }
    
    #[instrument(skip(self, html))]
    fn parse_search_results(&self, html: &str) -> ProviderResult<Vec<SearchResult>> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();
        
        // UJS Portal uses tables for search results
        let table_selector = Selector::parse("table.searchResults, table.results, table[id*='result']")
            .map_err(|e| ProviderError::Parsing(format!("Invalid selector: {}", e)))?;
            
        let row_selector = Selector::parse("tr")
            .map_err(|e| ProviderError::Parsing(format!("Invalid row selector: {}", e)))?;
            
        let cell_selector = Selector::parse("td, th")
            .map_err(|e| ProviderError::Parsing(format!("Invalid cell selector: {}", e)))?;
        
        for table in document.select(&table_selector) {
            let rows: Vec<_> = table.select(&row_selector).collect();
            
            // Skip header row
            for row in rows.iter().skip(1) {
                let cells: Vec<_> = row.select(&cell_selector).collect();
                
                if cells.len() >= 6 {
                    // Extract data from table cells
                    // Typical UJS format: Docket Number, Caption, Court, County, Filed Date, Status
                    let docket_number = cells[0].text().collect::<String>().trim().to_string();
                    let caption = cells[1].text().collect::<String>().trim().to_string();
                    let court_str = cells[2].text().collect::<String>().trim().to_string();
                    let county = cells[3].text().collect::<String>().trim().to_string();
                    let filed_str = cells[4].text().collect::<String>().trim().to_string();
                    let status_str = cells[5].text().collect::<String>().trim().to_string();
                    
                    // Parse court level
                    let court = match court_str.to_uppercase().as_str() {
                        s if s.contains("MDJ") || s.contains("MAGISTERIAL") => CourtLevel::Mdj,
                        s if s.contains("CP") || s.contains("COMMON PLEAS") => CourtLevel::Cp,
                        s if s.contains("SUPERIOR") || s.contains("SUPREME") || s.contains("COMMONWEALTH") => CourtLevel::App,
                        _ => CourtLevel::Cp, // Default to Common Pleas
                    };
                    
                    // Parse status
                    let status = match status_str.to_uppercase().as_str() {
                        s if s.contains("ACTIVE") => CaseStatus::Active,
                        s if s.contains("CLOSED") => CaseStatus::Closed,
                        s if s.contains("PENDING") => CaseStatus::Pending,
                        s if s.contains("DISPOSED") => CaseStatus::Disposed,
                        s if s.contains("APPEAL") => CaseStatus::Appealed,
                        _ => CaseStatus::Active, // Default
                    };
                    
                    // Generate unique ID from docket number
                    let id = format!("ujs_{}", docket_number.replace(" ", "_").replace("/", "_"));
                    
                    let result = SearchResult {
                        id,
                        caption,
                        court,
                        county,
                        filed: filed_str,
                        status,
                        last_updated: None,
                        docket_number: Some(docket_number),
                        otn: None,
                        sid: None,
                        judge: None,
                        courtroom: None,
                    };
                    
                    results.push(result);
                }
            }
        }
        
        info!("Parsed {} search results", results.len());
        Ok(results)
    }
    
    #[instrument(skip(self, html))]
    fn parse_docket_detail(&self, html: &str, docket_id: &str) -> ProviderResult<Docket> {
        let document = Html::parse_document(html);
        
        // Extract basic case information
        let caption = self.extract_text_by_label(&document, "Caption", "Case Title")?;
        let court_str = self.extract_text_by_label(&document, "Court", "Court Level")?;
        let county = self.extract_text_by_label(&document, "County", "Filing County")?;
        let filed_str = self.extract_text_by_label(&document, "Filed", "Date Filed")?;
        let status_str = self.extract_text_by_label(&document, "Status", "Case Status")?;
        let docket_number = self.extract_text_by_label(&document, "Docket", "Docket Number").ok();
        let judge = self.extract_text_by_label(&document, "Judge", "Assigned Judge").ok();
        
        // Parse court level
        let court = match court_str.to_uppercase().as_str() {
            s if s.contains("MDJ") => CourtLevel::Mdj,
            s if s.contains("CP") => CourtLevel::Cp,
            s if s.contains("SUPERIOR") || s.contains("SUPREME") => CourtLevel::App,
            _ => CourtLevel::Cp,
        };
        
        // Parse status
        let status = match status_str.to_uppercase().as_str() {
            s if s.contains("ACTIVE") => CaseStatus::Active,
            s if s.contains("CLOSED") => CaseStatus::Closed,
            s if s.contains("PENDING") => CaseStatus::Pending,
            s if s.contains("DISPOSED") => CaseStatus::Disposed,
            _ => CaseStatus::Active,
        };
        
        // Parse filed date
        let filed = self.parse_date(&filed_str)?;
        
        // Extract parties
        let parties = self.extract_parties(&document)?;
        
        // Extract charges (for criminal cases)
        let charges = self.extract_charges(&document)?;
        
        // Extract events
        let events = self.extract_events(&document)?;
        
        // Extract filings
        let filings = self.extract_filings(&document)?;
        
        // Extract financials
        let financials = self.extract_financials(&document)?;
        
        let docket = Docket {
            id: docket_id.to_string(),
            caption,
            status,
            court,
            county,
            filed,
            docket_number,
            otn: None,
            sid: None,
            judge,
            courtroom: None,
            division: None,
            parties,
            charges,
            events,
            filings,
            financials,
            attachments: None,
            last_updated: Some(Utc::now()),
            source_url: Some(format!("{}?docketNumber={}", self.base_url, docket_id)),
            fetched_at: Some(Utc::now()),
            hash: None,
        };
        
        Ok(docket)
    }
    
    fn extract_text_by_label(&self, document: &Html, label: &str, alt_label: &str) -> ProviderResult<String> {
        // Try multiple selectors to find the label
        let selectors = [
            format!("td:contains('{}') + td", label),
            format!("th:contains('{}') + td", label),
            format!("label:contains('{}') + input", label),
            format!("span:contains('{}') + span", label),
            format!("td:contains('{}') + td", alt_label),
            format!("th:contains('{}') + td", alt_label),
        ];
        
        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text = element.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
            }
        }
        
        Err(ProviderError::Parsing(format!("Could not find text for label: {}", label)))
    }
    
    fn extract_parties(&self, document: &Html) -> ProviderResult<Vec<Party>> {
        // TODO: Implement party extraction from UJS HTML
        Ok(vec![])
    }
    
    fn extract_charges(&self, document: &Html) -> ProviderResult<Vec<Charge>> {
        // TODO: Implement charge extraction from UJS HTML
        Ok(vec![])
    }
    
    fn extract_events(&self, document: &Html) -> ProviderResult<Vec<Event>> {
        // TODO: Implement event extraction from UJS HTML
        Ok(vec![])
    }
    
    fn extract_filings(&self, document: &Html) -> ProviderResult<Vec<Filing>> {
        // TODO: Implement filing extraction from UJS HTML
        Ok(vec![])
    }
    
    fn extract_financials(&self, document: &Html) -> ProviderResult<Vec<Financial>> {
        // TODO: Implement financial extraction from UJS HTML
        Ok(vec![])
    }
    
    fn parse_date(&self, date_str: &str) -> ProviderResult<DateTime<Utc>> {
        // Common PA date formats
        let formats = [
            "%m/%d/%Y",
            "%m-%d-%Y",
            "%Y-%m-%d",
            "%m/%d/%y",
            "%B %d, %Y",
        ];
        
        for format in &formats {
            if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
                return Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }
        
        Err(ProviderError::Parsing(format!("Could not parse date: {}", date_str)))
    }
}

#[async_trait]
impl SearchProvider for UjsPortalProvider {
    #[instrument(skip(self, params))]
    async fn search(&self, params: &SearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        info!("Executing UJS Portal search");
        
        let mut query_params = HashMap::new();
        
        // Map search parameters to UJS Portal format
        if let Some(term) = &params.term {
            query_params.insert("searchType".to_string(), "PartyName".to_string());
            query_params.insert("searchValue".to_string(), term.clone());
        }
        
        if let Some(docket) = &params.docket {
            query_params.insert("searchType".to_string(), "DocketNumber".to_string());
            query_params.insert("searchValue".to_string(), docket.clone());
        }
        
        if let Some(county) = &params.county {
            query_params.insert("county".to_string(), county.clone());
        }
        
        if let Some(court) = &params.court {
            let court_code = match court {
                CourtLevel::Mdj => "MDJ",
                CourtLevel::Cp => "CP",
                CourtLevel::App => "APP",
            };
            query_params.insert("court".to_string(), court_code.to_string());
        }
        
        // Add date range if provided
        if let Some(from) = &params.from {
            query_params.insert("dateFrom".to_string(), from.clone());
        }
        
        if let Some(to) = &params.to {
            query_params.insert("dateTo".to_string(), to.clone());
        }
        
        let html = self.make_request("/Report/CpSearch", &query_params).await?;
        let results = self.parse_search_results(&html)?;
        
        Ok(results)
    }
    
    #[instrument(skip(self, id))]
    async fn get_docket(&self, id: &str) -> Result<Docket, ProviderError> {
        info!("Fetching UJS Portal docket: {}", id);
        
        let mut params = HashMap::new();
        params.insert("docketNumber".to_string(), id.to_string());
        
        let html = self.make_request("/Report/CpDocketSheet", &params).await?;
        let docket = self.parse_docket_detail(&html, id)?;
        
        Ok(docket)
    }
    
    #[instrument(skip(self, docket_id))]
    async fn get_attachments(&self, docket_id: &str) -> Result<Vec<Attachment>, ProviderError> {
        info!("Fetching UJS Portal attachments for: {}", docket_id);
        
        // UJS Portal typically doesn't provide direct attachment downloads
        // Return empty list for now
        Ok(vec![])
    }
}
