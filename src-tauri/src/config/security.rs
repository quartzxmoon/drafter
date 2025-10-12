// Security configuration for PA eDocket Desktop

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub csp: ContentSecurityPolicy,
    pub https: HttpsConfig,
    pub authentication: AuthConfig,
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityPolicy {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub font_src: Vec<String>,
    pub object_src: Vec<String>,
    pub media_src: Vec<String>,
    pub frame_src: Vec<String>,
    pub worker_src: Vec<String>,
    pub manifest_src: Vec<String>,
    pub form_action: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub upgrade_insecure_requests: bool,
    pub block_all_mixed_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsConfig {
    pub enforce_https: bool,
    pub require_valid_certificates: bool,
    pub allowed_insecure_hosts: Vec<String>,
    pub hsts_max_age: u32,
    pub hsts_include_subdomains: bool,
    pub hsts_preload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub session_timeout_minutes: u32,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub require_mfa: bool,
    pub password_policy: PasswordPolicy,
    pub allowed_auth_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub history_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub key_size: u32,
    pub use_hardware_security: bool,
    pub encrypt_local_storage: bool,
    pub encrypt_network_traffic: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            csp: ContentSecurityPolicy::default(),
            https: HttpsConfig::default(),
            authentication: AuthConfig::default(),
            encryption: EncryptionConfig::default(),
        }
    }
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec![
                "'self'".to_string(),
                "'unsafe-inline'".to_string(), // Required for Tauri
                "tauri:".to_string(),
            ],
            style_src: vec![
                "'self'".to_string(),
                "'unsafe-inline'".to_string(), // Required for CSS-in-JS
                "https://fonts.googleapis.com".to_string(),
            ],
            img_src: vec![
                "'self'".to_string(),
                "data:".to_string(),
                "https:".to_string(),
            ],
            connect_src: vec![
                "'self'".to_string(),
                "https://ujsportal.pacourts.us".to_string(),
                "https://www.pacourts.us".to_string(),
                "https://api.courtlistener.com".to_string(),
                "https://api.govinfo.gov".to_string(),
                "wss://localhost:*".to_string(), // Tauri dev server
                "ws://localhost:*".to_string(),  // Tauri dev server
            ],
            font_src: vec![
                "'self'".to_string(),
                "https://fonts.gstatic.com".to_string(),
            ],
            object_src: vec!["'none'".to_string()],
            media_src: vec!["'self'".to_string()],
            frame_src: vec!["'none'".to_string()],
            worker_src: vec!["'self'".to_string()],
            manifest_src: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            upgrade_insecure_requests: true,
            block_all_mixed_content: true,
        }
    }
}

impl Default for HttpsConfig {
    fn default() -> Self {
        Self {
            enforce_https: true,
            require_valid_certificates: true,
            allowed_insecure_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: false,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 30,
            max_failed_attempts: 3,
            lockout_duration_minutes: 15,
            require_mfa: false,
            password_policy: PasswordPolicy::default(),
            allowed_auth_methods: vec![
                "password".to_string(),
                "oauth2".to_string(),
                "session".to_string(),
            ],
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: Some(90),
            history_count: 5,
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            use_hardware_security: true,
            encrypt_local_storage: true,
            encrypt_network_traffic: true,
        }
    }
}

impl ContentSecurityPolicy {
    pub fn to_header_value(&self) -> String {
        let mut directives = Vec::new();

        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }
        if !self.script_src.is_empty() {
            directives.push(format!("script-src {}", self.script_src.join(" ")));
        }
        if !self.style_src.is_empty() {
            directives.push(format!("style-src {}", self.style_src.join(" ")));
        }
        if !self.img_src.is_empty() {
            directives.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            directives.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.font_src.is_empty() {
            directives.push(format!("font-src {}", self.font_src.join(" ")));
        }
        if !self.object_src.is_empty() {
            directives.push(format!("object-src {}", self.object_src.join(" ")));
        }
        if !self.media_src.is_empty() {
            directives.push(format!("media-src {}", self.media_src.join(" ")));
        }
        if !self.frame_src.is_empty() {
            directives.push(format!("frame-src {}", self.frame_src.join(" ")));
        }
        if !self.worker_src.is_empty() {
            directives.push(format!("worker-src {}", self.worker_src.join(" ")));
        }
        if !self.manifest_src.is_empty() {
            directives.push(format!("manifest-src {}", self.manifest_src.join(" ")));
        }
        if !self.form_action.is_empty() {
            directives.push(format!("form-action {}", self.form_action.join(" ")));
        }
        if !self.frame_ancestors.is_empty() {
            directives.push(format!("frame-ancestors {}", self.frame_ancestors.join(" ")));
        }
        if !self.base_uri.is_empty() {
            directives.push(format!("base-uri {}", self.base_uri.join(" ")));
        }

        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }
        if self.block_all_mixed_content {
            directives.push("block-all-mixed-content".to_string());
        }

        directives.join("; ")
    }

    pub fn validate_url(&self, url: &str, directive: &str) -> bool {
        let sources = match directive {
            "script-src" => &self.script_src,
            "style-src" => &self.style_src,
            "img-src" => &self.img_src,
            "connect-src" => &self.connect_src,
            "font-src" => &self.font_src,
            "object-src" => &self.object_src,
            "media-src" => &self.media_src,
            "frame-src" => &self.frame_src,
            "worker-src" => &self.worker_src,
            "manifest-src" => &self.manifest_src,
            _ => &self.default_src,
        };

        // Check if URL matches any allowed source
        for source in sources {
            if source == "'self'" {
                // Check if URL is same-origin
                if url.starts_with("/") || url.starts_with("./") || url.starts_with("../") {
                    return true;
                }
            } else if source == "'unsafe-inline'" {
                // Allow inline content (not applicable for URLs)
                continue;
            } else if source.starts_with("https://") || source.starts_with("http://") {
                if url.starts_with(source) {
                    return true;
                }
            } else if source.ends_with(":") {
                // Protocol check (e.g., "https:", "data:")
                if url.starts_with(source) {
                    return true;
                }
            }
        }

        false
    }
}

impl HttpsConfig {
    pub fn validate_url(&self, url: &str) -> Result<(), String> {
        if !self.enforce_https {
            return Ok(());
        }

        if url.starts_with("https://") {
            return Ok(());
        }

        if url.starts_with("http://") {
            // Check if host is in allowed insecure hosts
            if let Ok(parsed_url) = url::Url::parse(url) {
                if let Some(host) = parsed_url.host_str() {
                    if self.allowed_insecure_hosts.contains(&host.to_string()) {
                        return Ok(());
                    }
                }
            }
            return Err("HTTPS is required for this connection".to_string());
        }

        // Allow relative URLs and other protocols
        Ok(())
    }

    pub fn get_hsts_header(&self) -> Option<String> {
        if !self.enforce_https {
            return None;
        }

        let mut header = format!("max-age={}", self.hsts_max_age);
        
        if self.hsts_include_subdomains {
            header.push_str("; includeSubDomains");
        }
        
        if self.hsts_preload {
            header.push_str("; preload");
        }

        Some(header)
    }
}
