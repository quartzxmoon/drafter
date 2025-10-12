// Configuration management for PA eDocket Desktop

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};
use validator::{Validate, ValidationError};

pub mod security;

pub use security::SecurityConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    #[validate]
    pub courts: CourtsConfig,
    #[validate]
    pub providers: ProvidersConfig,
    #[validate]
    pub global: GlobalConfig,
    #[validate]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CourtsConfig {
    #[validate]
    pub courts: HashMap<String, CourtConfig>,
    #[validate]
    pub counties: HashMap<String, CountyConfig>,
    #[validate]
    pub templates: HashMap<String, TemplateConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourtConfig {
    pub name: String,
    pub level: String,
    pub jurisdiction: String,
    pub formatting: FormattingConfig,
    pub efiling: Option<EFilingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingConfig {
    pub margins: MarginsConfig,
    pub font: FontConfig,
    pub caption: CaptionConfig,
    pub signature: SignatureConfig,
    pub service_certificate: bool,
    pub page_limits: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginsConfig {
    pub top: String,
    pub bottom: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: String,
    pub line_spacing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionConfig {
    pub format: String,
    pub include_docket: bool,
    pub include_court: bool,
    pub include_county: bool,
    pub include_judge: bool,
    pub include_division: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureConfig {
    pub attorney_name: bool,
    pub attorney_id: bool,
    pub firm_name: bool,
    pub address: bool,
    pub phone: bool,
    pub email: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EFilingConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyConfig {
    pub name: String,
    pub cp_court_id: String,
    pub efiling: Option<EFilingConfig>,
    pub local_rules: LocalRulesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRulesConfig {
    pub cover_sheet_required: bool,
    pub electronic_service: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub category: String,
    pub courts: Vec<String>,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub providers: HashMap<String, ProviderConfig>,
    pub global: GlobalProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub base_url: String,
    pub rate_limit: RateLimitConfig,
    pub retry: RetryConfig,
    pub endpoints: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub auth: Option<AuthConfig>,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_multiplier: f64,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: String,
    pub token_endpoint: Option<String>,
    pub refresh_endpoint: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub max_entries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProviderConfig {
    pub timeout_seconds: u64,
    pub connection_pool: ConnectionPoolConfig,
    pub tls: TlsConfig,
    pub logging: LoggingConfig,
    pub error_handling: ErrorHandlingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    pub max_connections: u32,
    pub max_idle_connections: u32,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub min_tls_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub structured: bool,
    pub redact_pii: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    pub max_retries: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub app_name: String,
    pub version: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub log_dir: String,
    pub max_log_files: u32,
    pub max_log_size_mb: u64,
}

pub struct ConfigManager {
    config_dir: PathBuf,
    cache: Option<AppConfig>,
}

impl ConfigManager {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            cache: None,
        }
    }

    pub async fn load_config(&mut self) -> Result<&AppConfig> {
        if self.cache.is_none() {
            info!("Loading configuration from: {:?}", self.config_dir);

            // Load individual config files
            let courts_config = self.load_courts_config().await?;
            let providers_config = self.load_providers_config().await?;
            let global_config = self.load_global_config().await?;
            let security_config = self.load_security_config().await?;

            let config = AppConfig {
                courts: courts_config,
                providers: providers_config,
                global: global_config,
                security: security_config,
            };

            // Validate the complete configuration
            config.validate()
                .context("Configuration validation failed")?;

            self.cache = Some(config);
            info!("Configuration loaded and validated successfully");
        }

        Ok(self.cache.as_ref().unwrap())
    }

    pub async fn reload_config(&mut self) -> Result<&AppConfig> {
        info!("Reloading configuration");
        self.cache = None;
        self.load_config().await
    }

    pub async fn save_config(&self, config: &AppConfig) -> Result<()> {
        info!("Saving configuration to: {:?}", self.config_dir);

        // Validate before saving
        config.validate()
            .context("Cannot save invalid configuration")?;

        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir)
            .context("Failed to create config directory")?;

        // Save individual config files
        self.save_courts_config(&config.courts).await?;
        self.save_providers_config(&config.providers).await?;
        self.save_global_config(&config.global).await?;
        self.save_security_config(&config.security).await?;

        info!("Configuration saved successfully");
        Ok(())
    }

    async fn load_courts_config(&self) -> Result<CourtsConfig> {
        let courts_path = self.config_dir.join("courts.yaml");

        if courts_path.exists() {
            debug!("Loading courts config from: {:?}", courts_path);
            let content = fs::read_to_string(&courts_path)
                .context("Failed to read courts.yaml")?;
            let config: CourtsConfig = serde_yaml::from_str(&content)
                .context("Failed to parse courts.yaml")?;
            config.validate()
                .context("Courts configuration validation failed")?;
            Ok(config)
        } else {
            warn!("Courts config file not found, using defaults");
            Ok(CourtsConfig::default())
        }
    }

    async fn load_providers_config(&self) -> Result<ProvidersConfig> {
        let providers_path = self.config_dir.join("providers.yaml");

        if providers_path.exists() {
            debug!("Loading providers config from: {:?}", providers_path);
            let content = fs::read_to_string(&providers_path)
                .context("Failed to read providers.yaml")?;
            let config: ProvidersConfig = serde_yaml::from_str(&content)
                .context("Failed to parse providers.yaml")?;
            config.validate()
                .context("Providers configuration validation failed")?;
            Ok(config)
        } else {
            warn!("Providers config file not found, using defaults");
            Ok(ProvidersConfig::default())
        }
    }

    async fn load_global_config(&self) -> Result<GlobalConfig> {
        let global_path = self.config_dir.join("global.yaml");

        if global_path.exists() {
            debug!("Loading global config from: {:?}", global_path);
            let content = fs::read_to_string(&global_path)
                .context("Failed to read global.yaml")?;
            let config: GlobalConfig = serde_yaml::from_str(&content)
                .context("Failed to parse global.yaml")?;
            config.validate()
                .context("Global configuration validation failed")?;
            Ok(config)
        } else {
            warn!("Global config file not found, using defaults");
            Ok(GlobalConfig::default())
        }
    }

    async fn load_security_config(&self) -> Result<SecurityConfig> {
        let security_path = self.config_dir.join("security.yaml");

        if security_path.exists() {
            debug!("Loading security config from: {:?}", security_path);
            let content = fs::read_to_string(&security_path)
                .context("Failed to read security.yaml")?;
            let config: SecurityConfig = serde_yaml::from_str(&content)
                .context("Failed to parse security.yaml")?;
            Ok(config)
        } else {
            warn!("Security config file not found, using defaults");
            Ok(SecurityConfig::default())
        }
    }

    async fn save_courts_config(&self, config: &CourtsConfig) -> Result<()> {
        let courts_path = self.config_dir.join("courts.yaml");
        let content = serde_yaml::to_string(config)
            .context("Failed to serialize courts config")?;
        fs::write(courts_path, content)
            .context("Failed to write courts.yaml")?;
        Ok(())
    }

    async fn save_providers_config(&self, config: &ProvidersConfig) -> Result<()> {
        let providers_path = self.config_dir.join("providers.yaml");
        let content = serde_yaml::to_string(config)
            .context("Failed to serialize providers config")?;
        fs::write(providers_path, content)
            .context("Failed to write providers.yaml")?;
        Ok(())
    }

    async fn save_global_config(&self, config: &GlobalConfig) -> Result<()> {
        let global_path = self.config_dir.join("global.yaml");
        let content = serde_yaml::to_string(config)
            .context("Failed to serialize global config")?;
        fs::write(global_path, content)
            .context("Failed to write global.yaml")?;
        Ok(())
    }

    async fn save_security_config(&self, config: &SecurityConfig) -> Result<()> {
        let security_path = self.config_dir.join("security.yaml");
        let content = serde_yaml::to_string(config)
            .context("Failed to serialize security config")?;
        fs::write(security_path, content)
            .context("Failed to write security.yaml")?;
        Ok(())
    }
}

// Convenience function for backward compatibility
pub async fn load_config() -> Result<AppConfig> {
    let config_dir = PathBuf::from("config");
    let mut manager = ConfigManager::new(config_dir);
    manager.load_config().await.map(|c| c.clone())
}

pub async fn save_config(config: &AppConfig, path: &Path) -> Result<()> {
    let mut manager = ConfigManager::new(path.to_path_buf());
    manager.save_config(config).await
}

// Default implementations
impl Default for CourtsConfig {
    fn default() -> Self {
        Self {
            courts: HashMap::new(),
            counties: HashMap::new(),
            templates: HashMap::new(),
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            providers: HashMap::new(),
            global: GlobalProviderConfig::default(),
        }
    }
}

impl Default for GlobalProviderConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            connection_pool: ConnectionPoolConfig::default(),
            tls: TlsConfig::default(),
            logging: LoggingConfig::default(),
            error_handling: ErrorHandlingConfig::default(),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            max_idle_connections: 10,
            idle_timeout_seconds: 300,
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            min_tls_version: "1.2".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            structured: true,
            redact_pii: true,
        }
    }
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            circuit_breaker_threshold: 10,
            circuit_breaker_timeout_seconds: 60,
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            app_name: "PA eDocket Desktop".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: "~/.pa-edocket".to_string(),
            cache_dir: "~/.pa-edocket/cache".to_string(),
            log_dir: "~/.pa-edocket/logs".to_string(),
            max_log_files: 10,
            max_log_size_mb: 100,
        }
    }
}

