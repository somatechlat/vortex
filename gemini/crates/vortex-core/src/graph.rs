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

    /// Detect cycles in the graph using DFS (P2.1.3)
    /// Returns Ok(()) if no cycles, Err with cycle nodes if found
    pub fn detect_cycles(&self) -> Result<(), Vec<String>> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycle_nodes = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id)
                && self.dfs_detect_cycle(node_id, &mut visited, &mut rec_stack, &mut cycle_nodes)
            {
                return Err(cycle_nodes);
            }
        }
        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycle(
        &self,
        node_id: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        cycle_nodes: &mut Vec<String>,
    ) -> bool {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        for child_id in self.get_children(node_id) {
            if !visited.contains(child_id) {
                if self.dfs_detect_cycle(child_id, visited, rec_stack, cycle_nodes) {
                    cycle_nodes.push(node_id.to_string());
                    return true;
                }
            } else if rec_stack.contains(child_id) {
                cycle_nodes.push(child_id.to_string());
                cycle_nodes.push(node_id.to_string());
                return true;
            }
        }

        rec_stack.remove(node_id);
        false
    }

    /// Validate the entire graph (P2.1.4 + P2.1.5)
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 1. Check for cycles
        if let Err(nodes) = self.detect_cycles() {
            return Err(ValidationError::CycleDetected { nodes });
        }

        // 2. Check all nodes referenced in links exist
        for link in &self.links {
            if !self.nodes.contains_key(&link.source.0) {
                return Err(ValidationError::NodeNotFound {
                    node_id: link.source.0.clone(),
                });
            }
            if !self.nodes.contains_key(&link.target.0) {
                return Err(ValidationError::NodeNotFound {
                    node_id: link.target.0.clone(),
                });
            }
        }

        // 3. Check for duplicate links
        let mut link_set = std::collections::HashSet::new();
        for link in &self.links {
            let key = (
                link.source.0.clone(),
                link.source.1.clone(),
                link.target.0.clone(),
                link.target.1.clone(),
            );
            if !link_set.insert(key) {
                return Err(ValidationError::DuplicateLink {
                    source: link.source.clone(),
                    target: link.target.clone(),
                });
            }
        }

        Ok(())
    }

    /// Topological sort using Kahn's algorithm
    pub fn topological_sort(&self) -> Result<Vec<String>, ValidationError> {
        // First validate
        self.validate()?;

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }

        // Count incoming edges
        for link in &self.links {
            *in_degree.entry(link.target.0.clone()).or_insert(0) += 1;
        }

        // Find all nodes with no incoming edges
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        // Sort for deterministic order
        queue.sort();

        while let Some(node_id) = queue.pop() {
            result.push(node_id.clone());

            for child_id in self.get_children(&node_id) {
                if let Some(deg) = in_degree.get_mut(child_id) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(child_id.to_string());
                        queue.sort();
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            // Cycle detected (shouldn't happen after validate, but safety check)
            return Err(ValidationError::CycleDetected {
                nodes: vec!["unknown".to_string()],
            });
        }

        Ok(result)
    }
}

/// Graph validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    CycleDetected { nodes: Vec<String> },
    NodeNotFound { node_id: String },
    DuplicateLink { source: PortID, target: PortID },
    TypeMismatch { source_type: String, target_type: String },
    RequiredInputMissing { node_id: String, port_name: String },
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

    #[test]
    fn test_cycle_detection() {
        let mut graph = GraphDSL::new();
        
        graph.add_node(Node::new("a", "Op::A"));
        graph.add_node(Node::new("b", "Op::B"));
        graph.add_node(Node::new("c", "Op::C"));
        
        // A -> B -> C -> A (cycle!)
        graph.add_link(("a".into(), "out".into()), ("b".into(), "in".into()));
        graph.add_link(("b".into(), "out".into()), ("c".into(), "in".into()));
        graph.add_link(("c".into(), "out".into()), ("a".into(), "in".into()));
        
        let result = graph.detect_cycles();
        assert!(result.is_err(), "Should detect cycle");
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = GraphDSL::new();
        
        graph.add_node(Node::new("a", "Op::A"));
        graph.add_node(Node::new("b", "Op::B"));
        graph.add_node(Node::new("c", "Op::C"));
        
        // A -> B -> C (no cycle)
        graph.add_link(("a".into(), "out".into()), ("b".into(), "in".into()));
        graph.add_link(("b".into(), "out".into()), ("c".into(), "in".into()));
        
        let result = graph.detect_cycles();
        assert!(result.is_ok(), "Should not detect cycle");
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = GraphDSL::new();
        
        graph.add_node(Node::new("a", "Op::A"));
        graph.add_node(Node::new("b", "Op::B"));
        graph.add_node(Node::new("c", "Op::C"));
        graph.add_node(Node::new("d", "Op::D"));
        
        // Diamond: A -> B, A -> C, B -> D, C -> D
        graph.add_link(("a".into(), "out".into()), ("b".into(), "in".into()));
        graph.add_link(("a".into(), "out".into()), ("c".into(), "in".into()));
        graph.add_link(("b".into(), "out".into()), ("d".into(), "in".into()));
        graph.add_link(("c".into(), "out".into()), ("d".into(), "in".into()));
        
        let result = graph.topological_sort().expect("Should sort");
        
        // D should come before B and C, A should come first
        let pos_a = result.iter().position(|x| x == "a").unwrap();
        let pos_b = result.iter().position(|x| x == "b").unwrap();
        let pos_c = result.iter().position(|x| x == "c").unwrap();
        let pos_d = result.iter().position(|x| x == "d").unwrap();
        
        assert!(pos_a < pos_b, "A should come before B");
        assert!(pos_a < pos_c, "A should come before C");
        assert!(pos_b < pos_d, "B should come before D");
        assert!(pos_c < pos_d, "C should come before D");
    }

    #[test]
    fn test_validate_missing_node() {
        let mut graph = GraphDSL::new();
        
        graph.add_node(Node::new("a", "Op::A"));
        // Link to non-existent node
        graph.add_link(("a".into(), "out".into()), ("nonexistent".into(), "in".into()));
        
        let result = graph.validate();
        assert!(matches!(result, Err(ValidationError::NodeNotFound { .. })));
    }
}
