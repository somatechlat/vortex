//! VORTEX Configuration Crate
//!
//! Enterprise-grade centralized configuration system.
//! Single source of truth for ALL settings across Rust and deployments.
//!
//! Design Patterns:
//! - Builder Pattern for runtime config construction
//! - Type-safe configuration with validation
//! - Environment-aware (SANDBOX/LIVE)
//! - Vault integration for secrets
//! - Zero secrets in environment variables

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// ═══════════════════════════════════════════════════════════════
//                    CONFIGURATION ROOT
// ═══════════════════════════════════════════════════════════════

/// Root configuration - single source of truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexConfig {
    /// Deployment mode (sandbox/live)
    pub mode: DeploymentMode,
    /// Service endpoints
    pub services: ServiceConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Worker configuration
    pub worker: WorkerConfig,
    /// API configuration
    pub api: ApiConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Multi-tenant configuration
    pub tenancy: TenancyConfig,
}

// ═══════════════════════════════════════════════════════════════
//                    DEPLOYMENT MODE
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentMode {
    #[default]
    Sandbox,
    Live,
}

impl DeploymentMode {
    pub fn is_sandbox(&self) -> bool { matches!(self, Self::Sandbox) }
    pub fn is_live(&self) -> bool { matches!(self, Self::Live) }
}

// ═══════════════════════════════════════════════════════════════
//                    SERVICE ENDPOINTS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Vault address (for secrets)
    pub vault_addr: String,
    /// Keycloak OIDC issuer
    pub keycloak_issuer: String,
    /// SpiceDB gRPC endpoint
    pub spicedb_endpoint: String,
    /// Milvus gRPC endpoint
    pub milvus_endpoint: String,
}

impl ServiceConfig {
    /// Load from environment variables - NO hardcoded defaults
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            vault_addr: std::env::var("VAULT_ADDR")
                .map_err(|_| ConfigError::MissingField("VAULT_ADDR".to_string()))?,
            keycloak_issuer: std::env::var("KEYCLOAK_ISSUER")
                .map_err(|_| ConfigError::MissingField("KEYCLOAK_ISSUER".to_string()))?,
            spicedb_endpoint: std::env::var("SPICEDB_ENDPOINT")
                .map_err(|_| ConfigError::MissingField("SPICEDB_ENDPOINT".to_string()))?,
            milvus_endpoint: std::env::var("MILVUS_ENDPOINT")
                .map_err(|_| ConfigError::MissingField("MILVUS_ENDPOINT".to_string()))?,
        })
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        // Attempt ENV lookup; fallback to explicit placeholders for visibility
        Self::from_env().unwrap_or_else(|_| {
            tracing::warn!("Service endpoints not configured via ENV, using placeholders");
            Self {
                vault_addr: "${VAULT_ADDR}".to_string(),
                keycloak_issuer: "${KEYCLOAK_ISSUER}".to_string(),
                spicedb_endpoint: "${SPICEDB_ENDPOINT}".to_string(),
                milvus_endpoint: "${MILVUS_ENDPOINT}".to_string(),
            }
        })
    }
}

// ═══════════════════════════════════════════════════════════════
//                    SECURITY CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Vault mount path for secrets
    pub vault_mount: String,
    /// Vault secret paths
    pub vault_paths: VaultPaths,
    /// JWT validation settings
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPaths {
    pub huggingface_token: String,
    pub postgres_credentials: String,
    pub spicedb_key: String,
    pub keycloak_secret: String,
    pub encryption_key: String,
    pub stripe_keys: String,
}

