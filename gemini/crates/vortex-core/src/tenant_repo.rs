//! Tenant Repository - Multi-DB Storage for Tenants
//!
//! Implements Tenant persistence using SeaORM for database agnosticism.

use sea_orm::*;
use crate::error::{VortexError, VortexResult};
use crate::entities::tenant;
use crate::db::Database;
use std::sync::Arc;

/// Tenant repository implementation using SeaORM
pub struct TenantRepository {
    db: Arc<Database>,
}

impl TenantRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Insert a new tenant
    pub async fn insert(&self, model: tenant::Model) -> VortexResult<()> {
        let active_model: tenant::ActiveModel = model.into();
        active_model.insert(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Get tenant by ID
    pub async fn get_by_id(&self, id: &str) -> VortexResult<Option<tenant::Model>> {
        tenant::Entity::find_by_id(id.to_string())
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))
    }

    /// Get tenant by slug
    pub async fn get_by_slug(&self, slug: &str) -> VortexResult<Option<tenant::Model>> {
        tenant::Entity::find()
            .filter(tenant::Column::Slug.eq(slug))
            .one(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))
    }

    /// Update a tenant
    pub async fn update(&self, model: tenant::Model) -> VortexResult<()> {
        let active_model: tenant::ActiveModel = model.into();
        active_model.update(self.db.connection())
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;
        Ok(())
    }
}
