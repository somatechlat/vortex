# VORTEX Workspace & Tools Specification
## Complete Development Environment Reference

> **Standard**: ISO/IEC 25051 (Software Package)  
> **Version**: 1.0.0  
> **Status**: PLANNING

---

## 1. WORKSPACE OVERVIEW

### 1.1 What is the VORTEX Workspace?

The **VORTEX Workspace** is the complete user environment for creating, editing, and executing rack-based AI workflows. It consists of:

| Component | Type | Purpose |
|-----------|------|---------|
| **Rack** | Visual | Vertical stack of Units (no infinite canvas) |
| **Signal Bus** | Visual | Unified data lanes for routing |
| **Unit Palette** | Tool | Library of available operations |
| **Property Panel** | Tool | Unit parameter editing |
| **Queue Panel** | Tool | Execution monitoring |
| **Review Panel** | Tool | Human approval and audit review |
| **Template Library** | Tool | Saved and AI-generated templates |
| **Gallery** | Tool | Output viewing |
| **Toolbar** | Tool | Quick actions |

---

## 2. WORKSPACE ARCHITECTURE

### 2.1 Workspace Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            VORTEX WORKSPACE                              │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                         TOOLBAR                                   │  │
│  │  [New] [Save] [Load] | [Undo] [Redo] | [Queue] [Cancel] | [⚙️]    │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌─────────┬─────────────────────────────────────────────────────────┐  │
│  │         │                                                         │  │
│  │  UNIT   │                    RACK                                │  │
│  │ PALETTE │                                                         │  │
│  │         │   ┌─────────┐            ┌─────────┐                    │  │
│  │ 📂 Load │   │  CLIP   │            │ KSamp   │                    │  │
│  │ 📂 Cond │   │  Encode │            │  ler    │                    │  │
│  │ 📂 Samp │   └─────────┘            └────┬────┘                    │  │
│  │ 📂 Imag │                               │                         │  │
│  │         │                               ▼                         │  │
│  ├─────────┤              ┌─────────────────────────┐                │  │
│  │         │              │        VAE Decode       │                │  │
│  │PROPERTY │              └───────────┬─────────────┘                │  │
│  │  PANEL  │                          │                              │  │
│  │         │                          ▼                              │  │
│  │ Steps:  │              ┌───────────────────┐      ┌────────────┐  │  │
│  │ [===20] │              │   Save Image      │      │  REVIEW    │  │  │
│  │         │              └───────────────────┘      │  PANEL     │  │  │
│  │ CFG:    │                                         │  ┌──┐      │  │  │
│  │ [==7.0] │                                         │  │OK│      │  │  │
│  │         │                                         │  └──┘      │  │  │
│  └─────────┴─────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │ STATUS: Ready | Workers: 4/4 | VRAM: 4.2GB/24GB | Queue: 0        │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 3. WORKSPACE TOOLS

### 3.1 Toolbar

| Tool | Icon | Shortcut | Description |
|------|------|----------|-------------|
| **New** | 📄 | Ctrl+N | Create new workflow |
| **Save** | 💾 | Ctrl+S | Save current workflow |
| **Load** | 📂 | Ctrl+O | Open existing workflow |
| **Undo** | Undo | Ctrl+Z | Undo last action |
| **Redo** | Redo | Ctrl+Y | Redo undone action |
| **Queue** | Run | Ctrl+Enter | Execute workflow |
| **Cancel** | Stop | Ctrl+. | Cancel execution |
| **Settings** | ⚙️ | Ctrl+, | Open settings |
| **Gallery** | Gallery | G | View outputs |

### 3.2 Unit Palette (Why It Exists)

**Purpose**: Enable users to discover and add nodes to their workflow.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Categories** | Organized by function | Reduces cognitive load |
| **Search** | Fuzzy matching | Fast node discovery |
| **Favorites** | User-pinned nodes | Quick access to common ops |
| **Recent** | Recently used | Speed up repeated workflows |
| **Preview** | Hover for info | Learn before adding |

### 3.3 Property Panel (Why It Exists)

**Purpose**: Configure selected node parameters with contextual controls.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Context-aware** | Shows only relevant params | Reduces confusion |
| **Live validation** | Real-time error checking | Prevent invalid configs |
| **Widget types** | Sliders, dropdowns, etc. | Appropriate input method |
| **Defaults** | Smart defaults | Lower barrier to start |
| **Reset** | Per-param reset | Easy experimentation |

### 3.4 Queue Panel (Why It Exists)

**Purpose**: Manage and monitor execution queue.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Queue view** | List of pending jobs | Batch workflow support |
| **Progress** | Real-time progress | User feedback |
| **Cancel** | Per-job cancel | Control over execution |
| **Retry** | Re-run failed jobs | Error recovery |
| **History** | Recent executions | Review past runs |

### 3.5 Gallery (Why It Exists)

**Purpose**: View, compare, and manage generated outputs.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Grid view** | Thumbnail overview | Quick browsing |
| **Full view** | Detailed inspection | Quality review |
| **Compare** | Side-by-side | A/B testing |
| **Metadata** | Workflow info | Reproducibility |
| **Actions** | Download, delete, etc. | Asset management |

