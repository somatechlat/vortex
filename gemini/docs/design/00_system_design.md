# VORTEX System Design (Derived from SRS)

> **Purpose**: Consolidated system design derived from the SRS set. This document summarizes architecture, responsibilities, interfaces, and operational modes without replacing the SRS.
>
> **Sources**:
> - `gemini/docs/specs/00_master_srs.md`
> - `gemini/docs/specs/01_core_engine_srs.md`
> - `gemini/docs/specs/02_frontend_ui_srs.md`
> - `gemini/docs/specs/03_compute_fabric_srs.md`
> - `gemini/docs/specs/04_registry_srs.md`
> - `gemini/docs/specs/05_ui_screens_spec.md`
> - `gemini/docs/specs/06_environment_config_spec.md`
> - `gemini/docs/specs/06_enterprise_infrastructure_srs.md`
> - `gemini/docs/specs/07_data_flow_spec.md`
> - `gemini/docs/specs/08_workspace_tools_spec.md`

---

## 1. Objective

VORTEX is an agentic workflow engine and MCP-style tool box for generative AI. It provides a rack-first UI that compiles to a DAG, executes on a Python worker fabric with zero-copy transport, and enforces policy, authorization, and human approvals.

---

## 2. Architecture Overview

### 2.1 Control Plane (Rust)
- Graph validation and DAG compilation.
- Incremental caching (Salsa) and scheduling.
- IPC gateway and SHM arena coordination.
- Policy and approval gate enforcement.

### 2.2 Compute Fabric (Python)
- Worker lifecycle and execution loop.
- Arrow + DLPack zero-copy tensor bridge.
- Sandbox and restricted execution environment.
- Remote worker registration in cluster mode.

### 2.3 Frontend UI (Svelte)
- Vertical rack (no infinite canvas).
- Signal bus routing and parameter inspection.
- Kernel AI for agent-first flow creation.
- Approval gate and review UX.

### 2.4 Registry System
- Tool and dependency metadata.
- Static security scanning and lockfile reproducibility.
- Risk flags and approval requirements per tool.

---

## 3. Agentic Flow Studio

- Agent consumes MCP tool metadata and builds a complete rack from intent.
- Agent provides rationale for tool selection and parameters.
- User edits are supported with updated rationale.
- Approval gates are enforced server-side before execution.

---

## 4. Data Flow

- UI emits rack state to Core.
- Core compiles rack to DAG and dispatches jobs.
- Workers execute jobs, reading/writing tensors via SHM.
- Progress and artifacts are streamed back to UI.

---

## 5. Security and Governance

- AuthN: Keycloak (OIDC).
- AuthZ: SpiceDB (relationship-based permissions).
- Policy: OPA (policy decision point for gating).
- Secrets: Vault only (no secrets in env vars).
- HITL: Approval gates with audit trail.

---

## 6. Deployment Modes

- **Local Mode**: Core and Worker on one machine (UDS + SHM).
- **Cluster Mode**: Core on CPU nodes; Workers on GPU nodes via secure transport.
- Configuration modes: `development_sandbox`, `development_live`, `production`.

---

## 7. Interfaces

- API: HTTP + WebSocket.
- IPC: Protobuf with length-prefix framing.
- SHM: POSIX shared memory with Arrow layout.

---

## 8. Traceability

All requirements are defined in the SRS set. This design document is a summary and does not supersede the SRS.
