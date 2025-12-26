# 🚀 แผนยกระดับ Rust 2D Game Engine สู่ 100/100: ระบบ Render บนมือถือระดับโลก

**วันที่**: 26 ธันวาคม 2025
**เป้าหมาย**: พัฒนาระบบ Render บนมือถือให้เทียบเท่า Unreal Engine 5
**คะแนนปัจจุบัน**: 78/100
**คะแนนเป้าหมาย**: 100/100
**ช่วงเวลาพัฒนา**: 6-9 เดือน (แบ่งเป็น 7 เฟส)

---

## 📊 สรุปภาพรวม

### คะแนนปัจจุบัน vs เป้าหมาย

| หมวดหมู่ | ปัจจุบัน | เป้าหมาย | ต้องพัฒนา |
|----------|----------|----------|-----------|
| 1. สถาปัตยกรรม Rendering | 8.5/10 | 10/10 | +1.5 |
| 2. HDR & Post-Processing | 6/10 | 10/10 | +4.0 ⚠️ |
| 3. ระบบแสง (Lighting) | 9/10 | 10/10 | +1.0 |
| 4. ระบบเงา (Shadow) | 8/10 | 10/10 | +2.0 |
| 5. Material & Shader | 7.5/10 | 10/10 | +2.5 |
| 6. Texture & Resource | 7/10 | 10/10 | +3.0 |
| 7. Performance & Optimization | 8/10 | 10/10 | +2.0 |
| 8. 2D Rendering | 9/10 | 10/10 | +1.0 |
| 9. Developer Experience | 6/10 | 9/10 | +3.0 |
| 10. Platform Support | 8/10 | 10/10 | +2.0 |
| **รวม** | **78/100** | **100/100** | **+22** |

### จุดแข็งที่มีอยู่ ⭐

1. **Clustered Forward+ Lighting** - ระบบแสงแบบ Cluster ทันสมัย (1024 ไลท์, 64 ไลท์/คลัสเตอร์)
2. **Reverse-Z Depth** - ความแม่นยำ Depth ที่ดีกว่า
3. **Contact Shadows Infrastructure** - โครงสร้างสำหรับเงาละเอียด
4. **Batch Rendering** - รองรับ 10,000 instances
5. **Efficient Tilemap System** - ระบบ Tilemap ที่เหมาะสมกับ 2D
6. **WGPU Cross-Platform** - รองรับหลายแพลตฟอร์มผ่าน Vulkan/Metal/DX12
7. **Rust Safety & Performance** - ความปลอดภัยและประสิทธิภาพจาก Rust

### จุดที่ต้องพัฒนาเร่งด่วน ⚠️

1. **HDR Post-Processing** - ยังไม่มีเลย (ส่งผลกระทบมากที่สุด)
2. **Texture Compression** - ไม่มี ASTC/ETC2 สำหรับมือถือ
3. **Visual Shader Editor** - ต้องเขียนโค้ดทั้งหมด
4. **LOD System** - ไม่มีระบบ Level of Detail
5. **Occlusion Culling** - ไม่มีการตัดวัตถุที่มองไม่เห็น
6. **Advanced Material Features** - ขาด SSS, Parallax Mapping
7. **Profiling Tools** - ขาดเครื่องมือวิเคราะห์ประสิทธิภาพ

---

## 🎯 แผนพัฒนา 7 เฟส

### 📅 ไทม์ไลน์โดยรวม

```
เฟส 1: HDR Foundation (สัปดาห์ 1-4)        ████████░░░░░░░░░░░░
เฟส 2: Post-Processing (สัปดาห์ 5-10)     ░░░░░░░░████████████
เฟส 3: Advanced Lighting (สัปดาห์ 11-16)  ░░░░░░░░░░░░░░░░████████
เฟส 4: Material System (สัปดาห์ 17-22)    ░░░░░░░░░░░░░░░░░░░░░░████████
เฟส 5: Optimization (สัปดาห์ 23-28)       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████
เฟส 6: Tooling & DX (สัปดาห์ 29-34)       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████
เฟส 7: Polish & Test (สัปดาห์ 35-38)      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████
```

---

## 🏗️ เฟส 1: HDR Foundation (4 สัปดาห์)

**เป้าหมาย**: สร้างโครงสร้างพื้นฐาน HDR และ Render Target
**คะแนนที่เพิ่ม**: 6/10 → 7.5/10 (HDR & Post-Processing)

### สัปดาห์ 1-2: HDR Render Target Infrastructure

#### Task 1.1: HDR Texture Setup
```rust
// ไฟล์: render/src/lib.rs

pub struct RenderModule {
    // เพิ่ม HDR render target
    hdr_texture: wgpu::Texture,
    hdr_view: wgpu::TextureView,
    hdr_format: wgpu::TextureFormat, // Rgba16Float

    // Depth textures ที่มีอยู่แล้ว
    depth_texture: wgpu::Texture,
    scene_depth_texture: wgpu::Texture,
}

impl RenderModule {
    fn create_hdr_target(&mut self, width: u32, height: u32) {
        self.hdr_format = wgpu::TextureFormat::Rgba16Float;

        self.hdr_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("HDR Render Target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.hdr_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                 | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.hdr_view = self.hdr_texture.create_view(&Default::default());
    }
}
```

**ผลลัพธ์**:
- สามารถ Render ไปยัง HDR texture (Rgba16Float)
- รองรับค่าสีที่สว่างเกิน 1.0 (HDR)
- พร้อมสำหรับ Post-Processing

#### Task 1.2: Pipeline Modification
```rust
// อัปเดต render_scene() ให้ render ไปยัง HDR target

pub fn render_scene(&mut self, encoder: &mut CommandEncoder) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &self.hdr_view, // ← เปลี่ยนจาก surface_view
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(/* ... */),
    });

    // Render meshes, sprites, etc.
}
```

### สัปดาห์ 3-4: Basic Tonemapping

#### Task 1.3: Simple Post-Process Renderer
```rust
// ไฟล์ใหม่: render/src/post_process_renderer.rs

pub struct PostProcessRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
}

impl PostProcessRenderer {
    pub fn render(&self,
                  encoder: &mut CommandEncoder,
                  hdr_view: &TextureView,
                  output_view: &TextureView) {

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view, // Swapchain
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1); // Fullscreen triangle
    }
}
```

#### Task 1.4: Basic ACES Tonemapping Shader
```wgsl
// ไฟล์ใหม่: render/assets/shaders/post_process_basic.wgsl

@group(0) @binding(0) var t_hdr: texture_2d<f32>;
@group(0) @binding(1) var s_linear: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle (no vertex buffer)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

// ACES Tonemapping (Narkowicz fit)
fn aces_tone_map(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

// Linear to sRGB
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / 2.2));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Sample HDR color
    var color = textureSample(t_hdr, s_linear, in.uv).rgb;

    // 2. ACES Tonemapping
    color = aces_tone_map(color);

    // 3. Gamma correction
    color = linear_to_srgb(color);

    return vec4<f32>(color, 1.0);
}
```

### Checklist เฟส 1

- [ ] สร้าง HDR render target (Rgba16Float)
- [ ] แก้ไข render_scene() ให้ render ไปยัง HDR texture
- [ ] สร้าง PostProcessRenderer struct
- [ ] เขียน post_process_basic.wgsl
- [ ] Implement ACES tonemapping
- [ ] เชื่อมต่อ post-process renderer เข้ากับ render loop
- [ ] ทดสอบกับ scene ที่มีแสงสว่าง (bloom emissive)

**ผลลัพธ์**: เห็นความแตกต่างของ HDR ชัดเจน, ไม่มีสีไหม้ (blown-out highlights)

---

## 🌸 เฟส 2: Advanced Post-Processing (6 สัปดาห์)

**เป้าหมาย**: ระบบ Post-Processing แบบ AAA (Bloom, Color Grading, Effects)
**คะแนนที่เพิ่ม**: 7.5/10 → 10/10 (HDR & Post-Processing)

### สัปดาห์ 5-7: Bloom System

