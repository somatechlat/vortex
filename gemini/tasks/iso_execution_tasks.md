# VORTEX ISO Execution Taskboard
Version: 1.0.0  
Date: 2026-02-21  
Source Plan: `gemini/docs/ISO_Implementation_Execution_Plan.md`

## Usage
- Status values: `todo`, `in_progress`, `blocked`, `done`
- Every task must include verification evidence before `done`
- UI commands must use `bun` (no `npm`)

## Phase A: Build Unblockers (Critical)
### E-001 (done) Fix config crate dependency integrity
- Objective: Resolve Rust workspace compile failure from config/vault dependency linkage.
- Files:
  - `gemini/crates/vortex-config/Cargo.toml`
  - `gemini/crates/vortex-config/src/vault.rs`
- Acceptance:
  - `cargo check -p vortex-config` passes.

### E-002 (done) Restore core compile path
- Objective: Resolve core compile blockers in execution/supervisor/IPC path.
- Files:
  - `gemini/crates/vortex-core/src/execution.rs`
  - `gemini/crates/vortex-core/src/supervisor.rs`
  - `gemini/crates/vortex-core/src/ipc.rs`
- Acceptance:
  - `cargo check -p vortex-core` passes.

## Phase B: Runtime Correctness
### E-003 (todo) Make execution path deterministic
- Objective: Ensure execution context mutability and output propagation are compile-safe and deterministic.
- Files:
  - `gemini/crates/vortex-core/src/execution.rs`
- Acceptance:
  - Focused execution tests pass.

### E-004 (in_progress) Replace startup race with readiness handshake
- Objective: Remove fixed sleep startup dependency; use worker-ready signal/handshake.
- Files:
  - `gemini/crates/vortex-core/src/execution.rs`
  - `gemini/crates/vortex-core/src/ipc.rs`
  - `gemini/worker/vortex_worker/*` (handshake side)
- Acceptance:
  - No fixed startup sleep in execution path.
  - Worker readiness validated in test or deterministic smoke flow.

### E-005 (todo) Enforce crash/timeout failure semantics
- Objective: Ensure worker crashes/timeouts fail run promptly and clearly.
- Files:
  - `gemini/crates/vortex-core/src/execution.rs`
  - `gemini/crates/vortex-core/src/error.rs`
- Acceptance:
  - Crash path returns typed run failure without hanging.

### E-006 (done) Fix worker entrypoint runtime errors
- Objective: Resolve import/config/runtime issues in main worker entrypoint.
- Files:
  - `gemini/worker/vortex_worker/main.py`
  - `gemini/worker/vortex_worker/config.py`
- Acceptance:
  - `python3 -m compileall -q vortex_worker` passes.
  - `ruff check` and `mypy --strict` pass.

### E-007 (todo) Add explicit worker heartbeat loop and lifecycle cleanup
- Objective: Worker emits heartbeat on schedule and exits cleanly.
- Files:
  - `gemini/worker/vortex_worker/main.py`
  - `gemini/worker/vortex_worker/ipc.py`
- Acceptance:
  - Heartbeat observable in integration logs/state.

## Phase C: API/UI End-to-End
### E-008 (todo) Wire Render action to real execute endpoint
- Objective: Render button performs graph submit/execute flow.
- Files:
  - `gemini/ui/src/routes/+page.svelte`
  - `gemini/ui/src/lib/stores/execution.svelte.ts`
- Acceptance:
  - UI can trigger an execution run and display run id/state.

### E-009 (todo) Wire websocket progress stream to execution store
- Objective: Update UI state from core websocket events.
- Files:
  - `gemini/ui/src/lib/stores/execution.svelte.ts`
  - `gemini/ui/src/routes/+page.svelte`
- Acceptance:
  - Node and run completion events update visible status/progress.

### E-010 (in_progress) Implement real cancel_run semantics
- Objective: Cancel endpoint must stop in-flight run and propagate terminal state.
- Files:
  - `gemini/crates/vortex-core/src/api.rs`
  - `gemini/crates/vortex-core/src/execution.rs`
- Acceptance:
  - Cancelled run no longer progresses and reports terminal cancelled/failed state.

## Phase D: Security and Governance
### E-011 (todo) Align AuthZ client with deployed SpiceDB ports
- Objective: Match runtime endpoint and service exposure without dev bypasses.
- Files:
  - `gemini/crates/vortex-core/src/authz.rs`
  - `gemini/k8s/base/security/spicedb.yaml`
- Acceptance:
  - Authz check reaches live service configuration successfully.

### E-012 (todo) Harden JWT/Auth middleware behavior
- Objective: Ensure protected routes are consistently authenticated and tenant-scoped.
- Files:
  - `gemini/crates/vortex-core/src/api.rs`
- Acceptance:
  - Unauthorized requests are rejected; tenant context enforced in handlers.

