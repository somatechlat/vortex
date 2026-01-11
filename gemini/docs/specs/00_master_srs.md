# Software Requirements Specification (SRS): Master System Architecture
**Project**: VORTEX-GEN 3.0 "Centaur"
**Module**: Architecture & Core Constraints
**Version**: 9.1.1 (ISO Standard)
**Date**: 2026-01-06
**Standard**: ISO/IEC 29148:2018

---

> **Policy**: Development must mirror production behavior; only local resource limits may differ.

## 1. Introduction

### 1.1 Purpose
This SRS defines the architectural standards, global constraints, and interface boundaries for VORTEX-GEN 3.0. It documents the "Centaur" pattern (Rust Host + Python Compute) and the "Vortex Flow Stream" UI paradigm (Vertical Rack).

### 1.2 Scope
VORTEX-GEN 3.0 is a Local-First, Hybrid AI Execution Environment and agentic tool box (MCP-style) for humans and AI agents. It also supports cluster deployments where the control plane runs on CPU nodes and inference runs on GPU workers.
**The software facilitates**:
*   **Rack-Based Orchestration**: Linear, vertical organization of generative steps.
*   **Graph Compilation**: converting the linear Rack state into an optimized DAG for execution.
*   **Compute Virtualization**: Isolated Python workers.
*   **Zero-Copy Transport**: 64GB Shared Memory plane.
*   **Agentic Templates**: AI-authored workflow templates that are reviewable and reusable.
*   **Human-in-the-Loop Governance**: Explicit approvals for sensitive or costly actions.

### 1.3 Definitions
| Term | Definition |
| :--- | :--- |
| **Rack** | The linear list of Units (Blades) acting as the primary UI. |
| **Signal Bus** | The virtual patchbay connecting Units. |
| **Centaur** | Rust Host + Python Compute architecture. |
| **Zero-Copy** | Passing pointers via SHM instead of serializing bytes. |
| **Template** | A parameterized workflow graph authored by AI or humans for reuse. |
| **Run** | A single execution of a Template with specific parameters. |
| **Artifact** | A persisted output from a Run (image, tensor, metadata). |
| **Approval Gate** | A mandatory human approval step in a workflow. |
| **HITL** | Human-in-the-loop governance for approvals and policy exceptions. |
| **OPA** | Open Policy Agent, the policy decision point (PDP). |
| **SpiceDB** | Relationship-based authorization service (ReBAC). |

---

## 2. Overall Description

### 2.1 Product Perspective
VORTEX-GEN 3.0 abandons the traditional node-graph "spaghetti" UI in favor of a structured **Vertical Rack**. The Rust Core Engine acts as the translation layer, compiling this linear Rack representation into a computational Dependency Graph (DAG) that is executed by Python Workers. Deployments may be **single-node** (local) or **multi-node** (CPU control plane with remote GPU workers).

### 2.2 Product Functions
*   **F-01: Rack Compilation**: Transforming the Ordered List of Units + Signal Bus Taps into a valid execution graph.
*   **F-02: Zero-Copy Transport**: Passing pointer references between processes.
*   **F-03: Process Isolation**: Sandboxing Python execution.
*   **F-04: Cinematic Feedback**: 60fps WebGL background reflecting generation state.
*   **F-05: Agentic Template Authoring**: AI proposes templates from natural language.
*   **F-06: HITL Enforcement**: Human approval is required for sensitive steps.

### 2.3 User Classes
*   **Creator**: Interacts with the Rack and Signal Bus.
*   **Engineer**: Configures the underlying Python environments.

### 2.4 Operating Environment
*   **Host OS**: Linux (5.15+), macOS (13.0+).
*   **Hardware**: Apple Silicon or NVIDIA GPU (16GB+ RAM recommended).

---

## 3. Specific Requirements

### 3.1 External Interface Requirements
#### 3.1.1 User Interfaces
*   **UI-01**: The System shall serve a Single Page Application (SPA).
*   **UI-02**: The UI shall implement the "Vertical Rack" paradigm (no infinite canvas).
*   **UI-03**: The UI shall use WebGL for the "Cinematic Background" but DOM for the Rack Units.
*   **UI-04**: Port 11188 (Default, served by Core HTTP).

#### 3.1.2 Hardware Interfaces
*   **HW-01**: GPU Interface via PyTorch/CUDA/MPS.
*   **HW-02**: Shared Memory via `/dev/shm`.

