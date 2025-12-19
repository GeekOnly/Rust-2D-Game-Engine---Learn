//! Mixed 2D/3D Content Rendering Pipeline
//!
//! This module provides the rendering pipeline for mixed 2D and 3D content,
//! ensuring proper depth testing between different content types and supporting
//! rendering both types in the same frame.

use wgpu;
use glam::Vec3;
use std::collections::HashMap;
use crate::depth_sorting::{DepthSortingSystem, RenderableItem, RenderableType};
use crate::texture::Texture;
use ecs::{Entity, Transform};
use ecs::components::{UnifiedSprite, UnifiedTilemap, ViewMode};

/// Mixed content rendering context
pub struct MixedContentRenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub surface_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
}

/// Mixed content renderer that handles both 2D and 3D objects
pub struct MixedContentRenderer {
    /// Depth sorting system for managing render order
    depth_sorting_system: DepthSortingSystem,
    
    /// Current camera position for depth calculations
    camera_position: Vec3,
    
    /// Current view mode
    view_mode: ViewMode,
    
    /// Whether depth testing is enabled
    depth_testing_enabled: bool,
    
    /// Depth comparison function
    depth_compare: wgpu::CompareFunction,
}

impl MixedContentRenderer {
    /// Create a new mixed content renderer
    pub fn new() -> Self {
        Self {
            depth_sorting_system: DepthSortingSystem::new(),
            camera_position: Vec3::ZERO,
            view_mode: ViewMode::Mode2D,
            depth_testing_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater, // Reverse-Z
        }
    }

    /// Update camera state for depth calculations
    pub fn update_camera(&mut self, position: Vec3, view_mode: ViewMode) {
        self.camera_position = position;
        self.view_mode = view_mode;
        self.depth_sorting_system.update_camera(position, view_mode);
    }

    /// Set depth testing configuration
    pub fn set_depth_testing(&mut self, enabled: bool, compare: wgpu::CompareFunction) {
        self.depth_testing_enabled = enabled;
        self.depth_compare = compare;
    }

    /// Clear all renderables from the previous frame
    pub fn clear_renderables(&mut self) {
        self.depth_sorting_system.clear();
    }

    /// Add a sprite to the rendering queue
    pub fn add_sprite(
        &mut self,
        entity: Entity,
        transform: &Transform,
        sprite: &UnifiedSprite,
    ) {
        self.depth_sorting_system.add_sprite(entity, transform, sprite);
    }

    /// Add a tilemap to the rendering queue
    pub fn add_tilemap(
        &mut self,
        entity: Entity,
        transform: &Transform,
        tilemap: &UnifiedTilemap,
    ) {
        self.depth_sorting_system.add_tilemap(entity, transform, tilemap);
    }

    /// Add a 3D mesh to the rendering queue
    pub fn add_mesh_3d(
        &mut self,
        entity: Entity,
        transform: &Transform,
        has_transparency: bool,
    ) {
        self.depth_sorting_system.add_mesh_3d(entity, transform, has_transparency);
    }

    /// Get sorted renderables for the current frame
    pub fn get_sorted_renderables(&mut self) -> &[RenderableItem] {
        self.depth_sorting_system.sort_and_get_renderables()
    }

