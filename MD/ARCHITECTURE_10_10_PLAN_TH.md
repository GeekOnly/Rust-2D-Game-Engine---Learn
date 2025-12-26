# 🏗️ แผนยกระดับ Rendering Architecture สู่ 10/10

**วันที่**: 26 ธันวาคม 2025
**คะแนนปัจจุบัน**: 8.5/10
**คะแนนเป้าหมาย**: 10/10
**ระยะเวลา**: 3-4 สัปดาห์
**ผลที่ได้**: สถาปัตยกรรม Rendering ที่ยืดหยุ่น, ทรงพลัง, และทันสมัยที่สุด

---

## 📊 วิเคราะห์ช่องว่าง (Gap Analysis)

### จุดแข็งที่มีอยู่ ✅

1. **Clustered Forward+ Lighting** - ระบบแสงทันสมัย (1024 lights, 64/cluster)
2. **WGPU Cross-Platform** - รองรับ Vulkan/Metal/DX12
3. **Reverse-Z Depth** - ความแม่นยำสูง
4. **Dual Depth Textures** - รองรับ Contact Shadows
5. **Smart Render Cache** - ระบบ cache มีประสิทธิภาพ
6. **Pipeline Separation** - Main, Shadow, Depth pre-pass

### จุดอ่อนที่ต้องแก้ไข ⚠️

| ปัญหา | ผลกระทบ | คะแนนที่เสีย |
|-------|----------|--------------|
| **No MRT Support** | ไม่สามารถทำ Deferred Rendering | -0.5 |
| **Single Render Path** | ไม่มีความยืดหยุ่น (Forward+ เท่านั้น) | -0.5 |
| **No Render Graph** | จัดการ dependencies ด้วยตัวเอง | -0.3 |
| **Limited Format Options** | ไม่มี R11G11B10Float, RG16F, etc. | -0.2 |

**รวมช่องว่าง**: 1.5 คะแนน

---

## 🎯 เป้าหมายการพัฒนา

### Phase A: MRT (Multiple Render Targets) Support
**ระยะเวลา**: สัปดาห์ 1
**คะแนนที่เพิ่ม**: +0.5

### Phase B: Runtime Render Path Switching
**ระยะเวลา**: สัปดาห์ 2
**คะแนนที่เพิ่ม**: +0.5

### Phase C: Render Graph System
**ระยะเวลา**: สัปดาห์ 3
**คะแนนที่เพิ่ม**: +0.3

### Phase D: Format Flexibility & Optimization
**ระยะเวลา**: สัปดาห์ 4
**คะแนนที่เพิ่ม**: +0.2

**รวม**: 8.5 + 1.5 = **10/10** 🎉

---

## 📅 Phase A: MRT (Multiple Render Targets) Support

**เป้าหมาย**: รองรับการ render ไปยังหลาย texture พร้อมกัน
**ประโยชน์**: เตรียมพร้อมสำหรับ Deferred Rendering, G-Buffer

### Task A.1: Extend RenderModule for MRT

```rust
// ไฟล์: render/src/lib.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTargetFormat {
    /// Standard HDR (64-bit/pixel)
    Rgba16Float,
    /// Compact HDR, no alpha (32-bit/pixel, saves 50% bandwidth)
    R11G11B10Float,
    /// Standard LDR sRGB
    Rgba8UnormSrgb,
    /// Normal storage (signed)
    Rgba16Snorm,
    /// High precision
    Rgba32Float,
}

impl RenderTargetFormat {
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            Self::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            Self::R11G11B10Float => wgpu::TextureFormat::Rg11b10Float,
            Self::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Rgba16Snorm => wgpu::TextureFormat::Rgba16Snorm,
            Self::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            Self::Rgba16Float => 8,
            Self::R11G11B10Float => 4,
            Self::Rgba8UnormSrgb => 4,
            Self::Rgba16Snorm => 8,
            Self::Rgba32Float => 16,
        }
    }
}

pub struct RenderTarget {
    pub name: String,
    pub format: RenderTargetFormat,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        width: u32,
        height: u32,
        format: RenderTargetFormat,
    ) -> Self {
        let wgpu_format = format.to_wgpu();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("RT_{}", name)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                 | wgpu::TextureUsages::TEXTURE_BINDING
                 | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            name: name.to_string(),
            format,
            texture,
            view,
            width,
            height,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        *self = Self::new(device, &self.name, width, height, self.format);
    }
}

pub struct RenderTargetSet {
    pub targets: Vec<RenderTarget>,
    pub depth_target: Option<RenderTarget>,
}

impl RenderTargetSet {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            depth_target: None,
        }
    }

    pub fn add_color_target(&mut self, target: RenderTarget) {
        self.targets.push(target);
    }

    pub fn set_depth_target(&mut self, target: RenderTarget) {
        self.depth_target = Some(target);
    }

    pub fn get_color_attachments(&self) -> Vec<Option<wgpu::RenderPassColorAttachment>> {
        self.targets
            .iter()
            .map(|target| {
                Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })
            })
            .collect()
    }

    pub fn get_depth_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.depth_target.as_ref().map(|target| {
            wgpu::RenderPassDepthStencilAttachment {
                view: &target.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0), // Reverse-Z
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }
        })
    }

    pub fn resize_all(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        for target in &mut self.targets {
            target.resize(device, width, height);
        }

        if let Some(depth) = &mut self.depth_target {
            depth.resize(device, width, height);
        }
    }
}
```

