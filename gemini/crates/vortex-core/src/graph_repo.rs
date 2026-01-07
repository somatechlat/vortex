//! Graph Repository - Multi-DB Storage for Graphs
//!
//! Implements graph persistence with SeaORM for database agnosticism.

use sea_orm::*;
use crate::error::{VortexError, VortexResult};
use crate::entities::graph;
use crate::db::Database;
use std::sync::Arc;

/// Graph repository implementation using SeaORM
pub struct GraphRepository {
    db: Arc<Database>,
}

impl GraphRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Insert a new graph
    pub async fn insert(&self, model: graph::Model) -> VortexResult<()> {
        let active_model: graph::ActiveModel = model.into();
        active_model.insert(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Get graph by ID
    pub async fn get_by_id(&self, id: &str) -> VortexResult<Option<graph::Model>> {
        graph::Entity::find_by_id(id.to_string())
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))
    }

    /// Update a graph
    pub async fn update(&self, model: graph::Model) -> VortexResult<()> {
        let active_model: graph::ActiveModel = model.into();
        active_model.update(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Delete a graph
    pub async fn delete(&self, id: &str) -> VortexResult<()> {
        graph::Entity::delete_by_id(id.to_string())
            .exec(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// List graphs by tenant
    pub async fn list_by_tenant(&self, tenant_id: &str, limit: u64, offset: u64) -> VortexResult<Vec<graph::Model>> {
        graph::Entity::find()
            .filter(graph::Column::TenantId.eq(tenant_id))
            .order_by_desc(graph::Column::UpdatedAt)
            .limit(limit)
            .offset(offset)
            .all(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))
    }
}
