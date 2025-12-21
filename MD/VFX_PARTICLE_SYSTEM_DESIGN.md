# Ultra-High Performance VFX System Design ("Niagara-Lite")

## Overview
This document outlines the design for a **Modular, GPU-Accelerated VFX System** inspired by *Unreal Engine's Niagara*.
The goal is to allow developers to build complex effects (Fire, Magic, Holograms) by stacking "Modules" rather than writing raw code, while running millions of particles on the GPU.

---

## 1. Core Philosophy: The "Module Stack"

Just like Niagara, an Effect is composed of **Emitters**, and each Emitter has a **Stack** of stages:

1.  **System Spawn:** Run once when effect starts (Set global vars).
2.  **System Update:** Run every frame (Global logic).
3.  **Particle Spawn:** Run when a particle is born (Set Initial Position/Color).
4.  **Particle Update:** Run every frame per particle (Gravity, Noise, Collision).
5.  **Renderer:** How to draw it (Sprite, Mesh, Ribbon, Light).

### 1.1 The "Modules"
We do not hardcode "Gravity". We create a **Gravity Module**.
*   **Input:** `Force (vec3)`
*   **Logic:** `Velocity += Force * DeltaTime`
*   **Output:** `Velocity`

---

## 2. Architecture: Graph-to-Shader Compiler

To achieve Niagara's flexibility with WGPU's performance, we cannot interpret nodes at runtime (too slow). We must **Transpile** the module stack into WGSL.

### 2.1 The Pipeline
1.  **Asset Logic (JSON/Graph):** User defines:
    *   `Spawn Rate: 100`
    *   `Update: Gravity(-9.8)`
    *   `Update: CurlNoise(Strength: 5.0)`
2.  **Compiler (Rust):**
    *   Concatenates the WGSL snippets of all used modules.
    *   Generates a `struct Particle { ... }` containing only the fields used (e.g., if we don't use Rotation, don't allocate memory for it).
    *   Compiles a unique Compute Shader for this specific Emitter.
3.  **Runtime (GPU):**
    *   The engine dispatches this custom Compute Shader.
    *   Particles live 100% on GPU buffers.

### 2.2 Data Layout (Structure of Arrays vs AoS)
For GPU coalescing, we generally use **AoS (Array of Structures)** for simplicity in Compute Shaders, though SoA is better for SIMD.
*   *Decision:* Use **AoS** (`struct Particle`) inside a `StorageBuffer`. It's easier to verify in WGSL.

---

## 3. Key Modules to Implement

### 3.1 Simulation Modules
*   **Solver:** Euler or Verlet integration.
*   **Curl Noise:** The "Magic" looking movement (using 3D Simplex Noise).
*   **Point Attraction:** Particles sucked into a black hole.
*   **Collision (Depth Buffer):** Reuse the logic from the Fluid Design (check Scene Depth).

### 3.2 Rendering Modules
*   **Sprite Renderer:** Billboard quads (Smoke, Fire).
*   **Mesh Renderer:** Instanced 3D Meshes (Debris, Rocks).
*   **Ribbon Renderer:** Connect particles with a trail (Sword slashes) - *Advanced*.
*   **Light Renderer:** Each particle emits actual point light (Sparks).

---

## 4. ECS Integration

Similar to the Fluid System:

*   **VfxSystem (Rust):**
    *   Query `Query<(&Transform, &VfxEmitterAsset)>`.
    *   Update Global Uniforms (Emitter Position, Time).
    *   Dispatch Compute Shaders.
*   **VfxEmitterAsset (Resource):**
    *   Holds the compiled `wgpu::ComputePipeline`.
    *   Holds the default property values.

## 5. "Data Interface" (The Niagara Secret)

Niagara can read from other systems. We need that too.
*   **Texture DI:** Sample a texture to determine particle color/position.
*   **Static Mesh DI:** Spawn particles on the surface of a mesh (e.g., burning character).
    *   *Implementation:* Pass the Mesh's Vertex Buffer as a `ReadOnlyStorageBuffer` to the Particle Shader. Randomly pick a triangle and sample position.

## 6. Roadmap

1.  **Architecture:** Build the `ShaderGraph` string builder.
2.  **Core:** Implement Basic Emitter (Spawn + Gravity + Kill).
3.  **Renderer:** Implement Sprite Renderer (Sort back-to-front for transparency).
4.  **Tooling:** We eventually need a Node Graph Editor (IMGOI/egui) to make this usable. Using JSON manually is painful.

---

### Comparison with Unity/Unreal

| Feature | XS Engine (This Design) | Unreal Niagara | Unity VFX Graph |
| :--- | :--- | :--- | :--- |
| **Logic** | Compiled WGSL | Compiled HLSL | Compiled HLSL/Compute |
| **Storage** | GPU Buffers | GPU Buffers / CPU SIMD | GPU Buffers |
| **Flexibility** | High (Code Generation) | Extreme | High |
| **Editor** | TBD (JSON first) | Node Graph | Node Graph |
