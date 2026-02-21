# VORTEX-GEN 3.0 — Master Development Plan

**Document ID:** VORTEX-MDP-001
**Version:** 1.0.0
**Classification:** Internal — Engineering
**Date:** 2026-02-20
**Author:** SomaTech Engineering
**Standard:** ISO/IEC 29148:2018 (structure only)

---

## 1. Purpose & Scope

This document is the **single source of truth** for the VORTEX-GEN 3.0 "Centaur" architecture from its current state to production deployment. It covers:

- Canonical repository structure and file hygiene
- 15-task implementation playbook (T-01 through T-15)
- Hybrid deployment architecture (CPU local Docker + GPU AWS SageMaker)
- Agent Toolbox transformation (MCP, Orchestrator, HITL)
- Quality gates and verification strategy

### 1.1 Audience

Engineers, agents, and CI/CD systems interacting with the `somatechlat/vortex` repository.

### 1.2 Governing Rules

| Rule | Source |
|------|--------|
| No `.unwrap()` outside `#[cfg(test)]` | Vibe Coding Rules §1 |
| No placeholders, stubs, TODOs | Vibe Coding Rules §4 |
| Django/Ninja for API (Python services) | User Rule §8 |
| Lit Web Components for UI (non-VORTEX) | User Rule §9 |
| `cargo clippy -D warnings` clean | T-Playbook Constraint §3 |
| `ruff check` + `mypy --strict` clean | T-Playbook Constraint §4 |

---

## 2. Canonical Repository Structure

### 2.1 Target File Tree

```
gemini/
├── Cargo.toml                  # Workspace root
├── Cargo.lock
├── LICENSE
├── README.md
├── Tiltfile                    # Local K8s dev orchestration
├── buildspec.yml               # AWS CodeBuild
├── .editorconfig
├── .gitignore
├── .pre-commit-config.yaml
├── .github/                    # CI/CD workflows, issue templates
│
├── crates/                     # Rust workspace members
│   ├── vortex-config/          # Centralized configuration (DeploymentMode, Vault)
│   ├── vortex-protocol/        # Protobuf types, IPC contract
│   ├── vortex-core/            # Execution engine, API, scheduler, authz, supervisor
│   ├── vortex-registry/        # Package registry, solver
│   ├── vortex-mcp/             # MCP server for Agent integration
│   └── vortex-telemetry/       # Metrics, tracing, logging
│
├── proto/                      # Protobuf definitions (control.proto, worker.proto)
│
├── worker/                     # Python compute workers
│   ├── pyproject.toml
│   ├── requirements.txt
│   ├── requirements.sandbox.txt
│   ├── requirements.live.txt
│   ├── vortex_worker/          # Main package
│   │   ├── __main__.py
│   │   ├── worker.py
│   │   ├── bridge.py           # Arrow ↔ Tensor bridge
│   │   ├── shm.py              # Shared memory bindings
│   │   ├── nodes/              # Compute node implementations
│   │   │   ├── __init__.py
│   │   │   ├── llm_director.py
│   │   │   ├── audio_tts.py
│   │   │   ├── video_gen.py
│   │   │   └── media_muxer.py
│   │   └── generated/          # Protobuf generated code
│   └── tests/
│
├── ui/                         # Svelte frontend (SvelteKit)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api.ts          # API client + WebSocket
│   │   │   ├── components/
│   │   │   └── stores/
│   │   └── routes/
│   └── svelte.config.js
│
├── docker/                     # Dockerfiles
│   ├── core/Dockerfile         # Rust API + engine
│   ├── worker/Dockerfile       # Python CPU worker
│   ├── Dockerfile.worker.cuda  # Python GPU worker (CUDA)
│   └── ui/Dockerfile           # Svelte frontend
│
├── k8s/                        # Kubernetes manifests (Kustomize)
│   ├── base/                   # Shared resources
│   └── overlays/
│       ├── sandbox/            # Local Minikube
│       └── live/               # AWS EKS
│
├── infra/                      # Terraform
│   ├── main.tf                 # Core infra (VPC, ECS, SG)
│   ├── main_serverless.tf      # SageMaker serverless endpoints
│   ├── variables.tf
│   └── outputs.tf
│
├── config/                     # Runtime configuration files
│
├── docs/                       # Documentation
│   ├── architecture/
│   ├── design/
│   ├── specs/
│   └── VORTEX_MASTER_DEVELOPMENT_PLAN.md  # THIS FILE
│
├── scripts/                    # Utility scripts (build, deploy)
│
└── tests/                      # Integration/E2E tests
    ├── integration/
    └── e2e/
```

### 2.2 Files to Purge (Root Pollution)

The following files at the repository root violate world-class organization standards and must be removed or relocated:

