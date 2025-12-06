# Modern AAA Mobile-First Renderer - Design Document

## Overview

This document describes the technical design for the XS Game Engine's modern rendering system. The renderer is built on wgpu (WebGPU) and provides AAA-quality graphics with mobile-first optimization, supporting advanced features including material editing, custom shaders, post-processing, VFX, fluid simulation, and destruction systems.

### Design Goals

1. **AAA Quality**: Desktop-class visual fidelity with PBR, advanced lighting, and post-processing
2. **Mobile-First**: 60 FPS on mid-range mobile devices with thermal management
3. **Flexibility**: Support both forward and deferred rendering pipelines
4. **Artist-Friendly**: Visual material editor with node-based shader authoring
5. **Performance**: GPU-accelerated particles, fluid simulation, and destruction
6. **Developer Experience**: Hot reload, profiling tools, and comprehensive debugging

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Game Logic  │  │  ECS Systems │  │  Editor UI   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Renderer Public API                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Material    │  │   Camera     │  │    Light     │          │
│  │   System     │  │   System     │  │   System     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Render Graph Engine                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Render Graph │  │  Pass        │  │  Resource    │          │
│  │  Builder     │  │  Scheduler   │  │  Manager     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Rendering Subsystems                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Pipeline    │  │  Post-       │  │  VFX         │          │
│  │  Manager     │  │  Processing  │  │  System      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Fluid       │  │  Destruction │  │  Shadow      │          │
│  │  Simulation  │  │  System      │  │  System      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                         wgpu Backend                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Device     │  │    Queue     │  │   Surface    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Components and Interfaces

### 1. Render Module (Core)

```rust
pub struct RenderModule {
    // wgpu core
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    
    // Subsystems
    pub pipeline_manager: PipelineManager,
    pub material_system: MaterialSystem,
    pub texture_manager: TextureManager,
    pub render_graph: RenderGraph,
    
    // Rendering systems
    pub forward_renderer: ForwardRenderer,
    pub deferred_renderer: DeferredRenderer,
    pub post_processor: PostProcessor,
    pub particle_system: ParticleSystem,
    pub fluid_simulator: FluidSimulator,
    pub destruction_system: DestructionSystem,
    
    // Performance
    pub adaptive_quality: AdaptiveQuality,
    pub thermal_manager: ThermalManager,
    pub profiler: GpuProfiler,
}
```


### 2. Pipeline Manager

```rust
/// Manages render pipelines and PSO caching
pub struct PipelineManager {
    /// Pipeline cache (keyed by PipelineDescriptor hash)
    cache: HashMap<u64, wgpu::RenderPipeline>,
    
    /// Shader module cache
    shaders: HashMap<String, wgpu::ShaderModule>,
    
    /// Bind group layout cache
    bind_group_layouts: HashMap<u64, wgpu::BindGroupLayout>,
    
    /// Pipeline statistics
    stats: PipelineStats,
}

impl PipelineManager {
    /// Get or create a render pipeline
    pub fn get_or_create_pipeline(
        &mut self,
        device: &wgpu::Device,
        desc: &PipelineDescriptor,
    ) -> &wgpu::RenderPipeline {
        let hash = desc.hash();
        
        self.cache.entry(hash).or_insert_with(|| {
            self.create_pipeline(device, desc)
        })
    }
    
    /// Hot reload a shader
    pub fn reload_shader(
        &mut self,
        device: &wgpu::Device,
        path: &str,
        source: &str,
    ) -> Result<()> {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        
        self.shaders.insert(path.to_string(), module);
        
        // Invalidate dependent pipelines
        self.invalidate_pipelines_using_shader(path);
        
        Ok(())
    }
}

#[derive(Hash)]
pub struct PipelineDescriptor {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_layout: VertexLayout,
    pub blend_mode: BlendMode,
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub topology: PrimitiveTopology,
}
```

### 3. Material System

```rust
/// Material definition
pub struct Material {
    pub name: String,
    pub shader: ShaderRef,
    pub parameters: MaterialParameters,
    pub textures: HashMap<String, TextureRef>,
    pub render_queue: RenderQueue,
    pub blend_mode: BlendMode,
}

/// Material parameters (uniforms)
pub struct MaterialParameters {
    /// Scalar parameters
    pub scalars: HashMap<String, f32>,
    
    /// Vector parameters
    pub vectors: HashMap<String, Vec4>,
    
    /// Color parameters
    pub colors: HashMap<String, Color>,
    
    /// GPU uniform buffer
    uniform_buffer: wgpu::Buffer,
}

impl MaterialParameters {
    /// Set a parameter and update GPU buffer
    pub fn set_scalar(&mut self, name: &str, value: f32, queue: &wgpu::Queue) {
        self.scalars.insert(name.to_string(), value);
        self.update_uniform_buffer(queue);
    }
    
    /// Update GPU uniform buffer
    fn update_uniform_buffer(&self, queue: &wgpu::Queue) {
        let data = self.pack_to_bytes();
        queue.write_buffer(&self.uniform_buffer, 0, &data);
    }
}

/// Material system manages all materials
pub struct MaterialSystem {
    materials: HashMap<MaterialId, Material>,
    default_material: MaterialId,
    material_editor: MaterialEditor,
}
```

### 4. Material Editor (Node-Based)

```rust
/// Visual material editor with node graph
pub struct MaterialEditor {
    /// Node graph
    graph: MaterialGraph,
    
    /// Code generator
    codegen: ShaderCodegen,
    
    /// Preview renderer
    preview: MaterialPreview,
}

/// Material node graph
pub struct MaterialGraph {
    nodes: Vec<MaterialNode>,
    connections: Vec<Connection>,
}

/// Material node types
pub enum MaterialNode {
    // Input nodes
    VertexPosition,
    VertexNormal,
    VertexUV,
    Time,
    
    // Texture nodes
    TextureSample { texture: String },
    
    // Math nodes
    Add { a: NodeInput, b: NodeInput },
    Multiply { a: NodeInput, b: NodeInput },
    Lerp { a: NodeInput, b: NodeInput, t: NodeInput },
    
    // Output nodes
    BaseColor { input: NodeInput },
    Metallic { input: NodeInput },
    Roughness { input: NodeInput },
    Normal { input: NodeInput },
    Emissive { input: NodeInput },
}

/// Shader code generator
pub struct ShaderCodegen {
    /// Generate WGSL code from graph
    pub fn generate(&self, graph: &MaterialGraph) -> Result<String> {
        let mut code = String::new();
        
        // Generate vertex shader
        code.push_str(&self.generate_vertex_shader(graph)?);
        
        // Generate fragment shader
        code.push_str(&self.generate_fragment_shader(graph)?);
        
        Ok(code)
    }
}
```