#### Task 2.1: Bloom Downsample Chain
```rust
// ไฟล์: render/src/bloom_renderer.rs

pub struct BloomRenderer {
    // Downsample chain (1/2 -> 1/4 -> 1/8 -> 1/16)
    mip_textures: Vec<wgpu::Texture>,
    mip_views: Vec<wgpu::TextureView>,

    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,

    threshold: f32, // Default: 1.0 (only bright pixels)
    intensity: f32, // Default: 0.04
}

impl BloomRenderer {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let mut mip_textures = Vec::new();
        let mip_count = 4; // 1/2, 1/4, 1/8, 1/16

        for i in 0..mip_count {
            let scale = 2u32.pow(i + 1);
            let mip_width = (width / scale).max(1);
            let mip_height = (height / scale).max(1);

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom Mip {}", i)),
                size: wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 1
                },
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                     | wgpu::TextureUsages::TEXTURE_BINDING,
                // ...
            });

            mip_textures.push(texture);
        }

        // Create pipelines, bind groups...
        Self { /* ... */ }
    }

    pub fn render(&self,
                  encoder: &mut CommandEncoder,
                  hdr_view: &TextureView) -> &TextureView {

        // 1. Downsample chain with threshold
        self.downsample_pass(encoder, hdr_view);

        // 2. Upsample and blend
        self.upsample_pass(encoder);

        // Return final bloom texture
        &self.mip_views[0]
    }
}
```

#### Task 2.2: Downsample Shader
```wgsl
// ไฟล์: render/assets/shaders/bloom_downsample.wgsl

@group(0) @binding(0) var t_input: texture_2d<f32>;
@group(0) @binding(1) var s_linear: sampler;

struct BloomUniforms {
    threshold: f32,
    knee: f32,
    _padding: vec2<f32>,
}
@group(0) @binding(2) var<uniform> u_bloom: BloomUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 13-tap downsample (tent filter)
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_input));

    var color = vec3<f32>(0.0);

    // Center
    color += textureSample(t_input, s_linear, in.uv).rgb * 0.25;

    // 4 corners
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0, -1.0) * texel_size).rgb * 0.125;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0, -1.0) * texel_size).rgb * 0.125;
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0,  1.0) * texel_size).rgb * 0.125;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0,  1.0) * texel_size).rgb * 0.125;

    // 4 edges
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0,  0.0) * texel_size).rgb * 0.0625;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0,  0.0) * texel_size).rgb * 0.0625;
    color += textureSample(t_input, s_linear, in.uv + vec2( 0.0, -1.0) * texel_size).rgb * 0.0625;
    color += textureSample(t_input, s_linear, in.uv + vec2( 0.0,  1.0) * texel_size).rgb * 0.0625;

    // Luminance-based threshold (soft knee)
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    let soft = luminance - u_bloom.threshold + u_bloom.knee;
    soft = clamp(soft, 0.0, 2.0 * u_bloom.knee);
    soft = soft * soft / (4.0 * u_bloom.knee + 0.00001);

    let contribution = max(soft, luminance - u_bloom.threshold);
    contribution /= max(luminance, 0.00001);

    color *= contribution;

    return vec4<f32>(color, 1.0);
}
```

#### Task 2.3: Upsample Shader (Tent Filter)
```wgsl
// ไฟล์: render/assets/shaders/bloom_upsample.wgsl

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(t_input));

    // 9-tap tent filter
    var color = vec3<f32>(0.0);

    // Center
    color += textureSample(t_input, s_linear, in.uv).rgb * 4.0;

    // Cross pattern
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0,  0.0) * texel_size).rgb * 2.0;
    color += textureSample(t_input, s_linear, in.uv + vec2( 0.0, -1.0) * texel_size).rgb * 2.0;
    color += textureSample(t_input, s_linear, in.uv + vec2( 0.0,  1.0) * texel_size).rgb * 2.0;

    // Diagonal corners
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0, -1.0) * texel_size).rgb;
    color += textureSample(t_input, s_linear, in.uv + vec2(-1.0,  1.0) * texel_size).rgb;
    color += textureSample(t_input, s_linear, in.uv + vec2( 1.0,  1.0) * texel_size).rgb;

    color /= 16.0;

    return vec4<f32>(color, 1.0);
}
```

### สัปดาห์ 8-10: Uber Post-Processing Shader

#### Task 2.4: Complete Uber-Shader
```wgsl
// ไฟล์: render/assets/shaders/post_process_uber.wgsl

@group(0) @binding(0) var t_hdr: texture_2d<f32>;
@group(0) @binding(1) var t_bloom: texture_2d<f32>;
@group(0) @binding(2) var t_lut: texture_3d<f32>; // Optional color LUT
@group(0) @binding(3) var s_linear: sampler;

struct PostProcessUniforms {
    exposure: f32,
    bloom_intensity: f32,
    contrast: f32,
    saturation: f32,

    vignette_strength: f32,
    vignette_smoothness: f32,
    grain_intensity: f32,
    chromatic_aberration: f32,

    temperature: f32,   // Color temperature (-1 to 1)
    tint: f32,          // Color tint (-1 to 1)
    _padding: vec2<f32>,
}
@group(0) @binding(4) var<uniform> u_post: PostProcessUniforms;

// ACES Tonemapping
fn aces_tone_map(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

// Contrast adjustment
fn adjust_contrast(color: vec3<f32>, contrast: f32) -> vec3<f32> {
    return (color - 0.5) * contrast + 0.5;
}

// Saturation adjustment
fn adjust_saturation(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luminance), color, saturation);
}

// Film grain
fn film_grain(uv: vec2<f32>, time: f32) -> f32 {
    let noise = fract(sin(dot(uv + time, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    return noise * 2.0 - 1.0;
}

// Chromatic aberration
fn chromatic_aberration(uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let offset_dir = (uv - 0.5) * strength * 0.01;

    let r = textureSample(t_hdr, s_linear, uv + offset_dir).r;
    let g = textureSample(t_hdr, s_linear, uv).g;
    let b = textureSample(t_hdr, s_linear, uv - offset_dir).b;

    return vec3<f32>(r, g, b);
}

// Color temperature
fn apply_temperature(color: vec3<f32>, temp: f32) -> vec3<f32> {
    // Warm = more red/yellow, Cool = more blue
    let warm = vec3<f32>(1.0, 0.9, 0.7);
    let cool = vec3<f32>(0.7, 0.9, 1.0);

    let color_shift = mix(cool, warm, temp * 0.5 + 0.5);
    return color * color_shift;
}

// 3D LUT lookup
fn apply_lut(color: vec3<f32>) -> vec3<f32> {
    // Assume 16x16x16 LUT
    let lut_size = 16.0;
    let scale = (lut_size - 1.0) / lut_size;
    let offset = 0.5 / lut_size;

    let uv3d = color * scale + offset;
    return textureSample(t_lut, s_linear, uv3d).rgb;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // 1. Chromatic Aberration (optional)
    var color: vec3<f32>;
    if (u_post.chromatic_aberration > 0.0) {
        color = chromatic_aberration(uv, u_post.chromatic_aberration);
    } else {
        color = textureSample(t_hdr, s_linear, uv).rgb;
    }

    // 2. Exposure
    color *= u_post.exposure;

    // 3. Bloom
    let bloom = textureSample(t_bloom, s_linear, uv).rgb;
    color += bloom * u_post.bloom_intensity;

    // 4. Color temperature
    color = apply_temperature(color, u_post.temperature);

    // 5. Contrast & Saturation
    color = adjust_contrast(color, u_post.contrast);
    color = adjust_saturation(color, u_post.saturation);

    // 6. Color LUT (if available)
    // color = apply_lut(color);

    // 7. ACES Tonemapping
    color = aces_tone_map(color);

    // 8. Vignette
    let dist = distance(uv, vec2<f32>(0.5));
    let vignette = smoothstep(
        u_post.vignette_strength,
        u_post.vignette_strength - u_post.vignette_smoothness,
        dist
    );
    color *= vignette;

    // 9. Film Grain (time-based, requires uniform)
    // let grain = film_grain(uv, u_time) * u_post.grain_intensity;
    // color += vec3<f32>(grain);

    // 10. Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
```