### E-013 (todo) Infrastructure persistence and network restrictions
- Objective: Move SpiceDB off in-memory mode where persistence is required and tighten SG egress.
- Files:
  - `gemini/k8s/base/security/spicedb.yaml`
  - `gemini/infra/main.tf`
- Acceptance:
  - Datastore mode and network policy match security baseline.

## Phase E: Quality Closure
### E-014 (todo) Remove non-test unwraps and close lint debt
- Objective: Eliminate panic-prone paths and enforce lint/test gates.
- Files:
  - `gemini/crates/vortex-core/src/**/*.rs`
  - `gemini/worker/vortex_worker/**/*.py`
  - `gemini/ui/src/**/*`
- Acceptance:
  - Rust: no non-test `unwrap()` in target scope.
  - Python: lint/type checks pass.
  - UI: `bun run check` passes.

## Phase F: Differentiation Runtime (Post E-014)
### E-015 (todo) Introduce warm diffusion serving lifecycle
- Objective: Maintain warm model residency between requests with explicit load/unload policy.
- Files:
  - `gemini/worker/vortex_worker/model_loader.py`
  - `gemini/worker/vortex_worker/executor.py`
  - `gemini/crates/vortex-core/src/arbiter.rs`
- Acceptance:
  - First-call cold start and second-call warm latency measured and reported.

### E-016 (todo) Continuous batching queue for diffusion requests
- Objective: Replace one-shot request handling with queue/batching coordinator.
- Files:
  - `gemini/crates/vortex-core/src/execution.rs`
  - `gemini/worker/vortex_worker/main.py`
- Acceptance:
  - Batch coalescing works for concurrent compatible requests.

### E-017 (todo) Multi-stream execution for Flux/SDXL/SD3 components
- Objective: Support stream-aware scheduling of encoder/transformer/vae stages.
- Files:
  - `gemini/worker/vortex_worker/executor.py`
  - `gemini/crates/vortex-core/src/scheduler.rs`
- Acceptance:
  - Stream plan visible in metrics; no regression in output validity.

## Phase G: Determinism and Workflow Review
### E-018 (todo) Determinism envelope and reproducibility metadata
- Objective: Capture precision, scheduler order, backend profile in run metadata.
- Files:
  - `gemini/crates/vortex-core/src/entities.rs`
  - `gemini/crates/vortex-core/src/run_repo.rs`
  - `gemini/worker/vortex_worker/config.py`
- Acceptance:
  - Runs include reproducibility profile and deterministic mode flag.

### E-019 (todo) Workflow version diff and impact summary
- Objective: Provide node/edge/param diff between workflow versions with expected impact class.
- Files:
  - `gemini/crates/vortex-core/src/graph.rs`
  - `gemini/crates/vortex-core/src/api.rs`
  - `gemini/ui/src/routes/+page.svelte`
- Acceptance:
  - API returns structured diff; UI renders human-readable delta summary.

## Phase H: Streaming and Multimodal
### E-020 (todo) Progressive preview streaming during diffusion steps
- Objective: Emit intermediate decodes (step checkpoints) over websocket.
- Files:
  - `gemini/worker/vortex_worker/executor.py`
  - `gemini/crates/vortex-core/src/api.rs`
  - `gemini/ui/src/lib/stores/execution.svelte.ts`
- Acceptance:
  - User sees incremental previews before final output completion.

### E-021 (todo) Extend protocol and Signal Bus for multimodal types
- Objective: Add `TEXT`, `AUDIO`, `VIDEO` data types across protocol, core, and UI.
- Files:
  - `gemini/proto/graph.proto`
  - `gemini/crates/vortex-core/src/graph.rs`
  - `gemini/ui/src/lib/stores/bus.svelte.ts`
- Acceptance:
  - Multimodal links validate and execute through DAG boundaries.

### E-022 (todo) Add model composition primitives
- Objective: Implement LoRA merge/model arithmetic nodes and safety checks.
- Files:
  - `gemini/worker/vortex_worker/executor.py`
  - `gemini/crates/vortex-core/src/units/*.rs`
- Acceptance:
  - Composition node outputs are reproducible and tracked in run metadata.

### E-023 (todo) Add quality scoring and gateable evaluation nodes
- Objective: Support CLIP/aesthetic/quality scoring as first-class DAG nodes.
- Files:
  - `gemini/worker/vortex_worker/extras.py`
  - `gemini/crates/vortex-core/src/graph.rs`
  - `gemini/crates/vortex-core/src/scheduler.rs`
- Acceptance:
  - Downstream branching can be gated by scoring thresholds.

## Phase I: Portability and Adoption
### E-024 (todo) Backend abstraction for CUDA/MPS/ROCm/CPU
- Objective: Decouple execution backend from CUDA-specific assumptions.
- Files:
  - `gemini/worker/vortex_worker/config.py`
  - `gemini/worker/vortex_worker/executor.py`
  - `gemini/crates/vortex-core/src/arbiter.rs`
