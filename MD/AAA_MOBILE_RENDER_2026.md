# Update to AAA Mobile Render 2026: ECS-Driven Architecture

**Target**: AAA Visual Quality on High-End Mobile Devices (iOS/Android) while maintaining 60 FPS.
**Core Philosophy**: Leverage the ECS architecture to feed a Data-Oriented Render Pipeline, minimizing CPU overhead and maximizing GPU bandwidth efficiency for Tile-Based Deferred Renderers (TBDR).

---

## ☁️ Architectural Overhaul: The "Render Proxy" Pattern

Currently, `render_game_world` iterates over the ECS `World` directly. This couples simulation logic with rendering and hurts cache locality.

### 1.1 The Extraction Phase (CPU)
Instead of the Renderer querying the World, we introduce a **Render Extraction System**.
- **Input**: ECS World (Transforms, Meshes, Materials, Visible Flags).
- **Process**:
  1.  **Cull**: Frustum culling using ECS Spatial Acceleration (e.g., BVH or Grid from `MultiplayerArchitecture`).
  2.  **Extract**: Copy *only* the data needed for one frame into a flat, linear structure called the **Render Frame**.
- **Output**: `RenderFrame` struct.
  ```rust
  struct RenderFrame {
      pub camera_data: CameraUniform,
      pub opaque_instances: Vec<InstanceData>, // Linear memory = Fast iteration
      pub transparent_instances: Vec<InstanceData>,
      pub lights: Vec<LightData>,
      pub ui_commands: Vec<UiCommand>,
  }
  ```
- **Benefit**: The Renderer no longer locks the ECS World. Simulation can continue (in a multi-threaded future) while Rendering processes the specific frame snapshot.

### 1.2 Data-Oriented GPU Upload
**Current**: Individual `Buffer` and `BindGroup` for every entity. (Draw Call Hell).
**New**: **Global Instance Buffers**.
- Upload all `InstanceData` (Mat4 Model, Color, UV Offset) into **one massive Vertex Buffer** per frame (or a dynamic Storage Buffer).
- Use **GPU Instancing** to draw thousands of identical meshes (e.g., Grass, Crates) in a single draw call.
- **Mobile Win**: Drastically reduces Driver Overhead (CPU cost).

---

## 📱 Mobile-First Optimizations (TBDR)

Mobile GPUs (Adreno, Mali, Apple) work differently than Desktop (Nvidia). They render in small tiles.

### 2.1 Render Pass Management (Load/Store Ops)
**Critical Upgrade**: Explicit control over `LoadOp` and `StoreOp`.
- **Depth Buffer**: Set `StoreOp::Discard` (Don't write depth back to main memory).
  - *Why?* Depth is only needed *during* the tile render. Writing it out kills bandwidth/battery.
- **MSAA**: Resolve MSAA *on-chip* within the tile. Never write the multisampled buffer to memory.

### 2.2 Bandwidth Saving
- **Texture Compression**: Implement **ASTC (Adaptive Scalable Texture Compression)** pipeline.
  - Reduces Asset size by 75%.
  - Reduces Memory Bandwidth (less heat).
- **Vertex Input Quantization**:
  - Store Normals/Tangents as `SNorm8` (8-bit) instead of `Float32`.
  - Store UVs as `UNorm16`.
  - Reduces Vertex Fetch bandwidth by ~50%.

### 2.3 Shader Precision
- Force `mediump` (f16) in WGSL where possible.
- Use `f16` for Color calculations and `f32` only for Position/Depth.
- **Result**: 2x Register Throughput on many mobile GPUs.

---

## 🎨 Advanced Features for 2026

### 3.1 Clustered Forward Rendering (Lite)
**Goal**: Support 100+ dynamic lights on mobile.
- CPU computes which lights affect which "Cluster" (3D chunk of screen).
- Shader reads light list from a Storage Buffer.
- **Result**: "Console-quality" lighting without the cost of Deferred Rendering (G-Buffer is too heavy for mobile bandwidth).

### 3.2 GPU-Driven Culling (Compute Shader)
- Move Frustum Culling and Occlusion Culling from CPU layout to a **Compute Shader**.
- Send raw Instance list to GPU -> GPU filters visible items -> GPU writes Indirect Draw buffer.
- **Result**: CPU sends *everything*, GPU decides what to draw. 0 CPU impact for complex scenes.

### 3.3 Stylized PPR (Programmable Post-Processing)
- Implement a **Frame Graph** to manage dependencies.
- Pass: Sun Shafts (Bloom).
- Pass: Color Grading (LUT).
- Pass: Anime Outline (Depth/Normal edge detection).

---

## 📅 Roadmap Execution

| Phase | Feature | Tech Stack Integration |
| :--- | :--- | :--- |
| **Q1** | **Extraction Layer** | Create `RenderFrame` struct. Decouple `render_system.rs` from `World`. |
| **Q2** | **Instancing Core** | Rewrite `MeshRenderer` to use one big Instance Buffer instead of per-entity binds. |
| **Q3** | **Mobile Tuning** | ASTC Support, Shader f16 optimization, LoadOp/StoreOp audit. |
| **Q4** | **Next-Gen Light** | Implement Clustered Forward Rendering for multi-light support. |
