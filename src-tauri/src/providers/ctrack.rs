// C-Track Provider
// Integration with C-Track civil case management systems

use crate::domain::*;
use crate::providers::{client::ProviderClient, ProviderConfig, ProviderError, ProviderResult, SearchProvider};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct CTrackSearchRequest {
    case_number: Option<String>,
    party_name: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    case_type: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CTrackSearchResponse {
    cases: Vec<CTrackCase>,
    total: u32,
    page: u32,
    limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CTrackCase {
    case_id: String,
    case_number: String,
    caption: String,
    case_type: String,
    status: String,
    filed_date: String,
    court: String,
    judge: Option<String>,
    parties: Vec<CTrackParty>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CTrackParty {
    name: String,
    role: String,
    attorney: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CTrackDocument {
    document_id: String,
    filename: String,
    description: Option<String>,
    file_size: Option<u64>,
    content_type: String,
    upload_date: Option<DateTime<Utc>>,
    is_sealed: Option<bool>,
}

pub struct CTrackProvider {
    client: ProviderClient,
    config: ProviderConfig,
    county_endpoints: HashMap<String, CTrackEndpoint>,
}

#[derive(Debug, Clone)]
struct CTrackEndpoint {
    base_url: String,
    api_version: String,
    auth_type: CTrackAuthType,
    supports_civil: bool,
    supports_criminal: bool,
}

#[derive(Debug, Clone)]
enum CTrackAuthType {
    ApiKey { key: String },
    Basic { username: String, password: String },
    None,
}

impl CTrackProvider {
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let client = ProviderClient::new(config.clone())?;
        let mut county_endpoints = HashMap::new();

        // Initialize known C-Track endpoints for PA counties
        // Philadelphia County C-Track
        county_endpoints.insert("philadelphia".to_string(), CTrackEndpoint {
            base_url: "https://ctrack.courts.phila.gov".to_string(),
            api_version: "v2".to_string(),
            auth_type: CTrackAuthType::ApiKey {
                key: "".to_string() // Will be provided via configuration
            },
            supports_civil: true,
            supports_criminal: false,
        });

        // Allegheny County C-Track
        county_endpoints.insert("allegheny".to_string(), CTrackEndpoint {
            base_url: "https://ctrack.alleghenycourts.us".to_string(),
            api_version: "v1".to_string(),
            auth_type: CTrackAuthType::Basic {
                username: "".to_string(),
                password: "".to_string()
            },
            supports_civil: true,
            supports_criminal: true,
        });

        // Montgomery County C-Track
        county_endpoints.insert("montgomery".to_string(), CTrackEndpoint {
            base_url: "https://ctrack.montcopa.org".to_string(),
            api_version: "v1".to_string(),
            auth_type: CTrackAuthType::None,
            supports_civil: true,
            supports_criminal: false,
        });

        Ok(Self {
            client,
            config,
            county_endpoints,
        })
    }

    fn get_county_from_params(&self, params: &SearchParams) -> Option<&str> {
        // Extract county from court or jurisdiction
        if let Some(court) = &params.court {
            if court.to_lowercase().contains("philadelphia") {
                return Some("philadelphia");
            } else if court.to_lowercase().contains("allegheny") {
                return Some("allegheny");
            } else if court.to_lowercase().contains("montgomery") {
                return Some("montgomery");
            }
        }

        // Default to Philadelphia if no specific county found
        Some("philadelphia")
    }
    
    pub fn add_county_endpoint(&mut self, county: String, endpoint: String) {
        self.county_endpoints.insert(county, endpoint);
    }
    
    fn map_ctrack_case_to_search_result(&self, case: &CTrackCase) -> SearchResult {
        let court = if case.court.to_uppercase().contains("COMMON PLEAS") {
            CourtLevel::Cp
        } else {
            CourtLevel::Cp // Default for civil cases
        };
        
        let status = match case.status.to_uppercase().as_str() {
            s if s.contains("ACTIVE") => CaseStatus::Active,
            s if s.contains("CLOSED") => CaseStatus::Closed,
            s if s.contains("DISPOSED") => CaseStatus::Disposed,
            _ => CaseStatus::Active,
        };
        
        SearchResult {
            id: format!("ctrack_{}", case.case_id),
            caption: case.caption.clone(),
            court,
            county: "Unknown".to_string(), // TODO: Extract from config
            filed: case.filed_date.clone(),
            status,
            last_updated: None,
            docket_number: Some(case.case_number.clone()),
            otn: None,
            sid: None,
            judge: case.judge.clone(),
            courtroom: None,
        }
    }
    
    fn map_ctrack_case_to_docket(&self, case: &CTrackCase) -> Docket {
        let court = if case.court.to_uppercase().contains("COMMON PLEAS") {
            CourtLevel::Cp
        } else {
            CourtLevel::Cp
        };
        
        let status = match case.status.to_uppercase().as_str() {
            s if s.contains("ACTIVE") => CaseStatus::Active,
            s if s.contains("CLOSED") => CaseStatus::Closed,
            s if s.contains("DISPOSED") => CaseStatus::Disposed,
            _ => CaseStatus::Active,
        };
        
        // Convert C-Track parties to domain parties
        let parties: Vec<Party> = case
            .parties
            .iter()
            .map(|p| {
                let role = match p.role.to_uppercase().as_str() {
                    "PLAINTIFF" => PartyRole::Plaintiff,
                    "DEFENDANT" => PartyRole::Defendant,
                    "PETITIONER" => PartyRole::Petitioner,
                    "RESPONDENT" => PartyRole::Respondent,
                    _ => PartyRole::Plaintiff,
                };
                
                Party {
                    id: None,
                    name: p.name.clone(),
                    role,
                    address: None,
                    city: None,
                    state: None,
                    zip_code: None,
                    phone: None,
                    email: None,
                    attorney: p.attorney.clone(),
                    attorney_id: None,
                    attorney_phone: None,
                    attorney_email: None,
                    date_added: None,
                }
            })
            .collect();
        
        // Parse filed date
        let filed = chrono::NaiveDate::parse_from_str(&case.filed_date, "%Y-%m-%d")
            .or_else(|_| chrono::NaiveDate::parse_from_str(&case.filed_date, "%m/%d/%Y"))
            .unwrap_or_else(|_| chrono::Utc::now().date_naive())
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        
        Docket {
            id: format!("ctrack_{}", case.case_id),
            caption: case.caption.clone(),
            status,
            court,
            county: "Unknown".to_string(),
            filed,
            docket_number: Some(case.case_number.clone()),
            otn: None,
            sid: None,
            judge: case.judge.clone(),
            courtroom: None,
            division: None,
            parties,
            charges: vec![], // Civil cases don't have charges
            events: vec![], // TODO: Extract from C-Track if available
            filings: vec![], // TODO: Extract from C-Track if available
            financials: vec![], // TODO: Extract from C-Track if available
            attachments: None,
            last_updated: Some(chrono::Utc::now()),
            source_url: Some(format!("ctrack://{}", case.case_id)),
            fetched_at: Some(chrono::Utc::now()),
            hash: None,
        }
    }
}

#[async_trait]
impl SearchProvider for CTrackProvider {
    #[instrument(skip(self, params))]
    async fn search(&self, params: &SearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        info!("Executing C-Track search for: {:?}", params.query);

        let county = self.get_county_from_params(params)
            .ok_or_else(|| ProviderError::Configuration("Unable to determine county for C-Track search".to_string()))?;

        let endpoint = self.county_endpoints.get(county)
            .ok_or_else(|| ProviderError::Configuration(format!("No C-Track endpoint configured for county: {}", county)))?;

        // Build C-Track search request
        let search_request = CTrackSearchRequest {
            case_number: params.docket_number.clone(),
            party_name: params.participant_name.clone(),
            date_from: params.date_filed_start.as_ref().map(|d| d.format("%Y-%m-%d").to_string()),
            date_to: params.date_filed_end.as_ref().map(|d| d.format("%Y-%m-%d").to_string()),
            case_type: if params.case_type == CaseType::Civil {
                Some("civil".to_string())
            } else {
                Some("criminal".to_string())
            },
            page: Some(1),
            limit: Some(50),
        };

        // Make API call to C-Track
        let url = format!("{}/api/{}/search", endpoint.base_url, endpoint.api_version);

        match self.client.post_json::<CTrackSearchResponse>(&url, &search_request).await {
            Ok(response) => {
                debug!("C-Track search returned {} cases", response.cases.len());

                let results = response.cases.into_iter()
                    .map(|case| self.map_ctrack_case_to_search_result(&case))
                    .collect();

                Ok(results)
            },
            Err(e) => {
                warn!("C-Track search failed: {}", e);
                // Return empty results instead of failing completely
                Ok(vec![])
            }
        }
    }

    #[instrument(skip(self, id))]
    async fn get_docket(&self, id: &str) -> Result<Docket, ProviderError> {
        info!("Fetching C-Track docket: {}", id);

        // Extract county and case ID from the docket ID
        // Format: "ctrack-{county}-{case_id}"
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() < 3 || parts[0] != "ctrack" {
            return Err(ProviderError::InvalidInput(format!("Invalid C-Track docket ID format: {}", id)));
        }

        let county = parts[1];
        let case_id = parts[2..].join("-");

        let endpoint = self.county_endpoints.get(county)
            .ok_or_else(|| ProviderError::Configuration(format!("No C-Track endpoint configured for county: {}", county)))?;

        // Fetch case details from C-Track
        let url = format!("{}/api/{}/cases/{}", endpoint.base_url, endpoint.api_version, case_id);

        match self.client.get_json::<CTrackCase>(&url).await {
            Ok(case) => {
                let docket = self.map_ctrack_case_to_docket(&case);
                Ok(docket)
            },
            Err(e) => {
                error!("Failed to fetch C-Track docket {}: {}", id, e);
                Err(e)
            }
        }
    }

    #[instrument(skip(self, docket_id))]
    async fn get_attachments(&self, docket_id: &str) -> Result<Vec<Attachment>, ProviderError> {
        info!("Fetching C-Track attachments for: {}", docket_id);

        // Extract county and case ID from the docket ID
        let parts: Vec<&str> = docket_id.split('-').collect();
        if parts.len() < 3 || parts[0] != "ctrack" {
            return Ok(vec![]); // Return empty if invalid format
        }

        let county = parts[1];
        let case_id = parts[2..].join("-");

        let endpoint = self.county_endpoints.get(county)
            .ok_or_else(|| ProviderError::Configuration(format!("No C-Track endpoint configured for county: {}", county)))?;

        // Fetch attachments from C-Track
        let url = format!("{}/api/{}/cases/{}/documents", endpoint.base_url, endpoint.api_version, case_id);

        match self.client.get_json::<Vec<CTrackDocument>>(&url).await {
            Ok(documents) => {
                let attachments = documents.into_iter()
                    .map(|doc| Attachment {
                        id: doc.document_id,
                        filename: doc.filename,
                        description: doc.description.unwrap_or_else(|| doc.filename.clone()),
                        file_size: doc.file_size,
                        content_type: doc.content_type,
                        upload_date: doc.upload_date.unwrap_or_else(|| Utc::now()),
                        is_sealed: doc.is_sealed.unwrap_or(false),
                        download_url: Some(format!("{}/api/{}/documents/{}/download",
                            endpoint.base_url, endpoint.api_version, doc.document_id)),
                    })
                    .collect();

                Ok(attachments)
            },
            Err(_) => {
                // Many C-Track systems don't support document retrieval
                debug!("C-Track document retrieval not supported for county: {}", county);
                Ok(vec![])
            }
        }
    }
}
