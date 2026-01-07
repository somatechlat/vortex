//! API - HTTP/WebSocket Server using Axum
//!
//! Implements SRS Section 3.6.3 (REST API) and Section 3.6.4 (WebSocket)
//! Port Authority: 11000 (HTTP)

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
    /// Broadcast channel for WebSocket updates
    pub tx: broadcast::Sender<WsMessage>,
    /// In-memory graph storage (placeholder)
    pub graphs: parking_lot::RwLock<std::collections::HashMap<String, serde_json::Value>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            tx,
            graphs: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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

/// POST /api/graph - Submit a graph
async fn submit_graph(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GraphRequest>,
) -> Result<Json<GraphResponse>, AppError> {
    let graph_id = uuid::Uuid::new_v4().to_string();
    
    // Store graph
    state.graphs.write().insert(graph_id.clone(), request.graph);
    
    tracing::info!("Graph {} submitted", graph_id);
    
    Ok(Json(GraphResponse {
        graph_id,
        version: 1,
    }))
}

/// GET /api/graph/:id - Get a graph
async fn get_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let graphs = state.graphs.read();
    
    match graphs.get(&id) {
        Some(graph) => Ok(Json(graph.clone())),
        None => Err(AppError::NotFound(format!("Graph {} not found", id))),
    }
}

/// POST /api/graph/:id/execute - Execute a graph
async fn execute_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(_request): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    // Verify graph exists
    if !state.graphs.read().contains_key(&id) {
        return Err(AppError::NotFound(format!("Graph {} not found", id)));
    }
    
    let run_id = uuid::Uuid::new_v4().to_string();
    
    // TODO: Actually schedule execution
    tracing::info!("Execution {} started for graph {}", run_id, id);
    
    Ok(Json(ExecuteResponse {
        run_id,
        estimated_time_ms: 1000,
    }))
}

/// GET /api/run/:id/status - Get run status
async fn run_status(
    Path(id): Path<String>,
) -> Result<Json<RunStatusResponse>, AppError> {
    // TODO: Query actual run status
    Ok(Json(RunStatusResponse {
        run_id: id,
        status: "RUNNING".to_string(),
        progress: 0.5,
        current_node: Some("node_1".to_string()),
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

/// GET /metrics - Prometheus metrics
async fn metrics() -> impl IntoResponse {
    // TODO: Actual Prometheus metrics
    "# HELP vortex_requests_total Total requests\n\
     # TYPE vortex_requests_total counter\n\
     vortex_requests_total 0\n"
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
//                    SERVER
// ═══════════════════════════════════════════════════════════════

/// Start the HTTP server
pub async fn serve(addr: std::net::SocketAddr) -> std::io::Result<()> {
    let state = Arc::new(AppState::new());
    let app = create_router(state);
    
    tracing::info!("Starting API server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

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
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.graphs.read().is_empty());
    }
}
