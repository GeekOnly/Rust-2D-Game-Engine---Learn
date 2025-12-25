# AAA Mobile Render Refactor Tasks (2026 Roadmap)

This tracking document details the step-by-step tasks required to refactor the current Game Engine from a "One-Draw-Call-Per-Entity" model to a "Data-Oriented, GPU-Instanced, ECS-Driven" architecture suitable for high-end mobile devices.

---

## 📦 Module: `render` (Low-Level Graphics)
*Goal: Enable the ability to draw thousands of meshes in a single GPU command.*

### Task 1.1: Define Instance Data Structure
- **File**: `render/src/mesh_renderer.rs` (or new `instancing.rs`)
- **Action**: Create a `MeshInstance` struct that matches the GPU buffer layout.
- **Details**:
  ```rust
  #[repr(C)]
  #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
  pub struct MeshInstance {
      pub model_matrix: [[f32; 4]; 4], // Mat4
      pub color: [f32; 4],             // Multiply color
      pub normal_matrix: [[f32; 3]; 3], // Mat3 for normals (optional, or derive in shader)
      pub _padding: [f32; ...],        // Alignment padding
  }
  ```
- **Requirements**: Must derive `wgpu::VertexBufferLayout` with `wgpu::VertexStepMode::Instance`.

### Task 1.2: Upgrade `MeshRenderer` for Batching
- **File**: `render/src/mesh_renderer.rs`
- **Action**: Add methods to create and update distinct `InstanceBuffers`.
- **New Method**: `create_instance_buffer(device, instances: &[MeshInstance]) -> wgpu::Buffer`.
- **New Method**: `render_instanced(...)` that takes the instance buffer and an `instance_count`.

### Task 1.3: Shader Instancing Support
- **File**: `render/src/pbr.wgsl`
- **Action**: Modify Vertex Shader input struct.
- **Changes**:
  - Remove `@group(2) @binding(0) var<uniform> object: ObjectUniform;`
  - Add attributes to `struct VertexInput`:
    - `@location(5) model_0: vec4<f32>` (Rows of Matrix)
    - `@location(6) model_1: vec4<f32>`
    - ...
    - `@location(9) color: vec4<f32>`
  - Reconstruct `mat4x4` inside `vs_main`.

---

## ⚙️ Module: `engine` (Runtime & Systems)
*Goal: Efficiently gather data from ECS and prepare it for the Renderer without locking the World for too long.*

### Task 2.1: Implement `RenderFrame` DTO (Data Transfer Object)
- **File**: `engine/src/runtime/render_frame.rs` (New File)
- **Action**: Define the "Frame Packet" that strictly contains what needs to be drawn.
- **Structure**:
  ```rust
  pub struct RenderFrame {
      pub camera: CameraUniform,
      // Key: (MeshAssetID, MaterialID) -> Value: List of Instances
      pub batches: HashMap<(String, String), Vec<MeshInstance>>,
      pub lights: Vec<LightUniform>,
  }
  ```

### Task 2.2: Create Extraction System
- **File**: `engine/src/runtime/extraction_system.rs` (New File)
- **Action**: Implement a system that queries ECS and populates `RenderFrame`.
- **Logic**:
  1. Iterate `Query<(&Transform, &Mesh, &Visibility)>`.
  2. Perform **Frustum Culling** (CPU side) against the Camera.
  3. If visible, convert `Transform` + `Mesh.color` -> `MeshInstance`.
  4. Push into `RenderFrame.batches`.

### Task 2.3: Rewrite `render_game_world`
- **File**: `engine/src/runtime/render_system.rs`
- **Action**: Replace the current monolithic loop.
- **New Flow**:
  1. `let frame = extraction_system::extract(world);`
  2. `renderer.upload(frame);` (Upload instance buffers to GPU).
  3. `renderer.draw(frame);` (Record RenderPass commands).
- **Cleanup**: Remove `RenderCache` logic that holds individual Object BindGroups.

---

## 🧠 Module: `ecs` (Data Management)
*Goal: Support efficient querying and dirty checking.*

### Task 3.1: Visibility Component
- **File**: `ecs/src/components/rendering.rs`
- **Action**: Add `pub struct Visible { is_visible: bool, last_frame_visible: bool }`.
- **Usage**: Used by Extraction System to skip entities early.

### Task 3.2: Static vs Dynamic Optimization (Optional)
- **File**: `ecs/src/components/transform.rs`
- **Action**: Add `pub enum Mobility { Static, Dynamic }`.
- **Goal**: Static objects can have their `InstanceBuffer` created once and cached. Dynamic objects upload every frame.

---

## 📱 Module: `render` -> `mobile_optimization`
*Goal: Configure WGPU for Tile-Based Deferred Rendering (TBDR) behaviors.*

### Task 4.1: Depth Buffer Opt-in
- **File**: `render/src/lib.rs` (RenderModule setup)
- **Action**: Change `store: StoreOp::Store` to `StoreOp::Discard` for the Depth Stencil attachment.
- **Reason**: Mobile GPUs shouldn't write depth back to VRAM unless post-processing needs it.

### Task 4.2: Texture Format Review
- **File**: `render/src/texture_manager.rs`
- **Action**: Add support/check for `TextureFormat::Astc...` if available.
- **Action**: Fallback to `Bc3` (Desktop) or `Etc2` (Android old) gracefully.

### Task 4.3: Shader Precision
- **File**: `*.wgsl`
- **Action**: Review all `f32` usage.
- **Change**: Use `vec3<f16>` for colors/normals if the `shader-f16` extension is enabled in WGPU features.

---

## 🧪 Verification Plan

1.  **Stress Test**: Spawn 5,000 Cubes (Static).
    - *Pass Criteria*: 60 FPS on PC, >30 FPS on Mobile. (Old system would crawl at <10 FPS).
2.  **Memory Audit**: Check VRAM usage.
    - *Expected*: Significant drop in Binding overhead.
3.  **RenderDoc Capture**:
    - Verify that 5,000 Cubes result in ~1-5 Draw Calls (depending on material count), not 5,000 calls.
