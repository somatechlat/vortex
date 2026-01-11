//! Salsa Module - Incremental Computation Database
//!
//! Implements SRS Section 3.2.2 [F-02] Incremental Computation
//! 
//! # Architecture
//!
//! Salsa provides a demand-driven, incremental computation framework.
//! When a graph parameter changes, only affected nodes re-compute.
//!
//! # Key Concepts:
//!
//! - **Input Queries**: Node parameters (mutable)
//! - **Derived Queries**: Node hashes (computed from inputs)
//! - **Revisions**: Track when inputs change
//! - **Durability**: Propagate change information
//!
//! # Usage:
//!
//! ```rust
//! use salsa::Database;
//!
//! let db = SalsaDb::new();
//! db.set_node_params("node_a", Arc::new(params));
//! let hash = db.node_hash("node_a");
//! ```

pub mod db;
pub mod queries;
pub mod inputs;

// Re-exports for convenience
pub use db::SalsaDb;
pub use queries::{GraphQuery, NodeHashQuery};
pub use inputs::{NodeParams, NodeDef};

use crate::graph::NodeID;
use std::sync::Arc;

/// Node definition for Salsa input
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeParamDef {
    pub op_type: String,
    pub params: std::collections::HashMap<String, crate::graph::ParamValue>,
}

/// Salsa database trait
pub trait SalsaDatabase {
    /// Get node parameters (input query)
    fn get_node_params(&self, node_id: NodeID) -> Option<Arc<NodeParamDef>>;
    
    /// Get node hash (derived query)
    fn get_node_hash(&self, node_id: NodeID) -> Option<[u8; 32]>;
    
    /// Set node parameters
    fn set_node_params(&mut self, node_id: NodeID, params: Arc<NodeParamDef>);
    
    /// Invalidate a node
    fn invalidate_node(&mut self, node_id: NodeID);
    
    /// Get current revision
    fn revision(&self) -> u64;
}