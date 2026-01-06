# VORTEX Data Flow Specification
## Complete Data & Message Flow Documentation

> **Standard**: ISO/IEC 25024 (Data Quality)  
> **Version**: 1.0.0  
> **Status**: PLANNING

---

## 1. DATA FLOW OVERVIEW

### 1.1 System Data Flow Diagram

```mermaid
flowchart TB
    subgraph "User Interface"
        UI[Frontend UI]
        LS[(localStorage)]
    end
    
    subgraph "Control Plane"
        API[HTTP API]
        WS[WebSocket]
        CORE[Core Engine]
        DB[(SQLite)]
    end
    
    subgraph "Data Plane"
        SHM[(Shared Memory<br/>64GB Arena)]
        W1[Worker 1]
        W2[Worker 2]
        W3[Worker N]
    end
    
    subgraph "External"
        GPU[GPU/CUDA]
        FS[File System]
    end
    
    UI <-->|HTTP/JSON| API
    UI <-->|WebSocket| WS
    UI <-->|Preferences| LS
    
    API <--> CORE
    WS <--> CORE
    CORE <--> DB
    
    CORE <-->|UDS/Protobuf| W1 & W2 & W3
    W1 & W2 & W3 <-->|mmap| SHM
    W1 & W2 & W3 <-->|CUDA| GPU
    
    CORE <--> FS
    W1 & W2 & W3 <--> FS
```

---

## 2. MESSAGE FLOW BY OPERATION

### 2.1 Graph Execution Flow

```mermaid
sequenceDiagram
    participant UI as Frontend
    participant API as HTTP API
    participant Core as Core Engine
    participant Sched as Scheduler
    participant Sup as Supervisor
    participant IPC as IPC Gateway
    participant Worker
    participant SHM as Shared Memory
    participant GPU
    
    Note over UI,GPU: PHASE 1: Request
    UI->>API: POST /api/graph/{id}/execute
    API->>Core: ExecuteGraphCommand
    
    Note over UI,GPU: PHASE 2: Compilation
    Core->>Sched: compile(graph)
    Sched->>Sched: topological_sort()
    Sched->>Sched: compute_dirty_set()
    Sched-->>Core: ExecutionPlan
    
    Note over UI,GPU: PHASE 3: Dispatch (per node)
    loop For each node in plan
        Core->>Sup: dispatch(node)
        Sup->>IPC: send_job(worker_id, job)
        IPC->>Worker: JobSubmit (protobuf)
        
        Note over UI,GPU: PHASE 4: Execution
        Worker->>SHM: read input tensors
        Worker->>GPU: execute()
        GPU-->>Worker: result tensors
        Worker->>SHM: write output tensors
        Worker->>IPC: JobResult
        IPC-->>Sup: result
        Sup-->>Core: NodeComplete
        
        Note over UI,GPU: PHASE 5: Progress
        Core->>API: progress event
        API->>UI: WS: execution.progress
    end
    
    Note over UI,GPU: PHASE 6: Complete
    Core->>API: ExecutionComplete
    API->>UI: WS: execution.complete
```

### 2.2 Real-time Collaboration Flow

```mermaid
sequenceDiagram
    participant UserA as User A (Browser)
    participant YjsA as Yjs Client A
    participant Server as y-websocket
    participant YjsB as Yjs Client B
    participant UserB as User B (Browser)
    
    Note over UserA,UserB: Initial Sync
    YjsA->>Server: Subscribe(room_id)
    YjsB->>Server: Subscribe(room_id)
    Server->>YjsA: StateVector
    Server->>YjsB: StateVector
    
    Note over UserA,UserB: User A moves node
    UserA->>YjsA: moveNode(id, x, y)
    YjsA->>YjsA: Y.Map.set('x', newX)
    YjsA->>Server: Update Vector (binary)
    Server->>YjsB: Update Vector (binary)
    YjsB->>YjsB: Apply update
    YjsB->>UserB: Re-render canvas
    
    Note over UserA,UserB: Conflict Resolution (automatic)
    UserA->>YjsA: moveNode(id, 100, 100)
    UserB->>YjsB: moveNode(id, 200, 200)
    Note over Server: CRDT auto-merges
    Server->>YjsA: Merged state
    Server->>YjsB: Merged state
```