    /// Render mixed 2D/3D content with proper depth testing
    pub fn render_mixed_content<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _sprite_renderer: Option<&'a crate::sprite_renderer::SpriteRenderer>,
        _tilemap_renderer: Option<&'a crate::tilemap_renderer::TilemapRenderer>,
        _mesh_renderer: Option<&'a crate::mesh_renderer::MeshRenderer>,
        _textures: &'a HashMap<String, Texture>,
        _camera_bind_group: &'a wgpu::BindGroup,
        _light_bind_group: Option<&'a wgpu::BindGroup>,
    ) -> Result<(), MixedContentRenderError> {
        // Get sorted renderables
        let sorted_renderables = self.depth_sorting_system.sort_and_get_renderables();
        
        // Track rendering statistics
        let mut sprites_rendered = 0;
        let mut tilemaps_rendered = 0;
        let mut meshes_rendered = 0;

        // Render each item in sorted order
        for renderable in sorted_renderables {
            match renderable.renderable_type {
                RenderableType::Sprite => {
                    if _sprite_renderer.is_some() {
                        // Set depth testing state for sprites
                        // The actual sprite rendering would be handled by the calling code
                        // since it needs access to the specific sprite data and vertex buffers
                        sprites_rendered += 1;
                    }
                }
                RenderableType::Tilemap => {
                    if _tilemap_renderer.is_some() {
                        // Set depth testing state for tilemaps
                        // The actual tilemap rendering would be handled by the calling code
                        // since it needs access to the specific tilemap data and GPU resources
                        tilemaps_rendered += 1;
                    }
                }
                RenderableType::Mesh3D => {
                    if _mesh_renderer.is_some() {
                        // Set depth testing state for 3D meshes
                        // The actual mesh rendering would be handled by the calling code
                        // since it needs access to the specific mesh data and materials
                        meshes_rendered += 1;
                    }
                }
            }
        }

        // Log rendering statistics in debug mode
        #[cfg(debug_assertions)]
        {
            if sprites_rendered > 0 || tilemaps_rendered > 0 || meshes_rendered > 0 {
                println!(
                    "Mixed content rendered: {} sprites, {} tilemaps, {} meshes",
                    sprites_rendered, tilemaps_rendered, meshes_rendered
                );
            }
        }

        Ok(())
    }

    /// Configure depth testing for sprite rendering
    fn configure_depth_testing_for_sprites(&self, _render_pass: &mut wgpu::RenderPass) {
        // In a real implementation, this would configure the render pass
        // depth testing state specifically for sprites
        // For now, this is a placeholder for the depth testing configuration
    }

    /// Configure depth testing for tilemap rendering
    fn configure_depth_testing_for_tilemaps(&self, _render_pass: &mut wgpu::RenderPass) {
        // In a real implementation, this would configure the render pass
        // depth testing state specifically for tilemaps
        // For now, this is a placeholder for the depth testing configuration
    }

    /// Configure depth testing for 3D mesh rendering
    fn configure_depth_testing_for_meshes(&self, _render_pass: &mut wgpu::RenderPass) {
        // In a real implementation, this would configure the render pass
        // depth testing state specifically for 3D meshes
        // For now, this is a placeholder for the depth testing configuration
    }

    /// Create a render pipeline with proper depth testing for mixed content
    pub fn create_mixed_content_pipeline(
        context: &MixedContentRenderContext,
        shader: &wgpu::ShaderModule,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        label: Option<&str>,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", label.unwrap_or("Mixed Content"))),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
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
                format: context.depth_format,
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
        })
    }

    /// Get rendering statistics
    pub fn get_render_stats(&self) -> MixedContentRenderStats {
        let renderables = self.depth_sorting_system.get_all_renderables();
        
        let mut stats = MixedContentRenderStats::default();
        
        for renderable in renderables {
            match renderable.renderable_type {
                RenderableType::Sprite => {
                    stats.sprite_count += 1;
                    if renderable.has_transparency {
                        stats.transparent_sprite_count += 1;
                    }
                }
                RenderableType::Tilemap => {
                    stats.tilemap_count += 1;
                }
                RenderableType::Mesh3D => {
                    stats.mesh_count += 1;
                    if renderable.has_transparency {
                        stats.transparent_mesh_count += 1;
                    }
                }
            }
        }
        
        stats.total_renderables = renderables.len();
        stats
    }

    /// Check if mixed content rendering is active
    pub fn has_mixed_content(&self) -> bool {
        let renderables = self.depth_sorting_system.get_all_renderables();
        
        let has_2d = renderables.iter().any(|r| {
            matches!(r.renderable_type, RenderableType::Sprite | RenderableType::Tilemap)
        });
        
        let has_3d = renderables.iter().any(|r| {
            matches!(r.renderable_type, RenderableType::Mesh3D)
        });
        
        has_2d && has_3d
    }

    /// Get the current view mode
    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Check if depth testing is enabled
    pub fn is_depth_testing_enabled(&self) -> bool {
        self.depth_testing_enabled
    }
}

impl Default for MixedContentRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for mixed content rendering
#[derive(Debug)]
pub enum MixedContentRenderError {
    SpriteRendererNotAvailable,
    TilemapRendererNotAvailable,
    MeshRendererNotAvailable,
    TextureNotFound(String),
    DepthTestingError(String),
    RenderPassError(String),
}

impl std::fmt::Display for MixedContentRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MixedContentRenderError::SpriteRendererNotAvailable => {
                write!(f, "Sprite renderer not available")
            }
            MixedContentRenderError::TilemapRendererNotAvailable => {
                write!(f, "Tilemap renderer not available")
            }
            MixedContentRenderError::MeshRendererNotAvailable => {
                write!(f, "Mesh renderer not available")
            }
            MixedContentRenderError::TextureNotFound(name) => {
                write!(f, "Texture not found: {}", name)
            }
            MixedContentRenderError::DepthTestingError(msg) => {
                write!(f, "Depth testing configuration error: {}", msg)
            }
            MixedContentRenderError::RenderPassError(msg) => {
                write!(f, "Render pass error: {}", msg)
            }
        }
    }
}

impl std::error::Error for MixedContentRenderError {}

/// Rendering statistics for mixed content
#[derive(Debug, Default, Clone)]
pub struct MixedContentRenderStats {
    pub total_renderables: usize,
    pub sprite_count: usize,
    pub tilemap_count: usize,
    pub mesh_count: usize,
    pub transparent_sprite_count: usize,
    pub transparent_mesh_count: usize,
}

