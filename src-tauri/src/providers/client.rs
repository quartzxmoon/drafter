// HTTP client with retry logic and error handling
// Production-ready client for provider integrations

use crate::providers::{ProviderConfig, ProviderError, ProviderResult, RetryConfig};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub struct ProviderClient {
    client: Client,
    config: ProviderConfig,
}

impl ProviderClient {
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(300));
            
        // Add default headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|e| ProviderError::Configuration(format!("Invalid header name {}: {}", key, e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(value)
                .map_err(|e| ProviderError::Configuration(format!("Invalid header value {}: {}", value, e)))?;
            headers.insert(header_name, header_value);
        }
        
        builder = builder.default_headers(headers);
        
        let client = builder.build().map_err(ProviderError::Network)?;
        
        Ok(Self { client, config })
    }
    
    pub async fn get(&self, url: &str) -> ProviderResult<Response> {
        self.request_with_retry(|| self.client.get(url)).await
    }
    
    pub async fn post<T: serde::Serialize>(&self, url: &str, body: &T) -> ProviderResult<Response> {
        self.request_with_retry(|| self.client.post(url).json(body)).await
    }
    
    pub async fn put<T: serde::Serialize>(&self, url: &str, body: &T) -> ProviderResult<Response> {
        self.request_with_retry(|| self.client.put(url).json(body)).await
    }
    
    pub async fn delete(&self, url: &str) -> ProviderResult<Response> {
        self.request_with_retry(|| self.client.delete(url)).await
    }
    
    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> ProviderResult<T> {
        let response = self.get(url).await?;
        self.parse_json_response(response).await
    }
    
    pub async fn post_json<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> ProviderResult<T> {
        let response = self.post(url, body).await?;
        self.parse_json_response(response).await
    }
    
    pub async fn get_text(&self, url: &str) -> ProviderResult<String> {
        let response = self.get(url).await?;
        self.parse_text_response(response).await
    }
    
    async fn request_with_retry<F>(&self, request_fn: F) -> ProviderResult<Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let retry_config = &self.config.retry;
        let mut attempt = 0;
        let mut delay = Duration::from_millis(retry_config.initial_delay_ms);
        
        loop {
            attempt += 1;
            
            debug!("Making request attempt {} for {}", attempt, self.config.name);
            
            let request = request_fn().build().map_err(ProviderError::Network)?;
            let url = request.url().clone();
            
            match self.client.execute(request).await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Request successful for {}: {}", self.config.name, response.status());
                        return Ok(response);
                    } else if response.status().is_server_error() && attempt < retry_config.max_attempts {
                        warn!(
                            "Server error {} for {}, retrying in {:?} (attempt {}/{})",
                            response.status(),
                            self.config.name,
                            delay,
                            attempt,
                            retry_config.max_attempts
                        );
                        
                        tokio::time::sleep(delay).await;
                        delay = Duration::from_millis(
                            (delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64
                        ).min(Duration::from_millis(retry_config.max_delay_ms));
                        
                        continue;
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        return Err(match status {
                            reqwest::StatusCode::UNAUTHORIZED => {
                                ProviderError::AuthenticationFailed("Invalid credentials".to_string())
                            }
                            reqwest::StatusCode::FORBIDDEN => {
                                ProviderError::AuthenticationFailed("Access forbidden".to_string())
                            }
                            reqwest::StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimited,
                            reqwest::StatusCode::NOT_FOUND => {
                                ProviderError::InvalidResponse("Resource not found".to_string())
                            }
                            _ => ProviderError::ServiceUnavailable(format!(
                                "HTTP {}: {}",
                                status,
                                error_text.chars().take(200).collect::<String>()
                            )),
                        });
                    }
                }
                Err(e) => {
                    if attempt < retry_config.max_attempts && self.is_retryable_error(&e) {
                        warn!(
                            "Network error for {}, retrying in {:?} (attempt {}/{}): {}",
                            self.config.name, delay, attempt, retry_config.max_attempts, e
                        );
                        
                        tokio::time::sleep(delay).await;
                        delay = Duration::from_millis(
                            (delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64
                        ).min(Duration::from_millis(retry_config.max_delay_ms));
                        
                        continue;
                    } else {
                        error!("Request failed for {} after {} attempts: {}", self.config.name, attempt, e);
                        return Err(ProviderError::Network(e));
                    }
                }
            }
        }
    }
    
    async fn parse_json_response<T: DeserializeOwned>(&self, response: Response) -> ProviderResult<T> {
        let text = response.text().await.map_err(ProviderError::Network)?;
        
        serde_json::from_str(&text).map_err(|e| {
            error!("Failed to parse JSON response: {}", e);
            debug!("Response text: {}", text.chars().take(500).collect::<String>());
            ProviderError::InvalidResponse(format!("Invalid JSON: {}", e))
        })
    }
    
    async fn parse_text_response(&self, response: Response) -> ProviderResult<String> {
        response.text().await.map_err(ProviderError::Network)
    }
    
    fn is_retryable_error(&self, error: &reqwest::Error) -> bool {
        error.is_timeout() || error.is_connect() || error.is_request()
    }
}

// Utility functions for common HTTP patterns
pub async fn check_service_health(client: &ProviderClient, health_endpoint: &str) -> ProviderResult<bool> {
    match client.get(health_endpoint).await {
        Ok(response) => Ok(response.status().is_success()),
        Err(ProviderError::Network(_)) => Ok(false),
        Err(_) => Ok(false),
    }
}

pub fn build_query_string(params: &std::collections::HashMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }
    
    let pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect();
        
    format!("?{}", pairs.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::RateLimitConfig;
    use std::collections::HashMap;
    
    fn create_test_config() -> ProviderConfig {
        ProviderConfig {
            name: "test".to_string(),
            enabled: true,
            base_url: "https://httpbin.org".to_string(),
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                burst_limit: 10,
            },
            retry: RetryConfig {
                max_attempts: 3,
                backoff_multiplier: 2.0,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
            },
            headers: HashMap::new(),
            timeout_seconds: 30,
        }
    }
    
    #[tokio::test]
    async fn test_client_creation() {
        let config = create_test_config();
        let client = ProviderClient::new(config);
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_query_string_builder() {
        let mut params = HashMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value with spaces".to_string());
        
        let query = build_query_string(&params);
        assert!(query.contains("key1=value1"));
        assert!(query.contains("key2=value%20with%20spaces"));
    }
}
