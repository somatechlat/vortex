//! Database - SQLite persistence for execution history
//!
//! Implements SRS Section 3.4.2 (Database Schema)

use crate::error::VortexResult;
use serde::{Deserialize, Serialize};

/// Run status enum matching SRS schema CHECK constraint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Pending => write!(f, "PENDING"),
            RunStatus::Running => write!(f, "RUNNING"),
            RunStatus::Completed => write!(f, "COMPLETED"),
            RunStatus::Failed => write!(f, "FAILED"),
        }
    }
}

/// Execution run record (SRS Table: runs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub graph_hash: String,
    pub status: RunStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub error_json: Option<String>,
}

/// Step metrics record (SRS Table: run_steps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStep {
    pub run_id: String,
    pub node_id: String,
    pub worker_pid: i32,
    pub duration_us: i64,
    pub peak_vram_mb: i64,
}

/// Database connection wrapper
pub struct Database {
    path: String,
    #[cfg(feature = "sqlite")]
    pool: Option<sqlx::SqlitePool>,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            #[cfg(feature = "sqlite")]
            pool: None,
        }
    }
    
    /// Initialize the database with schema (SRS Section 3.4.2)
    pub async fn init(&mut self) -> VortexResult<()> {
        #[cfg(feature = "sqlite")]
        {
            use sqlx::sqlite::SqlitePoolOptions;
            
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect(&format!("sqlite:{}", self.path))
                .await?;
            
            // Create tables (SRS Section 3.4.2)
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS runs (
                    id TEXT PRIMARY KEY NOT NULL,
                    graph_hash TEXT NOT NULL,
                    status TEXT CHECK(status IN ('PENDING', 'RUNNING', 'COMPLETED', 'FAILED')),
                    created_at INTEGER NOT NULL,
                    completed_at INTEGER,
                    error_json TEXT
                )
            "#)
            .execute(&pool)
            .await?;
            
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS run_steps (
                    run_id TEXT NOT NULL REFERENCES runs(id),
                    node_id TEXT NOT NULL,
                    worker_pid INTEGER NOT NULL,
                    duration_us INTEGER NOT NULL,
                    peak_vram_mb INTEGER NOT NULL,
                    PRIMARY KEY (run_id, node_id)
                )
            "#)
            .execute(&pool)
            .await?;
            
            self.pool = Some(pool);
        }
        
        Ok(())
    }
    
    /// Insert a new run
    #[cfg(feature = "sqlite")]
    pub async fn insert_run(&self, run: &Run) -> VortexResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            crate::error::VortexError::Internal("Database not initialized".to_string())
        })?;
        
        sqlx::query(r#"
            INSERT INTO runs (id, graph_hash, status, created_at, completed_at, error_json)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&run.id)
        .bind(&run.graph_hash)
        .bind(run.status.to_string())
        .bind(run.created_at)
        .bind(run.completed_at)
        .bind(&run.error_json)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Update run status
    #[cfg(feature = "sqlite")]
    pub async fn update_run_status(
        &self,
        run_id: &str,
        status: RunStatus,
        completed_at: Option<i64>,
        error_json: Option<String>,
    ) -> VortexResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            crate::error::VortexError::Internal("Database not initialized".to_string())
        })?;
        
        sqlx::query(r#"
            UPDATE runs SET status = ?, completed_at = ?, error_json = ? WHERE id = ?
        "#)
        .bind(status.to_string())
        .bind(completed_at)
        .bind(error_json)
        .bind(run_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Insert a step metric
    #[cfg(feature = "sqlite")]
    pub async fn insert_step(&self, step: &RunStep) -> VortexResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            crate::error::VortexError::Internal("Database not initialized".to_string())
        })?;
        
        sqlx::query(r#"
            INSERT INTO run_steps (run_id, node_id, worker_pid, duration_us, peak_vram_mb)
            VALUES (?, ?, ?, ?, ?)
        "#)
        .bind(&step.run_id)
        .bind(&step.node_id)
        .bind(step.worker_pid)
        .bind(step.duration_us)
        .bind(step.peak_vram_mb)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Get a run by ID
    #[cfg(feature = "sqlite")]
    pub async fn get_run(&self, run_id: &str) -> VortexResult<Option<Run>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            crate::error::VortexError::Internal("Database not initialized".to_string())
        })?;
        
        let row = sqlx::query_as::<_, (String, String, String, i64, Option<i64>, Option<String>)>(
            "SELECT id, graph_hash, status, created_at, completed_at, error_json FROM runs WHERE id = ?"
        )
        .bind(run_id)
        .fetch_optional(pool)
        .await?;
        
        Ok(row.map(|(id, graph_hash, status, created_at, completed_at, error_json)| {
            Run {
                id,
                graph_hash,
                status: match status.as_str() {
                    "PENDING" => RunStatus::Pending,
                    "RUNNING" => RunStatus::Running,
                    "COMPLETED" => RunStatus::Completed,
                    _ => RunStatus::Failed,
                },
                created_at,
                completed_at,
                error_json,
            }
        }))
    }
    
    // Stub implementations for non-sqlite builds
    #[cfg(not(feature = "sqlite"))]
    pub async fn insert_run(&self, _run: &Run) -> VortexResult<()> { Ok(()) }
    #[cfg(not(feature = "sqlite"))]
    pub async fn update_run_status(&self, _: &str, _: RunStatus, _: Option<i64>, _: Option<String>) -> VortexResult<()> { Ok(()) }
    #[cfg(not(feature = "sqlite"))]
    pub async fn insert_step(&self, _step: &RunStep) -> VortexResult<()> { Ok(()) }
    #[cfg(not(feature = "sqlite"))]
    pub async fn get_run(&self, _run_id: &str) -> VortexResult<Option<Run>> { Ok(None) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(RunStatus::Pending.to_string(), "PENDING");
        assert_eq!(RunStatus::Running.to_string(), "RUNNING");
        assert_eq!(RunStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(RunStatus::Failed.to_string(), "FAILED");
    }
}
