# Software Requirements Specification (SRS): Agent Journey of Creation
**Project**: VORTEX-GEN 3.0 "Centaur"  
**Module**: MCP Agent Creation Journey  
**Version**: 1.0.0  
**Date**: 2026-02-21  
**Standard**: ISO/IEC 29148:2018

---

## 1. Introduction

### 1.1 Purpose
This SRS defines the end-to-end "instruction to commercial asset" journey executed by VORTEX through MCP-aware agent tooling. It formalizes the required behavior for converting a user instruction into validated workflow execution and deliverable image outputs.

### 1.2 Scope
In scope:
- Natural-language instruction intake
- MCP tool discovery and planning
- Rack and DAG construction
- Policy and approval gating
- GPU execution path selection
- Artifact return and run traceability

Out of scope:
- Final legal approval workflows external to VORTEX
- Billing policy and chargeback implementation details

### 1.3 Definitions
| Term | Definition |
| :--- | :--- |
| Agent Journey | End-to-end process from instruction to generated commercial artifact. |
| MCP | Model Context Protocol interface used for tools, planning, and execution orchestration. |
| Creation Plan | Structured plan of tools, parameters, risks, and expected outputs. |
| Approval Gate | Human decision point required before sensitive or costly actions. |
| Artifact Package | Generated outputs plus metadata and reproducibility context. |

---

## 2. Overall Description

### 2.1 Product Perspective
The Agent Journey runs on the VORTEX architecture:
- CPU control plane for planning, policy, orchestration, and status.
- GPU inference plane for model execution and rendering.
- Shared execution trace through run IDs, workflow versions, and output metadata.

### 2.2 User Classes
- Creator: submits instructions and accepts/rejects agent proposals.
- Operator: monitors run health, cost, and policy compliance.
- Reviewer: validates output fitness for commercial use.

### 2.3 Assumptions and Dependencies
- MCP tool metadata is available and queryable.
- Core API and worker runtime are reachable.
- GPU endpoint is available for real generation tests.
- Bun is used for UI workflow execution.

---

## 3. External Interface Requirements

### 3.1 User Interface
- AJ-UI-001: System shall provide a single instruction input interface for creator prompts.
- AJ-UI-002: System shall display agent-generated creation plan before execution.
- AJ-UI-003: System shall show live run status and final artifact package.

### 3.2 MCP Interface
- AJ-MCP-001: System shall expose tool discovery metadata (inputs, outputs, constraints, cost class).
- AJ-MCP-002: System shall support plan generation and plan revision through MCP operations.
- AJ-MCP-003: System shall return structured execution results with run_id and artifact references.

### 3.3 API Interface
- AJ-API-001: System shall expose instruction submission endpoint.
- AJ-API-002: System shall expose workflow execute and cancel endpoints.
- AJ-API-003: System shall expose run status and artifact retrieval endpoints.

---

## 4. Functional Requirements

### 4.1 Instruction Intake and Normalization
- AJ-F-001: System shall accept free-form user instruction text.
- AJ-F-002: System shall normalize instruction into intent, constraints, and quality targets.
- AJ-F-003: System shall persist normalized request with deterministic request ID.

### 4.2 MCP Planning
- AJ-F-004: Agent shall fetch MCP tool metadata before creating a plan.
- AJ-F-005: Agent shall produce a Creation Plan containing:
  - selected tools and ordering,
  - required parameters and defaults,
  - estimated cost/risk class,
  - expected outputs.
- AJ-F-006: Agent shall provide rationale for each selected tool.
- AJ-F-007: User edits shall trigger plan regeneration without data loss.

### 4.3 Workflow Materialization
- AJ-F-008: System shall convert Creation Plan to Rack representation.
- AJ-F-009: System shall compile Rack to DAG and validate node/link types.
- AJ-F-010: System shall version the workflow and record structural diff on update.

### 4.4 Policy and Approval
- AJ-F-011: System shall evaluate policy rules before execution.
- AJ-F-012: System shall enforce approval gates for high-cost or sensitive runs.
- AJ-F-013: Rejected approvals shall terminate run start and preserve plan context.

### 4.5 Execution
- AJ-F-014: System shall dispatch inference workloads to GPU runtime in test/prod profiles.
- AJ-F-015: System shall stream progress events and terminal status for each run.
- AJ-F-016: System shall support run cancellation and terminal cancellation state.

