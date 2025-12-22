# Infinite Terrain & Stylized Grass System Design

## Overview
This document outlines the design for a **High-Performance Infinite Terrain System** with AAA-quality **Stylized Grass**.
The goal is to replicate the "Lush, Tactile" feel of top-tier Unity/Unreal assets but optimized for **Mobile Devices** (where the reference asset failed) using **GPU-Driven Rendering**.

---

## 1. The Terrain Architecture (Infinite Worlds)

To support "Infinity", we cannot load one giant mesh. We use a **Chunk-based LOD System**.

### 1.1 CDLOD (Continuous Distance-Dependent Level of Detail)
*   **Quadtree Structure:** The world is divided into a Quadtree.
*   **Node Selection:** Nodes close to the camera are high-res (LOD0). Nodes far away are low-res (LOD4).
*   **Morphing:** Vertex shader morphs geometry between LOD levels to prevent "popping".

### 1.2 Data Streaming
*   **Heightmap:** Stored as `R16` textures. Streamed in chunks.
*   **Splatmap:** Controls texture blending (Grass, Dirt, Snow).

---

## 2. The "Infinity Grass" System (GPU-Driven)

The Unity asset mentioned "Not suitable for mobile" because it likely relies on standard Geometry Shaders or heavy Overdraw.
We will use **Compute Shader + Indirect Draw** to render millions of blades efficiently on mobile.

### 2.1 The Pipeline
1.  **Generation (Compute Shader):**
    *   Input: Terrain Heightmap + Density Map.
    *   Logic: Each thread corresponds to a patch of ground.
    *   **Culling:** Check if blade is inside Camera Frustum AND within Max Distance.
    *   **Output:** Append visible blade instances to a `StorageBuffer<InstanceData>`.
2.  **Rendering (Draw Indirect):**
    *   The CPU knows *nothing* about the blade count.
    *   We call `render_pass.draw_indirect(buffer)` which reads the count from the GPU.
    *   *Result:* Zero CPU overhead.

### 2.2 Stylized Grass Rendering
To achieve the "Ghibli/Anime" look:
*   **Normal Adjustment:** Point vertex normals **Upwards** (0, 1, 0) instead of matching the mesh curvature. This makes lighting smooth and "fluffy".
*   **Translucency:** Simulate sun passing through by adding a "Backlight" term: `dot(ViewDir, -LightDir)`.
*   **Terrain Blending:** Sample the Terrain Splatmap color and blend it into the bottom of the grass blade.

---

## 3. Interaction System (Tactile Grass)

Making the grass bend when the player walks.

### 3.1 The "Interactor Buffer" Approach
We do not use Physics Colliders for grass (too slow).
1.  **Global Trail Texture:** A low-res RenderTarget (e.g., 512x512) centered on the player.
2.  **Painting:** Every frame, draw the Player's position as a "Brush" into this texture (fading out over time).
3.  **Vertex Shader:**
    *   Sample the Trail Texture.
    *   If value > 0, **push** the grass vertex away from the center.
    *   `VertexPos.xz += PushDir * TrailValue`.
    *   `VertexPos.y -= TrailValue * 0.5` (Flatten).

---

## 4. Mobile Optimization (The Secret Sauce)

How to make this run on mobile when others fail?

### 4.1 Solving Overdraw
Grass usually kills mobile GPUs because of transparent pixels (Overdraw).
*   **Technique 1: Opaque First:** Sort grass chunks front-to-back.
*   **Technique 2: Z-Prepass:** Render a "Depth Only" pass of the grass first (fast) then render color.
*   **Technique 3: Density Scaling:** Reduce grass density based on distance. Far away grass = Less blades, but Thicker blades.

### 4.2 Early Exit Culling
The Compute Shader ensures we *never* submit a blade that is behind the camera or too far away. This creates the "Infinity" illusion without the cost.

---

## 5. ECS Integration

```rust
#[derive(Component)]
struct TerrainChunk {
    pub lod_level: u8,
    pub heightmap: Handle<Image>,
}

#[derive(Component)]
struct GrassField {
    pub density_map: Handle<Image>,
    pub wind_params: WindSettings,
}

#[derive(Resource)]
struct WindSettings {
    pub strength: f32,
    pub direction: Vec2,
    pub noise_scale: f32,
    pub time_scale: f32,
}
```

## 6. Comparison

| Feature | Standard Unity/Unreal Terrain | **XS Engine (Proposed)** |
| :--- | :--- | :--- |
| **Rendering** | CPU Batching / Instancing | **GPU Indirect Draw** (Zero CPU) |
| **LOD** | Distance distance | **CDLOD** (Morphing) |
| **Grass** | GameObjects / Painted | **Procedural Compute** |
| **Interaction** | Expensive Physics | **Trail Texture** (Cheap) |
| **Mobile** | Heavy | **Optimized** (Culling/Scaling) |
