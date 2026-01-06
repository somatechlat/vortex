# VORTEX UI/UX Specification ("UI SRS")
## ISO-aligned UI Requirements and Interaction Design
**Alignment goals:**
- ISO 9241-210 (human-centred design process outcomes)
- ISO 9241-110 (interaction/dialogue principles as UI heuristics)
- ISO/IEC 25010 usability/interaction-capability goals (quality attributes)

**Purpose:** Define the complete VORTEX user interface—layouts, screens, interactions, states, accessibility, developer tooling—so engineers can implement the UI and related APIs with minimal ambiguity.

---

## 0. Document Control
- **Owner:** Product/UX + Engineering
- **Applies to:** VORTEX Web UI + optional Desktop shell (Tauri)
- **Platforms:** Chromium-based browsers, Firefox; Desktop shell uses OS webview

---

## 1. UX Vision
### 1.1 Core UX Principles (Non-negotiable)
P1. **Fast feedback loops**: every change should show impact quickly (caching + partial rerun surfaces clearly).

P2. **Reproducible by default**: any artifact can reconstruct its full run (workflow + inputs + models + seeds + environment).

P3. **Developer-friendly**: the UI is also an IDE: inspect types, node docs, logs, performance, and plugin dev tools.

P4. **Human-friendly**: progressive disclosure, clear language, safe defaults, forgiving actions (undo/redo, confirmations).

P5. **No hidden state**: what’s running, what’s cached, what changed, and why must be visible.

### 1.2 Primary Personas
- **Creator** (80%): builds workflows, iterates quickly, cares about visual clarity and speed.
- **Power Creator** (15%): heavy use of shortcuts, templates, batching, multiple workflows.
- **Plugin Developer** (5%): builds nodes, tests quickly, needs debugging + docs + versioning.
- **Operator/Admin** (server mode): manages users, quotas, workers, models, policies.

---

## 2. Information Architecture
### 2.1 Top-Level Areas
A1. **Workflows** (graph editor, tabs, templates)
A2. **Runs** (queue, active runs, history)
A3. **Artifacts** (gallery, metadata, lineage)
A4. **Models** (registry, versions, storage)
A5. **Plugins** (install/update/policies)
A6. **Workers** (capabilities, health, utilization)
A7. **Settings** (preferences, performance, accounts)
A8. **Admin** (RBAC, audit, quotas) – server mode only

### 2.2 Navigation Model
**VTX-UI-FR-IA-001**: The UI shall provide a left navigation rail with icons + labels for A1–A8.

**VTX-UI-FR-IA-002**: The UI shall support keyboard navigation and a global command palette (`Ctrl/⌘+K`).

**VTX-UI-FR-IA-003**: The UI shall preserve context across areas (selected workspace, active workflow tab).

---

## 3. Layout System
### 3.1 Standard Layout
- **Top Bar**: Workspace selector, search, run controls, quick status (GPU, queue lag), user menu
- **Left Rail**: Areas A1–A8
- **Left Panel** (contextual): Workflow Explorer / Node Palette / Files
- **Center**: Graph canvas or primary content
- **Right Panel**: Inspector (node params, types, docs, validation)
- **Bottom Panel**: Console (logs), Timeline/Profiler, Run events, Debug

**VTX-UI-FR-LYT-001**: Panels shall be resizable, collapsible, and remembered per user.

**VTX-UI-FR-LYT-002**: The UI shall support 3 preset modes:
- **Create Mode** (graph-focused)
- **Run Mode** (queue + progress-focused)
- **IDE Mode** (files + logs + docs-heavy)

**VTX-UI-FR-LYT-003**: The UI shall support multi-tab workflows (like an IDE) with a tab bar above the canvas.

---

## 4. Graph Editor Specification
### 4.1 Canvas Interaction
**VTX-UI-FR-GRAPH-001**: The canvas shall support pan (space+drag), zoom (wheel/pinch), and zoom-to-fit.

**VTX-UI-FR-GRAPH-002**: The editor shall support drag/drop node creation from palette and quick-add (`A` then search).

**VTX-UI-FR-GRAPH-003**: The editor shall support multi-select, group nodes, and group collapse/expand.

**VTX-UI-FR-GRAPH-004**: The editor shall provide undo/redo for all graph operations.

### 4.2 Edges and Types
**VTX-UI-FR-GRAPH-010**: Ports shall be visually typed with icons and tooltips.

**VTX-UI-FR-GRAPH-011**: Invalid connections shall be blocked with a clear reason and suggested adapters.

### 4.3 Node Card UI
Node card includes:
- title + namespace badge (core/plugin)
- key params summary chips
- status pill (idle/cached/running/failed)
- quick actions (disable, isolate branch, view logs)

**VTX-UI-FR-GRAPH-020**: Node cards shall show real-time status during runs.

**VTX-UI-FR-GRAPH-021**: Clicking a node shall open the Inspector focused on:
- parameters (with validation)
- input/output types
- docs and examples
- provenance (plugin, version)

### 4.4 Validation UX
**VTX-UI-FR-GRAPH-030**: The editor shall provide a "Problems" panel (like VS Code) listing errors/warnings with jump-to-node.

**VTX-UI-FR-GRAPH-031**: Validation shall run continuously but be debounced to avoid UI lag.

---

