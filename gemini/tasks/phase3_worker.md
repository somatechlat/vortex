# Phase 3: Compute Fabric

> **Duration**: Weeks 7-9  
> **Dependencies**: Phase 1  
> **Status**: 🔵 ~75%  
> **SDD Reference**: [03_compute_fabric_sdd.md](../docs/design/03_compute_fabric_sdd.md)

---

## P3.1 Worker Main Loop (SDD §3.1)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.1.1 | Implement `main.py` entry point | Done | Standalone mode |
| P3.1.2 | Implement config loading from environment | Done | `config.py` |
| P3.1.3 | Implement `event_loop()` with select() | Done | Heartbeat loop |
| P3.1.4 | Implement heartbeat update | Done | SHM slot update |
| P3.1.5 | Write worker startup test | Pending | Needs E2E |

---

## P3.2 IPC Module (SDD §3.2)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.2.1 | Implement `IPCSocket` class | Done | `ipc.py` |
| P3.2.2 | Implement `receive()` with length prefix | Done | Protobuf framing |
| P3.2.3 | Implement `send()` with protobuf | Done | |
| P3.2.4 | Implement handshake protocol | Done | |
| P3.2.5 | Write IPC integration test with Rust | Pending | **Critical** |

---

## P3.3 SHM Module (SDD §3.3)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.3.1 | Implement `ShmArena` with mmap | Done | `shm.py` |
| P3.3.2 | Implement ctypes `ShmHeader` | Done | 64MB arena |
| P3.3.3 | Implement ctypes `WorkerSlot` | Done | |
| P3.3.4 | Implement `register_worker()` | Done | |
| P3.3.5 | Write SHM read/write test with Rust | Pending | |

---

## P3.4 Bridge Module (SDD §3.4)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.4.1 | Implement `arrow_to_tensor()` zero-copy | Done | `bridge.py` |
| P3.4.2 | Implement `tensor_to_arrow()` | Done | |
| P3.4.3 | Implement DLPack conversion | Done | |
| P3.4.4 | Write benchmark (target: <1ms for 1GB tensor) | Pending | |

---

## P3.5 Sandbox Module (SDD §3.5)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.5.1 | Implement Seccomp BPF filter (Linux) | Pending | Platform-specific |
| P3.5.2 | Implement import hook for blocked modules | Done | `sandbox.py` |
| P3.5.3 | Test `os.system()` is blocked | Pending | |
| P3.5.4 | Test `subprocess` is blocked | Pending | |
| P3.5.5 | Test `socket` is blocked | Pending | |

---

## P3.6 Executor Module (SDD §3.6)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.6.1 | Implement `AbstractExecutor` base class | Done | `executor.py` |
| P3.6.2 | Implement `ExecutorRegistry` with discovery | Done | |
| P3.6.3 | Implement `KSamplerExecutor` | Done | Initial implementation |
| P3.6.4 | Implement `VAEDecodeExecutor` | Done | Initial implementation |
| P3.6.5 | Implement `CLIPTextEncodeExecutor` | Done | Initial implementation |
| P3.6.6 | Implement `LoadCheckpointExecutor` | Done | Initial implementation |
| P3.6.7 | Write executor unit tests | Pending | |

---

## P3.7 Error Handling

| ID | Task | Status | Notes |
|----|------|--------|-------|
| P3.7.1 | Implement OOM recovery with gc.collect() | Pending | |
| P3.7.2 | Implement CUDA error catching | Pending | |
| P3.7.3 | Implement graceful shutdown | Done | SIGTERM |
| P3.7.4 | Write stress test | Pending | |

---

## Completion Checklist

- [x] Worker main.py with event loop
- [x] SHM arena with ctypes
- [x] IPC socket with protobuf
- [x] Bridge with DLPack
- [x] Executor framework
- [ ] Worker pod running in K8s
- [ ] E2E integration test

---

**Unblocks**: Phase 6 (Integration)