### 2.3 Zero-Copy Tensor Flow

```mermaid
sequenceDiagram
    participant Core as Rust Core
    participant SHM as Shared Memory
    participant Python as Python Worker
    participant Arrow as PyArrow
    participant Torch as PyTorch
    participant GPU as CUDA Device
    
    Note over Core,GPU: Rust writes tensor
    Core->>SHM: allocate(size, align=64)
    Core->>SHM: write Arrow IPC buffer
    Core->>Python: JobSubmit(shm_offset, shape, dtype)
    
    Note over Core,GPU: Python reads (zero-copy)
    Python->>SHM: mmap view at offset
    Python->>Arrow: pa.ipc.read_tensor(buffer)
    Arrow->>Arrow: Zero-copy view
    Arrow->>Torch: from_dlpack(arrow_tensor)
    Torch->>GPU: .to('cuda', non_blocking=True)
    
    Note over Core,GPU: GPU execution
    GPU->>GPU: Forward pass
    
    Note over Core,GPU: Write back (zero-copy)
    Torch->>Torch: .cpu()
    Torch->>Arrow: dlpack export
    Arrow->>SHM: Write Arrow IPC
    Python->>Core: JobResult(output_offset)
```

---

## 3. DATA SCHEMAS

### 3.1 API Request/Response

**POST /api/graph/execute**

| Direction | Format | Content |
|-----------|--------|---------|
| Request | JSON | `{ graph_id: string, options?: ExecutionOptions }` |
| Response | JSON | `{ execution_id: string, status: "queued" }` |

**WebSocket Messages**

| Direction | Type | Payload |
|-----------|------|---------|
| Server→Client | `execution.progress` | `{ node_id, progress: 0-100 }` |
| Server→Client | `execution.complete` | `{ execution_id, outputs }` |
| Server→Client | `execution.error` | `{ node_id, error }` |
| Client→Server | `graph.update` | `{ mutations: Mutation[] }` |
| Server→Client | `graph.sync` | `{ graph: GraphDSL }` |

### 3.2 IPC Protocol (Protobuf)

**Control Packet Structure:**
```
┌─────────────────────────────────────────────────────┐
│ Length (4 bytes, little-endian)                     │
├─────────────────────────────────────────────────────┤
│ ControlPacket (protobuf)                            │
│  ├── request_id: string (UUID)                      │
│  ├── timestamp: int64 (unix millis)                 │
│  ├── trace_context:                                 │
│  │    ├── trace_id: bytes[16]                       │
│  │    ├── span_id: bytes[8]                         │
│  │    └── parent_span_id: bytes[8]                  │
│  └── payload: oneof {                               │
│       JobSubmit, JobResult, Handshake, Heartbeat    │
│      }                                              │
└─────────────────────────────────────────────────────┘
```

### 3.3 Shared Memory Layout

```
┌────────────────────────────────────────────────────────────────┐
│ Offset 0x0000: ShmHeader (4KB)                                 │
│  ├── magic: u64 = 0x5654_5833_0000_0001                        │
│  ├── version: u32                                              │
│  ├── flags: AtomicU32                                          │
│  ├── clock_tick: AtomicU64                                     │
│  ├── reserved: [u8; 40]                                        │
│  └── slots: [WorkerSlot; 256]                                  │
├────────────────────────────────────────────────────────────────┤
│ Offset 0x1000: Tensor Arena (64GB - 4KB)                       │
│  ├── Block 0: [Tensor A - 512MB]                               │
│  ├── Block 1: [Tensor B - 2GB]                                 │
│  ├── Block 2: [FREE - 1GB]                                     │
│  ├── Block 3: [Tensor C - 256MB]                               │
│  └── ...                                                       │
└────────────────────────────────────────────────────────────────┘
```

