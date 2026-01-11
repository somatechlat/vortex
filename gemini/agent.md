# VORTEX-GEN 3.0 "Centaur" - Agent Context Document

> **Purpose**: This file provides immediate, complete context for any AI agent or developer joining this project. Read this FIRST before any work.

---

## 🎯 Project Identity

**Name**: VORTEX-GEN 3.0 "Centaur"  
**Type**: Local-First, Hybrid AI Execution Environment  
**Architecture**: Rust Control Plane + Python Compute Fabric  
**Status**: SRS Complete → Ready for Implementation

---

## 🏗️ The Centaur Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    VORTEX SYSTEM                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│    ┌─────────────┐          ┌─────────────────────────┐    │
│    │  Frontend   │◄────────►│     Core Engine         │    │
│    │  (Svelte)   │  HTTP/WS │       (Rust)            │    │
│    │  Port 11188 │          │                         │    │
│    └─────────────┘          └───────────┬─────────────┘    │
│                                         │                   │
│                                         │ Protobuf/UDS      │
│                                         ▼                   │
│    ┌─────────────────────────────────────────────────┐     │
│    │              Compute Fabric (Python)            │     │
│    │  ┌─────────┐ ┌─────────┐ ┌─────────┐          │     │
│    │  │Worker 1 │ │Worker 2 │ │Worker N │          │     │
│    │  └────┬────┘ └────┬────┘ └────┬────┘          │     │
│    └───────┼───────────┼───────────┼────────────────┘     │
│            │           │           │                       │
│            └───────────┼───────────┘                       │
│                        ▼                                   │
│    ┌─────────────────────────────────────────────────┐    │
│    │         Shared Memory Arena (64GB)              │    │
│    │              Apache Arrow Format                │    │
│    └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 Repository Structure

```
vortex/
├── .agent/                    # Agent configuration
│   └── workflows/             # Reusable workflows
├── docs/                      # Documentation
│   ├── specs/                 # SRS documents (ISO 29148)
│   │   ├── 00_master_srs.md          # 1,601 lines
│   │   ├── 01_core_engine_srs.md     # 1,906 lines
│   │   ├── 02_frontend_ui_srs.md     # 1,764 lines
│   │   ├── 03_compute_fabric_srs.md  # 1,670 lines
│   │   └── 04_registry_srs.md        # 1,753 lines
│   ├── architecture/          # Design docs
│   └── research/              # Competitive analysis
├── crates/                    # Rust workspace
│   ├── vortex-core/           # Core engine
│   │   └── src/
│   │       ├── arbiter.rs     # VRAM management
│   │       ├── db.rs          # SQLite persistence
│   │       ├── error.rs       # Error types
│   │       ├── graph.rs       # DAG structures
│   │       ├── ipc.rs         # Worker communication
│   │       ├── scheduler.rs   # Kahn's algorithm
│   │       ├── shm.rs         # Shared memory
│   │       └── supervisor.rs  # Worker lifecycle
│   ├── vortex-registry/       # Package manager
│   └── vortex-protocol/       # Protobuf definitions
├── worker/                    # Python compute fabric
│   ├── worker.py              # Main loop
│   ├── executors/             # Node implementations
│   └── sandbox/               # Security layer
├── ui/                        # Svelte frontend
│   ├── src/
│   │   ├── lib/
│   │   │   ├── stores/        # Svelte stores
│   │   │   ├── components/    # UI components
│   │   │   └── canvas/        # WebGL rendering
│   │   └── routes/
│   └── static/
├── agent.md                   # THIS FILE
├── rules.md                   # VIBE Coding Rules
├── Cargo.toml                 # Rust workspace
└── README.md                  # Project overview
```

---

## ⚡ Key Technologies

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Host Runtime** | Rust 1.75+ | Async control plane |
| **Async Framework** | Tokio | Non-blocking I/O |
| **Incremental DB** | Salsa | Graph memoization |
| **Persistence** | SQLite/SQLx | Workflow history |
| **Memory Format** | Apache Arrow | Zero-copy tensors |
| **IPC** | Protobuf + UDS | Worker communication |
| **Worker Runtime** | Python 3.10+ | AI inference |
| **Tensor Library** | PyTorch | GPU compute |
| **Frontend** | Svelte 5 | Reactive UI |
| **Canvas** | WebGL2 | High-perf rendering |
| **Collaboration** | Yjs | CRDT sync |
| **Sandbox** | Seccomp BPF | Worker isolation |

---

## 🔴 VIBE Coding Rules (CRITICAL)

**These rules are NON-NEGOTIABLE. Memorize them:**

