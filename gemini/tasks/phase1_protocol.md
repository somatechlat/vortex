# Phase 1: Protocol & Shared Types

> **Duration**: Week 2  
> **Dependencies**: Phase 0  
> **Status**: Pending Blocked

---

## P1.1 Protobuf Definitions

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P1.1.1 | Define `proto/control.proto` (IPC messages) | Pending | - | `ControlPacket`, `JobSubmit`, `JobResult` |
| P1.1.2 | Define `proto/graph.proto` (graph structures) | Pending | - | `GraphDSL`, `NodeDef`, `EdgeDef`, `PortRef` |
| P1.1.3 | Define `proto/worker.proto` (worker messages) | Pending | - | `Handshake`, `Heartbeat`, `Capability` |
| P1.1.4 | Setup `prost-build` in `vortex-protocol/build.rs` | Pending | - | |

---

## P1.2 Shared Rust Types

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P1.2.1 | Create `types.rs` (NodeId, GraphId, JobId newtypes) | Pending | - | |
| P1.2.2 | Create `errors.rs` with VE-XXX error codes | Pending | - | See SRS for codes |
| P1.2.3 | Create `constants.rs` (magic bytes, port numbers) | Pending | - | Port Authority spec |
| P1.2.4 | Add `#[derive]` for Serialize, Debug, Clone | Pending | - | |

---

## P1.3 Shared Memory Header

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P1.3.1 | Define `ShmHeader` struct (Rust) | Pending | - | 64-byte aligned |
| P1.3.2 | Define `WorkerSlot` struct (Rust) | Pending | - | 256 slots |
| P1.3.3 | Define matching ctypes in Python | Pending | - | Must match exactly |
| P1.3.4 | Write cross-language verification test | Pending | - | **Critical** |

---

## Completion Checklist

- [ ] `cargo build -p vortex-protocol` succeeds
- [ ] Proto files compile without warnings
- [ ] Python ctypes match Rust layout (verified by test)
- [ ] All error codes documented

---

**Unblocks**: Phase 2 (Core), Phase 3 (Worker)
