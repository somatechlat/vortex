use std::collections::HashMap;
use serde_json::Value;
use vortex_protocol::graph::{Node, Edge};
use vortex_protocol::VortexResult;
use crate::client::McpClient;

/// Orchestrates the execution of MCP tools.
pub struct McpProxyExecutor<C: McpClient> {
    client: C,
}

impl<C: McpClient> McpProxyExecutor<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Execute a tool call based on a VORTEX Node.
    pub async fn execute_tool(
        &self,
        node: &Node,
        input_data: HashMap<String, Value>,
    ) -> VortexResult<Value> {
        // Extract tool name from node type (e.g., "mcp.search" -> "search")
        let tool_name = node.r#type.strip_prefix("mcp.").unwrap_or(&node.r#type);

        // Merge node parameters with incoming bus data
        let mut arguments = serde_json::from_slice::<Value>(&node.params_json)
            .unwrap_or(Value::Object(serde_json::Map::new()));

        if let Some(args_map) = arguments.as_object_mut() {
            for (k, v) in input_data {
                args_map.insert(k, v);
            }
        }

        // Call the MCP tool
        let result = self.client.call_tool(tool_name, arguments)
            .await
            .map_err(|e| vortex_protocol::VortexError::IpcFailure {
                reason: format!("MCP Tool Call Failed: {}", e)
            })?;

        Ok(result)
    }
}
