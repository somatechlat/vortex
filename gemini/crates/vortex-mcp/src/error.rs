use serde::{Deserialize, Serialize};
use thiserror::Error;
use vortex_protocol::VortexError;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("MCP Protocol Error: {0}")]
    Protocol(String),

    #[error("MCP Connection Error: {0}")]
    Connection(String),

    #[error("MCP Schema Error: {0}")]
    Schema(String),

    #[error("Vortex Protocol Error: {0}")]
    Vortex(#[from] VortexError),

    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),
}
