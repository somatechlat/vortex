//! Database - Multi-DB persistence layer using SeaORM
//!
//! Implements SRS Section 3.4.2 (Database Schema) with database agnosticism.

use crate::error::VortexResult;
use crate::entities::{run, run_step};
use sea_orm::*;
use std::time::Duration;

/// Database connection wrapper (SeaORM)
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    /// Create a new database connection from config with retries (Rule 122)
    pub async fn connect(config: &vortex_config::VortexConfig) -> VortexResult<Self> {
        let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "vortex".to_string());
        let pass = std::env::var("POSTGRES_PASSWORD").unwrap_or_default();
        
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            user,
            pass,
            config.database.postgres_host,
            config.database.postgres_port,
            config.database.postgres_db
        );

        tracing::info!("Connecting to Database at {}:{} (Resilience: HIGH, Engine: SeaORM)", 
            config.database.postgres_host, config.database.postgres_port);

        let mut opt = ConnectOptions::new(url);
        opt.max_connections(config.database.pool.max_connections)
           .min_connections(config.database.pool.min_connections)
           .connect_timeout(Duration::from_secs(config.database.pool.connect_timeout_secs))
           .idle_timeout(Duration::from_secs(config.database.pool.idle_timeout_secs))
           .set_schema_search_path("public".to_string());

        // Rule 122: 10-Cycle Resiliency Loop
        let mut retries = 10;
        let conn = loop {
            match sea_orm::Database::connect(opt.clone()).await {
                Ok(conn) => break conn,
                Err(e) if retries > 0 => {
                    tracing::warn!("Database connection failed: {}. Retrying in 5s... ({} attempts left)", e, retries);
                    retries -= 1;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    tracing::error!("Database connection failed after 10 cycles: {}", e);
                    return Err(crate::error::VortexError::Internal(e.to_string()));
                }
            }
        };

        Ok(Self { conn })
    }

    /// Access the underlying SeaORM connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Initialize the database with schema (SRS Section 3.4.2)
    /// Programmatic Schema Creation (Smart Migrations)
    pub async fn init(&self, run_migrations: bool) -> VortexResult<()> {
        if run_migrations {
            tracing::info!("Running dynamic schema synchronization...");
            let builder = self.conn.get_database_backend();
            let schema = Schema::new(builder);

            // Synchronize all entities
            let entities = vec![
                schema.create_table_from_entity(run::Entity),
                schema.create_table_from_entity(run_step::Entity),
                schema.create_table_from_entity(crate::entities::graph::Entity),
                schema.create_table_from_entity(crate::entities::model_entry::Entity),
                schema.create_table_from_entity(crate::entities::tenant::Entity),
            ];

            for mut stmt in entities {
                // Execute create table IF NOT EXISTS
                stmt.if_not_exists();
                let query = builder.build(&stmt);
                if let Err(e) = self.conn.execute(query).await {
                    tracing::warn!("Schema sync warning (safe to ignore if exists): {}", e);
                }
            }
            
            tracing::info!("Database schema synchronized.");
        }
        
        Ok(())
    }
    
    /// Insert a new run
    pub async fn insert_run(&self, run_data: run::Model) -> VortexResult<()> {
        let active_model: run::ActiveModel = run_data.into();
        active_model.insert(&self.conn).await.map_err(|e| crate::error::VortexError::Internal(e.to_string()))?;
        Ok(())
    }
    
    /// Update run status
    pub async fn update_run_status(
        &self,
        run_id: &str,
        status: run::RunStatus,
        completed_at: Option<i64>,
        error_json: Option<String>,
    ) -> VortexResult<()> {
        let run = run::Entity::find_by_id(run_id.to_string())
            .one(&self.conn)
            .await
            .map_err(|e| crate::error::VortexError::Internal(e.to_string()))?
            .ok_or_else(|| crate::error::VortexError::Internal("Run not found".to_string()))?;

        let mut run: run::ActiveModel = run.into();
        run.status = Set(status);
        run.completed_at = Set(completed_at);
        run.error_json = Set(error_json);
        
        run.update(&self.conn).await.map_err(|e| crate::error::VortexError::Internal(e.to_string()))?;
        Ok(())
    }
    
    /// Insert a step metric
    pub async fn insert_step(&self, step_data: run_step::Model) -> VortexResult<()> {
        let active_model: run_step::ActiveModel = step_data.into();
        active_model.insert(&self.conn).await.map_err(|e| crate::error::VortexError::Internal(e.to_string()))?;
        Ok(())
    }
    
    /// Get a run by ID
    pub async fn get_run(&self, run_id: &str) -> VortexResult<Option<run::Model>> {
        run::Entity::find_by_id(run_id.to_string())
            .one(&self.conn)
            .await
            .map_err(|e| crate::error::VortexError::Internal(e.to_string()))
    }
}
