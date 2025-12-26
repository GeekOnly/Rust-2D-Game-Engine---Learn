# 🎯 วิเคราะห์ Mobile-First Architecture Strategy

**วันที่**: 26 ธันวาคม 2025
**คำถาม**: ระบบนี้ยังเน้น Mobile First ใช่ไหม?
**คำตอบ**: ใช่ แต่ควรเป็น **"Mobile-First with Desktop Excellence"**

---

## 📊 สถานะปัจจุบัน

### ✅ จุดแข็ง Mobile-First ที่มีอยู่

1. **Clustered Forward+ Lighting**
   - เหมาะกับ Mobile TBDR GPUs (Tile-Based Deferred Rendering)
   - ประหยัด bandwidth เมื่อเทียบกับ Deferred Rendering

2. **Reverse-Z Depth**
   - ประหยัด precision, ลด Z-fighting
   - เหมาะกับ Mobile GPUs ที่มี precision จำกัด

3. **Dual Depth Textures**
   - On-chip optimization สำหรับ Contact Shadows
   - ลด memory bandwidth (สำคัญมากบน Mobile)

4. **HDR Post-Processing Plan**
   - Uber-Shader (single-pass) แทน multiple passes
   - ลด "Render Pass Hell" บน Mobile
   - R11G11B10Float (ประหยัด bandwidth 50%)

### ⚠️ ข้อกังวล: Deferred Rendering บน Mobile

**ปัญหา**:
```
G-Buffer Memory Usage:
- Albedo+Metallic:  4 bytes/pixel (Rgba8UnormSrgb)
- Normal+Roughness: 8 bytes/pixel (Rgba16Snorm)
- Emissive+AO:      8 bytes/pixel (Rgba16Float)
Total: 20 bytes/pixel

1080p: 1920×1080 × 20 = 41.5 MB (G-Buffer only!)
1440p: 2560×1440 × 20 = 73.7 MB
4K:    3840×2160 × 20 = 166 MB
```

**Bandwidth Cost**:
- Mobile GPUs: ~50-100 GB/s
- Desktop GPUs: ~300-1000 GB/s
- Deferred อ่าน G-Buffer หลายครั้ง = bandwidth intensive!

---

## 🎯 กลยุทธ์ที่แนะนำ: "Mobile-First, Desktop-Optimized"

### แนวคิด

```
Priority 1: Mobile (60 FPS @ High-End, 30 FPS @ Mid-Range)
Priority 2: Desktop (144+ FPS, Advanced Features)
```

### Render Path Strategy

| Platform | Default Path | Optional Paths | เหตุผล |
|----------|--------------|----------------|--------|
| **Mobile Low-End** | Forward | - | เร็วที่สุด, ใช้ memory น้อย |
| **Mobile Mid-Range** | Forward+ | Forward | Clustered lighting, ยังประหยัด |
| **Mobile High-End** | Forward+ | Forward, Deferred¹ | เต็มประสิทธิภาพ |
| **Desktop** | Forward+ | Forward, Deferred | ยืดหยุ่น, ปรับตามเกม |

¹ Deferred บน Mobile: **เฉพาะ high-end** (Snapdragon 8 Gen 2+, A16+) และ **ต้อง optimize**

---

## 🏗️ แผนปรับปรุง: Mobile-First Architecture (Revised)

### Phase A: Mobile-Optimized MRT (สัปดาห์ 1)

#### เปลี่ยนแปลง

**Before**:
```rust
// G-Buffer: 20 bytes/pixel (ไม่เหมาะกับ Mobile!)
pub struct GBuffer {
    pub albedo_metallic: RenderTarget,    // Rgba8UnormSrgb (4 bytes)
    pub normal_roughness: RenderTarget,   // Rgba16Snorm (8 bytes)
    pub emissive_ao: RenderTarget,        // Rgba16Float (8 bytes)
}
```