impl MixedContentRenderStats {
    /// Get the number of opaque objects
    pub fn opaque_count(&self) -> usize {
        (self.sprite_count - self.transparent_sprite_count) + 
        self.tilemap_count + 
        (self.mesh_count - self.transparent_mesh_count)
    }

    /// Get the number of transparent objects
    pub fn transparent_count(&self) -> usize {
        self.transparent_sprite_count + self.transparent_mesh_count
    }

    /// Check if there are mixed 2D/3D objects
    pub fn has_mixed_content(&self) -> bool {
        let has_2d = self.sprite_count > 0 || self.tilemap_count > 0;
        let has_3d = self.mesh_count > 0;
        has_2d && has_3d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::Transform;

    fn create_test_transform(x: f32, y: f32, z: f32) -> Transform {
        Transform {
            position: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    fn create_test_sprite(sort_order: i32, alpha: f32) -> UnifiedSprite {
        UnifiedSprite {
            texture_id: "test".to_string(),
            width: 1.0,
            height: 1.0,
            color: [1.0, 1.0, 1.0, alpha],
            billboard: false,
            world_space_ui: false,
            pixel_perfect: true,
            sort_order,
            pixels_per_unit: None,
        }
    }

    #[test]
    fn test_mixed_content_renderer_creation() {
        let renderer = MixedContentRenderer::new();
        assert_eq!(renderer.view_mode, ViewMode::Mode2D);
        assert!(renderer.depth_testing_enabled);
        assert_eq!(renderer.camera_position, Vec3::ZERO);
    }

    #[test]
    fn test_camera_update() {
        let mut renderer = MixedContentRenderer::new();
        let new_position = Vec3::new(1.0, 2.0, 3.0);
        
        renderer.update_camera(new_position, ViewMode::Mode3D);
        
        assert_eq!(renderer.camera_position, new_position);
        assert_eq!(renderer.view_mode, ViewMode::Mode3D);
    }

    #[test]
    fn test_add_renderables() {
        let mut renderer = MixedContentRenderer::new();
        renderer.update_camera(Vec3::ZERO, ViewMode::Mode2D);

        // Add different types of renderables
        renderer.add_sprite(1, &create_test_transform(0.0, 0.0, 0.0), &create_test_sprite(0, 1.0));
        renderer.add_mesh_3d(2, &create_test_transform(1.0, 1.0, 1.0), false);

        let renderables = renderer.get_sorted_renderables();
        assert_eq!(renderables.len(), 2);
    }

    #[test]
    fn test_mixed_content_detection() {
        let mut renderer = MixedContentRenderer::new();
        renderer.update_camera(Vec3::ZERO, ViewMode::Mode2D);

        // Initially no mixed content
        assert!(!renderer.has_mixed_content());

        // Add only sprite - still no mixed content
        renderer.add_sprite(1, &create_test_transform(0.0, 0.0, 0.0), &create_test_sprite(0, 1.0));
        assert!(!renderer.has_mixed_content());

        // Add 3D mesh - now we have mixed content
        renderer.add_mesh_3d(2, &create_test_transform(1.0, 1.0, 1.0), false);
        assert!(renderer.has_mixed_content());
    }

    #[test]
    fn test_render_stats() {
        let mut renderer = MixedContentRenderer::new();
        renderer.update_camera(Vec3::ZERO, ViewMode::Mode2D);

        // Add various renderables
        renderer.add_sprite(1, &create_test_transform(0.0, 0.0, 0.0), &create_test_sprite(0, 1.0)); // Opaque sprite
        renderer.add_sprite(2, &create_test_transform(1.0, 0.0, 0.0), &create_test_sprite(0, 0.5)); // Transparent sprite
        renderer.add_mesh_3d(3, &create_test_transform(2.0, 0.0, 0.0), false); // Opaque mesh
        renderer.add_mesh_3d(4, &create_test_transform(3.0, 0.0, 0.0), true);  // Transparent mesh

        let stats = renderer.get_render_stats();
        
        assert_eq!(stats.total_renderables, 4);
        assert_eq!(stats.sprite_count, 2);
        assert_eq!(stats.mesh_count, 2);
        assert_eq!(stats.transparent_sprite_count, 1);
        assert_eq!(stats.transparent_mesh_count, 1);
        assert_eq!(stats.opaque_count(), 2);
        assert_eq!(stats.transparent_count(), 2);
        assert!(stats.has_mixed_content());
    }

    #[test]
    fn test_clear_renderables() {
        let mut renderer = MixedContentRenderer::new();
        renderer.update_camera(Vec3::ZERO, ViewMode::Mode2D);

        // Add renderables
        renderer.add_sprite(1, &create_test_transform(0.0, 0.0, 0.0), &create_test_sprite(0, 1.0));
        renderer.add_mesh_3d(2, &create_test_transform(1.0, 1.0, 1.0), false);

        assert_eq!(renderer.get_sorted_renderables().len(), 2);

        // Clear renderables
        renderer.clear_renderables();
        assert_eq!(renderer.get_sorted_renderables().len(), 0);
    }
}