### 5. Render Graph System

```rust
/// Render graph for automatic pass scheduling
pub struct RenderGraph {
    passes: Vec<RenderPass>,
    resources: HashMap<ResourceId, RenderResource>,
    execution_order: Vec<PassId>,
}

impl RenderGraph {
    /// Add a render pass
    pub fn add_pass(&mut self, pass: RenderPass) -> PassId {
        let id = PassId(self.passes.len());
        self.passes.push(pass);
        id
    }
    
    /// Build execution order (topological sort)
    pub fn build(&mut self) -> Result<()> {
        self.execution_order = self.topological_sort()?;
        self.optimize_resource_usage();
        Ok(())
    }
    
    /// Execute all passes
    pub fn execute(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for pass_id in &self.execution_order {
            let pass = &mut self.passes[pass_id.0];
            
            if pass.enabled {
                pass.execute(device, queue, &self.resources);
            }
        }
    }
}

/// Render pass definition
pub struct RenderPass {
    pub name: String,
    pub enabled: bool,
    pub inputs: Vec<ResourceId>,
    pub outputs: Vec<ResourceId>,
    pub execute_fn: Box<dyn FnMut(&wgpu::Device, &wgpu::Queue, &HashMap<ResourceId, RenderResource>)>,
}

/// Render resource (texture, buffer)
pub enum RenderResource {
    Texture {
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    },
    Buffer {
        buffer: wgpu::Buffer,
        size: u64,
    },
}
```

### 6. Forward Renderer

```rust
/// Forward rendering pipeline
pub struct ForwardRenderer {
    /// Main render pipeline
    pipeline: wgpu::RenderPipeline,
    
    /// Depth texture
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    
    /// Light buffer (for forward+ clustered lighting)
    light_buffer: wgpu::Buffer,
    light_clusters: LightClusters,
    
    /// Draw call batching
    batch_manager: BatchManager,
}

impl ForwardRenderer {
    /// Render scene with forward rendering
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &Camera,
        entities: &[RenderEntity],
    ) {
        // Update light clusters
        self.light_clusters.update(camera, &entities);
        
        // Sort entities (opaque front-to-back, transparent back-to-front)
        let sorted = self.sort_entities(entities, camera);
        
        // Begin render pass
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Forward Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
            ..Default::default()
        });
        
        // Render opaque objects
        self.render_opaque(&mut pass, &sorted.opaque);
        
        // Render transparent objects
        self.render_transparent(&mut pass, &sorted.transparent);
    }
}

/// Light clustering for forward+
pub struct LightClusters {
    /// 3D grid of light indices
    grid: Vec<Vec<u32>>,
    grid_size: (u32, u32, u32),
    
    /// Light index buffer (GPU)
    light_index_buffer: wgpu::Buffer,
}
```

### 7. Deferred Renderer

```rust
/// Deferred rendering pipeline
pub struct DeferredRenderer {
    /// G-buffer textures
    gbuffer: GBuffer,
    
    /// Geometry pass pipeline
    geometry_pipeline: wgpu::RenderPipeline,
    
    /// Lighting pass pipeline
    lighting_pipeline: wgpu::RenderPipeline,
    
    /// Light volume meshes (for light culling)
    light_volumes: LightVolumeRenderer,
}

/// G-Buffer layout
pub struct GBuffer {
    /// Albedo + Metallic (RGBA8)
    pub albedo_metallic: wgpu::Texture,
    
    /// Normal + Roughness (RGBA16F)
    pub normal_roughness: wgpu::Texture,
    
    /// Emissive + AO (RGBA16F)
    pub emissive_ao: wgpu::Texture,
    
    /// Depth (Depth32F)
    pub depth: wgpu::Texture,
    
    /// Views
    pub views: GBufferViews,
}

impl DeferredRenderer {
    /// Render scene with deferred rendering
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &Camera,
        entities: &[RenderEntity],
        lights: &[Light],
    ) {
        // Geometry pass: render to G-buffer
        self.geometry_pass(encoder, camera, entities);
        
        // Lighting pass: compute lighting from G-buffer
        self.lighting_pass(encoder, view, camera, lights);
    }
    
    fn geometry_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        entities: &[RenderEntity],
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Geometry Pass"),
            color_attachments: &[
                Some(self.gbuffer.views.albedo_metallic_attachment()),
                Some(self.gbuffer.views.normal_roughness_attachment()),
                Some(self.gbuffer.views.emissive_ao_attachment()),
            ],
            depth_stencil_attachment: Some(self.gbuffer.views.depth_attachment()),
            ..Default::default()
        });
        
        // Render all entities to G-buffer
        for entity in entities {
            self.render_entity(&mut pass, entity, camera);
        }
    }
}
```


### 8. Post-Processing Stack

