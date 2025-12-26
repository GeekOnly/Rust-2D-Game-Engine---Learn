// 🌸 Bloom Renderer - Dual-Filter Bloom for Mobile
// Efficient downsample/upsample chain with hardware filtering

use wgpu::util::DeviceExt;

const BLOOM_MIP_COUNT: u32 = 5; // 1/2, 1/4, 1/8, 1/16, 1/32

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomUniforms {
    pub threshold: f32,        // Luminance threshold (default: 1.0)
    pub soft_threshold: f32,   // Soft knee range (default: 0.5)
    pub intensity: f32,        // Overall bloom intensity (default: 0.04)
    pub _padding: f32,
}

impl Default for BloomUniforms {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            soft_threshold: 0.5,
            intensity: 0.04,
            _padding: 0.0,
        }
    }
}

pub struct BloomMip {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

pub struct BloomRenderer {
    // Mip chain (5 levels: 1/2, 1/4, 1/8, 1/16, 1/32)
    pub mips: Vec<BloomMip>,

    // Pipelines
    pub downsample_prefilter_pipeline: wgpu::RenderPipeline,
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub upsample_first_pipeline: wgpu::RenderPipeline,
    pub upsample_pipeline: wgpu::RenderPipeline,

    // Bind group layouts
    pub source_bind_group_layout: wgpu::BindGroupLayout,
    pub add_bind_group_layout: wgpu::BindGroupLayout,
    pub uniforms_bind_group_layout: wgpu::BindGroupLayout,

    // Uniforms
    pub uniform_buffer: wgpu::Buffer,
    pub uniforms: BloomUniforms,
    pub uniforms_bind_group: wgpu::BindGroup,

    // Sampler
    pub sampler: wgpu::Sampler,
}

impl BloomRenderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        // Create mip chain
        let mut mips = Vec::new();
        let mut mip_width = width / 2;
        let mut mip_height = height / 2;

        for i in 0..BLOOM_MIP_COUNT {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom Mip {}", i)),
                size: wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            mips.push(BloomMip {
                texture,
                view,
                width: mip_width,
                height: mip_height,
            });

            // Next mip is half size
            mip_width = (mip_width / 2).max(1);
            mip_height = (mip_height / 2).max(1);
        }

        // Create sampler (linear filtering for free blur!)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Bloom Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create bind group layouts
        let source_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Source Bind Group Layout"),
            entries: &[
                // Source texture
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
                // Source sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let add_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Add Bind Group Layout"),
            entries: &[
                // Add texture
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
                // Add sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let uniforms_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom Uniforms Bind Group Layout"),
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
        let uniforms = BloomUniforms::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bloom Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom Uniforms Bind Group"),
            layout: &uniforms_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Load shaders
        let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Downsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/bloom_downsample.wgsl").into()),
        });

        let upsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Upsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/bloom_upsample.wgsl").into()),
        });

        // Create pipelines
        let downsample_prefilter_pipeline = Self::create_downsample_pipeline(
            device,
            &downsample_shader,
            "fs_downsample_prefilter",
            &source_bind_group_layout,
            &uniforms_bind_group_layout,
        );

        let downsample_pipeline = Self::create_downsample_pipeline(
            device,
            &downsample_shader,
            "fs_downsample",
            &source_bind_group_layout,
            &uniforms_bind_group_layout,
        );

        let upsample_first_pipeline = Self::create_upsample_first_pipeline(
            device,
            &upsample_shader,
            &source_bind_group_layout,
        );

        let upsample_pipeline = Self::create_upsample_pipeline(
            device,
            &upsample_shader,
            &source_bind_group_layout,
            &add_bind_group_layout,
        );

        Self {
            mips,
            downsample_prefilter_pipeline,
            downsample_pipeline,
            upsample_first_pipeline,
            upsample_pipeline,
            source_bind_group_layout,
            add_bind_group_layout,
            uniforms_bind_group_layout,
            uniform_buffer,
            uniforms,
            uniforms_bind_group,
            sampler,
        }
    }

    fn create_downsample_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        entry_point: &str,
        source_layout: &wgpu::BindGroupLayout,
        uniforms_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Downsample Pipeline Layout"),
            bind_group_layouts: &[source_layout, uniforms_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Bloom Downsample Pipeline ({})", entry_point)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(entry_point),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        })
    }

    fn create_upsample_first_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        source_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Upsample First Pipeline Layout"),
            bind_group_layouts: &[source_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Upsample First Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_upsample_first"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        })
    }

    fn create_upsample_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        source_layout: &wgpu::BindGroupLayout,
        add_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Upsample Pipeline Layout"),
            bind_group_layouts: &[source_layout, add_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Upsample Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_upsample"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        })
    }

    pub fn update_uniforms(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // Recreate mip chain
        self.mips.clear();

        let mut mip_width = width / 2;
        let mut mip_height = height / 2;

        for i in 0..BLOOM_MIP_COUNT {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom Mip {}", i)),
                size: wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.mips.push(BloomMip {
                texture,
                view,
                width: mip_width,
                height: mip_height,
            });

            mip_width = (mip_width / 2).max(1);
            mip_height = (mip_height / 2).max(1);
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        hdr_view: &wgpu::TextureView,
    ) -> &wgpu::TextureView {
        // === DOWNSAMPLE PASS ===
        // Pass 0: HDR → Mip0 (with prefilter/threshold)
        {
            let source_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Downsample Source (HDR)"),
                layout: &self.source_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(hdr_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Downsample Prefilter Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mips[0].view,
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

            render_pass.set_pipeline(&self.downsample_prefilter_pipeline);
            render_pass.set_bind_group(0, &source_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniforms_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Pass 1-4: Mip0 → Mip1 → Mip2 → Mip3 → Mip4
        for i in 1..BLOOM_MIP_COUNT as usize {
            let source_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bloom Downsample Source (Mip {})", i - 1)),
                layout: &self.source_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.mips[i - 1].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Bloom Downsample Pass (Mip {})", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mips[i].view,
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

            render_pass.set_pipeline(&self.downsample_pipeline);
            render_pass.set_bind_group(0, &source_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniforms_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // === UPSAMPLE PASS ===
        // First upsample: Mip4 → Mip3 (no addition)
        {
            let source_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bloom Upsample First Source (Mip 4)"),
                layout: &self.source_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.mips[4].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Upsample First Pass (Mip 4 → Mip 3)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mips[3].view,
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

            render_pass.set_pipeline(&self.upsample_first_pipeline);
            render_pass.set_bind_group(0, &source_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Upsample passes: Mip3 → Mip2 → Mip1 → Mip0
        for i in (0..3).rev() {
            let source_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bloom Upsample Source (Mip {})", i + 1)),
                layout: &self.source_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.mips[i + 1].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let add_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bloom Upsample Add (Mip {})", i)),
                layout: &self.add_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.mips[i].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Bloom Upsample Pass (Mip {} → Mip {})", i + 1, i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.mips[i].view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve existing content to add to
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.upsample_pipeline);
            render_pass.set_bind_group(0, &source_bind_group, &[]);
            render_pass.set_bind_group(1, &add_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Return final bloom result (Mip 0 at 1/2 resolution)
        &self.mips[0].view
    }
}
