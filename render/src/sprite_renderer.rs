use wgpu::util::DeviceExt;
use crate::texture::Texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct SpriteRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    bind_group_layout: wgpu::BindGroupLayout,
    // Cache for custom sprite rect rendering
    custom_vertex_buffer: Option<wgpu::Buffer>,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Simple Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("simple_sprite_shader.wgsl").into()),
        });

        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater, // Reverse-Z
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -1, // Negative for Reverse-Z // Small bias for sprites
                    slope_scale: -1.0, // Negative for Reverse-Z // Small slope bias for sprites
                    clamp: 0.0, // Maximum depth bias clamp
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Basic quad vertices
        let vertices = &[
            Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // Top Left
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // Bottom Left
            Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // Bottom Right
            Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // Top Right
        ];

        let indices: &[u16] = &[
            0, 1, 2,
            0, 2, 3,
        ];

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

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            bind_group_layout: texture_bind_group_layout,
            custom_vertex_buffer: None,
        }
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &'a Texture,
        _device: &wgpu::Device,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if let Some(bind_group) = &texture.bind_group {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    /// Render sprite with custom sprite rect (Unity-style)
    /// sprite_rect: [x, y, width, height] in pixels
    /// texture_size: [width, height] of the full texture in pixels
    pub fn render_with_rect<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &'a Texture,
        device: &wgpu::Device,
        sprite_rect: [u32; 4],
        texture_size: [u32; 2],
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if let Some(bind_group) = &texture.bind_group {
            // Calculate UV coordinates from sprite rect
            let u_min = sprite_rect[0] as f32 / texture_size[0] as f32;
            let v_min = sprite_rect[1] as f32 / texture_size[1] as f32;
            let u_max = (sprite_rect[0] + sprite_rect[2]) as f32 / texture_size[0] as f32;
            let v_max = (sprite_rect[1] + sprite_rect[3]) as f32 / texture_size[1] as f32;

            // Create vertices with custom UV coordinates
            let vertices = &[
                Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [u_min, v_min] }, // Top Left
                Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [u_min, v_max] }, // Bottom Left
                Vertex { position: [0.5, -0.5, 0.0], tex_coords: [u_max, v_max] }, // Bottom Right
                Vertex { position: [0.5, 0.5, 0.0], tex_coords: [u_max, v_min] }, // Top Right
            ];

            // Create/update custom vertex buffer
            let custom_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Custom Sprite Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            self.custom_vertex_buffer = Some(custom_buffer);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.custom_vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }
}
