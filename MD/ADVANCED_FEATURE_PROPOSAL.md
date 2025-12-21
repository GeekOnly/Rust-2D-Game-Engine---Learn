# Advanced Feature Proposal: Beyond AAA

## Overview
You asked: *"Is there anything better/additional?"*
While Tuanjie/Unity are catching up to standards, the **Bleeding Edge** of Game Tech (used by Unreal 5, Call of Duty, Frostbite) has moved to new paradigms.
Since we are writing a custom Engine in Rust/WGPU, we have the unique opportunity to implement these **"Next-Gen"** architectures that Unity cannot easily switch to.

---

## 1. Visibility Buffer (The "End Game" of Rendering)
**Status Quo:** Unity/Unreal use **Deferred Rendering** (G-Buffer stores Albedo, Normal, Roughness).
*   *Problem:* Huge memory bandwidth (GBuffer is fat). Limited material models.
*   *Solution:* **Visibility Buffer**.

### How it Works
1.  **Raster Pass:** Write only `InstanceID` (u32) and `TriangleID` (u32) to a generic `R32Uint` texture. (Extremely fast).
2.  **Material Pass (Compute):**
    *   Read the ID.
    *   Fetch Vertex Data (Position, UV) for that specific triangle manually.
    *   Interpolate attributes.
    *   Calculate lighting.
*   **Benefit:**
    *   **Unlimited Material Variety:** You don't need a "Standard Shader". Every pixel fetches its own material logic.
    *   **Tiny Bandwidth:** Writing 1 integer is faster than writing 4 textures.
    *   **Nanite Capability:** This is the foundational tech required for systems like Unreal's Nanite.

---

## 2. SDF Modeling & Blending (The "Dreams" Approach)
**Status Quo:** Modeling in Blender -> UV Unwrap -> Bake -> Import.
*   *Problem:* Destructive workflow. Hard to blend objects (rocks sticking into ground look fake).
*   *Solution:* **SDF (Signed Distance Field) Primitives**.

### How it Works
*   The world is defined by Math functions (`Sphere`, `Box`, `Torus`) combined with smooth operators (`SmoothUnion`).
*   **Renderer:** A Raymarcher (Compute Shader) renders the scene.
*   **Benefit:**
    *   **Smooth Blending:** Objects "melt" into each other (like Clay). Perfect for organic terrain/caves.
    *   **No UVs:** Triplanar mapping handles texturing.
    *   **Destructible:** Subtracting a sphere from the SDF = Real-time holes.

---

## 3. Wave Function Collapse (WFC) Level Gen
**Status Quo:** Placing Modular Kit pieces manually.
*   *Problem:* Tedious.
*   *Solution:* **WFC Constraint Solver**.

### How it Works
*   You define "Rules": *Door must connect to Hallway*. *Window cannot be next to Mirror*.
*   **Algorithm:** The Engine fills a void with blocks that satisfy these rules.
*   **Benefit:** Generate infinite, logically correct dungeons/cities inside the Editor.

---

## 4. WebAssembly (WASM) Editor
**Status Quo:** You must download/install the Engine.
*   *Solution:* **Run the Editor in Chrome**.

### How it Works
*   Rust -> Wasm32-unknown-unknown.
*   WGPU -> WebGPU Backend.
*   **Benefit:**
    *   Send a link to your Artist. They open it, edit the level, and hit Save.
    *   No Git setup, no 20GB install.
    *   Tuanjie/Unity cannot do this nicely (too heavy). Bevy/XS Engine CAN.

---

## Recommendation

If you want to be "Better" than Tuanjie:
1.  **Visibility Buffer:** Adopt this instead of standard Forward/Deferred. It positions you for 2026+ hardware.
2.  **WASM Editor:** This is the "Killer Feature" for a small team. Being able to edit the game on a Chromebook/Tablet via Browser is powerful.
