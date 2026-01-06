# VORTEX Implementation Tasks
## Derived from Software Design Documents

> **Source**: 4 SDDs (1,800+ lines) | 5 SRS (8,700+ lines)  
> **Phases**: 6 | **Total Tasks**: 120+  
> **Est. Duration**: 12-16 weeks (2 developers)

---

## 📊 Phase Overview

| Phase | Module | Duration | Dependencies |
|-------|--------|----------|--------------|
| **0** | Project Setup | 1 week | None |
| **1** | Protocol & Shared Types | 1 week | Phase 0 |
| **2** | Core Engine | 4 weeks | Phase 1 |
| **3** | Compute Fabric | 3 weeks | Phase 1 |
| **4** | Frontend UI | 3 weeks | Phase 2 |
| **5** | Registry System | 2 weeks | Phase 2 |
| **6** | Integration & Deploy | 2 weeks | All |

---

## PHASE 0: PROJECT SETUP (Week 1)

### P0.1 Repository Scaffolding
- [ ] **P0.1.1** Create directory structure per `project_structure.md`
- [ ] **P0.1.2** Initialize Cargo workspace with `vortex-core`, `vortex-registry`, `vortex-protocol`
- [ ] **P0.1.3** Create Python `worker/` package with `pyproject.toml`
- [ ] **P0.1.4** Create Svelte app in `ui/` with Vite + TypeScript
- [ ] **P0.1.5** Setup `.editorconfig`, `.gitignore`, `.pre-commit-config.yaml`

### P0.2 CI/CD Foundation
- [ ] **P0.2.1** Create `.github/workflows/ci.yml` (build + test)
- [ ] **P0.2.2** Create `.github/workflows/security.yml` (cargo-audit, bandit)
- [ ] **P0.2.3** Setup Dependabot for Rust and Python
- [ ] **P0.2.4** Create issue templates

### P0.3 Development Environment
- [ ] **P0.3.1** Create `docker-compose.yml` for local dev
- [ ] **P0.3.2** Create `Tiltfile` for Minikube
- [ ] **P0.3.3** Create `k8s/` base manifests
- [ ] **P0.3.4** Write `scripts/setup-dev.sh`

---

## PHASE 1: PROTOCOL & SHARED TYPES (Week 2)

### P1.1 Protobuf Definitions
- [ ] **P1.1.1** Define `proto/control.proto` (IPC messages)
  - `ControlPacket`, `JobSubmit`, `JobResult`
  - Tracing context fields (`trace_id`, `span_id`)
- [ ] **P1.1.2** Define `proto/graph.proto` (graph structures)
  - `GraphDSL`, `NodeDef`, `EdgeDef`, `PortRef`
- [ ] **P1.1.3** Define `proto/worker.proto` (worker messages)
  - `Handshake`, `Heartbeat`, `Capability`
- [ ] **P1.1.4** Setup `prost-build` in `vortex-protocol/build.rs`

### P1.2 Shared Rust Types
- [ ] **P1.2.1** Create `types.rs` (NodeId, GraphId, JobId newtypes)
- [ ] **P1.2.2** Create `errors.rs` with VE-XXX error codes
- [ ] **P1.2.3** Create `constants.rs` (magic bytes, port numbers)
- [ ] **P1.2.4** Add `#[derive]` for Serialize, Debug, Clone

### P1.3 Shared Memory Header
- [ ] **P1.3.1** Define `ShmHeader` struct (Rust)
- [ ] **P1.3.2** Define `WorkerSlot` struct (Rust)
- [ ] **P1.3.3** Define matching ctypes in Python
- [ ] **P1.3.4** Write cross-language verification test

---

## PHASE 2: CORE ENGINE (Weeks 3-6)

### P2.1 Graph Module (SDD §3.1)
- [ ] **P2.1.1** Implement `GraphDSL` struct with serde
- [ ] **P2.1.2** Implement `NodeDef`, `EdgeDef`, `PortRef`
- [ ] **P2.1.3** Implement `validate()` - cycle detection
- [ ] **P2.1.4** Implement `validate()` - port type checking
- [ ] **P2.1.5** Implement `validate()` - required inputs
- [ ] **P2.1.6** Write unit tests for validation

### P2.2 Scheduler Module (SDD §3.2)
- [ ] **P2.2.1** Implement Kahn's algorithm in `kahn.rs`
- [ ] **P2.2.2** Implement `compute_dirty_set()` in `dirty.rs`
- [ ] **P2.2.3** Implement `compute_node_hash()` with SHA256
- [ ] **P2.2.4** Implement `ExecutionPlan` struct
- [ ] **P2.2.5** Write benchmark for 10,000 node graph

