# VORTEX Workflow Studio ("VORTEX") — Software Requirements Specification (SRS)

**Standard alignment:** ISO/IEC/IEEE 29148 (requirements information items + quality guidance)  
**Product goal:** Build a modern, production-grade successor to ComfyUI for node/graph-based generative workflows—API-first, reproducible, extensible, and safe—usable locally and at enterprise scale.

---

## 0. Document Control
### 0.1 Version History
| Version | Date       | Author      | Notes                                   |
| ------- | ---------- | ----------- | --------------------------------------- |
| 0.1     | 2026-01-06 | YACHAQ + AI | Initial draft from conversation context |
| 1.0     | 2026-01-06 | YACHAQ + AI | Detailed, engineer-ready SRS            |

### 0.2 Status
Draft — Ready for implementation kickoff.

### 0.3 Intended Audience
- Backend engineers (API, queue, workers, storage)
- Frontend engineers (graph editor, run UI)
- ML/platform engineers (model registry, GPU execution)
- DevOps/SRE (deployment, observability, security)
- QA (test plans + verification matrix)
- Security, Performance, UX

---

## 1. Scope
### 1.1 Product Name
**VORTEX Workflow Studio** (“VORTEX”).

### 1.2 Problem Statement
Users need a node/graph-based workflow system to build, run, and iterate generative pipelines (image/video/audio/text, post-processing, upscaling, control nets, LoRA, schedulers) with:
- reproducibility and artifact lineage
- fast iteration (caching + partial re-execution)
- safe extensibility (plugins/custom nodes)
- scalable execution (local → distributed GPU workers)
- robust API/WebSocket automation

### 1.3 In-Scope
- Graph editor UI (Lit Web Components)
- Workflow definition format (VTXW JSON) + Comfy import/export
- Execution engine (node runtime, scheduling, caching)
- Queue/run management
- Artifact storage + metadata + lineage
- Model registry integration (local and remote)
- Plugin system with policies
- HTTP API + WebSocket events (Django + Django Ninja)
- Authentication/authorization (server mode)
- Deployment modes: local single-user + server/cluster

### 1.4 Out-of-Scope (v1)
- Training/fine-tuning pipelines (beyond calling external trainers)
- Full SaaS billing/marketplace
- Mobile client

### 1.5 Success Criteria (Definition of Done)
- Import a non-trivial ComfyUI workflow and run it with clear compatibility notes.
- Runs reproducible via stored workflow + inputs + model versions + seeds.
- Plugins add nodes without core changes; safety controls enforced.
- Queue supports concurrency, cancellation, retries, progress streaming.
- Local mode install <10 minutes; server mode deployable on Kubernetes.

---

## 2. References
- ISO/IEC/IEEE 29148
- VORTEX UI SRS (UI/UX specification)
- Internal architecture direction: event-driven microservices with asynchronous pub/sub; HTTP+WS; optional Protobuf/gRPC internally

*(ISO style for clarity; not enforcing ISO compliance.)*

---

## 3. Definitions, Acronyms, Abbreviations
- **Workflow**: Directed graph of nodes (cycles only via explicit loop operators)
- **Node**: Unit of computation with typed inputs/outputs
- **Run**: Single execution of a workflow with concrete inputs + parameters
- **Artifact**: Produced output (image/video/audio/text, intermediate tensors, logs)
- **Prompt**: Run request payload (workflow reference + input overrides)
- **Plugin**: External package adding nodes, UI panels, integrations
- **Cache**: Store of node outputs keyed by deterministic hash of inputs + code + model versions
- **VTXW**: VORTEX workflow JSON schema (versioned)
- **Workspace**: Isolation domain for workflows/runs/artifacts/models

---

## 4. Overall Description
### 4.1 Product Perspective
- Replacement/improvement for Comfy-style node workflows.
- Supports Local Mode (browser or optional desktop shell) and Server Mode (multi-user, distributed workers).
- UI built with Lit 3.x Web Components (custom elements, shadow DOM, Lit reactive controllers).
- API built with Django + Django Ninja; data layer with Django ORM (models in `admin/<app>/models.py` following `AgentSkin` pattern).