### Task A.2: Update RenderModule

```rust
// ไฟล์: render/src/lib.rs (ต่อ)

pub struct RenderModule {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    // Render targets
    pub render_targets: RenderTargetSet,
    pub hdr_target: Option<RenderTarget>,  // Main HDR color
    pub gbuffer: Option<GBuffer>,          // For deferred rendering

    // Pipelines
    pub render_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,

    // Depth (existing)
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub scene_depth_texture: wgpu::Texture,
    pub scene_depth_view: wgpu::TextureView,

    // Renderers (existing)
    pub texture_manager: TextureManager,
    pub sprite_renderer: SpriteRenderer,
    pub tilemap_renderer: TilemapRenderer,
    pub batch_renderer: BatchRenderer,
    pub mesh_renderer: MeshRenderer,
    pub cluster_renderer: ClusterRenderer,
    pub camera_binding: CameraBinding,
    pub light_binding: LightBinding,
}

impl RenderModule {
    pub fn create_hdr_target(&mut self) {
        let width = self.size.width;
        let height = self.size.height;

        // Option 1: Rgba16Float (full HDR with alpha)
        // Option 2: R11G11B10Float (compact, 50% bandwidth savings, no alpha)
        let hdr_target = RenderTarget::new(
            &self.device,
            "HDR_Main",
            width,
            height,
            RenderTargetFormat::Rgba16Float, // Can switch to R11G11B10Float later
        );

        self.hdr_target = Some(hdr_target);
    }

    pub fn get_main_color_target(&self) -> &wgpu::TextureView {
        if let Some(hdr) = &self.hdr_target {
            &hdr.view
        } else {
            // Fallback to direct swapchain rendering
            panic!("HDR target not initialized");
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Resize all render targets
            self.render_targets.resize_all(&self.device, new_size.width, new_size.height);

            if let Some(hdr) = &mut self.hdr_target {
                hdr.resize(&self.device, new_size.width, new_size.height);
            }

            if let Some(gbuffer) = &mut self.gbuffer {
                gbuffer.resize(&self.device, new_size.width, new_size.height);
            }

            // Resize depth textures (existing code)
            // ...
        }
    }
}
```

### Task A.3: G-Buffer Definition (Optional, for Deferred)

```rust
// ไฟล์: render/src/gbuffer.rs (ไฟล์ใหม่)

use super::{RenderTarget, RenderTargetFormat};

/// G-Buffer for Deferred Rendering
///
/// Layout:
/// - RT0: Albedo (RGB) + Metallic (A)  [Rgba8UnormSrgb, 4 bytes/pixel]
/// - RT1: Normal (RGB) + Roughness (A) [Rgba16Snorm, 8 bytes/pixel]
/// - RT2: Emissive (RGB) + AO (A)      [Rgba16Float, 8 bytes/pixel]
///
/// Total: 20 bytes/pixel vs 8 bytes (forward HDR)
/// Trade-off: More memory, but decouple geometry and lighting
pub struct GBuffer {
    pub albedo_metallic: RenderTarget,
    pub normal_roughness: RenderTarget,
    pub emissive_ao: RenderTarget,
    pub depth: RenderTarget,
}

impl GBuffer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        Self {
            albedo_metallic: RenderTarget::new(
                device,
                "GBuffer_AlbedoMetallic",
                width,
                height,
                RenderTargetFormat::Rgba8UnormSrgb,
            ),
            normal_roughness: RenderTarget::new(
                device,
                "GBuffer_NormalRoughness",
                width,
                height,
                RenderTargetFormat::Rgba16Snorm,
            ),
            emissive_ao: RenderTarget::new(
                device,
                "GBuffer_EmissiveAO",
                width,
                height,
                RenderTargetFormat::Rgba16Float,
            ),
            depth: RenderTarget::new(
                device,
                "GBuffer_Depth",
                width,
                height,
                RenderTargetFormat::Rgba16Float, // Dummy, will use actual depth texture
            ),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.albedo_metallic.resize(device, width, height);
        self.normal_roughness.resize(device, width, height);
        self.emissive_ao.resize(device, width, height);
        self.depth.resize(device, width, height);
    }

    pub fn get_color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment>; 3] {
        [
            Some(wgpu::RenderPassColorAttachment {
                view: &self.albedo_metallic.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal_roughness.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.5,
                        g: 0.5,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.emissive_ao.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ]
    }
}
```

