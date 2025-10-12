// Rate limiter for provider requests
// Production-ready token bucket implementation with burst support

use crate::providers::{ProviderError, RateLimitConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate,
        }
    }
    
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
    
    fn time_until_available(&mut self, tokens: f64) -> Duration {
        self.refill();
        
        if self.tokens >= tokens {
            Duration::from_secs(0)
        } else {
            let needed_tokens = tokens - self.tokens;
            let wait_time = needed_tokens / self.refill_rate;
            Duration::from_secs_f64(wait_time)
        }
    }
}

pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn check_rate_limit(&self, provider: &str, config: &RateLimitConfig) -> Result<(), ProviderError> {
        let mut buckets = self.buckets.lock().await;
        
        // Get or create bucket for this provider
        let bucket = buckets.entry(provider.to_string()).or_insert_with(|| {
            // Use the more restrictive of per-minute or per-hour limits
            let per_minute_rate = config.requests_per_minute as f64 / 60.0; // tokens per second
            let per_hour_rate = config.requests_per_hour as f64 / 3600.0;   // tokens per second
            
            let rate = per_minute_rate.min(per_hour_rate);
            let capacity = config.burst_limit as f64;
            
            debug!("Creating rate limiter for {}: {} tokens/sec, {} capacity", 
                   provider, rate, capacity);
            
            TokenBucket::new(capacity, rate)
        });
        
        // Try to consume one token
        if bucket.try_consume(1.0) {
            debug!("Rate limit check passed for {}", provider);
            Ok(())
        } else {
            let wait_time = bucket.time_until_available(1.0);
            warn!("Rate limit exceeded for {}, need to wait {:?}", provider, wait_time);
            Err(ProviderError::RateLimited)
        }
    }
    
    pub async fn wait_for_rate_limit(&self, provider: &str, config: &RateLimitConfig) -> Result<(), ProviderError> {
        loop {
            match self.check_rate_limit(provider, config).await {
                Ok(()) => return Ok(()),
                Err(ProviderError::RateLimited) => {
                    let wait_time = {
                        let mut buckets = self.buckets.lock().await;
                        if let Some(bucket) = buckets.get_mut(provider) {
                            bucket.time_until_available(1.0)
                        } else {
                            Duration::from_millis(100) // Fallback
                        }
                    };
                    
                    debug!("Waiting {:?} for rate limit on {}", wait_time, provider);
                    tokio::time::sleep(wait_time).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    pub async fn reset_bucket(&self, provider: &str) {
        let mut buckets = self.buckets.lock().await;
        buckets.remove(provider);
        debug!("Reset rate limiter bucket for {}", provider);
    }
    
    pub async fn get_bucket_status(&self, provider: &str) -> Option<(f64, f64)> {
        let mut buckets = self.buckets.lock().await;
        if let Some(bucket) = buckets.get_mut(provider) {
            bucket.refill();
            Some((bucket.tokens, bucket.capacity))
        } else {
            None
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(5.0, 1.0); // 5 tokens, 1 per second
        
        // Should be able to consume 5 tokens immediately
        for _ in 0..5 {
            assert!(bucket.try_consume(1.0));
        }
        
        // Should fail on the 6th
        assert!(!bucket.try_consume(1.0));
    }
    
    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(2.0, 2.0); // 2 tokens, 2 per second
        
        // Consume all tokens
        assert!(bucket.try_consume(2.0));
        assert!(!bucket.try_consume(1.0));
        
        // Wait for refill
        sleep(Duration::from_millis(600)).await; // Should get ~1.2 tokens
        assert!(bucket.try_consume(1.0));
        assert!(!bucket.try_consume(1.0));
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        let config = RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_limit: 5,
        };
        
        // Should allow burst requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("test", &config).await.is_ok());
        }
        
        // Should fail on the 6th
        assert!(limiter.check_rate_limit("test", &config).await.is_err());
    }
}
