//! Integration Tests - Real PostgreSQL
//!
//! These tests require PostgreSQL running (via port-forward).
//! Run with: DATABASE_URL=postgres://vortex:dev@localhost:5432/vortex cargo test --test integration

use vortex_core::graph_repo::{PgGraphRepository, StoredGraph, GraphRepository};
use vortex_core::tenant_repo::PgTenantRepository;
use vortex_core::run_repo::{PgRunRepository, StoredRun, RunRepository};
use vortex_core::tenant::{Tenant, TenantTier, TenantStatus, TenantQuota, TenantRepository};
use vortex_core::db::RunStatus;
use sqlx::types::chrono::Utc;

async fn get_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://vortex:dev@localhost:5432/vortex".to_string());
    
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to PostgreSQL")
}

#[tokio::test]
async fn test_tenant_repo_crud() {
    let pool = get_pool().await;
    let repo = PgTenantRepository::new(pool);
    
    // Initialize schema
    repo.init_schema().await.expect("Failed to init schema");
    
    // Create tenant
    let tenant = Tenant {
        id: format!("test-tenant-{}", uuid::Uuid::new_v4()),
        name: "Test Tenant".to_string(),
        slug: format!("test-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
        tier: TenantTier::Free,
        status: TenantStatus::Active,
        quota: TenantQuota::for_tier(TenantTier::Free),
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    
    // Insert
    repo.insert(&tenant).await.expect("Failed to insert tenant");
    
    // Read by ID
    let fetched = repo.get_by_id(&tenant.id).await.expect("Failed to get tenant");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Test Tenant");
    
    // Read by slug
    let by_slug = repo.get_by_slug(&tenant.slug).await.expect("Failed to get by slug");
    assert!(by_slug.is_some());
    
    println!("✅ Tenant CRUD test passed!");
}

#[tokio::test]
async fn test_graph_repo_crud() {
    let pool = get_pool().await;
    let repo = PgGraphRepository::new(pool);
    
    // Initialize schema
    repo.init_schema().await.expect("Failed to init schema");
    
    // Create graph
    let graph = StoredGraph {
        id: format!("test-graph-{}", uuid::Uuid::new_v4()),
        tenant_id: "test-tenant".to_string(),
        name: "Test Graph".to_string(),
        version: 1,
        graph_json: serde_json::json!({
            "nodes": [
                {"id": "n1", "type": "Loader::Checkpoint"},
                {"id": "n2", "type": "Sampler::KSampler"}
            ],
            "edges": [
                {"from": "n1", "to": "n2"}
            ]
        }),
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    };
    
    // Insert
    repo.insert(&graph).await.expect("Failed to insert graph");
    
    // Read
    let fetched = repo.get_by_id(&graph.id).await.expect("Failed to get graph");
    assert!(fetched.is_some());
    let g = fetched.unwrap();
    assert_eq!(g.name, "Test Graph");
    assert_eq!(g.version, 1);
    
    println!("✅ Graph CRUD test passed!");
}

#[tokio::test]
async fn test_run_repo_crud() {
    let pool = get_pool().await;
    let repo = PgRunRepository::new(pool);
    
    // Initialize schema
    repo.init_schema().await.expect("Failed to init schema");
    
    // Create run
    let run = StoredRun {
        id: format!("test-run-{}", uuid::Uuid::new_v4()),
        graph_id: "test-graph".to_string(),
        tenant_id: "test-tenant".to_string(),
        status: RunStatus::Pending,
        progress: 0.0,
        current_node: None,
        error: None,
        created_at: Utc::now().timestamp(),
        started_at: None,
        completed_at: None,
    };
    
    // Insert
    repo.insert(&run).await.expect("Failed to insert run");
    
    // Read
    let fetched = repo.get_by_id(&run.id).await.expect("Failed to get run");
    assert!(fetched.is_some());
    
    // Update status
    repo.update_status(&run.id, RunStatus::Running, 0.5, Some("node1"))
        .await.expect("Failed to update status");
    
    // Complete
    repo.complete(&run.id, true, None).await.expect("Failed to complete run");
    
    // Verify completed
    let completed = repo.get_by_id(&run.id).await.expect("Failed to get").unwrap();
    assert_eq!(completed.status, RunStatus::Completed);
    
    println!("✅ Run CRUD test passed!");
}
