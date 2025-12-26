// 🎬 AAA Mobile Post-Processing Renderer
// Handles HDR to SDR conversion with tonemapping, bloom, and effects

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessUniforms {
    pub exposure: f32,
    pub bloom_intensity: f32,
    pub contrast: f32,
    pub saturation: f32,

    pub vignette_strength: f32,
    pub vignette_smoothness: f32,
    pub chromatic_aberration: f32,
    pub _padding: f32,
}

impl Default for PostProcessUniforms {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            bloom_intensity: 0.04,
            contrast: 1.0,
            saturation: 1.0,
            vignette_strength: 0.5,
            vignette_smoothness: 0.3,
            chromatic_aberration: 0.0,
            _padding: 0.0,
        }
    }
}

pub struct PostProcessRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout_hdr: wgpu::BindGroupLayout,
    pub bind_group_layout_bloom: wgpu::BindGroupLayout,
    pub bind_group_layout_uniforms: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub uniforms: PostProcessUniforms,

    // Bind groups (created per-frame with textures)
    pub hdr_bind_group: Option<wgpu::BindGroup>,
    pub bloom_bind_group: Option<wgpu::BindGroup>,
    pub uniforms_bind_group: wgpu::BindGroup,
}

impl PostProcessRenderer {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
        // Create bind group layouts
        let bind_group_layout_hdr = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Process HDR Bind Group Layout"),
            entries: &[
                // HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // HDR sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group_layout_bloom = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Process Bloom Bind Group Layout"),
            entries: &[
                // Bloom texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Bloom sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group_layout_uniforms = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Process Uniforms Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create uniform buffer
        let uniforms = PostProcessUniforms::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Post Process Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post Process Uniforms Bind Group"),
            layout: &bind_group_layout_uniforms,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Process Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/post_process.wgsl").into()),
        });

        // Create pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Process Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout_hdr,
                &bind_group_layout_bloom,
                &bind_group_layout_uniforms,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post Process Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Fullscreen quad
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // No depth test for fullscreen quad
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout_hdr,
            bind_group_layout_bloom,
            bind_group_layout_uniforms,
            uniform_buffer,
            uniforms,
            hdr_bind_group: None,
            bloom_bind_group: None,
            uniforms_bind_group,
        }
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn create_hdr_bind_group(
        &mut self,
        device: &wgpu::Device,
        hdr_view: &wgpu::TextureView,
        hdr_sampler: &wgpu::Sampler,
    ) {
        self.hdr_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post Process HDR Bind Group"),
            layout: &self.bind_group_layout_hdr,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(hdr_sampler),
                },
            ],
        }));
    }

    pub fn create_bloom_bind_group(
        &mut self,
        device: &wgpu::Device,
        bloom_view: &wgpu::TextureView,
        bloom_sampler: &wgpu::Sampler,
    ) {
        self.bloom_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post Process Bloom Bind Group"),
            layout: &self.bind_group_layout_bloom,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(bloom_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(bloom_sampler),
                },
            ],
        }));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post Process Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        if let Some(hdr_bind_group) = &self.hdr_bind_group {
            render_pass.set_bind_group(0, hdr_bind_group, &[]);
        }

        if let Some(bloom_bind_group) = &self.bloom_bind_group {
            render_pass.set_bind_group(1, bloom_bind_group, &[]);
        }

        render_pass.set_bind_group(2, &self.uniforms_bind_group, &[]);

        // Draw fullscreen triangle (3 vertices, no index buffer needed)
        render_pass.draw(0..3, 0..1);
    }
}