**After (Mobile-Optimized)**:
```rust
// Compact G-Buffer: 12 bytes/pixel (40% reduction!)
pub struct MobileGBuffer {
    pub albedo_metallic: RenderTarget,    // Rgba8UnormSrgb (4 bytes)
    pub normal_roughness: RenderTarget,   // Rgba8Unorm (4 bytes) ← ลดจาก 8
    pub emissive: RenderTarget,           // Rgba8Unorm (4 bytes) ← ลดจาก 8
}

impl MobileGBuffer {
    pub fn memory_footprint(&self, width: u32, height: u32) -> f32 {
        (width * height * 12) as f32 / 1_048_576.0 // MB
    }
}
```

**ผลลัพธ์**:
- 1080p: 41.5 MB → **24.9 MB** (-40%)
- 1440p: 73.7 MB → **44.2 MB** (-40%)

#### Task A.1: Compact G-Buffer Encoding

```rust
// ไฟล์: render/src/mobile_gbuffer.rs (ไฟล์ใหม่)

use super::{RenderTarget, RenderTargetFormat};

pub struct MobileGBuffer {
    pub albedo_metallic: RenderTarget,
    pub normal_roughness: RenderTarget,
    pub emissive: RenderTarget,
    pub depth: RenderTarget,
}

impl MobileGBuffer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        Self {
            // RT0: Albedo (RGB) + Metallic (A)
            albedo_metallic: RenderTarget::new(
                device,
                "MobileGBuffer_AlbedoMetallic",
                width,
                height,
                RenderTargetFormat::Rgba8UnormSrgb,
            ),

            // RT1: Normal (RGB, packed) + Roughness (A)
            // Normal: Octahedral encoding (2 components) stored in RG
            normal_roughness: RenderTarget::new(
                device,
                "MobileGBuffer_NormalRoughness",
                width,
                height,
                RenderTargetFormat::Rgba8Unorm, // ← 4 bytes แทน 8
            ),

            // RT2: Emissive (RGB) + AO (A)
            // Emissive: Encoded with simple gamma
            emissive: RenderTarget::new(
                device,
                "MobileGBuffer_Emissive",
                width,
                height,
                RenderTargetFormat::Rgba8Unorm, // ← 4 bytes แทน 8
            ),

            depth: RenderTarget::new(
                device,
                "MobileGBuffer_Depth",
                width,
                height,
                RenderTargetFormat::Rgba16Float,
            ),
        }
    }

    pub fn memory_footprint_mb(&self) -> f32 {
        let pixels = (self.albedo_metallic.width * self.albedo_metallic.height) as f32;
        pixels * 12.0 / 1_048_576.0
    }

    pub fn bandwidth_cost_mb_per_frame(&self) -> f32 {
        // Write once (geometry pass) + Read once (lighting pass)
        self.memory_footprint_mb() * 2.0
    }
}
```

#### Task A.2: Octahedral Normal Encoding (WGSL)

```wgsl
// ไฟล์: render/assets/shaders/normal_encoding.wgsl

// Encode normal vector to 2D octahedral map
fn encode_normal_octahedral(n: vec3<f32>) -> vec2<f32> {
    // Project onto octahedron, then onto xy plane
    let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));

    // Fold the four quadrants
    var result = p;
    if (n.z <= 0.0) {
        result = (1.0 - abs(p.yx)) * sign_not_zero(p);
    }

    return result * 0.5 + 0.5; // Map to [0, 1]
}

fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        select(-1.0, 1.0, v.x >= 0.0),
        select(-1.0, 1.0, v.y >= 0.0)
    );
}

// Decode 2D octahedral map to normal vector
fn decode_normal_octahedral(encoded: vec2<f32>) -> vec3<f32> {
    let f = encoded * 2.0 - 1.0; // Map from [0, 1] to [-1, 1]

    var n = vec3<f32>(f.x, f.y, 1.0 - abs(f.x) - abs(f.y));
    let t = saturate(-n.z);

    n.x += select(t, -t, n.x >= 0.0);
    n.y += select(t, -t, n.y >= 0.0);

    return normalize(n);
}

// Geometry pass: Write to G-Buffer
@fragment
fn fs_gbuffer(in: VertexOutput) -> GBufferOutput {
    var output: GBufferOutput;

    // Sample textures
    let albedo = textureSample(t_albedo, s_sampler, in.uv);
    let metallic = textureSample(t_metallic, s_sampler, in.uv).r;
    let roughness = textureSample(t_roughness, s_sampler, in.uv).r;
    let normal = normalize(in.normal); // World space

    // RT0: Albedo + Metallic
    output.albedo_metallic = vec4<f32>(albedo.rgb, metallic);

    // RT1: Normal (octahedral) + Roughness
    let encoded_normal = encode_normal_octahedral(normal);
    output.normal_roughness = vec4<f32>(encoded_normal, 0.0, roughness);

    // RT2: Emissive + AO
    let emissive = textureSample(t_emissive, s_sampler, in.uv).rgb;
    output.emissive = vec4<f32>(emissive, 1.0);

    return output;
}

// Lighting pass: Read from G-Buffer
@fragment
fn fs_lighting(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample G-Buffer
    let albedo_metallic = textureSample(t_gbuffer0, s_sampler, in.uv);
    let normal_roughness = textureSample(t_gbuffer1, s_sampler, in.uv);
    let emissive = textureSample(t_gbuffer2, s_sampler, in.uv);

    // Decode
    let albedo = albedo_metallic.rgb;
    let metallic = albedo_metallic.a;
    let normal = decode_normal_octahedral(normal_roughness.xy);
    let roughness = normal_roughness.a;

    // PBR lighting calculations...
    let final_color = calculate_pbr_lighting(albedo, normal, metallic, roughness);

    return vec4<f32>(final_color + emissive.rgb, 1.0);
}
```