```rust
/// Post-processing effect stack
pub struct PostProcessor {
    /// Effect chain
    effects: Vec<Box<dyn PostEffect>>,
    
    /// Ping-pong buffers for multi-pass effects
    buffers: [wgpu::Texture; 2],
    current_buffer: usize,
    
    /// Effect-specific resources
    bloom: BloomEffect,
    dof: DepthOfFieldEffect,
    color_grading: ColorGradingEffect,
    tonemapping: TonemappingEffect,
}

impl PostProcessor {
    /// Apply all enabled effects
    pub fn process(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let mut current_input = input;
        
        for effect in &mut self.effects {
            if effect.is_enabled() {
                let output_view = self.get_next_buffer();
                effect.apply(encoder, current_input, output_view);
                current_input = output_view;
            }
        }
        
        // Copy final result to output
        self.blit(encoder, current_input, output);
    }
}

/// Post-processing effect trait
pub trait PostEffect: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn apply(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    );
}

/// Bloom effect
pub struct BloomEffect {
    enabled: bool,
    threshold: f32,
    intensity: f32,
    
    /// Downsampled mip chain
    mip_chain: Vec<wgpu::Texture>,
    
    /// Blur pipeline
    blur_pipeline: wgpu::RenderPipeline,
    
    /// Composite pipeline
    composite_pipeline: wgpu::RenderPipeline,
}

impl PostEffect for BloomEffect {
    fn apply(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        // 1. Extract bright pixels
        self.extract_bright(encoder, input);
        
        // 2. Downsample and blur
        self.downsample_and_blur(encoder);
        
        // 3. Upsample and combine
        self.upsample_and_combine(encoder);
        
        // 4. Composite with original
        self.composite(encoder, input, output);
    }
}

/// Depth of Field effect
pub struct DepthOfFieldEffect {
    enabled: bool,
    focus_distance: f32,
    focus_range: f32,
    bokeh_shape: BokehShape,
    
    /// CoC (Circle of Confusion) texture
    coc_texture: wgpu::Texture,
    
    /// Blur pipeline
    blur_pipeline: wgpu::RenderPipeline,
}

/// Color grading with LUT
pub struct ColorGradingEffect {
    enabled: bool,
    
    /// 3D LUT texture (32x32x32)
    lut_texture: wgpu::Texture,
    
    /// Color grading parameters
    exposure: f32,
    contrast: f32,
    saturation: f32,
    temperature: f32,
    tint: f32,
    
    /// Apply pipeline
    pipeline: wgpu::RenderPipeline,
}
```

### 9. Particle System (GPU-Accelerated)

```rust
/// GPU particle system
pub struct ParticleSystem {
    /// Particle buffers (double-buffered for compute)
    particle_buffers: [wgpu::Buffer; 2],
    current_buffer: usize,
    
    /// Particle count
    max_particles: u32,
    active_particles: u32,
    
    /// Compute pipeline for particle update
    update_pipeline: wgpu::ComputePipeline,
    
    /// Render pipeline for particle rendering
    render_pipeline: wgpu::RenderPipeline,
    
    /// Emitters
    emitters: Vec<ParticleEmitter>,
    
    /// Particle texture atlas
    texture_atlas: wgpu::Texture,
}

/// Particle data (GPU layout)
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 3],
    pub lifetime: f32,
    
    pub velocity: [f32; 3],
    pub age: f32,
    
    pub color: [f32; 4],
    
    pub size: f32,
    pub rotation: f32,
    pub texture_index: u32,
    pub _padding: u32,
}

impl ParticleSystem {
    /// Update particles on GPU
    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder, dt: f32) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Update"),
            ..Default::default()
        });
        
        pass.set_pipeline(&self.update_pipeline);
        pass.set_bind_group(0, &self.get_bind_group(), &[]);
        
        // Dispatch compute shader (one thread per particle)
        let workgroups = (self.active_particles + 255) / 256;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
    
    /// Render particles with instancing
    pub fn render(
        &mut self,
        pass: &mut wgpu::RenderPass,
        camera: &Camera,
    ) {
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.get_bind_group(), &[]);
        pass.set_vertex_buffer(0, self.particle_buffers[self.current_buffer].slice(..));
        
        // Draw instanced quads (one per particle)
        pass.draw(0..6, 0..self.active_particles);
    }
}

/// Particle emitter
pub struct ParticleEmitter {
    pub position: Vec3,
    pub emission_rate: f32,
    pub lifetime: f32,
    pub initial_velocity: Vec3,
    pub velocity_randomness: f32,
    pub size: f32,
    pub color: Color,
}
```


### 10. SPH Fluid Simulation

```rust
/// SPH (Smoothed Particle Hydrodynamics) fluid simulator
pub struct FluidSimulator {
    /// Particle data
    particles: wgpu::Buffer,
    particle_count: u32,
    
    /// Spatial hash grid for neighbor search
    spatial_grid: SpatialHashGrid,
    
    /// Compute pipelines
    density_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
    integrate_pipeline: wgpu::ComputePipeline,
    
    /// Simulation parameters
    params: FluidParams,
    
    /// Render pipeline
    render_pipeline: wgpu::RenderPipeline,
}

/// Fluid particle (GPU layout)
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FluidParticle {
    pub position: [f32; 3],
    pub density: f32,
    
    pub velocity: [f32; 3],
    pub pressure: f32,
    
    pub force: [f32; 3],
    pub _padding: f32,
}

/// Fluid simulation parameters
pub struct FluidParams {
    pub rest_density: f32,
    pub gas_constant: f32,
    pub viscosity: f32,
    pub surface_tension: f32,
    pub gravity: Vec3,
    pub particle_mass: f32,
    pub smoothing_radius: f32,
}

impl FluidSimulator {
    /// Simulate one timestep
    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder, dt: f32) {
        // 1. Update spatial hash grid
        self.spatial_grid.update(encoder, &self.particles);
        
        // 2. Compute density and pressure
        self.compute_density(encoder);
        
        // 3. Compute forces (pressure, viscosity, surface tension)
        self.compute_forces(encoder);
        
        // 4. Integrate (update positions and velocities)
        self.integrate(encoder, dt);
    }
    
    fn compute_density(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SPH Density"),
            ..Default::default()
        });
        
        pass.set_pipeline(&self.density_pipeline);
        pass.set_bind_group(0, &self.get_bind_group(), &[]);
        
        let workgroups = (self.particle_count + 255) / 256;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
}

/// Spatial hash grid for fast neighbor search
pub struct SpatialHashGrid {
    /// Grid cells (each cell contains particle indices)
    cells: wgpu::Buffer,
    
    /// Cell size (= smoothing radius)
    cell_size: f32,
    
    /// Grid dimensions
    grid_size: (u32, u32, u32),
    
    /// Hash function pipeline
    hash_pipeline: wgpu::ComputePipeline,
}
```

### 11. Water Rendering

