# VORTEX Design System & Testing Standards

> **Source**: Abstracted from [YACHAQ](https://www.yachaq.org) and [SOMATECH](https://www.somatech.dev)  
> **Standard**: SOMATECH Visual Identity v2.0

---

## 🎨 COLOR PALETTE

### Primary Brand Colors
| Token | Hex | Usage |
|-------|-----|-------|
| `--vtx-accent` | `#FF4D00` | Primary accent (SOMATECH orange) |
| `--vtx-accent-alt` | `#F04E23` | Secondary accent (YACHAQ coral) |
| `--vtx-accent-hover` | `#e13a12` | Accent hover state |

### Neutral Palette (Dark Mode First)
| Token | Hex | Usage |
|-------|-----|-------|
| `--vtx-bg-primary` | `#0a0a0a` | Main background |
| `--vtx-bg-secondary` | `#141414` | Card/panel background |
| `--vtx-bg-tertiary` | `#1f1f1f` | Elevated surfaces |
| `--vtx-bg-quaternary` | `#262626` | Hover states |
| `--vtx-border` | `#2e2e2e` | Borders/dividers |
| `--vtx-border-subtle` | `#1a1a1a` | Subtle separators |

### Text Colors
| Token | Hex | Usage |
|-------|-----|-------|
| `--vtx-text-primary` | `#fafafa` | Primary text |
| `--vtx-text-secondary` | `#a1a1a1` | Secondary/muted text |
| `--vtx-text-tertiary` | `#737373` | Disabled/placeholder |

### Semantic Colors
| Token | Hex | Usage |
|-------|-----|-------|
| `--vtx-success` | `#22c55e` | Success states |
| `--vtx-success-subtle` | `#166534` | Success backgrounds |
| `--vtx-warning` | `#f59e0b` | Warning states |
| `--vtx-error` | `#ef4444` | Error states |
| `--vtx-error-subtle` | `#7f1d1d` | Error backgrounds |
| `--vtx-info` | `#3b82f6` | Info states |

---

## 🔤 TYPOGRAPHY

### Font Stack
```css
:root {
  /* Primary Sans - Geist (SOMATECH/YACHAQ standard) */
  --vtx-font-sans: 'Geist Sans', 'Inter', system-ui, -apple-system, sans-serif;
  
  /* Monospace - Geist Mono */
  --vtx-font-mono: 'Geist Mono', 'JetBrains Mono', 'Fira Code', monospace;
  
  /* Display/Serif - Playfair (optional accent) */
  --vtx-font-serif: 'Playfair Display', Georgia, serif;
}
```

### Font Loading
```html
<!-- Required CDN links -->
<link href="https://cdn.jsdelivr.net/npm/geist@1.0.0/dist/fonts/geist-sans/style.css" rel="stylesheet">
<link href="https://cdn.jsdelivr.net/npm/geist@1.0.0/dist/fonts/geist-mono/style.css" rel="stylesheet">
```

### Type Scale
| Token | Size | Line Height | Usage |
|-------|------|-------------|-------|
| `--vtx-text-xs` | 11px | 1.4 | Captions, badges |
| `--vtx-text-sm` | 13px | 1.5 | Secondary UI text |
| `--vtx-text-base` | 14px | 1.5 | Body text |
| `--vtx-text-md` | 16px | 1.5 | Emphasis |
| `--vtx-text-lg` | 18px | 1.4 | Section headers |
| `--vtx-text-xl` | 24px | 1.3 | Page headers |
| `--vtx-text-2xl` | 32px | 1.2 | Hero text |

### Letter Spacing
| Token | Value | Usage |
|-------|-------|-------|
| `--vtx-tracking-tight` | -0.02em | Headlines |
| `--vtx-tracking-normal` | 0 | Body text |
| `--vtx-tracking-wide` | 0.05em | Uppercase labels |

---

## 📐 SPACING & LAYOUT

### Spacing Scale
| Token | Value | Usage |
|-------|-------|-------|
| `--vtx-space-1` | 4px | Micro gaps |
| `--vtx-space-2` | 8px | Tight padding |
| `--vtx-space-3` | 12px | Default padding |
| `--vtx-space-4` | 16px | Standard gaps |
| `--vtx-space-5` | 20px | Section spacing |
| `--vtx-space-6` | 24px | Card padding |
| `--vtx-space-8` | 32px | Large gaps |
| `--vtx-space-10` | 40px | Section breaks |
| `--vtx-space-12` | 48px | Page margins |

### Border Radius
| Token | Value | Usage |
|-------|-------|-------|
| `--vtx-radius-sm` | 4px | Buttons, inputs |
| `--vtx-radius-md` | 8px | Cards, panels |
| `--vtx-radius-lg` | 12px | Modals, dialogs |
| `--vtx-radius-xl` | 16px | Large containers |
| `--vtx-radius-full` | 9999px | Pills, avatars |

---

## 🎭 COMPONENT PATTERNS

### Card Pattern (from screenshots)
```css
.card {
  background: var(--vtx-bg-secondary);
  border: 1px solid var(--vtx-border);
  border-radius: var(--vtx-radius-md);
  padding: var(--vtx-space-6);
}

.card--elevated {
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);
}
```

### Button Patterns
```css
/* Primary Button */
.btn-primary {
  background: var(--vtx-text-primary);
  color: var(--vtx-bg-primary);
  border-radius: var(--vtx-radius-sm);
  padding: var(--vtx-space-2) var(--vtx-space-4);
  font-weight: 500;
}

/* Secondary/Ghost Button */
.btn-secondary {
  background: transparent;
  border: 1px solid var(--vtx-border);
  color: var(--vtx-text-primary);
  border-radius: var(--vtx-radius-full);
}

/* Pill Button (like screenshots) */
.btn-pill {
  border-radius: var(--vtx-radius-full);
  padding: var(--vtx-space-2) var(--vtx-space-4);
}
```

### Status Badges
```css
.badge--approved { 
  background: var(--vtx-success-subtle); 
  color: var(--vtx-success); 
}
.badge--pending { 
  background: var(--vtx-bg-tertiary); 
  color: var(--vtx-text-secondary); 
}
.badge--rejected { 
  background: var(--vtx-error-subtle); 
  color: var(--vtx-error); 
}
```

---

## 🧪 PLAYWRIGHT TESTING STANDARDS

### Rule 23: Playwright Console Development Policy

| Requirement | Details |
|-------------|---------|
| **Framework** | Playwright (NOT browser_subagent) |
| **Execution** | Console-based via `bun playwright test` |
| **Screenshots** | Save to `tests/screenshots/` |
| **Videos** | Save to `tests/videos/` on failure |
| **Selectors** | Use `data-testid` attributes only |
| **No Agent** | NEVER use browser_subagent for testing |

### Test File Structure
```
tests/
├── e2e/
│   ├── auth.spec.ts       # Authentication flows
│   ├── canvas.spec.ts     # Node graph interactions
│   ├── workflow.spec.ts   # Workflow execution
│   └── settings.spec.ts   # Settings pages
├── integration/
│   ├── api.spec.ts        # API endpoints
│   └── websocket.spec.ts  # WS connections
├── screenshots/           # Captured during tests
├── videos/                # Failure recordings
└── playwright.config.ts   # Configuration
```

### Playwright Configuration
```typescript
// tests/playwright.config.ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 30000,
  retries: 2,
  use: {
    baseURL: 'http://localhost:8188',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { browserName: 'chromium' } },
    { name: 'firefox', use: { browserName: 'firefox' } },
  ],
});
```

### Test ID Convention
```svelte
<!-- ✅ Always use data-testid -->
<button data-testid="queue-button">Queue</button>
<input data-testid="prompt-input" />
<div data-testid="node-{nodeId}">...</div>

<!-- ❌ Never rely on class names or text -->
```

### Common Test Patterns
```typescript
// tests/e2e/canvas.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Canvas', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('[data-testid="canvas-ready"]');
  });

  test('should add a node via drag', async ({ page }) => {
    const nodePalette = page.locator('[data-testid="node-palette"]');
    const canvas = page.locator('[data-testid="canvas"]');
    
    await nodePalette.locator('[data-testid="node-type-loader"]').dragTo(canvas);
    await expect(page.locator('[data-testid^="node-"]')).toHaveCount(1);
  });

  test('should connect two nodes', async ({ page }) => {
    // Add nodes, then connect
    const output = page.locator('[data-testid="port-output-0"]');
    const input = page.locator('[data-testid="port-input-0"]');
    
    await output.dragTo(input);
    await expect(page.locator('[data-testid^="edge-"]')).toHaveCount(1);
  });
});
```

### Console Commands
```bash
# Run all tests
bun playwright test

# Run specific test file
bun playwright test tests/e2e/canvas.spec.ts

# Run with UI mode (debugging)
bun playwright test --ui

# Run headed (visible browser)
bun playwright test --headed

# Generate HTML report
bun playwright show-report

# Update snapshots
bun playwright test --update-snapshots
```

### Screenshot Capture for Debugging
```typescript
// Capture specific element
await page.locator('[data-testid="canvas"]').screenshot({
  path: 'tests/screenshots/canvas-state.png'
});

// Full page screenshot
await page.screenshot({ 
  path: 'tests/screenshots/full-page.png',
  fullPage: true 
});
```

---

## 📋 Quick Reference: tokens.css

```css
/* ui/src/lib/styles/tokens.css */
:root {
  /* Brand */
  --vtx-accent: #FF4D00;
  --vtx-accent-alt: #F04E23;
  
  /* Backgrounds */
  --vtx-bg-primary: #0a0a0a;
  --vtx-bg-secondary: #141414;
  --vtx-bg-tertiary: #1f1f1f;
  
  /* Text */
  --vtx-text-primary: #fafafa;
  --vtx-text-secondary: #a1a1a1;
  
  /* Typography */
  --vtx-font-sans: 'Geist Sans', system-ui, sans-serif;
  --vtx-font-mono: 'Geist Mono', monospace;
  
  /* Spacing */
  --vtx-space-1: 4px;
  --vtx-space-2: 8px;
  --vtx-space-3: 12px;
  --vtx-space-4: 16px;
  --vtx-space-6: 24px;
  
  /* Radius */
  --vtx-radius-sm: 4px;
  --vtx-radius-md: 8px;
  --vtx-radius-lg: 12px;
  
  /* Borders */
  --vtx-border: #2e2e2e;
  
  /* Semantic */
  --vtx-success: #22c55e;
  --vtx-warning: #f59e0b;
  --vtx-error: #ef4444;
}
```

---

**Sources**: 
- YACHAQ Design System (Geist fonts, #F04E23, Playfair Display)
- SOMATECH Visual Identity (#FF4D00 accent, dark-first design)
- Uploaded UI Screenshots (card patterns, pill buttons, status badges)

**Last Updated**: 2026-01-06