### Checklist Phase A

- [ ] สร้าง `RenderTargetFormat` enum
- [ ] สร้าง `RenderTarget` struct พร้อม resize
- [ ] สร้าง `RenderTargetSet` สำหรับจัดการ MRT
- [ ] อัปเดต `RenderModule` ให้รองรับ `render_targets`
- [ ] สร้าง `create_hdr_target()` method
- [ ] ทดสอบ render ไปยัง HDR target
- [ ] (Optional) สร้าง `GBuffer` struct สำหรับ deferred
- [ ] อัปเดต `resize()` ให้ resize ทุก targets

**ผลลัพธ์**: รองรับ MRT, พร้อมสำหรับ Deferred Rendering

---

## 🔄 Phase B: Runtime Render Path Switching

**เป้าหมาย**: สามารถสลับระหว่าง Forward+, Forward, และ Deferred ได้ runtime
**ประโยชน์**: ยืดหยุ่น, ปรับให้เหมาะกับ hardware แต่ละเครื่อง

### Task B.1: Render Path Enumeration

```rust
// ไฟล์: render/src/render_path.rs (ไฟล์ใหม่)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPath {
    /// Clustered Forward+ (current implementation)
    /// - Best for: Many dynamic lights
    /// - Memory: Low (single HDR target)
    /// - Performance: Excellent on modern GPUs
    ForwardPlus,

    /// Standard Forward Rendering
    /// - Best for: Simple scenes, <8 lights
    /// - Memory: Lowest
    /// - Performance: Best for low-end devices
    Forward,

    /// Deferred Rendering
    /// - Best for: Many lights, complex materials
    /// - Memory: High (G-Buffer)
    /// - Performance: Good on desktop, OK on high-end mobile
    Deferred,
}

impl RenderPath {
    pub fn supports_platform(&self, platform: Platform) -> bool {
        match (self, platform) {
            (RenderPath::ForwardPlus, _) => true,
            (RenderPath::Forward, _) => true,
            (RenderPath::Deferred, Platform::Desktop) => true,
            (RenderPath::Deferred, Platform::Mobile) => {
                // Only on high-end mobile
                cfg!(feature = "mobile-deferred")
            },
        }
    }

    pub fn memory_cost_mb(&self, width: u32, height: u32) -> f32 {
        let pixels = (width * height) as f32;

        match self {
            RenderPath::Forward => {
                // No offscreen targets
                0.0
            },
            RenderPath::ForwardPlus => {
                // HDR target (8 bytes/pixel)
                pixels * 8.0 / 1_048_576.0
            },
            RenderPath::Deferred => {
                // G-Buffer (20 bytes/pixel) + HDR (8 bytes/pixel)
                pixels * 28.0 / 1_048_576.0
            },
        }
    }

    pub fn recommended_for_device(light_count: u32, is_mobile: bool) -> Self {
        if is_mobile {
            if light_count <= 4 {
                RenderPath::Forward
            } else {
                RenderPath::ForwardPlus
            }
        } else {
            // Desktop
            if light_count <= 8 {
                RenderPath::Forward
            } else if light_count <= 100 {
                RenderPath::ForwardPlus
            } else {
                RenderPath::Deferred
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Mobile,
}
```

### Task B.2: Render Path Manager

