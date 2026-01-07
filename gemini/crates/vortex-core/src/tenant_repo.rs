//! PostgreSQL Tenant Repository - Real Database Implementation
//!
//! Implements TenantRepository trait with actual PostgreSQL queries via SQLx.
//! Connection string from Vault or POSTGRES_DSN environment variable.

use std::future::Future;
use std::pin::Pin;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::tenant::{Tenant, TenantError, TenantQuota, TenantRepository, TenantStatus, TenantTier};

// ═══════════════════════════════════════════════════════════════
//                    REPOSITORY
// ═══════════════════════════════════════════════════════════════

/// PostgreSQL-backed tenant repository
pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    /// Create repository from existing pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connect to PostgreSQL with DSN from environment
    pub async fn connect() -> Result<Self, TenantError> {
        let dsn = std::env::var("POSTGRES_DSN")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| TenantError::Database("POSTGRES_DSN not set".into()))?;

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&dsn)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Initialize tenant schema
    pub async fn init_schema(&self) -> Result<(), TenantError> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tenants (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                tier TEXT NOT NULL CHECK (tier IN ('free', 'pro', 'enterprise')),
                status TEXT NOT NULL CHECK (status IN ('provisioning', 'active', 'suspended', 'pending_deletion', 'deleted')),
                max_concurrent_jobs INTEGER NOT NULL,
                max_gpu_hours_month BIGINT NOT NULL,
                max_graphs INTEGER NOT NULL,
                max_models INTEGER NOT NULL,
                max_members INTEGER NOT NULL,
                max_storage_bytes BIGINT NOT NULL,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::Database(e.to_string()))?;

        // Create index on slug for lookups
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug)")
            .execute(&self.pool)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

        tracing::info!("Tenant schema initialized");
        Ok(())
    }

    /// Parse tier from string
    fn parse_tier(s: &str) -> TenantTier {
        match s {
            "pro" => TenantTier::Pro,
            "enterprise" => TenantTier::Enterprise,
            _ => TenantTier::Free,
        }
    }

    /// Parse status from string
    fn parse_status(s: &str) -> TenantStatus {
        match s {
            "active" => TenantStatus::Active,
            "suspended" => TenantStatus::Suspended,
            "pending_deletion" => TenantStatus::PendingDeletion,
            "deleted" => TenantStatus::Deleted,
            _ => TenantStatus::Provisioning,
        }
    }

    /// Get tier string for DB
    fn tier_str(tier: &TenantTier) -> &'static str {
        match tier {
            TenantTier::Free => "free",
            TenantTier::Pro => "pro",
            TenantTier::Enterprise => "enterprise",
        }
    }

    /// Get status string for DB
    fn status_str(status: &TenantStatus) -> &'static str {
        match status {
            TenantStatus::Provisioning => "provisioning",
            TenantStatus::Active => "active",
            TenantStatus::Suspended => "suspended",
            TenantStatus::PendingDeletion => "pending_deletion",
            TenantStatus::Deleted => "deleted",
        }
    }
}

