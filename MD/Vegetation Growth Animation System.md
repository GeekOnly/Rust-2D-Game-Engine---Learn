🌱 Vegetation Growth Animation System

Technical Specification v1.1 (Consolidated)

1. Objective

ระบบ Growth Animation สำหรับพืชแบบ procedural โดยใช้ **Rust 2D/3D Game Engine (WGPU)**:

ใช้ Vertex Animation (VAT) ผ่าน WGPU Compute/Vertex Shader

ไม่ใช้ skeleton / physics runtime (ใช้ Vertex Displacement)

deterministic, network-safe (SYNC ผ่าน Seed & Time)

ประสิทธิภาพสูง (Instanced Rendering ใน `render::VegetationRenderer`)

รองรับ instance จำนวนมาก

2. Supported Vegetation Types
Type	Core Behavior
Tree	Trunk → Branch → Leaf
Flower	Stem → Bud → Bloom
Vine	Segment-based climbing
3. Growth State Machine
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GrowthState {
  Seed,
  Growing,
  Blooming,
  Mature,
  Dormant,
  Dead,
}


State เปลี่ยนได้ด้วยเวลา / event

Growth speed ปรับต่อ instance

รองรับ pause / resume / accelerate

4. Growth Phases (Authoring Time)
Phase	Description
Germination	จุดเริ่ม
Structural	โครงสร้างหลัก
Detail	ใบ / กลีบ
Mature	รูปร่างสมบูรณ์

ทุก phase ถูก bake เป็น VAT (multi-stage)

5. Per-Vertex Attributes (Required)
Attribute	Range	Usage
growth_weight	0–1	ลำดับการโต
wind_weight	0–1	ความอ่อน
rigidity	0–1	ต้านแรงลม
segment_index	int	vine
height_mask	0–1	tree
6. Vegetation Type Rules
6.1 Tree

Trunk โต bottom → top

Branch delay ตาม depth

Leaf spawn หลัง branch mature

Root optional (close-up)

6.2 Flower

Stem โต → Bud scale

Petal bloom ด้วย local rotation

Bloom curve แบบ non-linear

6.3 Vine

โตแบบเพิ่ม segment

ปลายเถาอ่อนสุด

Direction bias: surface normal + gravity

7. Growth Evaluation (Bake-Time Only)
growth = smoothstep(t - mask * delay)
final_pos = base_pos * growth


Noise / random ใช้เฉพาะตอน bake

Seed-based deterministic

8. Vertex Animation (VAT)
8.1 Encoding

Delta position from base mesh

Fixed topology required

delta = animated_pos - base_pos

8.2 Storage Layout (Preferred)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VatVertex {
    dx: i16, dy: i16, dz: i16, // Snorm16
    normal_dx: i8, normal_dy: i8, normal_dz: i8, // Snorm8
    padding: u8,
}
// index = frame * vertex_count + vertex_id;
// Uploaded as a specialized storage buffer or Texture2D (Rg32Sint/Float)


Texture VAT เป็น optional fallback

9. Multi-Stage VAT

แบ่ง VAT ตาม phase:

Stage 0: structural

Stage 1: branch / stem

Stage 2: leaf / bloom

Runtime:

final = blend(stage0, stage1, stage2)

10. Runtime Playback
10.1 Shader Logic (WGPU WGSL)

```wgsl
// vegetation.wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(10) growth_params: vec4<f32>, // Instanced: [growth_t, wind_speed, seed, phase]
};

// Fetch VAT Frame
let frame_idx = loop_animation(time, total_frames);
let offset = textureLoad(vat_texture, vec2<i32>(vertex_index, frame_idx), 0).xyz;

var final_pos = base_pos + offset * smoothstep(0.0, 1.0, growth_t);
```

10.2 Instance Parameters (Rust Struct)
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VegetationInstance {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub growth_params: [f32; 4], // time, speed, var_seed, phase
}
```

11. Wind & Secondary Motion
final_pos += wind_offset * wind_weight * (1.0 - rigidity);


Wind แยกจาก growth (Global Wind Uniform)

ปิดได้ตาม state (Optimization)

12. LOD & Optimization
LOD	Behavior
LOD0	Full VAT
LOD1	Reduced frames
LOD2	Static mesh

Growth complete → bake static

Frame decimation

Quantization profile per platform

13. ECS Integration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vegetation {
    pub state: GrowthState,
    pub current_time: f32,
    pub growth_speed: f32,
    pub asset_id: String, // Link to logic/VAT
    pub variation_seed: u32,
}

// Component Storage in `CustomWorld`
// pub vegetations: HashMap<CustomEntity, Vegetation>,

14. Tooling
14.1 Editor Integration (Rust)

Add "Import Vegetation" to Editor:
1.  Load GLTF with animation.
2.  Bake Position/Normal deltas to Texture (VAT).
3.  Save as `.xsg` (Custom Asset) or `.vat` + `.png`.

Usage:
`engine_core::assets::vegetation_baker::bake_from_gltf(path, settings)`

14.2 Validation

Topology check

Precision error threshold

Vertex / frame budget

15. Platform Profiles
Platform	Limits
Mobile	≤5k verts, ≤64 frames
PC	≤20k verts, ≤256 frames
Web	Buffer VAT preferred
16. Determinism & Networking

Growth driven by frame index

Seed-based variation

No runtime randomness

17. Deliverables

VAT baker tool

Runtime shader (WGSL / GLSL)

Sample assets (tree / flower / vine)

Engine integration example

🎯 Minimal Production Set
✔ Multi-stage VAT
✔ Per-vertex masks
✔ Growth state machine
✔ Wind blend
✔ LOD switching
✔ Static bake on mature