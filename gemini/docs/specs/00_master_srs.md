# Software Requirements Specification (SRS): Master System Architecture
**Project**: VORTEX-GEN 3.0 "Centaur"
**Module**: Architecture & Core Constraints
**Version**: 9.1.0 (ISO Standard)
**Date**: 2026-01-06
**Standard**: ISO/IEC 29148:2018

---

## 1. Introduction

### 1.1 Purpose
This SRS defines the architectural standards, global constraints, and interface boundaries for VORTEX-GEN 3.0. It documents the "Centaur" pattern (Rust Host + Python Compute) and the "Vortex Flow Stream" UI paradigm (Vertical Rack).

### 1.2 Scope
VORTEX-GEN 3.0 is a Local-First, Hybrid AI Execution Environment.
**The software facilitates**:
*   **Rack-Based Orchestration**: Linear, vertical organization of generative steps.
*   **Graph Compilation**: converting the linear Rack state into an optimized DAG for execution.
*   **Compute Virtualization**: Isolated Python workers.
*   **Zero-Copy Transport**: 64GB Shared Memory plane.

### 1.3 Definitions
| Term | Definition |
| :--- | :--- |
| **Rack** | The linear list of Units (Blades) acting as the primary UI. |
| **Signal Bus** | The virtual patchbay connecting Units. |
| **Centaur** | Rust Host + Python Compute architecture. |
| **Zero-Copy** | Passing pointers via SHM instead of serializing bytes. |

---

## 2. Overall Description

### 2.1 Product Perspective
VORTEX-GEN 3.0 abandons the traditional node-graph "spaghetti" UI in favor of a structured **Vertical Rack**. The Rust Core Engine acts as the translation layer, compiling this linear Rack representation into a computational Dependency Graph (DAG) that is executed by Python Workers.

### 2.2 Product Functions
*   **F-01: Rack Compilation**: Transforming the Ordered List of Units + Signal Bus Taps into a valid execution graph.
*   **F-02: Zero-Copy Transport**: Passing pointer references between processes.
*   **F-03: Process Isolation**: Sandboxing Python execution.
*   **F-04: Cinematic Feedback**: 60fps WebGL background reflecting generation state.

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
*   **UI-04**: Port 11188 (Default).

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

### 3.3 Non-Functional Requirements

#### 3.3.1 Performance
*   **PERF-01**: Rack Compilation < 50ms.
*   **PERF-02**: UI Animation (Reorder) at 60fps.

#### 3.3.2 Security
*   **SEC-01**: Workers sandboxed (no network access except localhost).

---

### 3.4 Data Models

#### 3.4.1 Shared Memory Layout
(unchanged from v9.0.0)

#### 3.4.2 IPC Protocol
(unchanged from v9.0.0)

---