### 4.2 Users and User Classes
- Creator: builds workflows, runs and iterates
- Operator: manages workers/models, monitors health
- Developer: writes plugins/custom nodes
- Admin: manages auth, quotas, policies

### 4.3 Operating Environment
- Local (Desktop/Web): Windows, Linux, macOS. UI in browser; optional Tauri desktop shell.
- Server: Linux server running API/UI and orchestration on CPU-only hosts.
- Workers: GPU or CPU hosts/pods.
- Deployment targets: local single-user bundle; server multi-user (Docker Compose/Helm/Kubernetes).

### 4.4 Design and Implementation Constraints
- Deterministic, versioned workflow execution wherever feasible.
- Backward compatibility tooling for Comfy workflows.
- Plugin execution controlled by security policies and sandbox levels.
- All user-facing strings i18n-ready via centralized message catalog (`admin.common.messages` + `get_message`); no hardcoded UI/API strings.
- UI components must use Lit; no Alpine.js. API must use Django + Django Ninja. DB models must use Django ORM.

### 4.5 Assumptions and Dependencies
- ML backend: PyTorch initially.
- Object storage: local FS or S3-compatible for artifacts.
- Auth provider for server mode (API keys and/or OIDC).
- Control plane CPU-only; workers may require NVIDIA runtime for GPU tasks.

---

## 5. Product Capabilities (High-Level)
- Graph editor UI and workflow authoring/versioning.
- Execution engine with queue, caching, partial re-execution, progress streaming.
- Artifact storage, metadata, lineage, rerun.
- Model registry with pinning and diagnostics.
- Plugin system with policies and audit.
- API + WebSocket automation.

---

## 6. External Interface Requirements
### 6.1 User Interface (UI)
**Web-first UI (Lit Web Components)**
- **VTX-FR-UI-000/000a**: Web app; server serves client (single origin) with option to host separately.
- Graph editor: pan/zoom, palette, multi-select/group/align/comment, typed validation, Problems panel.
- Run controls: queue/run now/cancel/rerun; per-node state; live progress via WebSocket.
- Artifact browser: filters (workflow/date/status/tags), download/view metadata, rerun from artifact.
- IDE Mode: file explorer, workflow tabs, console/logs, node docs/SDK hints, command palette, keyboard-first.
- Desktop/Tauri (optional): permissioned local FS access, offline mode.
- Settings/Admin: model locations/registries, plugin management, RBAC surfaces.
- A11y: keyboard-first primary flows; visible focus; non-color-only status.
- i18n: all user-facing strings via `admin.common.messages`/`get_message`; no inline literals.

### 6.2 HTTP API (Django + Django Ninja)
- Auth: API keys and/or OAuth2/OIDC (server mode); local mode may be unauthenticated.
- Endpoints (v1):
  - `POST /v1/runs` submit run → `run_id`.
  - `GET /v1/runs/{run_id}` status/summary.
  - `POST /v1/runs/{run_id}/cancel`.
  - `GET /v1/queue` queue state.
  - `POST /v1/workflows`; `GET/PUT /v1/workflows/{id}`.
  - `GET /v1/artifacts/{id}`; `GET /v1/runs/{run_id}/artifacts`.
  - `GET /v1/models`.
  - `GET /v1/plugins`; `POST /v1/plugins/install`; `POST /v1/plugins/{id}/enable|disable`.
- Error contract: `{code, message, details, request_id}` (i18n via message catalog). Backward compatible within `/v1`.

### 6.3 WebSocket
- `GET /v1/ws` global or per-run subscription.
- Events: `run.queued`, `run.started`, `node.started`, `node.progress`, `node.cached`, `node.completed`, `artifact.created`, `run.completed`, `run.failed`, `run.canceled`.
- Payload: `{event_type, ts, run_id, node_id?, payload, request_id}`.