```rust
// ไฟล์: render/src/render_path.rs (ต่อ)

pub struct RenderPathManager {
    current_path: RenderPath,
    available_paths: Vec<RenderPath>,
}

impl RenderPathManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let available_paths = Self::detect_available_paths(device);
        let current_path = available_paths[0]; // Default to first available

        Self {
            current_path,
            available_paths,
        }
    }

    fn detect_available_paths(device: &wgpu::Device) -> Vec<RenderPath> {
        let mut paths = vec![
            RenderPath::Forward,
            RenderPath::ForwardPlus,
        ];

        // Check if deferred is supported
        let features = device.features();
        if features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
            paths.push(RenderPath::Deferred);
        }

        paths
    }

    pub fn switch_path(&mut self, new_path: RenderPath) -> Result<(), String> {
        if !self.available_paths.contains(&new_path) {
            return Err(format!("Render path {:?} not supported on this device", new_path));
        }

        println!("Switching render path: {:?} -> {:?}", self.current_path, new_path);
        self.current_path = new_path;

        Ok(())
    }

    pub fn get_current_path(&self) -> RenderPath {
        self.current_path
    }

    pub fn is_available(&self, path: RenderPath) -> bool {
        self.available_paths.contains(&path)
    }
}
```

### Task B.3: Update RenderModule with Path Manager

```rust
// ไฟล์: render/src/lib.rs (ต่อ)

use render_path::{RenderPath, RenderPathManager};

pub struct RenderModule {
    // ... existing fields ...

    pub render_path_manager: RenderPathManager,
}

impl RenderModule {
    pub async fn new(window: &Window) -> Result<Self> {
        // ... existing initialization ...

        let render_path_manager = RenderPathManager::new(&device);

        // Initialize based on current path
        let current_path = render_path_manager.get_current_path();
        let (hdr_target, gbuffer) = match current_path {
            RenderPath::Forward => (None, None),
            RenderPath::ForwardPlus => {
                let hdr = RenderTarget::new(
                    &device,
                    "HDR_Main",
                    size.width,
                    size.height,
                    RenderTargetFormat::Rgba16Float,
                );
                (Some(hdr), None)
            },
            RenderPath::Deferred => {
                let hdr = RenderTarget::new(
                    &device,
                    "HDR_Main",
                    size.width,
                    size.height,
                    RenderTargetFormat::Rgba16Float,
                );
                let gbuffer = GBuffer::new(&device, size.width, size.height);
                (Some(hdr), Some(gbuffer))
            },
        };

        Ok(Self {
            // ... existing fields ...
            render_path_manager,
            hdr_target,
            gbuffer,
            // ...
        })
    }

    pub fn switch_render_path(&mut self, new_path: RenderPath) -> Result<(), String> {
        self.render_path_manager.switch_path(new_path)?;

        // Recreate render targets based on new path
        let width = self.size.width;
        let height = self.size.height;

        match new_path {
            RenderPath::Forward => {
                self.hdr_target = None;
                self.gbuffer = None;
            },
            RenderPath::ForwardPlus => {
                self.hdr_target = Some(RenderTarget::new(
                    &self.device,
                    "HDR_Main",
                    width,
                    height,
                    RenderTargetFormat::Rgba16Float,
                ));
                self.gbuffer = None;
            },
            RenderPath::Deferred => {
                self.hdr_target = Some(RenderTarget::new(
                    &self.device,
                    "HDR_Main",
                    width,
                    height,
                    RenderTargetFormat::Rgba16Float,
                ));
                self.gbuffer = Some(GBuffer::new(&self.device, width, height));
            },
        }

        Ok(())
    }

    pub fn render_frame(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
        match self.render_path_manager.get_current_path() {
            RenderPath::Forward => self.render_forward(encoder, surface_view),
            RenderPath::ForwardPlus => self.render_forward_plus(encoder, surface_view),
            RenderPath::Deferred => self.render_deferred(encoder, surface_view),
        }
    }

    fn render_forward(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
        // Direct rendering to swapchain
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            // ...
        });

        // Render geometry with simple lighting
        // ...
    }

    fn render_forward_plus(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
        // Current implementation
        // 1. Render to HDR target
        // 2. Clustered lighting
        // 3. Post-process to swapchain
    }

    fn render_deferred(&mut self, encoder: &mut wgpu::CommandEncoder, surface_view: &wgpu::TextureView) {
        if let Some(gbuffer) = &self.gbuffer {
            // 1. Geometry pass -> G-Buffer
            {
                let color_attachments = gbuffer.get_color_attachments();
                let mut geometry_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Deferred Geometry Pass"),
                    color_attachments: &color_attachments,
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    // ...
                });

                // Render all geometry (writes to G-Buffer)
                // ...
            }

            // 2. Lighting pass -> HDR target
            if let Some(hdr) = &self.hdr_target {
                let mut lighting_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Deferred Lighting Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &hdr.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    // ...
                });

                // Fullscreen quad with G-Buffer sampling
                // Calculate lighting for all lights
                // ...
            }

            // 3. Post-process to swapchain
            // ...
        }
    }
}
```

