//! Draw Call Batching Optimization System
//!
//! This module provides efficient batching of similar objects to minimize WGPU state changes
//! and improve rendering performance for both 2D and 3D content.

use std::collections::{HashMap, BTreeMap};
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};
use crate::sprite_renderer::UnifiedVertex;
use crate::texture::Texture;

/// Batch key for grouping similar renderables
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchKey {
    /// Texture identifier for grouping sprites/tilemaps with same texture
    pub texture_id: String,
    /// Render pipeline type
    pub pipeline_type: PipelineType,
    /// Depth layer for sorting
    pub depth_layer: i32,
    /// Perfect pixel settings hash for grouping compatible objects
    pub perfect_pixel_hash: u64,
    /// View mode (2D/3D) for appropriate shader selection
    pub view_mode: ViewModeKey,
}

/// Pipeline type for batching similar rendering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PipelineType {
    /// 2D sprites with unified shader
    UnifiedSprite2D,
    /// 3D sprites (billboards) with unified shader
    UnifiedSprite3D,
    /// 2D tilemaps with unified shader
    UnifiedTilemap2D,
    /// 3D tilemaps with unified shader
    UnifiedTilemap3D,
    /// Legacy sprite rendering (backward compatibility)
    LegacySprite,
    /// Legacy tilemap rendering (backward compatibility)
    LegacyTilemap,
}

/// View mode key for batching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewModeKey {
    Mode2D,
    Mode3D,
}

impl From<ecs::components::ViewMode> for ViewModeKey {
    fn from(mode: ecs::components::ViewMode) -> Self {
        match mode {
            ecs::components::ViewMode::Mode2D => ViewModeKey::Mode2D,
            ecs::components::ViewMode::Mode3D => ViewModeKey::Mode3D,
        }
    }
}

/// Batch data for sprites
#[derive(Debug)]
pub struct SpriteBatch {
    /// Batch key for this group
    pub key: BatchKey,
    /// Vertex data for all sprites in this batch
    pub vertices: Vec<UnifiedVertex>,
    /// Index data for all sprites in this batch
    pub indices: Vec<u16>,
    /// Number of sprites in this batch
    pub sprite_count: u32,
    /// GPU vertex buffer (created when batch is finalized)
    pub vertex_buffer: Option<wgpu::Buffer>,
    /// GPU index buffer (created when batch is finalized)
    pub index_buffer: Option<wgpu::Buffer>,
    /// Texture reference for this batch
    pub texture: Option<String>,
}

/// Batch data for tilemaps
#[derive(Debug)]
pub struct TilemapBatch {
    /// Batch key for this group
    pub key: BatchKey,
    /// Tilemap GPU resources grouped by similar properties
    pub resources: Vec<crate::tilemap_renderer::UnifiedTilemapGpuResources>,
    /// Number of tilemaps in this batch
    pub tilemap_count: u32,
    /// Texture reference for this batch
    pub texture: Option<String>,
}

/// Performance monitoring for batch efficiency
#[derive(Debug, Default)]
pub struct BatchPerformanceStats {
    /// Total number of draw calls before batching
    pub draw_calls_before: u32,
    /// Total number of draw calls after batching
    pub draw_calls_after: u32,
    /// Number of sprite batches created
    pub sprite_batches: u32,
    /// Number of tilemap batches created
    pub tilemap_batches: u32,
    /// Total vertices processed
    pub total_vertices: u32,
    /// Total sprites batched
    pub total_sprites: u32,
    /// Total tilemaps batched
    pub total_tilemaps: u32,
    /// Time spent batching (in microseconds)
    pub batching_time_us: u64,
    /// Memory saved by batching (estimated bytes)
    pub memory_saved_bytes: u64,
}

