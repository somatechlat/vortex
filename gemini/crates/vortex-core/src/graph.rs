//! Graph DSL - Data structures for the workflow graph
//!
//! Implements the GraphDSL Specification from SRS Section 3.4.1

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Unique identifier for a node (UUID v4)
pub type NodeID = String;

/// Unique identifier for a link/edge
pub type LinkID = String;

/// Port identifier (node_id, port_name)
pub type PortID = (String, String);

/// The canonical graph input format (GraphDSL Specification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDSL {
    /// Schema version
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    
    /// Graph version
    pub version: String,
    
    /// Map of node_id -> Node
    pub nodes: HashMap<NodeID, Node>,
    
    /// List of connections
    pub links: Vec<Link>,
    
    /// Metadata
    #[serde(default)]
    pub meta: GraphMeta,
}

/// A single node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: NodeID,
    
    /// Operation type (e.g., "Loader::Image")
    pub op_type: String,
    
    /// Parameter values
    #[serde(default)]
    pub params: HashMap<String, ParamValue>,
    
    /// UI position
    #[serde(default)]
    pub ui: Option<NodeUI>,
    
    /// Cached inputs (filled during compilation)
    #[serde(skip)]
    pub input_connections: Vec<(String, NodeID)>,
}

/// Parameter value with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamValue {
    #[serde(rename = "type")]
    pub param_type: String,
    pub value: serde_json::Value,
}

/// UI position data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeUI {
    pub x: f64,
    pub y: f64,
}

/// A connection between two ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Source (node_id, port_name)
    pub source: (String, String),
    
    /// Target (node_id, port_name)
    pub target: (String, String),
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphMeta {
    pub priority: Option<String>,
    pub user_id: Option<String>,
}

impl GraphDSL {
    /// Create an empty graph
    pub fn new() -> Self {
        Self {
            schema: Some("http://vortex.ai/schemas/v3/graph.json".to_string()),
            version: "3.0.0".to_string(),
            nodes: HashMap::new(),
            links: Vec::new(),
            meta: GraphMeta::default(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add a link between ports
    pub fn add_link(&mut self, source: PortID, target: PortID) {
        self.links.push(Link { source, target });
    }

    /// Get all parent node IDs for a given node
    pub fn get_parents(&self, node_id: &str) -> Vec<&str> {
        self.links
            .iter()
            .filter(|link| link.target.0 == node_id)
            .map(|link| link.source.0.as_str())
            .collect()
    }

    /// Get all child node IDs for a given node
    pub fn get_children(&self, node_id: &str) -> Vec<&str> {
        self.links
            .iter()
            .filter(|link| link.source.0 == node_id)
            .map(|link| link.target.0.as_str())
            .collect()
    }

    /// Compute total node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Node {
    /// Create a new node
    pub fn new(id: impl Into<String>, op_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            op_type: op_type.into(),
            params: HashMap::new(),
            ui: None,
            input_connections: Vec::new(),
        }
    }

    /// Compute the Merkle hash of this node (SRS Section 3.5.2)
    ///
    /// Hash = SHA256(op_type + sorted_params + parent_hashes)
    pub fn compute_hash(&self, parent_hashes: &[&[u8]]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // 1. Structural Identity
        hasher.update(self.op_type.as_bytes());
        
        // 2. Parameter Identity (Canonical Sorted Order)
        let mut param_keys: Vec<_> = self.params.keys().collect();
        param_keys.sort();
        for key in param_keys {
            hasher.update(key.as_bytes());
            if let Some(param) = self.params.get(key) {
                hasher.update(param.value.to_string().as_bytes());
            }
        }
        
        // 3. Upstream Identity (Recursive)
        for parent_hash in parent_hashes {
            hasher.update(parent_hash);
        }
        
        hasher.finalize().into()
    }

    /// Set a parameter value
    pub fn set_param(&mut self, key: impl Into<String>, param_type: impl Into<String>, value: serde_json::Value) {
        self.params.insert(
            key.into(),
            ParamValue {
                param_type: param_type.into(),
                value,
            },
        );
    }
}

impl Default for GraphDSL {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = GraphDSL::new();
        
        let mut node_a = Node::new("node_a", "Loader::Image");
        node_a.set_param("path", "STRING", serde_json::json!("/data/img.png"));
        
        let node_b = Node::new("node_b", "Process::Resize");
        
        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_link(
            ("node_a".to_string(), "image_out".to_string()),
            ("node_b".to_string(), "image_in".to_string()),
        );
        
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.get_parents("node_b"), vec!["node_a"]);
        assert_eq!(graph.get_children("node_a"), vec!["node_b"]);
    }

    #[test]
    fn test_hash_changes_with_params() {
        let mut node = Node::new("test", "Op::Test");
        let hash1 = node.compute_hash(&[]);
        
        node.set_param("seed", "INT", serde_json::json!(42));
        let hash2 = node.compute_hash(&[]);
        
        assert_ne!(hash1, hash2, "Hash should change when params change");
    }
}
