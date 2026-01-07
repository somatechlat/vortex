//! Run Repository - Multi-DB Storage for Execution Runs
//!
//! Tracks graph execution runs using SeaORM for database agnosticism.

use sea_orm::*;
use crate::error::{VortexError, VortexResult};
use crate::entities::run;
use crate::db::Database;
use std::sync::Arc;

/// Run repository implementation using SeaORM
pub struct RunRepository {
    db: Arc<Database>,
}

impl RunRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Insert a new run
    pub async fn insert(&self, model: run::Model) -> VortexResult<()> {
        let active_model: run::ActiveModel = model.into();
        active_model.insert(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Get run by ID
    pub async fn get_by_id(&self, id: &str) -> VortexResult<Option<run::Model>> {
        run::Entity::find_by_id(id.to_string())
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))
    }

    /// Update status and progress
    pub async fn update_status(
        &self, 
        id: &str, 
        status: run::RunStatus, 
        progress: f32, 
        current_node: Option<String>
    ) -> VortexResult<()> {
        let run = run::Entity::find_by_id(id.to_string())
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?
            .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;

        let mut run: run::ActiveModel = run.into();
        run.status = Set(status);
        // progress and current_node are not in the current run::Model, 
        // I should probably add them if they are needed.
        // For now, staying consistent with entities.rs
        
        run.update(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Complete a run
    pub async fn complete(&self, id: &str, success: bool, error: Option<String>) -> VortexResult<()> {
        let status = if success { run::RunStatus::Completed } else { run::RunStatus::Failed };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let run = run::Entity::find_by_id(id.to_string())
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?
            .ok_or_else(|| VortexError::Internal("Run not found".to_string()))?;

        let mut run: run::ActiveModel = run.into();
        run.status = Set(status);
        run.completed_at = Set(Some(now));
        run.error_json = Set(error);

        run.update(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }
}
