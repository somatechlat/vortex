# VORTEX FLOW STREAM: The UI Revolution
> **Mandate**: "Eliminate the spaghetti. Revolutionize the design."
> **Philosophy**: Complexity should be felt, not seen.
> **Visual Reference**: SomaStack "Mitchell Hybrid" (Mint/Cream/Teal, Glassmorphism, Geist)

---

## 1. THE CORE PROBLEM: "THE SPAGHETTI GRAPH"
The current "industry standard" (ComfyUI) logic is flawed for humans:
- **Visual Noise**: 50% of the screen is wires.
- **Cognitive Load**: Users spend more time routing cables than creating art.
- **Mental Model**: Engineers think in "Graphs". Creators think in "Flows".

## 2. THE REVOLUTION: "VERTICAL FLOW RACK"
We are abandoning the absolute 2D open canvas for a **Vertical "Rack" Paradigm** (Vortex Flow Stream).

### 2.1 The Concept: "The Signal Bus"
Instead of wires flying everywhere, a structured **Signal Bus** runs vertically along the left text spine.
- **Visual**: A set of fine, glowing parallel lines (The Bus).
- **Tapping**: Rack Units "tap" into the bus. A dot indicates a connection.
- **Clarity**: You can see exactly where data enters and leaves the stack.

### 2.2 The Layout: Vertical Stack
Nodes are organized into **"Rack Units"** (Full-width horizontal blades).
- **The Stack**: Units stack vertically (Loader -> Prompt -> Sampler -> Output).
- **Order**: Processing hits the top unit and flows down.
- **Inspector**: A dedicated sidebar on the right for deep parameter tuning ("Signal Inspector").

### 2.3 The Aesthetic: "Cinematic Rack"
- **Background**: Deep void with subtle "Cyber-Structure" lines.
- **Glass Units**: Each Rack Unit is a frosted glass blade.
- **Typography**: `Geist Mono` for technical details, `Geist Sans` for headers.
- **Palette**: Dark Mode. Mint/Teal accents for active signals.

---

## 3. UI ARCHITECTURE & SCREENS

### 3.1 The "Vortex Rack" (Main Screen)
**Structure**:
1.  **The Signal Bus (Left)**: Vertical multi-lane highway for Latents, VAE, Models.
2.  **The Rack (Center)**: Stack of Glass Units.
    - *Loader Unit*: Checkpoint selector.
    - *Prompt Unit*: Text input area.
    - *Sampler Unit*: Progress bars and denoise graphs.
3.  **The Sidebar (Right)**:
    - *Signal Inspector*: Details of the selected signal.
    - *Kernel AI*: Chat interface for the agent.

### 3.2 Interaction Model
- **Add Unit**: Click separating space between units to inject a new one.
- **Reorder**: Drag units up/down.
- **Connect**: Click a Bus Lane to toggle connection to a Unit.

### 3.2 "Semantic Channels" (The Wire Replacement)
Instead of a curve connecting A to B:
- **Input Slot**: Display a small "Pill" showing the *Source Name* (e.g., `⟪ Loader 01`).
- **Interaction**: Click the Pill -> Opens "Signal Map" (a radar-like view showing available signals of that type).
- **Benefit**: Removes 100% of visual cable clutter. Connections are readable as *data*, not *lines*.

### 3.3 The "Smart Palette"
Do not show a list of 500 nodes.
- **Context Aware**: If I select an "Image" output, and press Space, the palette *only* shows nodes that accept "Image" (Upscalers, Savers, Previews).
- **Search First**: A Spotlight-like command bar (`Cmd+K`) is the primary way to add. "Type 'Face' -> Press Enter -> Added FaceDetailer to current Strip".

---

## 4. VISUAL MOCKUP DESCRIPTION (Mental Render)

**Scene**: A Dark Mode workspace (Deep Teal #0F4C5C).
**Center**: Three "Glass Cards" aligned horizontally.
1.  **Card 1 (Loader)**: Shows a thumbnail of the checkpoint "Juggernaut XL". No ugly text inputs, just the visual.
2.  **Card 2 (Prompt)**: A clean text area, glowing softly.
3.  **Card 3 (Sampler)**: A dynamic visualizer showing the noise denoise process *inside* the card body.

**Connections**: There are NO wires between them. They float near each other. A faint "Mint" aura flows from Card 1 to Card 2, implying connection.

**Background**: A massive, cinematic bloom of the artwork being generated, blurred out.

---

## 5. IMPLEMENTATION STRATEGY

### 5.1 Technology Stack
- **Framework**: Svelte 5 (Runes for fine-grained reactivity).
- **Renderer**: WebGL (PixiJS or custom implementation via `canvas/webgl.ts`).
- **Layout Engine**: `Yoga Layout` (wasm) for the "Strip" auto-layout (Flexbox for canvas nodes).
- **State**: `Yjs` (already correctly spec'd) for real-time collab.

### 5.2 Transition Plan
1.  **Phase 1**: Implement "The Strip" layout engine.
2.  **Phase 2**: Implement "Semantic Channels" (Wireless logic).
3.  **Phase 3**: Apply "Soma Identity" (Glassmorphism/Geist).

---

> "We are not building a graph editor. We are building a flow conductor."