```rust
/// Water surface renderer
pub struct WaterRenderer {
    /// Water mesh (plane with tessellation)
    mesh: Mesh,
    
    /// Normal maps for waves
    normal_maps: [wgpu::Texture; 2],
    
    /// Reflection texture
    reflection_texture: wgpu::Texture,
    
    /// Refraction texture
    refraction_texture: wgpu::Texture,
    
    /// Depth texture (for depth-based effects)
    depth_texture: wgpu::Texture,
    
    /// Water material
    material: WaterMaterial,
    
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
}

pub struct WaterMaterial {
    pub color_shallow: Color,
    pub color_deep: Color,
    pub wave_speed: f32,
    pub wave_scale: f32,
    pub reflection_strength: f32,
    pub refraction_strength: f32,
    pub foam_threshold: f32,
}

impl WaterRenderer {
    /// Render water surface
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        scene: &Scene,
    ) {
        // 1. Render reflection (flip camera)
        self.render_reflection(encoder, camera, scene);
        
        // 2. Render refraction (underwater scene)
        self.render_refraction(encoder, camera, scene);
        
        // 3. Render water surface with reflection/refraction
        self.render_surface(encoder, camera);
    }
}
```

### 12. Volumetric Fog

```rust
/// Volumetric fog renderer
pub struct VolumetricFog {
    /// 3D fog density texture
    density_texture: wgpu::Texture,
    
    /// Fog parameters
    params: FogParams,
    
    /// Raymarch pipeline
    raymarch_pipeline: wgpu::RenderPipeline,
    
    /// Light scattering compute pipeline
    scatter_pipeline: wgpu::ComputePipeline,
}

pub struct FogParams {
    pub density: f32,
    pub height_falloff: f32,
    pub scattering: f32,
    pub absorption: f32,
    pub phase_g: f32, // Henyey-Greenstein phase function
    pub max_steps: u32,
}

impl VolumetricFog {
    /// Render volumetric fog
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        lights: &[Light],
        depth_texture: &wgpu::TextureView,
    ) {
        // 1. Compute light scattering in fog volume
        self.compute_scattering(encoder, lights);
        
        // 2. Raymarch through fog volume
        self.raymarch(encoder, camera, depth_texture);
    }
}
```

### 13. Grid-Based Physics (2D/3D)

```rust
/// 2D cellular automata grid
pub struct Grid2DPhysics {
    /// Grid cells (material type per cell)
    cells: wgpu::Buffer,
    
    /// Grid size
    width: u32,
    height: u32,
    
    /// Update pipeline (compute shader)
    update_pipeline: wgpu::ComputePipeline,
    
    /// Render pipeline
    render_pipeline: wgpu::RenderPipeline,
    
    /// Material rules
    rules: MaterialRules,
}

/// Material types for cellular automata
#[repr(u32)]
pub enum CellMaterial {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Stone = 3,
    Wood = 4,
    Fire = 5,
}

impl Grid2DPhysics {
    /// Update grid (cellular automata rules)
    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Grid Physics Update"),
            ..Default::default()
        });
        
        pass.set_pipeline(&self.update_pipeline);
        pass.set_bind_group(0, &self.get_bind_group(), &[]);
        
        // Dispatch one thread per cell
        let workgroups_x = (self.width + 15) / 16;
        let workgroups_y = (self.height + 15) / 16;
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}

/// 3D voxel physics
pub struct Grid3DPhysics {
    /// Voxel grid
    voxels: wgpu::Buffer,
    
    /// Grid size
    size: (u32, u32, u32),
    
    /// Update pipeline
    update_pipeline: wgpu::ComputePipeline,
    
    /// Meshing pipeline (greedy meshing)
    meshing_pipeline: wgpu::ComputePipeline,
    
    /// Render mesh
    mesh: DynamicMesh,
}
```


### 14. Destruction System

```rust
/// Destruction system for breakable objects
pub struct DestructionSystem {
    /// Pre-fractured meshes cache
    fractured_cache: HashMap<MeshId, FracturedMesh>,
    
    /// Active debris
    debris: Vec<Debris>,
    
    /// Physics integration
    physics: DebrisPhysics,
    
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
}

/// Pre-fractured mesh
pub struct FracturedMesh {
    /// Original mesh
    original: Mesh,
    
    /// Fracture pieces
    pieces: Vec<MeshPiece>,
    
    /// Connectivity graph (for progressive destruction)
    connectivity: Vec<Vec<usize>>,
}

/// Debris piece
pub struct Debris {
    pub mesh_piece: MeshPiece,
    pub transform: Transform,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub lifetime: f32,
    pub fade_start: f32,
}

impl DestructionSystem {
    /// Destroy an object
    pub fn destroy(
        &mut self,
        mesh_id: MeshId,
        impact_point: Vec3,
        impact_force: Vec3,
    ) {
        // Get or create fractured mesh
        let fractured = self.fractured_cache.entry(mesh_id)
            .or_insert_with(|| self.fracture_mesh(mesh_id));
        
        // Spawn debris pieces
        for piece in &fractured.pieces {
            let velocity = self.calculate_debris_velocity(
                piece.center,
                impact_point,
                impact_force,
            );
            
            self.debris.push(Debris {
                mesh_piece: piece.clone(),
                transform: Transform::from_translation(piece.center),
                velocity,
                angular_velocity: Vec3::random() * 5.0,
                lifetime: 10.0,
                fade_start: 8.0,
            });
        }
    }
    
    /// Update debris physics
    pub fn update(&mut self, dt: f32) {
        for debris in &mut self.debris {
            // Apply gravity
            debris.velocity.y -= 9.81 * dt;
            
            // Update position
            debris.transform.position += debris.velocity * dt;
            
            // Update rotation
            let rotation_delta = Quat::from_scaled_axis(debris.angular_velocity * dt);
            debris.transform.rotation = rotation_delta * debris.transform.rotation;
            
            // Update lifetime
            debris.lifetime -= dt;
        }
        
        // Remove expired debris
        self.debris.retain(|d| d.lifetime > 0.0);
        
        // Merge distant debris to reduce draw calls
        if self.debris.len() > 1000 {
            self.merge_distant_debris();
        }
    }
}
```