### Checklist Phase B

- [ ] สร้าง `RenderPath` enum
- [ ] สร้าง `Platform` enum
- [ ] Implement `supports_platform()`, `memory_cost_mb()`, `recommended_for_device()`
- [ ] สร้าง `RenderPathManager`
- [ ] Implement `detect_available_paths()`
- [ ] เพิ่ม `render_path_manager` ใน `RenderModule`
- [ ] Implement `switch_render_path()`
- [ ] แยก `render_frame()` เป็น `render_forward()`, `render_forward_plus()`, `render_deferred()`
- [ ] ทดสอบการสลับระหว่าง render paths
- [ ] สร้าง UI controls สำหรับสลับ render path

**ผลลัพธ์**: สามารถสลับ render path ได้แบบ real-time

---

## 📊 Phase C: Render Graph System

**เป้าหมาย**: ระบบจัดการ render passes อัตโนมัติ, optimize barriers
**ประโยชน์**: ลด boilerplate code, จัดการ dependencies เอง, optimize GPU

### Task C.1: Render Graph Core

```rust
// ไฟล์: render/src/render_graph.rs (ไฟล์ใหม่)

use std::collections::HashMap;

pub type PassId = u32;
pub type ResourceId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Texture,
    Buffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    ReadWrite,
}

pub struct ResourceAccess {
    pub resource_id: ResourceId,
    pub resource_type: ResourceType,
    pub access_type: AccessType,
}

pub struct RenderPass {
    pub id: PassId,
    pub name: String,
    pub inputs: Vec<ResourceAccess>,
    pub outputs: Vec<ResourceAccess>,
    pub execute: Box<dyn Fn(&mut wgpu::CommandEncoder)>,
}

pub struct RenderGraph {
    passes: HashMap<PassId, RenderPass>,
    resources: HashMap<ResourceId, String>, // resource_id -> name
    next_pass_id: PassId,
    next_resource_id: ResourceId,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            resources: HashMap::new(),
            next_pass_id: 0,
            next_resource_id: 0,
        }
    }

    pub fn add_pass(
        &mut self,
        name: &str,
        inputs: Vec<ResourceAccess>,
        outputs: Vec<ResourceAccess>,
        execute: Box<dyn Fn(&mut wgpu::CommandEncoder)>,
    ) -> PassId {
        let pass_id = self.next_pass_id;
        self.next_pass_id += 1;

        let pass = RenderPass {
            id: pass_id,
            name: name.to_string(),
            inputs,
            outputs,
            execute,
        };

        self.passes.insert(pass_id, pass);

        pass_id
    }

    pub fn register_resource(&mut self, name: &str, resource_type: ResourceType) -> ResourceId {
        let resource_id = self.next_resource_id;
        self.next_resource_id += 1;

        self.resources.insert(resource_id, name.to_string());

        resource_id
    }

    pub fn compile(&self) -> Vec<PassId> {
        // Topological sort based on dependencies
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit(
            pass_id: PassId,
            graph: &RenderGraph,
            visited: &mut std::collections::HashSet<PassId>,
            sorted: &mut Vec<PassId>,
        ) {
            if visited.contains(&pass_id) {
                return;
            }

            visited.insert(pass_id);

            // Visit dependencies first
            let pass = &graph.passes[&pass_id];
            for input in &pass.inputs {
                // Find which pass writes to this resource
                for (other_id, other_pass) in &graph.passes {
                    if *other_id == pass_id {
                        continue;
                    }

                    for output in &other_pass.outputs {
                        if output.resource_id == input.resource_id {
                            visit(*other_id, graph, visited, sorted);
                        }
                    }
                }
            }

            sorted.push(pass_id);
        }

        for pass_id in self.passes.keys() {
            visit(*pass_id, self, &mut visited, &mut sorted);
        }

        sorted
    }

    pub fn execute(&self, encoder: &mut wgpu::CommandEncoder) {
        let sorted_passes = self.compile();

        println!("=== Render Graph Execution Order ===");
        for pass_id in &sorted_passes {
            let pass = &self.passes[pass_id];
            println!("  {}: {}", pass.id, pass.name);
        }

        // Execute passes in order
        for pass_id in sorted_passes {
            let pass = &self.passes[&pass_id];
            (pass.execute)(encoder);
        }
    }
}
```

