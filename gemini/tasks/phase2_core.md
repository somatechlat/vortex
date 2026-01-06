# Phase 2: Core Engine

> **Duration**: Weeks 3-6  
> **Dependencies**: Phase 1  
> **Status**: ⚪ Blocked  
> **SDD Reference**: [01_core_engine_sdd.md](../docs/design/01_core_engine_sdd.md)

---

## P2.1 Graph Module (SDD §3.1)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.1.1 | Implement `GraphDSL` struct with serde | ⚪ | - | |
| P2.1.2 | Implement `NodeDef`, `EdgeDef`, `PortRef` | ⚪ | - | |
| P2.1.3 | Implement `validate()` - cycle detection | ⚪ | - | Tarjan's algorithm |
| P2.1.4 | Implement `validate()` - port type checking | ⚪ | - | |
| P2.1.5 | Implement `validate()` - required inputs | ⚪ | - | |
| P2.1.6 | Write unit tests for validation | ⚪ | - | 90% coverage |

---

## P2.2 Scheduler Module (SDD §3.2)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.2.1 | Implement Kahn's algorithm in `kahn.rs` | ⚪ | - | O(V+E) |
| P2.2.2 | Implement `compute_dirty_set()` in `dirty.rs` | ⚪ | - | |
| P2.2.3 | Implement `compute_node_hash()` with SHA256 | ⚪ | - | |
| P2.2.4 | Implement `ExecutionPlan` struct | ⚪ | - | |
| P2.2.5 | Write benchmark for 10,000 node graph | ⚪ | - | Target: <5ms |

---

## P2.3 Salsa Module (SDD §3.3)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.3.1 | Define Salsa database with `#[salsa::database]` | ⚪ | - | |
| P2.3.2 | Implement `node_params` input query | ⚪ | - | |
| P2.3.3 | Implement `node_hash` derived query | ⚪ | - | |
| P2.3.4 | Implement `execution_plan` derived query | ⚪ | - | |
| P2.3.5 | Write incremental recomputation test | ⚪ | - | |

---

## P2.4 Arbiter Module (SDD §3.4)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.4.1 | Implement `Arbiter` struct with VRAM tracking | ⚪ | - | |
| P2.4.2 | Implement `TensorMeta` with LFU scoring | ⚪ | - | |
| P2.4.3 | Implement `request_allocation()` with eviction | ⚪ | - | |
| P2.4.4 | Implement `evict_lfu()` algorithm | ⚪ | - | |
| P2.4.5 | Write memory pressure test | ⚪ | - | |

---

## P2.5 Supervisor Module (SDD §3.5)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.5.1 | Implement `spawn_worker()` with fork/exec | ⚪ | - | |
| P2.5.2 | Implement handshake protocol | ⚪ | - | |
| P2.5.3 | Implement SIGCHLD handler for crash detection | ⚪ | - | |
| P2.5.4 | Implement `handle_crash()` with respawn | ⚪ | - | |
| P2.5.5 | Write crash recovery integration test | ⚪ | - | |

---

## P2.6 IPC Gateway Module (SDD §3.6)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.6.1 | Implement `IpcGateway` with UnixListener | ⚪ | - | |
| P2.6.2 | Implement connection accept loop | ⚪ | - | |
| P2.6.3 | Implement length-prefixed protobuf framing | ⚪ | - | |
| P2.6.4 | Implement `send_job()` with tracing context | ⚪ | - | |
| P2.6.5 | Write round-trip latency benchmark | ⚪ | - | Target: <50μs |

---

## P2.7 SHM Arena Module (SDD §3.7)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.7.1 | Implement POSIX shm_open/mmap | ⚪ | - | |
| P2.7.2 | Implement `ShmHeader` initialization | ⚪ | - | |
| P2.7.3 | Implement `BumpAllocator` with 64-byte alignment | ⚪ | - | |
| P2.7.4 | Implement `allocate()` and `free()` | ⚪ | - | |
| P2.7.5 | Write cross-process sharing test | ⚪ | - | |

---

## P2.8 Database Module

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.8.1 | Create SQLite schema (runs, nodes, edges) | ⚪ | - | |
| P2.8.2 | Setup SQLx with compile-time checking | ⚪ | - | |
| P2.8.3 | Implement `create_run()`, `update_run()` queries | ⚪ | - | |
| P2.8.4 | Implement history queries | ⚪ | - | |
| P2.8.5 | Write migration script | ⚪ | - | |

---

## P2.9 API Module (SDD §3.8)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.9.1 | Setup Axum router with tracing middleware | ⚪ | - | |
| P2.9.2 | Implement `POST /api/graph` (create) | ⚪ | - | |
| P2.9.3 | Implement `POST /api/graph/:id/execute` | ⚪ | - | |
| P2.9.4 | Implement WebSocket upgrade handler | ⚪ | - | |
| P2.9.5 | Implement Prometheus metrics endpoint | ⚪ | - | Port 11191 |
| P2.9.6 | Write OpenAPI spec | ⚪ | - | |

---

## P2.10 Telemetry

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P2.10.1 | Setup OpenTelemetry with Jaeger exporter | ⚪ | - | |
| P2.10.2 | Instrument all public functions with spans | ⚪ | - | |
| P2.10.3 | Setup structured JSON logging | ⚪ | - | |
| P2.10.4 | Create Grafana dashboard | ⚪ | - | |

---

## Completion Checklist

- [ ] All 45 tasks complete
- [ ] `cargo test -p vortex-core` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] IPC round-trip < 50μs (benchmark)
- [ ] 10,000 node compile < 5ms

---

**Unblocks**: Phase 4 (UI), Phase 5 (Registry)