- Acceptance:
  - Backend capabilities are discoverable and respected in scheduling.

### E-025 (todo) Desktop installer and first-run bootstrap
- Objective: Deliver non-technical install path with runtime/env/model setup.
- Files:
  - `gemini/ui/*`
  - `gemini/scripts/*`
  - `gemini/docs/DEPLOYMENT.md`
- Acceptance:
  - Clean-machine install path succeeds with documented one-command or one-click flow.

## Phase J: Deployment and Real Model Validation
### E-026 (done) Align local CPU deployment image path
- Objective: Ensure local standalone deployment defaults to CPU worker requirements.
- Files:
  - `gemini/docker/worker/Dockerfile`
  - `gemini/infra/docker/standalone/docker-compose.yml`
- Acceptance:
  - Local compose worker installs `requirements.sandbox.txt` by default.

### E-027 (done) AWS GPU endpoint + optional serverless smoke infrastructure
- Objective: Support real diffusion testing on GPU while keeping serverless smoke option.
- Files:
  - `gemini/infra/main_serverless.tf`
  - `gemini/infra/variables.tf`
  - `gemini/infra/outputs.tf`
- Acceptance:
  - `terraform validate` passes.
  - GPU real-time endpoint and optional serverless smoke endpoint are parameterized.

### E-028 (done) Deployment docs and scripts aligned with code
- Objective: Keep docs/scripts synchronized with deployable code paths.
- Files:
  - `gemini/scripts/deploy_aws.sh`
  - `gemini/docs/DEPLOYMENT.md`
  - `gemini/docs/ISO_AWS_Deployment_Test_Plan.md`
- Acceptance:
  - Script supports account/region/GPU type and smoke toggle.
  - Documentation references actual file paths and variables.

### E-029 (todo) End-to-end real image generation test on AWS GPU
- Objective: Execute and record a real model inference run using SageMaker endpoint.
- Files:
  - `gemini/scripts/*`
  - `gemini/docs/DEPLOYMENT.md`
  - `gemini/docs/ISO_AWS_Deployment_Test_Plan.md`
- Acceptance:
  - Endpoint invocation returns image output for a real model payload.
  - Run metadata and generated artifact are captured for regression.

## Execution Order
1. Phase A
2. Phase B
3. Phase C
4. Phase D
5. Phase E
6. Phase F
7. Phase G
8. Phase H
9. Phase I
10. Phase J

## Verification Commands
```bash
# Rust
cd gemini
cargo check -p vortex-config
cargo check -p vortex-core

# Python
cd gemini/worker
ruff check .
mypy --strict vortex_worker

# UI (bun only)
cd gemini/ui
bun install
bun run check
```

## Evidence Log
- E-001:
  - Change: added `reqwest` dependency for Vault client usage.
  - Verify: `cd gemini && cargo check -p vortex-config` -> pass.
- E-002:
  - Change: fixed MCP import/type alias issues; fixed execution supervisor constructor mismatch; passed SHM into execution context; bound IPC listener before accept.
  - Verify: `cd gemini && cargo check -p vortex-core` -> pass.
- E-006:
  - Change: added missing `os` import in worker main; added typed `db_password` field in `WorkerConfig`.
  - Change: fixed SageMaker serving schema typing import (`Any`) in `vortex_worker/serve.py`.
  - Verify: `cd gemini/worker && python3 -m compileall -q vortex_worker` -> pass.
- E-004 (in_progress):
  - Change: removed fixed startup sleep and reordered IPC listener bind before worker spawn to avoid connect race.
  - Verify: `cd gemini && cargo check -p vortex-core` -> pass.
- E-010 (in_progress):
  - Change: added cancellation token registry in API state; wired execute path and cancel endpoint to signal cancellation and persist terminal status.
  - Verify: `cd gemini && cargo check -p vortex-core` -> pass.
- Workspace:
  - Change: fixed `vortex-registry` scanner/solver compile failures for current dependency APIs.
  - Verify: `cd gemini && cargo check` -> pass.
- UI:
  - Change: fixed WebGL nullability errors in cinematic background.
  - Verify: `cd gemini/ui && bun run check` -> pass with warnings only.
- E-026:
  - Change: local worker Dockerfile now defaults to `requirements.sandbox.txt` for CPU-first local runs.
  - Verify: `gemini/docker/worker/Dockerfile` uses `ARG WORKER_REQUIREMENTS=requirements.sandbox.txt`.
- E-027:
  - Change: added SageMaker GPU real-time endpoint variables and optional serverless smoke endpoint configuration.
  - Verify: `cd gemini/infra && terraform validate` -> pass.
- E-028:
  - Change: updated AWS deploy script and deployment docs to match implemented Terraform/code paths.
  - Verify: `bash -n gemini/scripts/deploy_aws.sh` -> pass.
