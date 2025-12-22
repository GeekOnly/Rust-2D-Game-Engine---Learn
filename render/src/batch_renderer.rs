use wgpu::util::DeviceExt;
use crate::texture::Texture;
use crate::sprite_renderer::Vertex;
use crate::texture::TextureManager; // Added import

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // UV Offset
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // UV Scale
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use glam::Mat4;
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, view_projection_matrix: glam::Mat4) {
        self.view_proj = view_projection_matrix.to_cols_array_2d();
    }
}

// Batch Data for Deferred Rendering
struct BatchData {
    buffer: wgpu::Buffer,
    texture_id: String,
    count: u32,
}

pub struct BatchRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    // Instance data
    pub instances: Vec<InstanceRaw>,
    instance_buffer: wgpu::Buffer,
    
    // Camera
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    // Deferred Batches
    batches: Vec<BatchData>,
}

impl BatchRenderer {
    pub const MAX_INSTANCES: u64 = 10000;

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Batch Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite_shader.wgsl").into()),
        });

        // 1. Texture Bind Group Layout
        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        // 2. Camera Bind Group Layout & Buffer
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                // Match CameraBinding visibility (Vertex | Fragment) for layout compatibility
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // 3. Pipeline Layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Batch Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout, // Group 0
                &camera_bind_group_layout,  // Group 1
            ],
            push_constant_ranges: &[],
        });

        // 4. Render Pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Batch Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Standard Z
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // 5. Geometry (Quad)
        let vertices = &[
            Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // TL
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // BL
            Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // BR
            Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // TR
        ];
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // 6. Instance Buffer (Pre-allocated)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: Self::MAX_INSTANCES * std::mem::size_of::<InstanceRaw>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            instances: Vec::with_capacity(Self::MAX_INSTANCES as usize),
            instance_buffer,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            batches: Vec::new(),
        }
    }

    pub fn start_frame(&mut self) {
        self.batches.clear();
        self.instances.clear();
    }

    pub fn begin_frame(&mut self) {
        self.instances.clear();
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, view_proj: glam::Mat4) {
        self.camera_uniform.update_view_proj(view_proj);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        // Debug: Print once to verify update_camera is called
        static mut UPDATE_LOGGED: bool = false;
        unsafe {
            if !UPDATE_LOGGED {
                println!("DEBUG: BatchRenderer.update_camera() called with view_proj = {:?}", view_proj);
                UPDATE_LOGGED = true;
            }
        }
    }

    pub fn draw_sprite(
        &mut self,
        position: glam::Vec3,
        rotation: glam::Quat,
        scale: glam::Vec3,
        color: [f32; 4],
        uv_offset: [f32; 2],
        uv_scale: [f32; 2],
    ) {
        // Build transform matrix: T * R * S (Translation, Rotation, Scale)
        // from_scale_rotation_translation applies them in the correct order
        let transform = glam::Mat4::from_scale_rotation_translation(scale, rotation, position);

        // Debug: Print first sprite transform
        static mut FIRST_TRANSFORM_LOGGED: bool = false;
        unsafe {
            if !FIRST_TRANSFORM_LOGGED {
                println!("DEBUG: First sprite transform - pos: {:?}, scale: {:?}", position, scale);
                println!("DEBUG: Model matrix = {:?}", transform);
                FIRST_TRANSFORM_LOGGED = true;
            }
        }

        let instance = InstanceRaw {
            model: transform.to_cols_array_2d(),
            color,
            uv_offset,
            uv_scale,
        };
        self.instances.push(instance);
    }

    /// Complete the current batch, creating a buffer for it.
    pub fn finish_batch(
        &mut self,
        device: &wgpu::Device,
        texture_id: String,
    ) {
        if self.instances.is_empty() {
            return;
        }

        let instance_bytes = bytemuck::cast_slice(&self.instances);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
             label: Some("Batch Instance Buffer"),
             contents: instance_bytes,
             usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.batches.push(BatchData {
            buffer,
            texture_id,
            count: self.instances.len() as u32,
        });
        
        self.instances.clear();
    }
    
    /// Render all collected batches
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture_manager: &'a TextureManager,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if self.batches.is_empty() {
             return;
        }
         
        render_pass.set_pipeline(&self.render_pipeline);
        // Use the passed camera bind group (from CameraBinding) instead of internal one
        render_pass.set_bind_group(1, camera_bind_group, &[]);
        
        for batch in &self.batches {
             if let Some(texture) = texture_manager.get_texture(&batch.texture_id) {
                 if let Some(bind_group) = &texture.bind_group {
                     // Bind Texture
                     render_pass.set_bind_group(0, bind_group, &[]);
                     
                     // Bind Buffers
                     render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                     render_pass.set_vertex_buffer(1, batch.buffer.slice(..));
                     render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                     
                     // Draw
                     render_pass.draw_indexed(0..self.num_indices, 0, 0..batch.count);
                 }
             }
        }
    }
}