### Phase B: Adaptive Render Path (สัปดาห์ 2)

#### Task B.1: Device Capability Detection

```rust
// ไฟล์: render/src/device_profile.rs (ไฟล์ใหม่)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTier {
    /// Mobile low-end: Snapdragon 600 series, A12 Bionic
    MobileLow,

    /// Mobile mid-range: Snapdragon 700 series, A14 Bionic
    MobileMid,

    /// Mobile high-end: Snapdragon 8 Gen 2+, A16 Bionic+
    MobileHigh,

    /// Desktop integrated: Intel UHD, AMD Vega
    DesktopIntegrated,

    /// Desktop discrete: GTX 1060+, RX 580+
    DesktopDiscrete,
}

pub struct DeviceProfile {
    pub tier: DeviceTier,
    pub vram_mb: u32,
    pub bandwidth_gb_s: f32,
    pub max_texture_size: u32,
    pub supports_compute: bool,
}

impl DeviceProfile {
    pub fn detect(adapter: &wgpu::Adapter) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();

        // Detect based on adapter info
        let tier = Self::detect_tier(&info);

        let vram_mb = match tier {
            DeviceTier::MobileLow => 2048,
            DeviceTier::MobileMid => 4096,
            DeviceTier::MobileHigh => 6144,
            DeviceTier::DesktopIntegrated => 4096,
            DeviceTier::DesktopDiscrete => 8192,
        };

        let bandwidth_gb_s = match tier {
            DeviceTier::MobileLow => 25.0,
            DeviceTier::MobileMid => 50.0,
            DeviceTier::MobileHigh => 100.0,
            DeviceTier::DesktopIntegrated => 50.0,
            DeviceTier::DesktopDiscrete => 300.0,
        };

        Self {
            tier,
            vram_mb,
            bandwidth_gb_s,
            max_texture_size: limits.max_texture_dimension_2d,
            supports_compute: true, // WGPU always supports compute
        }
    }

    fn detect_tier(info: &wgpu::AdapterInfo) -> DeviceTier {
        // Check backend
        let is_mobile = matches!(info.backend, wgpu::Backend::Metal)
                     && cfg!(target_os = "ios")
                     || matches!(info.backend, wgpu::Backend::Vulkan)
                     && cfg!(target_os = "android");

        if is_mobile {
            // TODO: Detect actual GPU model via platform APIs
            DeviceTier::MobileHigh // Conservative default
        } else {
            // Desktop
            if info.device_type == wgpu::DeviceType::IntegratedGpu {
                DeviceTier::DesktopIntegrated
            } else {
                DeviceTier::DesktopDiscrete
            }
        }
    }

    pub fn recommended_render_path(&self) -> RenderPath {
        match self.tier {
            DeviceTier::MobileLow => RenderPath::Forward,
            DeviceTier::MobileMid => RenderPath::ForwardPlus,
            DeviceTier::MobileHigh => RenderPath::ForwardPlus, // NOT Deferred by default
            DeviceTier::DesktopIntegrated => RenderPath::ForwardPlus,
            DeviceTier::DesktopDiscrete => RenderPath::ForwardPlus,
        }
    }

    pub fn supports_deferred(&self) -> bool {
        matches!(self.tier,
            DeviceTier::MobileHigh |
            DeviceTier::DesktopIntegrated |
            DeviceTier::DesktopDiscrete
        )
    }

    pub fn max_dynamic_lights(&self) -> u32 {
        match self.tier {
            DeviceTier::MobileLow => 16,
            DeviceTier::MobileMid => 64,
            DeviceTier::MobileHigh => 256,
            DeviceTier::DesktopIntegrated => 512,
            DeviceTier::DesktopDiscrete => 1024,
        }
    }
}
```

