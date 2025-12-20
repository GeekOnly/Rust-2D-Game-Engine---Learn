use wgpu::util::DeviceExt;
use crate::texture::Texture;
use glam::{Vec3, Mat4};

// Import ECS components and Transform
use ecs;
use ecs::components::unified_rendering::pixel_perfect_utils;

/// Unified vertex structure that matches the unified shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UnifiedVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub normal: [f32; 3],
}

impl UnifiedVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnifiedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Texture coordinates
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Legacy vertex structure for backward compatibility
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
    // Unified rendering pipeline
    pub unified_render_pipeline: wgpu::RenderPipeline,
    // Legacy pipeline for backward compatibility
    pub legacy_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    unified_vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    bind_group_layout: wgpu::BindGroupLayout,
    // Cache for custom sprite rect rendering
    custom_vertex_buffer: Option<wgpu::Buffer>,
    custom_unified_vertex_buffer: Option<wgpu::Buffer>,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        // Create legacy shader for backward compatibility
        let legacy_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Simple Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("simple_sprite_shader.wgsl").into()),
        });

        // Create unified shader for 2D/3D rendering
        let unified_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Unified Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("unified_shader.wgsl").into()),
        });

        let texture_bind_group_layout = Texture::create_bind_group_layout(device);

        // Legacy pipeline layout
        let legacy_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Legacy Sprite Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Unified pipeline layout (camera first, then texture)
        let unified_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Unified Sprite Render Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create legacy render pipeline
        let legacy_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Legacy Sprite Render Pipeline"),
            layout: Some(&legacy_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &legacy_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &legacy_shader,
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
                    constant: -1,
                    slope_scale: -1.0,
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

        // Create unified render pipeline
        let unified_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Unified Sprite Render Pipeline"),
            layout: Some(&unified_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &unified_shader,
                entry_point: "vs_sprite_2d",
                buffers: &[UnifiedVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &unified_shader,
                entry_point: "fs_sprite_2d",
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
                cull_mode: None, // Disable culling for billboards
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
                    constant: -1,
                    slope_scale: -1.0,
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

        // Basic quad vertices (legacy)
        let vertices = &[
            Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // Top Left
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // Bottom Left
            Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // Bottom Right
            Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // Top Right
        ];

        // Unified quad vertices with color and normal
        let unified_vertices = &[
            UnifiedVertex { 
                position: [-0.5, 0.5, 0.0], 
                tex_coords: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex { 
                position: [-0.5, -0.5, 0.0], 
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex { 
                position: [0.5, -0.5, 0.0], 
                tex_coords: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex { 
                position: [0.5, 0.5, 0.0], 
                tex_coords: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
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

        let unified_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unified Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(unified_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            unified_render_pipeline,
            legacy_render_pipeline,
            vertex_buffer,
            unified_vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            bind_group_layout: texture_bind_group_layout,
            custom_vertex_buffer: None,
            custom_unified_vertex_buffer: None,
        }
    }

    /// Legacy render method for backward compatibility
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &'a Texture,
        _device: &wgpu::Device,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if let Some(bind_group) = &texture.bind_group {
            render_pass.set_pipeline(&self.legacy_render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    /// Render UnifiedSprite with unified shader and perfect pixel positioning
    pub fn render_unified_sprite<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture: &'a Texture,
        vertex_buffer: &'a wgpu::Buffer,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if let Some(bind_group) = &texture.bind_group {
            render_pass.set_pipeline(&self.unified_render_pipeline);
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    /// Create vertex buffer for UnifiedSprite
    pub fn create_unified_sprite_buffer(
        &self,
        device: &wgpu::Device,
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        camera_position: Vec3,
    ) -> wgpu::Buffer {
        let vertices = self.calculate_sprite_vertices(
            sprite, 
            transform, 
            view_mode, 
            perfect_pixel_settings,
            camera_position
        );

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dynamic Unified Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    /// Calculate sprite vertices based on UnifiedSprite properties
    fn calculate_sprite_vertices(
        &self,
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        camera_position: Vec3,
    ) -> [UnifiedVertex; 4] {
        let half_width = sprite.width * 0.5;
        let half_height = sprite.height * 0.5;

        // Base quad positions
        let mut positions = [
            Vec3::new(-half_width, half_height, 0.0),   // Top Left
            Vec3::new(-half_width, -half_height, 0.0),  // Bottom Left
            Vec3::new(half_width, -half_height, 0.0),   // Bottom Right
            Vec3::new(half_width, half_height, 0.0),    // Top Right
        ];

        // Convert transform arrays to Vec3
        let transform_position = Vec3::from_array(transform.position);

        // Apply billboard transformation in 3D mode
        if view_mode == ecs::components::ViewMode::Mode3D && sprite.billboard {
            positions = self.apply_billboard_transform(positions, camera_position, transform_position);
        }

        // Apply transform matrix
        let transform_matrix = self.calculate_transform_matrix(transform, perfect_pixel_settings);
        for pos in &mut positions {
            let transformed = transform_matrix * pos.extend(1.0);
            *pos = transformed.truncate();
        }

        // Apply perfect pixel snapping if enabled
        if sprite.pixel_perfect && perfect_pixel_settings.enabled {
            let pixels_per_unit = sprite.pixels_per_unit.unwrap_or(perfect_pixel_settings.pixels_per_unit);
            for pos in &mut positions {
                *pos = ecs::components::PixelPerfectTransform::snap_to_pixel(*pos, pixels_per_unit);
            }
        }

        // Create vertices
        [
            UnifiedVertex {
                position: positions[0].into(),
                tex_coords: [0.0, 0.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: positions[1].into(),
                tex_coords: [0.0, 1.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: positions[2].into(),
                tex_coords: [1.0, 1.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: positions[3].into(),
                tex_coords: [1.0, 0.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
        ]
    }

    /// Apply billboard transformation to make sprite face camera
    fn apply_billboard_transform(
        &self,
        positions: [Vec3; 4],
        camera_position: Vec3,
        sprite_position: Vec3,
    ) -> [Vec3; 4] {
        // Calculate direction from sprite to camera
        let to_camera = (camera_position - sprite_position).normalize();
        
        // Create billboard rotation matrix (simplified - faces camera on Y axis)
        let right = Vec3::new(1.0, 0.0, 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let forward = -to_camera;
        
        // Apply billboard rotation to each position
        positions.map(|pos| {
            Vec3::new(
                pos.x * right.x + pos.y * up.x + pos.z * forward.x,
                pos.x * right.y + pos.y * up.y + pos.z * forward.y,
                pos.x * right.z + pos.y * up.z + pos.z * forward.z,
            )
        })
    }

    /// Calculate transform matrix with perfect pixel considerations
    fn calculate_transform_matrix(
        &self,
        transform: &ecs::Transform,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
    ) -> Mat4 {
        // Convert arrays to Vec3 and create rotation quaternion from Euler angles
        let position = Vec3::from_array(transform.position);
        let scale = Vec3::from_array(transform.scale);
        
        // Convert Euler angles (degrees) to quaternion
        let rotation_radians = Vec3::from_array(transform.rotation) * std::f32::consts::PI / 180.0;
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            rotation_radians.x,
            rotation_radians.y,
            rotation_radians.z,
        );

        let mut matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);

        // Apply perfect pixel snapping to the matrix if enabled
        if perfect_pixel_settings.enabled && perfect_pixel_settings.snap_to_pixel {
            matrix = pixel_perfect_utils::snap_matrix_to_pixels(
                matrix,
                perfect_pixel_settings.pixels_per_unit,
                perfect_pixel_settings.reference_resolution,
            );
        }

        matrix
    }

    /// Render sprite with custom sprite rect (Unity-style) - Legacy version
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

            render_pass.set_pipeline(&self.legacy_render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.custom_vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    /// Create vertex buffer for UnifiedSprite with custom sprite rect
    pub fn create_unified_sprite_buffer_with_rect(
        &self,
        device: &wgpu::Device,
        sprite_rect: [u32; 4],
        texture_size: [u32; 2],
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        camera_position: Vec3,
    ) -> wgpu::Buffer {
        // Calculate UV coordinates from sprite rect
        let u_min = sprite_rect[0] as f32 / texture_size[0] as f32;
        let v_min = sprite_rect[1] as f32 / texture_size[1] as f32;
        let u_max = (sprite_rect[0] + sprite_rect[2]) as f32 / texture_size[0] as f32;
        let v_max = (sprite_rect[1] + sprite_rect[3]) as f32 / texture_size[1] as f32;

        // Calculate vertices with custom UV coordinates
        let mut vertices = self.calculate_sprite_vertices(
            sprite, 
            transform, 
            view_mode, 
            perfect_pixel_settings,
            camera_position
        );

        // Update UV coordinates
        vertices[0].tex_coords = [u_min, v_min]; // Top Left
        vertices[1].tex_coords = [u_min, v_max]; // Bottom Left
        vertices[2].tex_coords = [u_max, v_max]; // Bottom Right
        vertices[3].tex_coords = [u_max, v_min]; // Top Right

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Custom Unified Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    /// Render UnifiedSprite with unified texture integration
    pub fn render_unified_sprite_integrated<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture_ref: &'a crate::unified_texture_integration::UnifiedTextureRef,
        vertex_buffer: &'a wgpu::Buffer,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if let Some(bind_group) = texture_ref.get_bind_group() {
            render_pass.set_pipeline(&self.unified_render_pipeline);
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.set_bind_group(1, bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    /// Get the bind group layout for texture binding
    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Check if a sprite should use nearest neighbor filtering for perfect pixels
    pub fn should_use_nearest_filter(
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
    ) -> bool {
        if !sprite.pixel_perfect || !perfect_pixel_settings.enabled {
            return false;
        }

        match perfect_pixel_settings.filter_mode {
            ecs::components::FilterMode::Nearest => true,
            ecs::components::FilterMode::Linear => {
                // Use nearest filtering for integer scales
                let scale = Vec3::from_array(transform.scale);
                ecs::components::PixelPerfectTransform::should_use_nearest_filter(scale)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::components::{UnifiedSprite, ViewMode, PerfectPixelSettings, FilterMode};
    use ecs::Transform;
    use glam::Vec3;

    /// Test that UnifiedVertex has the correct memory layout for the unified shader
    #[test]
    fn test_unified_vertex_layout() {
        let vertex = UnifiedVertex {
            position: [1.0, 2.0, 3.0],
            tex_coords: [0.5, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
        };

        // Verify the vertex can be converted to bytes (required for WGPU)
        let _bytes: &[u8] = bytemuck::cast_slice(&[vertex]);
        
        // Verify size matches expected layout
        assert_eq!(
            std::mem::size_of::<UnifiedVertex>(),
            std::mem::size_of::<[f32; 3]>() + // position
            std::mem::size_of::<[f32; 2]>() + // tex_coords
            std::mem::size_of::<[f32; 4]>() + // color
            std::mem::size_of::<[f32; 3]>()   // normal
        );
    }

    /// Test that UnifiedSprite default values are sensible
    #[test]
    fn test_unified_sprite_defaults() {
        let sprite = UnifiedSprite::default();
        
        assert_eq!(sprite.width, 1.0);
        assert_eq!(sprite.height, 1.0);
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]); // White
        assert!(!sprite.billboard); // Default to non-billboard
        assert!(sprite.pixel_perfect); // Default to pixel perfect
        assert_eq!(sprite.sort_order, 0);
        assert!(sprite.pixels_per_unit.is_none()); // Use global setting
    }

    /// Test perfect pixel settings for different use cases
    #[test]
    fn test_perfect_pixel_settings() {
        // Test pixel art settings
        let pixel_art = PerfectPixelSettings::pixel_art();
        assert!(pixel_art.enabled);
        assert!(pixel_art.snap_to_pixel);
        assert_eq!(pixel_art.filter_mode, FilterMode::Nearest);
        assert_eq!(pixel_art.pixels_per_unit, 100.0);

        // Test smooth 2D settings
        let smooth_2d = PerfectPixelSettings::smooth_2d();
        assert!(smooth_2d.enabled);
        assert!(!smooth_2d.snap_to_pixel);
        assert_eq!(smooth_2d.filter_mode, FilterMode::Linear);

        // Test disabled settings
        let disabled = PerfectPixelSettings::disabled();
        assert!(!disabled.enabled);
        assert!(!disabled.snap_to_pixel);
    }

    /// Test that sprite renderer filtering logic works correctly
    #[test]
    fn test_sprite_filtering_logic() {
        let sprite = UnifiedSprite {
            pixel_perfect: true,
            ..Default::default()
        };

        let transform = Transform {
            scale: [2.0, 2.0, 1.0], // Integer scale
            ..Default::default()
        };

        // Test with nearest filter mode
        let nearest_settings = PerfectPixelSettings {
            enabled: true,
            filter_mode: FilterMode::Nearest,
            ..Default::default()
        };

        assert!(SpriteRenderer::should_use_nearest_filter(
            &sprite,
            &transform,
            &nearest_settings
        ));

        // Test with linear filter mode but integer scale
        let linear_settings = PerfectPixelSettings {
            enabled: true,
            filter_mode: FilterMode::Linear,
            ..Default::default()
        };

        assert!(SpriteRenderer::should_use_nearest_filter(
            &sprite,
            &transform,
            &linear_settings
        ));

        // Test with non-integer scale
        let non_integer_transform = Transform {
            scale: [1.5, 1.5, 1.0], // Non-integer scale
            ..Default::default()
        };

        assert!(!SpriteRenderer::should_use_nearest_filter(
            &sprite,
            &non_integer_transform,
            &linear_settings
        ));
    }

    /// Test view mode enumeration
    #[test]
    fn test_view_mode() {
        assert_eq!(ViewMode::default(), ViewMode::Mode2D);
        assert_ne!(ViewMode::Mode2D, ViewMode::Mode3D);
    }

    /// Test transform conversion from arrays to Vec3
    #[test]
    fn test_transform_conversion() {
        let transform = Transform {
            position: [1.0, 2.0, 3.0],
            rotation: [90.0, 0.0, 0.0], // 90 degrees around X
            scale: [2.0, 2.0, 2.0],
        };

        let position = Vec3::from_array(transform.position);
        let scale = Vec3::from_array(transform.scale);

        assert_eq!(position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(scale, Vec3::new(2.0, 2.0, 2.0));

        // Test rotation conversion to radians
        let rotation_radians = Vec3::from_array(transform.rotation) * std::f32::consts::PI / 180.0;
        assert!((rotation_radians.x - std::f32::consts::PI / 2.0).abs() < 0.001);
    }
}
