# 2026 Mobile Render Pipeline Architecture (Flux)

## 1. Overview
เป้าหมายคือการสร้าง Render Pipeline ที่มีความยืดหยุ่นสูง (Flexible), ประสิทธิภาพสูงบน Mobile (Tile-Based Rendering Friendly), และรองรับ Custom Material ที่ผู้ใช้สามารถเขียน Shader เองได้ (User Generated Content).

คอนเซปต์หลักประกอบด้วย 3 ส่วน:
1.  **Render Graph (Flux Graph)**: จัดการลำดับการ Render และ Resource (Transient Attachments)
2.  **Material System 2.0**: ระบบ Material ที่ Data-Driven แยก Definition ออกจาก Implementation
3.  **Hybrid Data Flow (ECS -> GPU)**: การส่งข้อมูลจาก World ไปยัง GPU อย่างมีประสิทธิภาพ

---

## 2. Render Graph (Flux)
Mobile GPU (TBR/TBDR) ต้องการให้เราจัดการ `LoadOp` และ `StoreOp` อย่างระมัดระวังเพื่อลด Bandwidth. Render Graph จะช่วยจัดการเรื่องนี้อัตโนมัติ.

### 2.1 Node & Edge
*   **RenderNode**: แทน 1 Pass (เช่น `ShadowPass`, `GBufferPass`, `LightingPass`, `TransparentPass`, `PostProcessPass`).
*   **Resource**: Texture หรือ Buffer ที่ถูกสร้างและส่งต่อระหว่าง Node.
*   **Transient Resource**: Resource ที่ใช้แค่ใน Frame นั้นๆ (เช่น Depth Buffer, G-Buffer) สามารถระบุเป็น `Memoryless` ได้บน Metal/Vulkan.

### 2.2 Global & Pass Data binding
เพื่อให้ Custom Material เข้าถึงข้อมูลส่วนกลางได้ เราจะ Standardization Bind Group Layout ดังนี้:

| Group | Frequency | Description | Content |
| :--- | :--- | :--- | :--- |
| **0** | **Global** | เปลี่ยน 1 ครั้งต่อ Frame | Camera (View/Proj), Time, Ambient Light, Global Fog |
| **1** | **Pass** | เปลี่ยน 1 ครั้งต่อ Pass | Pass-specific data (เช่น Shadow Matrix, Lights ใน Forward Pass, G-Buffer Input ใน Lighting Pass) |
| **2** | **Material** | เปลี่ยนต่อ Material | Texture (Albedo, Normal), Material Params (Color, Smoothness) |
| **3** | **Object** | เปลี่ยนต่อ Object (Instance) | Model Matrix, Instance Color, Skinning Data |

การออกแบบนี้ทำให้ Material (Group 2) สามารถเขียน WGSL โดย declare `var<uniform> global: GlobalData;` (Group 0) หรือ `var<uniform> pass: PassData;` (Group 1) ได้ทันที

---

## 3. Custom Material System
ผู้ใช้ต้องการสร้าง "Cel Shader" หรือ Custom Shader อื่นๆ.

### 3.1 Shader Asset (`.wgsl` + Meta)
User จะเขียนไฟล์ `.wgsl` โดย Engine จะทำการ **Inject** common structs ให้
**Example User Shader (`cel_shader.wgsl`):**
```wgsl
#import engine::global // Get Camera, Time
#import engine::pass   // Get Lights

struct CustomMaterial {
    color: vec4<f32>,
    threshold: f32,
}
@group(2) @binding(0) var<uniform> material: CustomMaterial;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let L = normalize(pass.main_light_dir);
    let diff = max(dot(N, L), 0.0);
    
    // Custom Logic: Cel Shading
    let cel = step(material.threshold, diff); 
    return material.color * cel;
}
```

### 3.2 Material Asset
Data file (`.mat` หรือ `.json`) ที่ระบุ:
1.  **Shader**: path/to/cel_shader.wgsl
2.  **Properties**:
    *   `color`: [1.0, 0.5, 0.0, 1.0]
    *   `threshold`: 0.5

### 3.3 Pipeline Generator
Engine จะ parse `.wgsl` และสร้าง `wgpu::RenderPipeline` ให้รันไทม์ โดยดูจาก:
*   **Blend State**: (Inherit from Material config)
*   **Depth State**: (Inherit from Material config)
*   **Input Layout**: Standard `VertexInput`

---

## 4. Implementation Steps (Roadmap)

### Phase 1: Standardization (Refactor)
1.  **Refactor `MeshRenderer`**: แยก Logic การสร้าง Pipeline ออกจากตัว Renderer ย้ายไป `MaterialSystem`.
2.  **Define `GlobalLayout` & `PassLayout`**: สร้าง BindGroup Layout มาตรฐานที่ทุก Shader ต้องใช้.
3.  **Update Shaders**: แก้ `pbr.wgsl`, `toon.wgsl` ให้ใช้ Layout ใหม่ (Group 0, 1, 2, 3).

### Phase 2: Material Asset System
1.  **Asset Loader**: สร้าง Loader สำหรับ `.wgsl` และ `.mat`.
2.  **Material Instance**: สร้าง Struct `MaterialInstance` ที่เก็บ `BindGroup` (Group 2) ของตัวเอง.

### Phase 3: Render Graph
1.  **Basic Graph**: สร้าง Graph อย่างง่าย (Hardcoded ก่อน) ที่มี `MainPass`.
2.  **Transient Attachments**: จัดการ Texture creation/destruction ตาม Graph.

---

## 5. ECS Integration
ใน `render_app`:
1.  **Extract System**: Query `(Entity, &Mesh, &MaterialHandle, &Transform)` -> ส่งเข้า `RenderQueue`.
2.  **Prepare System**: `RenderQueue` ถูก sort และจัดกลุ่ม (Batching).
3.  **Render System**: รันผ่าน Graph -> สั่ง Draw Call.

แนวทางนี้จะทำให้ Render Logic (Render Graph) แยกขาดจาก Game Logic (ECS) แต่เชื่อมกันด้วย Data.