### Checklist เฟส 2

- [ ] สร้าง BloomRenderer พร้อม downsample chain
- [ ] เขียน bloom_downsample.wgsl (13-tap tent filter)
- [ ] เขียน bloom_upsample.wgsl (9-tap tent filter)
- [ ] ทดสอบ bloom กับวัตถุที่เปล่งแสง (emissive)
- [ ] สร้าง PostProcessUniforms struct
- [ ] เขียน post_process_uber.wgsl แบบสมบูรณ์
- [ ] Implement exposure control
- [ ] Implement contrast & saturation
- [ ] Implement vignette
- [ ] Implement chromatic aberration
- [ ] Implement color temperature
- [ ] (Optional) Implement 3D LUT color grading
- [ ] สร้าง UI controls สำหรับ post-processing parameters

**ผลลัพธ์**: ภาพมีความสวยงามระดับ AAA, bloom ที่นุ่มนวล, ควบคุม look & feel ได้ตามต้องการ

---

## 💡 เฟส 3: Advanced Lighting & Shadows (6 สัปดาห์)

**เป้าหมาย**: ระบบแสงและเงาระดับ Production
**คะแนนที่เพิ่ม**:
- Lighting: 9/10 → 10/10
- Shadow: 8/10 → 10/10

### สัปดาห์ 11-13: Spot & Area Lights

#### Task 3.1: Extend Light Structure
```rust
// ไฟล์: render/src/lighting.rs

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLight {
    pub position: [f32; 4],      // w = light type (0=point, 1=spot, 2=area)
    pub color: [f32; 4],          // rgb + intensity
    pub direction: [f32; 4],      // For spot/area lights, w = spot angle
    pub params: [f32; 4],         // x=radius, y=inner_angle, z=outer_angle, w=area_size
}

pub enum LightType {
    Point { radius: f32 },
    Spot {
        radius: f32,
        direction: Vec3,
        inner_angle: f32,  // Full brightness cone
        outer_angle: f32,  // Falloff cone
    },
    Area {
        size: f32,
        direction: Vec3,
    },
}
```

#### Task 3.2: Update Cluster Culling for Spot Lights
```wgsl
// ไฟล์: render/assets/shaders/cluster_culling.wgsl

fn sphere_vs_cone_intersection(
    light_pos: vec3<f32>,
    light_dir: vec3<f32>,
    cone_angle: f32,
    radius: f32,
    cluster_min: vec3<f32>,
    cluster_max: vec3<f32>
) -> bool {
    // Test if AABB intersects with cone
    // Implementation based on separating axis theorem

    let cluster_center = (cluster_min + cluster_max) * 0.5;
    let cluster_extent = (cluster_max - cluster_min) * 0.5;

    let to_cluster = cluster_center - light_pos;
    let distance = length(to_cluster);

    if (distance > radius) {
        return false;
    }

    let cos_angle = cos(cone_angle);
    let dot_dir = dot(normalize(to_cluster), light_dir);

    // Cone test with cluster radius consideration
    let cluster_radius = length(cluster_extent);
    let sin_angle = sin(cone_angle);

    return dot_dir >= cos_angle - (cluster_radius / distance) * sin_angle;
}

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // ... existing code ...

    for (var i = 0u; i < light_count; i++) {
        let light = lights[i];
        let light_type = u32(light.position.w);

        var intersects = false;

        if (light_type == 0u) {
            // Point light (existing sphere test)
            intersects = sphere_vs_aabb(light.position.xyz, light.params.x,
                                       cluster_min, cluster_max);
        } else if (light_type == 1u) {
            // Spot light (cone test)
            intersects = sphere_vs_cone_intersection(
                light.position.xyz,
                light.direction.xyz,
                light.params.z, // outer_angle
                light.params.x, // radius
                cluster_min,
                cluster_max
            );
        }

        if (intersects) {
            // Add to cluster...
        }
    }
}
```

#### Task 3.3: Update PBR Shader for Spot/Area Lights
```wgsl
// ไฟล์: render/assets/shaders/pbr.wgsl

fn calculate_spot_attenuation(
    light_dir: vec3<f32>,
    to_light: vec3<f32>,
    inner_angle: f32,
    outer_angle: f32
) -> f32 {
    let cos_outer = cos(outer_angle);
    let cos_inner = cos(inner_angle);

    let cos_angle = dot(normalize(-to_light), light_dir);

    // Smooth falloff between inner and outer cone
    return smoothstep(cos_outer, cos_inner, cos_angle);
}

fn calculate_lighting(
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    light: Light
) -> vec3<f32> {

    let light_type = u32(light.position.w);
    var to_light = light.position.xyz - world_pos;
    let distance = length(to_light);
    to_light = normalize(to_light);

    // Distance attenuation
    var attenuation = calculate_attenuation(distance, light.params.x);

    // Spot light angular attenuation
    if (light_type == 1u) {
        attenuation *= calculate_spot_attenuation(
            light.direction.xyz,
            to_light,
            light.params.y, // inner_angle
            light.params.z  // outer_angle
        );
    }

    // PBR calculations (existing code)
    // ...

    return final_color * attenuation;
}
```

### สัปดาห์ 14-16: Advanced Shadow Techniques

#### Task 3.4: 4 Cascade Shadow Maps
```rust
// อัปเกรด ShadowTexture จาก 2 cascades เป็น 4

pub struct ShadowTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    // เพิ่มจาก 2 เป็น 4 cascades
    pub num_cascades: u32, // = 4
}

impl ShadowTexture {
    pub fn new(device: &Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 4, // 4 cascades
            },
            format: wgpu::TextureFormat::Depth32Float,
            // ...
        });

        // ...
    }
}
```

#### Task 3.5: Percentage Closer Soft Shadows (PCSS)
```wgsl
// ไฟล์: render/assets/shaders/shadow.wgsl

// PCF with variable kernel size (PCSS-style)
fn shadow_pcss(
    shadow_coord: vec3<f32>,
    cascade_index: u32,
    normal_bias: f32
) -> f32 {
    // 1. Blocker search (find average depth of blockers)
    let search_radius = 0.01;
    var blocker_depth = 0.0;
    var blocker_count = 0.0;

    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * search_radius;
            let sample_coord = shadow_coord.xy + offset;

            let depth = textureSample(
                t_shadow,
                s_shadow,
                sample_coord,
                cascade_index
            );

            if (depth < shadow_coord.z) {
                blocker_depth += depth;
                blocker_count += 1.0;
            }
        }
    }

    if (blocker_count < 1.0) {
        return 1.0; // No blockers, fully lit
    }

    blocker_depth /= blocker_count;

    // 2. Calculate penumbra size
    let penumbra = (shadow_coord.z - blocker_depth) / blocker_depth;
    let filter_radius = penumbra * 0.02;

    // 3. PCF with variable kernel
    var shadow = 0.0;
    let sample_count = 16.0;

    for (var i = 0; i < 16; i++) {
        let offset = poisson_disk[i] * filter_radius;
        let sample_coord = shadow_coord.xy + offset;

        shadow += textureSampleCompare(
            t_shadow,
            s_shadow_compare,
            sample_coord,
            cascade_index,
            shadow_coord.z
        );
    }

    return shadow / sample_count;
}

// Poisson disk samples for soft shadows
const poisson_disk: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-0.94201624, -0.39906216),
    vec2<f32>( 0.94558609, -0.76890725),
    vec2<f32>(-0.094184101,-0.92938870),
    vec2<f32>( 0.34495938,  0.29387760),
    // ... 12 more samples
);
```

