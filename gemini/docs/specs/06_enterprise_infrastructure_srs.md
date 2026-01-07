# VORTEX Enterprise Infrastructure SRS

> **Document ID**: VORTEX-SRS-INFRA-001  
> **Version**: 1.0.0  
> **Status**: Approved  
> **Standard**: ISO/IEC 29148:2018

---

## 1. Introduction

### 1.1 Purpose
This document specifies the enterprise infrastructure requirements for VORTEX-GEN 3.0, covering security, authentication, authorization, database, and observability components.

### 1.2 Scope
All infrastructure components deployed via Kubernetes (Tilt + Minikube for development).

### 1.3 Definitions

| Term | Definition |
|------|------------|
| **AuthN** | Authentication - Verifying identity |
| **AuthZ** | Authorization - Verifying permissions |
| **ReBAC** | Relationship-Based Access Control |
| **ORM** | Object-Relational Mapping |
| **OIDC** | OpenID Connect |

---

## 2. Port Authority

All services follow the VORTEX Port Authority specification (11000+ range).

| Port | Service | Protocol | Description |
|------|---------|----------|-------------|
| 11100 | UI | HTTP | Svelte Frontend |
| 11188 | Core API | HTTP | REST API |
| 11189 | Core WS | WS | WebSocket |
| 11191 | Core Metrics | HTTP | Prometheus |
| 11200 | Vault | HTTP | Secrets API |
| 11201 | Keycloak | HTTP | OIDC Admin |
| 11202 | PostgreSQL | TCP | Database |
| 11203 | Milvus gRPC | gRPC | Vector DB |
| 11204 | Milvus Metrics | HTTP | Prometheus |
| 11205 | SpiceDB gRPC | gRPC | Authorization |
| 11206 | SpiceDB HTTP | HTTP | Authorization REST |

---

## 3. Security Stack

### 3.1 HashiCorp Vault (REQ-SEC-001)

**Purpose**: Centralized secrets management

#### 3.1.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SEC-001.1 | Store HuggingFace API tokens | MUST |
| REQ-SEC-001.2 | Store database credentials | MUST |
| REQ-SEC-001.3 | Audit all secret access | MUST |
| REQ-SEC-001.4 | Support K8s service account auth | SHOULD |

#### 3.1.2 Secret Paths

```
secret/vortex/
├── huggingface/
│   └── token           # HuggingFace API token
├── civitai/
│   └── token           # CivitAI API key
├── postgres/
│   ├── username
│   └── password
├── spicedb/
│   └── preshared_key
└── internal/
    └── encryption_key  # Fallback encryption
```

#### 3.1.3 Deployment

```yaml
image: hashicorp/vault:1.15
ports:
  - 8200 (HTTP API)
mode: dev (development), HA (production)
root_token: vortex-dev-token (dev only)
```

---

### 3.2 Keycloak (REQ-SEC-002)

**Purpose**: OpenID Connect (OIDC) authentication provider

#### 3.2.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SEC-002.1 | OIDC authorization code flow with PKCE | MUST |
| REQ-SEC-002.2 | JWT token issuance (RS256) | MUST |
| REQ-SEC-002.3 | Realm: vortex | MUST |
| REQ-SEC-002.4 | Client: vortex-api (confidential) | MUST |
| REQ-SEC-002.5 | Client: vortex-ui (public, PKCE) | MUST |
| REQ-SEC-002.6 | Role mapping to SpiceDB | SHOULD |

#### 3.2.2 Realm Configuration

```yaml
realm: vortex
clients:
  - id: vortex-api
    type: confidential
    service_account: true
  - id: vortex-ui
    type: public
    pkce: true
    redirect_uris:
      - http://localhost:11100/*
roles:
  - model-admin    # CRUD models, manage credentials
  - model-user     # Read models, trigger execution
  - graph-admin    # CRUD graphs
  - graph-executor # Execute graphs
```

#### 3.2.3 Deployment

```yaml
image: quay.io/keycloak/keycloak:23.0
ports:
  - 8080 (HTTP)
mode: start-dev (development)
admin: admin / vortex-dev-admin
```

---

### 3.3 SpiceDB (REQ-SEC-003)

**Purpose**: Relationship-Based Access Control (ReBAC) authorization

#### 3.3.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-SEC-003.1 | Google Zanzibar-compatible API | MUST |
| REQ-SEC-003.2 | Graph ownership permissions | MUST |
| REQ-SEC-003.3 | Model access permissions | MUST |
| REQ-SEC-003.4 | Organization hierarchy | SHOULD |
| REQ-SEC-003.5 | Permission check < 10ms | MUST |

#### 3.3.2 Schema (Zanzibar ZED)

```zed
definition user {}

definition organization {
    relation admin: user
    relation member: user
    
    permission view = admin + member
    permission manage = admin
}

definition graph {
    relation owner: user | organization#member
    relation editor: user
    relation viewer: user
    
    permission view = owner + editor + viewer
    permission edit = owner + editor
    permission execute = owner + editor
    permission delete = owner
}

definition model {
    relation owner: user | organization#member
    
    permission load = owner
    permission delete = owner
}
```

#### 3.3.3 Deployment

```yaml
image: authzed/spicedb:v1.29.0
ports:
  - 50051 (gRPC)
  - 8443 (HTTP)
  - 9090 (Metrics)
datastore: memory (dev), postgres (prod)
preshared_key: vortex-dev-key
```