1. **NO BULLSHIT** - No lies, no guesses, no invented APIs
2. **CHECK FIRST** - Read architecture before touching code
3. **NO UNNECESSARY FILES** - Modify existing, don't create new
4. **REAL IMPLEMENTATIONS** - No mocks, no stubs, no TODOs
5. **DOCUMENTATION = TRUTH** - Read docs, cite sources
6. **COMPLETE CONTEXT** - Understand full data flow first
7. **REAL DATA ONLY** - No assumptions, verify everything

**Framework Rules:**
- Rust for Core (Tokio/Arrow)
- Svelte 5 + Tauri for UI
- Python ONLY for inference kernels
- No `unwrap()`, no panics, use `Result<T, E>`

---

## 📊 SRS Document Summary

| Document | Lines | Key Contents |
|----------|-------|--------------|
| **00_master_srs.md** | 1,601 | System architecture, SHM layout, IPC protocol |
| **01_core_engine_srs.md** | 1,906 | Kahn's algorithm, Salsa caching, Arbiter eviction |
| **02_frontend_ui_srs.md** | 1,764 | Hybrid rendering, CRDT collab, accessibility |
| **03_compute_fabric_srs.md** | 1,670 | Zero-copy bridge, seccomp sandbox, DLPack |
| **04_registry_srs.md** | 1,753 | PubGrub solver, AST security scan, lockfiles |

**Total: 8,694 lines of ISO 29148 compliant specifications**

---

## 🔑 Critical Data Structures

### Shared Memory Header (C-Layout)
```c
struct ShmHeader {
    uint64_t magic_bytes;       // 0x56545833 ("VTX3")
    uint32_t version;           // 1
    atomic_uint32_t flags;      // Bit 0: READY, Bit 1: MAINT
    atomic_uint64_t clock_tick; // Heartbeat
    struct WorkerSlot slots[256];
};
```

### Node State (TypeScript)
```typescript
interface Node {
  id: string;
  type: string;
  position: { x: number; y: number };
  inputs: Record<string, PortHandle>;
  outputs: Record<string, PortHandle>;
  params: Record<string, WidgetValue>;
  $status: "IDLE" | "RUNNING" | "ERROR";
  $progress: number;
}
```

### Job Definition (Protobuf)
```protobuf
message Job {
    string node_id = 1;
    string job_id = 2;
    string op_type = 3;
    map<string, int64> input_handles = 4;
    map<string, string> params = 5;
}
```

---

## 🚀 Implementation Priority

### Phase 1 (Core - 13 weeks)
1. Node workflow system (Svelte Flow)
2. Zero-copy SHM transport (Arrow)
3. Incremental compute (Salsa)
4. Basic node library (8 nodes)
5. Model caching (LFU)

### Phase 2 (Power - 10 weeks)
6. ControlNet integration
7. IP-Adapter support
8. Unified canvas
9. ADetailer nodes
10. Low VRAM mode

### Phase 3 (Unique - 8 weeks)
11. Secure sandbox (seccomp)
12. CRDT collaboration (Yjs)
13. Package manager (PubGrub)

---

## 📈 Mathematical Formulas

**Merkle Hash**: $H(n) = \text{SHA256}(\text{op} \| \text{params} \| \bigoplus H(\text{parents}))$

**VRAM Prediction**: $\text{size}(T) = \prod_{i} d_i \times \text{sizeof}(\tau)$

**Eviction Score**: $S(t) = \min\{i : P[i] \text{ uses } t\}$

**Kahn Complexity**: $O(n + m)$ for $n$ nodes, $m$ edges

---

## ❌ Anti-Patterns to Avoid

- **NO** Gradio UI (slow, limited)
- **NO** synchronous Python server (GIL)
- **NO** pickle serialization (security)
- **NO** `unwrap()` or panics
- **NO** hardcoded strings (use i18n)
- **NO** spaghetti node wiring

---

## ✅ Quick Start Checklist

1. [ ] Read `rules.md` completely
2. [ ] Read all 5 SRS documents
3. [ ] Understand Centaur architecture
4. [ ] Review existing Rust code in `crates/vortex-core/`
5. [ ] Review worker.py implementation
6. [ ] Check competitive_analysis.md for context
7. [ ] Review feature_implementation_plan.md for roadmap

---

## 📞 Key References

| Resource | Location |
|----------|----------|
| VIBE Rules | `rules.md` |
| Master SRS | `docs/specs/00_master_srs.md` |
| Core Engine SRS | `docs/specs/01_core_engine_srs.md` |
| Frontend SRS | `docs/specs/02_frontend_ui_srs.md` |
| Compute Fabric SRS | `docs/specs/03_compute_fabric_srs.md` |
| Registry SRS | `docs/specs/04_registry_srs.md` |
| Competitive Analysis | `docs/research/competitive_analysis.md` |
| Feature Plan | `docs/architecture/feature_implementation_plan.md` |

---

**Last Updated**: 2026-01-06  
**Status**: Ready for Implementation 🚀