#### Task 3.6: Contact Shadows Enhancement
```wgsl
// Screen Space Contact Shadows (SSCS) - ปรับปรุง

fn screen_space_contact_shadow(
    world_pos: vec3<f32>,
    light_dir: vec3<f32>,
    view_proj: mat4x4<f32>
) -> f32 {
    let ray_length = 0.5;  // meters
    let step_count = 16;

    var current_pos = world_pos;
    let step_size = ray_length / f32(step_count);

    for (var i = 0; i < step_count; i++) {
        current_pos += light_dir * step_size;

        // Project to screen space
        let clip_pos = view_proj * vec4<f32>(current_pos, 1.0);
        let ndc = clip_pos.xyz / clip_pos.w;
        let screen_uv = ndc.xy * 0.5 + 0.5;

        if (screen_uv.x < 0.0 || screen_uv.x > 1.0 ||
            screen_uv.y < 0.0 || screen_uv.y > 1.0) {
            break;
        }

        // Sample scene depth
        let scene_depth = textureSample(t_scene_depth, s_depth, screen_uv).r;

        // Reverse-Z comparison
        if (ndc.z < scene_depth - 0.001) {
            // Hit occluder
            let fade = 1.0 - f32(i) / f32(step_count);
            return fade * 0.5; // Partially shadowed
        }
    }

    return 1.0; // No contact shadow
}
```

### Checklist เฟส 3

- [ ] Extend Light structure สำหรับ Spot/Area lights
- [ ] อัปเดต cluster culling สำหรับ cone testing
- [ ] อัปเดต PBR shader สำหรับ spot light attenuation
- [ ] ทดสอบ spot lights ในหลายสถานการณ์
- [ ] เพิ่มจำนวน shadow cascades เป็น 4
- [ ] Implement PCSS (Percentage Closer Soft Shadows)
- [ ] สร้าง Poisson disk sampling pattern
- [ ] ปรับปรุง contact shadows ให้นุ่มนวลขึ้น
- [ ] เพิ่ม shadow quality settings (Low/Medium/High/Ultra)
- [ ] Optimize shadow performance สำหรับมือถือ

**ผลลัพธ์**: แสงและเงาคุณภาพระดับ Console, เงานุ่มและสมจริง

---

## 🎨 เฟส 4: Advanced Material System (6 สัปดาห์)

**เป้าหมาย**: ระบบ Material ที่ทรงพลังและยืดหยุ่น
**คะแนนที่เพิ่ม**: Materials & Shaders: 7.5/10 → 10/10

### สัปดาห์ 17-19: Subsurface Scattering (SSS)

#### Task 4.1: SSS Material Type
```rust
// ไฟล์: render/src/material.rs

#[derive(Clone)]
pub struct SssMaterial {
    pub albedo_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub thickness_texture: Option<Arc<Texture>>, // New

    pub albedo_factor: [f32; 4],
    pub subsurface_color: [f32; 3],  // Scattering color
    pub subsurface_power: f32,        // Scattering intensity
    pub thickness_scale: f32,         // Thickness multiplier
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SssMaterialUniform {
    pub albedo_factor: [f32; 4],
    pub subsurface_color: [f32; 4], // rgb + power
    pub thickness_scale: f32,
    pub _padding: [f32; 3],
}
```

#### Task 4.2: SSS Shader
```wgsl
// ไฟล์: render/assets/shaders/sss.wgsl

// Subsurface Scattering (Screen-Space Approximation)
fn calculate_sss(
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    thickness: f32,
    subsurface_color: vec3<f32>,
    subsurface_power: f32,
    light_dir: vec3<f32>,
    light_color: vec3<f32>
) -> vec3<f32> {

    // Translucency (light passing through thin surfaces)
    let translucency = pow(
        saturate(dot(-light_dir, normal) + 0.5),
        subsurface_power
    ) * thickness;

    // Wrap lighting (softer diffuse)
    let wrap = 0.5;
    let ndotl = dot(normal, light_dir);
    let wrap_diffuse = saturate((ndotl + wrap) / (1.0 + wrap));

    // Combine
    let sss = (translucency + wrap_diffuse * 0.5) * subsurface_color * light_color;

    return sss;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ... sample textures ...

    let thickness = textureSample(t_thickness, s_sampler, in.uv).r
                  * u_material.thickness_scale;

    var final_color = vec3<f32>(0.0);

    // Loop through lights
    for (var i = 0u; i < light_count; i++) {
        let light = lights[i];

        // Standard PBR
        let pbr = calculate_pbr(/* ... */);

        // SSS
        let sss = calculate_sss(
            in.world_pos,
            normal,
            view_dir,
            thickness,
            u_material.subsurface_color.rgb,
            u_material.subsurface_color.a,
            to_light,
            light.color.rgb
        );

        final_color += pbr + sss;
    }

    return vec4<f32>(final_color, 1.0);
}
```

### สัปดาห์ 20-22: Parallax Occlusion Mapping & Texture Features

#### Task 4.3: Parallax Occlusion Mapping
```wgsl
// ไฟล์: render/assets/shaders/parallax.wgsl

fn parallax_occlusion_mapping(
    uv: vec2<f32>,
    view_dir_tangent: vec3<f32>,
    height_scale: f32
) -> vec2<f32> {

    // Number of depth layers
    let min_layers = 8.0;
    let max_layers = 32.0;
    let num_layers = mix(max_layers, min_layers, abs(dot(vec3<f32>(0.0, 0.0, 1.0), view_dir_tangent)));

    // Calculate step size
    let layer_depth = 1.0 / num_layers;
    var current_layer_depth = 0.0;

    // Direction to step
    let p = view_dir_tangent.xy / view_dir_tangent.z * height_scale;
    let delta_uv = p / num_layers;

    var current_uv = uv;
    var current_depth = textureSample(t_height, s_sampler, current_uv).r;

    // Ray marching
    for (var i = 0; i < 32; i++) {
        if (current_layer_depth >= current_depth) {
            break;
        }

        current_uv -= delta_uv;
        current_depth = textureSample(t_height, s_sampler, current_uv).r;
        current_layer_depth += layer_depth;
    }

    // Parallax occlusion (find exact intersection)
    let prev_uv = current_uv + delta_uv;

    let after_depth = current_depth - current_layer_depth;
    let before_depth = textureSample(t_height, s_sampler, prev_uv).r
                     - current_layer_depth + layer_depth;

    let weight = after_depth / (after_depth - before_depth);
    let final_uv = mix(current_uv, prev_uv, weight);

    return final_uv;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate view direction in tangent space
    let view_dir_tangent = normalize(mat3x3<f32>(
        in.tangent,
        in.bitangent,
        in.normal
    ) * (u_camera.view_pos.xyz - in.world_pos));

    // Apply parallax
    let uv = parallax_occlusion_mapping(
        in.uv,
        view_dir_tangent,
        u_material.height_scale
    );

    // Sample textures with displaced UV
    let albedo = textureSample(t_albedo, s_sampler, uv);
    let normal_map = textureSample(t_normal, s_sampler, uv);

    // ... rest of PBR ...
}
```

#### Task 4.4: Texture Compression System
```rust
// ไฟล์: render/src/texture_compression.rs

pub struct CompressedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: CompressionFormat,
}

pub enum CompressionFormat {
    ASTC4x4,     // Mobile (4x4 blocks)
    ASTC8x8,     // Mobile (lower quality, higher compression)
    BC7,         // Desktop (DirectX)
    ETC2,        // Mobile fallback
    Uncompressed,
}

impl CompressedTexture {
    pub fn from_file(
        device: &Device,
        queue: &Queue,
        path: &str,
        target_format: CompressionFormat
    ) -> Result<Self, TextureError> {

        // Load image
        let img = image::open(path)?;
        let rgba = img.to_rgba8();

        // Choose format based on platform
        let format = Self::select_format(device, target_format);

        match format {
            CompressionFormat::ASTC4x4 => {
                // Use astc-encoder crate
                let compressed = astc_encoder::compress(
                    &rgba,
                    astc_encoder::Profile::LDR,
                    4, 4 // block size
                )?;

                Self::create_from_compressed(
                    device,
                    queue,
                    &compressed,
                    wgpu::TextureFormat::Astc {
                        block: AstcBlock::B4x4,
                        channel: AstcChannel::Unorm,
                    },
                    img.width(),
                    img.height()
                )
            },
            CompressionFormat::BC7 => {
                // Use intel-tex crate for BC7
                let compressed = intel_tex_2::bc7::compress_blocks(
                    intel_tex_2::RgbaSurface {
                        width: img.width(),
                        height: img.height(),
                        stride: img.width() * 4,
                        data: &rgba,
                    }
                );

                Self::create_from_compressed(
                    device,
                    queue,
                    &compressed,
                    wgpu::TextureFormat::Bc7RgbaUnormSrgb,
                    img.width(),
                    img.height()
                )
            },
            _ => {
                // Fallback to uncompressed
                Self::from_rgba8(device, queue, &rgba, img.width(), img.height())
            }
        }
    }

    fn select_format(device: &Device, target: CompressionFormat) -> CompressionFormat {
        // Check device features
        let features = device.features();

        match target {
            CompressionFormat::ASTC4x4 if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) => {
                CompressionFormat::ASTC4x4
            },
            CompressionFormat::BC7 if features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) => {
                CompressionFormat::BC7
            },
            CompressionFormat::ETC2 if features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) => {
                CompressionFormat::ETC2
            },
            _ => CompressionFormat::Uncompressed
        }
    }
}
```

