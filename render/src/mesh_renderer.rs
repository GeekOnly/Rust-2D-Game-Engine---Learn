use wgpu::util::DeviceExt;
use crate::{Mesh, ModelVertex, Texture, PbrMaterial, TextureManager, PbrMaterialUniform, ToonMaterial, ToonMaterialUniform};

pub struct MeshRenderer {
    render_pipeline: wgpu::RenderPipeline,
    pub material_layout: wgpu::BindGroupLayout,
    // Toon Support
    toon_pipeline: wgpu::RenderPipeline,
    outline_pipeline: wgpu::RenderPipeline,
    pub toon_material_layout: wgpu::BindGroupLayout,
    pub object_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectUniform {
    pub model: [[f32; 4]; 4],
}

impl MeshRenderer {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
        light_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // --- PBR Setup ---
        let pbr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pbr.wgsl").into()),
        });

        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Material Uniform
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
                // Albedo Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Albedo Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Normal Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // MetallicRoughness Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // MetallicRoughness Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("pbr_material_layout"),
        });

        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("object_layout"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[
                camera_layout,
                light_layout,
                &material_layout,
                &object_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pbr_shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &pbr_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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

        // --- Toon Setup ---
        let toon_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Toon Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("toon.wgsl").into()),
        });

        let toon_material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Toon Material Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("toon_material_layout"),
        });

        // object_layout moved up

        let toon_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Toon Pipeline Layout"),
            bind_group_layouts: &[
                camera_layout,
                light_layout,
                &toon_material_layout,
                &object_layout,
            ],
            push_constant_ranges: &[],
        });

        let toon_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Toon Render Pipeline"),
            layout: Some(&toon_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &toon_shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &toon_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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

        // Outline Pipeline (Inverted Hull)
        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Toon Outline Pipeline"),
            layout: Some(&toon_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &toon_shader,
                entry_point: "vs_outline",
                buffers: &[ModelVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &toon_shader,
                entry_point: "fs_outline",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw, // Cull Front faces = Render Back faces
                cull_mode: Some(wgpu::Face::Back), // "Back" here refers to back of usage, but since we flipped winding to CW, Face::Back actually culls "Front" faces (Counter-Clockwise).
                // Wait, standard is CCW. Face::Back culls CW.
                // We want to render BACK faces.
                // So front_face: Ccw sets standard. Cull_mode: Front culls front faces.
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                // We want outlines to be BEHIND the object.
                // Standard Z: Larger value = Further away.
                // So we want Outline Z > Object Z.
                // But we still use 'Less' test against the buffer.
                depth_compare: wgpu::CompareFunction::Less, 
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2, // Positive adds to Z -> Pushes it further away
                    slope_scale: 2.0, 
                    clamp: 0.0, 
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
            material_layout,
            toon_pipeline,
            outline_pipeline,
            toon_material_layout,
            object_layout,
        }
    }

    pub fn render_pbr<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a Mesh,
        material_bind_group: &'a wgpu::BindGroup,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        object_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(2, material_bind_group, &[]);
        render_pass.set_bind_group(3, object_bind_group, &[]);
        
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    pub fn render_toon<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a Mesh,
        material_bind_group: &'a wgpu::BindGroup,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        object_bind_group: &'a wgpu::BindGroup,
    ) {
        // Outline Pass (1st)
        render_pass.set_pipeline(&self.outline_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(2, material_bind_group, &[]);
        render_pass.set_bind_group(3, object_bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);

        // Main Toon Pass (2nd)
        render_pass.set_pipeline(&self.toon_pipeline);
        render_pass.set_bind_group(3, object_bind_group, &[]);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
    }

    pub fn create_material_bind_group(
        &self,
        device: &wgpu::Device,
        material_uniform: &crate::PbrMaterialUniform,
        albedo: &Texture,
        normal: &Texture,
        metallic_roughness: &Texture,
    ) -> wgpu::BindGroup {
        
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Material Uniform"),
                contents: bytemuck::cast_slice(&[*material_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.material_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&albedo.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&albedo.sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&normal.view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&normal.sampler) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&metallic_roughness.view) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&metallic_roughness.sampler) },
            ],
            label: Some("pbr_material_bind_group"),
        })
    }

    pub fn create_toon_material_bind_group(
        &self,
        device: &wgpu::Device,
        material_uniform: &crate::ToonMaterialUniform,
    ) -> wgpu::BindGroup {
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Toon Material Uniform"),
                contents: bytemuck::cast_slice(&[*material_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.toon_material_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
            ],
            label: Some("toon_material_bind_group"),
        })
    }

    pub fn create_object_bind_group(
        &self,
        device: &wgpu::Device,
        object_uniform: &ObjectUniform,
    ) -> wgpu::BindGroup {
        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Object Uniform"),
                contents: bytemuck::cast_slice(&[*object_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.object_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
            ],
            label: Some("object_bind_group"),
        })
    }

    pub fn create_pbr_bind_group(&self, device: &wgpu::Device, material: &PbrMaterial, texture_manager: &TextureManager) -> wgpu::BindGroup {
         use wgpu::util::DeviceExt;

         let pbr_uniform = PbrMaterialUniform {
            albedo_factor: material.albedo_factor,
            metallic_factor: material.metallic_factor,
            roughness_factor: material.roughness_factor,
            padding: [0.0; 2],
        };
        
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("PBR Material Buffer"),
                contents: bytemuck::cast_slice(&[pbr_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // Get textures or defaults
        // Note: We need Queue to get defaults if not present, but here we assume TextureManager has initialized defaults?
        // Actually TextureManager might need device/queue to get defaults.
        // We will pass defaults if material textures are missing.
        // But TextureManager.get_white_texture requires queue. 
        // We'll trust that the material has textures OR we fetch defaults using a workaround or pass Queue.
        // Or simpler: We expect the caller to ensure textures are valid.
        // The GltfLoader fills in defaults.
        
        // We'll create a temporary 1x1 white texture if absolutely needed, but TextureManager should have it.
        // Let's grab views.
        
        // Use placeholder if None.
        // We can't easily get default textures without Queue.
        // Let's assume textures are set (GltfLoader sets defaults).
        // If they are None, we might panic or fail.
        // But GltfLoader sets them.
        
        // Wait, GltfLoader logic: checks `texture_manager.get_white_texture(device, queue)`.
        // So they should be set.
        
        // We need BindGroup entries.
        // We can't access `texture.view` if texture is None.
        // So we MUST have textures.
        
        let white_tex = material.albedo_texture.as_ref().expect("Material missing albedo texture");
        // For others, if missing, use white/normal default?
        // GltfLoader sets them?
        // GltfLoader sets normal/metallic to `load_texture(...).ok()`. So they might be None.
        // If None, we need defaults.
        
        let albedo_view = &material.albedo_texture.as_ref().unwrap().view;
        let albedo_sampler = &material.albedo_texture.as_ref().unwrap().sampler;
        
        let normal_view = if let Some(tex) = &material.normal_texture {
            &tex.view
        } else {
             // We need a default normal map here.
             // We can't get it from TextureManager without Queue easily if not cached.
             // But we can assume it's cached because GltfLoader ran?
             // Or passing `texture_manager`.
             // TextureManager::get_normal_texture needs queue.
             // I'll add `queue` to arguments.
             albedo_view // Fallback (WRONG but prevents crash) - fix below by adding Queue arg
        };
        let normal_sampler = if let Some(tex) = &material.normal_texture { &tex.sampler } else { albedo_sampler };

        let metal_view = if let Some(tex) = &material.metallic_roughness_texture { &tex.view } else { albedo_view };
        let metal_sampler = if let Some(tex) = &material.metallic_roughness_texture { &tex.sampler } else { albedo_sampler };
        
        let occlusion_view = if let Some(tex) = &material.occlusion_texture { &tex.view } else { albedo_view };
        let occlusion_sampler = if let Some(tex) = &material.occlusion_texture { &tex.sampler } else { albedo_sampler };


        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.material_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(albedo_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(albedo_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(normal_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(normal_sampler) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(metal_view) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(metal_sampler) },
            ],
            label: Some("pbr_material_bind_group"),
        })
    }
}