### 6.4 Data Interfaces / File Formats
- VTXW JSON: versioned; nodes `{id,type,inputs,ui{x,y},meta}`, edges `{from{node,port},to{node,port}}`, workflow meta, compatibility info.
- Comfy import/export: structured compatibility report (missing/substituted nodes, parameter mappings).
- Artifact metadata embedding: workflow/run metadata in PNG/WebP or sidecar JSON; rerunnable snapshot.

---

## 7. System Features (Functional Requirements)
### 7.1 Workflow Authoring
- CRUD, duplicate, templates; versioning with immutable history; parameter promotion to workflow inputs.

### 7.2 Node Model
- Node schema: `type_id`, typed inputs/outputs, param schema, determinism flag, resource hints (CPU/GPU/VRAM/concurrency).
- Port types at minimum: Image, Mask, Latent, Tensor, Text, Number, Bool, JSON, File, ModelRef. Typed validation with adapters.

### 7.3 Execution Engine
- Run = workflow version + inputs + runtime options (seed/priority/tags/backend/cache policy).
- Topological scheduling; parallel branches; cancellation safe points.
- Cache key includes node type/version, params, upstream hashes, model versions, runtime opts; policy use/invalidate; retries per-node/run.
- Failure: mark failed, emit error summary, stop dependents; preserve logs/intermediates (policy).

### 7.4 Queue
- Priority + FIFO; enqueue/dequeue/peek/cancel/reorder/pause/resume; multi-worker; queue lag metrics.

### 7.5 Worker Fleet
- Workers register capabilities (GPU/VRAM/models/plugins); scheduler matches constraints; stream progress/events.

### 7.6 Artifacts & Lineage
- Immutable run record; content-addressed artifacts + stable IDs; lineage graph; storage backends: local FS, S3-compatible.

### 7.7 Model Management
- Registry: name/type/version/location/checksum; pinning; diagnostics; install/download (policy); prevent ambiguous resolution.

### 7.8 Plugins
- Packaging: id/version/author/nodes/deps/UI.
- Policies: allow/deny, workspace enablement, permission scopes (FS/network/exec), sandbox levels (trusted/restricted/isolated), warnings for untrusted; audit events.

### 7.9 Multi-User & RBAC (Server Mode)
- Workspaces isolation; roles: Admin, Operator, Creator, Viewer; audit events for login/run/plugin/policy changes.

### 7.10 Observability
- Metrics (Prometheus): queue lag, run durations, cache hit rates, worker utilization, errors.
- Logs with `request_id`; run debug bundle export.

---

## 8. Non-Functional Requirements
- Performance: UI load ≤2s for ≤200-node workflow; POST /runs p95 ≤200ms; WS p95 ≤500ms; control plane CPU-only; scale: 1k queued / 100 active runs.
- Reliability: durable queue; re-queue on worker failure; default RPO 15m, RTO 60m.
- Security: TLS for external traffic; secrets encrypted; rate limiting; secure-by-default plugins.
- Usability: rerun from metadata ≤3 steps; actionable errors referencing node/param.
- Maintainability: semantic versioning; workflow schema migrations.
- Portability: local Win/Linux/macOS; server via Docker Compose/Helm.
- Scalability: horizontal worker scale; API/WS concurrency targets.
- I18n: all user-facing strings via `admin.common.messages`/`get_message`; no hardcoded strings.

---

## 9. Data Requirements
- Entities: User, Workspace, Workflow (versions), Run, NodeExecution (optional), Artifact, Model, Plugin, AuditEvent.
- Run snapshot includes: workflow version, inputs, model versions, runtime options, environment fingerprint.
- Retention policies per workspace (runs/artifacts/debug).

---

## 10. Architecture and Project Split
- UI: `apps/web` (Lit 3.x Web Components), `packages/ui`, `packages/design-tokens`, `packages/types`.
- Backend/API: `apps/api` (Django + Django Ninja).
- Scheduler/Queue: `services/scheduler`.
- Worker: `services/worker` (execution, caching, progress).
- Storage/Registry: `services/storage` (artifacts/lineage), `services/registry` (models/plugins).
- Shared: `packages/workflow-schema` (VTXW), `packages/node-sdk`, `packages/proto` (optional).
- Infra: `infra/helm`, `infra/compose`.