---

## 4. DATA LIFECYCLE

### 4.1 Graph Data Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created: User creates graph
    Created --> Modified: User edits
    Modified --> Modified: User edits more
    Modified --> Saved: Auto-save / Manual save
    Saved --> Modified: User edits
    Saved --> Executed: Queue prompt
    Executed --> Completed: Success
    Executed --> Failed: Error
    Completed --> Modified: User edits result
    Failed --> Modified: User fixes
    Modified --> Deleted: User deletes
    Deleted --> [*]
```

### 4.2 Tensor Data Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Allocated: arbiter.allocate()
    Allocated --> Written: worker writes
    Written --> Mapped: downstream reads
    Mapped --> InUse: GPU operation
    InUse --> Done: Operation complete
    Done --> Cached: May be reused
    Cached --> Evicted: LFU eviction
    Cached --> Mapped: Reused
    Evicted --> [*]
```

### 4.3 Execution Data Lifecycle

| Stage | Data Created | Storage | Retention |
|-------|--------------|---------|-----------|
| Request | ExecutionRequest | Memory | Until complete |
| Queued | QueueEntry | Database | 24h |
| Running | Progress updates | WebSocket | Real-time |
| Complete | Output metadata | Database | 90 days |
| Output | Generated images | Filesystem | User-defined |

---

## 5. DATA VALIDATION

### 5.1 Input Validation Points

| Component | Validation | Action on Failure |
|-----------|------------|-------------------|
| **API** | JSON schema | 400 Bad Request |
| **Graph** | Cycle detection | VE-002 error |
| **Graph** | Port type matching | VE-006 error |
| **Worker** | Input handle valid | WK-002 error |
| **Worker** | Parameter bounds | VE-010 error |

### 5.2 Data Integrity Checks

| Check | Trigger | Verification |
|-------|---------|--------------|
| Graph hash | Before execution | SHA256 match |
| Tensor checksum | After transfer | CRC32 |
| Lockfile hash | On install | SHA256 |
| Model hash | On load | SHA256 |

---

## 6. EVENT SOURCING DATA

### 6.1 Event Store Schema

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | UUID | Unique event identifier |
| `sequence_number` | BIGINT | Global ordering |
| `trace_id` | UUID | Distributed trace correlation |
| `span_id` | UUID | Current span |
| `event_type` | STRING | e.g., "NodeCreated" |
| `aggregate_type` | STRING | e.g., "Graph" |
| `aggregate_id` | UUID | Target entity |
| `payload` | JSON | Event data |
| `metadata` | JSON | Context (user, IP, etc.) |
| `created_at` | TIMESTAMP | When event occurred |

### 6.2 Event Types

| Category | Events |
|----------|--------|
| **Graph** | GraphCreated, GraphDeleted, GraphCloned |
| **Node** | NodeCreated, NodeDeleted, NodeMoved, NodeParamUpdated |
| **Edge** | EdgeCreated, EdgeDeleted |
| **Execution** | ExecutionQueued, ExecutionStarted, ExecutionCompleted, ExecutionFailed |
| **Worker** | WorkerSpawned, WorkerCrashed, WorkerRecovered |
| **Memory** | TensorAllocated, TensorEvicted |

---

## 7. DATA FLOW METRICS

### 7.1 Key Flow Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| API latency (p99) | < 50ms | Histogram |
| IPC round-trip | < 50μs | Histogram |
| Tensor mapping | < 1ms | Histogram |
| WS message delivery | < 10ms | Histogram |
| Event store write | < 5ms | Histogram |

### 7.2 Throughput Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Graph compiles/s | > 100 | Counter |
| Jobs dispatched/s | > 50 | Counter |
| WS messages/s | > 1000 | Counter |
| Events stored/s | > 500 | Counter |

---

**Document Status**: COMPLETE  
**Total Flow Diagrams**: 8  
**Total Data Schemas**: 5  
**Ready for Implementation**: ✅
