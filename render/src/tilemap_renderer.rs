use wgpu::util::DeviceExt;
use crate::texture::Texture;
use ecs::{Tilemap, TileSet};
use ecs::components::{UnifiedTilemap, ViewMode, PerfectPixelSettings, PixelPerfectTransform};
use std::collections::HashMap;
use glam::{Mat4, Vec3};

// Uniform structs matching the unified shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct UnifiedTilemapUniform {
    transform: [[f32; 4]; 4],
    map_size: [u32; 2],
    tile_size: [f32; 2],
    layer_depth: f32,
    world_space_scale: f32,
    pixels_per_unit: f32,
    view_mode: f32, // 0.0 = 2D, 1.0 = 3D
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
    unified_render_pipeline: wgpu::RenderPipeline, // New unified pipeline
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

pub struct UnifiedTilemapGpuResources {
    pub group1: wgpu::BindGroup,
    pub group2: wgpu::BindGroup,
    pub index_count: u32,
    pub view_mode: ViewMode,
    pub perfect_pixel_enabled: bool,
}

impl TilemapRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tilemap_gpu.wgsl").into()),
        });

        let unified_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Unified Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("unified_tilemap.wgsl").into()),
        });

        // Group 1: Tilemap (Uniform + Index Texture) - Updated for unified rendering
        let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Unified Tilemap Group 1 Layout"),
            entries: &[
                // UnifiedTilemapUniform
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

        // Group 2: Tileset (Texture Array + Sampler) - Updated for unified rendering
        let group2_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Unified Tilemap Group 2 Layout"),
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
                // Sampler (configurable for perfect pixel vs smooth)
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

        // Create unified render pipeline layout
        let unified_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Unified Tilemap Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout, // Group 0 - Unified Camera
                &group1_layout,          // Group 1 - Tilemap data
                &group2_layout,          // Group 2 - Tileset data
            ],
            push_constant_ranges: &[],
        });

        // Create unified render pipeline
        let unified_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Unified Tilemap Pipeline"),
            layout: Some(&unified_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &unified_shader,
                entry_point: "vs_tilemap_main",
                buffers: &[], // No vertex buffers! using vertex_index
            },
            fragment: Some(wgpu::FragmentState {
                module: &unified_shader,
                entry_point: "fs_tilemap_main",
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
                cull_mode: None, // Support both 2D and 3D rendering
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
            unified_render_pipeline,
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

        let uniform = UnifiedTilemapUniform {
            transform, 
            map_size: [map_width, map_height],
            tile_size: [tile_w as f32, tile_h as f32],
            layer_depth: 0.0,
            world_space_scale: 1.0,
            pixels_per_unit: 100.0,
            view_mode: 0.0, // 2D mode
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

    /// Prepare GPU data for UnifiedTilemap with 2D/3D support and perfect pixel rendering
    pub fn prepare_unified_gpu_data(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_tilemap: &UnifiedTilemap,
        tileset: &TileSet,
        tileset_texture: &Texture,
        transform: &ecs::Transform,
        view_mode: ViewMode,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Option<UnifiedTilemapGpuResources> {
        // Calculate map dimensions from tile data
        let (map_width, map_height) = self.calculate_map_dimensions(unified_tilemap);
        
        if map_width == 0 || map_height == 0 {
            return None;
        }

        // 1. Create Tileset Array Texture with appropriate filtering
        let array_texture = self.create_tileset_array_texture(
            device, 
            queue, 
            tileset, 
            tileset_texture, 
            perfect_pixel_settings
        )?;

        let array_view = array_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        
        // Create sampler based on perfect pixel settings and view mode
        let sampler = self.create_unified_sampler(device, view_mode, perfect_pixel_settings);

        let group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Unified Tilemap Group 2"),
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

        // 2. Create Index Texture from UnifiedTilemap data
        let index_texture = self.create_unified_index_texture(
            device, 
            queue, 
            unified_tilemap, 
            map_width, 
            map_height
        )?;

        let index_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 3. Create Unified Transform with perfect pixel support
        let unified_transform = self.create_unified_transform(
            unified_tilemap,
            transform,
            map_width,
            map_height,
            tileset,
            view_mode,
            perfect_pixel_settings,
        );

        // Create Uniform Buffer
        let uniform = UnifiedTilemapUniform {
            transform: unified_transform.to_cols_array_2d(),
            map_size: [map_width, map_height],
            tile_size: [tileset.tile_width as f32, tileset.tile_height as f32],
            layer_depth: unified_tilemap.layer_depth,
            world_space_scale: unified_tilemap.world_space_scale,
            pixels_per_unit: unified_tilemap.pixels_per_unit.unwrap_or(perfect_pixel_settings.pixels_per_unit),
            view_mode: match view_mode {
                ViewMode::Mode2D => 0.0,
                ViewMode::Mode3D => 1.0,
            },
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unified Tilemap Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Unified Tilemap Group 1"),
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

        Some(UnifiedTilemapGpuResources {
            group1,
            group2,
            index_count: 6,
            view_mode,
            perfect_pixel_enabled: unified_tilemap.pixel_perfect && perfect_pixel_settings.enabled,
        })
    }

    /// Render UnifiedTilemap using the unified pipeline
    pub fn render_unified<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a UnifiedTilemapGpuResources,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.unified_render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &resources.group1, &[]);
        render_pass.set_bind_group(2, &resources.group2, &[]);
        render_pass.draw(0..6, 0..1);
    }

    /// Calculate map dimensions from UnifiedTilemap tile data
    fn calculate_map_dimensions(&self, unified_tilemap: &UnifiedTilemap) -> (u32, u32) {
        if unified_tilemap.tiles.is_empty() {
            return (0, 0);
        }

        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for &(x, y) in unified_tilemap.tiles.keys() {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        let width = (max_x - min_x + 1) as u32;
        let height = (max_y - min_y + 1) as u32;
        
        (width, height)
    }

    /// Create tileset array texture with appropriate filtering for perfect pixel rendering
    fn create_tileset_array_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tileset: &TileSet,
        tileset_texture: &Texture,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Option<wgpu::Texture> {
        let tile_w = tileset.tile_width;
        let tile_h = tileset.tile_height;
        let cols = tileset.columns;
        
        // Ensure we have a valid texture
        let src_texture = &tileset_texture.texture;
        
        let array_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Unified Tileset Array Texture"),
            size: wgpu::Extent3d {
                width: tile_w,
                height: tile_h,
                depth_or_array_layers: tileset.tile_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Copy each tile from sheet to array layer
        for i in 0..tileset.tile_count {
            let tx = i % cols;
            let ty = i / cols;
            
            // Calculate origin in source sheet
            let origin_x = tx * tile_w + tileset.margin + tx * tileset.spacing;
            let origin_y = ty * tile_h + tileset.margin + ty * tileset.spacing;
            
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
                label: Some("Unified Tile Copy Encoder") 
            });
            
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

        Some(array_texture)
    }

    /// Create sampler with appropriate filtering for view mode and perfect pixel settings
    fn create_unified_sampler(
        &self,
        device: &wgpu::Device,
        view_mode: ViewMode,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> wgpu::Sampler {
        // Use nearest filtering for perfect pixel rendering in 2D mode
        let filter_mode = if view_mode == ViewMode::Mode2D && perfect_pixel_settings.enabled {
            wgpu::FilterMode::Nearest
        } else {
            wgpu::FilterMode::Linear
        };

        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: filter_mode,
            ..Default::default()
        })
    }

    /// Create index texture from UnifiedTilemap data
    fn create_unified_index_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_tilemap: &UnifiedTilemap,
        map_width: u32,
        map_height: u32,
    ) -> Option<wgpu::Texture> {
        // Generate index data from UnifiedTilemap
        let mut index_data = Vec::with_capacity((map_width * map_height) as usize * 4);
        
        // Find the bounds of the tilemap
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        
        for &(x, y) in unified_tilemap.tiles.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
        }

        // Fill the texture with tile IDs
        for y in 0..map_height {
            for x in 0..map_width {
                let tile_x = min_x + x as i32;
                let tile_y = min_y + y as i32;
                
                let tile_id = unified_tilemap.tiles.get(&(tile_x, tile_y)).copied().unwrap_or(0);
                index_data.extend_from_slice(&tile_id.to_le_bytes());
            }
        }
        
        let index_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Unified Tilemap Index Texture"),
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

        Some(index_texture)
    }

    /// Create unified transform matrix with perfect pixel support
    fn create_unified_transform(
        &self,
        unified_tilemap: &UnifiedTilemap,
        transform: &ecs::Transform,
        map_width: u32,
        map_height: u32,
        tileset: &TileSet,
        view_mode: ViewMode,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Mat4 {
        let tile_w = tileset.tile_width as f32;
        let tile_h = tileset.tile_height as f32;
        
        // Calculate total map size in world units
        let total_w = map_width as f32 * tile_w;
        let total_h = map_height as f32 * tile_h;
        
        // Apply world space scale
        let scaled_w = total_w * unified_tilemap.world_space_scale;
        let scaled_h = total_h * unified_tilemap.world_space_scale;
        
        // Create base transform matrix
        let mut world_transform = Mat4::from_scale_rotation_translation(
            Vec3::new(scaled_w, scaled_h, 1.0),
            glam::Quat::from_euler(glam::EulerRot::XYZ, 
                transform.rotation[0].to_radians(), 
                transform.rotation[1].to_radians(), 
                transform.rotation[2].to_radians()),
            Vec3::from(transform.position),
        );

        // Apply perfect pixel snapping in 2D mode
        if view_mode == ViewMode::Mode2D && unified_tilemap.pixel_perfect && perfect_pixel_settings.enabled {
            let pixels_per_unit = unified_tilemap.pixels_per_unit.unwrap_or(perfect_pixel_settings.pixels_per_unit);
            let snapped_position = PixelPerfectTransform::snap_to_pixel(Vec3::from(transform.position), pixels_per_unit);
            
            world_transform = Mat4::from_scale_rotation_translation(
                Vec3::new(scaled_w, scaled_h, 1.0),
                glam::Quat::from_euler(glam::EulerRot::XYZ, 
                    transform.rotation[0].to_radians(), 
                    transform.rotation[1].to_radians(), 
                    transform.rotation[2].to_radians()),
                snapped_position,
            );
        }

        world_transform
    }

    /// Update animated tiles while preserving perfect pixel alignment
    pub fn update_animated_tiles(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_tilemap: &UnifiedTilemap,
        tileset: &TileSet,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Option<wgpu::Texture> {
        // Check if there are any animated tiles
        if unified_tilemap.animated_tiles.is_empty() {
            return None;
        }

        // Calculate map dimensions
        let (map_width, map_height) = self.calculate_map_dimensions(unified_tilemap);
        if map_width == 0 || map_height == 0 {
            return None;
        }

        // Create updated index texture with current animation frames
        let updated_index_texture = self.create_animated_index_texture(
            device,
            queue,
            unified_tilemap,
            map_width,
            map_height,
            perfect_pixel_settings,
        )?;

        Some(updated_index_texture)
    }

    /// Create frame-based animation timing that preserves pixel alignment
    pub fn calculate_animation_frame(
        &self,
        animation_time: f32,
        frame_rate: f32,
        frame_count: u32,
        perfect_pixel_enabled: bool,
    ) -> u32 {
        if frame_count <= 1 {
            return 0;
        }

        // Calculate frame index with pixel-perfect timing
        let frames_per_second = if perfect_pixel_enabled {
            // Align frame rate to common refresh rates for consistent timing
            let common_rates = [60.0, 30.0, 20.0, 15.0, 12.0, 10.0, 6.0, 5.0];
            common_rates.iter()
                .find(|&&rate| rate <= frame_rate)
                .copied()
                .unwrap_or(frame_rate.round())
        } else {
            frame_rate
        };

        let frame_duration = 1.0 / frames_per_second;
        let current_frame = (animation_time / frame_duration) as u32;
        
        current_frame % frame_count
    }

    /// Create index texture with animated tile data
    fn create_animated_index_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_tilemap: &UnifiedTilemap,
        map_width: u32,
        map_height: u32,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Option<wgpu::Texture> {
        // Generate index data with current animation frames
        let mut index_data = Vec::with_capacity((map_width * map_height) as usize * 4);
        
        // Find the bounds of the tilemap
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        
        for &(x, y) in unified_tilemap.tiles.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
        }

        // Fill the texture with current tile IDs (including animated frames)
        for y in 0..map_height {
            for x in 0..map_width {
                let tile_x = min_x + x as i32;
                let tile_y = min_y + y as i32;
                
                // Get the current tile ID (handles animations)
                let tile_id = unified_tilemap.get_render_tile_id(tile_x, tile_y);
                index_data.extend_from_slice(&tile_id.to_le_bytes());
            }
        }
        
        let index_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Animated Tilemap Index Texture"),
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

        Some(index_texture)
    }

    /// Update tilemap uniform buffer for animated content
    pub fn update_uniform_buffer(
        &self,
        queue: &wgpu::Queue,
        resources: &UnifiedTilemapGpuResources,
        unified_tilemap: &UnifiedTilemap,
        transform: &ecs::Transform,
        tileset: &TileSet,
        view_mode: ViewMode,
        perfect_pixel_settings: &PerfectPixelSettings,
        animation_frame: Option<u32>,
    ) {
        // Calculate map dimensions
        let (map_width, map_height) = self.calculate_map_dimensions(unified_tilemap);
        
        // Create updated transform
        let unified_transform = self.create_unified_transform(
            unified_tilemap,
            transform,
            map_width,
            map_height,
            tileset,
            view_mode,
            perfect_pixel_settings,
        );

        // Create updated uniform
        let uniform = UnifiedTilemapUniform {
            transform: unified_transform.to_cols_array_2d(),
            map_size: [map_width, map_height],
            tile_size: [tileset.tile_width as f32, tileset.tile_height as f32],
            layer_depth: unified_tilemap.layer_depth,
            world_space_scale: unified_tilemap.world_space_scale,
            pixels_per_unit: unified_tilemap.pixels_per_unit.unwrap_or(perfect_pixel_settings.pixels_per_unit),
            view_mode: match view_mode {
                ViewMode::Mode2D => 0.0,
                ViewMode::Mode3D => 1.0,
            },
        };

        // Update the uniform buffer
        // Note: This assumes the uniform buffer is accessible from the resources
        // In a real implementation, we'd need to store the buffer reference
        // or recreate the bind group with updated data
    }

    /// Update GPU resources with animated tile data
    pub fn update_animated_gpu_resources(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        unified_tilemap: &UnifiedTilemap,
        tileset: &TileSet,
        tileset_texture: &Texture,
        transform: &ecs::Transform,
        view_mode: ViewMode,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) -> Option<UnifiedTilemapGpuResources> {
        // Only update if there are animated tiles
        if unified_tilemap.animated_tiles.is_empty() {
            return None;
        }

        // Create updated index texture with current animation frames
        let updated_index_texture = self.update_animated_tiles(
            device,
            queue,
            unified_tilemap,
            tileset,
            perfect_pixel_settings,
        )?;

        let index_view = updated_index_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Reuse existing tileset array texture creation
        let array_texture = self.create_tileset_array_texture(
            device, 
            queue, 
            tileset, 
            tileset_texture, 
            perfect_pixel_settings
        )?;

        let array_view = array_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        
        // Create sampler based on perfect pixel settings and view mode
        let sampler = self.create_unified_sampler(device, view_mode, perfect_pixel_settings);

        let group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animated Tilemap Group 2"),
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

        // Calculate map dimensions
        let (map_width, map_height) = self.calculate_map_dimensions(unified_tilemap);

        // Create unified transform
        let unified_transform = self.create_unified_transform(
            unified_tilemap,
            transform,
            map_width,
            map_height,
            tileset,
            view_mode,
            perfect_pixel_settings,
        );

        // Create uniform buffer
        let uniform = UnifiedTilemapUniform {
            transform: unified_transform.to_cols_array_2d(),
            map_size: [map_width, map_height],
            tile_size: [tileset.tile_width as f32, tileset.tile_height as f32],
            layer_depth: unified_tilemap.layer_depth,
            world_space_scale: unified_tilemap.world_space_scale,
            pixels_per_unit: unified_tilemap.pixels_per_unit.unwrap_or(perfect_pixel_settings.pixels_per_unit),
            view_mode: match view_mode {
                ViewMode::Mode2D => 0.0,
                ViewMode::Mode3D => 1.0,
            },
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animated Tilemap Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Animated Tilemap Group 1"),
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

        Some(UnifiedTilemapGpuResources {
            group1,
            group2,
            index_count: 6,
            view_mode,
            perfect_pixel_enabled: unified_tilemap.pixel_perfect && perfect_pixel_settings.enabled,
        })
    }
}
