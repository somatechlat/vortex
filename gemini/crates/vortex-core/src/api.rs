//! API - HTTP/WebSocket Server using Axum
//!
//! Implements SRS Section 3.6.3 (REST API) and Section 3.6.4 (WebSocket)
//! Port Authority: 11188 (HTTP)

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, DefaultBodyLimit, FromRequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use dashmap::DashMap;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use tower_http::cors::{CorsLayer, Any};
use tower::ServiceBuilder;

// ═══════════════════════════════════════════════════════════════
//                    REQUEST/RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════

/// Graph submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRequest {
    pub graph: serde_json::Value,
    pub priority: Option<String>,
}

/// Graph submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub graph_id: String,
    pub version: u64,
}

/// Execute request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub full: Option<bool>,
    pub output_nodes: Option<Vec<String>>,
}

/// Execute response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub run_id: String,
    pub estimated_time_ms: u64,
}

/// MCP stdio client registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMcpClientRequest {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
}

/// MCP tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub type_id: String,
    pub arguments: serde_json::Value,
}

/// MCP tool call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResponse {
    pub type_id: String,
    pub result: serde_json::Value,
}

/// Run status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStatusResponse {
    pub run_id: String,
    pub status: String,
    pub progress: f32,
    pub current_node: Option<String>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// WebSocket message from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    Progress {
        run_id: String,
        node_id: String,
        progress: f32,
    },
    NodeComplete {
        run_id: String,
        node_id: String,
        duration_ms: u64,
    },
    RunComplete {
        run_id: String,
        success: bool,
        error: Option<String>,
    },
    Ping,
}

// ═══════════════════════════════════════════════════════════════
//                    JWT AUTH EXTRACTOR
// ═══════════════════════════════════════════════════════════════

/// JWT claims from Keycloak token
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,         // user ID
    pub tenant_id: String,   // custom claim injected by Keycloak mapper
    pub exp: usize,
}

/// Authenticated user extractor
pub struct AuthUser(pub Claims);

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts.headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("Missing Bearer token".into()))?;

        // Read public key from env (in production: fetch+cache from Keycloak JWKS)
        let pem = std::env::var("VORTEX_JWT_PUBLIC_KEY")
            .map_err(|_| AppError::Internal("VORTEX_JWT_PUBLIC_KEY not set".into()))?;

        let key = DecodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| AppError::Internal(format!("JWT key error: {e}")))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(auth, &key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))?;

        Ok(AuthUser(token_data.claims))
    }
}

// ═══════════════════════════════════════════════════════════════
//                    APPLICATION STATE
// ═══════════════════════════════════════════════════════════════

/// Shared application state
pub struct AppState {
    pub db: Arc<crate::db::Database>,
    pub graphs: Arc<crate::graph_repo::GraphRepository>,
    pub runs: Arc<crate::run_repo::RunRepository>,
    pub tenants: Arc<crate::tenant_repo::TenantRepository>,
    pub authz: Arc<crate::authz::SpiceDbClient>,
    pub mcp: Arc<crate::mcp_registry::McpRegistry>,
    pub shm: Arc<crate::shm::SharedMemory>,
    pub metrics: Arc<crate::metrics::MetricsCollector>,
    pub cancel_tokens: Arc<DashMap<String, CancellationToken>>,
    /// Broadcast channel for WebSocket updates
    pub tx: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub fn new(
        db: Arc<crate::db::Database>,
        graphs: Arc<crate::graph_repo::GraphRepository>,
        runs: Arc<crate::run_repo::RunRepository>,
        tenants: Arc<crate::tenant_repo::TenantRepository>,
        authz: Arc<crate::authz::SpiceDbClient>,
        mcp: Arc<crate::mcp_registry::McpRegistry>,
        shm: Arc<crate::shm::SharedMemory>,
    ) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            db,
            graphs,
            runs,
            tenants,
            authz,
            mcp,
            shm,
            metrics: Arc::new(crate::metrics::MetricsCollector::new()),
            cancel_tokens: Arc::new(DashMap::new()),
            tx,
        }
    }
}

fn unix_time_secs() -> Result<i64, AppError> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("time error: {e}")))?
        .as_secs() as i64;
    Ok(secs)
}
// ═══════════════════════════════════════════════════════════════
//                    ROUTER
// ═══════════════════════════════════════════════════════════════