#### 3.1.3 Network Interfaces (Port Authority)
*   **Range**: 11000-11999 (Reserved).
*   **Core**: 11188 (HTTP), 11189 (WS).

### 3.2 Functional Requirements

#### 3.2.1 [F-01] Rack-to-Graph Compilation
*   **Description**: Converting the UI state to an execution plan.
*   **Inputs**: Rack State (List of Units, Bus Taps).
*   **Processing**:
    1.  Receive `RackState` from Frontend.
    2.  Identify Dependencies based on Bus Lane usage.
    3.  Construct Topological Sort (Kahn's Algorithm).
    4.  Validate Types (e.g., Image -> Image).
    5.  Schedule Nodes for execution.
*   **Outputs**: `UserGraph` sent to Core Engine.

#### 3.2.2 [F-02] Shared Memory
*   **Description**: Zero-Copy Arrow-based memory plane.
*   **Implementation**: `shm_open` / `mmap` (See Section 3.4).

#### 3.2.3 [F-03] Agentic Workflow & Template Model
*   **Description**: Formalizing AI-authored templates as first-class system artifacts.
*   **Requirements**:
    *   **TEMPLATE-01**: The System shall persist Templates with version history.
    *   **TEMPLATE-02**: Each Template shall declare inputs, outputs, and tool dependencies.
    *   **TEMPLATE-03**: Templates shall be runnable with a bounded parameter schema.
    *   **TEMPLATE-04**: The System shall generate a deterministic Template ID hash.
    *   **TEMPLATE-05**: The System shall allow users to fork Templates without data loss.

#### 3.2.4 [F-04] Human-in-the-Loop Governance (HITL)
*   **Description**: Mandatory human approvals for sensitive or expensive actions.
*   **Approval Triggers**:
    1.  **External Side Effects**: Network writes, outbound API calls, or filesystem writes.
    2.  **High Cost**: GPU-heavy runs above configured thresholds.
    3.  **Data Exposure**: Any access to non-local or classified data.
    4.  **Policy Overrides**: Manual bypass of system policy.
*   **Requirements**:
    *   **HITL-01**: Approval Gates shall be represented explicitly in the workflow graph.
    *   **HITL-02**: Approvals shall be enforced server-side (not UI-only).
    *   **HITL-03**: Each approval shall record user, time, reason, and diff.
    *   **HITL-04**: A rejected approval shall halt the Run and preserve state.
    *   **HITL-05**: Emergency overrides require elevated role and audit entry.

#### 3.2.5 [F-05] Agentic Flow Studio
*   **Description**: Agent-first creation of racks and templates from natural language.
*   **Requirements**:
    *   **AGENT-01**: The system shall expose MCP tool metadata to the agent.
    *   **AGENT-02**: The agent shall generate a complete rack for a given intent.
    *   **AGENT-03**: The agent shall provide rationale for tool selection and connections.
    *   **AGENT-04**: The user shall be able to request edits and receive updated rationale.

### 3.3 Non-Functional Requirements

#### 3.3.1 Performance
*   **PERF-01**: Rack Compilation < 50ms.
*   **PERF-02**: UI Animation (Reorder) at 60fps.

#### 3.3.2 Security
*   **SEC-01**: Workers sandboxed (no network access except localhost).
*   **SEC-02**: Approval Gates shall be immutable once a Run starts.
*   **SEC-03**: Authorization decisions shall be enforced via SpiceDB.
*   **SEC-04**: Policy decisions shall be enforced via OPA.

#### 3.3.3 Configuration Modes
*   **CFG-01**: The system shall support `development_sandbox`, `development_live`, and `production` modes.
*   **CFG-02**: Development modes shall behave like production; only resource limits may differ.
*   **CFG-03**: Mode selection shall be centralized and consistent across services.

---

### 3.4 Data Models

#### 3.4.1 Shared Memory Layout
(unchanged from v9.0.0)

#### 3.4.2 IPC Protocol
*   **Protocol**: Protobuf over Unix Domain Socket (UDS).
*   **Framing**: 4-byte unsigned length prefix (little-endian), followed by protobuf payload.
*   **Schema Source**: `proto/control.proto` and `proto/worker.proto`.
*   **Socket Path**: `/tmp/vortex.sock` (single authoritative path).
*   **Compatibility**: Protocol version negotiated via `WorkerHandshake`.

---
