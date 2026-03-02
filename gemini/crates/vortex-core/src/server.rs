//! Server - VORTEX API Server Builder
//!
//! Provides a production-ready server with SeaORM repository injection.
//! Connects to real infrastructure: Vault, Keycloak, SpiceDB, PostgreSQL.

use std::sync::Arc;
use axum::Router;

use crate::api::{AppState, create_router};
use crate::graph_repo::GraphRepository;
use crate::run_repo::RunRepository;
use crate::tenant_repo::TenantRepository;
use crate::authz::SpiceDbClient;
use crate::db::Database;
use crate::error::{VortexError, VortexResult};

// ═══════════════════════════════════════════════════════════════
//                    SERVER BUILDER
// ═══════════════════════════════════════════════════════════════

/// VORTEX API Server with production dependencies
pub struct VortexServer {
    db: Arc<Database>,
    graph_repo: Arc<GraphRepository>,
    run_repo: Arc<RunRepository>,
    tenant_repo: Arc<TenantRepository>,
    authz: Arc<SpiceDbClient>,
    mcp: Arc<crate::mcp_registry::McpRegistry>,
    shm: Arc<crate::shm::SharedMemory>,
}

impl VortexServer {
    /// Build server from environment configuration
    pub async fn from_config(config: Arc<vortex_config::VortexConfig>, db: Arc<Database>) -> VortexResult<Self> {
        // Create repositories
        let graph_repo = Arc::new(GraphRepository::new(db.clone()));
        let run_repo = Arc::new(RunRepository::new(db.clone()));
        let tenant_repo = Arc::new(TenantRepository::new(db.clone()));

        // SpiceDB client
        let authz = Arc::new(SpiceDbClient::from_env()?);

        // MCP Toolbox Registry
        let mcp = Arc::new(crate::mcp_registry::McpRegistry::new());

        // Shared Memory (64GB Arena)
        let shm = Arc::new(crate::shm::SharedMemory::open(true)?);

        Ok(Self {
            db,
            graph_repo,
            run_repo,
            tenant_repo,
            authz,
            mcp,
            shm,
        })
    }

    /// Create the Axum router with all dependencies
    pub fn router(&self) -> Router {
        let state = Arc::new(AppState::new(
            self.db.clone(),
            self.graph_repo.clone(),
            self.run_repo.clone(),
            self.tenant_repo.clone(),
            self.authz.clone(),
            self.mcp.clone(),
            self.shm.clone(),
        ));
        create_router(state)
    }

    /// Get the database
    pub fn db(&self) -> &Arc<Database> {
        &self.db
    }

    /// Run the server on specified port with graceful shutdown
    pub async fn run(self, port: u16) -> VortexResult<()> {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
        let router = self.router();

        tracing::info!("VORTEX API server starting on {addr} (Engine: SEAORM)");

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        axum::serve(listener, router)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        tracing::info!("VORTEX API server shut down gracefully");
        Ok(())
    }

    /// Listen for SIGTERM or SIGINT for graceful shutdown
    async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => tracing::info!("Received SIGINT, shutting down..."),
            _ = terminate => tracing::info!("Received SIGTERM, shutting down..."),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════

/// Start VORTEX server (called from main.rs)
pub async fn start_server() -> VortexResult<()> {
    let config = Arc::new(vortex_config::VortexConfig::from_env().map_err(|e| VortexError::Internal(e.to_string()))?);

    // Resilient Connection
    let db = Arc::new(Database::connect(&config).await?);

    // Smart Migration (vortex-core init)
    db.init(true).await?;

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(11188);

    let server = VortexServer::from_config(config, db).await?;

    // Start background clock tick (100ms — sufficient for heartbeat detection)
    let shm_clone = server.shm.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            shm_clone.header().tick();
        }
    });

    server.run(port).await
}
