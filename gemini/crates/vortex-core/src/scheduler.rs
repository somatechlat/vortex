//! Scheduler - Topological Compilation using Kahn's Algorithm
//!
//! Implements SRS Section 3.2.1 [F-01] Topological Compilation

use crate::error::{VortexError, VortexResult};
use crate::graph::{GraphDSL, NodeID};
use std::collections::{HashMap, VecDeque};

/// The Scheduler trait as defined in SRS Section 3.6.1
pub trait SchedulerTrait {
    /// Converts a raw graph into an executable sequence.
    ///
    /// # Arguments
    /// * `graph` - The user-defined graph topology.
    ///
    /// # Returns
    /// * `Ok(Vec<NodeID>)` - The order of execution.
    /// * `Err(VortexError::CycleDetected)` - If the graph is not a DAG.
    fn schedule(&self, graph: &GraphDSL) -> VortexResult<Vec<NodeID>>;

    /// Validates type compatibility between connected ports.
    fn validate_types(&self, graph: &GraphDSL) -> Vec<VortexError>;
}

/// Default scheduler implementation using Kahn's Algorithm
pub struct Scheduler {
    /// Type compatibility rules (source_type -> allowed_target_types)
    type_rules: HashMap<String, Vec<String>>,
}

impl Scheduler {
    /// Create a new scheduler with default type rules
    pub fn new() -> Self {
        let mut type_rules = HashMap::new();
        
        // Default type compatibility (same type always allowed)
        type_rules.insert("IMAGE".to_string(), vec!["IMAGE".to_string()]);
        type_rules.insert("LATENT".to_string(), vec!["LATENT".to_string()]);
        type_rules.insert("CONDITIONING".to_string(), vec!["CONDITIONING".to_string()]);
        type_rules.insert("MODEL".to_string(), vec!["MODEL".to_string()]);
        type_rules.insert("CLIP".to_string(), vec!["CLIP".to_string()]);
        type_rules.insert("VAE".to_string(), vec!["VAE".to_string()]);
        type_rules.insert("MASK".to_string(), vec!["MASK".to_string(), "IMAGE".to_string()]);
        
        Self { type_rules }
    }

    /// Add a type compatibility rule
    pub fn add_type_rule(&mut self, source: impl Into<String>, targets: Vec<String>) {
        self.type_rules.insert(source.into(), targets);
    }

    /// Kahn's Algorithm implementation (SRS Section 3.2.1)
    ///
    /// Processing:
    /// 1. Initialize `InDegree` map for all nodes.
    /// 2. Identify Roots (`InDegree == 0`). Push to `Queue`.
    /// 3. Loop while `Queue` is not empty:
    ///    - Pop `N`. Add to `ExecutionList`.
    ///    - Decrement `InDegree` of neighbors.
    ///    - If `NeighborDegree == 0`, Push to `Queue`.
    /// 4. If `ExecutionList.len() != NodeCount`, Return `Error::CycleDetected`.
    fn kahn_sort(&self, graph: &GraphDSL) -> VortexResult<Vec<NodeID>> {
        let node_count = graph.node_count();
        
        // Step 1: Initialize InDegree map
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for node_id in graph.nodes.keys() {
            in_degree.insert(node_id.as_str(), 0);
        }
        
        // Calculate in-degrees from links
        for link in &graph.links {
            if let Some(degree) = in_degree.get_mut(link.target.0.as_str()) {
                *degree += 1;
            }
        }
        
        // Step 2: Find roots (InDegree == 0)
        let mut queue: VecDeque<&str> = VecDeque::new();
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id);
            }
        }
        
        // Step 3: Process queue
        let mut execution_list: Vec<NodeID> = Vec::with_capacity(node_count);
        
        while let Some(node_id) = queue.pop_front() {
            execution_list.push(node_id.to_string());
            
            // Decrement in-degree of children
            for child_id in graph.get_children(node_id) {
                if let Some(degree) = in_degree.get_mut(child_id) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(child_id);
                    }
                }
            }
        }
        
        // Step 4: Check for cycle
        if execution_list.len() != node_count {
            // Find nodes involved in cycle (those with remaining in-degree > 0)
            let cycle_nodes: Vec<String> = in_degree
                .iter()
                .filter(|(_, &degree)| degree > 0)
                .map(|(id, _)| id.to_string())
                .collect();
            
            return Err(VortexError::CycleDetected { nodes: cycle_nodes });
        }
        
        Ok(execution_list)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulerTrait for Scheduler {
    fn schedule(&self, graph: &GraphDSL) -> VortexResult<Vec<NodeID>> {
        // First validate types
        let type_errors = self.validate_types(graph);
        if !type_errors.is_empty() {
            // Return the first type error
            return Err(type_errors.into_iter().next().unwrap());
        }
        
        // Then perform topological sort
        self.kahn_sort(graph)
    }

    fn validate_types(&self, _graph: &GraphDSL) -> Vec<VortexError> {
        // TODO: Implement type validation based on port metadata
        // For now, return empty (all types valid)
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_linear_chain() {
        // SRS VCRM: Input `A->B->C` yields `Vec[A,B,C]`
        let mut graph = GraphDSL::new();
        graph.add_node(Node::new("A", "Op::A"));
        graph.add_node(Node::new("B", "Op::B"));
        graph.add_node(Node::new("C", "Op::C"));
        
        graph.add_link(("A".into(), "out".into()), ("B".into(), "in".into()));
        graph.add_link(("B".into(), "out".into()), ("C".into(), "in".into()));
        
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(&graph).unwrap();
        
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cycle_error() {
        // SRS VCRM: Input `A->B->A` returns `Err(CycleDetected)`
        let mut graph = GraphDSL::new();
        graph.add_node(Node::new("A", "Op::A"));
        graph.add_node(Node::new("B", "Op::B"));
        
        graph.add_link(("A".into(), "out".into()), ("B".into(), "in".into()));
        graph.add_link(("B".into(), "out".into()), ("A".into(), "in".into()));
        
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(&graph);
        
        assert!(matches!(result, Err(VortexError::CycleDetected { .. })));
    }

    #[test]
    fn test_diamond_dependency() {
        // A -> B -> D
        // A -> C -> D
        let mut graph = GraphDSL::new();
        graph.add_node(Node::new("A", "Op::A"));
        graph.add_node(Node::new("B", "Op::B"));
        graph.add_node(Node::new("C", "Op::C"));
        graph.add_node(Node::new("D", "Op::D"));
        
        graph.add_link(("A".into(), "out".into()), ("B".into(), "in".into()));
        graph.add_link(("A".into(), "out".into()), ("C".into(), "in".into()));
        graph.add_link(("B".into(), "out".into()), ("D".into(), "in".into()));
        graph.add_link(("C".into(), "out".into()), ("D".into(), "in".into()));
        
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(&graph).unwrap();
        
        // A must come first, D must come last
        assert_eq!(result[0], "A");
        assert_eq!(result[3], "D");
        // B and C can be in any order between A and D
        assert!(result.contains(&"B".to_string()));
        assert!(result.contains(&"C".to_string()));
    }
}