---

## 11. Quality, Security, Performance, UX, i18n
- Security: auth (API keys/OIDC), RBAC, plugin sandboxing, TLS, secret handling, audit logs.
- Performance: canvas perf budgets; queue/scheduler throughput; cache hit optimization; WS delivery targets.
- UX: keyboard-first; clear validation/errors; node/param context; responsive layouts; A11y focus states.
- i18n: all strings routed via message catalog; no inline literals; include codes in `admin.common.messages`.

---

## 12. Verification Matrix (seed)
| Requirement ID       | Method      | Acceptance                                                       |
| -------------------- | ----------- | ---------------------------------------------------------------- |
| VTX-FR-API-010       | Test        | Submit run returns run_id and emits run.queued event             |
| VTX-FR-EXEC-020      | Test        | Upstream param change invalidates cache key and skips cache hit  |
| VTX-FR-DATA-021      | Test        | Artifact rerun reproduces workflow + pinned models               |
| VTX-FR-PLUG-011      | Inspection  | Restricted plugin blocked from network when policy denies        |
| VTX-NFR-PERF-003     | Benchmark   | WS progress p95 ≤ 500ms in reference deployment                  |
| VTX-UI-FR-GRAPH-004  | Test        | Undo/redo restores node positions/edges/params                   |
| VTX-UI-FR-RUN-011    | Test        | Timeline shows cache hits and node durations accurately          |
| VTX-UI-NFR-A11Y-001  | Manual/E2E  | Can build and run a workflow keyboard-only                       |

---

## 13. Milestones (UI + Code)
- M0: Repo scaffolding, CI, lint/test, Storybook; shared types/schema; command palette stub; API skeleton/auth.
- M1: Design system + shell/layout presets/persistence/tabs; workflows CRUD API.
- M2: Graph editor core + Problems panel + undo/redo; run submit/cancel + queue basics + WS events + cache key.
- M3: Inspector/docs/promotions/dirty/revert; batch panel + timeline + failure UX; worker matching + retries/cache hits.
- M4: Artifacts/gallery/detail/rerun; Models pinning/diagnostics; Plugins list/risk/audit; storage backends.
- M5: Workers view; Admin (RBAC/audit/quotas); plugin policy enforcement; observability metrics.
- M6: IDE Mode tools + plugin dev loop; desktop/Tauri extras (if in scope); A11y/I18n/perf/telemetry hardening; SRS/UI verification.

---

## 14. Open Items to Confirm
- Stack specifics: FE build tooling for Lit; BE Django/Ninja project layout; queue/storage choices.
- Real-time: WS only or also SSE?
- Desktop/Tauri scope for v1?
- Auth provider and RBAC role definitions finalized?
- Test stack: E2E (Playwright/Cypress), visual, a11y tools.
- Perf budgets: canvas scale targets; LCP/TTI targets.
- Telemetry/error provider and PII policy.
- Migration expectations for Comfy import coverage.

---

# 15. API & WebSocket Schemas (developer-ready)
### 15.1 Conventions
- Style: REST/JSON for external clients; Django + Django Ninja for implementation.
- Auth: API key and/or OIDC bearer; include `request_id` on all responses.
- Errors: `{ code: string, message: string, details?: object, request_id: string }` (strings via `admin.common.messages`/`get_message`).
- Pagination: cursor-based where lists may exceed page; include `next_cursor`.

### 15.2 Core Endpoints (v1) — Request/Response Shapes
- `POST /v1/runs`
  - Request: `{ workflow_id: string, workflow_version?: string, inputs: object, options?: { priority?: number, tags?: string[], target_backend?: string, cache_policy?: "use"|"bypass"|"refresh", seed?: number|number[] } }`
  - Response 200: `{ run_id: string, status: "queued", request_id: string }`
