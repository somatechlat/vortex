//! Graph Repository - PostgreSQL Storage for Graphs
//!
//! Implements graph persistence with PostgreSQL via SQLx.
//! Uses JSON storage for graph DSL flexibility.

use std::future::Future;
use std::pin::Pin;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::error::{VortexError, VortexResult};

// ═══════════════════════════════════════════════════════════════
//                    TYPES
// ═══════════════════════════════════════════════════════════════

/// Stored graph record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGraph {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub version: i64,
    pub graph_json: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Graph repository trait for storage abstraction
pub trait GraphRepository: Send + Sync {
    fn insert(&self, graph: &StoredGraph)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn get_by_id(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<Option<StoredGraph>>> + Send + '_>>;

    fn update(&self, graph: &StoredGraph)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn delete(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn list_by_tenant(&self, tenant_id: &str, limit: i64, offset: i64)
        -> Pin<Box<dyn Future<Output = VortexResult<Vec<StoredGraph>>> + Send + '_>>;
}

// ═══════════════════════════════════════════════════════════════
//                    POSTGRESQL IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════

/// PostgreSQL-backed graph repository
pub struct PgGraphRepository {
    pool: PgPool,
}

impl PgGraphRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize schema
    pub async fn init_schema(&self) -> VortexResult<()> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS graphs (
                id TEXT PRIMARY KEY NOT NULL,
                tenant_id TEXT NOT NULL,
                name TEXT NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                graph_json JSONB NOT NULL,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| VortexError::Internal(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_graphs_tenant ON graphs(tenant_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        tracing::info!("Graph schema initialized");
        Ok(())
    }
}

impl GraphRepository for PgGraphRepository {
    fn insert(&self, graph: &StoredGraph)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let graph = graph.clone();
        Box::pin(async move {
            sqlx::query(r#"
                INSERT INTO graphs (id, tenant_id, name, version, graph_json, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#)
            .bind(&graph.id)
            .bind(&graph.tenant_id)
            .bind(&graph.name)
            .bind(graph.version)
            .bind(&graph.graph_json)
            .bind(graph.created_at)
            .bind(graph.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::info!(graph_id = %graph.id, "Graph inserted");
            Ok(())
        })
    }

    fn get_by_id(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<Option<StoredGraph>>> + Send + '_>>
    {
        let id = id.to_string();
        Box::pin(async move {
            let row = sqlx::query_as::<_, GraphRow>(
                "SELECT * FROM graphs WHERE id = $1"
            )
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            Ok(row.map(|r| r.into_stored()))
        })
    }

    fn update(&self, graph: &StoredGraph)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let graph = graph.clone();
        Box::pin(async move {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time")
                .as_secs() as i64;

            sqlx::query(r#"
                UPDATE graphs SET
                    name = $1, version = version + 1, graph_json = $2, updated_at = $3
                WHERE id = $4
            "#)
            .bind(&graph.name)
            .bind(&graph.graph_json)
            .bind(now)
            .bind(&graph.id)
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::info!(graph_id = %graph.id, "Graph updated");
            Ok(())
        })
    }

    fn delete(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let id = id.to_string();
        Box::pin(async move {
            sqlx::query("DELETE FROM graphs WHERE id = $1")
                .bind(&id)
                .execute(&self.pool)
                .await
                .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::info!(graph_id = %id, "Graph deleted");
            Ok(())
        })
    }

    fn list_by_tenant(&self, tenant_id: &str, limit: i64, offset: i64)
        -> Pin<Box<dyn Future<Output = VortexResult<Vec<StoredGraph>>> + Send + '_>>
    {
        let tenant_id = tenant_id.to_string();
        Box::pin(async move {
            let rows = sqlx::query_as::<_, GraphRow>(
                "SELECT * FROM graphs WHERE tenant_id = $1 ORDER BY updated_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(&tenant_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            Ok(rows.into_iter().map(|r| r.into_stored()).collect())
        })
    }
}

// ═══════════════════════════════════════════════════════════════
//                    SQLX ROW TYPE
// ═══════════════════════════════════════════════════════════════

#[derive(sqlx::FromRow)]
struct GraphRow {
    id: String,
    tenant_id: String,
    name: String,
    version: i64,
    graph_json: serde_json::Value,
    created_at: i64,
    updated_at: i64,
}

impl GraphRow {
    fn into_stored(self) -> StoredGraph {
        StoredGraph {
            id: self.id,
            tenant_id: self.tenant_id,
            name: self.name,
            version: self.version,
            graph_json: self.graph_json,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_graph_serialization() {
        let graph = StoredGraph {
            id: "g1".to_string(),
            tenant_id: "t1".to_string(),
            name: "Test Graph".to_string(),
            version: 1,
            graph_json: serde_json::json!({"nodes": []}),
            created_at: 12345,
            updated_at: 12345,
        };
        let json = serde_json::to_string(&graph).unwrap();
        assert!(json.contains("Test Graph"));
    }
}