---

## 4. Database Stack

### 4.1 PostgreSQL (REQ-DB-001)

**Purpose**: Primary relational database

#### 4.1.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-DB-001.1 | PostgreSQL 16+ | MUST |
| REQ-DB-001.2 | All access via ORM (SeaORM/SQLx) | MUST |
| REQ-DB-001.3 | No raw SQL in application code | MUST |
| REQ-DB-001.4 | Connection pooling | MUST |
| REQ-DB-001.5 | Automatic migrations | SHOULD |

#### 4.1.2 Schema

```sql
-- Managed by SeaORM migrations, NOT direct SQL
CREATE TABLE runs (
    id VARCHAR(36) PRIMARY KEY,
    graph_hash VARCHAR(64) NOT NULL,
    status VARCHAR(16) CHECK(status IN ('PENDING','RUNNING','COMPLETED','FAILED')),
    created_at BIGINT NOT NULL,
    completed_at BIGINT,
    error_json TEXT
);

CREATE TABLE run_steps (
    run_id VARCHAR(36) REFERENCES runs(id),
    node_id VARCHAR(64) NOT NULL,
    worker_pid INTEGER NOT NULL,
    duration_us BIGINT NOT NULL,
    peak_vram_mb BIGINT NOT NULL,
    PRIMARY KEY (run_id, node_id)
);

CREATE TABLE graphs (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    owner_id VARCHAR(36) NOT NULL,
    version INTEGER DEFAULT 1,
    graph_json TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE TABLE models (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(16) NOT NULL,
    source_repo VARCHAR(512) NOT NULL,
    model_type VARCHAR(16) NOT NULL,
    size_bytes BIGINT,
    hash VARCHAR(64),
    cached_path TEXT,
    last_used BIGINT,
    created_at BIGINT NOT NULL,
    UNIQUE(source_type, source_repo)
);
```

#### 4.1.3 Deployment

```yaml
image: postgres:16-alpine
ports:
  - 5432
database: vortex
user: vortex
storage: 5Gi PVC
```

---

### 4.2 Milvus (REQ-DB-002)

**Purpose**: Vector database for embeddings and similarity search

#### 4.2.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-DB-002.1 | Store CLIP embeddings | MUST |
| REQ-DB-002.2 | ANN similarity search < 100ms | MUST |
| REQ-DB-002.3 | Support 1M+ vectors | SHOULD |
| REQ-DB-002.4 | IVF_FLAT index type | SHOULD |

#### 4.2.2 Collections

```python
# Image embeddings for similarity search
collection = Collection(
    name="image_embeddings",
    schema=CollectionSchema(
        fields=[
            FieldSchema("id", DataType.VARCHAR, max_length=36, is_primary=True),
            FieldSchema("embedding", DataType.FLOAT_VECTOR, dim=768),
            FieldSchema("model_id", DataType.VARCHAR, max_length=36),
            FieldSchema("created_at", DataType.INT64),
        ]
    )
)
```

#### 4.2.3 Deployment

```yaml
image: milvusdb/milvus:v2.3.4
ports:
  - 19530 (gRPC)
  - 9091 (Metrics)
mode: standalone
storage: 10Gi PVC
```

---

## 5. ORM Layer

### 5.1 SeaORM (REQ-ORM-001)

**Purpose**: Enterprise-grade Rust ORM built on SQLx

#### 5.1.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-ORM-001.1 | No raw SQL strings | MUST |
| REQ-ORM-001.2 | Entity-based CRUD | MUST |
| REQ-ORM-001.3 | Async/await support | MUST |
| REQ-ORM-001.4 | Migration management | MUST |
| REQ-ORM-001.5 | PostgreSQL backend | MUST |

#### 5.1.2 Entity Example

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "runs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub graph_hash: String,
    pub status: RunStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

// CRUD Operations
let run = runs::Entity::find_by_id(run_id).one(&db).await?;
let run = runs::ActiveModel { ... }.insert(&db).await?;
```

---

## 6. Resource Limits

### 6.1 Namespace: vortex

```yaml
# ResourceQuota
limits.memory: 10Gi
limits.cpu: 8
requests.storage: 50Gi
pods: 20
```

### 6.2 LimitRange

```yaml
# Per-container defaults
default.memory: 512Mi
default.cpu: 500m
max.memory: 4Gi
max.cpu: 4
```

---

## 7. Tilt Development Workflow

### 7.1 Starting Development

```bash
# 1. Start Minikube
minikube start --memory 10240 --cpus 6

# 2. Start Tilt
tilt up

# 3. Access services
open http://localhost:11100  # UI
open http://localhost:11200  # Vault
open http://localhost:11201  # Keycloak
```

### 7.2 Storing HuggingFace Token

```bash
export HF_TOKEN="hf_xxxxx"
tilt trigger vault-init
```

---

## 8. Traceability Matrix

| Requirement | K8s Manifest | Tiltfile | Test |
|-------------|--------------|----------|------|
| REQ-SEC-001 | vault.yaml | ✅ | pending |
| REQ-SEC-002 | keycloak.yaml | ✅ | pending |
| REQ-SEC-003 | spicedb.yaml | ✅ | pending |
| REQ-DB-001 | postgres.yaml | ✅ | pending |
| REQ-DB-002 | milvus.yaml | ✅ | pending |
| REQ-ORM-001 | entities.rs | N/A | pending |
