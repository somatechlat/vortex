//! Centralized System Settings
//!
//! ALL configuration in one place:
//! - System variables: ENV (non-secret operational settings)
//! - Secrets: Vault ONLY (tokens, passwords, keys)
//!
//! NO SECRETS IN ENV. EVER.

use serde::{Deserialize, Serialize};
use std::env;

// ═══════════════════════════════════════════════════════════════
//                    CENTRALIZED SETTINGS
// ═══════════════════════════════════════════════════════════════

/// Centralized system settings - single source of truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Deployment environment
    pub environment: Environment,
    /// Service endpoints (non-secret)
    pub services: ServiceEndpoints,
    /// Feature configuration
    pub features: FeatureConfig,
    /// Resource limits
    pub resources: ResourceConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Environment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// SANDBOX or LIVE
    pub mode: String,
    /// Cluster namespace
    pub namespace: String,
    /// Region/zone
    pub region: String,
}

/// Service endpoints (ALL non-secret URLs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Vault address - secrets come from here, not ENV
    pub vault_addr: String,
    /// Keycloak issuer URL
    pub keycloak_issuer: String,
    /// SpiceDB gRPC endpoint
    pub spicedb_endpoint: String,
    /// PostgreSQL host (NOT connection string - password in Vault)
    pub postgres_host: String,
    /// PostgreSQL port
    pub postgres_port: u16,
    /// PostgreSQL database name
    pub postgres_db: String,
    /// Milvus gRPC endpoint
    pub milvus_endpoint: String,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub debug_panel: bool,
    pub admin_tools: bool,
    pub billing: bool,
    pub usage_limits: bool,
    pub swagger: bool,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_concurrent_jobs: usize,
    pub max_vram_mb: u64,
    pub api_rate_limit: u32,
    pub max_tenants: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub trace_sampling: f64,
}

impl Settings {
    /// Load settings from environment variables
    /// NOTE: No secrets here - secrets come from Vault at runtime
    pub fn from_env() -> Self {
        let mode = env::var("VORTEX_MODE").unwrap_or_else(|_| "sandbox".to_string());
        let is_live = mode == "live";
        
        Self {
            environment: Environment {
                mode: mode.clone(),
                namespace: env::var("VORTEX_NAMESPACE")
                    .unwrap_or_else(|_| "vortex".to_string()),
                region: env::var("VORTEX_REGION")
                    .unwrap_or_else(|_| "local".to_string()),
            },
            services: ServiceEndpoints {
                vault_addr: env::var("VAULT_ADDR")
                    .unwrap_or_else(|_| "http://vault:8200".to_string()),
                keycloak_issuer: env::var("KEYCLOAK_ISSUER")
                    .unwrap_or_else(|_| "http://keycloak:8080/realms/vortex".to_string()),
                spicedb_endpoint: env::var("SPICEDB_ENDPOINT")
                    .unwrap_or_else(|_| "spicedb:50051".to_string()),
                postgres_host: env::var("POSTGRES_HOST")
                    .unwrap_or_else(|_| "postgres".to_string()),
                postgres_port: env::var("POSTGRES_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432),
                postgres_db: env::var("POSTGRES_DB")
                    .unwrap_or_else(|_| "vortex".to_string()),
                milvus_endpoint: env::var("MILVUS_ENDPOINT")
                    .unwrap_or_else(|_| "milvus:19530".to_string()),
            },
            features: FeatureConfig {
                debug_panel: !is_live,
                admin_tools: !is_live,
                billing: is_live,
                usage_limits: is_live,
                swagger: !is_live,
            },
            resources: if is_live {
                ResourceConfig {
                    max_concurrent_jobs: 32,
                    max_vram_mb: 24576,
                    api_rate_limit: 100,
                    max_tenants: 1000,
                }
            } else {
                ResourceConfig {
                    max_concurrent_jobs: 2,
                    max_vram_mb: 4096,
                    api_rate_limit: 1000,
                    max_tenants: 10,
                }
            },
            logging: if is_live {
                LoggingConfig {
                    level: "error".to_string(),
                    format: "json".to_string(),
                    trace_sampling: 0.01,
                }
            } else {
                LoggingConfig {
                    level: "debug".to_string(),
                    format: "pretty".to_string(),
                    trace_sampling: 1.0,
                }
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    VAULT SECRET PATHS
// ═══════════════════════════════════════════════════════════════

/// Vault secret paths - ALL secrets are here, NEVER in ENV
pub struct VaultPaths;

impl VaultPaths {
    /// HuggingFace API token
    pub const HUGGINGFACE_TOKEN: &'static str = "secret/vortex/huggingface/token";
    
    /// PostgreSQL credentials
    pub const POSTGRES_USERNAME: &'static str = "secret/vortex/postgres/username";
    pub const POSTGRES_PASSWORD: &'static str = "secret/vortex/postgres/password";
    
    /// SpiceDB preshared key
    pub const SPICEDB_KEY: &'static str = "secret/vortex/spicedb/preshared_key";
    
    /// Keycloak client secret
    pub const KEYCLOAK_CLIENT_SECRET: &'static str = "secret/vortex/keycloak/client_secret";
    
    /// Milvus access key
    pub const MILVUS_ACCESS_KEY: &'static str = "secret/vortex/milvus/access_key";
    
    /// Internal encryption key
    pub const ENCRYPTION_KEY: &'static str = "secret/vortex/internal/encryption_key";
    
    /// Stripe API keys (for billing)
    pub const STRIPE_SECRET_KEY: &'static str = "secret/vortex/stripe/secret_key";
    pub const STRIPE_WEBHOOK_SECRET: &'static str = "secret/vortex/stripe/webhook_secret";
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_from_env() {
        let settings = Settings::from_env();
        assert!(!settings.environment.mode.is_empty());
        assert!(!settings.services.vault_addr.is_empty());
    }
    
    #[test]
    fn test_vault_paths() {
        assert!(VaultPaths::HUGGINGFACE_TOKEN.starts_with("secret/"));
        assert!(VaultPaths::POSTGRES_PASSWORD.starts_with("secret/"));
    }
}