### Checklist เฟส 4

- [ ] สร้าง SssMaterial struct และ uniform
- [ ] เขียน SSS shader (screen-space approximation)
- [ ] ทดสอบ SSS กับ character skin, leaves
- [ ] สร้าง ParallaxMaterial สำหรับ height mapping
- [ ] เขียน parallax occlusion mapping shader
- [ ] ทดสอบ parallax กับพื้นผิวต่างๆ (stone, brick)
- [ ] Implement texture compression system
- [ ] รองรับ ASTC (mobile), BC7 (desktop), ETC2 (fallback)
- [ ] สร้าง texture import pipeline พร้อม compression
- [ ] เพิ่ม mipmap generation อัตโนมัติ
- [ ] Optimize texture memory usage

**ผลลัพธ์**: Material มีความหลากหลาย, SSS สำหรับ organic materials, Parallax สำหรับความลึก

---

## ⚡ เฟส 5: Performance & Optimization (6 สัปดาห์)

**เป้าหมาย**: ประสิทธิภาพระดับ Production
**คะแนนที่เพิ่ม**: Performance: 8/10 → 10/10

### สัปดาห์ 23-25: LOD System

#### Task 5.1: LOD Component
```rust
// ไฟล์: engine/src/components/lod.rs

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct LodComponent {
    pub levels: Vec<LodLevel>,
    pub current_lod: usize,
    pub bias: f32, // LOD selection bias
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LodLevel {
    pub mesh: String,           // Mesh asset path
    pub distance: f32,          // Switch distance (meters)
    pub screen_coverage: f32,   // Alternative: switch by screen %
}

impl LodComponent {
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            current_lod: 0,
            bias: 1.0,
        }
    }

    pub fn add_level(&mut self, mesh: String, distance: f32) {
        self.levels.push(LodLevel {
            mesh,
            distance,
            screen_coverage: 0.0,
        });

        // Sort by distance
        self.levels.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    }

    pub fn select_lod(&mut self, distance_to_camera: f32) -> &str {
        for (i, level) in self.levels.iter().enumerate() {
            if distance_to_camera < level.distance * self.bias {
                self.current_lod = i;
                return &level.mesh;
            }
        }

        // Use lowest quality LOD
        if let Some(last) = self.levels.last() {
            self.current_lod = self.levels.len() - 1;
            &last.mesh
        } else {
            ""
        }
    }
}
```

#### Task 5.2: LOD System
```rust
// ไฟล์: engine/src/systems/lod_system.rs

pub struct LodSystem;

impl LodSystem {
    pub fn update(world: &mut World, camera_pos: Vec3) {
        let mut query = world.query::<(&mut LodComponent, &Transform)>();

        for (lod, transform) in query.iter() {
            let distance = (transform.position - camera_pos).length();
            let selected_mesh = lod.select_lod(distance);

            // Update MeshRenderer component
            if let Some(mesh_renderer) = world.get_component_mut::<MeshRenderer>(entity) {
                if mesh_renderer.mesh_path != selected_mesh {
                    mesh_renderer.mesh_path = selected_mesh.to_string();
                    mesh_renderer.dirty = true; // Force reload
                }
            }
        }
    }
}
```

#### Task 5.3: Automatic LOD Generation
```rust
// ไฟล์: tools/mesh_simplifier.rs

use meshopt::*;

pub struct MeshSimplifier;

impl MeshSimplifier {
    /// Generate LOD chain from base mesh
    pub fn generate_lods(
        base_mesh: &Mesh,
        lod_levels: &[f32] // [1.0, 0.5, 0.25, 0.125] (% of original)
    ) -> Vec<Mesh> {

        let mut lods = Vec::new();

        for &target_ratio in lod_levels {
            let target_index_count = (base_mesh.indices.len() as f32 * target_ratio) as usize;

            // Use meshopt for simplification
            let simplified_indices = meshopt::simplify(
                &base_mesh.indices,
                &base_mesh.vertices,
                target_index_count,
                1e-2, // target error
            );

            let simplified_mesh = Mesh {
                name: format!("{}_LOD{}", base_mesh.name, lods.len()),
                vertices: base_mesh.vertices.clone(), // Reuse vertices
                indices: simplified_indices,
                // ...
            };

            lods.push(simplified_mesh);
        }

        lods
    }
}
```

### สัปดาห์ 26-28: Occlusion Culling & GPU Optimization

#### Task 5.4: Frustum Culling (Basic)
```rust
// ไฟล์: render/src/culling.rs

pub struct Frustum {
    pub planes: [Plane; 6], // Left, Right, Bottom, Top, Near, Far
}

pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Frustum {
    pub fn from_view_projection(view_proj: &Mat4) -> Self {
        // Extract frustum planes from view-projection matrix
        let mut planes = [Plane::default(); 6];

        // Left plane
        planes[0] = Plane::new(
            Vec3::new(
                view_proj.w_axis.x + view_proj.x_axis.x,
                view_proj.w_axis.y + view_proj.x_axis.y,
                view_proj.w_axis.z + view_proj.x_axis.z,
            ),
            view_proj.w_axis.w + view_proj.x_axis.w
        );

        // Right plane
        planes[1] = Plane::new(
            Vec3::new(
                view_proj.w_axis.x - view_proj.x_axis.x,
                view_proj.w_axis.y - view_proj.x_axis.y,
                view_proj.w_axis.z - view_proj.x_axis.z,
            ),
            view_proj.w_axis.w - view_proj.x_axis.w
        );

        // ... Bottom, Top, Near, Far ...

        Self { planes }
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.normal.dot(center) + plane.distance;
            if distance < -radius {
                return false; // Outside frustum
            }
        }
        true
    }

    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find positive vertex (furthest along normal)
            let p = Vec3::new(
                if plane.normal.x > 0.0 { max.x } else { min.x },
                if plane.normal.y > 0.0 { max.y } else { min.y },
                if plane.normal.z > 0.0 { max.z } else { min.z },
            );

            if plane.normal.dot(p) + plane.distance < 0.0 {
                return false;
            }
        }
        true
    }
}
```

#### Task 5.5: Hardware Occlusion Queries
```rust
// ไฟล์: render/src/occlusion_culling.rs

pub struct OcclusionCullingSystem {
    query_sets: Vec<wgpu::QuerySet>,
    query_buffers: Vec<wgpu::Buffer>,
    results: Vec<u64>,
}

impl OcclusionCullingSystem {
    pub fn new(device: &Device, max_queries: u32) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Occlusion Queries"),
            ty: wgpu::QueryType::Occlusion,
            count: max_queries,
        });

        let query_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Occlusion Query Results"),
            size: (max_queries as u64) * 8, // u64 per query
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            query_sets: vec![query_set],
            query_buffers: vec![query_buffer],
            results: vec![0; max_queries as usize],
        }
    }

    pub fn render_bounding_boxes(
        &self,
        encoder: &mut CommandEncoder,
        render_pass: &mut RenderPass,
        entities: &[(Entity, AABB)]
    ) {
        // Render simplified bounding boxes with occlusion queries
        for (i, (entity, aabb)) in entities.iter().enumerate() {
            render_pass.begin_occlusion_query(i as u32);

            // Render bounding box (low-poly cube)
            self.render_bbox(render_pass, aabb);

            render_pass.end_occlusion_query();
        }
    }

    pub async fn get_results(&mut self, device: &Device) -> &[u64] {
        // Resolve queries to buffer
        // Read back results
        // Update self.results

        &self.results
    }
}
```