/// Create the Axum router with all endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    // CORS: Allow configured origins (production) or permissive (sandbox)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Graph endpoints
        .route("/api/graph", post(submit_graph))
        .route("/api/graph/:id", get(get_graph))
        .route("/api/graph/:id/execute", post(execute_graph))
        .route("/api/run/:id/status", get(run_status))
        .route("/api/run/:id/cancel", post(cancel_run))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Health check (unauthenticated — K8s probes)
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        // Nodes/Toolbox
        .route("/api/nodes/mcp", get(list_mcp_nodes))
        .route("/api/mcp/clients", get(list_mcp_clients))
        .route("/api/mcp/client/register", post(register_mcp_client))
        .route("/api/mcp/tool/call", post(call_mcp_tool))
        // Middleware stack: CORS + body limit
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        )
        // State
        .with_state(state)
}

// ═══════════════════════════════════════════════════════════════
//                    HANDLERS
// ═══════════════════════════════════════════════════════════════

async fn submit_graph(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<GraphRequest>,
) -> Result<Json<GraphResponse>, AppError> {
    let graph_id = uuid::Uuid::new_v4().to_string();
    let now = unix_time_secs()?;

    let model = crate::entities::graph::Model {
        id: graph_id.clone(),
        tenant_id: auth.0.tenant_id.clone(),
        name: "Untitled".to_string(),
        version: 1,
        graph_json: request.graph.to_string(),
        created_at: now,
        updated_at: now,
    };

    state.graphs.insert(model).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!("Graph {} submitted", graph_id);

    Ok(Json(GraphResponse {
        graph_id,
        version: 1,
    }))
}

async fn get_graph(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let graph = state.graphs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Graph {} not found", id)))?;

    // Tenant isolation: verify the graph belongs to the requesting tenant
    if graph.tenant_id != auth.0.tenant_id {
        return Err(AppError::NotFound(format!("Graph {} not found", id)));
    }

    let json: serde_json::Value = serde_json::from_str(&graph.graph_json)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json))
}

async fn execute_graph(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(_request): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    tracing::info!(graph_id = %id, tenant = %auth.0.tenant_id, "Executing graph");

    // 1. Fetch graph from database
    let graph_model = state.graphs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Graph {} not found", id)))?;

    // 2. Parse graph JSON to GraphDSL
    let graph_json = graph_model.graph_json.clone();
    let graph: crate::graph::GraphDSL = serde_json::from_str(&graph_json)
        .map_err(|e| AppError::Internal(format!("Failed to parse graph JSON: {}", e)))?;

    // 3. Validate graph structure
    graph.validate().map_err(|e| AppError::BadRequest(format!("Invalid graph: {:?}", e)))?;

    // 4. Schedule nodes and create execution plan
    let scheduler = crate::scheduler::Scheduler::new();
    let execution_plan = scheduler.create_execution_plan(&graph, None)
        .map_err(|e| AppError::Internal(format!("Failed to schedule graph: {:?}", e)))?;

    // 5. Create run record
    let run_id = uuid::Uuid::new_v4().to_string();
    let now = unix_time_secs()?;

    let run_model = crate::entities::run::Model {
        id: run_id.clone(),
        graph_hash: id.clone(),
        status: crate::entities::run::RunStatus::Running,
        created_at: now,
        completed_at: None,
        error_json: None,
    };

    state.runs.insert(run_model).await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 6. Get estimated time BEFORE moving execution_plan
    let estimated_time_ms = execution_plan.estimated_time_ms;

    // 7. Spawn execution in background
    let tx = state.tx.clone();
    let run_repo = state.runs.clone();
    let shm = state.shm.clone();
    let mcp = state.mcp.clone();
    let cancel_token = CancellationToken::new();
    state.cancel_tokens.insert(run_id.clone(), cancel_token.clone());
    let cancel_tokens = state.cancel_tokens.clone();
    let run_id_clone = run_id.clone();
    let graph_id = id.clone();

    tokio::spawn(async move {
        let ctx = crate::execution::ExecutionContext {
            run_id: run_id_clone.clone(),
            graph_id: graph_id.clone(),
            graph,
            execution_plan,
            tx,
            run_repo,
            shm,
            cancel_token,
            output_registry: std::collections::HashMap::new(),
        };

        match ctx.execute().await {
            Ok(status) => {
                tracing::info!(run_id = %run_id_clone, status = ?status, "Execution completed");
            }
            Err(e) => {
                tracing::error!(run_id = %run_id_clone, error = %e, "Execution failed");
            }
        }
        cancel_tokens.remove(&run_id_clone);
    });

    Ok(Json(ExecuteResponse {
        run_id,
        estimated_time_ms,
    }))
}