### 15. Shadow System

```rust
/// Shadow mapping system
pub struct ShadowSystem {
    /// Shadow maps for each light
    shadow_maps: HashMap<LightId, ShadowMap>,
    
    /// Shadow render pipeline
    shadow_pipeline: wgpu::RenderPipeline,
    
    /// PCF filter pipeline
    pcf_pipeline: wgpu::RenderPipeline,
    
    /// Cascade shadow maps for directional lights
    csm: CascadedShadowMaps,
}

/// Shadow map for a single light
pub struct ShadowMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub resolution: u32,
    pub light_view_proj: Mat4,
}

/// Cascaded shadow maps (for directional lights)
pub struct CascadedShadowMaps {
    /// Shadow map cascades
    cascades: Vec<ShadowCascade>,
    
    /// Split distances
    split_distances: Vec<f32>,
}

pub struct ShadowCascade {
    pub shadow_map: ShadowMap,
    pub near: f32,
    pub far: f32,
}

impl ShadowSystem {
    /// Render shadow maps for all lights
    pub fn render_shadows(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        lights: &[Light],
        entities: &[RenderEntity],
    ) {
        for light in lights {
            if light.cast_shadows {
                self.render_shadow_map(encoder, light, entities);
            }
        }
    }
    
    fn render_shadow_map(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        light: &Light,
        entities: &[RenderEntity],
    ) {
        let shadow_map = self.shadow_maps.entry(light.id)
            .or_insert_with(|| self.create_shadow_map(light));
        
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &shadow_map.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        
        pass.set_pipeline(&self.shadow_pipeline);
        
        // Render entities from light's perspective
        for entity in entities {
            if entity.cast_shadows {
                self.render_entity_shadow(&mut pass, entity, &shadow_map.light_view_proj);
            }
        }
    }
}
```

### 16. Light System

```rust
/// Light management system
pub struct LightSystem {
    /// All lights in scene
    lights: Vec<Light>,
    
    /// Light buffer (GPU)
    light_buffer: wgpu::Buffer,
    
    /// Light culling (for clustered/tiled rendering)
    light_culling: LightCulling,
}

/// Light types
pub enum Light {
    Directional {
        id: LightId,
        direction: Vec3,
        color: Color,
        intensity: f32,
        cast_shadows: bool,
    },
    Point {
        id: LightId,
        position: Vec3,
        color: Color,
        intensity: f32,
        radius: f32,
        cast_shadows: bool,
    },
    Spot {
        id: LightId,
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
        radius: f32,
        cast_shadows: bool,
    },
}

/// Light culling for clustered rendering
pub struct LightCulling {
    /// 3D grid of clusters
    clusters: Vec<Cluster>,
    cluster_size: (u32, u32, u32),
    
    /// Light index lists per cluster
    light_indices: wgpu::Buffer,
    
    /// Culling compute pipeline
    culling_pipeline: wgpu::ComputePipeline,
}

pub struct Cluster {
    pub min: Vec3,
    pub max: Vec3,
    pub light_count: u32,
    pub light_offset: u32,
}
```


### 17. Mobile Optimization Systems

```rust
/// Thermal management for mobile devices
pub struct ThermalManager {
    /// Current thermal state
    thermal_state: ThermalState,
    
    /// Temperature history
    temperature_history: Vec<f32>,
    
    /// Quality settings per thermal state
    quality_profiles: HashMap<ThermalState, QualityProfile>,
    
    /// Platform thermal API
    platform_api: Box<dyn PlatformThermalAPI>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThermalState {
    Nominal,      // Normal operation
    Fair,         // Slightly warm
    Serious,      // Hot, reduce quality
    Critical,     // Very hot, aggressive reduction
}

pub struct QualityProfile {
    pub resolution_scale: f32,
    pub shadow_quality: ShadowQuality,
    pub post_processing: bool,
    pub particle_limit: u32,
    pub draw_distance: f32,
    pub target_fps: u32,
}

impl ThermalManager {
    /// Update thermal state and adjust quality
    pub fn update(&mut self, renderer: &mut RenderModule) {
        // Query platform thermal state
        let temp = self.platform_api.get_temperature();
        self.temperature_history.push(temp);
        
        // Determine thermal state
        let new_state = self.determine_thermal_state(temp);
        
        if new_state != self.thermal_state {
            self.thermal_state = new_state;
            
            // Apply quality profile
            let profile = &self.quality_profiles[&new_state];
            self.apply_quality_profile(renderer, profile);
        }
    }
    
    fn determine_thermal_state(&self, temp: f32) -> ThermalState {
        match temp {
            t if t < 40.0 => ThermalState::Nominal,
            t if t < 50.0 => ThermalState::Fair,
            t if t < 60.0 => ThermalState::Serious,
            _ => ThermalState::Critical,
        }
    }
}

/// Adaptive quality system
pub struct AdaptiveQuality {
    /// Target frame time (ms)
    target_frame_time: f32,
    
    /// Frame time history
    frame_times: Vec<f32>,
    
    /// Current quality level (0.0 - 1.0)
    quality_level: f32,
    
    /// Adjustment rate
    adjustment_rate: f32,
}

impl AdaptiveQuality {
    /// Update quality based on frame time
    pub fn update(&mut self, frame_time: f32, renderer: &mut RenderModule) {
        self.frame_times.push(frame_time);
        
        // Keep last 60 frames
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Calculate average frame time
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        
        // Adjust quality
        if avg_frame_time > self.target_frame_time * 1.1 {
            // Too slow, reduce quality
            self.quality_level = (self.quality_level - self.adjustment_rate).max(0.0);
            self.apply_quality(renderer);
        } else if avg_frame_time < self.target_frame_time * 0.9 {
            // Fast enough, increase quality
            self.quality_level = (self.quality_level + self.adjustment_rate * 0.5).min(1.0);
            self.apply_quality(renderer);
        }
    }
    
    fn apply_quality(&self, renderer: &mut RenderModule) {
        // Adjust resolution
        let scale = 0.5 + self.quality_level * 0.5; // 0.5x to 1.0x
        renderer.set_resolution_scale(scale);
        
        // Adjust shadow quality
        if self.quality_level < 0.3 {
            renderer.shadow_system.set_resolution(512);
        } else if self.quality_level < 0.7 {
            renderer.shadow_system.set_resolution(1024);
        } else {
            renderer.shadow_system.set_resolution(2048);
        }
        
        // Adjust particle count
        let particle_limit = (2000.0 + self.quality_level * 8000.0) as u32;
        renderer.particle_system.set_max_particles(particle_limit);
    }
}
```