### P2.3 Salsa Module (SDD §3.3)
- [ ] **P2.3.1** Define Salsa database with `#[salsa::database]`
- [ ] **P2.3.2** Implement `node_params` input query
- [ ] **P2.3.3** Implement `node_hash` derived query
- [ ] **P2.3.4** Implement `execution_plan` derived query
- [ ] **P2.3.5** Write incremental recomputation test

### P2.4 Arbiter Module (SDD §3.4)
- [ ] **P2.4.1** Implement `Arbiter` struct with VRAM tracking
- [ ] **P2.4.2** Implement `TensorMeta` with LFU scoring
- [ ] **P2.4.3** Implement `request_allocation()` with eviction
- [ ] **P2.4.4** Implement `evict_lfu()` algorithm
- [ ] **P2.4.5** Write memory pressure test

### P2.5 Supervisor Module (SDD §3.5)
- [ ] **P2.5.1** Implement `spawn_worker()` with fork/exec
- [ ] **P2.5.2** Implement handshake protocol
- [ ] **P2.5.3** Implement SIGCHLD handler for crash detection
- [ ] **P2.5.4** Implement `handle_crash()` with respawn
- [ ] **P2.5.5** Write crash recovery integration test

### P2.6 IPC Gateway Module (SDD §3.6)
- [ ] **P2.6.1** Implement `IpcGateway` with UnixListener
- [ ] **P2.6.2** Implement connection accept loop
- [ ] **P2.6.3** Implement length-prefixed protobuf framing
- [ ] **P2.6.4** Implement `send_job()` with tracing context
- [ ] **P2.6.5** Write round-trip latency benchmark (target: <50μs)

### P2.7 SHM Arena Module (SDD §3.7)
- [ ] **P2.7.1** Implement POSIX shm_open/mmap
- [ ] **P2.7.2** Implement `ShmHeader` initialization
- [ ] **P2.7.3** Implement `BumpAllocator` with 64-byte alignment
- [ ] **P2.7.4** Implement `allocate()` and `free()`
- [ ] **P2.7.5** Write cross-process sharing test

### P2.8 Database Module
- [ ] **P2.8.1** Create SQLite schema (runs, nodes, edges)
- [ ] **P2.8.2** Setup SQLx with compile-time checking
- [ ] **P2.8.3** Implement `create_run()`, `update_run()` queries
- [ ] **P2.8.4** Implement history queries
- [ ] **P2.8.5** Write migration script

### P2.9 API Module (SDD §3.8)
- [ ] **P2.9.1** Setup Axum router with tracing middleware
- [ ] **P2.9.2** Implement `POST /api/graph` (create)
- [ ] **P2.9.3** Implement `POST /api/graph/:id/execute`
- [ ] **P2.9.4** Implement WebSocket upgrade handler
- [ ] **P2.9.5** Implement Prometheus metrics endpoint
- [ ] **P2.9.6** Write OpenAPI spec

### P2.10 Telemetry
- [ ] **P2.10.1** Setup OpenTelemetry with Jaeger exporter
- [ ] **P2.10.2** Instrument all public functions with spans
- [ ] **P2.10.3** Setup structured JSON logging
- [ ] **P2.10.4** Create Grafana dashboard

---

## PHASE 3: COMPUTE FABRIC (Weeks 7-9)

### P3.1 Worker Main Loop (SDD §3.1)
- [ ] **P3.1.1** Implement `main.py` entry point
- [ ] **P3.1.2** Implement config loading from environment
- [ ] **P3.1.3** Implement `event_loop()` with select()
- [ ] **P3.1.4** Implement heartbeat update
- [ ] **P3.1.5** Write worker startup test

### P3.2 IPC Module (SDD §3.2)
- [ ] **P3.2.1** Implement `IPCSocket` class
- [ ] **P3.2.2** Implement `receive()` with length prefix
- [ ] **P3.2.3** Implement `send()` with protobuf
- [ ] **P3.2.4** Implement handshake protocol
- [ ] **P3.2.5** Write IPC integration test with Rust

### P3.3 SHM Module (SDD §3.3)
- [ ] **P3.3.1** Implement `ShmArena` with mmap
- [ ] **P3.3.2** Implement ctypes `ShmHeader`
- [ ] **P3.3.3** Implement ctypes `WorkerSlot`
- [ ] **P3.3.4** Implement `register_worker()`
- [ ] **P3.3.5** Write SHM read/write test with Rust

