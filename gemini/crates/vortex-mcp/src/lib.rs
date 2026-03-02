//! VORTEX MCP — Model Context Protocol Bridge & Agentic Toolbox
//!
//! Implements MCP client/server communication, tool mapping to VORTEX NodeDefs,
//! and proxy execution of remote tool calls.

// Scaffold phase: allow dead_code until all modules are connected
#![allow(dead_code, unused_variables, unused_imports)]

pub mod client;
pub mod mapper;
pub mod executor;

/// Error types for MCP operations
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum McpError {
        #[error("MCP connection error: {0}")]
        Connection(String),

        #[error("MCP protocol error: {0}")]
        Protocol(String),

        #[error("MCP schema error: {0}")]
        Schema(String),

        #[error("MCP serialization error: {0}")]
        Serialization(#[from] serde_json::Error),
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, error::McpError>;

// Re-exports for convenience
pub use client::{McpClient, StdioMcpClient, McpTool};
pub use mapper::McpToolMapper;
pub use executor::McpProxyExecutor;