### 18. GPU Profiling and Debugging

```rust
/// GPU profiler
pub struct GpuProfiler {
    /// Timestamp queries
    query_set: wgpu::QuerySet,
    
    /// Query results buffer
    query_buffer: wgpu::Buffer,
    
    /// Profiling scopes
    scopes: Vec<ProfileScope>,
    
    /// Frame statistics
    frame_stats: FrameStats,
    
    /// Enabled flag
    enabled: bool,
}

pub struct ProfileScope {
    pub name: String,
    pub start_query: u32,
    pub end_query: u32,
    pub gpu_time_ms: f32,
}

pub struct FrameStats {
    pub total_gpu_time: f32,
    pub draw_calls: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub texture_memory: u64,
    pub buffer_memory: u64,
}

impl GpuProfiler {
    /// Begin a profiling scope
    pub fn begin_scope(&mut self, encoder: &mut wgpu::CommandEncoder, name: &str) -> ScopeId {
        if !self.enabled {
            return ScopeId::invalid();
        }
        
        let scope_id = ScopeId(self.scopes.len());
        let query_id = self.scopes.len() as u32 * 2;
        
        encoder.write_timestamp(&self.query_set, query_id);
        
        self.scopes.push(ProfileScope {
            name: name.to_string(),
            start_query: query_id,
            end_query: query_id + 1,
            gpu_time_ms: 0.0,
        });
        
        scope_id
    }
    
    /// End a profiling scope
    pub fn end_scope(&mut self, encoder: &mut wgpu::CommandEncoder, scope_id: ScopeId) {
        if !self.enabled || scope_id.is_invalid() {
            return;
        }
        
        let scope = &self.scopes[scope_id.0];
        encoder.write_timestamp(&self.query_set, scope.end_query);
    }
    
    /// Resolve queries and calculate timings
    pub fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.enabled {
            return;
        }
        
        encoder.resolve_query_set(
            &self.query_set,
            0..self.scopes.len() as u32 * 2,
            &self.query_buffer,
            0,
        );
    }
    
    /// Get profiling results
    pub fn get_results(&self) -> &[ProfileScope] {
        &self.scopes
    }
}

/// Shader debugger
pub struct ShaderDebugger {
    /// Debug visualization mode
    mode: DebugMode,
    
    /// Debug render pipeline
    debug_pipeline: wgpu::RenderPipeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    None,
    Normals,
    UVs,
    Depth,
    Albedo,
    Metallic,
    Roughness,
    AO,
    Emissive,
    Wireframe,
    Overdraw,
}

impl ShaderDebugger {
    /// Render debug visualization
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        gbuffer: &GBuffer,
    ) {
        if self.mode == DebugMode::None {
            return;
        }
        
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Debug Visualization"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        
        pass.set_pipeline(&self.debug_pipeline);
        
        // Bind appropriate G-buffer texture based on mode
        let texture = match self.mode {
            DebugMode::Normals => &gbuffer.normal_roughness,
            DebugMode::Albedo => &gbuffer.albedo_metallic,
            DebugMode::Depth => &gbuffer.depth,
            _ => return,
        };
        
        // Draw fullscreen quad
        pass.draw(0..6, 0..1);
    }
}
```


---

## Data Models

### Material Data Model

```rust
/// Material asset (serializable)
#[derive(Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: String,
    pub shader: String,
    pub parameters: HashMap<String, MaterialParameter>,
    pub textures: HashMap<String, String>, // name -> path
    pub render_queue: RenderQueue,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub depth_test: bool,
    pub depth_write: bool,
}

#[derive(Serialize, Deserialize)]
pub enum MaterialParameter {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color(Color),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum RenderQueue {
    Background = 1000,
    Geometry = 2000,
    AlphaTest = 2450,
    Transparent = 3000,
    Overlay = 4000,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Additive,
    Multiply,
}
```

### Mesh Data Model

```rust
/// Mesh data
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounds: BoundingBox,
    
    /// GPU buffers
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

/// Vertex layout
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4], // w = handedness
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tangent
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
```

### Camera Data Model

```rust
/// Camera component
pub struct Camera {
    pub projection: Projection,
    pub transform: Transform,
    pub viewport: Viewport,
    pub clear_color: Color,
    pub clear_flags: ClearFlags,
    pub render_target: Option<RenderTargetId>,
}

pub enum Projection {
    Perspective {
        fov: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        size: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.transform.position,
            self.transform.position + self.transform.forward(),
            Vec3::Y,
        )
    }
    
    /// Get projection matrix
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective { fov, near, far } => {
                Mat4::perspective_rh(fov, aspect, near, far)
            }
            Projection::Orthographic { size, near, far } => {
                let half_width = size * aspect * 0.5;
                let half_height = size * 0.5;
                Mat4::orthographic_rh(
                    -half_width, half_width,
                    -half_height, half_height,
                    near, far,
                )
            }
        }
    }
    
    /// Get view-projection matrix
    pub fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }
}
```

### Render Entity Data Model

```rust
/// Entity to be rendered
pub struct RenderEntity {
    pub entity_id: Entity,
    pub mesh: MeshId,
    pub material: MaterialId,
    pub transform: Transform,
    pub bounds: BoundingBox,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub layer: u32,
}

/// Transform component
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    /// Get model matrix
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.position,
        )
    }
    
    /// Get forward vector
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }
    
    /// Get right vector
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    
    /// Get up vector
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
}
```

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property 1: Material data completeness
*For any* material creation, the created material should contain all specified shader references, textures, and parameters.
**Validates: Requirements 1.1**

### Property 2: Material parameter GPU synchronization
*For any* material parameter change, reading the GPU uniform buffer should return the updated value within one frame.
**Validates: Requirements 1.2**

