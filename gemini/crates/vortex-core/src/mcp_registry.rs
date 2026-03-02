use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;
use vortex_mcp::{McpClient, McpToolMapper};
use vortex_mcp::client::StdioMcpClient;
use vortex_protocol::graph::NodeDef;
use crate::error::VortexResult;

/// Manages dynamically discovered MCP tools and their Vortex NodeDefs.
pub struct McpRegistry {
    /// Discovered NodeDefs keyed by type_id (e.g., "mcp.search")
    node_defs: Arc<DashMap<String, NodeDef>>,
    /// Active MCP clients
    clients: Arc<DashMap<String, Arc<dyn McpClient>>>,
    /// Tool type_id -> client ID routing map (e.g., mcp.search.web -> web)
    tool_routes: Arc<DashMap<String, String>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        let registry = Self {
            node_defs: Arc::new(DashMap::new()),
            clients: Arc::new(DashMap::new()),
            tool_routes: Arc::new(DashMap::new()),
        };

        // Register Native Units automatically
        registry.register_native_units();

        registry
    }

    fn register_native_units(&self) {
        use crate::units::VortexUnit;
        use crate::units::loader::ModelLoader;
        use crate::units::sampler::KSampler;
        use crate::units::encoder::TextEncoder;

        let units: Vec<Box<dyn VortexUnit>> = vec![
            Box::new(ModelLoader),
            Box::new(KSampler),
            Box::new(TextEncoder),
        ];

        for unit in units {
            let def = unit.definition();
            self.node_defs.insert(def.type_id.clone(), def);
        }
    }

    /// Register a new MCP client and discover its tools.
    pub async fn add_client(&self, id: &str, client: Arc<dyn McpClient>) -> VortexResult<()> {
        let tools = client.list_tools().await
            .map_err(|e| crate::error::VortexError::IpcFailure { reason: e.to_string() })?;

        for tool in tools {
            let node_def = McpToolMapper::map_to_node_def_with_namespace(&tool, Some(id));
            self.tool_routes.insert(node_def.type_id.clone(), id.to_string());
            self.node_defs.insert(node_def.type_id.clone(), node_def);
        }

        self.clients.insert(id.to_string(), client);
        Ok(())
    }

    /// Spawn a stdio MCP process and register it.
    pub async fn add_stdio_client(
        &self,
        id: &str,
        command: &str,
        args: &[String],
    ) -> VortexResult<()> {
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let client = StdioMcpClient::spawn(command, &arg_refs)
            .map_err(|e| crate::error::VortexError::IpcFailure { reason: e.to_string() })?;
        self.add_client(id, Arc::new(client)).await
    }

    /// Call a discovered MCP tool by its VORTEX type_id.
    pub async fn call_tool_by_type(&self, type_id: &str, arguments: Value) -> VortexResult<Value> {
        let client_id = self.tool_routes.get(type_id)
            .map(|v| v.value().clone())
            .ok_or_else(|| crate::error::VortexError::Internal(format!("MCP tool not registered: {}", type_id)))?;

        let client = self.clients.get(&client_id)
            .map(|v| v.value().clone())
            .ok_or_else(|| crate::error::VortexError::Internal(format!("MCP client not found: {}", client_id)))?;

        let tool_name = type_id
            .strip_prefix("mcp.")
            .and_then(|rest| rest.split_once('.').map(|(_, tool)| tool.to_string()))
            .ok_or_else(|| crate::error::VortexError::Internal(format!("Invalid MCP type_id: {}", type_id)))?;

        client.call_tool(&tool_name, arguments).await
            .map_err(|e| crate::error::VortexError::IpcFailure { reason: e.to_string() })
    }

    /// Get all discovered tool definitions.
    pub fn list_node_defs(&self) -> Vec<NodeDef> {
        self.node_defs.iter().map(|r| r.value().clone()).collect()
    }

    /// List registered MCP client identifiers.
    pub fn list_clients(&self) -> Vec<String> {
        self.clients.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Remove a dead MCP client and its tool routes
    pub fn remove_client(&self, id: &str) {
        self.clients.remove(id);
        // Remove tool routes that point to this client
        let dead_routes: Vec<String> = self.tool_routes.iter()
            .filter(|r| r.value() == id)
            .map(|r| r.key().clone())
            .collect();
        for route in &dead_routes {
            self.tool_routes.remove(route);
            self.node_defs.remove(route);
        }
        tracing::info!("Removed MCP client '{}' and {} tool routes", id, dead_routes.len());
    }

    /// Health-check all MCP clients, removing dead ones
    pub async fn health_check(&self) -> Vec<String> {
        let client_ids: Vec<String> = self.list_clients();
        let mut dead: Vec<String> = Vec::new();

        for id in &client_ids {
            if let Some(client) = self.clients.get(id) {
                // Try listing tools — if the process is dead, this will fail
                if client.list_tools().await.is_err() {
                    dead.push(id.clone());
                }
            }
        }

        for id in &dead {
            self.remove_client(id);
        }

        dead
    }
}
