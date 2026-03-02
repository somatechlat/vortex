# VORTEX ISO Implementation Execution Plan
Document ID: VTX-PLAN-ISO-001  
Version: 1.0.0  
Date: 2026-02-21  
Status: Approved for Execution  
Owner: Engineering

## 1. Purpose
Define a single, execution-ready, ISO-aligned implementation plan that merges current architecture intent, real repository state, and delivery tasks.

## 2. Scope
In scope:
- Rust Core Engine (`gemini/crates/vortex-core`)
- Rust Config (`gemini/crates/vortex-config`)
- Python Worker (`gemini/worker/vortex_worker`)
- UI (`gemini/ui`)
- Infrastructure and platform manifests (`gemini/k8s`, `gemini/infra`)
- Delivery tasks (`gemini/tasks`)

Out of scope:
- New product features not needed for execution reliability
- Full redesign of SRS corpus

## 3. Normative References
- `gemini/docs/specs/00_master_srs.md`
- `gemini/docs/specs/01_core_engine_srs.md`
- `gemini/docs/specs/02_frontend_ui_srs.md`
- `gemini/docs/specs/03_compute_fabric_srs.md`
- `gemini/docs/specs/04_registry_srs.md`
- `gemini/docs/specs/09_agent_journey_creation_srs.md`
- `gemini/docs/design/00_system_design.md`
- `gemini/docs/coding_rules.md`

## 4. Definitions
- Core: Rust control plane API/scheduler/execution.
- Worker: Python compute process handling node execution.
- SHM: Shared memory arena for tensor references.
- RTM: Requirements traceability matrix.

## 5. Current Baseline (Observed)
### 5.1 Build and Runtime Baseline
- Rust workspace currently blocked by missing dependency linkage in config crate.
- Worker entrypoint contains runtime import/attribute issues.
- UI contains type and accessibility issues; execution path not connected to backend.

### 5.2 Integration Baseline
- Graph execution flow is partially wired.
- AuthN extractor exists but full auth/authz integration is incomplete.
- Cancel endpoint exists as API surface but behavior is not implemented.

### 5.3 Platform Baseline
- SpiceDB deployment is configured in memory mode.
- Worker security group egress is broad/open.

## 6. Target System Design (Execution-Focused)
### 6.1 Design Objective
Deliver a stable, testable vertical slice where:
1. UI can submit/execute/cancel runs.
2. Core can schedule/dispatch safely and fail fast on worker faults.
3. Worker can handshake, execute, heartbeat, and return results reliably.
4. Auth and authz are enforced without unsafe bypasses.

### 6.2 Design Constraints
- No `unwrap()` outside tests.
- No placeholder handlers for core execution APIs.
- JavaScript toolchain policy: use `bun`; do not use `npm`.
- Changes must include verification commands and expected outcomes.

## 7. Requirements Baseline for This Plan
R-01: Workspace builds (`cargo check`) without dependency failures.  
R-02: Core execution path is compile-clean and functionally complete for single-run execution.  
R-03: Worker startup path is runtime-safe and heartbeat-capable.  
R-04: UI render action triggers real API execution and receives run status updates.  
R-05: Cancel endpoint terminates active execution deterministically.  
R-06: AuthN and AuthZ checks are enforced in runtime-compatible configuration.  
R-07: Infra defaults do not use in-memory authz datastore for persistence-required environments.  
R-08: Delivery includes task-level acceptance criteria and verification evidence.
R-09: Diffusion serving supports warm residency and low-latency second-call execution.  
R-10: FluxDev/SDXL/SD3 execution supports multi-stream component scheduling.  
R-11: Reproducibility includes numerical determinism policy and capability signaling.  
R-12: Workflow versioning includes structural and parameter diff with impact summary.  
R-13: Diffusion execution supports progressive preview streaming at intermediate steps.  
R-14: Type system supports multimodal pipeline composition (`TEXT`, `AUDIO`, `VIDEO`).  
R-15: Model composition supports weight-level arithmetic and merge algorithms.  
R-16: Quality evaluation supports scoring nodes and policy-gated downstream branching.  
R-17: Worker backend abstraction supports non-NVIDIA accelerators.  
R-18: Installer UX supports one-command or one-click desktop onboarding.

