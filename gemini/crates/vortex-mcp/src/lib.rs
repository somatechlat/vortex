//! VORTEX MCP Bridge
//!
//! Implements the Model Context Protocol (MCP) bridge for VORTEX.
//! This allows external tools to be used as native nodes in the VORTEX Vertical Rack.

pub mod client;
pub mod mapper;
pub mod executor;
pub mod error;

pub use client::McpClient;
pub use mapper::McpToolMapper;
pub use executor::McpProxyExecutor;
pub use error::McpError;

pub type McpResult<T> = Result<T, McpError>;