### 3.6 Rack Overview (Why It Exists)

**Purpose**: Navigate long racks efficiently without an infinite canvas.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Overview** | Scaled view of the rack | Context awareness |
| **Viewport bar** | Current view indicator | Orientation |
| **Click nav** | Jump to position | Fast navigation |
| **Drag nav** | Scroll the rack | Fluid movement |

### 3.7 Review Panel (Human-in-the-Loop)

**Purpose**: Provide mandatory approvals and audit transparency.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Approval summary** | Tool name, version, risk flags | Clear reason for block |
| **Parameter diff** | Before/after values | Prevent silent changes |
| **Decision log** | Approver, timestamp, reason | Audit compliance |
| **Reject workflow** | Reject with mandatory reason | Responsible governance |

### 3.8 Template Library (Why It Exists)

**Purpose**: Store and reuse AI-generated or human-authored templates.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Template cards** | Thumbnail + summary | Fast discovery |
| **Version history** | Revisions with diff | Safe iteration |
| **Fork** | Create variant templates | Reuse without loss |
| **Run with params** | Launch with inputs | Single-click execution |

### 3.9 Agentic Flow Designer (Why It Exists)

**Purpose**: Allow the agent to generate a full rack from intent using MCP tool metadata.

| Feature | Description | Rationale |
|---------|-------------|-----------|
| **Intent prompt** | Natural language command to the agent | Fast flow creation |
| **Tool rationale** | Explain why each tool is used | Expert transparency |
| **Editable rack** | User can modify any generated unit | Human control |
| **Policy preview** | Show approval gates before run | Governance clarity |

---

## 4. NODE TOOLS

### 4.1 Node Types (Complete Catalog)

| Category | Node | Purpose | Inputs | Outputs |
|----------|------|---------|--------|---------|
| **Loaders** | Load Checkpoint | Load model weights | - | MODEL, CLIP, VAE |
| **Loaders** | Load LoRA | Load LoRA fine-tune | MODEL, CLIP | MODEL, CLIP |
| **Loaders** | Load VAE | Load VAE model | - | VAE |
| **Loaders** | Load Image | Load image file | - | IMAGE |
| **Conditioning** | CLIP Text Encode | Text to conditioning | CLIP, text | CONDITIONING |
| **Conditioning** | CLIP Set Last Layer | Set CLIP layer | CLIP | CLIP |
| **Conditioning** | Conditioning Combine | Merge conditions | COND, COND | CONDITIONING |
| **Sampling** | KSampler | Diffusion sampling | MODEL, COND+, COND-, LATENT | LATENT |
| **Sampling** | KSampler Advanced | Extended options | MODEL, COND+, COND-, LATENT | LATENT |
| **Latent** | Empty Latent Image | Create empty latent | dimensions | LATENT |
| **Latent** | Latent Upscale | Upscale latent | LATENT | LATENT |
| **Image** | Save Image | Save to disk | IMAGE | - |
| **Image** | Preview Image | Preview in UI | IMAGE | - |
| **Image** | Image Scale | Resize image | IMAGE | IMAGE |
| **VAE** | VAE Decode | Latent to image | VAE, LATENT | IMAGE |
| **VAE** | VAE Encode | Image to latent | VAE, IMAGE | LATENT |
| **Utils** | Reroute | Wire routing | ANY | ANY |
| **Utils** | Note | Text annotation | - | - |
| **Utils** | Primitive | Value constant | - | VALUE |

### 4.2 Node Structure

```
┌───────────────────────────────────────┐
│ ● Load Checkpoint                     │  ← Header (type + badge)
├───────────────────────────────────────┤
│                                       │
│  Model: [SD1.5_base.safetensors  ▼]  │  ← Widgets
│                                       │
├───────────────────────────────────────┤
│  ○ MODEL    ○ CLIP    ○ VAE          │  ← Output ports
└───────────────────────────────────────┘
```

### 4.3 Port Types

| Type | Color | Data |
|------|-------|------|
| **MODEL** | 🟣 Purple | Diffusion model |
| **CLIP** | 🟡 Yellow | Text encoder |
| **VAE** | 🔴 Red | Image encoder/decoder |
| **CONDITIONING** | 🟠 Orange | Text embeddings |
| **LATENT** | 🟣 Pink | Latent space tensors |
| **IMAGE** | Done Green | Pixel images |
| **MASK** | Pending White | Binary masks |
| **INT** | 🔵 Blue | Integer values |
| **FLOAT** | 🔵 Cyan | Float values |
| **STRING** | Pending Gray | Text strings |

---

## 5. EDITING TOOLS

### 5.1 Selection Tools

| Action | Trigger | Effect |
|--------|---------|--------|
| **Single select** | Click node | Select one node |
| **Multi-select** | Ctrl+Click | Add to selection |
| **Marquee select** | Drag on canvas | Select enclosed nodes |
| **Select all** | Ctrl+A | Select all nodes |
| **Deselect** | Click empty | Clear selection |
| **Invert** | Ctrl+I | Invert selection |

### 5.2 Transform Tools

