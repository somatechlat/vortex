# Phase 4: Frontend UI

> **Duration**: Weeks 10-12  
> **Dependencies**: Phase 2  
> **Status**: ⚪ Blocked  
> **SDD Reference**: [02_frontend_ui_sdd.md](../docs/design/02_frontend_ui_sdd.md)

---

## P4.1 State Management (SDD §3.1)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.1.1 | Create `rack.svelte.ts` with $state | ⚪ | - | |
| P4.1.2 | Create `bus.svelte.ts` | ⚪ | - | |
| P4.1.3 | Create `selection.svelte.ts` | ⚪ | - | |
| P4.1.4 | Create `execution.svelte.ts` | ⚪ | - | |
| P4.1.5 | Implement mutation recording for undo/redo | ⚪ | - | |

---

## P4.2 Canvas Rendering (SDD §3.3)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.2.1 | Implement WebGL2 context setup | ⚪ | - | |
| P4.2.2 | Implement cinematic background shader | ⚪ | - | |
| P4.2.3 | Implement breathing/bloom animation | ⚪ | - | |
| P4.2.4 | Implement progress-driven brightness | ⚪ | - | |
| P4.2.5 | Test GPU usage < 10% | ⚪ | - | |

---

## P4.3 Rack Components (SDD §3.2)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.3.1 | Create `Rack.svelte` with drag handling | ⚪ | - | |
| P4.3.2 | Implement FLIP animations for reorder | ⚪ | - | |
| P4.3.3 | Create `Blade.svelte` (generic unit) | ⚪ | - | |
| P4.3.4 | Create `LoaderBlade.svelte` | ⚪ | - | |
| P4.3.5 | Create `PromptBlade.svelte` | ⚪ | - | |
| P4.3.6 | Create `SamplerBlade.svelte` | ⚪ | - | |
| P4.3.7 | Create `TapIndicator.svelte` | ⚪ | - | |

---

## P4.4 Signal Bus Component

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.4.1 | Create `SignalBus.svelte` | ⚪ | - | |
| P4.4.2 | Implement lane highlighting on hover | ⚪ | - | |
| P4.4.3 | Implement pulse animation on data flow | ⚪ | - | |
| P4.4.4 | Implement tap connection logic | ⚪ | - | |

---

## P4.5 Collaboration (SDD §3.4)

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.5.1 | Setup Yjs document | ⚪ | - | |
| P4.5.2 | Setup y-websocket provider | ⚪ | - | |
| P4.5.3 | Sync Yjs ↔ Svelte store | ⚪ | - | |
| P4.5.4 | Implement cursor awareness | ⚪ | - | |
| P4.5.5 | Write multi-user sync test | ⚪ | - | |

---

## P4.6 Panels & Overlays

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.6.1 | Create `Inspector.svelte` | ⚪ | - | |
| P4.6.2 | Create `KernelAI.svelte` | ⚪ | - | |
| P4.6.3 | Create `Toolbox.svelte` | ⚪ | - | |
| P4.6.4 | Create `ContextMenu.svelte` | ⚪ | - | |
| P4.6.5 | Create `Toast.svelte` notifications | ⚪ | - | |

---

## P4.7 Services

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.7.1 | Implement HTTP client with fetch | ⚪ | - | |
| P4.7.2 | Implement WebSocket client | ⚪ | - | |
| P4.7.3 | Implement reconnection logic | ⚪ | - | |
| P4.7.4 | Implement localStorage persistence | ⚪ | - | |

---

## P4.8 Design System

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.8.1 | Create `tokens.css` with design tokens | ⚪ | - | Mitchell Hybrid |
| P4.8.2 | Create `reset.css` | ⚪ | - | |
| P4.8.3 | Create `typography.css` with Geist fonts | ⚪ | - | |
| P4.8.4 | Create `animations.css` | ⚪ | - | |

---

## P4.9 Testing

| ID | Task | Status | Assignee | Notes |
|----|------|--------|----------|-------|
| P4.9.1 | Setup Playwright | ⚪ | - | |
| P4.9.2 | Write `rack.spec.ts` | ⚪ | - | |
| P4.9.3 | Write `blade.spec.ts` | ⚪ | - | |
| P4.9.4 | Write `bus.spec.ts` | ⚪ | - | |
| P4.9.5 | Write `collab.spec.ts` | ⚪ | - | |
| P4.9.6 | Setup visual regression | ⚪ | - | |

---

## Completion Checklist

- [ ] 60fps rack reorder animations
- [ ] Signal Bus routing works
- [ ] WebGL background < 10% GPU
- [ ] Yjs sync works between 2 clients

---

**Unblocks**: Phase 6 (Integration)
