//! Integration Tests - Real Database CRUD
//!
//! By default uses SQLite in-memory (real database, NOT a mock).
//! Set DATABASE_URL=postgres://user:pass@host:port/db to test against PostgreSQL.
//!
//! Run: cargo test --test integration

use vortex_core::graph_repo::GraphRepository;
use vortex_core::run_repo::RunRepository;
use vortex_core::tenant_repo::TenantRepository;
use vortex_core::entities::{run, graph, tenant};
use vortex_core::db::Database;
use std::sync::Arc;

/// Connect to a real database for testing.
///
/// Strategy:
/// 1. If DATABASE_URL is set → connect to PostgreSQL (CI / port-forward)
/// 2. Otherwise → SQLite in-memory (always works, real DB, not a mock)
///
/// In both cases, `db.init(true)` creates all tables via SeaORM schema sync.
async fn get_db() -> Arc<Database> {
    let db = if let Ok(url) = std::env::var("DATABASE_URL") {
        // Explicit PostgreSQL connection requested
        eprintln!("[integration] Connecting to PostgreSQL: {}", url);

        // Set env vars that VortexConfig expects
        let parts: Vec<&str> = url.split(&['/', ':', '@'][..]).collect();
        if parts.len() >= 8 {
            std::env::set_var("POSTGRES_USER", parts[3]);
            std::env::set_var("POSTGRES_PASSWORD", parts[4]);
            std::env::set_var("POSTGRES_HOST", parts[5]);
            std::env::set_var("POSTGRES_PORT", parts[6]);
            std::env::set_var("POSTGRES_DB", parts[7]);
        }

        let config = vortex_config::VortexConfig::from_env()
            .expect("Failed to load config from env");
        Database::connect(&config).await
            .expect("Failed to connect to PostgreSQL")
    } else {
        // Default: SQLite in-memory — real DB, always works
        eprintln!("[integration] Using SQLite in-memory (set DATABASE_URL for PostgreSQL)");
        Database::connect_sqlite().await
            .expect("Failed to create SQLite database")
    };

    // Create all tables (idempotent: IF NOT EXISTS)
    db.init(true).await.expect("Failed to initialize schema");

    Arc::new(db)
}

#[tokio::test]
async fn test_tenant_repo_crud() {
    let db = get_db().await;
    let repo = TenantRepository::new(db.clone());

    // Create tenant
    let tenant_id = format!("test-tenant-{}", uuid::Uuid::new_v4());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let model = tenant::Model {
        id: tenant_id.clone(),
        name: "Test Tenant".to_string(),
        slug: format!("test-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        tier: tenant::TenantTier::Free,
        status: tenant::TenantStatus::Active,
        max_concurrent_jobs: 10,
        max_gpu_hours_month: 100,
        max_graphs: 50,
        max_models: 20,
        max_members: 5,
        max_storage_bytes: 1024 * 1024 * 1024,
        created_at: now,
        updated_at: now,
    };

    // Insert
    repo.insert(model.clone()).await.expect("Failed to insert tenant");

    // Read by ID
    let fetched = repo.get_by_id(&tenant_id).await.expect("Failed to get tenant");
    assert!(fetched.is_some(), "Tenant should exist after insert");
    assert_eq!(fetched.unwrap().name, "Test Tenant");

    eprintln!("[integration] Tenant CRUD: PASSED");
}

#[tokio::test]
async fn test_graph_repo_crud() {
    let db = get_db().await;
    let repo = GraphRepository::new(db.clone());

    let graph_id = format!("test-graph-{}", uuid::Uuid::new_v4());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let model = graph::Model {
        id: graph_id.clone(),
        tenant_id: "test-tenant".to_string(),
        name: "Test Graph".to_string(),
        version: 1,
        graph_json: "{}".to_string(),
        created_at: now,
        updated_at: now,
    };

    repo.insert(model.clone()).await.expect("Failed to insert graph");

    let fetched = repo.get_by_id(&graph_id).await.expect("Failed to get graph");
    assert!(fetched.is_some(), "Graph should exist after insert");
    assert_eq!(fetched.unwrap().name, "Test Graph");

    eprintln!("[integration] Graph CRUD: PASSED");
}

#[tokio::test]
async fn test_run_repo_crud() {
    let db = get_db().await;
    let repo = RunRepository::new(db.clone());

    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let model = run::Model {
        id: run_id.clone(),
        graph_hash: "test-graph".to_string(),
        status: run::RunStatus::Pending,
        created_at: now,
        completed_at: None,
        error_json: None,
    };

    repo.insert(model.clone()).await.expect("Failed to insert run");

    let fetched = repo.get_by_id(&run_id).await.expect("Failed to get run");
    assert!(fetched.is_some(), "Run should exist after insert");
    assert_eq!(fetched.unwrap().status, run::RunStatus::Pending);

    eprintln!("[integration] Run CRUD: PASSED");
}