- `GET /v1/runs/{run_id}`
  - Response 200: `{ run_id, workflow_id, workflow_version, status, created_at, started_at?, completed_at?, options, summary?, queue_position?, request_id }`
- `POST /v1/runs/{run_id}/cancel`
  - Response 200: `{ run_id, status: "cancel_requested", request_id }`
- `GET /v1/queue`
  - Response 200: `{ items: [ { run_id, workflow_id, priority, status, worker_id?, eta? } ], lag_ms?: number, request_id }`
- `POST /v1/workflows`
  - Request: `{ name: string, description?: string, tags?: string[], definition: object }` (definition = VTXW JSON)
  - Response 201: `{ workflow_id, version, request_id }`
- `GET/PUT /v1/workflows/{id}`
  - GET Response 200: `{ workflow_id, version, definition: object, meta, compatibility? }`
  - PUT Request: `{ definition: object, meta? }`; Response 200: `{ workflow_id, version, request_id }`
- `GET /v1/artifacts/{id}`
  - Response 200: binary stream with metadata headers; or 302 to signed URL.
- `GET /v1/runs/{run_id}/artifacts`
  - Response 200: `{ artifacts: [ { id, type, mime, size, created_at, lineage: { node_id } } ], request_id }`
- `GET /v1/models`
  - Response 200: `{ models: [ { id, name, type, version, location, checksum, last_used?, pinned_workflows?: string[] } ], request_id }`
- `GET /v1/plugins`
  - Response 200: `{ plugins: [ { id, version, author, status, nodes: string[], origin, risk_level, sandbox_level } ], request_id }`
- `POST /v1/plugins/install`
  - Request: `{ source: string, version?: string, options?: object }`
  - Response 202: `{ task_id, request_id }`
- `POST /v1/plugins/{id}/enable|disable`
  - Response 200: `{ id, status, request_id }`

### 15.3 WebSocket Event Schema
- Envelope: `{ event_type: string, ts: string (ISO8601), run_id: string, node_id?: string, payload: object, request_id: string }`
- Event payload examples:
  - `run.queued`: `{ position?: number }`
  - `run.started`: `{ worker_id?: string }`
  - `node.progress`: `{ progress: number, message?: string }`
  - `node.cached`: `{ cache_key: string }`
  - `node.failed`: `{ error_code: string, message: string, details?: object }`
  - `artifact.created`: `{ artifact_id: string, type: string, mime: string, size: number }`

---

# 16. Workflow JSON Schema (VTXW)
- Top-level: `{ vtxw_version: "1.x", nodes: Node[], edges: Edge[], meta: WorkflowMeta, compatibility?: Compatibility }`
- Node: `{ id: string, type: string, inputs: Record<string, any>, ui: { x: number, y: number }, meta?: object }`
- Edge: `{ from: { node: string, port: string }, to: { node: string, port: string } }`
- WorkflowMeta: `{ name: string, description?: string, tags?: string[], created_at?: string, updated_at?: string, author?: string }`
- Compatibility: `{ imported_from?: string, warnings?: string[], substitutions?: array }`
- Determinism: node types declare determinism and resource hints; captured in node type registry, not per-instance.
- Validation: JSON Schema to be published in `packages/workflow-schema` (TS/Python bindings).

---

# 17. Data Model (Django ORM outlines)
- Models live in `admin/<app>/models.py` (follow `AgentSkin` pattern). Use Django ORM and migrations.
- Core models (fields indicative, to be finalized in schema package):
  - User: id (UUID), email, name, role (Admin/Operator/Creator/Viewer), workspace memberships.
  - Workspace: id, name, settings (JSON), retention policies.
  - Workflow: id, workspace FK, name, description, tags, current_version.
  - WorkflowVersion: id, workflow FK, version, definition (JSONField), meta (JSONField), created_at.
  - Run: id, workflow_version FK, workspace FK, status, options (JSON), inputs (JSON), created_at/started_at/completed_at, request_id.
  - NodeExecution (optional denorm): run FK, node_id, status, timings, cache_key, logs pointer.
  - Artifact: id, run FK, node_id, type, mime, size, location, hash, created_at, metadata (JSON).
  - Model: id, name, type, version, location, checksum, last_used, workspace FK (if scoped).
  - Plugin: id, version, author, origin, status, sandbox_level, nodes (JSON), workspace enablement (M2M).
  - AuditEvent: id, user FK, workspace FK, action, target_type/id, ts, details (JSON), request_id.
