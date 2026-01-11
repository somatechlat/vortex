# Software Requirements Specification (SRS): UI Screens & Interaction
**Project**: VORTEX-GEN 3.0 "Centaur"
**Module**: Frontend UI (`vortex-ui`)
**Version**: 9.0.0 (ISO Standard)
**Date**: 2026-01-06
**Standard**: ISO/IEC 29148:2018

---

## 1. Introduction

### 1.1 Purpose
This SRS specifies the **Visual and Interaction Design** for the VORTEX "Flow Stream" interface. It replaces the traditional Node Graph with a revolutionary **"Vertical Rack" Paradigm**, focusing on semantic signal flow, glassmorphic aesthetics, and cinematic immersion.

### 1.2 Scope
The UI defines the **"Vortex Rack"**, a vertical stack of atomic processing units connected by a unified **"Signal Bus"**.
**Key Concepts**:
*   **The Signal Bus**: A vertical, multi-lane data highway replacing wires.
*   **The Rack**: A stack of horizontal, glassmorphic "Blade" units.
*   **The Inspector**: A precise, context-aware sidebar for parameter tuning.
*   **Kernel AI**: An embedded LLM agent for natural language control.
*   **Human Approval**: Explicit approval gates with audit-friendly review.

### 1.3 Definitions
| Term | Definition |
| :--- | :--- |
| **Rack Unit (RU)** | A standardized horizontal UI container (e.g., Loader, Sampler). |
| **Signal Bus** | The visual representation of data flow (Vertical Lines). |
| **Blade** | The glassmorphic visual style of a Rack Unit. |
| **Tap** | A connection point where a Unit reads/writes to the Bus. |
| **Soma Identity** | The "Mitchell Hybrid" aesthetic (Mint/Cream/Teal, Geist Font). |
| **Approval Gate** | A UI blade that blocks execution until a human approves. |

---

## 2. Visual Design System (SomaStack)

### 2.1 Color Palette "Mitchell Hybrid"
| Token | Hex | Role | Usage |
| :--- | :--- | :--- | :--- |
| **Void** | `#0a0a0a` | Background | Deepest background layer. |
| **Teal Deep** | `#0F4C5C` | Canvas | The "Cinematic" ambient glow. |
| **Mint** | `#98DDCA` | Axion | Primary Action, Active Signal, Success. |
| **Cream** | `#F5F5F5` | Text | Primary Typography, Icons. |
| **Glass** | `rgba(255,255,255,0.05)` | Surface | Panel backgrounds (Blur 24px). |
| **Bus-1** | `#98DDCA` | Signal | Lane 1 (e.g., Latent). |
| **Bus-2** | `#0F4C5C` | Signal | Lane 2 (e.g., Model). |
| **Bus-3** | `#c4b5fd` | Signal | Lane 3 (e.g., CLIP). |

### 2.2 Typography "Geist"
*   **Headers**: `Geist Sans` (Bold, Tracking -0.02em).
*   **Technical**: `Geist Mono` (Regular, Tabular Numerals).
*   **Scale**:
    *   `Display`: 32px (App Title)
    *   `H1`: 18px (Unit Titles)
    *   `Body`: 14px (Parameters)
    *   `Micro`: 12px (Bus Labels)

### 2.3 Component Tokens
*   **Radius**: `18px` (Standard for all Blades and Panels).
*   **Border**: `1px solid rgba(255,255,255,0.1)` (Inner Glow).
*   **Shadow**: `0 20px 40px rgba(0,0,0,0.4)` (Cinematic Depth).
*   **Blur**: `backdrop-filter: blur(24px)` (Heavy Frostation).

---

## 3. Screen Specifications

### 3.1 Screen S-001: The Vortex Rack (Main Workspace)
**ID**: `SCR-MAIN`
**Description**: The primary interface for creating generative flows. A full-screen, frameless environment.

#### 3.1.1 Layout Structure
1.  **Global Background**:
    *   A deep, animating gradient (`#0a0a0a` to `#0F4C5C`).
    *   **Bloom**: A central, soft-focus "Bloom" representing the creative potential.
    *   **Grid**: Faint, precision grid lines (5% opacity).
2.  **The Signal Bus (Left)**:
    *   **Location**: Fixed, Left of Center.
    *   **Visual**: 8 parallel glowing lines running Top-to-Bottom.
    *   **Behavior**: Lines pulse when data flows.
3.  **The Rack (Center)**:
    *   **Location**: Centered interactions.
    *   **Content**: Stack of "Rack Units" (S-002, S-003...).
    *   **Interaction**: Sortable via Drag-and-Drop.
4.  **The Sidebar (Right)**:
    *   **Location**: Fixed Right (Collapsible).
    *   **Content**: Signal Inspector (S-100) + Kernel AI (S-101).
5.  **The Toolbox (Bottom)**:
    *   **Location**: Fixed Bottom Center.
    *   **Visual**: Floating Glass Pill (MacOS Dock style).

#### 3.1.2 Interactions
*   **Scroll**: Vertical scroll moves the entire Rack up/down.
*   **Semantic Zoom**:
    *   **Zoom Out**: See the entire "Pipeline".
    *   **Zoom In**: Focus on a single "Blade" to edit params inline.

---

### 3.2 Screen S-002: Rack Unit "Blade" (Generic)
**ID**: `CMP-BLADE`
**Description**: The atomic container for a processing node.