### Property 3: Draw call batching
*For any* set of entities sharing the same material, the number of draw calls should be less than or equal to the number of unique materials.
**Validates: Requirements 1.3**

### Property 4: Node graph type validation
*For any* node connection attempt, connecting incompatible types should be rejected with a type error.
**Validates: Requirements 2.2**

### Property 5: Shader code generation validity
*For any* valid material node graph, the generated WGSL code should compile without errors.
**Validates: Requirements 2.3**

### Property 6: Shader include resolution
*For any* shader with include directives, the final compiled code should contain the included file's content.
**Validates: Requirements 4.1**

### Property 7: Include dependency recompilation
*For any* include file modification, all shaders that depend on it should be marked for recompilation.
**Validates: Requirements 4.2**

### Property 8: Post-processing effect ordering
*For any* sequence of enabled post-processing effects, they should execute in the configured order.
**Validates: Requirements 5.2**

### Property 9: Pipeline caching
*For any* pipeline descriptor, creating a pipeline with the same descriptor twice should return the cached instance on the second call.
**Validates: Requirements 21.2**

### Property 10: Pipeline variant creation
*For any* pipeline state modification, a new pipeline variant should be created rather than modifying the existing one.
**Validates: Requirements 21.3**

### Property 11: PSO completeness
*For any* pipeline state object, it should contain all required fields: shader, blend mode, depth test, and culling settings.
**Validates: Requirements 22.1**

### Property 12: Render graph dependency ordering
*For any* render graph with pass dependencies, the execution order should respect all dependency constraints (topological sort).
**Validates: Requirements 23.2**

### Property 13: Render graph resource reuse
*For any* two render passes that can share a texture, the system should reuse the same texture rather than allocating a new one.
**Validates: Requirements 23.3**

### Property 14: Post-processing performance budget
*For any* frame with post-processing enabled, the total post-processing time should be less than 4ms.
**Validates: Requirements 43.1**

### Property 15: Particle system performance budget
*For any* frame with active particles, the particle update time should be less than 2ms.
**Validates: Requirements 43.2**

### Property 16: Material serialization completeness
*For any* material, serializing it to JSON should preserve all parameters, textures, and settings.
**Validates: Requirements 44.1**

### Property 17: Material deserialization accuracy
*For any* serialized material, deserializing it should restore all parameter values exactly.
**Validates: Requirements 44.2**

### Property 18: Material serialization round-trip
*For any* material, serializing then deserializing should produce a material that renders identically to the original.
**Validates: Requirements 44.3**

### Property 19: Shader compilation caching
*For any* compiled shader, a cache file should exist on disk after compilation.
**Validates: Requirements 45.1**

### Property 20: Shader cache validation
*For any* cached shader, loading it should verify the source hash matches before use.
**Validates: Requirements 45.2**

### Property 21: Shader cache invalidation
*For any* shader source modification, the cached version should be recompiled and the cache updated.
**Validates: Requirements 45.3**

### Property 22: GPU memory tracking
*For any* GPU resource allocation, the total tracked memory usage should equal the sum of all allocated resource sizes.
**Validates: Requirements 46.1**

### Property 23: Memory pressure quality reduction
*For any* memory usage exceeding 80%, texture quality should be automatically reduced.
**Validates: Requirements 46.3**

---

## Error Handling

### Error Types

```rust
#[derive(Debug, Clone)]
pub enum RenderError {
    // Shader errors
    ShaderCompilationFailed { path: String, error: String },
    ShaderNotFound { path: String },
    CircularInclude { chain: Vec<String> },
    
    // Pipeline errors
    PipelineCreationFailed { desc: String, error: String },
    InvalidPipelineState { reason: String },
    
    // Material errors
    MaterialNotFound { id: MaterialId },
    InvalidMaterialParameter { name: String, expected_type: String },
    TextureNotFound { path: String },
    
    // Resource errors
    OutOfMemory { requested: u64, available: u64 },
    TextureCreationFailed { size: (u32, u32), format: String },
    BufferCreationFailed { size: u64 },
    
    // Render graph errors
    CyclicDependency { passes: Vec<String> },
    InvalidPassDependency { pass: String, dependency: String },
    
    // Platform errors
    SurfaceLost,
    DeviceLost,
    Timeout,
}
```

### Error Handling Strategy

1. **Shader Compilation Errors**
   - Display detailed error messages with line numbers
   - Fall back to default shader
   - Visualize error with pink/magenta material
   - Log to console and error file

2. **GPU Memory Errors**
   - Attempt to free unused resources
   - Reduce texture quality automatically
   - Disable non-essential effects
   - Log memory usage report
   - Retry allocation once

3. **Pipeline Errors**
   - Fall back to default pipeline
   - Log error details
   - Continue rendering with reduced quality

4. **Surface/Device Lost**
   - Attempt to recreate surface
   - Rebuild all GPU resources
   - Resume rendering if successful
   - Exit gracefully if unrecoverable

---

## Testing Strategy

### Unit Tests

**Material System:**
- Material creation and parameter setting
- Material serialization/deserialization
- Material parameter GPU buffer updates
- Material batching logic

**Pipeline Manager:**
- Pipeline creation and caching
- Pipeline descriptor hashing
- Shader hot-reload
- Pipeline variant generation

**Render Graph:**
- Pass registration and dependency tracking
- Topological sorting
- Resource allocation and reuse
- Cycle detection

**Post-Processing:**
- Effect chain execution
- Ping-pong buffer management
- Individual effect correctness (bloom, DOF, etc.)

**Particle System:**
- Particle spawning and lifecycle
- GPU compute shader execution
- Particle sorting and rendering
- Emitter configuration

### Property-Based Tests

**Property 1: Material data completeness**
- Generate random materials with various parameters
- Verify all data is stored correctly

**Property 2: Material parameter GPU synchronization**
- Generate random parameter changes
- Verify GPU buffer contains updated values

**Property 3: Draw call batching**
- Generate scenes with shared materials
- Verify draw call count ≤ unique material count

**Property 9: Pipeline caching**
- Generate random pipeline descriptors
- Create same pipeline twice
- Verify second creation returns cached instance