#### Task B.2: Auto-Select Render Path

```rust
// ไฟล์: render/src/render_path.rs (ปรับปรุง)

impl RenderPathManager {
    pub fn auto_select_path(
        device_profile: &DeviceProfile,
        scene_light_count: u32,
    ) -> RenderPath {
        let recommended = device_profile.recommended_render_path();

        // Override based on scene complexity
        if scene_light_count <= 8 {
            // Simple lighting: Forward is enough
            RenderPath::Forward
        } else if scene_light_count <= device_profile.max_dynamic_lights() / 2 {
            // Moderate lighting: Forward+ is optimal
            RenderPath::ForwardPlus
        } else if device_profile.supports_deferred() && scene_light_count > 100 {
            // Many lights + capable device: Consider Deferred
            println!("WARNING: Many lights ({}), consider Deferred (experimental on mobile)",
                     scene_light_count);
            RenderPath::ForwardPlus // Still default to Forward+
        } else {
            recommended
        }
    }

    pub fn can_enable_deferred(&self, device_profile: &DeviceProfile) -> bool {
        if !device_profile.supports_deferred() {
            println!("Deferred rendering not supported on this device");
            return false;
        }

        // Check memory budget
        let resolution = (1920, 1080); // Assume 1080p
        let gbuffer_mb = MobileGBuffer::estimate_memory(resolution.0, resolution.1);

        if gbuffer_mb > device_profile.vram_mb as f32 * 0.3 {
            println!("G-Buffer ({:.1} MB) exceeds 30% of VRAM budget", gbuffer_mb);
            return false;
        }

        true
    }
}
```

### Phase C: Mobile-First Render Graph (สัปดาห์ 3)

#### Task C.1: Bandwidth-Aware Pass Ordering

```rust
// ไฟล์: render/src/mobile_render_graph.rs

impl RenderGraph {
    pub fn optimize_for_mobile(&mut self, device_profile: &DeviceProfile) {
        if !device_profile.tier.is_mobile() {
            return; // Desktop doesn't need aggressive optimization
        }

        // Mobile optimization: Minimize texture reads/writes
        self.merge_compatible_passes();
        self.reorder_for_bandwidth();
        self.enable_on_chip_resolve();
    }

    fn merge_compatible_passes(&mut self) {
        // Example: Merge multiple small post-processing passes into one
        println!("Merging compatible passes for mobile...");
    }

    fn reorder_for_bandwidth(&mut self) {
        // Reorder passes to maximize on-chip cache hits
        println!("Reordering passes for bandwidth optimization...");
    }

    fn enable_on_chip_resolve(&mut self) {
        // Enable TBDR-friendly features
        println!("Enabling on-chip resolve...");
    }
}

impl DeviceTier {
    pub fn is_mobile(&self) -> bool {
        matches!(self,
            DeviceTier::MobileLow |
            DeviceTier::MobileMid |
            DeviceTier::MobileHigh
        )
    }
}
```

### Phase D: Format Selection (สัปดาห์ 4)

#### Task D.1: Platform-Specific Format Selection