### Task C.2: Example Usage

```rust
// ไฟล์: render/src/lib.rs (example)

impl RenderModule {
    pub fn setup_render_graph(&mut self) -> RenderGraph {
        let mut graph = RenderGraph::new();

        // Register resources
        let depth_buffer = graph.register_resource("Depth Buffer", ResourceType::Texture);
        let shadow_map = graph.register_resource("Shadow Map", ResourceType::Texture);
        let hdr_buffer = graph.register_resource("HDR Buffer", ResourceType::Texture);
        let bloom_buffer = graph.register_resource("Bloom Buffer", ResourceType::Texture);
        let final_output = graph.register_resource("Final Output", ResourceType::Texture);

        // Pass 1: Shadow Map Generation
        graph.add_pass(
            "Shadow Pass",
            vec![], // No inputs
            vec![ResourceAccess {
                resource_id: shadow_map,
                resource_type: ResourceType::Texture,
                access_type: AccessType::Write,
            }],
            Box::new(|encoder| {
                // Shadow rendering code
            }),
        );

        // Pass 2: Main Scene Rendering
        graph.add_pass(
            "Scene Pass",
            vec![
                ResourceAccess {
                    resource_id: shadow_map,
                    resource_type: ResourceType::Texture,
                    access_type: AccessType::Read,
                },
            ],
            vec![
                ResourceAccess {
                    resource_id: hdr_buffer,
                    resource_type: ResourceType::Texture,
                    access_type: AccessType::Write,
                },
                ResourceAccess {
                    resource_id: depth_buffer,
                    resource_type: ResourceType::Texture,
                    access_type: AccessType::Write,
                },
            ],
            Box::new(|encoder| {
                // Main scene rendering
            }),
        );

        // Pass 3: Bloom Generation
        graph.add_pass(
            "Bloom Pass",
            vec![ResourceAccess {
                resource_id: hdr_buffer,
                resource_type: ResourceType::Texture,
                access_type: AccessType::Read,
            }],
            vec![ResourceAccess {
                resource_id: bloom_buffer,
                resource_type: ResourceType::Texture,
                access_type: AccessType::Write,
            }],
            Box::new(|encoder| {
                // Bloom downsample/upsample
            }),
        );

        // Pass 4: Post-Processing
        graph.add_pass(
            "Post-Process Pass",
            vec![
                ResourceAccess {
                    resource_id: hdr_buffer,
                    resource_type: ResourceType::Texture,
                    access_type: AccessType::Read,
                },
                ResourceAccess {
                    resource_id: bloom_buffer,
                    resource_type: ResourceType::Texture,
                    access_type: AccessType::Read,
                },
            ],
            vec![ResourceAccess {
                resource_id: final_output,
                resource_type: ResourceType::Texture,
                access_type: AccessType::Write,
            }],
            Box::new(|encoder| {
                // Tonemapping, color grading, etc.
            }),
        );

        graph
    }

    pub fn render_with_graph(&mut self) {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let graph = self.setup_render_graph();
        graph.execute(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
```

### Checklist Phase C

- [ ] สร้าง `RenderGraph` struct
- [ ] Implement `add_pass()`, `register_resource()`
- [ ] Implement topological sort สำหรับ dependency resolution
- [ ] Implement `compile()` และ `execute()`
- [ ] สร้าง example render graph setup
- [ ] ทดสอบ automatic pass ordering
- [ ] เพิ่ม resource aliasing (reuse memory)
- [ ] เพิ่ม automatic barrier insertion
- [ ] Profile และวัด overhead ของ render graph

**ผลลัพธ์**: จัดการ render passes อัตโนมัติ, โค้ดสะอาดขึ้น

---

## 🎨 Phase D: Format Flexibility & Optimization

**เป้าหมาย**: รองรับ format ที่หลากหลาย, optimize bandwidth
**ประโยชน์**: ลดการใช้ memory, เพิ่มประสิทธิภาพบนมือถือ

### Task D.1: R11G11B10Float Support