**Property 12: Render graph dependency ordering**
- Generate random render graphs with dependencies
- Verify execution order respects all dependencies

**Property 18: Material serialization round-trip**
- Generate random materials
- Serialize, deserialize, compare rendering output

**Property 21: Shader cache invalidation**
- Compile shader, modify source, reload
- Verify cache is updated

**Property 22: GPU memory tracking**
- Allocate various resources
- Verify tracked memory equals sum of allocations

### Performance Tests

**Frame Time Budgets:**
- Post-processing: <4ms
- Particle system: <2ms
- Shadow mapping: <3ms
- Fluid simulation: <5ms (desktop), <8ms (mobile)

**Throughput Tests:**
- 10,000 draw calls per frame (desktop)
- 1,000 draw calls per frame (mobile)
- 100,000 particles (desktop)
- 10,000 particles (mobile)

**Memory Tests:**
- Texture streaming under memory pressure
- GPU memory exhaustion recovery
- Memory leak detection (run for 1000 frames)

### Integration Tests

**Full Rendering Pipeline:**
- Create scene with various entities
- Render with forward pipeline
- Render with deferred pipeline
- Apply post-processing
- Verify output correctness

**Material Editor:**
- Create material in editor
- Generate shader code
- Compile and use in scene
- Verify rendering matches preview

**Hot Reload:**
- Modify shader file
- Verify automatic recompilation
- Verify rendering updates without restart

**Mobile Optimization:**
- Simulate thermal throttling
- Verify quality reduction
- Verify frame rate maintained
- Verify no crashes under pressure

---

## Performance Optimization Techniques

### 1. Draw Call Reduction

**Batching:**
- Static batching: Pre-combine meshes at build time
- Dynamic batching: Combine small meshes at runtime
- Instancing: Draw multiple copies with one call
- Material batching: Group by material to reduce state changes

**Culling:**
- Frustum culling: Skip objects outside camera view
- Occlusion culling: Skip objects hidden behind others
- Distance culling: Skip distant objects
- Layer culling: Skip objects on disabled layers

### 2. GPU Optimization

**Compute Shaders:**
- Particle updates on GPU
- Fluid simulation on GPU
- Culling on GPU
- Post-processing on GPU

**Memory Access:**
- Coalesce memory reads
- Use shared memory in compute shaders
- Minimize texture fetches
- Use texture atlases

**Pipeline Optimization:**
- Minimize pipeline switches
- Sort by pipeline state
- Cache pipelines aggressively
- Use pipeline derivatives

### 3. Mobile-Specific Optimizations

**Resolution Scaling:**
- Render at lower resolution
- Upscale to display resolution
- Adaptive resolution based on performance

**Quality Reduction:**
- Lower shadow resolution
- Reduce particle count
- Disable expensive effects
- Use simpler shaders

**Thermal Management:**
- Monitor device temperature
- Reduce quality when hot
- Cap frame rate when necessary
- Pause non-essential systems

### 4. Memory Optimization

**Texture Streaming:**
- Load low-res mips first
- Stream high-res mips on demand
- Unload distant textures
- Use texture compression

**Resource Pooling:**
- Reuse render targets
- Pool particle buffers
- Cache pipelines
- Reuse uniform buffers

---

## Platform-Specific Considerations

### Desktop (Windows, macOS, Linux)

**Features:**
- High-quality shadows (4K)
- Full post-processing
- Many particles (100K+)
- Complex fluid simulation
- High draw distance

**Optimizations:**
- Use compute shaders extensively
- Enable all visual effects
- High-resolution textures
- MSAA or TAA anti-aliasing

### Mobile (iOS, Android)

**Features:**
- Medium-quality shadows (1K)
- Selective post-processing
- Moderate particles (10K)
- Simplified fluid simulation
- Reduced draw distance

**Optimizations:**
- Thermal management
- Adaptive quality
- Resolution scaling
- Aggressive culling
- Texture compression (ASTC, ETC2)

### Web (WebAssembly + WebGPU)

**Features:**
- Low-quality shadows (512)
- Minimal post-processing
- Few particles (2K)
- No fluid simulation
- Short draw distance

**Optimizations:**
- Minimize shader complexity
- Reduce texture sizes
- Limit draw calls
- Use texture atlases
- Aggressive LOD

---

## Migration and Integration

### Integration with ECS

```rust
// Rendering components
pub struct MeshRenderer {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

pub struct Camera {
    pub projection: Projection,
    pub clear_color: Color,
    pub render_target: Option<RenderTargetId>,
}

pub struct Light {
    pub light_type: LightType,
    pub color: Color,
    pub intensity: f32,
    pub cast_shadows: bool,
}

// Rendering system
pub fn render_system(
    world: &World,
    renderer: &mut RenderModule,
) {
    // Query all cameras
    for (entity, camera, transform) in world.query::<(&Camera, &Transform)>() {
        // Collect visible entities
        let entities = collect_visible_entities(world, camera, transform);
        
        // Render scene
        renderer.render_scene(camera, &entities);
    }
}
```

### Backward Compatibility

The new renderer maintains compatibility with existing sprite and tilemap renderers:

```rust
impl RenderModule {
    /// Legacy sprite rendering (compatibility)
    pub fn render_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        // Convert to new material system
        let material = self.get_or_create_sprite_material(sprite);
        
        // Render using new pipeline
        self.render_entity(&RenderEntity {
            mesh: self.quad_mesh,
            material,
            transform: *transform,
            ..Default::default()
        });
    }
}
```

---

## Conclusion

This design provides a comprehensive, AAA-quality rendering system with:

- **Flexibility**: Support for both forward and deferred rendering
- **Artist Tools**: Visual material editor with node-based shader authoring
- **Advanced Features**: PBR, post-processing, VFX, fluid simulation, destruction
- **Mobile-First**: Thermal management, adaptive quality, 60 FPS on mid-range devices
- **Performance**: GPU-accelerated everything, aggressive optimization
- **Developer Experience**: Hot reload, profiling, debugging tools

The architecture is modular, extensible, and production-ready for both 2D pixel art games and modern 3D stylized games.