| File | Action | Reason |
|------|--------|--------|
| `1#` | **DELETE** | Orphan garbage file |
| `P1_P3_SUMMARY.md` | **DELETE** | Superseded by this plan |
| `agent.md` | **MOVE** → `docs/agent_instructions.md` | Documentation belongs in `docs/` |
| `rules.md` | **MOVE** → `docs/coding_rules.md` | Documentation belongs in `docs/` |
| `tilt_startup.log` | **DELETE** + add to `.gitignore` | Runtime log, never commit |
| `test_dlpack_integration.sh` | **MOVE** → `scripts/test_dlpack.sh` | Test scripts belong in `scripts/` |
| `test_p4_integration.sh` | **MOVE** → `scripts/test_p4.sh` | Test scripts belong in `scripts/` |
| `test_p5_coordinator.sh` | **MOVE** → `scripts/test_p5.sh` | Test scripts belong in `scripts/` |
| `test_p6_complete.sh` | **MOVE** → `scripts/test_p6.sh` | Test scripts belong in `scripts/` |
| `test_real_worker_connection.py` | **MOVE** → `tests/integration/` | Integration test |
| `test_worker_standalone.py` | **MOVE** → `tests/integration/` | Integration test |
| `start_vortex_system.sh` | **MOVE** → `scripts/start_system.sh` | Utility script |
| `deploy_aws.sh` | **MOVE** → `scripts/deploy_aws.sh` | Deploy script |
| `tasks/TASKS.md` (20MB) | **DELETE** | Superseded, bloats repo |
| `.ruff_cache/` | Add to `.gitignore` | Build artifact |
| `tmp/` | Add to `.gitignore` | Temp directory |
| `test-results/` | Add to `.gitignore` | CI artifact |
| `worker/__pycache__/` | Add to `.gitignore` | Python cache |

---

## 3. Hybrid Deployment Architecture

### 3.1 Overview

```
┌─────────────────────────────────────────────────────┐
│              LOCAL (Docker Compose / Minikube)       │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Postgres │  │ SpiceDB  │  │  VORTEX Core API │  │
│  │  :5432   │  │  :8080   │  │     :11188       │  │
│  └──────────┘  └──────────┘  └────────┬─────────┘  │
│                                       │ IPC/UDS     │
│                              ┌────────▼─────────┐   │
│                              │  Python Worker   │   │
│                              │  (CPU Inference) │   │
│                              │  Slot 0..N       │   │
│                              └──────────────────┘   │
│                                                     │
│  ┌──────────┐  ┌──────────┐                         │
│  │  Svelte  │  │ Keycloak │                         │
│  │  UI :5173│  │  :8080   │                         │
│  └──────────┘  └──────────┘                         │
└─────────────────────────────────────────────────────┘
          │
          │  HTTPS (reqwest)
          ▼
┌─────────────────────────────────────────────────────┐
│                  AWS CLOUD                           │
│                                                     │
│  ┌──────────────────────────────────────────────┐   │
│  │        SageMaker Serverless Endpoints        │   │
│  │  ┌────────────┐ ┌────────────┐ ┌──────────┐  │   │
│  │  │ Stable Diff│ │  Whisper   │ │ ElevenLab│  │   │
│  │  │  GPU VRAM  │ │  GPU VRAM  │ │  API Fwd │  │   │
│  │  └────────────┘ └────────────┘ └──────────┘  │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐   │
│  │    ECR     │  │ CloudWatch │  │   S3 Assets  │   │
│  │  Registry  │  │   Logs     │  │   (Output)   │   │
│  └────────────┘  └────────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────┘
```

### 3.2 Local CPU Stack (Docker Compose)

All control-plane services run locally. CPU inference is used for testing and development.

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| `vortex-core` | `docker/core/Dockerfile` | 11188 | Rust API + engine |
| `vortex-worker-cpu` | `docker/worker/Dockerfile` | IPC (UDS) | Python CPU worker |
| `vortex-ui` | `docker/ui/Dockerfile` | 5173 | Svelte frontend |
| `postgres` | `postgres:16-alpine` | 5432 | Graph/run/tenant storage |
| `spicedb` | `authzed/spicedb:v1.33` | 8080 | ReBAC authorization |
| `keycloak` | `quay.io/keycloak/keycloak:24` | 8080 | OIDC/JWT provider |

### 3.3 AWS GPU Stack (Terraform)

GPU-intensive inference runs on AWS SageMaker Serverless Endpoints, managed via Terraform in `infra/`.

| Resource | Terraform File | Purpose |
|----------|----------------|---------|
| SageMaker Endpoints | `main_serverless.tf` | GPU model inference (SD, Whisper) |
| ECR Repository | `main.tf` | Docker image registry |
| S3 Bucket | `main.tf` | Output asset storage |
| CloudWatch | `main.tf` | Centralized logging |
| VPC + Subnets | `main.tf` | Network isolation |
| Security Groups | `main.tf` | Restricted egress (T-14) |
| CodeBuild Project | `buildspec.yml` | CI/CD image builds |