- All user-facing strings/error messages via `admin.common.messages` codes; no inline literals.

---

# 18. Core Flows (textual sequences)
- Run submission: Client → POST /runs → queue enqueue → scheduler assigns worker → worker streams WS events → artifacts stored → run completed/failed.
- Cache hit: Worker computes cache key → lookup → if hit and policy allows, mark node cached, emit events, skip exec.
- Plugin install: Admin → policy check → fetch package → register nodes → audit event → UI shows risk banner.
- Artifact rerun: UI “rerun from artifact” → load stored snapshot → POST /runs with pinned models/inputs.
- Comfy import: Upload JSON → importer maps nodes/types → report compatibility (warnings/substitutions) → creates VTXW definition.

---

# 19. UI Specification (Lit Web Components)
- Framework: Lit 3.x custom elements, shadow DOM; state via Lit reactive controllers; routing with SPA router; build with Vite/Rollup (to confirm).
- Component inventory (non-exhaustive):
  - `vtx-app-shell`, `vtx-left-rail`, `vtx-top-bar`, `vtx-panel` (resizable/collapsible), `vtx-tabs`.
  - Graph: `vtx-graph-canvas`, `vtx-node-card`, `vtx-port`, `vtx-edge`, `vtx-palette`, `vtx-problems-panel`.
  - Inspector: `vtx-inspector`, type-specific editors (slider, dropdown, json-editor, file-picker).
  - Runs: `vtx-run-controls`, `vtx-run-timeline`, `vtx-batch-panel`, `vtx-progress-stream`.
  - Artifacts: `vtx-artifact-gallery`, `vtx-artifact-detail`, `vtx-lineage-graph`.
  - Models/Plugins/Workers/Admin views: list/detail components with risk banners and status chips.
  - Command palette: `vtx-command-palette` (keyboard `Ctrl/⌘+K`).
- Keyboard map: command palette, search nodes, jump to node, run, cancel, open inspector, panel toggles, graph pan/zoom shortcuts. All primary flows must be keyboard operable.
- A11y: focus rings, ARIA labels, non-color status indicators, trap focus in modals, keyboard nav for lists/menus.
- i18n: no hardcoded strings; all text via message catalog codes.

---

# 20. Testing & Verification
- Unit and component tests (UI and API).
- E2E (Playwright/Cypress) covering graph edit, run submit/cancel, timeline, cache hits, artifact rerun, plugin risk banner, keyboard-only flows.
- Visual regression (Storybook/Cypress component) for core components and statuses.
- A11y: axe-core/pa11y; keyboard-only scenarios.
- Performance: benchmarks for POST /runs (p95 ≤200ms), WS delivery (p95 ≤500ms), UI LCP/TTI targets; canvas perf on large graphs.
- Security: auth/RBAC tests, plugin sandbox/policies, rate limiting, TLS enforcement (server mode).
- Observability: metrics exposure, request_id propagation, debug bundle export.

---

# 21. Security and Compliance
- TLS for external traffic; secrets encrypted at rest.
- RBAC enforced in server mode; workspaces isolation.
- Plugin sandbox levels and permission scopes enforced server-side; warnings for untrusted plugins.
- Audit events for login, run submissions, plugin installs, permission changes.
- Rate limiting for API and WS.

---

# 22. Internationalization
- All user-facing text (UI, API errors/messages) must use `admin.common.messages` via `get_message`; no inline literals.
- Message codes must be defined centrally for new strings; ready for translation.

---

# End of SRS
