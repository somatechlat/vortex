# Phase 4: Frontend UI

> **Duration**: Weeks 10-12  
> **Dependencies**: Phase 2  
> **Status**: Pending Blocked  
> **SDD Reference**: [02_frontend_ui_sdd.md](../docs/design/02_frontend_ui_sdd.md)

---

## P4.1 State Management (SDD §3.1)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.1.1 | Create `rack.svelte.ts` with $state | Pending | - | |
| P4.1.2 | Create `bus.svelte.ts` | Pending | - | |
| P4.1.3 | Create `selection.svelte.ts` | Pending | - | |
| P4.1.4 | Create `execution.svelte.ts` | Pending | - | |
| P4.1.5 | Implement mutation recording for undo/redo | Pending | - | |

---

## P4.2 Canvas Rendering (SDD §3.3)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.2.1 | Implement WebGL2 context setup | Pending | - | |
| P4.2.2 | Implement cinematic background shader | Pending | - | |
| P4.2.3 | Implement breathing/bloom animation | Pending | - | |
| P4.2.4 | Implement progress-driven brightness | Pending | - | |
| P4.2.5 | Test GPU usage < 10% | Pending | - | |

---

## P4.3 Rack Components (SDD §3.2)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.3.1 | Create `Rack.svelte` with drag handling | Pending | - | |
| P4.3.2 | Implement FLIP animations for reorder | Pending | - | |
| P4.3.3 | Create `Blade.svelte` (generic unit) | Pending | - | |
| P4.3.4 | Create `LoaderBlade.svelte` | Pending | - | |
| P4.3.5 | Create `PromptBlade.svelte` | Pending | - | |
| P4.3.6 | Create `SamplerBlade.svelte` | Pending | - | |
| P4.3.7 | Create `TapIndicator.svelte` | Pending | - | |

---

## P4.4 Signal Bus Component

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.4.1 | Create `SignalBus.svelte` | Pending | - | |
| P4.4.2 | Implement lane highlighting on hover | Pending | - | |
| P4.4.3 | Implement pulse animation on data flow | Pending | - | |
| P4.4.4 | Implement tap connection logic | Pending | - | |

---

## P4.5 Collaboration (SDD §3.4)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.5.1 | Setup Yjs document | Pending | - | |
| P4.5.2 | Setup y-websocket provider | Pending | - | |
| P4.5.3 | Sync Yjs ↔ Svelte store | Pending | - | |
| P4.5.4 | Implement cursor awareness | Pending | - | |
| P4.5.5 | Write multi-user sync test | Pending | - | |

---

## P4.6 Panels & Overlays

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.6.1 | Create `Inspector.svelte` | Pending | - | |
| P4.6.2 | Create `KernelAI.svelte` | Pending | - | |
| P4.6.3 | Create `Toolbox.svelte` | Pending | - | |
| P4.6.4 | Create `ContextMenu.svelte` | Pending | - | |
| P4.6.5 | Create `Toast.svelte` notifications | Pending | - | |

---

## P4.7 Services

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.7.1 | Implement HTTP client with fetch | Pending | - | |
| P4.7.2 | Implement WebSocket client | Pending | - | |
| P4.7.3 | Implement reconnection logic | Pending | - | |
| P4.7.4 | Implement localStorage persistence | Pending | - | |

---

## P4.8 Design System

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.8.1 | Create `tokens.css` with design tokens | Pending | - | Mitchell Hybrid |
| P4.8.2 | Create `reset.css` | Pending | - | |
| P4.8.3 | Create `typography.css` with Geist fonts | Pending | - | |
| P4.8.4 | Create `animations.css` | Pending | - | |

---

## P4.9 Testing

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.9.1 | Setup Playwright | Pending | - | |
| P4.9.2 | Write `rack.spec.ts` | Pending | - | |
| P4.9.3 | Write `blade.spec.ts` | Pending | - | |
| P4.9.4 | Write `bus.spec.ts` | Pending | - | |
| P4.9.5 | Write `collab.spec.ts` | Pending | - | |
| P4.9.6 | Setup visual regression | Pending | - | |

---

## Completion Checklist

- [ ] 60fps rack reorder animations
- [ ] Signal Bus routing works
- [ ] WebGL background < 10% GPU
- [ ] Yjs sync works between 2 clients

---

**Unblocks**: Phase 6 (Integration)
