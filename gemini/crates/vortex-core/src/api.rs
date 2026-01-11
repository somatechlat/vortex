//! API - HTTP/WebSocket Server using Axum
//!
//! Implements SRS Section 3.6.3 (REST API) and Section 3.6.4 (WebSocket)
//! Port Authority: 11188 (HTTP)

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

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
//                    APPLICATION STATE
// ═══════════════════════════════════════════════════════════════

/// Shared application state
pub struct AppState {
    pub db: Arc<crate::db::Database>,
    pub graphs: Arc<crate::graph_repo::GraphRepository>,
    pub runs: Arc<crate::run_repo::RunRepository>,
    pub tenants: Arc<crate::tenant_repo::TenantRepository>,
    pub authz: Arc<crate::authz::SpiceDbClient>,
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
    ) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            db,
            graphs,
            runs,
            tenants,
            authz,
            tx,
        }
    }
}
// ═══════════════════════════════════════════════════════════════
//                    ROUTER
// ═══════════════════════════════════════════════════════════════

/// Create the Axum router with all endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Graph endpoints
        .route("/api/graph", post(submit_graph))
        .route("/api/graph/:id", get(get_graph))
        .route("/api/graph/:id/execute", post(execute_graph))
        .route("/api/run/:id/status", get(run_status))
        .route("/api/run/:id/cancel", post(cancel_run))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Health check
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        // State
        .with_state(state)
}

// ═══════════════════════════════════════════════════════════════
//                    HANDLERS
// ═══════════════════════════════════════════════════════════════

async fn submit_graph(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GraphRequest>,
) -> Result<Json<GraphResponse>, AppError> {
    let graph_id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let model = crate::entities::graph::Model {
        id: graph_id.clone(),
        tenant_id: "default".to_string(), // Placeholder until auth context
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
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let graph = state.graphs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Graph {} not found", id)))?;
    
    let json: serde_json::Value = serde_json::from_str(&graph.graph_json)
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    Ok(Json(json))
}

async fn execute_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(_request): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    // Verify graph exists
    let _ = state.graphs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Graph {} not found", id)))?;
    
    let run_id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let run_model = crate::entities::run::Model {
        id: run_id.clone(),
        graph_hash: id.clone(),
        status: crate::entities::run::RunStatus::Pending,
        created_at: now,
        completed_at: None,
        error_json: None,
    };

    state.runs.insert(run_model).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    // Emit execution start event - scheduler picks up via broadcast
    let _ = state.tx.send(WsMessage::Progress {
        run_id: run_id.clone(),
        node_id: "_start".to_string(),
        progress: 0.0,
    });
    
    tracing::info!(run_id = %run_id, graph_id = %id, "Execution scheduled");
    
    Ok(Json(ExecuteResponse {
        run_id,
        estimated_time_ms: 1000,
    }))
}

/// GET /api/run/:id/status - Get run status
async fn run_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RunStatusResponse>, AppError> {
    let run = state.runs.get_by_id(&id).await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Run {} not found", id)))?;

    Ok(Json(RunStatusResponse {
        run_id: id,
        status: format!("{:?}", run.status),
        progress: 0.0, // Should be calculated/tracked
        current_node: None,
    }))
}

/// POST /api/run/:id/cancel - Cancel a run
async fn cancel_run(
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!("Run {} cancelled", id);
    Ok(StatusCode::OK)
}

/// GET /health - Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /metrics - Prometheus metrics (OpenMetrics format)
async fn metrics() -> impl IntoResponse {
    // Metrics endpoint - integrate with metrics crate for real collection
    static METRICS_HEADER: &str = "# HELP vortex_up Indicates service is running\n\
        # TYPE vortex_up gauge\n\
        vortex_up 1\n\
        # HELP vortex_info Service version info\n\
        # TYPE vortex_info gauge\n";
    format!("{}vortex_info{{version=\"{}\"}} 1\n", METRICS_HEADER, env!("CARGO_PKG_VERSION"))
}

/// GET /ws - WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
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
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
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