### P3.4 Bridge Module (SDD §3.4)
- [ ] **P3.4.1** Implement `arrow_to_tensor()` zero-copy
- [ ] **P3.4.2** Implement `tensor_to_arrow()` 
- [ ] **P3.4.3** Implement DLPack conversion
- [ ] **P3.4.4** Write benchmark (target: <1ms for 1GB tensor)

### P3.5 Sandbox Module (SDD §3.5)
- [ ] **P3.5.1** Implement Seccomp BPF filter (Linux)
- [ ] **P3.5.2** Implement import hook for blocked modules
- [ ] **P3.5.3** Test `os.system()` is blocked
- [ ] **P3.5.4** Test `subprocess` is blocked
- [ ] **P3.5.5** Test `socket` is blocked

### P3.6 Executor Module (SDD §3.6)
- [ ] **P3.6.1** Implement `AbstractExecutor` base class
- [ ] **P3.6.2** Implement `ExecutorRegistry` with discovery
- [ ] **P3.6.3** Implement `KSamplerExecutor`
- [ ] **P3.6.4** Implement `VAEDecodeExecutor`
- [ ] **P3.6.5** Implement `CLIPTextEncodeExecutor`
- [ ] **P3.6.6** Implement `LoadCheckpointExecutor`
- [ ] **P3.6.7** Write executor unit tests

### P3.7 Error Handling
- [ ] **P3.7.1** Implement OOM recovery with gc.collect()
- [ ] **P3.7.2** Implement CUDA error catching
- [ ] **P3.7.3** Implement graceful shutdown
- [ ] **P3.7.4** Write stress test

---

## PHASE 4: FRONTEND UI (Weeks 10-12)

### P4.1 State Management (SDD §3.1)
- [ ] **P4.1.1** Create `graph.svelte.ts` with $state
- [ ] **P4.1.2** Create `viewport.svelte.ts`
- [ ] **P4.1.3** Create `selection.svelte.ts`
- [ ] **P4.1.4** Create `execution.svelte.ts`
- [ ] **P4.1.5** Implement mutation recording for undo/redo

### P4.2 Canvas Rendering (SDD §3.2)
- [ ] **P4.2.1** Implement WebGL2 context setup
- [ ] **P4.2.2** Implement node instanced rendering shader
- [ ] **P4.2.3** Implement edge rendering shader
- [ ] **P4.2.4** Implement viewport culling
- [ ] **P4.2.5** Implement LOD switching at zoom thresholds
- [ ] **P4.2.6** Write FPS benchmark (target: 60fps @ 2000 nodes)

### P4.3 Node Components (SDD §3.3)
- [ ] **P4.3.1** Create `Node.svelte` with drag handling
- [ ] **P4.3.2** Create `NodeHeader.svelte`
- [ ] **P4.3.3** Create `NodeBody.svelte` with widgets
- [ ] **P4.3.4** Create `NodeLite.svelte` for LOD
- [ ] **P4.3.5** Create `Port.svelte` with connection handling

### P4.4 Edge Components
- [ ] **P4.4.1** Create `Edge.svelte` with Bezier path
- [ ] **P4.4.2** Implement edge drag preview
- [ ] **P4.4.3** Implement port compatibility checking
- [ ] **P4.4.4** Implement connection animation

### P4.5 Collaboration (SDD §3.4)
- [ ] **P4.5.1** Setup Yjs document
- [ ] **P4.5.2** Setup y-websocket provider
- [ ] **P4.5.3** Sync Yjs ↔ Svelte store
- [ ] **P4.5.4** Implement cursor awareness
- [ ] **P4.5.5** Write multi-user sync test

### P4.6 Panels & Overlays
- [ ] **P4.6.1** Create `Toolbar.svelte`
- [ ] **P4.6.2** Create `Sidebar.svelte` with node palette
- [ ] **P4.6.3** Create `ContextMenu.svelte`
- [ ] **P4.6.4** Create `CommandPalette.svelte`
- [ ] **P4.6.5** Create `Toast.svelte` notifications

### P4.7 Services
- [ ] **P4.7.1** Implement HTTP client with fetch
- [ ] **P4.7.2** Implement WebSocket client
- [ ] **P4.7.3** Implement reconnection logic
- [ ] **P4.7.4** Implement localStorage persistence

### P4.8 Design System
- [ ] **P4.8.1** Create `tokens.css` with design tokens
- [ ] **P4.8.2** Create `reset.css`
- [ ] **P4.8.3** Create `typography.css` with Geist fonts
- [ ] **P4.8.4** Create `animations.css`

