# Advanced Material System Design ("White Box" Philosophy)

## Overview
This document outlines the design for a **Fully Open Material System**.
Unlike Unreal Engine or Unity URP—where accessing the raw Shadow Map, Stencil Buffer, or specific G-Buffer data often requires "hacks" or custom render features—our engine handles this natively via a **Render Graph-Aware Material System**.

**Core Philosophy:** "The Material can request *any* resource produced by the Pipeline."

---

## 1. Core Architecture: Render Graph Integration

To allow a material to read the "Shadow Map", the Engine must know:
1.  **Dependency:** This material *needs* the Shadow Map.
2.  **Binding:** The Render Pass (e.g., Forward Pass) must bind the Shadow Map to the material's bind group.

### 1.1 The `PipelineResource` Node
In the Material Graph, we introduce a special node type: **`Pipeline Input`**.

*   **Available Inputs:**
    *   `SceneDepth`
    *   `SceneColor` (for transparency/distortion)
    *   `ShadowMap_Cascade0` (The raw 4k shadow texture)
    *   `ShadowMap_Cascade1`
    *   `VelocityBuffer`
    *   `PreviousFrameColor` (for SSR/Temporal)

### 1.2 Automatic Dependency Resolution
When the Material Compiler sees a `Pipeline Input: ShadowMap` node:
1.  It adds `ShadowMap` to the **Material BindGroupLayout**.
2.  It notifies the **Render Graph**: "Any pass using this material MUST have read-access to the 'ShadowMap' resource."
3.  If you try to use this material in a pass *before* shadows are drawn, the Render Graph throws a validation error (or auto-reorders the passes).

---

## 2. The Material Graph (WGSL Transpiler)

We do not parse nodes at runtime. We transpile to WGSL.

### 2.1 The "Custom Code" Node
For maximum flexibility (like Unreal's Custom HLSL), we provide a **WGSL Block**.
*   **User Input:** Writes raw WGSL code strings.
*   **Inputs:** `float3`, `texture2d`.
*   **Outputs:** `float4`.
*   **Usage:** User can implement their own PCF Shadow Filtering or custom Ray Marching loop using the exposed `Pipeline Input` textures.

### 2.2 Post-Process Materials
Post Processing is treated identically to Surface Materials, but:
*   **Vertex Shader:** Always a "Full Screen Triangle".
*   **Fragment Shader:** Output is just `Color`.
*   **Input:** Implicitly gets `SceneColor`.

---

## 3. Real-Time Material Preview System

How to see the material without running the game.

### 3.1 The "Preview Scene"
A lightweight, separate `World` instance running in the Editor.
*   **Mesh:** Sphere, Cube, or Custom Mesh (User selected).
*   **Lighting:** HDRI Probe + Dir Light (Rotatable).
*   **Background:** Checkerboard or Blur HDRI.

### 3.2 Live Recompilation Pipeline
1.  **User Change:** Connects a Node.
2.  **Debounce:** Wait 50ms.
3.  **Transpile:** Generate WGSL string.
4.  **WGPU CreateShader:** Compile shader.
    *   *If Error:* Display error overlay on the Node Graph (Line number mapping).
    *   *If Success:* Update the Preview Mesh's `RenderPipeline`.

---

## 4. Implementation Strategy (Rust/WGPU)

### 4.1 Data Structures
```rust
#[derive(Clone, Debug)]
pub enum MaterialDomain {
    Surface,    // Standard Mesh
    PostProcess, // Full screen
    Decal,      // Deferred Decal
    Volume,     // Volumetric Cube
}

#[derive(Clone, Debug)]
pub enum PipelineInput {
    // These map to specific textures in the RenderGraph
    GlobalShadowMap,
    SceneDepth,
    GBufferNormal,
}

pub struct MaterialAsset {
    pub domain: MaterialDomain,
    pub nodes: Graph<MaterialNode>,
    pub required_inputs: HashSet<PipelineInput>, // Generated from graph
    pub wgsl_source: String, // Compiled
}
```

### 4.2 The "Uber-BindGroup"
To make this work without recompiling the *Engine* every time:
*   **BindGroup 0:** Global (Camera, Time, **All Pipeline Textures**).
    *   We bind `ShadowMap`, `Depth`, `Normals` to slots 10, 11, 12 globally.
    *   The Material just declares `var shadow_tex: texture_depth_2d_array;` in the shader.
    *   If the material doesn't use it, WGPU optimizes it out (or we just ignore it).
    *   *Pros:* Extremely simple. Every shader has access to everything.
    *   *Cons:* Slight VRAM binding overhead.

---

## 5. Comparison w/ Unreal & Unity

| Feature | Unreal Engine 5 | XS Engine (This Design) |
| :--- | :--- | :--- |
| **Shadow Map Access** | **No** (Hidden in specialized HLSL functions) | **Yes** (Direct Texture Object) |
| **Custom Lighting** | Hard (Shading Models are rigid) | **Flexible** (Write raw lighting loop if wanted) |
| **Compilation** | Slow (C++ ShaderCompiler) | **Fast** (Naga/WGSL) |
| **Preview** | High Quality | **Instant** (WGPU pipeline caching) |

## 6. Use Case Example: "Shadow Art"
**Goal:** Make shadows purple and wavy.
1.  **Unreal:** Very hard. Modify engine source or post-process hack.
2.  **XS Engine:** 
    *   Create Material.
    *   Add node `PipelineInput: ShadowMap`.
    *   Sample ShadowMap at `UV + sin(Time)`.
    *   If `Shadow < 1.0`, Output `Purple`. Else `White`.
    *   Apply to object. Done.
