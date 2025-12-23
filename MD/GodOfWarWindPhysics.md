# 🌪️ Advanced Wind System (God of War 4 Style - Compute Shader)
## 📱 Target: High-End Mobile (AAA Tier)

> **Note:** เอกสารนี้สำหรับ **High-End Mobile Spec** (iPhone 13+, Snapdragon 8 Gen 1+) ที่รองรับ Compute Shader ได้ดี
> ฟีเจอร์นี้จะถูกเปิดใช้งานเมื่อผู้เล่นเลือก Graphics Quality: **High / Ultra**

---

## 🏗️ Architecture Overview (Mobile Optimized)
ระบบจำลองมวลอากาศ (Fluid Simulation) ผ่าน **Compute Shaders** โดยปรับจูนให้รันบนมือถือระดับสูงได้ 60 FPS

### 1. The Wind Volume (Mobile Tuned)
ใช้ 3D Texture ความละเอียดที่ **พอดีกับหน้าจอมือถือ** ไม่ละเอียดเกินจำเป็น
*   **Resolution:** `32 x 16 x 32` (Voxel Grid)
    *   *Why?* ความละเอียดนี้เพียงพอสำหรับความรู้สึกของลมบนจอมือถือ และประหยัด Bandwidth กว่า 64³ ถึง 8 เท่า
*   **Format:** `Rgba16Float` (Half-Float)
    *   *Why?* มือถือส่วนใหญ่รองรับ Half-Precision ได้เร็วกว่า Full-Precision 2 เท่า
*   **Usage:** `StorageBinding` (Read/Write)

### 2. Compute Frequency (Optimization)
ไม่จำเป็นต้อง Simulate ทุกเฟรม!
*   **Physics Loop:** 30 Hz (Simulate ทุกๆ 2 เฟรม)
*   **Rendering Loop:** 60 Hz (Interpolate ค่าเอาระหว่างเฟรม)
*   *Result:* ลดภาระ Compute Shader ลง 50% ทันที

---

## 🔧 Implementation Plan

### Phase 1: Resource Setup (`render/src/wind_system.rs`)
```rust
pub struct WindSystem {
    // Volume Simulation Textures
    volume_texture: wgpu::Texture, // 32x16x32 RGBA16F
    volume_view: wgpu::TextureView,
    ping_pong_texture: wgpu::Texture, // For double buffering
    
    // Bind Groups
    sim_bind_group: wgpu::BindGroup,
    
    // Pipelines
    shift_pipeline: wgpu::ComputePipeline,    // Scroll texture
    motor_pipeline: wgpu::ComputePipeline,    // Inject force
    advection_pipeline: wgpu::ComputePipeline,// Fluid flow
    diffusion_pipeline: wgpu::ComputePipeline,// Blur/Dissipate
}
```

### Phase 2: Compute Shaders (`render/src/shaders/wind_sim.wgsl`)
ใช้ Thread Group Size ที่เหมาะกับ Mobile GPU (e.g., 4x4x4 = 64 threads)

```wgsl
// Mobile Optimization: Use 16-bit float if possible or stick to f32 carefully
@group(0) @binding(0) var volume_read: texture_3d<f32>;
@group(0) @binding(1) var<storage, read_write> volume_write: texture_storage_3d<rgba16float, write>;

struct WindMotor {
    position: vec3<f32>,
    radius: f32, // Squared radius for faster mobile calc
    force: vec3<f32>,
    motor_type: u32, // 0=Directional, 1=Omni, 2=Vortex
};

// ... Advection Logic ...
```

### Phase 3: LOD Integration
ระบบต้อง Fallback ได้
*   **High-End Device:** เปิดใช้งาน `WindSystem` + Sample 3D Texture ใน Vertex Shader
*   **Mid-Low Device:** ปิด `WindSystem` (Graceful Degradation) -> กลับไปใช้ Sine Wave แบบ `FoliageForMobile.md` อัตโนมัติ

---

## ⚡ Mobile Performance Budget (High-End)
| Metric | Budget | Note |
| :--- | :--- | :--- |
| **Resolution** | 32x16x32 | ~16,384 Voxels (Very safe) |
| **Update Rate** | 30ms (Variable) | หรือ Fixed 30Hz |
| **Memory** | ~256 KB | 3D Texture ขนาดเล็กมาก (L2 Cache Friendly) |
| **Bandwidth** | Medium | Compute Shader อ่าน/เขียน Texture ตลอดเวลา |

---

## � Roadmap for Implementation
1.  **Setup 3D Texture:** สร้าง Texture Rgba16Float ใน Rust
2.  **Basic Advection:** เขียน Shader ให้ลม "ไหล" ไปตามทิศทาง Global
3.  **Motor Injection:** ส่ง Array ของ Motor ไปกวน Volume
4.  **Connect to Grass:** แก้ `foliage.wgsl` ให้สลับโหมดได้ระหว่าง *Simple Sine* กับ *Volume Sample*