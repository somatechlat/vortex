# Phase 5: Registry System

> **Duration**: Weeks 13-14  
> **Dependencies**: Phase 2  
> **Status**: ⚪ Blocked  
> **SDD Reference**: [04_registry_sdd.md](../docs/design/04_registry_sdd.md)

---

## P5.1 Solver Module (SDD §3.1)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P5.1.1 | Implement `Package`, `Version` types | ⚪ | - | |
| P5.1.2 | Implement `Term`, `Incompatibility` types | ⚪ | - | |
| P5.1.3 | Implement `unit_propagate()` | ⚪ | - | |
| P5.1.4 | Implement `resolve_conflict()` (CDCL) | ⚪ | - | |
| P5.1.5 | Implement main `solve()` loop | ⚪ | - | |
| P5.1.6 | Write solver correctness tests | ⚪ | - | |

---

## P5.2 Scanner Module (SDD §3.2)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P5.2.1 | Setup rustpython-parser | ⚪ | - | |
| P5.2.2 | Implement danger pattern matchers | ⚪ | - | |
| P5.2.3 | Implement `scan_package()` | ⚪ | - | |
| P5.2.4 | Implement `SecurityReport` | ⚪ | - | |
| P5.2.5 | Write scanner tests with known malware patterns | ⚪ | - | |

---

## P5.3 Venv Module (SDD §3.3)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P5.3.1 | Implement Python detection | ⚪ | - | |
| P5.3.2 | Implement `fork_environment()` | ⚪ | - | |
| P5.3.3 | Implement environment registry | ⚪ | - | |
| P5.3.4 | Write isolation test | ⚪ | - | |

---

## P5.4 Manifest & Lockfile (SDD §3.4)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P5.4.1 | Implement `vortex.toml` parser | ⚪ | - | |
| P5.4.2 | Implement `vortex.lock` parser/writer | ⚪ | - | |
| P5.4.3 | Implement SHA256 hash verification | ⚪ | - | |
| P5.4.4 | Write integrity test | ⚪ | - | |

---

## P5.5 CLI

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P5.5.1 | Implement `vtx install` | ⚪ | - | |
| P5.5.2 | Implement `vtx update` | ⚪ | - | |
| P5.5.3 | Implement `vtx remove` | ⚪ | - | |
| P5.5.4 | Implement `vtx scan` | ⚪ | - | |
| P5.5.5 | Implement `vtx tree` | ⚪ | - | |

---

## Completion Checklist

- [ ] PubGrub correctly resolves deps
- [ ] Scanner detects `os.system` calls
- [ ] Lockfile matches hash verification
- [ ] CLI commands work end-to-end

---

**Unblocks**: Phase 6 (Integration)