#### Task 5.6: GPU-Driven Rendering (Indirect Draw)
```rust
// ไฟล์: render/src/gpu_driven.rs

pub struct IndirectDrawBuffer {
    pub draw_commands: wgpu::Buffer,
    pub instance_data: wgpu::Buffer,
    pub command_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

impl IndirectDrawBuffer {
    pub fn render(&self, render_pass: &mut RenderPass) {
        render_pass.draw_indexed_indirect(
            &self.draw_commands,
            0, // offset
        );
    }
}
```

### Checklist เฟส 5

- [ ] สร้าง LodComponent และ LodLevel
- [ ] Implement LOD selection system (distance-based)
- [ ] สร้าง automatic LOD generation tool (meshopt)
- [ ] ทดสอบ LOD switching ในระยะต่างๆ
- [ ] Implement frustum culling (sphere และ AABB)
- [ ] สร้าง occlusion culling system (hardware queries)
- [ ] Implement GPU-driven rendering (indirect draw)
- [ ] Optimize batch rendering pipeline
- [ ] เพิ่ม instancing สำหรับ static objects
- [ ] Profile และ optimize hotspots
- [ ] สร้าง performance metrics dashboard

**ผลลัพธ์**: ประสิทธิภาพเพิ่มขึ้น 3-5 เท่า, รองรับ scene ขนาดใหญ่

---

## 🛠️ เฟส 6: Tooling & Developer Experience (6 สัปดาห์)

**เป้าหมาย**: เครื่องมือระดับมืออาชีพ
**คะแนนที่เพิ่ม**: Developer Experience: 6/10 → 9/10

### สัปดาห์ 29-31: Visual Shader Editor

#### Task 6.1: Shader Graph Architecture
```rust
// ไฟล์: editor/src/shader_graph/mod.rs

pub struct ShaderGraph {
    pub nodes: HashMap<NodeId, ShaderNode>,
    pub connections: Vec<Connection>,
    pub output_node: NodeId,
}

pub enum ShaderNode {
    // Inputs
    VertexPosition,
    VertexNormal,
    VertexTangent,
    VertexUV,

    // Textures
    TextureSample { texture_slot: u32 },

    // Math
    Add { a: NodeId, b: NodeId },
    Multiply { a: NodeId, b: NodeId },
    Dot { a: NodeId, b: NodeId },
    Normalize { input: NodeId },
    Lerp { a: NodeId, b: NodeId, t: NodeId },

    // Lighting
    Lambert { normal: NodeId, light_dir: NodeId },
    Phong { normal: NodeId, view_dir: NodeId, light_dir: NodeId, shininess: f32 },
    FresnelEffect { normal: NodeId, view_dir: NodeId, power: f32 },

    // Constants
    Float(f32),
    Vec3([f32; 3]),
    Color([f32; 4]),

    // Output
    FinalColor { albedo: NodeId, metallic: NodeId, roughness: NodeId },
}

pub struct Connection {
    pub from_node: NodeId,
    pub from_output: String,
    pub to_node: NodeId,
    pub to_input: String,
}

impl ShaderGraph {
    pub fn compile(&self) -> Result<String, ShaderCompileError> {
        // Generate WGSL code from graph
        let mut wgsl = String::new();

        // Header
        wgsl.push_str("@fragment\nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n");

        // Topological sort nodes
        let sorted_nodes = self.topological_sort()?;

        // Generate code for each node
        for node_id in sorted_nodes {
            let node = &self.nodes[&node_id];
            let code = self.generate_node_code(node, node_id);
            wgsl.push_str(&code);
        }

        // Output
        wgsl.push_str("    return final_color;\n}\n");

        Ok(wgsl)
    }

    fn generate_node_code(&self, node: &ShaderNode, id: NodeId) -> String {
        match node {
            ShaderNode::TextureSample { texture_slot } => {
                format!("    let node_{} = textureSample(t_texture_{}, s_sampler, in.uv);\n",
                       id, texture_slot)
            },
            ShaderNode::Add { a, b } => {
                format!("    let node_{} = node_{} + node_{};\n", id, a, b)
            },
            ShaderNode::Multiply { a, b } => {
                format!("    let node_{} = node_{} * node_{};\n", id, a, b)
            },
            // ... other nodes ...
            _ => String::new()
        }
    }
}
```

#### Task 6.2: Visual Editor UI (egui)
```rust
// ไฟล์: editor/src/shader_graph/editor_ui.rs

pub struct ShaderGraphEditorUi {
    graph: ShaderGraph,
    node_positions: HashMap<NodeId, egui::Pos2>,
    selected_node: Option<NodeId>,
    dragging_connection: Option<(NodeId, String)>,
}

impl ShaderGraphEditorUi {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Node palette (left sidebar)
        egui::SidePanel::left("node_palette").show(ui.ctx(), |ui| {
            ui.heading("Nodes");

            if ui.button("Texture Sample").clicked() {
                self.add_node(ShaderNode::TextureSample { texture_slot: 0 });
            }

            if ui.button("Add").clicked() {
                self.add_node(ShaderNode::Add { a: 0, b: 0 });
            }

            if ui.button("Multiply").clicked() {
                self.add_node(ShaderNode::Multiply { a: 0, b: 0 });
            }

            // ... more nodes ...
        });

        // Graph canvas
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag()
            );

            // Draw grid
            self.draw_grid(&painter, response.rect);

            // Draw connections
            for connection in &self.graph.connections {
                let from_pos = self.node_positions[&connection.from_node];
                let to_pos = self.node_positions[&connection.to_node];
                self.draw_connection(&painter, from_pos, to_pos);
            }

            // Draw nodes
            for (node_id, node) in &self.graph.nodes {
                let pos = self.node_positions[node_id];
                self.draw_node(ui, &painter, node_id, node, pos);
            }
        });

        // Properties panel (right sidebar)
        egui::SidePanel::right("properties").show(ui.ctx(), |ui| {
            ui.heading("Properties");

            if let Some(selected) = self.selected_node {
                self.show_node_properties(ui, selected);
            }
        });
    }

    fn draw_node(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node_id: NodeId,
        node: &ShaderNode,
        pos: egui::Pos2
    ) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(150.0, 80.0));

        // Background
        painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(50, 50, 50));

        // Border
        let border_color = if Some(node_id) == self.selected_node {
            egui::Color32::from_rgb(100, 150, 255)
        } else {
            egui::Color32::from_rgb(80, 80, 80)
        };
        painter.rect_stroke(rect, 5.0, egui::Stroke::new(2.0, border_color));

        // Title
        let title = self.get_node_title(node);
        painter.text(
            rect.min + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE
        );

        // Input/Output sockets
        // ...
    }
}
```

### สัปดาห์ 32-34: Profiling & Debugging Tools

#### Task 6.3: GPU Profiler
```rust
// ไฟล์: editor/src/profiler/gpu_profiler.rs

pub struct GpuProfiler {
    timestamp_queries: wgpu::QuerySet,
    timestamp_buffer: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,

    markers: Vec<ProfileMarker>,
    frame_times: VecDeque<FrameProfile>,
}

pub struct ProfileMarker {
    pub name: String,
    pub start_query: u32,
    pub end_query: u32,
}

pub struct FrameProfile {
    pub total_time: f64,
    pub passes: Vec<PassProfile>,
}

pub struct PassProfile {
    pub name: String,
    pub gpu_time: f64, // milliseconds
}

impl GpuProfiler {
    pub fn begin_pass(&mut self, encoder: &mut CommandEncoder, name: &str) {
        let query_index = self.markers.len() as u32 * 2;

        encoder.write_timestamp(&self.timestamp_queries, query_index);

        self.markers.push(ProfileMarker {
            name: name.to_string(),
            start_query: query_index,
            end_query: query_index + 1,
        });
    }

    pub fn end_pass(&mut self, encoder: &mut CommandEncoder) {
        if let Some(marker) = self.markers.last() {
            encoder.write_timestamp(&self.timestamp_queries, marker.end_query);
        }
    }

    pub async fn read_results(&mut self, device: &Device, queue: &Queue) -> Option<FrameProfile> {
        // Resolve timestamps
        queue.submit(Some(/* resolve command */));

        // Read back to CPU
        let buffer_slice = self.readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);

        let data = buffer_slice.get_mapped_range();
        let timestamps: &[u64] = bytemuck::cast_slice(&data);

        // Calculate pass times
        let mut passes = Vec::new();
        for marker in &self.markers {
            let start = timestamps[marker.start_query as usize];
            let end = timestamps[marker.end_query as usize];
            let duration_ns = end - start;
            let duration_ms = duration_ns as f64 / 1_000_000.0;

            passes.push(PassProfile {
                name: marker.name.clone(),
                gpu_time: duration_ms,
            });
        }

        self.markers.clear();

        Some(FrameProfile {
            total_time: passes.iter().map(|p| p.gpu_time).sum(),
            passes,
        })
    }
}
```