impl Default for VaultPaths {
    fn default() -> Self {
        Self {
            huggingface_token: "secret/vortex/huggingface/token".to_string(),
            postgres_credentials: "secret/vortex/postgres".to_string(),
            spicedb_key: "secret/vortex/spicedb/preshared_key".to_string(),
            keycloak_secret: "secret/vortex/keycloak/client_secret".to_string(),
            encryption_key: "secret/vortex/internal/encryption_key".to_string(),
            stripe_keys: "secret/vortex/stripe".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT issuer (must match Keycloak)
    pub issuer: String,
    /// JWT audience
    pub audience: String,
    /// Token expiry leeway (seconds)
    pub leeway_secs: u64,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            issuer: std::env::var("JWT_ISSUER")
                .or_else(|_| std::env::var("KEYCLOAK_ISSUER"))
                .map_err(|_| ConfigError::MissingField("JWT_ISSUER or KEYCLOAK_ISSUER".to_string()))?,
            audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "vortex-api".to_string()),
            leeway_secs: std::env::var("JWT_LEEWAY_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        })
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(|_| Self {
            issuer: "${JWT_ISSUER}".to_string(),
            audience: "vortex-api".to_string(),
            leeway_secs: 60,
        })
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            vault_mount: "secret".to_string(),
            vault_paths: VaultPaths::default(),
            jwt: JwtConfig::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    DATABASE CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL host
    pub postgres_host: String,
    /// PostgreSQL port
    pub postgres_port: u16,
    /// PostgreSQL database name
    pub postgres_db: String,
    /// Connection pool settings
    pub pool: PoolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum connections in pool
    pub min_connections: u32,
    /// Maximum connections in pool
    pub max_connections: u32,
    /// Connection timeout (seconds)
    pub connect_timeout_secs: u64,
    /// Idle timeout (seconds)
    pub idle_timeout_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            postgres_host: std::env::var("POSTGRES_HOST")
                .unwrap_or_else(|_| "${POSTGRES_HOST}".to_string()),
            postgres_port: std::env::var("POSTGRES_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5432),
            postgres_db: std::env::var("POSTGRES_DB")
                .unwrap_or_else(|_| "${POSTGRES_DB}".to_string()),
            pool: PoolConfig::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    WORKER CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Compute mode (cpu/gpu)
    pub compute_mode: ComputeMode,
    /// PyTorch precision
    pub precision: Precision,
    /// Model cache directory
    pub model_cache_dir: PathBuf,
    /// Maximum VRAM per job (MB)
    pub max_vram_mb: u64,
    /// Worker pool size
    pub pool_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ComputeMode {
    #[default]
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    #[default]
    Float32,
    Float16,
    BFloat16,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            compute_mode: ComputeMode::Cpu,
            precision: Precision::Float32,
            model_cache_dir: PathBuf::from("/var/cache/vortex/models"),
            max_vram_mb: 4096,
            pool_size: 2,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    API CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API host
    pub host: String,
    /// API port
    pub port: u16,
    /// WebSocket port
    pub ws_port: u16,
    /// Metrics port
    pub metrics_port: u16,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Rate limiting
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 1000,
            burst_size: 100,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 11188,
            ws_port: 11189,
            metrics_port: 11191,
            cors_origins: vec!["*".to_string()],
            rate_limit: RateLimitConfig::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    FEATURE FLAGS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub debug_panel: bool,
    pub admin_tools: bool,
    pub billing: bool,
    pub usage_limits: bool,
    pub swagger: bool,
    pub telemetry: bool,
}

impl FeatureFlags {
    pub fn sandbox() -> Self {
        Self {
            debug_panel: true,
            admin_tools: true,
            billing: false,
            usage_limits: false,
            swagger: true,
            telemetry: true,
        }
    }
    
    pub fn live() -> Self {
        Self {
            debug_panel: false,
            admin_tools: false,
            billing: true,
            usage_limits: true,
            swagger: false,
            telemetry: true,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::sandbox()
    }
}

// ═══════════════════════════════════════════════════════════════
//                    RESOURCE LIMITS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Max concurrent jobs system-wide
    pub max_concurrent_jobs: usize,
    /// Max graphs per tenant
    pub max_graphs_per_tenant: usize,
    /// Max models per tenant
    pub max_models_per_tenant: usize,
    /// Max storage per tenant (bytes)
    pub max_storage_per_tenant: u64,
}

impl ResourceLimits {
    pub fn sandbox() -> Self {
        Self {
            max_concurrent_jobs: 4,
            max_graphs_per_tenant: 50,
            max_models_per_tenant: 10,
            max_storage_per_tenant: 10 * 1024 * 1024 * 1024, // 10 GB
        }
    }
    
    pub fn live() -> Self {
        Self {
            max_concurrent_jobs: 100,
            max_graphs_per_tenant: 10000,
            max_models_per_tenant: 1000,
            max_storage_per_tenant: 1024 * 1024 * 1024 * 1024, // 1 TB
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::sandbox()
    }
}

// ═══════════════════════════════════════════════════════════════
//                    LOGGING CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    /// Log format
    pub format: LogFormat,
    /// OpenTelemetry trace sampling rate (0.0 - 1.0)
    pub trace_sampling: f64,
    /// Log output
    pub output: LogOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogOutput {
    #[default]
    Stdout,
    Stderr,
    File,
}

impl LoggingConfig {
    pub fn sandbox() -> Self {
        Self {
            level: LogLevel::Debug,
            format: LogFormat::Pretty,
            trace_sampling: 1.0,
            output: LogOutput::Stdout,
        }
    }
    
    pub fn live() -> Self {
        Self {
            level: LogLevel::Error,
            format: LogFormat::Json,
            trace_sampling: 0.01,
            output: LogOutput::Stdout,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self::sandbox()
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TENANCY CONFIG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenancyConfig {
    /// Tenant isolation mode
    pub isolation: TenantIsolation,
    /// Maximum tenants
    pub max_tenants: usize,
    /// Default tenant tier
    pub default_tier: TenantTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TenantIsolation {
    #[default]
    Namespace,
    Database,
    Schema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TenantTier {
    #[default]
    Free,
    Pro,
    Enterprise,
}

impl Default for TenancyConfig {
    fn default() -> Self {
        Self {
            isolation: TenantIsolation::Namespace,
            max_tenants: 10,
            default_tier: TenantTier::Free,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    CONFIG BUILDER
// ═══════════════════════════════════════════════════════════════

/// Builder for VortexConfig with validation
pub struct ConfigBuilder {
    mode: DeploymentMode,
    services: Option<ServiceConfig>,
    security: Option<SecurityConfig>,
    database: Option<DatabaseConfig>,
    worker: Option<WorkerConfig>,
    api: Option<ApiConfig>,
    features: Option<FeatureFlags>,
    resources: Option<ResourceLimits>,
    logging: Option<LoggingConfig>,
    tenancy: Option<TenancyConfig>,
}

impl ConfigBuilder {
    pub fn new(mode: DeploymentMode) -> Self {
        Self {
            mode,
            services: None,
            security: None,
            database: None,
            worker: None,
            api: None,
            features: None,
            resources: None,
            logging: None,
            tenancy: None,
        }
    }
    
    pub fn sandbox() -> Self {
        Self::new(DeploymentMode::Sandbox)
    }
    
    pub fn live() -> Self {
        Self::new(DeploymentMode::Live)
    }
    
    pub fn services(mut self, config: ServiceConfig) -> Self {
        self.services = Some(config);
        self
    }
    
    pub fn security(mut self, config: SecurityConfig) -> Self {
        self.security = Some(config);
        self
    }
    
    pub fn database(mut self, config: DatabaseConfig) -> Self {
        self.database = Some(config);
        self
    }
    
    pub fn worker(mut self, config: WorkerConfig) -> Self {
        self.worker = Some(config);
        self
    }
    
    pub fn api(mut self, config: ApiConfig) -> Self {
        self.api = Some(config);
        self
    }
    
    pub fn features(mut self, config: FeatureFlags) -> Self {
        self.features = Some(config);
        self
    }
    
    pub fn resources(mut self, config: ResourceLimits) -> Self {
        self.resources = Some(config);
        self
    }
    
    pub fn logging(mut self, config: LoggingConfig) -> Self {
        self.logging = Some(config);
        self
    }
    
    pub fn tenancy(mut self, config: TenancyConfig) -> Self {
        self.tenancy = Some(config);
        self
    }
    
    /// Build with validation
    pub fn build(self) -> Result<VortexConfig, ConfigError> {
        let (features, resources, logging) = match self.mode {
            DeploymentMode::Sandbox => (
                self.features.unwrap_or_else(FeatureFlags::sandbox),
                self.resources.unwrap_or_else(ResourceLimits::sandbox),
                self.logging.unwrap_or_else(LoggingConfig::sandbox),
            ),
            DeploymentMode::Live => (
                self.features.unwrap_or_else(FeatureFlags::live),
                self.resources.unwrap_or_else(ResourceLimits::live),
                self.logging.unwrap_or_else(LoggingConfig::live),
            ),
        };
        
        let config = VortexConfig {
            mode: self.mode,
            services: self.services.unwrap_or_default(),
            security: self.security.unwrap_or_default(),
            database: self.database.unwrap_or_default(),
            worker: self.worker.unwrap_or_default(),
            api: self.api.unwrap_or_default(),
            features,
            resources,
            logging,
            tenancy: self.tenancy.unwrap_or_default(),
        };
        
        // Validate
        config.validate()?;
        
        Ok(config)
    }
}

// ═══════════════════════════════════════════════════════════════
//                    CONFIG IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════

impl VortexConfig {
    /// Load from environment with defaults
    pub fn from_env() -> Result<Self, ConfigError> {
        let mode = std::env::var("VORTEX_MODE")
            .map(|m| match m.to_lowercase().as_str() {
                "live" | "production" | "prod" => DeploymentMode::Live,
                _ => DeploymentMode::Sandbox,
            })
            .unwrap_or(DeploymentMode::Sandbox);
        
        ConfigBuilder::new(mode).build()
    }
    
    /// Load from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        let config: VortexConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        const PORT_RANGE_START: u16 = 11000;
        const PORT_RANGE_END: u16 = 11999;

        // Validate API ports
        if self.api.port == self.api.ws_port {
            return Err(ConfigError::Validation(
                "API port and WebSocket port cannot be the same".to_string()
            ));
        }

        // Enforce port authority range
        for (name, port) in [
            ("api.port", self.api.port),
            ("api.ws_port", self.api.ws_port),
            ("api.metrics_port", self.api.metrics_port),
        ] {
            if port < PORT_RANGE_START || port > PORT_RANGE_END {
                return Err(ConfigError::Validation(format!(
                    "{name} must be within {PORT_RANGE_START}-{PORT_RANGE_END}"
                )));
            }
        }
        
        // Validate pool config
        if self.database.pool.min_connections > self.database.pool.max_connections {
            return Err(ConfigError::Validation(
                "min_connections cannot exceed max_connections".to_string()
            ));
        }
        
        // Validate trace sampling
        if !(0.0..=1.0).contains(&self.logging.trace_sampling) {
            return Err(ConfigError::Validation(
                "trace_sampling must be between 0.0 and 1.0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get Vault address
    pub fn vault_addr(&self) -> &str {
        &self.services.vault_addr
    }
    
    /// Get database connection string template (without password)
    pub fn postgres_connection_template(&self) -> String {
        format!(
            "postgres://{{username}}:{{password}}@{}:{}/{}",
            self.database.postgres_host,
            self.database.postgres_port,
            self.database.postgres_db
        )
    }
}

// ═══════════════════════════════════════════════════════════════
//                    ERRORS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config() {
        let config = ConfigBuilder::sandbox().build().unwrap();
        assert!(config.mode.is_sandbox());
        assert!(config.features.debug_panel);
        assert!(!config.features.billing);
        assert_eq!(config.logging.level, LogLevel::Debug);
    }
    
    #[test]
    fn test_live_config() {
        let config = ConfigBuilder::live().build().unwrap();
        assert!(config.mode.is_live());
        assert!(!config.features.debug_panel);
        assert!(config.features.billing);
        assert_eq!(config.logging.level, LogLevel::Error);
    }
    
    #[test]
    fn test_validation_fails_for_same_ports() {
        let config = ConfigBuilder::sandbox()
            .api(ApiConfig {
                port: 11188,
                ws_port: 11188,
                ..Default::default()
            })
            .build();
        
        assert!(config.is_err());
    }
    
    #[test]
    fn test_vault_paths() {
        let paths = VaultPaths::default();
        assert!(paths.huggingface_token.starts_with("secret/"));
        assert!(paths.postgres_credentials.contains("postgres"));
    }
    
    #[test]
    fn test_config_serialization() {
        let config = ConfigBuilder::sandbox().build().unwrap();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("sandbox"));
    }
    
    #[test]
    fn test_from_env_defaults_to_sandbox() {
        let config = VortexConfig::from_env().unwrap();
        // Without env var, defaults to sandbox
        assert!(config.mode.is_sandbox());
    }
}