### P4.9 Testing
- [ ] **P4.9.1** Setup Playwright
- [ ] **P4.9.2** Write `canvas.spec.ts`
- [ ] **P4.9.3** Write `node.spec.ts`
- [ ] **P4.9.4** Write `connection.spec.ts`
- [ ] **P4.9.5** Write `zoom.spec.ts`
- [ ] **P4.9.6** Setup visual regression

---

## PHASE 5: REGISTRY SYSTEM (Weeks 13-14)

### P5.1 Solver Module (SDD §3.1)
- [ ] **P5.1.1** Implement `Package`, `Version` types
- [ ] **P5.1.2** Implement `Term`, `Incompatibility` types
- [ ] **P5.1.3** Implement `unit_propagate()`
- [ ] **P5.1.4** Implement `resolve_conflict()` (CDCL)
- [ ] **P5.1.5** Implement main `solve()` loop
- [ ] **P5.1.6** Write solver correctness tests

### P5.2 Scanner Module (SDD §3.2)
- [ ] **P5.2.1** Setup rustpython-parser
- [ ] **P5.2.2** Implement danger pattern matchers
- [ ] **P5.2.3** Implement `scan_package()`
- [ ] **P5.2.4** Implement `SecurityReport`
- [ ] **P5.2.5** Write scanner tests with known malware patterns

### P5.3 Venv Module (SDD §3.3)
- [ ] **P5.3.1** Implement Python detection
- [ ] **P5.3.2** Implement `fork_environment()`
- [ ] **P5.3.3** Implement environment registry
- [ ] **P5.3.4** Write isolation test

### P5.4 Manifest & Lockfile (SDD §3.4)
- [ ] **P5.4.1** Implement `vortex.toml` parser
- [ ] **P5.4.2** Implement `vortex.lock` parser/writer
- [ ] **P5.4.3** Implement SHA256 hash verification
- [ ] **P5.4.4** Write integrity test

### P5.5 CLI
- [ ] **P5.5.1** Implement `vtx install`
- [ ] **P5.5.2** Implement `vtx update`
- [ ] **P5.5.3** Implement `vtx remove`
- [ ] **P5.5.4** Implement `vtx scan`
- [ ] **P5.5.5** Implement `vtx tree`

---

## PHASE 6: INTEGRATION & DEPLOY (Weeks 15-16)

### P6.1 End-to-End Testing
- [ ] **P6.1.1** Write full pipeline test (UI → Core → Worker)
- [ ] **P6.1.2** Write crash recovery E2E test
- [ ] **P6.1.3** Write collaboration E2E test
- [ ] **P6.1.4** Write performance regression test

### P6.2 Docker Images
- [ ] **P6.2.1** Create `docker/core/Dockerfile`
- [ ] **P6.2.2** Create `docker/worker/Dockerfile` (CUDA)
- [ ] **P6.2.3** Create `docker/ui/Dockerfile` (nginx)
- [ ] **P6.2.4** Optimize image sizes (<500MB each)

### P6.3 Kubernetes
- [ ] **P6.3.1** Create production overlays
- [ ] **P6.3.2** Create HPA for workers
- [ ] **P6.3.3** Create PodSecurityPolicy
- [ ] **P6.3.4** Create NetworkPolicy

### P6.4 Observability Stack
- [ ] **P6.4.1** Deploy Prometheus
- [ ] **P6.4.2** Deploy Grafana with dashboards
- [ ] **P6.4.3** Deploy Jaeger for tracing
- [ ] **P6.4.4** Create alerting rules

### P6.5 Documentation
- [ ] **P6.5.1** Write user guide
- [ ] **P6.5.2** Write API documentation
- [ ] **P6.5.3** Write deployment guide
- [ ] **P6.5.4** Write custom node development guide

### P6.6 Release
- [ ] **P6.6.1** Create release workflow
- [ ] **P6.6.2** Tag v1.0.0
- [ ] **P6.6.3** Publish Docker images
- [ ] **P6.6.4** Create GitHub Release

---

## 📈 Progress Tracking

| Phase | Planned | Completed | % |
|-------|---------|-----------|---|
| P0 | 12 | 0 | 0% |
| P1 | 12 | 0 | 0% |
| P2 | 45 | 0 | 0% |
| P3 | 30 | 0 | 0% |
| P4 | 40 | 0 | 0% |
| P5 | 20 | 0 | 0% |
| P6 | 20 | 0 | 0% |
| **Total** | **179** | **0** | **0%** |

---

**Generated From**: 4 SDDs  
**Total Tasks**: 179  
**Ready for Sprint Planning**: ✅
