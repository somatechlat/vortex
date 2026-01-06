# Phase 3: Compute Fabric

> **Duration**: Weeks 7-9  
> **Dependencies**: Phase 1  
> **Status**: ⚪ Blocked  
> **SDD Reference**: [03_compute_fabric_sdd.md](../docs/design/03_compute_fabric_sdd.md)

---

## P3.1 Worker Main Loop (SDD §3.1)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.1.1 | Implement `main.py` entry point | ⚪ | - | |
| P3.1.2 | Implement config loading from environment | ⚪ | - | |
| P3.1.3 | Implement `event_loop()` with select() | ⚪ | - | |
| P3.1.4 | Implement heartbeat update | ⚪ | - | |
| P3.1.5 | Write worker startup test | ⚪ | - | |

---

## P3.2 IPC Module (SDD §3.2)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.2.1 | Implement `IPCSocket` class | ⚪ | - | |
| P3.2.2 | Implement `receive()` with length prefix | ⚪ | - | |
| P3.2.3 | Implement `send()` with protobuf | ⚪ | - | |
| P3.2.4 | Implement handshake protocol | ⚪ | - | |
| P3.2.5 | Write IPC integration test with Rust | ⚪ | - | **Critical** |

---

## P3.3 SHM Module (SDD §3.3)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.3.1 | Implement `ShmArena` with mmap | ⚪ | - | |
| P3.3.2 | Implement ctypes `ShmHeader` | ⚪ | - | |
| P3.3.3 | Implement ctypes `WorkerSlot` | ⚪ | - | |
| P3.3.4 | Implement `register_worker()` | ⚪ | - | |
| P3.3.5 | Write SHM read/write test with Rust | ⚪ | - | |

---

## P3.4 Bridge Module (SDD §3.4)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.4.1 | Implement `arrow_to_tensor()` zero-copy | ⚪ | - | |
| P3.4.2 | Implement `tensor_to_arrow()` | ⚪ | - | |
| P3.4.3 | Implement DLPack conversion | ⚪ | - | |
| P3.4.4 | Write benchmark (target: <1ms for 1GB tensor) | ⚪ | - | |

---

## P3.5 Sandbox Module (SDD §3.5)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.5.1 | Implement Seccomp BPF filter (Linux) | ⚪ | - | |
| P3.5.2 | Implement import hook for blocked modules | ⚪ | - | |
| P3.5.3 | Test `os.system()` is blocked | ⚪ | - | |
| P3.5.4 | Test `subprocess` is blocked | ⚪ | - | |
| P3.5.5 | Test `socket` is blocked | ⚪ | - | |

---

## P3.6 Executor Module (SDD §3.6)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.6.1 | Implement `AbstractExecutor` base class | ⚪ | - | |
| P3.6.2 | Implement `ExecutorRegistry` with discovery | ⚪ | - | |
| P3.6.3 | Implement `KSamplerExecutor` | ⚪ | - | |
| P3.6.4 | Implement `VAEDecodeExecutor` | ⚪ | - | |
| P3.6.5 | Implement `CLIPTextEncodeExecutor` | ⚪ | - | |
| P3.6.6 | Implement `LoadCheckpointExecutor` | ⚪ | - | |
| P3.6.7 | Write executor unit tests | ⚪ | - | |

---

## P3.7 Error Handling

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P3.7.1 | Implement OOM recovery with gc.collect() | ⚪ | - | |
| P3.7.2 | Implement CUDA error catching | ⚪ | - | |
| P3.7.3 | Implement graceful shutdown | ⚪ | - | |
| P3.7.4 | Write stress test | ⚪ | - | |

---

## Completion Checklist

- [ ] Worker starts and connects to Rust host
- [ ] SHM read/write verified with Rust
- [ ] Sandbox blocks dangerous imports
- [ ] Basic executor runs SDXL inference

---

**Unblocks**: Phase 6 (Integration)