### 3.4 Worker Dispatch Strategy

The execution engine determines dispatch target per-node:

| Node Type | Dispatch Target | Protocol |
|-----------|-----------------|----------|
| `LoadModel`, `KSampler`, `VAEDecode` | Local CPU worker (IPC/UDS) | Protobuf over UDS |
| `StableDiffusion`, `Whisper`, `VideoGen` | SageMaker endpoint | HTTPS (boto3/reqwest) |
| `LLM::Director` | SageMaker or External API | HTTPS |
| `Media::Muxer` | Local worker (FFmpeg) | Protobuf over UDS |

---

## 4. Implementation Playbook — Sprint Plan

### Sprint 1 — P0 Critical (Week 1)

| Task | Status | Description |
|------|--------|-------------|
| T-01 | Complete Done | Wire `output_registry` in `execution.rs` |
| T-02 | Complete Done | Real SpiceDB HTTP `check_permission` |
| T-03 | 🔲 Next | Worker heartbeat + timeout crash detection |
| T-04 | 🔲 Next | JWT middleware + tenant_id extraction |

### Sprint 2 — P1 Execution (Week 2)

| Task | Status | Description |
|------|--------|-------------|
| T-12 | 🔲 | `kahn_levels()` returns `Vec<Vec<NodeID>>` |
| T-05 | 🔲 | Level-parallel node dispatch with worker pool |
| T-06 | 🔲 | Integrate VRAM Arbiter before each dispatch |
| T-07 | 🔲 | CancellationToken per run + wire `cancel_run` |

### Sprint 3 — P1 Integration (Weeks 2-3)

| Task | Status | Description |
|------|--------|-------------|
| T-08 | 🔲 | Rack → DAG compiler (`rack_compiler.rs`) |
| T-09 | 🔲 | Wire Render button to API + WebSocket progress |

### Sprint 4 — P2 Hardening (Week 3)

| Task | Status | Description |
|------|--------|-------------|
| T-10 | 🔲 | Readiness handshake replaces 500ms sleep |
| T-11 | 🔲 | Semaphore execution queue + 429 backpressure |
| T-13 | 🔲 | Replace DIY PubGrub with `pubgrub` crate |
| T-14 | 🔲 | SpiceDB postgres datastore + SG egress restrict |

### Sprint 5 — P3 Quality (Week 4)

| Task | Status | Description |
|------|--------|-------------|
| T-15 | 🔲 | Convert 49 `.unwrap()` calls to proper errors |

### Sprint 6 — Agent Toolbox (Weeks 4-5)

| Task | Status | Description |
|------|--------|-------------|
| T-16 | 🔲 | MCP Server interface in `vortex-mcp` |
| T-17 | 🔲 | NLP-to-DAG Orchestrator (Intent → GraphDSL) |
| T-18 | 🔲 | HITL Control Node (`Control::HITL`) |
| T-19 | 🔲 | Asset Manager (SHM → S3 → signed URL) |
| T-20 | 🔲 | Python node implementations (Director, TTS, Muxer) |

---

## 5. Quality Gates

### 5.1 Per-Commit (CI)

```
cargo clippy -D warnings
cargo test --workspace
ruff check worker/
mypy --strict worker/vortex_worker/
```

### 5.2 Per-Sprint Gate

- All tasks in sprint pass acceptance criteria
- Zero `.unwrap()` outside `#[cfg(test)]`
- Zero hardcoded strings (use `get_message()` or config)
- `docker compose up` succeeds with all services healthy
- Integration test: 3-node graph executes end-to-end

### 5.3 Production Gate

- `terraform plan` clean
- SpiceDB auth state persists across pod restart
- CancellationToken halts execution within 1 node boundary
- SageMaker endpoints respond under 30s for GPU inference
- VRAM arbiter prevents OOM on 24GB GPU
- All 15+ tasks ACCEPTED

---

## 6. Definition of Done

A 3-node workflow (`LoadModel → KSampler → VAEDecode`) submitted via the **Rack UI** completes successfully with:

1. Complete Correct tensor data flowing node-to-node through shared memory
2. Complete Authorization enforced via SpiceDB (postgres-backed)
3. Complete Real-time progress visible on the Signal Bus
4. Complete Cancellation working within 1 node boundary
5. Complete `cargo test --workspace` passes
6. Complete `bun run test:unit` passes
7. Complete `kubectl apply -k k8s/` deploys without errors
8. Complete VRAM limit prevents OOM on 24GB GPU
9. Complete Agent can trigger execution via MCP with HITL approval gates