```rust
// ไฟล์: render/src/render_target.rs (เพิ่มใน RenderTargetFormat)

impl RenderTargetFormat {
    pub fn is_mobile_optimized(&self) -> bool {
        matches!(self, Self::R11G11B10Float | Self::Rgba8UnormSrgb)
    }

    pub fn supports_alpha(&self) -> bool {
        !matches!(self, Self::R11G11B10Float)
    }

    pub fn bandwidth_savings_vs_rgba16f(&self) -> f32 {
        let rgba16f_bytes = 8.0;
        let self_bytes = self.bytes_per_pixel() as f32;

        (rgba16f_bytes - self_bytes) / rgba16f_bytes * 100.0
    }
}
```

### Task D.2: Automatic Format Selection

```rust
// ไฟล์: render/src/render_module.rs

impl RenderModule {
    pub fn select_optimal_hdr_format(&self) -> RenderTargetFormat {
        let is_mobile = cfg!(target_os = "android") || cfg!(target_os = "ios");
        let needs_alpha = false; // Check if transparency is needed

        if is_mobile && !needs_alpha {
            // 50% bandwidth savings!
            RenderTargetFormat::R11G11B10Float
        } else {
            RenderTargetFormat::Rgba16Float
        }
    }

    pub fn create_optimized_hdr_target(&mut self) {
        let format = self.select_optimal_hdr_format();

        println!("Selected HDR format: {:?}", format);
        println!("Bandwidth savings: {:.1}%", format.bandwidth_savings_vs_rgba16f());

        self.hdr_target = Some(RenderTarget::new(
            &self.device,
            "HDR_Main",
            self.size.width,
            self.size.height,
            format,
        ));
    }
}
```

### Task D.3: Half-Precision (f16) Shader Support

```rust
// ไฟล์: render/src/shader_features.rs (ไฟล์ใหม่)

pub struct ShaderFeatures {
    pub use_f16_precision: bool,
    pub use_mediump: bool,
}

impl ShaderFeatures {
    pub fn for_mobile() -> Self {
        Self {
            use_f16_precision: true,
            use_mediump: true,
        }
    }

    pub fn for_desktop() -> Self {
        Self {
            use_f16_precision: false,
            use_mediump: false,
        }
    }

    pub fn generate_shader_defines(&self) -> String {
        let mut defines = String::new();

        if self.use_f16_precision {
            defines.push_str("#define USE_F16 1\n");
        }

        if self.use_mediump {
            defines.push_str("#define USE_MEDIUMP 1\n");
        }

        defines
    }
}
```

```wgsl
// ไฟล์: render/assets/shaders/post_process.wgsl (example)

// Conditional precision based on platform
#ifdef USE_F16
    alias PrecisionFloat = f16;
#else
    alias PrecisionFloat = f32;
#endif

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Use PrecisionFloat for intermediate calculations
    var color: vec3<PrecisionFloat> = textureSample(t_hdr, s_linear, in.uv).rgb;

    // ACES tonemapping with lower precision (invisible difference)
    color = aces_tone_map(color);

    // Final output is always f32
    return vec4<f32>(color, 1.0);
}
```

### Task D.4: Memory Budget System

```rust
// ไฟล์: render/src/memory_budget.rs (ไฟล์ใหม่)

pub struct MemoryBudget {
    pub total_budget_mb: f32,
    pub texture_budget_mb: f32,
    pub buffer_budget_mb: f32,
    pub current_texture_usage_mb: f32,
    pub current_buffer_usage_mb: f32,
}

impl MemoryBudget {
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::Desktop => Self {
                total_budget_mb: 2048.0,
                texture_budget_mb: 1536.0,
                buffer_budget_mb: 512.0,
                current_texture_usage_mb: 0.0,
                current_buffer_usage_mb: 0.0,
            },
            Platform::Mobile => Self {
                total_budget_mb: 512.0,
                texture_budget_mb: 384.0,
                buffer_budget_mb: 128.0,
                current_texture_usage_mb: 0.0,
                current_buffer_usage_mb: 0.0,
            },
        }
    }

    pub fn allocate_texture(&mut self, size_mb: f32) -> Result<(), String> {
        if self.current_texture_usage_mb + size_mb > self.texture_budget_mb {
            return Err(format!(
                "Texture memory budget exceeded: {:.2} MB / {:.2} MB",
                self.current_texture_usage_mb + size_mb,
                self.texture_budget_mb
            ));
        }

        self.current_texture_usage_mb += size_mb;
        Ok(())
    }

    pub fn free_texture(&mut self, size_mb: f32) {
        self.current_texture_usage_mb -= size_mb;
    }

    pub fn get_usage_percentage(&self) -> f32 {
        (self.current_texture_usage_mb + self.current_buffer_usage_mb)
            / self.total_budget_mb
            * 100.0
    }

    pub fn print_status(&self) {
        println!("=== Memory Budget Status ===");
        println!("Textures: {:.2} MB / {:.2} MB ({:.1}%)",
                 self.current_texture_usage_mb,
                 self.texture_budget_mb,
                 self.current_texture_usage_mb / self.texture_budget_mb * 100.0);
        println!("Buffers: {:.2} MB / {:.2} MB ({:.1}%)",
                 self.current_buffer_usage_mb,
                 self.buffer_budget_mb,
                 self.current_buffer_usage_mb / self.buffer_budget_mb * 100.0);
        println!("Total: {:.1}%", self.get_usage_percentage());
    }
}
```

