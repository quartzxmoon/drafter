// GovInfo API Integration
// Provides access to US federal and state statutes, regulations, and legislative materials

use crate::providers::rate_limiter::RateLimiter;
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

const GOVINFO_BASE_URL: &str = "https://api.govinfo.gov";

pub struct GovInfoProvider {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
    api_key: String,
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub count: u32,
    pub offset: u32,
    #[serde(rename = "nextPage")]
    pub next_page: Option<String>,
    #[serde(rename = "previousPage")]
    pub previous_page: Option<String>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(rename = "packageId")]
    pub package_id: String,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
    #[serde(rename = "packageLink")]
    pub package_link: String,
    #[serde(rename = "docClass")]
    pub doc_class: String,
    pub title: String,
    pub congress: Option<String>,
    #[serde(rename = "dateIssued")]
    pub date_issued: Option<String>,
    #[serde(rename = "detailsLink")]
    pub details_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetails {
    #[serde(rename = "packageId")]
    pub package_id: String,
    pub title: String,
    #[serde(rename = "collectionCode")]
    pub collection_code: String,
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub category: String,
    #[serde(rename = "dateIssued")]
    pub date_issued: Option<String>,
    pub branch: Option<String>,
    pub pages: Option<u32>,
    #[serde(rename = "governmentAuthor")]
    pub government_author: Vec<String>,
    pub download: DownloadLinks,
    #[serde(rename = "relatedLink")]
    pub related_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLinks {
    #[serde(rename = "txtLink")]
    pub txt_link: Option<String>,
    #[serde(rename = "pdfLink")]
    pub pdf_link: Option<String>,
    #[serde(rename = "xmlLink")]
    pub xml_link: Option<String>,
    #[serde(rename = "modsLink")]
    pub mods_link: Option<String>,
    #[serde(rename = "premisLink")]
    pub premis_link: Option<String>,
    #[serde(rename = "zipLink")]
    pub zip_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub count: u32,
    pub offset: u32,
    pub pageSize: u32,
    #[serde(rename = "nextPage")]
    pub next_page: Option<String>,
    pub results: Vec<SearchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub title: String,
    #[serde(rename = "packageId")]
    pub package_id: String,
    #[serde(rename = "collectionCode")]
    pub collection_code: String,
    #[serde(rename = "dateIssued")]
    pub date_issued: Option<String>,
    pub download: DownloadLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USCode {
    pub title: String,
    pub section: String,
    pub text: String,
    pub citation: String,
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFR {
    pub title: u32,
    pub part: String,
    pub section: Option<String>,
    pub text: String,
    pub citation: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PennsylvaniaStatute {
    pub title: String,
    pub chapter: Option<String>,
    pub section: String,
    pub text: String,
    pub citation: String,
    pub effective_date: Option<NaiveDate>,
}

impl GovInfoProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .user_agent("PA-eDocket-Desktop/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = Arc::new(RateLimiter::new(10.0)); // 10 requests per second

        Self {
            client,
            rate_limiter,
            api_key,
        }
    }

    // ========================================================================
    // Collections
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn get_collection(&self, collection: &str, offset: u32, page_size: u32) -> Result<CollectionResponse> {
        info!("Fetching collection: {}", collection);

        self.rate_limiter.wait().await;

        let url = format!(
            "{}/collections/{}/{}?offset={}&pageSize={}&api_key={}",
            GOVINFO_BASE_URL, collection, chrono::Utc::now().format("%Y-%m-%d"), offset, page_size, self.api_key
        );

        debug!("GovInfo URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to GovInfo")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("GovInfo API error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("GovInfo API error: {}", status));
        }

        let result: CollectionResponse = response
            .json()
            .await
            .context("Failed to parse GovInfo response")?;

        info!("Found {} packages", result.count);
        Ok(result)
    }

    #[instrument(skip(self))]
    pub async fn get_package_details(&self, package_id: &str) -> Result<PackageDetails> {
        info!("Fetching package details: {}", package_id);

        self.rate_limiter.wait().await;

        let url = format!(
            "{}/packages/{}/summary?api_key={}",
            GOVINFO_BASE_URL, package_id, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch package details")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch package: {}", response.status()));
        }

        let details: PackageDetails = response
            .json()
            .await
            .context("Failed to parse package details")?;

        Ok(details)
    }

    // ========================================================================
    // Search
    // ========================================================================

    #[instrument(skip(self, query))]
    pub async fn search(&self, query: &str, collection: Option<&str>, offset: u32, page_size: u32) -> Result<SearchResult> {
        info!("Searching GovInfo: {}", query);

        self.rate_limiter.wait().await;

        let mut url = format!(
            "{}/search?query={}&offsetMark={}&pageSize={}&api_key={}",
            GOVINFO_BASE_URL,
            urlencoding::encode(query),
            urlencoding::encode(&offset.to_string()),
            page_size,
            self.api_key
        );

        if let Some(coll) = collection {
            url.push_str(&format!("&collection={}", coll));
        }

        debug!("Search URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to search GovInfo")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Search failed: {}", response.status()));
        }

        let result: SearchResult = response
            .json()
            .await
            .context("Failed to parse search results")?;

        info!("Found {} results", result.count);
        Ok(result)
    }

    // ========================================================================
    // US Code
    // ========================================================================

    pub async fn get_usc_title(&self, title: u32) -> Result<CollectionResponse> {
        info!("Fetching USC Title {}", title);
        self.get_collection(&format!("USCODE-{}", chrono::Utc::now().year()), 0, 100).await
    }

    pub async fn search_us_code(&self, query: &str) -> Result<SearchResult> {
        info!("Searching US Code: {}", query);
        self.search(query, Some("USCODE"), 0, 20).await
    }

    // ========================================================================
    // Code of Federal Regulations (CFR)
    // ========================================================================

    pub async fn get_cfr_title(&self, title: u32) -> Result<CollectionResponse> {
        info!("Fetching CFR Title {}", title);
        self.get_collection(&format!("CFR-{}", chrono::Utc::now().year()), 0, 100).await
    }

    pub async fn search_cfr(&self, query: &str) -> Result<SearchResult> {
        info!("Searching CFR: {}", query);
        self.search(query, Some("CFR"), 0, 20).await
    }

    // ========================================================================
    // Federal Register
    // ========================================================================

    pub async fn get_federal_register(&self, date: NaiveDate) -> Result<CollectionResponse> {
        info!("Fetching Federal Register for {}", date);
        let collection = format!("FR-{}", date.format("%Y-%m-%d"));
        self.get_collection(&collection, 0, 100).await
    }

    pub async fn search_federal_register(&self, query: &str) -> Result<SearchResult> {
        info!("Searching Federal Register: {}", query);
        self.search(query, Some("FR"), 0, 20).await
    }

    // ========================================================================
    // Congressional Bills and Reports
    // ========================================================================

    pub async fn get_congressional_bills(&self, congress: u32) -> Result<CollectionResponse> {
        info!("Fetching bills for Congress {}", congress);
        self.get_collection(&format!("BILLS-{}", congress), 0, 100).await
    }

    pub async fn search_congressional_bills(&self, query: &str) -> Result<SearchResult> {
        info!("Searching Congressional bills: {}", query);
        self.search(query, Some("BILLS"), 0, 20).await
    }

    pub async fn get_congressional_reports(&self, congress: u32) -> Result<CollectionResponse> {
        info!("Fetching reports for Congress {}", congress);
        self.get_collection(&format!("CRPT-{}", congress), 0, 100).await
    }

    // ========================================================================
    // Download Operations
    // ========================================================================

    #[instrument(skip(self))]
    pub async fn download_text(&self, package_id: &str) -> Result<String> {
        info!("Downloading text for package: {}", package_id);

        let details = self.get_package_details(package_id).await?;

        if let Some(txt_link) = details.download.txt_link {
            self.rate_limiter.wait().await;

            let response = self
                .client
                .get(&format!("{}?api_key={}", txt_link, self.api_key))
                .send()
                .await
                .context("Failed to download text")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Download failed: {}", response.status()));
            }

            let text = response.text().await.context("Failed to read text")?;
            Ok(text)
        } else {
            Err(anyhow::anyhow!("No text version available"))
        }
    }

    #[instrument(skip(self))]
    pub async fn download_pdf(&self, package_id: &str) -> Result<Vec<u8>> {
        info!("Downloading PDF for package: {}", package_id);

        let details = self.get_package_details(package_id).await?;

        if let Some(pdf_link) = details.download.pdf_link {
            self.rate_limiter.wait().await;

            let response = self
                .client
                .get(&format!("{}?api_key={}", pdf_link, self.api_key))
                .send()
                .await
                .context("Failed to download PDF")?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Download failed: {}", response.status()));
            }

            let bytes = response.bytes().await.context("Failed to read PDF")?;
            Ok(bytes.to_vec())
        } else {
            Err(anyhow::anyhow!("No PDF version available"))
        }
    }

    // ========================================================================
    // Pennsylvania-specific helpers (using external sources)
    // ========================================================================

    pub async fn search_pa_statutes(&self, query: &str) -> Result<Vec<PennsylvaniaStatute>> {
        // Note: GovInfo doesn't have PA state statutes
        // You would integrate with Pennsylvania's legislative website
        // or use Legal Information Institute (Cornell)
        warn!("PA statutes search requires additional integration");
        Ok(Vec::new())
    }

    // ========================================================================
    // Bulk operations
    // ========================================================================

    pub async fn bulk_download_collection(&self, collection: &str, max_items: u32) -> Result<Vec<Package>> {
        info!("Bulk downloading collection: {} (max {} items)", collection, max_items);

        let mut all_packages = Vec::new();
        let mut offset = 0;
        let page_size = 100;

        loop {
            let response = self.get_collection(collection, offset, page_size).await?;

            all_packages.extend(response.packages);

            if all_packages.len() >= max_items as usize || response.next_page.is_none() {
                break;
            }

            offset += page_size;
        }

        all_packages.truncate(max_items as usize);
        info!("Downloaded {} packages", all_packages.len());
        Ok(all_packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_search_us_code() {
        let api_key = std::env::var("GOVINFO_API_KEY").expect("API key required");
        let provider = GovInfoProvider::new(api_key);

        let result = provider
            .search_us_code("contract")
            .await
            .expect("Search failed");

        assert!(result.count > 0);
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_get_package_details() {
        let api_key = std::env::var("GOVINFO_API_KEY").expect("API key required");
        let provider = GovInfoProvider::new(api_key);

        // Test with a known package ID
        let details = provider
            .get_package_details("USCODE-2021-title18-partI-chap1-sec1")
            .await
            .expect("Failed to get package details");

        assert!(!details.title.is_empty());
    }
}