## 5. Inspector (Right Panel)
### 5.1 Parameter Editing
**VTX-UI-FR-INSP-001**: Parameters shall be edited with type-appropriate controls (slider, dropdown, file picker, JSON editor).

**VTX-UI-FR-INSP-002**: Parameter edits shall support per-field revert to default and show “dirty” markers.

**VTX-UI-FR-INSP-003**: The inspector shall support parameter promotion to workflow-level inputs.

### 5.2 Docs & Examples
**VTX-UI-FR-INSP-010**: Each node type shall have a docs view with description, inputs/outputs, examples, and constraints.

**VTX-UI-FR-INSP-011**: For plugin nodes, docs shall include plugin source/version and security level.

---

## 6. Runs, Queue, and Progress UI
### 6.1 Run Submission
**VTX-UI-FR-RUN-001**: The top bar shall include Run controls: Queue, Run Now, Cancel, Rerun.

**VTX-UI-FR-RUN-002**: The UI shall allow batch runs via a “Batch Panel” (seeds, prompts, parameter sweeps).

### 6.2 Live Progress
**VTX-UI-FR-RUN-010**: During execution, the UI shall show:
- per-node progress
- overall run timeline
- streaming previews where available

**VTX-UI-FR-RUN-011**: The UI shall provide a “Run Timeline” view with node durations and cache hits.

### 6.3 Failures
**VTX-UI-FR-RUN-020**: When a node fails, the UI shall:
- highlight the node
- show the error summary
- provide actions: retry node, retry run, open logs, open docs

---

## 7. Artifacts & Lineage
### 7.1 Gallery
**VTX-UI-FR-ART-001**: Artifacts view shall provide a grid gallery with filters (workflow, tag, date, model, status).

### 7.2 Metadata & Repro
**VTX-UI-FR-ART-010**: Artifact details shall show:
- prompt/inputs
- seeds
- model versions
- workflow snapshot
- environment fingerprint
- lineage graph (which nodes produced it)

**VTX-UI-FR-ART-011**: A “Re-run from this artifact” button shall create a new run with identical inputs by default.

---

## 8. Models UI
**VTX-UI-FR-MDL-001**: Models view shall list models with type, version, checksum, location, and last-used.

**VTX-UI-FR-MDL-002**: The UI shall allow pinning a workflow to specific model versions.

**VTX-UI-FR-MDL-003**: The UI shall show missing-model diagnostics on import and provide resolution actions.

---

## 9. Plugins UI
**VTX-UI-FR-PLG-001**: Plugins view shall list plugins with status, version, origin, and nodes provided.

**VTX-UI-FR-PLG-002**: Installing/updating plugins shall be policy-gated and produce an audit event.

**VTX-UI-FR-PLG-003**: The UI shall display a risk banner for non-trusted plugins (requested permissions + sandbox level).

---

## 10. IDE Mode (Developer Experience)
### 10.1 Built-in Tools
**VTX-UI-FR-IDE-001**: IDE Mode shall include a file explorer for:
- workflows
- templates
- plugin projects (desktop mode or server-mounted workspace)

**VTX-UI-FR-IDE-002**: IDE Mode shall include:
- node SDK docs viewer
- API explorer (OpenAPI-style)
- WebSocket event inspector
- run debug bundle export/import

### 10.2 Plugin Dev Loop
**VTX-UI-FR-IDE-010**: The UI shall support “hot reload” for plugin node definitions in development mode.

**VTX-UI-FR-IDE-011**: The UI shall provide a “Node Test Harness” to run a single node with synthetic inputs.

### 10.3 Desktop Shell Enhancements (Tauri)
**VTX-UI-FR-IDE-020**: In desktop mode, the app shall support:
- local filesystem mounts
- local model discovery
- optional embedded worker
- offline workflows and caching

---

## 11. Accessibility & Internationalization
**VTX-UI-NFR-A11Y-001**: The UI shall support full keyboard operation for primary tasks (build, run, inspect).

**VTX-UI-NFR-A11Y-002**: The UI shall include visible focus states and accessible labels for controls.

**VTX-UI-NFR-A11Y-003**: Color usage shall not be the sole carrier of meaning (status icons + text).

**VTX-UI-NFR-I18N-001**: The UI shall support translation-ready strings (no hardcoded UI text in components).

---

## 12. Visual Design System
**VTX-UI-FR-DS-001**: The UI shall ship with a design system including:
- typography scale
- spacing scale
- components (buttons, inputs, chips, tabs, toasts)
- statuses (success/warn/error/running/cached)

**VTX-UI-FR-DS-002**: The UI shall support light/dark themes and an accent color.

---

## 13. Copy & Microinteractions
**VTX-UI-NFR-COPY-001**: Microcopy shall be plain language and action-oriented (avoid jargon unless in developer view).

**VTX-UI-NFR-COPY-002**: The UI shall confirm destructive actions with clear consequences and undo when possible.

---

## 14. UI Verification (Examples)
| Requirement ID | Method | Acceptance |
|---|---|---|
| VTX-UI-FR-GRAPH-004 | Test | Undo/redo restores node positions/edges/params |
| VTX-UI-FR-RUN-011 | Test | Timeline shows cache hits and node durations accurately |
| VTX-UI-FR-PLG-003 | Inspection | Risk banner shows permission scopes and sandbox level |
| VTX-UI-NFR-A11Y-001 | Manual/Automated | Can create + run a workflow without mouse |

---

# End of UI SRS