### 4.6 Artifact Packaging
- AJ-F-017: System shall return generated image artifacts and associated metadata.
- AJ-F-018: Metadata shall include prompt, seed, model ID, run ID, workflow version, and timestamp.
- AJ-F-019: System shall provide reproducibility envelope fields when available.

### 4.7 Failure Handling
- AJ-F-020: System shall return typed error codes for planning, policy, execution, and artifact failures.
- AJ-F-021: System shall preserve partial state for troubleshooting and replay.

---

## 5. Non-Functional Requirements

### 5.1 Performance
- AJ-NF-001: Plan generation shall complete within 3 seconds for standard prompts.
- AJ-NF-002: First execution status event shall be emitted within 2 seconds of run acceptance.
- AJ-NF-003: Second request to warm model path should exhibit lower latency than first request (same model/profile).

### 5.2 Security and Compliance
- AJ-NF-004: Secrets shall not be embedded in client payloads.
- AJ-NF-005: Authorization checks shall be enforced server-side for all run operations.
- AJ-NF-006: Audit logs shall capture plan approval, execution start, and artifact delivery events.

### 5.3 Reliability
- AJ-NF-007: System shall provide deterministic run states: `queued`, `running`, `completed`, `failed`, `cancelled`.
- AJ-NF-008: Retriable failure classes shall be explicitly marked in API responses.

### 5.4 Portability
- AJ-NF-009: CPU development mode and GPU execution mode shall preserve the same logical workflow contract.

---

## 6. Agent Journey Flows

### 6.1 Primary Flow (Instruction to Image)
1. User submits instruction.
2. Agent discovers tools via MCP and generates Creation Plan.
3. User reviews/edits and approves plan.
4. System compiles plan to executable workflow.
5. Policy checks pass and run starts.
6. GPU endpoint executes generation.
7. System returns artifact package and run summary.

### 6.2 Alternate Flow A (Policy Gate Rejection)
1. Policy check identifies gated action.
2. User rejects approval.
3. System terminates pre-run state with clear reason.

### 6.3 Alternate Flow B (Execution Failure)
1. Run fails during execution.
2. System returns typed failure with error class.
3. User receives recovery options (retry/edit plan).

---

## 7. Data Requirements

### 7.1 Creation Request
Minimum fields:
- request_id
- user_id
- raw_instruction
- normalized_intent
- timestamp

### 7.2 Creation Plan
Minimum fields:
- plan_id
- request_id
- selected_tools[]
- tool_rationale[]
- risk_class
- estimated_cost
- workflow_version

### 7.3 Artifact Package
Minimum fields:
- run_id
- artifacts[]
- model_id
- prompt
- seed
- created_at

---

## 8. Verification and Acceptance Criteria

- AJ-AC-001: Given instruction input, system produces Creation Plan with tool rationale.
- AJ-AC-002: Plan approval results in executable workflow and run start.
- AJ-AC-003: Real GPU run returns at least one image artifact.
- AJ-AC-004: Run metadata contains run_id, seed, model identifier, and workflow version.
- AJ-AC-005: Rejected approval prevents run execution.
- AJ-AC-006: Cancel operation transitions run to `cancelled` terminal state.

---

## 9. Traceability Matrix

| Requirement | Validation Method |
| :--- | :--- |
| AJ-F-001..AJ-F-003 | API tests for request normalization and persistence |
| AJ-F-004..AJ-F-007 | MCP planning integration tests |
| AJ-F-008..AJ-F-010 | workflow compile/diff tests |
| AJ-F-011..AJ-F-013 | policy and approval gate tests |
| AJ-F-014..AJ-F-016 | GPU execution and cancellation tests |
| AJ-F-017..AJ-F-019 | artifact and metadata verification tests |
| AJ-F-020..AJ-F-021 | failure-path integration tests |
| AJ-NF-001..AJ-NF-009 | performance/security/reliability test suites |

---

## 10. Implementation References
- `gemini/crates/vortex-core/src/api.rs`
- `gemini/crates/vortex-core/src/execution.rs`
- `gemini/crates/vortex-core/src/scheduler.rs`
- `gemini/worker/vortex_worker/main.py`
- `gemini/worker/vortex_worker/executor.py`
- `gemini/worker/vortex_worker/extras.py`
- `gemini/docs/DEPLOYMENT_TEST_RUNBOOK_REAL.md`
