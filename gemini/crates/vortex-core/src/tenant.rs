//! Multi-Tenant Module
//!
//! Enterprise SaaS multi-tenancy with:
//! - Tenant provisioning and lifecycle
//! - SpiceDB ReBAC integration
//! - Per-tenant isolation (namespace, database)
//! - Quota and billing management

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
//                    TENANT MODEL
// ═══════════════════════════════════════════════════════════════

/// Tenant - top-level SaaS customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    /// URL-safe slug (e.g., "acme-corp")
    pub slug: String,
    pub tier: TenantTier,
    pub status: TenantStatus,
    pub quota: TenantQuota,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Subscription tier determining feature access and limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantTier {
    Free,
    Pro,
    Enterprise,
}

/// Tenant lifecycle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Provisioning,
    Active,
    Suspended,
    PendingDeletion,
    Deleted,
}

/// Resource quotas by tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_concurrent_jobs: usize,
    pub max_gpu_hours_month: u64,
    pub max_graphs: usize,
    pub max_models: usize,
    pub max_members: usize,
    pub max_storage_bytes: u64,
}

impl TenantQuota {
    pub fn for_tier(tier: TenantTier) -> Self {
        match tier {
            TenantTier::Free => Self {
                max_concurrent_jobs: 1,
                max_gpu_hours_month: 10,
                max_graphs: 5,
                max_models: 3,
                max_members: 1,
                max_storage_bytes: 1024 * 1024 * 1024,
            },
            TenantTier::Pro => Self {
                max_concurrent_jobs: 5,
                max_gpu_hours_month: 100,
                max_graphs: 100,
                max_models: 50,
                max_members: 10,
                max_storage_bytes: 50 * 1024 * 1024 * 1024,
            },
            TenantTier::Enterprise => Self {
                max_concurrent_jobs: 100,
                max_gpu_hours_month: 10000,
                max_graphs: 10000,
                max_models: 1000,
                max_members: 1000,
                max_storage_bytes: 1024 * 1024 * 1024 * 1024,
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    REPOSITORY TRAIT
// ═══════════════════════════════════════════════════════════════

/// Tenant persistence abstraction - implement with real DB
pub trait TenantRepository: Send + Sync {
    fn insert(&self, tenant: &Tenant) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>>;
    fn get_by_id(&self, id: &str) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>>;
    fn get_by_slug(&self, slug: &str) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>>;
    fn update(&self, tenant: &Tenant) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>>;
}

/// SpiceDB authorization abstraction - implement with real client
pub trait AuthorizationService: Send + Sync {
    fn set_relationship(&self, object: &str, relation: &str, subject: &str) 
        -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>>;
    fn check_permission(&self, object: &str, permission: &str, subject: &str)
        -> Pin<Box<dyn Future<Output = Result<bool, TenantError>> + Send + '_>>;
}

// ═══════════════════════════════════════════════════════════════
//                    TENANT SERVICE
// ═══════════════════════════════════════════════════════════════

/// Tenant service coordinating repository and authorization
pub struct TenantService<R: TenantRepository, A: AuthorizationService> {
    repo: R,
    auth: A,
}

impl<R: TenantRepository, A: AuthorizationService> TenantService<R, A> {
    pub fn new(repo: R, auth: A) -> Self {
        Self { repo, auth }
    }

    /// Provision new tenant with admin user
    pub async fn create_tenant(
        &self,
        name: String,
        slug: String,
        tier: TenantTier,
        admin_user_id: String,
    ) -> Result<Tenant, TenantError> {
        if !Self::is_valid_slug(&slug) {
            return Err(TenantError::InvalidSlug);
        }

        if self.repo.get_by_slug(&slug).await?.is_some() {
            return Err(TenantError::DuplicateSlug(slug));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time before epoch")
            .as_secs() as i64;

        let tenant = Tenant {
            id: Uuid::new_v4().to_string(),
            name,
            slug,
            tier,
            status: TenantStatus::Active,
            quota: TenantQuota::for_tier(tier),
            created_at: now,
            updated_at: now,
        };

        self.repo.insert(&tenant).await?;

        // SpiceDB: tenant:{id}#admin@user:{admin_user_id}
        self.auth.set_relationship(
            &format!("tenant:{}", tenant.id),
            "admin",
            &format!("user:{}", admin_user_id),
        ).await?;

        tracing::info!(
            tenant_id = %tenant.id,
            slug = %tenant.slug,
            admin = %admin_user_id,
            "Tenant created"
        );

        Ok(tenant)
    }

    /// Check tenant permission via SpiceDB
    pub async fn check_permission(
        &self,
        tenant_id: &str,
        user_id: &str,
        permission: &str,
    ) -> Result<bool, TenantError> {
        self.auth.check_permission(
            &format!("tenant:{}", tenant_id),
            permission,
            &format!("user:{}", user_id),
        ).await
    }

    /// RFC 1123 subdomain-compatible slug validation
    fn is_valid_slug(slug: &str) -> bool {
        !slug.is_empty()
            && slug.len() <= 63
            && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
    }

    pub async fn get_tenant(&self, id: &str) -> Result<Option<Tenant>, TenantError> {
        self.repo.get_by_id(id).await
    }
}

// ═══════════════════════════════════════════════════════════════
//                    ERRORS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Invalid tenant slug format")]
    InvalidSlug,
    #[error("Tenant not found: {0}")]
    NotFound(String),
    #[error("Tenant already exists with slug: {0}")]
    DuplicateSlug(String),
    #[error("Tenant is suspended")]
    Suspended,
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("Database error: {0}")]
    Database(String),
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        assert!(TenantService::<MockRepo, MockAuth>::is_valid_slug("acme-corp"));
        assert!(TenantService::<MockRepo, MockAuth>::is_valid_slug("company123"));
        assert!(TenantService::<MockRepo, MockAuth>::is_valid_slug("a"));
        assert!(!TenantService::<MockRepo, MockAuth>::is_valid_slug(""));
        assert!(!TenantService::<MockRepo, MockAuth>::is_valid_slug("-start"));
        assert!(!TenantService::<MockRepo, MockAuth>::is_valid_slug("end-"));
        assert!(!TenantService::<MockRepo, MockAuth>::is_valid_slug("UPPERCASE"));
    }

    #[test]
    fn test_quota_by_tier() {
        let free = TenantQuota::for_tier(TenantTier::Free);
        let pro = TenantQuota::for_tier(TenantTier::Pro);
        assert!(free.max_concurrent_jobs < pro.max_concurrent_jobs);
    }

    #[test]
    fn test_tenant_status_serialization() {
        let json = serde_json::to_string(&TenantStatus::Active).unwrap();
        assert_eq!(json, "\"active\"");
    }

    // Test doubles for unit testing only
    struct MockRepo;
    struct MockAuth;

    impl TenantRepository for MockRepo {
        fn insert(&self, _: &Tenant) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
        fn get_by_id(&self, _: &str) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>> {
            Box::pin(async { Ok(None) })
        }
        fn get_by_slug(&self, _: &str) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>> {
            Box::pin(async { Ok(None) })
        }
        fn update(&self, _: &Tenant) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
    }

    impl AuthorizationService for MockAuth {
        fn set_relationship(&self, _: &str, _: &str, _: &str) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
        fn check_permission(&self, _: &str, _: &str, _: &str) -> Pin<Box<dyn Future<Output = Result<bool, TenantError>> + Send + '_>> {
            Box::pin(async { Ok(true) })
        }
    }
}