#### Task 6.4: Frame Debugger UI
```rust
// ไฟล์: editor/src/frame_debugger.rs

pub struct FrameDebugger {
    pub enabled: bool,
    pub paused: bool,
    pub current_frame: usize,
    pub captured_frames: Vec<CapturedFrame>,
    pub selected_pass: Option<usize>,
}

pub struct CapturedFrame {
    pub passes: Vec<RenderPass>,
    pub final_image: Vec<u8>,
}

pub struct RenderPass {
    pub name: String,
    pub draw_calls: Vec<DrawCall>,
    pub output_image: Vec<u8>,
}

pub struct DrawCall {
    pub mesh: String,
    pub material: String,
    pub vertex_count: u32,
    pub instance_count: u32,
    pub pipeline_state: PipelineState,
}

impl FrameDebugger {
    pub fn capture_frame(&mut self, render_system: &RenderSystem) {
        if !self.enabled {
            return;
        }

        // Capture all render passes
        let mut passes = Vec::new();

        // Shadow pass
        passes.push(self.capture_shadow_pass(render_system));

        // Main render pass
        passes.push(self.capture_main_pass(render_system));

        // Post-processing
        passes.push(self.capture_post_process(render_system));

        // Final composite
        let final_image = self.capture_final_output(render_system);

        self.captured_frames.push(CapturedFrame {
            passes,
            final_image,
        });

        // Limit history
        if self.captured_frames.len() > 120 {
            self.captured_frames.remove(0);
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Frame Debugger");

        ui.horizontal(|ui| {
            if ui.button(if self.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                self.paused = !self.paused;
            }

            ui.label(format!("Frame: {}", self.current_frame));
        });

        if let Some(frame) = self.captured_frames.last() {
            // Pass list
            egui::SidePanel::left("passes").show(ui.ctx(), |ui| {
                ui.heading("Render Passes");

                for (i, pass) in frame.passes.iter().enumerate() {
                    let selected = self.selected_pass == Some(i);
                    if ui.selectable_label(selected, &pass.name).clicked() {
                        self.selected_pass = Some(i);
                    }

                    ui.label(format!("  {} draw calls", pass.draw_calls.len()));
                }
            });

            // Pass details
            egui::CentralPanel::default().show(ui.ctx(), |ui| {
                if let Some(pass_index) = self.selected_pass {
                    let pass = &frame.passes[pass_index];

                    ui.heading(&pass.name);

                    // Show output image
                    // Show draw call list
                    // Show pipeline state
                }
            });
        }
    }
}
```

### Checklist เฟส 6

- [ ] สร้าง ShaderGraph data structure
- [ ] Implement WGSL code generation from graph
- [ ] สร้าง visual shader editor UI (egui-based)
- [ ] เพิ่ม node palette (texture, math, lighting nodes)
- [ ] Implement connection dragging and validation
- [ ] สร้าง shader compilation และ hot-reload
- [ ] ทดสอบ shader graph กับ PBR materials
- [ ] สร้าง GPU profiler (timestamp queries)
- [ ] Implement frame debugger (capture render passes)
- [ ] สร้าง performance metrics UI
- [ ] เพิ่ม memory profiler
- [ ] สร้าง render statistics display

**ผลลัพธ์**: เครื่องมือครบครัน, สะดวกในการ debug และ optimize

---

## 🌐 เฟส 7: Platform Optimization & Polish (4 สัปดาห์)

**เป้าหมาย**: เพิ่มประสิทธิภาพ cross-platform และขัดเกลา
**คะแนนที่เพิ่ม**: Platform Support: 8/10 → 10/10

### สัปดาห์ 35-36: Mobile-Specific Optimizations

#### Task 7.1: Adaptive Quality Settings
```rust
// ไฟล์: engine/src/graphics_settings.rs

pub struct AdaptiveQualitySettings {
    pub target_fps: f32,
    pub current_quality: QualityLevel,
    pub frame_times: VecDeque<f32>,
    pub adjustment_cooldown: f32,
}

#[derive(Clone, Copy)]
pub enum QualityLevel {
    Ultra,
    High,
    Medium,
    Low,
}

impl AdaptiveQualitySettings {
    pub fn update(&mut self, delta_time: f32, actual_fps: f32) {
        self.frame_times.push_back(delta_time);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }

        self.adjustment_cooldown -= delta_time;

        if self.adjustment_cooldown <= 0.0 {
            let avg_fps: f32 = self.frame_times.len() as f32
                             / self.frame_times.iter().sum::<f32>();

            if avg_fps < self.target_fps * 0.9 {
                // Decrease quality
                self.decrease_quality();
                self.adjustment_cooldown = 2.0; // Wait 2 seconds
            } else if avg_fps > self.target_fps * 1.1 {
                // Increase quality
                self.increase_quality();
                self.adjustment_cooldown = 5.0; // Wait 5 seconds
            }
        }
    }

    pub fn apply_settings(&self, render_settings: &mut RenderSettings) {
        match self.current_quality {
            QualityLevel::Ultra => {
                render_settings.shadow_resolution = 2048;
                render_settings.shadow_cascades = 4;
                render_settings.bloom_quality = BloomQuality::High;
                render_settings.post_processing_enabled = true;
            },
            QualityLevel::High => {
                render_settings.shadow_resolution = 1024;
                render_settings.shadow_cascades = 3;
                render_settings.bloom_quality = BloomQuality::Medium;
                render_settings.post_processing_enabled = true;
            },
            QualityLevel::Medium => {
                render_settings.shadow_resolution = 512;
                render_settings.shadow_cascades = 2;
                render_settings.bloom_quality = BloomQuality::Low;
                render_settings.post_processing_enabled = true;
            },
            QualityLevel::Low => {
                render_settings.shadow_resolution = 256;
                render_settings.shadow_cascades = 1;
                render_settings.bloom_quality = BloomQuality::Disabled;
                render_settings.post_processing_enabled = false;
            },
        }
    }
}
```

#### Task 7.2: Thermal Throttling Detection
```rust
// ไฟล์: engine/src/platform/thermal.rs

pub struct ThermalMonitor {
    pub temperature: f32,
    pub throttling_active: bool,
    pub temperature_history: VecDeque<f32>,
}

impl ThermalMonitor {
    pub fn update(&mut self) {
        #[cfg(target_os = "android")]
        {
            // Android thermal API
            self.temperature = Self::get_android_thermal_status();
        }

        #[cfg(target_os = "ios")]
        {
            // iOS thermal API
            self.temperature = Self::get_ios_thermal_status();
        }

        self.temperature_history.push_back(self.temperature);
        if self.temperature_history.len() > 30 {
            self.temperature_history.pop_front();
        }

        // Check for throttling
        self.throttling_active = self.temperature > 0.8; // >80% of max
    }

    pub fn get_recommended_quality(&self) -> QualityLevel {
        if self.throttling_active {
            QualityLevel::Low
        } else if self.temperature > 0.6 {
            QualityLevel::Medium
        } else {
            QualityLevel::High
        }
    }

    #[cfg(target_os = "android")]
    fn get_android_thermal_status() -> f32 {
        // JNI call to Android ThermalManager
        // Return normalized 0.0-1.0
        0.5
    }

    #[cfg(target_os = "ios")]
    fn get_ios_thermal_status() -> f32 {
        // Objective-C call to ProcessInfo.processInfo.thermalState
        0.5
    }
}
```