#### 3.2.1 Visual Anatomy
```
┌────────────────────────────────────────────────────────┐
│  ┌─┐  LOADER // JUGGERNAUT_XL             [::: Handle] │
│  │•│←─(Tap)                                            │
│  └─┘                                                   │
│      [ Thumbnail / Content Area ]                      │
│                                                        │
│   STATUS: READY 🟢                   VRAM: 4.2GB       │
└────────────────────────────────────────────────────────┘
```
*   **Tap (Left)**: A connector dot that aligns with a Bus Lane. Clicking it toggles connection.
*   **Header**: `Geist Mono` Uppercase. Title of the unit.
*   **Handle (Right)**: Drag handle for reordering.
*   **Body**: Context-sensitive content (Image, Text, Graph).

### 3.3 Screen S-003: Specific Unit Types

#### A. Loader Unit
*   **Content**: Large preview of the Checkpoint's cover art.
*   **Controls**: Dropdown for `ckpt_name`, `vae`, `clip_skip`.
*   **Visual**: Background of the Blade is the blurred cover art.

#### B. Prompt Unit
*   **Content**: A clean, multi-line `textarea` using `Geist Mono`.
*   **Syntax Highlighting**: Weights `(word:1.2)` are highlighted Mint.
*   **Controls**: Positive/Negative toggle tabs.

#### C. Sampler Unit
*   **Content**: A real-time **Denoise Graph** (Waveform).
*   **Animation**: The waveform flattens as noise is removed (Entropy reduction).
*   **Controls**: `steps`, `cfg`, `sampler_name` sliders.

---

### 3.4 Screen S-100: Signal Inspector
**ID**: `PNL-INSPECT`
**Description**: The precision engineering panel.

#### 3.4.1 Content
*   **Header**: "SIGNAL INSPECTOR"
*   **Bus Status**: A live readout of the 8 bus lanes.
    *   `L1: LATENT [1, 4, 64, 64] (ACTIVE)`
    *   `L2: IMAGE [Empty]`
*   **Node Properties**: Detailed table of every input/output for the *selected* Rack Unit.
*   **Execution Metrics**: `Time: 12.4s`, `VRAM Peak: 8.1GB`.

### 3.5 Screen S-101: Kernel AI
**ID**: `PNL-KERNEL`
**Description**: The Chat interface for the "Centaur" agent.

#### 3.5.1 Content
*   **Location**: Bottom of Sidebar.
*   **Input**: "Enter command or query kernel..."
*   **Response**: Streaming text responses in `Geist Mono`.
*   **Capabilities**: Can auto-configure rack units based on natural language and MCP tool metadata (e.g., "Set up a flow for 25 character variations").
*   **Explanations**: Provides a "Why this flow" breakdown of tool choices and parameter defaults.

### 3.6 Screen S-102: Approval Gate + Review Panel
**ID**: `PNL-APPROVAL`
**Description**: The explicit human-in-the-loop checkpoint for sensitive actions.

#### 3.6.1 Content
*   **Location**: Sidebar (between Inspector and Kernel AI).
*   **Header**: "APPROVAL REQUIRED"
*   **Summary**:
    *   Tool name + version
    *   Risk flags (e.g., External IO, High Cost, Data Access)
    *   Estimated cost (GPU seconds, tokens)
*   **Diff View**:
    *   `param_name: before -> after` table
*   **Actions**:
    *   **Approve** (Mint)
    *   **Reject** (Destructive red)
    *   **Add Reason** (required for rejection)
*   **Audit Banner**: "All decisions are logged."

---

## 4. User Flow Sequences

### 4.1 Flow F-01: Creating a Flow
1.  User opens Vortex. **SCR-MAIN** loads (Empty Rack).
2.  User clicks "+" on Toolbox.
3.  User selects "Basic Pipeline".
4.  System injects 3 Units: **Loader**, **Prompt**, **Sampler**.
5.  System auto-taps them to Bus Lanes 1, 2, 3.
6.  User types "Cyberpunk City" in **Prompt Unit**.
7.  User clicks "Render" (Toolbox).
8.  **Sampler Unit** visualizes the denoise process.
9.  Final Image appears in a new **Display Unit** at bottom.

### 4.2 Flow F-02: Tuning a Signal
1.  User clicks on the **Sampler Unit**.
2.  **Signal Inspector** (S-100) slides out from right.
3.  User adjusts "CFG Scale" slider.
4.  User sees real-time breakdown of VRAM impact in Inspector.

### 4.3 Flow F-03: Human Approval Gate
1.  User initiates a run that triggers a policy gate.
2.  The Rack inserts an **Approval Gate Blade** at the triggering step.
3.  The **Approval Panel** opens with tool metadata and parameter diff.
4.  User clicks **Approve** or **Reject**.
5.  The decision is transmitted to Core and recorded in audit log.

### 4.4 Flow F-04: Agent-First Flow Creation (Expert User)
1.  User opens Vortex. **SCR-MAIN** loads (Empty Rack).
2.  User opens **Kernel AI** and asks: "I need a flow for this image with 25 variations."
3.  Agent generates a full rack using MCP tool metadata and inserts the complete flow.
4.  Agent displays rationale: tool selection, seed strategy, and identity preservation.
5.  User requests changes (e.g., "Change sampler; explain why").
6.  Agent applies changes and updates rationale and estimated cost.
7.  User runs the flow; Approval Gate appears if required.

---

## 5. Transition Matrix

| From | Action | To | Transition Style |
| :--- | :--- | :--- | :--- |
| **Main Rack** | Click Inspector Toggle | **Inspector Open** | Sidebar slides in (Spring physics). |
| **Main Rack** | Click Kernel Icon | **Kernel Open** | Chat expands from bottom-right. |
| **Main Rack** | Scroll Down | **Rack Scroll** | Parallax scroll (Bus moves slower than Units). |
| **Unit** | Drag Handle | **Reorder** | Unit lifts (`z-index`), others yield gap. |
