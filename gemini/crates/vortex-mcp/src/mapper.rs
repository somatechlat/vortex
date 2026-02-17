use serde_json::Value;
use vortex_protocol::graph::{NodeDef, PortDef, DataType};
use crate::client::McpTool;

/// Maps MCP Tool definitions to VORTEX NodeDefs.
pub struct McpToolMapper;

impl McpToolMapper {
    /// Map an MCP tool to a native VORTEX NodeDef.
    pub fn map_to_node_def(tool: &McpTool) -> NodeDef {
        let mut inputs = Vec::new();

        // Parse input_schema (JSON Schema)
        if let Some(properties) = tool.input_schema.get("properties").and_then(|p| p.as_object()) {
            for (name, schema) in properties {
                let data_type = Self::map_json_type_to_vortex(schema);
                let description = schema.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();

                inputs.push(PortDef {
                    name: name.clone(),
                    label: name.clone(),
                    data_type: data_type as i32,
                    required: tool.input_schema.get("required")
                        .and_then(|r| r.as_array())
                        .map(|r| r.contains(&Value::String(name.clone())))
                        .unwrap_or(false),
                    default_json: Vec::new(),
                    description,
                });
            }
        }

        // MCP tools traditionally return a single object or string.
        // We map this to a single output port for now.
        let outputs = vec![PortDef {
            name: "output".into(),
            label: "Output".into(),
            data_type: DataType::DataString as i32, // Defaulting to string for toolbox
            required: true,
            default_json: Vec::new(),
            description: "Result from tool call".into(),
        }];

        NodeDef {
            type_id: format!("mcp.{}", tool.name),
            display_name: tool.name.clone(),
            category: "MCP Toolbox".into(),
            description: tool.description.clone().unwrap_or_default(),
            inputs,
            outputs,
            author: "MCP Server".into(),
            version: "1.0.0".into(),
        }
    }

    fn map_json_type_to_vortex(schema: &Value) -> DataType {
        match schema.get("type").and_then(|t| t.as_str()) {
            Some("string") => DataType::DataString,
            Some("number") | Some("integer") => DataType::DataFloat,
            Some("boolean") => DataType::DataBoolean,
            _ => DataType::DataString,
        }
    }
}