## 7.1 Strategic Gap Addendum (Competitive Positioning)
The following are treated as first-class requirements, not optional research:
1. Warm diffusion serving (continuous batching + VRAM residency).
2. Native multi-stream execution for modern diffusion stacks.
3. Deterministic generation with auditable reproducibility envelope.
4. Workflow diffing/version review for team collaboration.
5. True progressive result streaming integrated into core transport.
6. Multimodal pipeline composition beyond image-only lanes.
7. Weight-level model composition primitives.
8. Built-in evaluation and quality scoring loop.
9. Accelerator-agnostic backend abstraction.
10. Install/adoption-first distribution path.

## 8. Verification Strategy
- Rust: `cargo check`, focused `cargo test` suites.
- Python: `ruff check`, `mypy --strict`, targeted tests.
- UI: `bun run check`, `bun run test` (where available).
- Integration: API execute + websocket progress + cancel flow smoke tests.

## 9. Risk Register
- RR-01: Architecture drift between documented design and live code paths.
  - Mitigation: enforce task-level file references and RTM.
- RR-02: Partial implementation creates false-positive “done” status.
  - Mitigation: hard acceptance gates per task.
- RR-03: Runtime regressions from cross-stack interfaces.
  - Mitigation: integration test pack before merge.

## 10. Change Control
- Every task completion updates:
  - task status,
  - verification evidence,
  - impacted requirement IDs.
- No task is closed without reproducible command output recorded in PR/commit notes.

## 11. RTM (Plan-Level)
| Requirement | Primary Tasks |
|---|---|
| R-01 | E-001, E-002 |
| R-02 | E-003, E-004, E-005 |
| R-03 | E-006, E-007 |
| R-04 | E-008, E-009 |
| R-05 | E-010 |
| R-06 | E-011, E-012 |
| R-07 | E-013 |
| R-08 | E-014 |
| R-09 | E-015, E-016 |
| R-10 | E-017 |
| R-11 | E-018 |
| R-12 | E-019 |
| R-13 | E-020 |
| R-14 | E-021 |
| R-15 | E-022 |
| R-16 | E-023 |
| R-17 | E-024 |
| R-18 | E-025 |

## 12. Execution Readiness Gate
Proceed with implementation only when:
1. Team agrees to task priorities and owners in `gemini/tasks/iso_execution_tasks.md`.
2. `bun` policy is enforced for UI workflows.
3. First milestone (build unblocked) is complete.

## 13. Post-Stabilization Strategic Program
### 13.1 Program Goal
After E-001..E-014, execute E-015..E-025 as the product differentiation track.

### 13.2 Phase F: Diffusion Serving and Runtime Advancements
- Warm model server lifecycle for UNet/TextEncoder/VAE residency.
- Continuous batching queue and request coalescing policy.
- CUDA stream orchestration for parallelizable subgraphs.
- Progressive decode and websocket frame streaming.

### 13.3 Phase G: Reproducibility and Team Workflows
- Determinism profile negotiation by backend/model/scheduler.
- Workflow semantic diff engine with node/param delta report.
- Review mode for asynchronous collaboration and change proposals.

### 13.4 Phase H: Multimodal and Evaluation Expansion
- Protocol and UI lane expansion: `TEXT`, `AUDIO`, `VIDEO`.
- Cross-modal node library and routing validation.
- Quality scoring nodes and policy gates for automated filtering.

### 13.5 Phase I: Portability and Adoption
- Backend abstraction layer for CUDA/MPS/ROCm/CPU execution modes.
- Desktop installer/wrapper with environment bootstrap and model setup.

### 13.6 Program Acceptance
Program is complete only when each strategic requirement (R-09..R-18) has:
1. Implemented code path,
2. integration test evidence,
3. operational doc,
4. release note and migration guidance.
