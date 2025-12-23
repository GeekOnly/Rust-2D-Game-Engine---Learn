# 🌿 Engine Implementation Plan: Foliage System (Mobile Optimized)

นี่คือแผนงาน (Technical Plan) สำหรับการสร้างระบบ Foliage บน Engine **Rust + WGPU** โดยเน้นประสิทธิภาพสูงสุดสำหรับ Mobile (Instanced Rendering & Vertex Animation)

---

## 🏗️ Phase 1: Core Rendering (Instancing)
เป้าหมาย: สร้าง Renderer ที่วาด Mesh เดียวหลายพันชิ้นได้ (Draw Call = 1)

### 1.1 Instance Data Structure
สร้าง struct ใหม่ใน `render/src/mesh.rs` หรือ `render/src/foliage_renderer.rs`
เราจะส่ง Transform Matrix + Custom Color ไปยัง GPU ต่อ Instance

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FoliageInstance {
    pub model_matrix: [[f32; 4]; 4], // 64 bytes
    pub color_tint: [f32; 4],       // 16 bytes (RGBA) - ใช้ทำ Fake AO / Variation
    pub custom_data: [f32; 4],      // 16 bytes (x=wind_phase, y=height_scale, z=unused, w=unused)
}

impl FoliageInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        // ... Define WGPU Vertex Buffer Layout with StepMode::Instance ...
    }
}
```

### 1.2 Foliage Shader (`foliage.wgsl`)
สร้างไฟล์ shader ใหม่ `render/src/foliage.wgsl`
เน้น **Vertex Animation** เพื่อลดภาระ CPU

```wgsl
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color_tint: vec4<f32>,
    @location(10) custom_data: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    // 1. Unpack transforms
    let instance_model = mat4x4<f32>(...);
    
    // 2. Vertex Color Decoding
    let bend_strength = model.color.r; // R = Bend Amount
    let wind_weight = model.color.g;   // G = Wind Influence
    
    // 3. Simple Wind Math (Sine wave approximation)
    let time = global_uniform.time;
    let wind_offset = sin(time + instance.custom_data.x + world_pos.x * 0.5) * wind_weight;
    
    // 4. Apply Offset
    let new_pos = model.position + vec3(wind_offset, 0.0, wind_offset * 0.5) * bend_strength;
    
    // ... Output ...
}
```

---

## 🌪️ Phase 2: Wind System (Global)
เป้าหมาย: ควบคุมลมทั้งฉากด้วย Uniform ตัวเดียว (God of War style)

### 2.1 Global Uniforms
แก้ไข `render/src/lib.rs` หรือสร้าง `render/src/environment.rs`

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalEnvironmentUniform {
    pub time: f32,
    pub wind_dir: [f32; 2],
    pub wind_strength: f32,
    pub padding: [f32; 4],
}
```

### 2.2 System Integration
*   ใน `App::update` ต้องส่งค่า `GlobalEnvironmentUniform` ไปยัง GPU ทุก/เฟรม
*   ใช้ `queue.write_buffer` อัปเดต Buffer นี้

---

## 💾 Phase 3: ECS Integration
เป้าหมาย: จัดการข้อมูล Instance จาก Entity Component System

### 3.1 Components
ใน `engine/src/components.rs`:
*   `FoliageComponent`: Tag ว่า Entity นี้เป็นต้นไม้/หญ้า
*   `InstanceBatchComponent`: (Optional) เก็บ `Vec<FoliageInstance>` สำหรับ Chunk นั้นๆ

### 3.2 Foliage System
สร้าง System ที่คอยรวบรวม (Cull & Collect) ข้อมูล Foliage ส่งให้ `FoliageRenderer`
*   **Chunking Strategy:** แบ่งเป็น Grid (e.g., 32x32m)
*   **Active Chunks:** Update เฉพาะ Chunk รอบตัว Player
*   **Draw Call Optimization:** 1 Draw Call ต่อ Mesh Asset (หญ้าแบบ A = 1 draw call รวมทุก chunk)

---

## 📱 Mobile Optimization Check
1.  **Alpha Testing:** ใช้ `discard` ใน Fragment Shader เมื่อ `alpha < 0.5` (ห้ามใช้ Alpha Blending ถ้าไม่จำเป็น เพราะแพงและมีปัญหากับ Depth)
2.  **No Lighting Calculation:** ใช้ `Vertex AO` + `Tint` แทน Real-time lighting
3.  **LOD:** (Phase ถัดไป) เปลี่ยน Mesh ตามระยะทาง

---

## 📝 Implementation Roadmap

1.  [ ] **Create `FoliageRenderer`**:
    *   Copy structure จาก `MeshRenderer`
    *   เพิ่ม `instance_buffer` support
2.  [ ] **Implement `foliage.wgsl`**:
    *   Basic Texture Mapping
    *   Alpha Clip
    *   Basic Wind Animation
3.  [ ] **Rust Integration**:
    *   Test Render หญ้า 10,000 ต้น ด้วย Random Position
    *   FPS Check (>60FPS on Mobile Target)

## 📌 Note
*   **Outline?**: ❌ **ห้ามใส่ outline** ให้ foliage เด็ดขาด (Scattered noise & Costly)
*   **Shadows?**: ถ้าจำเป็น ใช้ Simple Shadow Blob หรือ Bake ลง Texture terrain. ไม่ต้อง cast real-time shadow จากหญ้าทุกต้น