```rust
// ไฟล์: render/src/render_target.rs (ปรับปรุง)

impl RenderTarget {
    pub fn select_optimal_format(
        device_profile: &DeviceProfile,
        usage: RenderTargetUsage,
    ) -> RenderTargetFormat {
        match (device_profile.tier, usage) {
            // Mobile: Always prefer compact formats
            (DeviceTier::MobileLow | DeviceTier::MobileMid, RenderTargetUsage::HDR) => {
                RenderTargetFormat::R11G11B10Float // 50% bandwidth savings!
            },
            (DeviceTier::MobileHigh, RenderTargetUsage::HDR) => {
                RenderTargetFormat::Rgba16Float // Full quality on high-end
            },

            // Desktop: Prefer quality
            (DeviceTier::DesktopDiscrete, RenderTargetUsage::HDR) => {
                RenderTargetFormat::Rgba16Float
            },

            // G-Buffer: Always compact on mobile
            (tier, RenderTargetUsage::GBufferNormal) if tier.is_mobile() => {
                RenderTargetFormat::Rgba8Unorm // Octahedral encoding
            },
            (_, RenderTargetUsage::GBufferNormal) => {
                RenderTargetFormat::Rgba16Snorm // Full precision on desktop
            },

            _ => RenderTargetFormat::Rgba8UnormSrgb,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RenderTargetUsage {
    HDR,
    GBufferAlbedo,
    GBufferNormal,
    GBufferEmissive,
    Depth,
}
```

---

## 📊 สรุป: Mobile-First Strategy (Final)

### กลยุทธ์

```
1. Default: Forward+ (ทุก platform)
2. Fallback: Forward (Low-end mobile)
3. Optional: Deferred (High-end only, manual enable)
```

### Memory Budget

| Platform | HDR Buffer | G-Buffer | Total | สถานะ |
|----------|-----------|----------|-------|-------|
| Mobile Low | - | - | 0 MB | ✅ Forward only |
| Mobile Mid | 8 MB | - | 8 MB | ✅ Forward+ |
| Mobile High | 8 MB | 25 MB¹ | 33 MB | ⚠️ Deferred optional |
| Desktop | 8 MB | 42 MB² | 50 MB | ✅ Deferred OK |

¹ Mobile G-Buffer: 12 bytes/pixel (compact encoding)
² Desktop G-Buffer: 20 bytes/pixel (full precision)

### Bandwidth Cost (1080p @ 60 FPS)

| Render Path | Bandwidth/Frame | Bandwidth/Sec | Mobile OK? |
|-------------|----------------|---------------|------------|
| Forward | ~10 MB | 600 MB/s | ✅ Excellent |
| Forward+ | ~20 MB | 1.2 GB/s | ✅ Good |
| Deferred (Mobile) | ~50 MB | 3.0 GB/s | ⚠️ High-end only |
| Deferred (Desktop) | ~80 MB | 4.8 GB/s | ✅ OK |

---

## ✅ คำตอบสุดท้าย

### ระบบนี้ยังเน้น Mobile-First ใช่ไหม?

**ใช่ครับ!** แต่เป็น **"Intelligent Mobile-First"**:

1. **Default = Forward+** (เหมาะทั้ง Mobile และ Desktop)
2. **Compact G-Buffer** (12 bytes/pixel แทน 20 bytes/pixel)
3. **Octahedral Normal Encoding** (ประหยัด 50% ใน G-Buffer)
4. **R11G11B10Float HDR** (ประหยัด 50% bandwidth)
5. **Auto-Detect Device Tier** (ปรับอัตโนมัติตามเครื่อง)
6. **Deferred = Optional** (เฉพาะ Desktop/High-End Mobile ที่เปิดเอง)

### สรุป Architecture

```rust
Mobile Low     → Forward (16 lights max)
Mobile Mid     → Forward+ (64 lights, R11G11B10Float)
Mobile High    → Forward+ (256 lights, Rgba16Float)
                 └─ Deferred available (manual enable, compact G-Buffer)

Desktop        → Forward+ default
                 └─ Deferred available (full-precision G-Buffer)
```

**ผลลัพธ์**: ยืดหยุ่น, Mobile-First, แต่ไม่เสีย Desktop performance! 🎯
