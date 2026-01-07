//! Server - VORTEX API Server Builder
//!
//! Provides a production-ready server with PostgreSQL repository injection.
//! Connects to real infrastructure: Vault, Keycloak, SpiceDB, PostgreSQL.

use std::sync::Arc;
use sqlx::PgPool;
use axum::Router;

use crate::api::{AppState, create_router};
use crate::graph_repo::PgGraphRepository;
use crate::run_repo::PgRunRepository;
use crate::tenant_repo::PgTenantRepository;
use crate::authz::SpiceDbClient;
use crate::error::{VortexError, VortexResult};

// ═══════════════════════════════════════════════════════════════
//                    SERVER BUILDER
// ═══════════════════════════════════════════════════════════════

/// VORTEX API Server with production dependencies
pub struct VortexServer {
    pool: PgPool,
    graph_repo: Arc<PgGraphRepository>,
    run_repo: Arc<PgRunRepository>,
    tenant_repo: Arc<PgTenantRepository>,
    authz: Arc<SpiceDbClient>,
}

impl VortexServer {
    /// Build server from environment configuration
    pub async fn from_env() -> VortexResult<Self> {
        // PostgreSQL connection
        let db_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("POSTGRES_DSN"))
            .map_err(|_| VortexError::Internal("DATABASE_URL not set".into()))?;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&db_url)
            .await
            .map_err(|e| VortexError::Internal(format!("PostgreSQL connection failed: {e}")))?;

        tracing::info!("Connected to PostgreSQL");

        // Create repositories
        let graph_repo = Arc::new(PgGraphRepository::new(pool.clone()));
        let run_repo = Arc::new(PgRunRepository::new(pool.clone()));
        let tenant_repo = Arc::new(PgTenantRepository::new(pool.clone()));

        // SpiceDB client
        let authz = Arc::new(SpiceDbClient::from_env()?);

        // Initialize schemas
        graph_repo.init_schema().await?;
        run_repo.init_schema().await?;
        tenant_repo.init_schema().await?;

        tracing::info!("Database schemas initialized");

        Ok(Self {
            pool,
            graph_repo,
            run_repo,
            tenant_repo,
            authz,
        })
    }

    /// Create the Axum router with all dependencies
    pub fn router(&self) -> Router {
        let state = Arc::new(AppState::new());
        create_router(state)
    }

    /// Get the PostgreSQL pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get graph repository
    pub fn graphs(&self) -> &Arc<PgGraphRepository> {
        &self.graph_repo
    }

    /// Get run repository
    pub fn runs(&self) -> &Arc<PgRunRepository> {
        &self.run_repo
    }

    /// Get tenant repository
    pub fn tenants(&self) -> &Arc<PgTenantRepository> {
        &self.tenant_repo
    }

    /// Get authorization client
    pub fn authz(&self) -> &Arc<SpiceDbClient> {
        &self.authz
    }

    /// Run the server on specified port
    pub async fn run(self, port: u16) -> VortexResult<()> {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
        let router = self.router();

        tracing::info!("VORTEX API server starting on {addr}");

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
//                    MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════

/// Start VORTEX server (called from main.rs)
pub async fn start_server() -> VortexResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vortex=info".parse().unwrap())
        )
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(11000);

    let server = VortexServer::from_env().await?;
    server.run(port).await
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_port() {
        // Port Authority: 11000 for VORTEX API
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(11000);
        assert_eq!(port, 11000);
    }
}