impl BatchPerformanceStats {
    /// Calculate batching efficiency as a percentage
    pub fn batching_efficiency(&self) -> f32 {
        if self.draw_calls_before == 0 {
            return 100.0;
        }
        let reduction = self.draw_calls_before.saturating_sub(self.draw_calls_after);
        (reduction as f32 / self.draw_calls_before as f32) * 100.0
    }

    /// Calculate average sprites per batch
    pub fn average_sprites_per_batch(&self) -> f32 {
        if self.sprite_batches == 0 {
            return 0.0;
        }
        self.total_sprites as f32 / self.sprite_batches as f32
    }

    /// Calculate average tilemaps per batch
    pub fn average_tilemaps_per_batch(&self) -> f32 {
        if self.tilemap_batches == 0 {
            return 0.0;
        }
        self.total_tilemaps as f32 / self.tilemap_batches as f32
    }
}

/// Main batching system for optimizing draw calls
pub struct BatchOptimizationSystem {
    /// Sprite batches grouped by batch key
    sprite_batches: BTreeMap<BatchKey, SpriteBatch>,
    /// Tilemap batches grouped by batch key
    tilemap_batches: BTreeMap<BatchKey, TilemapBatch>,
    /// Performance statistics
    stats: BatchPerformanceStats,
    /// Maximum vertices per batch (to prevent buffer overflow)
    max_vertices_per_batch: u32,
    /// Maximum sprites per batch
    max_sprites_per_batch: u32,
    /// Whether batching is enabled
    enabled: bool,
    /// Frame counter for performance tracking
    frame_counter: u64,
}

impl BatchOptimizationSystem {
    /// Create a new batch optimization system
    pub fn new() -> Self {
        Self {
            sprite_batches: BTreeMap::new(),
            tilemap_batches: BTreeMap::new(),
            stats: BatchPerformanceStats::default(),
            max_vertices_per_batch: 65536, // 64K vertices max per batch
            max_sprites_per_batch: 16384,  // 16K sprites max per batch
            enabled: true,
            frame_counter: 0,
        }
    }