### สัปดาห์ 37-38: Final Polish & Testing

#### Task 7.3: Automated Performance Testing
```rust
// ไฟล์: tests/performance_tests.rs

#[test]
fn test_render_performance_1000_lights() {
    let mut engine = TestEngine::new();

    // Setup scene with 1000 lights
    for i in 0..1000 {
        engine.spawn_point_light(
            Vec3::new(
                (i % 10) as f32 * 10.0,
                5.0,
                (i / 10) as f32 * 10.0
            ),
            Color::rgb(1.0, 1.0, 1.0),
            10.0 // radius
        );
    }

    // Render 100 frames
    let mut frame_times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        engine.render_frame();
        let duration = start.elapsed();
        frame_times.push(duration.as_secs_f32());
    }

    // Calculate metrics
    let avg_frame_time: f32 = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
    let avg_fps = 1.0 / avg_frame_time;

    println!("1000 Lights Test:");
    println!("  Average Frame Time: {:.2}ms", avg_frame_time * 1000.0);
    println!("  Average FPS: {:.1}", avg_fps);

    // Assert performance target (60 FPS = 16.67ms)
    assert!(avg_frame_time < 0.0167, "Failed to maintain 60 FPS with 1000 lights");
}

#[test]
fn test_post_processing_cost() {
    let mut engine = TestEngine::new();

    // Measure without post-processing
    engine.set_post_processing(false);
    let without_pp = measure_average_frame_time(&mut engine, 100);

    // Measure with post-processing
    engine.set_post_processing(true);
    let with_pp = measure_average_frame_time(&mut engine, 100);

    let overhead = (with_pp - without_pp) / without_pp * 100.0;

    println!("Post-Processing Overhead: {:.1}%", overhead);

    // Assert overhead is acceptable (<20%)
    assert!(overhead < 20.0, "Post-processing overhead too high");
}
```

#### Task 7.4: Quality Assurance Checklist
```markdown
# Mobile Render System QA Checklist

## Performance (60 FPS target on high-end mobile)
- [ ] Empty scene: >60 FPS
- [ ] 1000 dynamic lights: >60 FPS
- [ ] 10,000 sprites (batched): >60 FPS
- [ ] Complex 3D scene (100k triangles): >60 FPS
- [ ] With full post-processing: >60 FPS
- [ ] Thermal throttling handled gracefully

## Visual Quality
- [ ] HDR rendering works correctly
- [ ] ACES tonemapping accurate colors
- [ ] Bloom effect smooth and natural
- [ ] Shadows free of artifacts
- [ ] No color banding
- [ ] Vignette smooth falloff
- [ ] Materials render correctly (PBR, SSS, Toon)

## Features
- [ ] Clustered lighting (1024 lights)
- [ ] Point lights working
- [ ] Spot lights working
- [ ] 4 shadow cascades
- [ ] PCSS soft shadows
- [ ] Contact shadows
- [ ] LOD system working
- [ ] Frustum culling working
- [ ] Occlusion culling working
- [ ] Texture compression (ASTC/BC7/ETC2)

## Cross-Platform
- [ ] Windows (DirectX 12): Working
- [ ] Windows (Vulkan): Working
- [ ] macOS (Metal): Working
- [ ] Linux (Vulkan): Working
- [ ] iOS (Metal): Working
- [ ] Android (Vulkan): Working

## Tools
- [ ] Visual shader editor working
- [ ] GPU profiler accurate
- [ ] Frame debugger captures correctly
- [ ] Performance metrics display

## Stability
- [ ] No crashes in 1-hour stress test
- [ ] No memory leaks
- [ ] No GPU memory leaks
- [ ] Handles window resize correctly
- [ ] Handles device loss gracefully
```

### Checklist เฟส 7

- [ ] Implement adaptive quality system
- [ ] สร้าง thermal monitoring (iOS/Android)
- [ ] ทดสอบ performance บนมือถือจริง (iOS/Android)
- [ ] Optimize shader precision (f16 on mobile)
- [ ] Implement dynamic resolution scaling
- [ ] สร้าง automated performance tests
- [ ] Run full QA checklist
- [ ] Fix all critical bugs
- [ ] Optimize memory usage
- [ ] Profile และ optimize final hotspots
- [ ] สร้าง documentation สำหรับทุก feature
- [ ] Create example projects showcasing features

**ผลลัพธ์**: ระบบ render ที่เสถียร, ประสิทธิภาพสูง, พร้อม production

---

## 📈 ติดตามความคืบหน้า

### Milestone Tracking

| เฟส | คะแนนเป้าหมาย | สถานะ | กำหนดเวลา |
|-----|---------------|--------|-----------|
| เฟส 1: HDR Foundation | +1.5 (7.5/10) | ⏳ Pending | สัปดาห์ 1-4 |
| เฟส 2: Post-Processing | +2.5 (10/10) | ⏳ Pending | สัปดาห์ 5-10 |
| เฟส 3: Advanced Lighting | +3.0 (10/10) | ⏳ Pending | สัปดาห์ 11-16 |
| เฟส 4: Material System | +2.5 (10/10) | ⏳ Pending | สัปดาห์ 17-22 |
| เฟส 5: Optimization | +2.0 (10/10) | ⏳ Pending | สัปดาห์ 23-28 |
| เฟส 6: Tooling | +3.0 (9/10) | ⏳ Pending | สัปดาห์ 29-34 |
| เฟส 7: Polish | +2.0 (10/10) | ⏳ Pending | สัปดาห์ 35-38 |
| **รวม** | **78 → 100** | **0%** | **9 เดือน** |

### KPI (Key Performance Indicators)

| Metric | ปัจจุบัน | เป้าหมาย |
|--------|----------|----------|
| Empty Scene FPS | ~200 | >240 |
| 1000 Lights FPS | N/A | >60 |
| 10K Sprites FPS | ~120 | >60 |
| Post-Processing Overhead | N/A | <20% |
| Memory Usage (Textures) | ~500MB | <300MB (compressed) |
| Shadow Quality | Medium | Ultra |
| Material Types | 2 | 5+ |
| Developer Tools | 2/10 | 9/10 |

---

## 🎯 สรุป: เส้นทางสู่ 100/100

### จุดเปลี่ยนสำคัญ (Critical Path)

1. **เฟส 2 (Post-Processing)**: ที่สำคัญที่สุด - เพิ่ม +4 คะแนนจาก HDR/Post-Processing
2. **เฟส 3 (Lighting & Shadows)**: เพิ่มคุณภาพภาพ - เพิ่ม +3 คะแนน
3. **เฟส 5 (Optimization)**: จำเป็นสำหรับ production - เพิ่ม +2 คะแนน

### ความท้าทายที่คาดการณ์

1. **Shader Complexity**: WGSL shader อาจซับซ้อน → แก้: สร้าง shader library
2. **Mobile Performance**: Thermal throttling → แก้: Adaptive quality system
3. **Cross-Platform Testing**: ต้องทดสอบหลาย platform → แก้: Automated CI/CD
4. **Visual Editor Complexity**: Shader graph ใหญ่ → แก้: Incremental development

### ผลลัพธ์ที่คาดหวัง

หลังจากเสร็จทั้ง 7 เฟส, Rust 2D Game Engine จะมี:

✅ ระบบ Render บนมือถือ **ระดับโลก** (100/100)
✅ คุณภาพภาพเทียบเท่า **Unreal Engine 5**
✅ ประสิทธิภาพดีกว่า Unity/Godot (Rust advantage)
✅ เครื่องมือระดับมืออาชีพ (Visual Shader Editor, Profiler)
✅ พร้อม **Production** สำหรับเกม AAA บนมือถือ

---

**เริ่มต้นจากเฟส 1 และสร้างระบบ HDR Post-Processing ที่จะเปลี่ยนเกมทั้งหมด! 🚀**