| Action | Trigger | Effect |
|--------|---------|--------|
| **Move** | Drag node | Reposition node |
| **Multi-move** | Drag selection | Move all selected |
| **Align left** | Menu | Align to left edge |
| **Align center** | Menu | Center horizontally |
| **Distribute** | Menu | Even spacing |

### 5.3 Connection Tools

| Action | Trigger | Effect |
|--------|---------|--------|
| **Connect** | Drag port→port | Create edge |
| **Disconnect** | Right-click edge | Remove edge |
| **Reroute** | Ctrl+Click edge | Add reroute point |
| **Reconnect** | Drag edge end | Move connection |

### 5.4 Organization Tools

| Action | Trigger | Effect |
|--------|---------|--------|
| **Group** | Ctrl+G | Create visual group |
| **Ungroup** | Ctrl+Shift+G | Remove group |
| **Color** | Right-click | Set node color |
| **Add note** | N | Add text annotation |
| **Bypass** | B | Skip node in execution |
| **Pin** | P | Prevent node movement |

---

## 6. NAVIGATION TOOLS

### 6.1 Pan & Zoom

| Action | Trigger | Effect |
|--------|---------|--------|
| **Pan** | Middle-mouse drag | Move viewport |
| **Pan** | Space + drag | Move viewport |
| **Zoom in** | Scroll up | Increase zoom |
| **Zoom out** | Scroll down | Decrease zoom |
| **Zoom to 100%** | 1 key | Reset zoom |
| **Zoom to fit** | F key | Fit all nodes |
| **Zoom to selection** | Ctrl+F | Fit selected nodes |

### 6.2 View Tools

| Action | Trigger | Effect |
|--------|---------|--------|
| **Toggle grid** | G key | Show/hide grid |
| **Toggle minimap** | M key | Show/hide minimap |
| **Toggle UI** | H key | Hide all panels |
| **Focus node** | Double-click | Center on node |

---

## 7. FILE TOOLS

### 7.1 Import/Export Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| **VORTEX Native** | `.vtx` | Default save format |
| **ComfyUI** | `.json` | ComfyUI compatibility |
| **PNG Workflow** | `.png` | Embedded in image |
| **API** | `.json` | Programmatic use |

### 7.2 File Operations

| Operation | Shortcut | Description |
|-----------|----------|-------------|
| **New** | Ctrl+N | Start fresh workflow |
| **Open** | Ctrl+O | Load from file |
| **Save** | Ctrl+S | Save current |
| **Save As** | Ctrl+Shift+S | Save with new name |
| **Export** | Ctrl+E | Export to format |
| **Import** | Ctrl+I | Import from format |

---

## 8. EXECUTION TOOLS

### 8.1 Queue Controls

| Action | Trigger | Effect |
|--------|---------|--------|
| **Queue Prompt** | Ctrl+Enter | Add to queue |
| **Cancel Current** | Ctrl+. | Stop running job |
| **Clear Queue** | Ctrl+Shift+. | Remove pending |
| **Pause Queue** | - | Hold new jobs |
| **Resume Queue** | - | Continue processing |

### 8.2 Debug Tools

| Tool | Purpose |
|------|---------|
| **Node Preview** | See intermediate outputs |
| **Tensor Inspector** | View tensor shapes/values |
| **Execution Log** | Step-by-step trace |
| **Memory View** | VRAM allocation |
| **Timing View** | Node execution times |

---

## 9. COLLABORATION TOOLS

### 9.1 Multi-User Features

| Feature | Description |
|---------|-------------|
| **Presence** | See other users' cursors |
| **Selection sync** | Real-time selection visibility |
| **Edit sync** | Instant graph changes |
| **Chat** | In-workspace communication |
| **Permissions** | View/edit access control |

### 9.2 Sharing Tools

| Action | Effect |
|--------|--------|
| **Share Link** | Generate shareable URL |
| **Invite User** | Send workspace invitation |
| **Export Template** | Create reusable template |
| **Publish** | Make publicly available |

---

## 10. CUSTOM NODE DEVELOPMENT

### 10.1 Custom Node Structure

| Component | Purpose | Required |
|-----------|---------|----------|
| **Class** | Node implementation | Complete |
| **INPUT_TYPES** | Port/widget definitions | Complete |
| **RETURN_TYPES** | Output port types | Complete |
| **FUNCTION** | Execution method name | Complete |
| **CATEGORY** | Palette location | Complete |
| **display_name** | UI name | Reject |
| **description** | Tooltip text | Reject |

### 10.2 Widget Types Available

| Widget | For | Description |
|--------|-----|-------------|
| `INT` | Integers | Number input with bounds |
| `FLOAT` | Decimals | Slider or input |
| `STRING` | Text | Single/multi-line input |
| `BOOLEAN` | Toggle | Checkbox |
| `COMBO` | Enum | Dropdown selection |
| `COLOR` | Color | Color picker |
| `IMAGE` | File | Image upload |

---

**Document Status**: COMPLETE  
**Total Tools Documented**: 50+  
**Total Node Types**: 18+  
**Ready for Implementation**: Complete
