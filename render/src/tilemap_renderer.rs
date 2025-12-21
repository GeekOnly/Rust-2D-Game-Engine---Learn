use wgpu::util::DeviceExt;
use crate::texture::Texture;
use crate::sprite_renderer::Vertex;
use ecs::{Tilemap, TileSet};

pub struct TilemapRenderer {
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
}

impl TilemapRenderer {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tilemap_shader.wgsl").into()),
        });

        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tilemap Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout, // Group 0: Texture
                camera_bind_group_layout,   // Group 1: Camera
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Render Pipeline"),
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
                cull_mode: None, // Disable culling for 2D tilemaps (Winding order safety)
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Standard Z (Matches BatchRenderer)
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 0, 
                    slope_scale: 0.0, 
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
            bind_group_layout: texture_bind_group_layout,
        }
    }

    pub fn prepare_mesh(
        &self,
        device: &wgpu::Device,
        tilemap: &Tilemap,
        tileset: &TileSet,
        transform_pos: glam::Vec3,
        pixels_per_unit: f32,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_count = 0;

        // Visual Size in World Units
        let world_tile_width = tileset.tile_width as f32 / pixels_per_unit;
        let world_tile_height = tileset.tile_height as f32 / pixels_per_unit;

        // Texture Size in Pixels (for UVs)
        let tile_width_px = tileset.tile_width as f32;
        let tile_height_px = tileset.tile_height as f32;
        let tex_width = (tileset.columns * tileset.tile_width) as f32;
        let tex_height = (tileset.tile_count as f32 / tileset.columns as f32).ceil() * tileset.tile_height as f32;
        
        // We need texture dimensions to calculate UVs correctly
        // Assuming tileset.columns and tileset.tile_count are correct
        let _cols = tileset.columns;
        
        for (i, tile) in tilemap.tiles.iter().enumerate() {
            // Skip empty tiles
            if tile.is_empty() {
                continue;
            }

            let x_idx = (i as u32) % tilemap.width;
            let y_idx = (i as u32) / tilemap.width;
            
            // Calculate world position relative to Tilemap Origin
            // Note: y_idx increases downwards. In 2D World Space (Y-Up), this means decreasing Y.
            // But we typically want the Tilemap Anchor to be Top-Left.
            // transform_pos is the Anchor.
            let x = transform_pos.x + (x_idx as f32 * world_tile_width);
            let y = transform_pos.y - (y_idx as f32 * world_tile_height);
            let z = transform_pos.z;

            // Calculate UVs for the tile
            if let Some((tx, ty)) = tileset.get_tile_coords(tile.tile_id) {
                // Calculate UV coordinates
                let u0 = tx as f32 / tex_width;
                let v0 = ty as f32 / tex_height;
                let u1 = (tx as f32 + tile_width_px) / tex_width;
                let v1 = (ty as f32 + tile_height_px) / tex_height;

                // Handle flip flags
                let (u0, u1) = if tile.flip_h { (u1, u0) } else { (u0, u1) };
                let (v0, v1) = if tile.flip_v { (v1, v0) } else { (v0, v1) };

                // Add vertices
                let start_idx = vertices.len() as u16;
                
                // Quad vertices (x, y, z, u, v)
                // Top Left (Anchor)
                vertices.push(Vertex { position: [x, y, z], tex_coords: [u0, v0] });
                // Top Right
                vertices.push(Vertex { position: [x + world_tile_width, y, z], tex_coords: [u1, v0] });
                // Bottom Right
                vertices.push(Vertex { position: [x + world_tile_width, y - world_tile_height, z], tex_coords: [u1, v1] });
                // Bottom Left
                vertices.push(Vertex { position: [x, y - world_tile_height, z], tex_coords: [u0, v1] });

                // Add indices (two triangles)
                indices.push(start_idx);
                indices.push(start_idx + 1);
                indices.push(start_idx + 2);
                indices.push(start_idx);
                indices.push(start_idx + 2);
                indices.push(start_idx + 3);

                index_count += 6;
            }
        }

        // Check if mesh was generated
        if vertices.len() == 0 {
             println!("DEBUG: Tilemap Mesh Empty! Tile Count: {}", tilemap.tiles.len());
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer, index_count)
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        vertex_buffer: &'a wgpu::Buffer,
        index_buffer: &'a wgpu::Buffer,
        index_count: u32,
        texture: &'a Texture,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if index_count == 0 {
            return;
        }

        if let Some(bind_group) = &texture.bind_group {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        } else {
             // Only log once per frame/texture ideally, but for now strict debug
             println!("DEBUG: ERROR! Texture has no BindGroup. Cannot render tilemap.");
        }
    }
}
