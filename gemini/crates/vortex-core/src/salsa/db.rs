//! Salsa Database Implementation
//!
//! Central storage for incremental graph computation.
//! Integrates with the VORTEX Core Engine's graph state.

use salsa::Database as SalsaTrait;
use std::sync::Arc;
use parking_lot::RwLock;

/// The Salsa database storage
#[salsa::database(GraphQueryStorage)]
#[derive(Default)]
pub struct SalsaDb {
    storage: salsa::Storage<Self>,
    /// Current revision counter (for manual invalidation tracking)
    revision: Arc<RwLock<u64>>,
}

impl SalsaDb {
    /// Create a new Salsa database
    pub fn new() -> Self {
        Self {
            storage: salsa::Storage::default(),
            revision: Arc::new(RwLock::new(0)),
        }
    }

    /// Increment revision (called on invalidation)
    pub fn bump_revision(&self) {
        *self.revision.write() += 1;
    }

    /// Get current revision number
    pub fn get_revision(&self) -> u64 {
        *self.revision.read()
    }

    /// Get a node's parameters
    pub fn get_node_params(&self, node_id: &str) -> Option<super::NodeParamDef> {
        super::inputs::node_params(self, node_id.to_string())
    }

    /// Get a node's computed hash
    pub fn get_node_hash(&self, node_id: &str) -> Option<[u8; 32]> {
        super::queries::node_hash(self, node_id.to_string())
    }

    /// Set node parameters (invalidates dependent queries)
    pub fn set_node_params(&mut self, node_id: String, params: super::NodeParamDef) {
        super::inputs::set_node_params(self, node_id, Arc::new(params));
        self.bump_revision();
    }

    /// Invalidate a specific node
    pub fn invalidate_node(&mut self, node_id: &str) {
        // Salsa automatically invalidates when inputs change
        // But we can force it by modifying the input
        if let Some(params) = self.get_node_params(node_id) {
            // Re-set the same value to trigger invalidation
            super::inputs::set_node_params(self, node_id.to_string(), params);
        }
        self.bump_revision();
    }

    /// Check if a node exists
    pub fn has_node(&self, node_id: &str) -> bool {
        self.get_node_params(node_id).is_some()
    }

    /// Clear all data (for testing)
    pub fn clear(&mut self) {
        self.storage = salsa::Storage::default();
        *self.revision.write() = 0;
    }
}

// Implement the salsa::Database trait
impl salsa::Database for SalsaDb {
    fn salsa_runtime(&self) -> &salsa::Runtime {
        self.storage.runtime()
    }
}

// Implement the query group trait
impl super::queries::GraphQuery for SalsaDb {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ParamValue;

    #[test]
    fn test_salsa_db_creation() {
        let db = SalsaDb::new();
        assert_eq!(db.get_revision(), 0);
    }

    #[test]
    fn test_node_params_storage() {
        let mut db = SalsaDb::new();
        
        let params = super::super::NodeParamDef {
            op_type: "Loader::Image".to_string(),
            params: std::collections::HashMap::new(),
        };
        
        db.set_node_params("node_a".to_string(), params.clone());
        
        let stored = db.get_node_params("node_a");
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().op_type, "Loader::Image");
    }

    #[test]
    fn test_revision_increment() {
        let mut db = SalsaDb::new();
        assert_eq!(db.get_revision(), 0);
        
        let params = super::super::NodeParamDef {
            op_type: "Op::Test".to_string(),
            params: std::collections::HashMap::new(),
        };
        
        db.set_node_params("node_x".to_string(), params);
        assert_eq!(db.get_revision(), 1);
    }
}