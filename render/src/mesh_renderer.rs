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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshInstance {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

impl MeshInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model Matrix (vec4 x 4) - Locations 5-8
                wgpu::VertexAttribute { offset: 0,  shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 16, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 32, shader_location: 7, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { offset: 48, shader_location: 8, format: wgpu::VertexFormat::Float32x4 },
                // Color - Location 9
                wgpu::VertexAttribute { offset: 64, shader_location: 9, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
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
                // &object_layout, // Removed: Instancing uses attributes now
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pbr_shader,
                entry_point: Some("vs_main"),
                buffers: &[ModelVertex::desc(), MeshInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pbr_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            cache: None,
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
                entry_point: Some("vs_main"),
                buffers: &[ModelVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &toon_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            cache: None,
            multiview: None,
        });

        // Outline Pipeline (Inverted Hull)
        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Toon Outline Pipeline"),
            layout: Some(&toon_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &toon_shader,
                entry_point: Some("vs_outline"),
                buffers: &[ModelVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &toon_shader,
                entry_point: Some("fs_outline"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            cache: None,
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

    pub fn render_instanced<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a Mesh,
        material_bind_group: &'a wgpu::BindGroup,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        instance_buffer: &'a wgpu::Buffer,
        instance_count: u32,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, light_bind_group, &[]);
        render_pass.set_bind_group(2, material_bind_group, &[]);
        
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..instance_count);
    }

    pub fn create_instance_buffer(
        &self,
        device: &wgpu::Device,
        instances: &[MeshInstance],
    ) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    // Legacy method - kept to avoid breaking engine compilation immediately, but will panic or misrender if used with new shader
    pub fn render_pbr<'a>(
        &'a self,
        _render_pass: &mut wgpu::RenderPass<'a>,
        _mesh: &'a Mesh,
        _material_bind_group: &'a wgpu::BindGroup,
        _camera_bind_group: &'a wgpu::BindGroup,
        _light_bind_group: &'a wgpu::BindGroup,
        _object_bind_group: &'a wgpu::BindGroup,
    ) {
        panic!("render_pbr is deprecated using ObjectUniform. Use render_instanced instead.");
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

    pub fn create_pbr_bind_group(
        &self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        material: &PbrMaterial, 
        texture_manager: &TextureManager
    ) -> wgpu::BindGroup {
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

       // Retrieve default textures (requires mutable access to TextureManager generally, but here we take immutable reference)
       // The TextureManager needs to have defaults initialized internally beforehand, 
       // OR we need inner mutability. 
       // Check TextureManager::get_normal_texture signature: it takes &mut self.
       // Here we have &TextureManager (immutable).
       // This is a problem if they aren't created yet.
       // However, render_system.rs explicitly calls get_white_texture/get_normal_texture at the start of render_game_world.
       // So they SHOULD exist. 
       // We can use get_texture("default_normal") which takes &self.
       
       let white_dummy_opt = texture_manager.get_texture("default_white");
       let normal_dummy_opt = texture_manager.get_texture("default_normal");
       
       // Fallback to a panic if defaults are missing is safer than rendering garbage, 
       // but strictly speaking render_system ensures they exist.
       // If they don't, we might crash on unwrap, but let's try to be safe.
       
       // Note with immutable TextureManager we can't create them if missing.
       let white_dummy = white_dummy_opt.or(normal_dummy_opt).expect("Critical: Default textures missing!");
       let normal_dummy = normal_dummy_opt.expect("Critical: Default normal texture missing!");

       let albedo_view = material.albedo_texture.as_ref().map(|t| &t.view).unwrap_or(&white_dummy.view);
       let albedo_sampler = material.albedo_texture.as_ref().map(|t| &t.sampler).unwrap_or(&white_dummy.sampler);
       
       let normal_view = material.normal_texture.as_ref().map(|t| &t.view).unwrap_or(&normal_dummy.view);
       let normal_sampler = material.normal_texture.as_ref().map(|t| &t.sampler).unwrap_or(&normal_dummy.sampler);

       let metal_view = material.metallic_roughness_texture.as_ref().map(|t| &t.view).unwrap_or(&white_dummy.view);
       let metal_sampler = material.metallic_roughness_texture.as_ref().map(|t| &t.sampler).unwrap_or(&white_dummy.sampler);
       
       let occlusion_view = material.occlusion_texture.as_ref().map(|t| &t.view).unwrap_or(&white_dummy.view);
       let occlusion_sampler = material.occlusion_texture.as_ref().map(|t| &t.sampler).unwrap_or(&white_dummy.sampler);


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
