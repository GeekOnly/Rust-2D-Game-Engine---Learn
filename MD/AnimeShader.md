# 🏆 Engine Implementation Plan: Anime Shader (AAA Quality)

นี่คือแผนงาน (Technical Plan) สำหรับการนำเทคนิค Anime Shader ระดับ AAA มาลงใน Engine ปัจจุบันที่ใช้ **Rust + WGPU** โดยยึดหลัก **Mobile First** (High Quality, Low Cost)

---

## 🏗️ Phase 1: Shader & Pipeline Extensions
เป้าหมาย: อัปเกรด `toon.wgsl` และ `MeshRenderer` ให้รองรับ Texture Masking และ Variable Outline

### 1.1 Update `ToonMaterial` (Rust Side)
แก้ไขไฟล์: `render/src/material.rs` และ `render/src/mesh_renderer.rs`
เพิ่ม Texture Support เข้าไปใน `ToonMaterial` จากเดิมที่มีแค่ Color

```rust
// render/src/mesh_renderer.rs struct MeshRenderer
// เพิ่ม BindGroup Entries ใน toon_material_layout:
// Binding 0: Uniform (Color, Params)
// Binding 1: SDF Texture (Face Shadow) - Optional
// Binding 2: SDF Sampler
// Binding 3: Mask Texture (Cheek/Nose/Detail) - Optional
// Binding 4: Mask Sampler
```

### 1.2 Variable Line Width (Vertex Shader)
แก้ไขไฟล์: `render/src/toon.wgsl` (Function `vs_outline`)
เราจะไม่ใช้ `params.x` คงที่ แต่จะคำนวณตามความลึก (Depth)

```wgsl
// Algorithm สำหรับ vs_outline
let distance = length(camera.view_pos.xyz - world_pos.xyz);
let depth_scale = clamp(distance / 10.0, 0.5, 2.0); // ปรับค่าตาม Scene scale
let dynamic_width = material.params.x * depth_scale * input.vertex_color.a; // ใช้ Vertex Color A เป็นตัวคุมน้ำหนักเส้นต่อ Vertex ได้ถ้าต้องการ
let extruded_pos = model.position + model.normal * dynamic_width;
```

### 1.3 Face Lighting (Fragment Shader)
แก้ไขไฟล์: `render/src/toon.wgsl` (Function `fs_main`)
เปลี่ยน Logic การคำนวณแสงจาก Simple Lambert เป็น SDF-based เมื่อเป็นส่วนหัว

```wgsl
// Pseudo Code
let light_dir = normalize(light.position.xyz - in.world_position);
let forward = normalize(in.normal); // ใช้ face forward vector
let right = cross(vec3(0.0, 1.0, 0.0), forward);

// คำนวณมุมแสงเทียบกับหน้า (Left/Right side)
let light_y_rot = dot(light_dir, right); 

// Sampling SDF Face Texture (Binding 1)
// SDF Map จะเก็บ threshold ของเงาในแต่ละองศาหันหน้า
let shadow_threshold = textureSample(sdf_texture, sdf_sampler, in.uv).r;

// เปรียบเทียบมุมแสงกับ Threshold ใน Texture
let is_shadow = step(shadow_threshold, light_y_rot);
// ผสมกับ Multi-Mask (Binding 3) สำหรับแก้ม/จมูก
```

---

## 🧩 Phase 2: Hair Shadow (System Architecture)
เป้าหมาย: เงาผมบังหน้า (Self-Shadowing specifics) โดยไม่กระทบ Performance รวม

### 2.1 Hair Shadow Pass Strategy
เพิ่ม Render Pass ใหม่ใน `render/src/tilemap_renderer.rs` (หรือแยก `ShadowRenderer`)
*   **Resolution:** 256x256 (Small texture is enough for hair strands)
*   **Format:** Depth16
*   **Culling:** Render เฉพาะ Entity ที่มี Component `ShadowCaster` และ Tag=`Hair`
*   **Target:** Face Mesh เท่านั้นที่จะ Sample map นี้

### 2.2 ECS Integration
ใน `engine/src/components.rs` เพิ่ม Tag Components เพื่อแยกแยะการ Render:
*   `AnimeFaceComponent`: บอก Renderer ว่าต้องใช้ SDF Shader logic
*   `AnimeHairComponent`: บอกว่าต้อง Cast เงาลงบนหน้า

---

## 🛠️ Implementation Steps

1.  **Material Upgrade**:
    *   [ ] แก้ `ToonMaterialUniform` ใน `render/src/mesh_renderer.rs` ให้รองรับ Parameter เพิ่มเติม (threshold, softness)
    *   [ ] แก้ `create_toon_material_bind_group` ให้รับ `scale_texture`, `offset_texture` หรือ `sdf_texture`

2.  **Shader Refactor (`toon.wgsl`)**:
    *   [ ] เขียน `vs_outline` ใหม่ให้รับ Depth scaling
    *   [ ] เขียน `fs_main` แยก flow ระหว่าง `Standard Toon` (Body/Prop) และ `Face Toon` (SDF) โดยใช้ Uniform switch (if/else)

3.  **Editor Integration**:
    *   [ ] เพิ่มช่องใส่ Texture (SDF Mask) ใน Inspector ของ Editor
    *   [ ] เพิ่ม Slider ปรับ Outline Width Curve

---

## 🚫 สิ่งที่ต้องระวัง (Engine Constraints)
*   **Geometry Shader**: Engine เราใช้ WGPU (WebGPU standard) **ไม่มี** Geometry Shader -> ใช้เทคนิค Inverted Hull (Render 2 pass) แบบปัจจุบันถูกต้องแล้ว
*   **Post-Process**: พยายามหลีกเลี่ยง Edge Detection แบบ Post-process บน Mobile เพราะ Bandwidth heavy -> Inverted Hull ดีกว่าในแง่ Performance สำหรับ Low-poly/Mid-poly anime characters

## 📝 Summary
แผนนี้เน้นแก้ที่ **Shader Code (`toon.wgsl`)** และ **BindGroup Layout (`mesh_renderer.rs`)** เป็นหลัก โดยไม่ต้องรื้อระบบ Render System ใหญ่ ทำให้ปลอดภัยและเริ่มทำได้ทันที