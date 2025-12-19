use wgpu::util::DeviceExt;
use crate::texture::Texture;
use ecs::{Tilemap, TileSet};
use std::collections::HashMap;

// Uniform structs matching the shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TilemapUniform {
    transform: [[f32; 4]; 4],
    map_size: [u32; 2],
    tile_size: [f32; 2],
    padding: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TilesetUniform {
    sheet_size: [u32; 2],
    tile_count: u32,
    padding: u32,
}

pub struct TilemapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    group1_layout: wgpu::BindGroupLayout,
    group2_layout: wgpu::BindGroupLayout,
    
    // Cache for GPU resources
    // Key: Tilemap Entity ID? Or just re-upload when needed?
    // For now, let's just create resources on the fly or return them.
    // Ideally we return a struct that holds the BindGroups and Buffers.
}

pub struct TilemapGpuResources {
    pub group1: wgpu::BindGroup,
    pub group2: wgpu::BindGroup,
    pub index_count: u32, // Always 6 vertices (1 quad)
}

impl TilemapRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tilemap_gpu.wgsl").into()),
        });

        // Group 1: Tilemap (Uniform + Index Texture)
        let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tilemap Group 1 Layout"),
            entries: &[
                // TilemapUniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Index Texture (R8Uint or R32Uint)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // Group 2: Tileset (Texture Array + Sampler)
        let group2_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tilemap Group 2 Layout"),
            entries: &[
                // Texture Array
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GPU Tilemap Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout, // Group 0
                &group1_layout,          // Group 1
                &group2_layout,          // Group 2
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GPU Tilemap Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffers! using vertex_index
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
                cull_mode: None, // Don't cull for now, or Back if winding correct
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater, // Reverse-Z
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

        Self {
            render_pipeline,
            group1_layout,
            group2_layout,
        }
    }

    pub fn prepare_gpu_data(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tilemap: &Tilemap,
        tileset: &TileSet,
        tileset_texture: &Texture, // The source image
    ) -> Option<TilemapGpuResources> {
        // 1. Create Tileset Array Texture
        // We need to slice the tileset texture into layers.
        // This is complex: need to copy from src texture to dst array texture.
        // Assume tileset_texture describes the full sheet.
        
        let tile_w = tileset.tile_width;
        let tile_h = tileset.tile_height;
        let cols = tileset.columns;
        let rows = tileset.tile_count / cols + if tileset.tile_count % cols > 0 { 1 } else { 0 };
        
        // Ensure we have a valid texture
        let src_texture = &tileset_texture.texture;
        
        let array_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Tileset Array Texture"),
            size: wgpu::Extent3d {
                width: tile_w,
                height: tile_h,
                depth_or_array_layers: tileset.tile_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // Assuming SRGB
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Copy each tile from sheet to array layer
        for i in 0..tileset.tile_count {
            let tx = i % cols;
            let ty = i / cols;
            
            // Calculate origin in source sheet
            let origin_x = tx * tile_w;
            let origin_y = ty * tile_h;
            
            // Copy command
            let cmd_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            // Wait, we can use queue.copy_texture_to_texture? No, queue only has write_texture.
            // But we can use command_encoder.copy_texture_to_texture.
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Tile Copy Encoder") });
            
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: src_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: origin_x, y: origin_y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &array_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: tile_w,
                    height: tile_h,
                    depth_or_array_layers: 1,
                }
            );
            queue.submit(Some(encoder.finish()));
        }

        let array_view = array_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tilemap Group 2"),
            layout: &self.group2_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // 2. Create Index Texture
        // This holds the tile IDs.
        // Needs R32Uint if count > 255, else R8Uint.
        // using index_texture: texture_2d<u32> in shader implies we can use R16Uint or R32Uint.
        // Let's safe bet R32Uint for now unless size concern.
        
        let map_width = tilemap.width;
        let map_height = tilemap.height; // Assuming height is known or calculated from tile count
        
        // Generate index data
        let mut index_data = Vec::with_capacity((map_width * map_height) as usize * 4);
        for tile in &tilemap.tiles {
            // tile_id comes from tile.tile_id
            index_data.extend_from_slice(&tile.tile_id.to_le_bytes()); // u32 to bytes
        }
        // Pad if needed?
        
        let index_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Tilemap Index Texture"),
            size: wgpu::Extent3d {
                width: map_width,
                height: map_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        queue.write_texture(
             wgpu::ImageCopyTexture {
                texture: &index_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &index_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(map_width * 4), // 4 bytes per u32
                rows_per_image: Some(map_height),
            },
            wgpu::Extent3d {
                width: map_width,
                height: map_height,
                depth_or_array_layers: 1,
            }
        );

        let index_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create Uniform Buffer
        // We construct a simple transform matrix: Scale * Translation
        // Scale = [map_width * tile_w, map_height * tile_h, 1]
        // This maps the 0..1 quad to full map size in pixels.
        // Translation = [0, 0, 0] (or tilemap position if valid)
        
        let total_w = (map_width * tile_w) as f32;
        let total_h = (map_height * tile_h) as f32;
        
        // Column-major matrix
        // Scale(total_w, total_h, 1.0)
        let transform = [
            [total_w, 0.0, 0.0, 0.0],
            [0.0, total_h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let uniform = TilemapUniform {
            transform, 
            map_size: [map_width, map_height],
            tile_size: [tile_w as f32, tile_h as f32],
            padding: [0, 0],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tilemap Group 1"),
            layout: &self.group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&index_view),
                },
            ],
        });

        Some(TilemapGpuResources {
            group1,
            group2,
            index_count: 6,
        })
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a TilemapGpuResources,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &resources.group1, &[]);
        render_pass.set_bind_group(2, &resources.group2, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