### Checklist Phase D

- [ ] เพิ่ม `is_mobile_optimized()`, `supports_alpha()` ใน `RenderTargetFormat`
- [ ] Implement `bandwidth_savings_vs_rgba16f()`
- [ ] สร้าง `select_optimal_hdr_format()`
- [ ] ทดสอบ R11G11B10Float vs Rgba16Float (quality comparison)
- [ ] สร้าง `ShaderFeatures` สำหรับ conditional compilation
- [ ] เพิ่ม f16 precision support ใน shaders
- [ ] สร้าง `MemoryBudget` system
- [ ] Track texture/buffer allocations
- [ ] เพิ่ม memory usage display ใน UI
- [ ] Profile memory usage และ bandwidth

**ผลลัพธ์**: Optimize memory และ bandwidth, เหมาะกับมือถือมากขึ้น

---

## 📈 สรุป: เส้นทางสู่ 10/10

### Timeline

```
Week 1: MRT Support
  ├─ RenderTarget, RenderTargetSet
  ├─ GBuffer (optional)
  └─ Testing

Week 2: Render Path Switching
  ├─ RenderPath enum
  ├─ RenderPathManager
  ├─ Forward/Forward+/Deferred separation
  └─ Runtime switching

Week 3: Render Graph
  ├─ RenderGraph core
  ├─ Dependency resolution
  ├─ Automatic execution
  └─ Example setup

Week 4: Format Optimization
  ├─ R11G11B10Float support
  ├─ Automatic format selection
  ├─ f16 shader precision
  └─ Memory budget system
```

### คะแนนที่เพิ่มขึ้น

| Phase | Feature | คะแนนเพิ่ม | รวม |
|-------|---------|-----------|-----|
| Start | - | - | 8.5/10 |
| A | MRT Support | +0.5 | 9.0/10 |
| B | Render Path Switching | +0.5 | 9.5/10 |
| C | Render Graph | +0.3 | 9.8/10 |
| D | Format Optimization | +0.2 | **10/10** 🎉 |

### จุดเด่นหลังอัปเกรด

✅ **Multiple Render Targets** - รองรับ Deferred Rendering
✅ **Runtime Path Switching** - ยืดหยุ่น, ปรับตาม hardware
✅ **Render Graph System** - จัดการ passes อัตโนมัติ
✅ **Format Optimization** - ประหยัด bandwidth 50% (R11G11B10Float)
✅ **Memory Budget** - ควบคุมการใช้ memory
✅ **f16 Precision** - เพิ่มประสิทธิภาพบนมือถือ

### เปรียบเทียบกับ Competition

| Feature | Rust 2D (หลัง) | Unreal 5 | Unity URP | Godot 4 |
|---------|---------------|----------|-----------|---------|
| MRT Support | ✅ | ✅ | ✅ | ✅ |
| Runtime Path Switch | ✅ | ✅ | ⚠️ Limited | ⚠️ Limited |
| Render Graph | ✅ | ✅ Advanced | ❌ | ⚠️ Basic |
| Format Options | ✅ 5+ formats | ✅ 10+ | ✅ 8+ | ✅ 6+ |
| Memory Budget | ✅ | ✅ | ⚠️ Manual | ❌ |

**ผลลัพธ์**: สถาปัตยกรรมระดับ **World-Class** เทียบเท่า Unreal Engine 5! 🚀

---

**เริ่มต้นจาก Phase A: MRT Support ได้เลย!** 💪