// Validation implementations
impl Validate for CourtsConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        // Validate all court configs
        for (id, court) in &self.courts {
            court.validate().map_err(|mut errors| {
                errors.add_field_error(id, ValidationError::new("court_validation_failed"));
                errors
            })?;
        }

        // Validate all county configs
        for (id, county) in &self.counties {
            county.validate().map_err(|mut errors| {
                errors.add_field_error(id, ValidationError::new("county_validation_failed"));
                errors
            })?;
        }

        Ok(())
    }
}

impl Validate for ProvidersConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        // Validate all provider configs
        for (id, provider) in &self.providers {
            provider.validate().map_err(|mut errors| {
                errors.add_field_error(id, ValidationError::new("provider_validation_failed"));
                errors
            })?;
        }

        self.global.validate()?;
        Ok(())
    }
}

impl Validate for GlobalConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.app_name.is_empty() {
            errors.add_field_error("app_name", ValidationError::new("required"));
        }

        if self.version.is_empty() {
            errors.add_field_error("version", ValidationError::new("required"));
        }

        if self.max_log_files == 0 {
            errors.add_field_error("max_log_files", ValidationError::new("min_value"));
        }

        if self.max_log_size_mb == 0 {
            errors.add_field_error("max_log_size_mb", ValidationError::new("min_value"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Add validation for other structs as needed
impl Validate for CourtConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.name.is_empty() {
            errors.add_field_error("name", ValidationError::new("required"));
        }

        if self.level.is_empty() {
            errors.add_field_error("level", ValidationError::new("required"));
        }

        if self.jurisdiction.is_empty() {
            errors.add_field_error("jurisdiction", ValidationError::new("required"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for CountyConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.name.is_empty() {
            errors.add_field_error("name", ValidationError::new("required"));
        }

        if self.cp_court_id.is_empty() {
            errors.add_field_error("cp_court_id", ValidationError::new("required"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for ProviderConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.name.is_empty() {
            errors.add_field_error("name", ValidationError::new("required"));
        }

        if self.base_url.is_empty() {
            errors.add_field_error("base_url", ValidationError::new("required"));
        }

        // Validate URL format
        if url::Url::parse(&self.base_url).is_err() {
            errors.add_field_error("base_url", ValidationError::new("invalid_url"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validate for GlobalProviderConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.timeout_seconds == 0 {
            errors.add_field_error("timeout_seconds", ValidationError::new("min_value"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
