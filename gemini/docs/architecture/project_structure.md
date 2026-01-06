# VORTEX Enterprise Project Structure
## ISO/IEC Compliant Software Development Standard

> **Compliance**: ISO 9001 (Quality), ISO/IEC 27001 (Security), ISO 31000 (Risk), ISO/IEC 29148 (SRS)  
> **Developer**: [SOMATECH](https://www.somatech.dev)  
> **Version**: 1.0.0  
> **Classification**: Enterprise Production

---

## 📁 Complete Enterprise File Tree

```
vortex/
│
├── 📄 README.md                        # Project overview
├── 📄 LICENSE                          # Apache 2.0
├── 📄 CHANGELOG.md                     # Release history (Keep-a-Changelog)
├── 📄 CONTRIBUTING.md                  # Contribution guidelines
├── 📄 CODE_OF_CONDUCT.md               # Community standards
├── 📄 SECURITY.md                      # Security policy & disclosure
├── 📄 CODEOWNERS                       # GitHub ownership matrix
├── 📄 agent.md                         # Agent context document
├── 📄 rules.md                         # VIBE Coding Rules (25 rules)
│
├── 📄 Cargo.toml                       # Rust workspace root
├── 📄 Cargo.lock                       # Locked dependencies
├── 📄 Tiltfile                         # Local K8s development
├── 📄 docker-compose.yml               # Container orchestration
├── 📄 docker-compose.prod.yml          # Production compose
├── 📄 Makefile                         # Build automation
│
├── 📄 .env.example                     # Environment template
├── 📄 .editorconfig                    # Editor standards
├── 📄 .gitignore                       # Git exclusions
├── 📄 .gitattributes                   # Git LFS / line endings
├── 📄 .pre-commit-config.yaml          # Pre-commit hooks
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🏛️ GOVERNANCE & COMPLIANCE
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 .governance/
│   ├── 📂 policies/
│   │   ├── data-classification.md      # Data handling tiers
│   │   ├── access-control.md           # RBAC policies
│   │   ├── incident-response.md        # Security incident SOP
│   │   └── change-management.md        # Change control board
│   ├── 📂 compliance/
│   │   ├── gdpr-checklist.md           # GDPR compliance
│   │   ├── soc2-controls.md            # SOC 2 Type II mapping
│   │   └── iso27001-controls.md        # ISO 27001 Annex A
│   └── 📂 audit/
│       ├── audit-log-schema.json       # Audit log format
│       └── retention-policy.md         # Data retention
│
│
│   ═══════════════════════════════════════════════════════════════
│                      📋 QUALITY MANAGEMENT (ISO 9001)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 .quality/
│   ├── 📂 processes/
│   │   ├── development-lifecycle.md    # SDLC process
│   │   ├── code-review-checklist.md    # Review standards
│   │   ├── release-process.md          # Release gates
│   │   └── retrospective-template.md   # Sprint retros
│   ├── 📂 metrics/
│   │   ├── kpi-definitions.md          # Key Performance Indicators
│   │   ├── sla-definitions.md          # Service Level Agreements
│   │   └── quality-gates.md            # CI/CD quality gates
│   └── 📂 templates/
│       ├── bug-report.md               # Issue templates
│       ├── feature-request.md
│       ├── adr-template.md             # Architecture Decision Record
│       └── post-mortem.md              # Incident analysis
│
│
│   ═══════════════════════════════════════════════════════════════
│                      ⚠️ RISK MANAGEMENT (ISO 31000)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 .risk/
│   ├── risk-register.md                # Active risk tracking
│   ├── fmea-analysis.md                # Failure Mode Effects Analysis
│   ├── threat-model.md                 # STRIDE threat modeling
│   └── 📂 mitigations/
│       ├── security-controls.md        # Security mitigations
│       └── operational-controls.md     # Ops mitigations
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🔐 SECURITY (ISO 27001)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 .security/
│   ├── 📂 policies/
│   │   ├── secret-management.md        # Vault/SOPS setup
│   │   ├── key-rotation.md             # Key lifecycle
│   │   └── vulnerability-mgmt.md       # CVE handling
│   ├── 📂 assessments/
│   │   ├── penetration-test.md         # Pentest reports
│   │   └── dependency-audit.md         # cargo-audit output
│   └── 📂 controls/
│       ├── seccomp-policy.json         # Seccomp BPF filter
│       ├── apparmor-profile            # AppArmor profile
│       └── sandbox-entitlements.plist  # macOS sandbox
│
│
│   ═══════════════════════════════════════════════════════════════
│                      📚 DOCUMENTATION (ISO 29148)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 docs/
│   │
│   ├── 📂 specs/                       # ISO/IEC 29148 SRS
│   │   ├── 00_master_srs.md            # System Architecture (1,628 lines)
│   │   ├── 01_core_engine_srs.md       # Core Engine (1,907 lines)
│   │   ├── 02_frontend_ui_srs.md       # Frontend UI (1,765 lines)
│   │   ├── 03_compute_fabric_srs.md    # Compute Fabric (1,671 lines)
│   │   ├── 04_registry_srs.md          # Registry System (1,754 lines)
│   │   └── 05_ui_ux_spec.md            # ISO 9241 UX spec
│   │
│   ├── 📂 architecture/
│   │   ├── project_structure.md        # This document
│   │   ├── feature_implementation_plan.md
│   │   ├── architecture_audit_report.md
│   │   ├── 📂 adr/                     # Architecture Decision Records
│   │   │   ├── ADR-001-rust-core.md
│   │   │   ├── ADR-002-arrow-shm.md
│   │   │   ├── ADR-003-svelte-ui.md
│   │   │   └── ADR-004-pubgrub.md
│   │   └── 📂 diagrams/
│   │       ├── system-context.puml     # C4 Context
│   │       ├── container.puml          # C4 Container
│   │       ├── component.puml          # C4 Component
│   │       └── deployment.puml         # Deployment diagram
│   │
│   ├── 📂 research/
│   │   ├── competitive_analysis.md
│   │   └── technology_radar.md         # Tech recommendations
│   │
│   ├── 📂 api/
│   │   ├── openapi.yaml                # REST API spec
│   │   ├── websocket-protocol.md       # WS message spec
│   │   └── protobuf-guide.md           # IPC protocol
│   │
│   ├── 📂 runbooks/
│   │   ├── deployment.md               # Deploy procedures
│   │   ├── rollback.md                 # Rollback procedures
│   │   ├── scaling.md                  # Scaling guide
│   │   └── troubleshooting.md          # Common issues
│   │
│   └── design_system.md                # CSS/Testing standards
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🦀 RUST WORKSPACE
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 crates/
│   │
│   ├── 📂 vortex-core/                 # Core Engine (SRS 01)
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # Build script (proto compile)
│   │   └── src/
│   │       ├── lib.rs                  # Library exports
│   │       ├── main.rs                 # Binary entrypoint
│   │       ├── config.rs               # Config loading (figment)
│   │       ├── telemetry.rs            # OpenTelemetry setup
│   │       ├── error.rs                # Error types (thiserror)
│   │       │
│   │       ├── 📂 graph/
│   │       │   ├── mod.rs
│   │       │   ├── dsl.rs              # GraphDSL parser (serde)
│   │       │   ├── node.rs             # Node definitions
│   │       │   ├── edge.rs             # Edge/connection logic
│   │       │   ├── validate.rs         # Graph validation
│   │       │   └── merkle.rs           # Merkle hashing
│   │       │
│   │       ├── 📂 scheduler/
│   │       │   ├── mod.rs
│   │       │   ├── kahn.rs             # Kahn's algorithm O(n+m)
│   │       │   ├── parallel.rs         # Parallel execution paths
│   │       │   ├── dirty.rs            # Dirty set detection
│   │       │   └── plan.rs             # ExecutionPlan struct
│   │       │
│   │       ├── 📂 salsa/
│   │       │   ├── mod.rs
│   │       │   ├── db.rs               # Salsa database
│   │       │   ├── queries.rs          # Incremental queries
│   │       │   └── inputs.rs           # Input tracking
│   │       │
│   │       ├── 📂 arbiter/
│   │       │   ├── mod.rs
│   │       │   ├── vram.rs             # VRAM tracking
│   │       │   ├── predictor.rs        # Memory prediction
│   │       │   └── eviction.rs         # LFU eviction
│   │       │
│   │       ├── 📂 supervisor/
│   │       │   ├── mod.rs
│   │       │   ├── spawn.rs            # fork()/exec()
│   │       │   ├── health.rs           # Heartbeat monitoring
│   │       │   ├── recovery.rs         # Crash recovery
│   │       │   └── signals.rs          # Signal handling
│   │       │
│   │       ├── 📂 ipc/
│   │       │   ├── mod.rs
│   │       │   ├── socket.rs           # Unix Domain Socket
│   │       │   ├── protocol.rs         # Message framing
│   │       │   ├── gateway.rs          # Connection manager
│   │       │   └── codec.rs            # Protobuf codec
│   │       │
│   │       ├── 📂 shm/
│   │       │   ├── mod.rs
│   │       │   ├── arena.rs            # 64GB arena manager
│   │       │   ├── header.rs           # ShmHeader struct
│   │       │   ├── slots.rs            # WorkerSlot array
│   │       │   ├── alloc.rs            # Bump allocator
│   │       │   └── safety.rs           # Pointer validation
│   │       │
│   │       ├── 📂 db/
│   │       │   ├── mod.rs
│   │       │   ├── schema.rs           # SQLite schema
│   │       │   ├── migrations/         # SQLx migrations
│   │       │   │   └── 001_initial.sql
│   │       │   └── queries.rs          # Typed queries
│   │       │
│   │       └── 📂 api/
│   │           ├── mod.rs
│   │           ├── http.rs             # Axum routes
│   │           ├── ws.rs               # WebSocket handler
│   │           ├── middleware.rs       # Auth, logging
│   │           └── handlers/
│   │               ├── graph.rs
│   │               ├── execution.rs
│   │               └── health.rs
│   │
│   ├── 📂 vortex-registry/             # Registry System (SRS 04)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── cli.rs                  # CLI commands (clap)
│   │       │
│   │       ├── 📂 solver/
│   │       │   ├── mod.rs
│   │       │   ├── pubgrub.rs          # PubGrub algorithm
│   │       │   ├── version.rs          # SemVer constraints
│   │       │   ├── conflict.rs         # Conflict analysis
│   │       │   └── explain.rs          # Error explanation
│   │       │
│   │       ├── 📂 scanner/
│   │       │   ├── mod.rs
│   │       │   ├── ast.rs              # Python AST parser
│   │       │   ├── patterns.rs         # Blacklist patterns
│   │       │   └── report.rs           # SecurityReport
│   │       │
│   │       ├── 📂 venv/
│   │       │   ├── mod.rs
│   │       │   ├── fork.rs             # Environment forking
│   │       │   ├── manager.rs          # Venv lifecycle
│   │       │   └── python.rs           # Python detection
│   │       │
│   │       ├── manifest.rs             # vortex.toml parser
│   │       ├── lockfile.rs             # vortex.lock parser/writer
│   │       └── registry.rs             # PyPI client
│   │
│   ├── 📂 vortex-protocol/             # Shared Protocol Types
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # prost-build
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── proto/
│   │       │   ├── control.proto       # IPC messages
│   │       │   ├── graph.proto         # Graph definitions
│   │       │   └── worker.proto        # Worker messages
│   │       ├── types.rs                # Shared Rust types
│   │       ├── errors.rs               # VE-XXX error codes
│   │       └── constants.rs            # Magic bytes, versions
│   │
│   └── 📂 vortex-telemetry/            # Observability
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── metrics.rs              # Prometheus metrics
│           ├── tracing.rs              # OpenTelemetry traces
│           └── logging.rs              # Structured JSON logs
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🐍 PYTHON COMPUTE FABRIC (SRS 03)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 worker/
│   ├── 📄 pyproject.toml               # PEP 517 project config
│   ├── 📄 setup.cfg                    # Package metadata
│   ├── 📄 requirements.txt             # Pinned dependencies
│   ├── 📄 requirements-dev.txt         # Dev dependencies
│   ├── 📄 mypy.ini                     # Type checking config
│   ├── 📄 pytest.ini                   # Test config
│   ├── 📄 .python-version              # pyenv version
│   │
│   ├── 📂 vortex_worker/
│   │   ├── __init__.py
│   │   ├── __main__.py                 # python -m entrypoint
│   │   ├── main.py                     # Worker entrypoint
│   │   ├── config.py                   # Configuration
│   │   ├── logging.py                  # Structured logging
│   │   │
│   │   ├── 📂 ipc/
│   │   │   ├── __init__.py
│   │   │   ├── socket.py               # UDS connection
│   │   │   ├── protocol.py             # Protobuf (de)serialize
│   │   │   └── handler.py              # Message routing
│   │   │
│   │   ├── 📂 shm/
│   │   │   ├── __init__.py
│   │   │   ├── mapping.py              # mmap wrapper
│   │   │   ├── header.py               # ShmHeader ctypes
│   │   │   └── slots.py                # WorkerSlot ctypes
│   │   │
│   │   ├── 📂 bridge/
│   │   │   ├── __init__.py
│   │   │   ├── arrow.py                # PyArrow handling
│   │   │   ├── dlpack.py               # DLPack conversion
│   │   │   └── tensor.py               # Tensor utilities
│   │   │
│   │   ├── 📂 device/
│   │   │   ├── __init__.py
│   │   │   ├── cuda.py                 # CUDA device manager
│   │   │   ├── metal.py                # Metal (macOS)
│   │   │   └── cpu.py                  # CPU fallback
│   │   │
│   │   ├── 📂 sandbox/
│   │   │   ├── __init__.py
│   │   │   ├── seccomp.py              # Seccomp BPF (Linux)
│   │   │   ├── macos.py                # App Sandbox (macOS)
│   │   │   └── import_hook.py          # Module blocking
│   │   │
│   │   └── 📂 executor/
│   │       ├── __init__.py
│   │       ├── base.py                 # AbstractExecutor
│   │       ├── registry.py             # Executor registry
│   │       ├── 📂 nodes/
│   │       │   ├── __init__.py
│   │       │   ├── loader.py           # Model loading
│   │       │   ├── sampler.py          # KSampler
│   │       │   ├── vae.py              # VAE encode/decode
│   │       │   ├── clip.py             # CLIP encode
│   │       │   ├── controlnet.py       # ControlNet
│   │       │   └── upscale.py          # Upscaling
│   │       └── 📂 custom/
│   │           └── __init__.py         # Custom node plugins
│   │
│   └── 📂 tests/
│       ├── __init__.py
│       ├── conftest.py                 # Pytest fixtures
│       ├── test_ipc.py
│       ├── test_shm.py
│       ├── test_bridge.py
│       └── test_executor.py
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🎨 FRONTEND UI (SRS 02)
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 ui/
│   ├── 📄 package.json                 # Bun/npm config
│   ├── 📄 bun.lockb                    # Bun lockfile
│   ├── 📄 svelte.config.js             # Svelte 5 config
│   ├── 📄 vite.config.ts               # Vite bundler
│   ├── 📄 tsconfig.json                # TypeScript config
│   ├── 📄 eslint.config.js             # ESLint flat config
│   ├── 📄 prettier.config.js           # Prettier config
│   ├── 📄 playwright.config.ts         # E2E test config
│   │
│   ├── 📂 src/
│   │   ├── app.html                    # HTML shell
│   │   ├── app.css                     # Global CSS imports
│   │   │
│   │   ├── 📂 lib/
│   │   │   │
│   │   │   ├── 📂 styles/
│   │   │   │   ├── tokens.css          # Design tokens
│   │   │   │   ├── reset.css           # CSS reset
│   │   │   │   ├── typography.css      # Font styles
│   │   │   │   ├── utilities.css       # Utility classes
│   │   │   │   └── animations.css      # Keyframes
│   │   │   │
│   │   │   ├── 📂 stores/
│   │   │   │   ├── graph.svelte.ts     # Graph state ($state)
│   │   │   │   ├── viewport.svelte.ts  # Pan/zoom state
│   │   │   │   ├── selection.svelte.ts # Selection state
│   │   │   │   ├── execution.svelte.ts # Run state
│   │   │   │   ├── collab.svelte.ts    # Yjs CRDT
│   │   │   │   └── settings.svelte.ts  # User prefs
│   │   │   │
│   │   │   ├── 📂 components/
│   │   │   │   ├── 📂 canvas/
│   │   │   │   │   ├── Canvas.svelte   # Main canvas
│   │   │   │   │   ├── Grid.svelte     # Background grid
│   │   │   │   │   └── Minimap.svelte  # Navigation
│   │   │   │   │
│   │   │   │   ├── 📂 node/
│   │   │   │   │   ├── Node.svelte     # Node container
│   │   │   │   │   ├── NodeHeader.svelte
│   │   │   │   │   ├── NodeBody.svelte
│   │   │   │   │   ├── NodePreview.svelte
│   │   │   │   │   └── NodeLite.svelte # LOD version
│   │   │   │   │
│   │   │   │   ├── 📂 port/
│   │   │   │   │   ├── Port.svelte
│   │   │   │   │   ├── InputPort.svelte
│   │   │   │   │   └── OutputPort.svelte
│   │   │   │   │
│   │   │   │   ├── 📂 edge/
│   │   │   │   │   ├── Edge.svelte      # Connection wire
│   │   │   │   │   ├── EdgePath.svelte  # Bezier path
│   │   │   │   │   └── EdgeDrag.svelte  # Drag preview
│   │   │   │   │
│   │   │   │   ├── 📂 widgets/
│   │   │   │   │   ├── Slider.svelte
│   │   │   │   │   ├── NumberInput.svelte
│   │   │   │   │   ├── TextInput.svelte
│   │   │   │   │   ├── Select.svelte
│   │   │   │   │   ├── ColorPicker.svelte
│   │   │   │   │   └── ImageUpload.svelte
│   │   │   │   │
│   │   │   │   ├── 📂 panels/
│   │   │   │   │   ├── Toolbar.svelte
│   │   │   │   │   ├── Sidebar.svelte
│   │   │   │   │   ├── NodePalette.svelte
│   │   │   │   │   ├── PropertyPanel.svelte
│   │   │   │   │   └── QueuePanel.svelte
│   │   │   │   │
│   │   │   │   └── 📂 overlays/
│   │   │   │       ├── ContextMenu.svelte
│   │   │   │       ├── CommandPalette.svelte
│   │   │   │       ├── Toast.svelte
│   │   │   │       └── Modal.svelte
│   │   │   │
│   │   │   ├── 📂 canvas/
│   │   │   │   ├── webgl.ts            # WebGL2 renderer
│   │   │   │   ├── shaders/
│   │   │   │   │   ├── node.vert
│   │   │   │   │   ├── node.frag
│   │   │   │   │   ├── edge.vert
│   │   │   │   │   └── edge.frag
│   │   │   │   ├── camera.ts           # Pan/zoom math
│   │   │   │   ├── culling.ts          # Viewport culling
│   │   │   │   ├── picking.ts          # GPU-based selection
│   │   │   │   └── instancing.ts       # Instanced rendering
│   │   │   │
│   │   │   ├── 📂 services/
│   │   │   │   ├── api.ts              # HTTP client (fetch)
│   │   │   │   ├── ws.ts               # WebSocket client
│   │   │   │   ├── yjs.ts              # Yjs + y-websocket
│   │   │   │   └── storage.ts          # localStorage
│   │   │   │
│   │   │   ├── 📂 utils/
│   │   │   │   ├── geometry.ts         # Math helpers
│   │   │   │   ├── bezier.ts           # Bezier calculations
│   │   │   │   ├── debounce.ts
│   │   │   │   └── keyboard.ts         # Hotkey handling
│   │   │   │
│   │   │   └── 📂 types/
│   │   │       ├── node.ts             # Node types
│   │   │       ├── graph.ts            # Graph types
│   │   │       └── protocol.ts         # WS message types
│   │   │
│   │   └── 📂 routes/
│   │       ├── +page.svelte            # Main canvas page
│   │       ├── +layout.svelte          # Root layout
│   │       ├── +error.svelte           # Error page
│   │       └── 📂 settings/
│   │           └── +page.svelte        # Settings page
│   │
│   ├── 📂 static/
│   │   ├── favicon.ico
│   │   ├── fonts/
│   │   │   ├── GeistSans.woff2
│   │   │   └── GeistMono.woff2
│   │   └── icons/
│   │       └── sprite.svg              # Icon sprite
│   │
│   └── 📂 tests/
│       ├── 📂 e2e/
│       │   ├── canvas.spec.ts
│       │   ├── node.spec.ts
│       │   ├── connection.spec.ts
│       │   ├── zoom.spec.ts
│       │   └── collab.spec.ts
│       ├── 📂 unit/
│       │   ├── geometry.test.ts
│       │   └── bezier.test.ts
│       ├── 📂 screenshots/             # Visual regression
│       └── 📂 videos/                  # Failure recordings
│
│
│   ═══════════════════════════════════════════════════════════════
│                      ☸️ KUBERNETES & INFRASTRUCTURE
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 k8s/
│   ├── 📂 base/                        # Kustomize base
│   │   ├── kustomization.yaml
│   │   ├── namespace.yaml              # vortex-dev namespace
│   │   ├── resourcequota.yaml          # 8GB quota
│   │   ├── limitrange.yaml             # Per-pod limits
│   │   ├── 📂 core/
│   │   │   ├── deployment.yaml
│   │   │   ├── service.yaml
│   │   │   └── configmap.yaml
│   │   ├── 📂 worker/
│   │   │   ├── deployment.yaml
│   │   │   └── pdb.yaml                # Pod disruption budget
│   │   └── 📂 ui/
│   │       ├── deployment.yaml
│   │       └── ingress.yaml
│   │
│   ├── 📂 overlays/                    # Kustomize overlays
│   │   ├── 📂 development/
│   │   │   ├── kustomization.yaml
│   │   │   └── patches/
│   │   ├── 📂 staging/
│   │   │   └── kustomization.yaml
│   │   └── 📂 production/
│   │       ├── kustomization.yaml
│   │       ├── hpa.yaml                # Horizontal Pod Autoscaler
│   │       └── psp.yaml                # Pod Security Policy
│   │
│   └── 📂 helm/                        # Helm chart (optional)
│       ├── Chart.yaml
│       ├── values.yaml
│       └── templates/
│
├── 📂 docker/
│   ├── 📂 core/
│   │   ├── Dockerfile
│   │   └── entrypoint.sh
│   ├── 📂 worker/
│   │   ├── Dockerfile
│   │   └── entrypoint.sh
│   └── 📂 ui/
│       ├── Dockerfile
│       └── nginx.conf
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🧪 TESTING
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 tests/
│   ├── 📂 unit/                        # Unit tests (cargo test)
│   │   ├── scheduler_test.rs
│   │   └── salsa_test.rs
│   │
│   ├── 📂 integration/                 # Integration tests
│   │   ├── shm_test.rs                 # Host ↔ Worker memory
│   │   ├── ipc_test.rs                 # Socket communication
│   │   └── db_test.rs                  # SQLite persistence
│   │
│   ├── 📂 e2e/                         # End-to-end tests
│   │   └── full_pipeline_test.rs       # Complete workflow
│   │
│   ├── 📂 benches/                     # Performance benchmarks
│   │   ├── scheduler_bench.rs          # Criterion benchmarks
│   │   ├── ipc_bench.rs
│   │   └── shm_bench.rs
│   │
│   ├── 📂 fixtures/                    # Test data
│   │   ├── graphs/
│   │   │   ├── simple.json
│   │   │   ├── complex.json
│   │   │   └── stress_10k.json
│   │   └── images/
│   │       └── test_512x512.png
│   │
│   └── 📂 golden/                      # Golden master outputs
│       └── sdxl_standard_hash.txt
│
│
│   ═══════════════════════════════════════════════════════════════
│                      🔄 CI/CD
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 .github/
│   ├── 📂 workflows/
│   │   ├── ci.yml                      # Build + Test
│   │   ├── release.yml                 # Semantic Release
│   │   ├── security.yml                # Dependency scan
│   │   └── docs.yml                    # Documentation deploy
│   │
│   ├── 📂 ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── config.yml
│   │
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── dependabot.yml                  # Dependency updates
│   └── CODEOWNERS
│
│
│   ═══════════════════════════════════════════════════════════════
│                      📊 OBSERVABILITY
│   ═══════════════════════════════════════════════════════════════
│
├── 📂 observability/
│   ├── 📂 prometheus/
│   │   ├── prometheus.yml              # Scrape configs
│   │   └── alerts.yml                  # Alert rules
│   │
│   ├── 📂 grafana/
│   │   ├── datasources.yaml
│   │   └── dashboards/
│   │       ├── overview.json
│   │       ├── workers.json
│   │       └── performance.json
│   │
│   └── 📂 jaeger/
│       └── jaeger.yml                  # Trace collection
│
│
└── 📂 scripts/                         # Automation scripts
    ├── setup-dev.sh                    # Dev environment setup
    ├── build-release.sh                # Release build
    ├── generate-proto.sh               # Protobuf codegen
    └── benchmark.sh                    # Run benchmarks
```

---

## 📊 Structure Statistics

| Category | Count |
|----------|-------|
| **Total Directories** | 120+ |
| **Rust Source Files** | 60+ |
| **Python Source Files** | 35+ |
| **Svelte Components** | 40+ |
| **Test Files** | 30+ |
| **Documentation Files** | 25+ |
| **Config Files** | 40+ |
| **Total Files** | **250+** |

---

## 🏛️ ISO Compliance Mapping

| ISO Standard | Directory | Purpose |
|--------------|-----------|---------|
| **ISO 9001** | `.quality/` | Quality management processes |
| **ISO 27001** | `.security/`, `.governance/` | Information security |
| **ISO 31000** | `.risk/` | Risk management |
| **ISO 29148** | `docs/specs/` | Requirements specification |
| **ISO 9241** | `docs/specs/05_ui_ux_spec.md` | Usability standards |

---

## 🚀 Implementation Order

1. **Phase 0**: Scaffolding (create empty structure)
2. **Phase 1**: Protocol (`vortex-protocol/`) - shared types
3. **Phase 2**: Core Engine (`vortex-core/`) - Rust implementation
4. **Phase 3**: Worker (`worker/`) - Python compute
5. **Phase 4**: UI (`ui/`) - Svelte frontend
6. **Phase 5**: Registry (`vortex-registry/`) - package manager
7. **Phase 6**: Infrastructure (`k8s/`, `docker/`) - deployment
8. **Phase 7**: Observability (`observability/`) - monitoring

---

**Total Files**: 250+  
**Lines of SRS**: 8,700+  
**ISO Compliant**: ✅  
**Enterprise Ready**: ✅
