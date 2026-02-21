use std::sync::Arc;
use dashmap::DashMap;
use vortex_mcp::{McpClient, McpToolMapper};
use vortex_protocol::graph::NodeDef;
use crate::error::VortexResult;

/// Manages dynamically discovered MCP tools and their Vortex NodeDefs.
pub struct McpRegistry {
    /// Discovered NodeDefs keyed by type_id (e.g., "mcp.search")
    node_defs: Arc<DashMap<String, NodeDef>>,
    /// Active MCP clients
    clients: Arc<DashMap<String, Arc<dyn McpClient>>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        let registry = Self {
            node_defs: Arc::new(DashMap::new()),
            clients: Arc::new(DashMap::new()),
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
            let node_def = McpToolMapper::map_to_node_def(&tool);
            self.node_defs.insert(node_def.type_id.clone(), node_def);
        }

        self.clients.insert(id.to_string(), client);
        Ok(())
    }

    /// Get all discovered tool definitions.
    pub fn list_node_defs(&self) -> Vec<NodeDef> {
        self.node_defs.iter().map(|r| r.value().clone()).collect()
    }
}