impl TenantRepository for PgTenantRepository {
    fn insert(
        &self,
        tenant: &Tenant,
    ) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
        let tenant = tenant.clone();
        Box::pin(async move {
            sqlx::query(r#"
                INSERT INTO tenants (
                    id, name, slug, tier, status,
                    max_concurrent_jobs, max_gpu_hours_month, max_graphs,
                    max_models, max_members, max_storage_bytes,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#)
            .bind(&tenant.id)
            .bind(&tenant.name)
            .bind(&tenant.slug)
            .bind(Self::tier_str(&tenant.tier))
            .bind(Self::status_str(&tenant.status))
            .bind(tenant.quota.max_concurrent_jobs as i32)
            .bind(tenant.quota.max_gpu_hours_month as i64)
            .bind(tenant.quota.max_graphs as i32)
            .bind(tenant.quota.max_models as i32)
            .bind(tenant.quota.max_members as i32)
            .bind(tenant.quota.max_storage_bytes as i64)
            .bind(tenant.created_at)
            .bind(tenant.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

            tracing::info!(tenant_id = %tenant.id, slug = %tenant.slug, "Tenant inserted");
            Ok(())
        })
    }

    fn get_by_id(
        &self,
        id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>> {
        let id = id.to_string();
        Box::pin(async move {
            let row = sqlx::query_as::<_, TenantRow>(
                "SELECT * FROM tenants WHERE id = $1"
            )
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

            Ok(row.map(|r| r.into_tenant()))
        })
    }

    fn get_by_slug(
        &self,
        slug: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, TenantError>> + Send + '_>> {
        let slug = slug.to_string();
        Box::pin(async move {
            let row = sqlx::query_as::<_, TenantRow>(
                "SELECT * FROM tenants WHERE slug = $1"
            )
            .bind(&slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

            Ok(row.map(|r| r.into_tenant()))
        })
    }

    fn update(
        &self,
        tenant: &Tenant,
    ) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
        let tenant = tenant.clone();
        Box::pin(async move {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time before epoch")
                .as_secs() as i64;

            sqlx::query(r#"
                UPDATE tenants SET
                    name = $1, tier = $2, status = $3,
                    max_concurrent_jobs = $4, max_gpu_hours_month = $5,
                    max_graphs = $6, max_models = $7, max_members = $8,
                    max_storage_bytes = $9, updated_at = $10
                WHERE id = $11
            "#)
            .bind(&tenant.name)
            .bind(Self::tier_str(&tenant.tier))
            .bind(Self::status_str(&tenant.status))
            .bind(tenant.quota.max_concurrent_jobs as i32)
            .bind(tenant.quota.max_gpu_hours_month as i64)
            .bind(tenant.quota.max_graphs as i32)
            .bind(tenant.quota.max_models as i32)
            .bind(tenant.quota.max_members as i32)
            .bind(tenant.quota.max_storage_bytes as i64)
            .bind(now)
            .bind(&tenant.id)
            .execute(&self.pool)
            .await
            .map_err(|e| TenantError::Database(e.to_string()))?;

            tracing::info!(tenant_id = %tenant.id, "Tenant updated");
            Ok(())
        })
    }
}

// ═══════════════════════════════════════════════════════════════
//                    SQLX ROW TYPE
// ═══════════════════════════════════════════════════════════════

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: String,
    name: String,
    slug: String,
    tier: String,
    status: String,
    max_concurrent_jobs: i32,
    max_gpu_hours_month: i64,
    max_graphs: i32,
    max_models: i32,
    max_members: i32,
    max_storage_bytes: i64,
    created_at: i64,
    updated_at: i64,
}

impl TenantRow {
    fn into_tenant(self) -> Tenant {
        Tenant {
            id: self.id,
            name: self.name,
            slug: self.slug,
            tier: PgTenantRepository::parse_tier(&self.tier),
            status: PgTenantRepository::parse_status(&self.status),
            quota: TenantQuota {
                max_concurrent_jobs: self.max_concurrent_jobs as usize,
                max_gpu_hours_month: self.max_gpu_hours_month as u64,
                max_graphs: self.max_graphs as usize,
                max_models: self.max_models as usize,
                max_members: self.max_members as usize,
                max_storage_bytes: self.max_storage_bytes as u64,
            },
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_parsing() {
        assert_eq!(PgTenantRepository::parse_tier("free"), TenantTier::Free);
        assert_eq!(PgTenantRepository::parse_tier("pro"), TenantTier::Pro);
        assert_eq!(PgTenantRepository::parse_tier("enterprise"), TenantTier::Enterprise);
        assert_eq!(PgTenantRepository::parse_tier("invalid"), TenantTier::Free);
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(PgTenantRepository::parse_status("active"), TenantStatus::Active);
        assert_eq!(PgTenantRepository::parse_status("suspended"), TenantStatus::Suspended);
        assert_eq!(PgTenantRepository::parse_status("invalid"), TenantStatus::Provisioning);
    }
}
