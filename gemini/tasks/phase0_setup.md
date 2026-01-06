OK PLESE KEEP IMPLE,ENTING AS THE WHOLE PERSONAS AL THE TIME AND HAVE THE VIBE CODING RULES ALWYS IN YOUR MIND, act here ass the full peronas in the rules and use them alwqys here i nthis session , TEST ALWAYS ON REAL INFRA EXCEP FOR UNIT TESTING when you have finished coding a milestone or whatever act as the peronsonas all of them at the same time all the time here!
# Phase 0: Project Setup

> **Duration**: Week 1  
> **Dependencies**: None  
> **Status**: 🔴 Not Started

---

## P0.1 Repository Scaffolding

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P0.1.1 | Create directory structure per `project_structure.md` | ⚪ | - | |
| P0.1.2 | Initialize Cargo workspace with `vortex-core`, `vortex-registry`, `vortex-protocol` | ⚪ | - | |
| P0.1.3 | Create Python `worker/` package with `pyproject.toml` | ⚪ | - | |
| P0.1.4 | Create Svelte app in `ui/` with Vite + TypeScript | ⚪ | - | |
| P0.1.5 | Setup `.editorconfig`, `.gitignore`, `.pre-commit-config.yaml` | ⚪ | - | |

---

## P0.2 CI/CD Foundation

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P0.2.1 | Create `.github/workflows/ci.yml` (build + test) | ⚪ | - | |
| P0.2.2 | Create `.github/workflows/security.yml` (cargo-audit, bandit) | ⚪ | - | |
| P0.2.3 | Setup Dependabot for Rust and Python | ⚪ | - | |
| P0.2.4 | Create issue templates | ⚪ | - | |

---

## P0.3 Development Environment

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P0.3.1 | Create `docker-compose.yml` for local dev | ⚪ | - | |
| P0.3.2 | Create `Tiltfile` for Minikube | ⚪ | - | |
| P0.3.3 | Create `k8s/` base manifests | ⚪ | - | |

---

## Completion Checklist

- [ ] All directories exist per structure doc
- [ ] `cargo build` succeeds
- [ ] `bun install && bun run dev` works in `ui/`
- [ ] CI pipeline passes on GitHub

---

**Unblocks**: Phase 1 (Protocol)
