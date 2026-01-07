//! Run Repository - PostgreSQL Storage for Execution Runs
//!
//! Tracks graph execution runs with status, progress, and metrics.
//! Uses PostgreSQL for production persistence.

use std::future::Future;
use std::pin::Pin;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::error::{VortexError, VortexResult};
use crate::db::RunStatus;

// ═══════════════════════════════════════════════════════════════
//                    TYPES
// ═══════════════════════════════════════════════════════════════

/// Stored run record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRun {
    pub id: String,
    pub graph_id: String,
    pub tenant_id: String,
    pub status: RunStatus,
    pub progress: f32,
    pub current_node: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

/// Run repository trait
pub trait RunRepository: Send + Sync {
    fn insert(&self, run: &StoredRun)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn get_by_id(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<Option<StoredRun>>> + Send + '_>>;

    fn update_status(&self, id: &str, status: RunStatus, progress: f32, current_node: Option<&str>)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn complete(&self, id: &str, success: bool, error: Option<&str>)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>;

    fn list_by_graph(&self, graph_id: &str, limit: i64)
        -> Pin<Box<dyn Future<Output = VortexResult<Vec<StoredRun>>> + Send + '_>>;
}

// ═══════════════════════════════════════════════════════════════
//                    POSTGRESQL IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════

/// PostgreSQL-backed run repository
pub struct PgRunRepository {
    pool: PgPool,
}

impl PgRunRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize schema
    pub async fn init_schema(&self) -> VortexResult<()> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS runs (
                id TEXT PRIMARY KEY NOT NULL,
                graph_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('PENDING', 'RUNNING', 'COMPLETED', 'FAILED')),
                progress REAL NOT NULL DEFAULT 0,
                current_node TEXT,
                error TEXT,
                created_at BIGINT NOT NULL,
                started_at BIGINT,
                completed_at BIGINT
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| VortexError::Internal(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_runs_graph ON runs(graph_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

        tracing::info!("Run schema initialized");
        Ok(())
    }

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time")
            .as_secs() as i64
    }
}

impl RunRepository for PgRunRepository {
    fn insert(&self, run: &StoredRun)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let run = run.clone();
        Box::pin(async move {
            sqlx::query(r#"
                INSERT INTO runs (id, graph_id, tenant_id, status, progress, current_node, error, created_at, started_at, completed_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#)
            .bind(&run.id)
            .bind(&run.graph_id)
            .bind(&run.tenant_id)
            .bind(run.status.to_string())
            .bind(run.progress)
            .bind(&run.current_node)
            .bind(&run.error)
            .bind(run.created_at)
            .bind(run.started_at)
            .bind(run.completed_at)
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::info!(run_id = %run.id, "Run inserted");
            Ok(())
        })
    }

    fn get_by_id(&self, id: &str)
        -> Pin<Box<dyn Future<Output = VortexResult<Option<StoredRun>>> + Send + '_>>
    {
        let id = id.to_string();
        Box::pin(async move {
            let row = sqlx::query_as::<_, RunRow>("SELECT * FROM runs WHERE id = $1")
                .bind(&id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| VortexError::Internal(e.to_string()))?;

            Ok(row.map(|r| r.into_stored()))
        })
    }

    fn update_status(&self, id: &str, status: RunStatus, progress: f32, current_node: Option<&str>)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let id = id.to_string();
        let status_str = status.to_string();
        let current_node = current_node.map(|s| s.to_string());
        let now = Self::now();

        Box::pin(async move {
            sqlx::query(r#"
                UPDATE runs SET status = $1, progress = $2, current_node = $3, started_at = COALESCE(started_at, $4)
                WHERE id = $5
            "#)
            .bind(&status_str)
            .bind(progress)
            .bind(&current_node)
            .bind(now)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::debug!(run_id = %id, status = %status_str, "Run status updated");
            Ok(())
        })
    }

    fn complete(&self, id: &str, success: bool, error: Option<&str>)
        -> Pin<Box<dyn Future<Output = VortexResult<()>> + Send + '_>>
    {
        let id = id.to_string();
        let status = if success { "COMPLETED" } else { "FAILED" };
        let error = error.map(|s| s.to_string());
        let now = Self::now();

        Box::pin(async move {
            sqlx::query(r#"
                UPDATE runs SET status = $1, progress = 1.0, error = $2, completed_at = $3
                WHERE id = $4
            "#)
            .bind(status)
            .bind(&error)
            .bind(now)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| VortexError::Internal(e.to_string()))?;

            tracing::info!(run_id = %id, success = success, "Run completed");
            Ok(())
        })
    }

    fn list_by_graph(&self, graph_id: &str, limit: i64)
        -> Pin<Box<dyn Future<Output = VortexResult<Vec<StoredRun>>> + Send + '_>>
    {
        let graph_id = graph_id.to_string();
        Box::pin(async move {
            let rows = sqlx::query_as::<_, RunRow>(
                "SELECT * FROM runs WHERE graph_id = $1 ORDER BY created_at DESC LIMIT $2"
            )
            .bind(&graph_id)
            .bind(limit)
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
struct RunRow {
    id: String,
    graph_id: String,
    tenant_id: String,
    status: String,
    progress: f32,
    current_node: Option<String>,
    error: Option<String>,
    created_at: i64,
    started_at: Option<i64>,
    completed_at: Option<i64>,
}

impl RunRow {
    fn into_stored(self) -> StoredRun {
        StoredRun {
            id: self.id,
            graph_id: self.graph_id,
            tenant_id: self.tenant_id,
            status: match self.status.as_str() {
                "RUNNING" => RunStatus::Running,
                "COMPLETED" => RunStatus::Completed,
                "FAILED" => RunStatus::Failed,
                _ => RunStatus::Pending,
            },
            progress: self.progress,
            current_node: self.current_node,
            error: self.error,
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
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
    fn test_stored_run_serialization() {
        let run = StoredRun {
            id: "r1".to_string(),
            graph_id: "g1".to_string(),
            tenant_id: "t1".to_string(),
            status: RunStatus::Running,
            progress: 0.5,
            current_node: Some("node1".to_string()),
            error: None,
            created_at: 12345,
            started_at: Some(12346),
            completed_at: None,
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("Running"));  // Serde uses PascalCase
    }
}
