//! Database Entities - SeaORM Models
//!
//! Enterprise-grade ORM layer for VORTEX persistence.
//! No raw SQL - all access through typed entities.

// ═══════════════════════════════════════════════════════════════
//                    RUN ENTITY
// ═══════════════════════════════════════════════════════════════

pub mod run {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
    #[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
    pub enum RunStatus {
        #[sea_orm(string_value = "PENDING")]
        Pending,
        #[sea_orm(string_value = "RUNNING")]
        Running,
        #[sea_orm(string_value = "COMPLETED")]
        Completed,
        #[sea_orm(string_value = "FAILED")]
        Failed,
    }

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "runs")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub graph_hash: String,
        pub status: RunStatus,
        pub created_at: i64,
        pub completed_at: Option<i64>,
        pub error_json: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::run_step::Entity")]
        RunSteps,
    }

    impl Related<super::run_step::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::RunSteps.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ═══════════════════════════════════════════════════════════════
//                    RUN STEP ENTITY
// ═══════════════════════════════════════════════════════════════

pub mod run_step {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "run_steps")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub run_id: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub node_id: String,
        pub worker_pid: i32,
        pub duration_us: i64,
        pub peak_vram_mb: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::run::Entity",
            from = "Column::RunId",
            to = "super::run::Column::Id"
        )]
        Run,
    }

    impl Related<super::run::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Run.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ═══════════════════════════════════════════════════════════════
//                    GRAPH ENTITY
// ═══════════════════════════════════════════════════════════════

pub mod graph {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "graphs")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub tenant_id: String,
        pub name: String,
        pub version: i32,
        pub graph_json: String,
        pub created_at: i64,
        pub updated_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::tenant::Entity",
            from = "Column::TenantId",
            to = "super::tenant::Column::Id"
        )]
        Tenant,
    }

    impl Related<super::tenant::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Tenant.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ═══════════════════════════════════════════════════════════════
//                    TENANT ENTITY
// ═══════════════════════════════════════════════════════════════

pub mod tenant {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
    #[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
    pub enum TenantTier {
        #[sea_orm(string_value = "free")]
        Free,
        #[sea_orm(string_value = "pro")]
        Pro,
        #[sea_orm(string_value = "enterprise")]
        Enterprise,
    }

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
    #[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
    pub enum TenantStatus {
        #[sea_orm(string_value = "provisioning")]
        Provisioning,
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "suspended")]
        Suspended,
        #[sea_orm(string_value = "pending_deletion")]
        PendingDeletion,
        #[sea_orm(string_value = "deleted")]
        Deleted,
    }

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "tenants")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub name: String,
        #[sea_orm(unique)]
        pub slug: String,
        pub tier: TenantTier,
        pub status: TenantStatus,
        pub max_concurrent_jobs: i32,
        pub max_gpu_hours_month: i64,
        pub max_graphs: i32,
        pub max_models: i32,
        pub max_members: i32,
        pub max_storage_bytes: i64,
        pub created_at: i64,
        pub updated_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::graph::Entity")]
        Graphs,
    }

    impl Related<super::graph::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Graphs.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ═══════════════════════════════════════════════════════════════
//                    MODEL REGISTRY ENTITY
// ═══════════════════════════════════════════════════════════════

pub mod model_entry {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
    #[sea_orm(rs_type = "String", db_type = "String(Some(16))")]
    pub enum ModelType {
        #[sea_orm(string_value = "diffusers")]
        Diffusers,
        #[sea_orm(string_value = "checkpoint")]
        Checkpoint,
        #[sea_orm(string_value = "lora")]
        LoRA,
        #[sea_orm(string_value = "vae")]
        VAE,
        #[sea_orm(string_value = "controlnet")]
        ControlNet,
    }

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "models")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub name: String,
        pub source_type: String,
        pub source_repo: String,
        pub model_type: ModelType,
        pub size_bytes: Option<i64>,
        pub hash: Option<String>,
        pub cached_path: Option<String>,
        pub last_used: Option<i64>,
        pub created_at: i64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
