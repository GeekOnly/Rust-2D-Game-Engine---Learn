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
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tilemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("simple_sprite_shader.wgsl").into()), // Use simple shader
        });

        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tilemap Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
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
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
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
            bind_group_layout: texture_bind_group_layout,
        }
    }

    pub fn prepare_mesh(
        &self,
        device: &wgpu::Device,
        tilemap: &Tilemap,
        tileset: &TileSet,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_count = 0;

        let tile_width = tileset.tile_width as f32;
        let tile_height = tileset.tile_height as f32;
        // Calculate texture width from columns and tile width
        let tex_width = (tileset.columns * tileset.tile_width) as f32;
        
        // We need texture dimensions to calculate UVs correctly
        // Assuming tileset.columns and tileset.tile_count are correct
        let cols = tileset.columns;
        
        // UV size for one tile
        // Note: This assumes no spacing/margin for simplicity. 
        // Real implementation should handle spacing/margin.
        let _uv_w = 1.0 / cols as f32;
        let _uv_h = 1.0 / (tileset.tile_count as f32 / cols as f32).ceil(); 

        for (i, tile) in tilemap.tiles.iter().enumerate() {
            // Skip empty tiles
            if tile.is_empty() {
                continue;
            }

            let x_idx = (i as u32) % tilemap.width;
            let y_idx = (i as u32) / tilemap.width;
            
            let x = x_idx as f32 * tile_width;
            let y = y_idx as f32 * tile_height;

            // Calculate UVs for the tile
            if let Some((tx, ty)) = tileset.get_tile_coords(tile.tile_id) {
                // Calculate UV coordinates
                let u0 = tx as f32 / tex_width;
                let v0 = ty as f32 / tex_width; // Assuming square texture
                let u1 = u0 + (tile_width / tex_width);
                let v1 = v0 + (tile_height / tex_width);

                // Handle flip flags
                let (u0, u1) = if tile.flip_h { (u1, u0) } else { (u0, u1) };
                let (v0, v1) = if tile.flip_v { (v1, v0) } else { (v0, v1) };

                // Add vertices
                let start_idx = vertices.len() as u16;
                
                // Quad vertices (x, y, z, u, v)
                // Top Left
                vertices.push(Vertex { position: [x, y, 0.0], tex_coords: [u0, v0] });
                // Top Right
                vertices.push(Vertex { position: [x + tile_width, y, 0.0], tex_coords: [u1, v0] });
                // Bottom Right
                vertices.push(Vertex { position: [x + tile_width, y + tile_height, 0.0], tex_coords: [u1, v1] });
                // Bottom Left
                vertices.push(Vertex { position: [x, y + tile_height, 0.0], tex_coords: [u0, v1] });

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
        _device: &wgpu::Device,
    ) {
        if let Some(bind_group) = &texture.bind_group {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
}