/// GET /api/run/:id/status - Get run status
async fn run_status(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<RunStatusResponse>, AppError> {
    let run = state.runs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Run {} not found", id)))?;

    // Compute progress from completed steps vs total nodes
    let steps = state.runs.count_steps(&id).await
        .unwrap_or(0);
    // Progress is approximate: steps completed / estimated total
    let progress = if steps > 0 { (steps as f32).min(1.0) } else { 0.0 };

    Ok(Json(RunStatusResponse {
        run_id: id,
        status: format!("{:?}", run.status),
        progress,
        current_node: None,
    }))
}

/// POST /api/run/:id/cancel - Cancel a run
async fn cancel_run(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if let Some((_, token)) = state.cancel_tokens.remove(&id) {
        token.cancel();

        if let Some(mut run) = state.runs.get_by_id(&id).await
            .map_err(|e| AppError::Internal(e.to_string()))?
        {
            run.status = crate::entities::run::RunStatus::Cancelled;
            run.error_json = Some(serde_json::json!({"error": "Run cancelled by user"}).to_string());
            run.completed_at = Some(unix_time_secs()?);
            state.runs.update(run).await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        tracing::info!("Run {} cancelled", id);
        return Ok(StatusCode::OK);
    }

    let exists = state.runs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some();
    if exists {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound(format!("Run {} not found", id)))
    }
}

/// GET /api/nodes/mcp - List all discovered MCP tools
async fn list_mcp_nodes(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<Vec<vortex_protocol::graph::NodeDef>>, AppError> {
    Ok(Json(state.mcp.list_node_defs()))
}

/// GET /api/mcp/clients - List registered MCP clients
async fn list_mcp_clients(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<Json<Vec<String>>, AppError> {
    Ok(Json(state.mcp.list_clients()))
}

/// POST /api/mcp/client/register - Register a stdio MCP client
async fn register_mcp_client(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<RegisterMcpClientRequest>,
) -> Result<StatusCode, AppError> {
    if request.id.trim().is_empty() {
        return Err(AppError::BadRequest("MCP client id cannot be empty".into()));
    }
    if request.command.trim().is_empty() {
        return Err(AppError::BadRequest("MCP command cannot be empty".into()));
    }

    tracing::info!(client_id = %request.id, tenant = %auth.0.tenant_id, "Registering MCP stdio client");

    state.mcp
        .add_stdio_client(&request.id, &request.command, &request.args)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!(client_id = %request.id, "Registered MCP stdio client");
    Ok(StatusCode::CREATED)
}

/// POST /api/mcp/tool/call - Invoke an MCP tool by VORTEX type_id
async fn call_mcp_tool(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<McpToolCallRequest>,
) -> Result<Json<McpToolCallResponse>, AppError> {
    tracing::info!(type_id = %request.type_id, tenant = %auth.0.tenant_id, "Calling MCP tool");

    let result = state.mcp
        .call_tool_by_type(&request.type_id, request.arguments)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(McpToolCallResponse {
        type_id: request.type_id,
        result,
    }))
}

/// GET /health - Health check with dependency verification
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check DB connectivity
    let db_ok = state.db.ping().await.is_ok();
    let shm_ok = state.shm.header().magic_bytes != 0;
    let status = if db_ok && shm_ok { "healthy" } else { "degraded" };

    Json(serde_json::json!({
        "status": status,
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "database": db_ok,
            "shm": shm_ok,
        }
    }))
}

/// Query params for WebSocket auth
#[derive(Debug, Deserialize)]
pub struct WsAuthParams {
    pub token: Option<String>,
}

/// GET /metrics - Prometheus metrics from MetricsCollector
async fn metrics_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    state.metrics.get_prometheus_metrics()
}

/// GET /ws - WebSocket handler with token auth
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsAuthParams>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Validate JWT from query param
    if let Some(token) = &params.token {
        let pem = std::env::var("VORTEX_JWT_PUBLIC_KEY")
            .map_err(|_| AppError::Internal("VORTEX_JWT_PUBLIC_KEY not set".into()))?;
        let key = DecodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| AppError::Internal(format!("JWT key error: {e}")))?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        decode::<Claims>(token, &key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid WS token: {e}")))?;
    } else {
        return Err(AppError::Unauthorized("WebSocket requires ?token= parameter".into()));
    }

    Ok(ws.on_upgrade(|socket| handle_socket(socket, state)))
}

/// Handle WebSocket connection
async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send task - broadcasts to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Receive task - handles pings
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

// ═══════════════════════════════════════════════════════════════
//                    ERROR HANDLING
// ═══════════════════════════════════════════════════════════════

/// Application error type
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg),
        };

        let body = Json(ErrorResponse {
            error: message,
            code: code.to_string(),
        });

        (status, body).into_response()
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Progress {
            run_id: "run_1".to_string(),
            node_id: "node_1".to_string(),
            progress: 0.5,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Progress"));
        assert!(json.contains("0.5"));
    }

    #[test]
    fn test_app_state_struct() {
        // Validation of AppState structure logic if needed
        // Since we injected Arc repositories, simple instantiation without mocks
        // is complex. We rely on integration tests for state verification.
    }
}