    /// Enable or disable batching optimization
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear_batches();
        }
    }

    /// Check if batching is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Begin a new frame - clear previous batches
    pub fn begin_frame(&mut self) {
        let start_time = std::time::Instant::now();
        
        self.clear_batches();
        self.stats = BatchPerformanceStats::default();
        self.frame_counter += 1;
        
        self.stats.batching_time_us = start_time.elapsed().as_micros() as u64;
    }

    /// Add a sprite to the appropriate batch
    pub fn add_sprite(
        &mut self,
        entity: ecs::Entity,
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        camera_position: Vec3,
    ) {
        if !self.enabled {
            return;
        }

        let start_time = std::time::Instant::now();

        // Create batch key for this sprite
        let batch_key = self.create_sprite_batch_key(sprite, view_mode, perfect_pixel_settings);

        // Check if we need to create a new batch or use existing one
        let needs_new_batch = if let Some(existing_batch) = self.sprite_batches.get(&batch_key) {
            existing_batch.sprite_count >= self.max_sprites_per_batch || 
            existing_batch.vertices.len() + 4 > self.max_vertices_per_batch as usize
        } else {
            false
        };

        let actual_key = if needs_new_batch {
            // Create a new batch with incremented depth layer
            let mut new_key = batch_key;
            new_key.depth_layer += 1;
            new_key
        } else {
            batch_key
        };

        // Calculate sprite vertices first (before borrowing self mutably)
        let vertices = self.calculate_sprite_vertices(
            sprite, 
            transform, 
            view_mode, 
            perfect_pixel_settings, 
            camera_position
        );

        // Get or create batch
        let batch = self.sprite_batches.entry(actual_key.clone()).or_insert_with(|| {
            SpriteBatch {
                key: actual_key.clone(),
                vertices: Vec::new(),
                indices: Vec::new(),
                sprite_count: 0,
                vertex_buffer: None,
                index_buffer: None,
                texture: Some(sprite.texture_id.clone()),
            }
        });

        // Add vertices to batch
        let base_index = batch.vertices.len() as u16;
        batch.vertices.extend_from_slice(&vertices);

        // Add indices for this sprite (two triangles)
        let sprite_indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];
        batch.indices.extend_from_slice(&sprite_indices);

        batch.sprite_count += 1;
        self.stats.total_sprites += 1;
        self.stats.total_vertices += 4;
        self.stats.draw_calls_before += 1; // Each sprite would be a separate draw call

        self.stats.batching_time_us += start_time.elapsed().as_micros() as u64;
    }

    /// Add a tilemap to the appropriate batch
    pub fn add_tilemap(
        &mut self,
        entity: ecs::Entity,
        tilemap: &ecs::components::UnifiedTilemap,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        gpu_resources: crate::tilemap_renderer::UnifiedTilemapGpuResources,
    ) {
        if !self.enabled {
            return;
        }

        let start_time = std::time::Instant::now();

        // Create batch key for this tilemap
        let batch_key = self.create_tilemap_batch_key(tilemap, view_mode, perfect_pixel_settings);

        // Get or create batch
        let batch = self.tilemap_batches.entry(batch_key.clone()).or_insert_with(|| {
            TilemapBatch {
                key: batch_key.clone(),
                resources: Vec::new(),
                tilemap_count: 0,
                texture: Some(tilemap.tileset_id.clone()),
            }
        });

        // Add GPU resources to batch
        batch.resources.push(gpu_resources);
        batch.tilemap_count += 1;
        self.stats.total_tilemaps += 1;
        self.stats.draw_calls_before += 1; // Each tilemap would be a separate draw call

        self.stats.batching_time_us += start_time.elapsed().as_micros() as u64;
    }

    /// Finalize all batches by creating GPU buffers
    pub fn finalize_batches(&mut self, device: &wgpu::Device) {
        if !self.enabled {
            return;
        }

        let start_time = std::time::Instant::now();

        // Finalize sprite batches
        for (_, batch) in self.sprite_batches.iter_mut() {
            if !batch.vertices.is_empty() {
                // Create vertex buffer
                batch.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Batched Sprite Vertex Buffer"),
                    contents: bytemuck::cast_slice(&batch.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }));

                // Create index buffer
                batch.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Batched Sprite Index Buffer"),
                    contents: bytemuck::cast_slice(&batch.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }));

                self.stats.sprite_batches += 1;
                self.stats.draw_calls_after += 1;
            }
        }

        // Finalize tilemap batches
        for (_, batch) in self.tilemap_batches.iter_mut() {
            if !batch.resources.is_empty() {
                self.stats.tilemap_batches += 1;
                self.stats.draw_calls_after += 1;
            }
        }

        // Calculate memory savings
        self.calculate_memory_savings();

        self.stats.batching_time_us += start_time.elapsed().as_micros() as u64;
    }

    /// Render all sprite batches
    pub fn render_sprite_batches<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        sprite_renderer: &'a crate::sprite_renderer::SpriteRenderer,
        textures: &'a HashMap<String, Texture>,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if !self.enabled {
            return;
        }

        for (_, batch) in &self.sprite_batches {
            if let (Some(vertex_buffer), Some(index_buffer), Some(texture_id)) = 
                (&batch.vertex_buffer, &batch.index_buffer, &batch.texture) {
                
                if let Some(texture) = textures.get(texture_id) {
                    // Set appropriate pipeline based on batch key
                    match batch.key.pipeline_type {
                        PipelineType::UnifiedSprite2D | PipelineType::UnifiedSprite3D => {
                            // Use unified sprite rendering
                            if let Some(bind_group) = &texture.bind_group {
                                render_pass.set_pipeline(&sprite_renderer.unified_render_pipeline);
                                render_pass.set_bind_group(0, camera_bind_group, &[]);
                                render_pass.set_bind_group(1, bind_group, &[]);
                                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                                render_pass.draw_indexed(0..batch.indices.len() as u32, 0, 0..1);
                            }
                        }
                        PipelineType::LegacySprite => {
                            // Use legacy sprite rendering for backward compatibility
                            sprite_renderer.render(render_pass, texture, unsafe { std::mem::zeroed() }, camera_bind_group);
                        }
                        _ => {
                            // Other pipeline types not supported for sprite batches
                        }
                    }
                }
            }
        }
    }

    /// Render all tilemap batches
    pub fn render_tilemap_batches<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        tilemap_renderer: &'a crate::tilemap_renderer::TilemapRenderer,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        if !self.enabled {
            return;
        }

        for (_, batch) in &self.tilemap_batches {
            for resources in &batch.resources {
                match batch.key.pipeline_type {
                    PipelineType::UnifiedTilemap2D | PipelineType::UnifiedTilemap3D => {
                        tilemap_renderer.render_unified(render_pass, resources, camera_bind_group);
                    }
                    PipelineType::LegacyTilemap => {
                        // Legacy tilemap rendering would go here
                        // tilemap_renderer.render(render_pass, resources, camera_bind_group);
                    }
                    _ => {
                        // Other pipeline types not supported for tilemap batches
                    }
                }
            }
        }
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &BatchPerformanceStats {
        &self.stats
    }

    /// Clear all batches
    pub fn clear_batches(&mut self) {
        self.sprite_batches.clear();
        self.tilemap_batches.clear();
    }

    /// Get number of sprite batches
    pub fn get_sprite_batch_count(&self) -> usize {
        self.sprite_batches.len()
    }

    /// Get number of tilemap batches
    pub fn get_tilemap_batch_count(&self) -> usize {
        self.tilemap_batches.len()
    }

    /// Create batch key for sprite
    fn create_sprite_batch_key(
        &self,
        sprite: &ecs::components::UnifiedSprite,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
    ) -> BatchKey {
        // Create hash for perfect pixel settings
        let perfect_pixel_hash = self.hash_perfect_pixel_settings(perfect_pixel_settings, sprite.pixel_perfect);

        // Determine pipeline type
        let pipeline_type = match view_mode {
            ecs::components::ViewMode::Mode2D => PipelineType::UnifiedSprite2D,
            ecs::components::ViewMode::Mode3D => {
                if sprite.billboard {
                    PipelineType::UnifiedSprite3D
                } else {
                    PipelineType::UnifiedSprite2D // World-space quads use 2D pipeline
                }
            }
        };

        BatchKey {
            texture_id: sprite.texture_id.clone(),
            pipeline_type,
            depth_layer: sprite.sort_order,
            perfect_pixel_hash,
            view_mode: view_mode.into(),
        }
    }

    /// Create batch key for tilemap
    fn create_tilemap_batch_key(
        &self,
        tilemap: &ecs::components::UnifiedTilemap,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
    ) -> BatchKey {
        // Create hash for perfect pixel settings
        let perfect_pixel_hash = self.hash_perfect_pixel_settings(perfect_pixel_settings, tilemap.pixel_perfect);

        // Determine pipeline type
        let pipeline_type = match view_mode {
            ecs::components::ViewMode::Mode2D => PipelineType::UnifiedTilemap2D,
            ecs::components::ViewMode::Mode3D => PipelineType::UnifiedTilemap3D,
        };

        BatchKey {
            texture_id: tilemap.tileset_id.clone(),
            pipeline_type,
            depth_layer: (tilemap.layer_depth * 1000.0) as i32, // Convert float to int for batching
            perfect_pixel_hash,
            view_mode: view_mode.into(),
        }
    }

    /// Create hash for perfect pixel settings to group compatible objects
    fn hash_perfect_pixel_settings(
        &self,
        settings: &ecs::components::PerfectPixelSettings,
        object_pixel_perfect: bool,
    ) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Hash the relevant settings that affect rendering compatibility
        settings.enabled.hash(&mut hasher);
        object_pixel_perfect.hash(&mut hasher);
        settings.snap_to_pixel.hash(&mut hasher);
        settings.filter_mode.hash(&mut hasher);
        
        // Hash pixels_per_unit as integer to avoid floating point precision issues
        ((settings.pixels_per_unit * 100.0) as u32).hash(&mut hasher);
        
        hasher.finish()
    }

    /// Calculate sprite vertices for batching
    fn calculate_sprite_vertices(
        &self,
        sprite: &ecs::components::UnifiedSprite,
        transform: &ecs::Transform,
        view_mode: ecs::components::ViewMode,
        perfect_pixel_settings: &ecs::components::PerfectPixelSettings,
        camera_position: Vec3,
    ) -> [UnifiedVertex; 4] {
        // This is a simplified version - in practice, this would use the same logic
        // as the sprite renderer's calculate_sprite_vertices method
        let half_width = sprite.width * 0.5;
        let half_height = sprite.height * 0.5;

        // Base quad positions
        let positions = [
            Vec3::new(-half_width, half_height, 0.0),   // Top Left
            Vec3::new(-half_width, -half_height, 0.0),  // Bottom Left
            Vec3::new(half_width, -half_height, 0.0),   // Bottom Right
            Vec3::new(half_width, half_height, 0.0),    // Top Right
        ];

        // Apply transform (simplified - would use full transform matrix in practice)
        let world_position = Vec3::from_array(transform.position);
        let transformed_positions = positions.map(|pos| pos + world_position);

        // Create vertices
        [
            UnifiedVertex {
                position: transformed_positions[0].into(),
                tex_coords: [0.0, 0.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: transformed_positions[1].into(),
                tex_coords: [0.0, 1.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: transformed_positions[2].into(),
                tex_coords: [1.0, 1.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
            UnifiedVertex {
                position: transformed_positions[3].into(),
                tex_coords: [1.0, 0.0],
                color: sprite.color,
                normal: [0.0, 0.0, 1.0],
            },
        ]
    }

    /// Calculate estimated memory savings from batching
    fn calculate_memory_savings(&mut self) {
        // Estimate memory savings by comparing individual draw calls vs batched draw calls
        let individual_overhead_per_sprite = 64; // Estimated bytes per individual draw call
        let individual_overhead_per_tilemap = 128; // Estimated bytes per individual tilemap draw call
        let batch_overhead = 256; // Estimated bytes per batch

        let sprite_memory_before = self.stats.total_sprites * individual_overhead_per_sprite;
        let sprite_memory_after = self.stats.sprite_batches * batch_overhead;
        let sprite_savings = sprite_memory_before.saturating_sub(sprite_memory_after);

        let tilemap_memory_before = self.stats.total_tilemaps * individual_overhead_per_tilemap;
        let tilemap_memory_after = self.stats.tilemap_batches * batch_overhead;
        let tilemap_savings = tilemap_memory_before.saturating_sub(tilemap_memory_after);

        self.stats.memory_saved_bytes = (sprite_savings + tilemap_savings) as u64;
    }

    /// Set maximum vertices per batch
    pub fn set_max_vertices_per_batch(&mut self, max_vertices: u32) {
        self.max_vertices_per_batch = max_vertices;
    }

    /// Set maximum sprites per batch
    pub fn set_max_sprites_per_batch(&mut self, max_sprites: u32) {
        self.max_sprites_per_batch = max_sprites;
    }

    /// Get current frame counter
    pub fn get_frame_counter(&self) -> u64 {
        self.frame_counter
    }

    /// Check if batching would be beneficial for current frame
    pub fn should_use_batching(&self) -> bool {
        self.enabled && (self.stats.total_sprites > 10 || self.stats.total_tilemaps > 5)
    }

    /// Get batching recommendations based on current performance
    pub fn get_batching_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.stats.batching_efficiency() < 50.0 {
            recommendations.push("Consider grouping objects with similar textures and properties".to_string());
        }

        if self.stats.average_sprites_per_batch() < 5.0 {
            recommendations.push("Increase sprite batch size by using fewer unique textures".to_string());
        }

        if self.stats.batching_time_us > 1000 {
            recommendations.push("Batching overhead is high - consider reducing batch complexity".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Batching is performing optimally".to_string());
        }

        recommendations
    }
}

impl Default for BatchOptimizationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::components::{UnifiedSprite, UnifiedTilemap, ViewMode, PerfectPixelSettings, FilterMode};
    use ecs::Transform;

    #[test]
    fn test_batch_key_creation() {
        let mut batch_system = BatchOptimizationSystem::new();
        
        let sprite = UnifiedSprite {
            texture_id: "test_texture".to_string(),
            sort_order: 5,
            pixel_perfect: true,
            ..Default::default()
        };

        let perfect_pixel_settings = PerfectPixelSettings {
            enabled: true,
            pixels_per_unit: 100.0,
            filter_mode: FilterMode::Nearest,
            ..Default::default()
        };

        let key = batch_system.create_sprite_batch_key(&sprite, ViewMode::Mode2D, &perfect_pixel_settings);
        
        assert_eq!(key.texture_id, "test_texture");
        assert_eq!(key.pipeline_type, PipelineType::UnifiedSprite2D);
        assert_eq!(key.depth_layer, 5);
        assert_eq!(key.view_mode, ViewModeKey::Mode2D);
    }

    #[test]
    fn test_batch_performance_stats() {
        let mut stats = BatchPerformanceStats::default();
        stats.draw_calls_before = 100;
        stats.draw_calls_after = 25;
        stats.sprite_batches = 5;
        stats.total_sprites = 100;

        assert_eq!(stats.batching_efficiency(), 75.0);
        assert_eq!(stats.average_sprites_per_batch(), 20.0);
    }

    #[test]
    fn test_batching_system_lifecycle() {
        let mut batch_system = BatchOptimizationSystem::new();
        
        assert!(batch_system.is_enabled());
        assert_eq!(batch_system.get_sprite_batch_count(), 0);
        assert_eq!(batch_system.get_tilemap_batch_count(), 0);

        batch_system.begin_frame();
        
        // Add some test data
        let sprite = UnifiedSprite::default();
        let transform = Transform::default();
        let view_mode = ViewMode::Mode2D;
        let perfect_pixel_settings = PerfectPixelSettings::default();
        let camera_position = Vec3::ZERO;

        batch_system.add_sprite(
            1u32,
            &sprite,
            &transform,
            view_mode,
            &perfect_pixel_settings,
            camera_position,
        );

        assert_eq!(batch_system.get_sprite_batch_count(), 1);
        assert_eq!(batch_system.get_performance_stats().total_sprites, 1);
    }

    #[test]
    fn test_perfect_pixel_hash_consistency() {
        let batch_system = BatchOptimizationSystem::new();
        
        let settings1 = PerfectPixelSettings {
            enabled: true,
            pixels_per_unit: 100.0,
            filter_mode: FilterMode::Nearest,
            ..Default::default()
        };

        let settings2 = PerfectPixelSettings {
            enabled: true,
            pixels_per_unit: 100.0,
            filter_mode: FilterMode::Nearest,
            ..Default::default()
        };

        let hash1 = batch_system.hash_perfect_pixel_settings(&settings1, true);
        let hash2 = batch_system.hash_perfect_pixel_settings(&settings2, true);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_batch_recommendations() {
        let mut batch_system = BatchOptimizationSystem::new();
        batch_system.stats.draw_calls_before = 100;
        batch_system.stats.draw_calls_after = 80; // Low efficiency
        batch_system.stats.sprite_batches = 20;
        batch_system.stats.total_sprites = 80; // Low sprites per batch

        let recommendations = batch_system.get_batching_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("similar textures")));
    }
}