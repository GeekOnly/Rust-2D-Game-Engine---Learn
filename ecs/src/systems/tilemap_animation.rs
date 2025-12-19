//! Tilemap Animation System
//!
//! This system handles updating animated tiles in UnifiedTilemap components
//! while preserving perfect pixel alignment for 2D rendering.

use crate::components::{UnifiedTilemap, PerfectPixelSettings, unified_rendering::AnimatedTileData};
use crate::traits::EcsWorld;

/// Tilemap animation system that updates all animated tiles
pub struct TilemapAnimationSystem {
    /// Delta time accumulator for consistent timing
    delta_accumulator: f32,
    /// Target update frequency for animations (Hz)
    update_frequency: f32,
}

impl Default for TilemapAnimationSystem {
    fn default() -> Self {
        Self {
            delta_accumulator: 0.0,
            update_frequency: 60.0, // 60 Hz for smooth animations
        }
    }
}

impl TilemapAnimationSystem {
    /// Create a new tilemap animation system
    pub fn new(update_frequency: f32) -> Self {
        Self {
            delta_accumulator: 0.0,
            update_frequency,
        }
    }

    /// Update all animated tilemaps in the world
    pub fn update<T: EcsWorld>(
        &mut self,
        world: &mut T,
        delta_time: f32,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) {
        self.delta_accumulator += delta_time;
        
        // Update at fixed frequency for consistent animation timing
        let update_interval = 1.0 / self.update_frequency;
        
        while self.delta_accumulator >= update_interval {
            self.delta_accumulator -= update_interval;
            
            // Update all UnifiedTilemap components with animations
            self.update_animated_tilemaps(world, update_interval, perfect_pixel_settings);
        }
    }

    /// Update all animated tilemaps
    fn update_animated_tilemaps<T: EcsWorld>(
        &self,
        world: &mut T,
        delta_time: f32,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) {
        // For now, this is a placeholder since we don't have a generic query system
        // In a real implementation, this would iterate over all entities with UnifiedTilemap components
        // and update their animations
    }

    /// Update a specific tilemap's animations
    pub fn update_tilemap_animations(
        tilemap: &mut UnifiedTilemap,
        delta_time: f32,
        perfect_pixel_settings: &PerfectPixelSettings,
    ) {
        if tilemap.animated_tiles.is_empty() {
            return;
        }

        // Apply global animation speed and perfect pixel settings
        let effective_delta = delta_time * tilemap.animation_speed;
        let preserve_alignment = tilemap.preserve_pixel_alignment && 
                                tilemap.pixel_perfect && 
                                perfect_pixel_settings.enabled;

        // Update each animated tile
        for animated_tile in tilemap.animated_tiles.values_mut() {
            animated_tile.update(effective_delta, preserve_alignment);
        }
    }

    /// Check if any tilemaps need GPU resource updates due to animation changes
    pub fn check_animation_updates<T: EcsWorld>(
        &self,
        world: &T,
    ) -> Vec<T::Entity> {
        // For now, return empty vector since we don't have a generic query system
        // In a real implementation, this would check all entities with UnifiedTilemap components
        Vec::new()
    }

    /// Check if a tilemap has animation frame changes that require GPU updates
    fn has_animation_frame_changes(&self, tilemap: &UnifiedTilemap) -> bool {
        // For now, assume any tilemap with animated tiles needs updates
        // In a more optimized implementation, we could track frame changes
        !tilemap.animated_tiles.is_empty()
    }

    /// Create a simple water animation tile sequence
    pub fn create_water_animation(
        base_tile_id: u32,
        frame_count: u32,
        fps: f32,
        perfect_pixel_enabled: bool,
    ) -> AnimatedTileData {
        let frame_sequence: Vec<u32> = (base_tile_id..base_tile_id + frame_count).collect();
        let frame_duration = UnifiedTilemap::calculate_pixel_perfect_frame_duration(fps, perfect_pixel_enabled);
        
        UnifiedTilemap::create_animated_tile(
            base_tile_id,
            frame_sequence,
            frame_duration,
            crate::components::sprite_sheet::AnimationMode::Loop,
        )
    }

    /// Create a torch flame animation tile sequence
    pub fn create_flame_animation(
        base_tile_id: u32,
        frame_count: u32,
        fps: f32,
        perfect_pixel_enabled: bool,
    ) -> AnimatedTileData {
        let frame_sequence: Vec<u32> = (base_tile_id..base_tile_id + frame_count).collect();
        let frame_duration = UnifiedTilemap::calculate_pixel_perfect_frame_duration(fps, perfect_pixel_enabled);
        
        UnifiedTilemap::create_animated_tile(
            base_tile_id,
            frame_sequence,
            frame_duration,
            crate::components::sprite_sheet::AnimationMode::Loop,
        )
    }

    /// Create a conveyor belt animation tile sequence
    pub fn create_conveyor_animation(
        base_tile_id: u32,
        frame_count: u32,
        fps: f32,
        perfect_pixel_enabled: bool,
    ) -> AnimatedTileData {
        let frame_sequence: Vec<u32> = (base_tile_id..base_tile_id + frame_count).collect();
        let frame_duration = UnifiedTilemap::calculate_pixel_perfect_frame_duration(fps, perfect_pixel_enabled);
        
        UnifiedTilemap::create_animated_tile(
            base_tile_id,
            frame_sequence,
            frame_duration,
            crate::components::sprite_sheet::AnimationMode::Loop,
        )
    }

    /// Pause all animations in a tilemap
    pub fn pause_tilemap_animations(tilemap: &mut UnifiedTilemap) {
        tilemap.pause_all_animations();
    }

    /// Resume all animations in a tilemap
    pub fn resume_tilemap_animations(tilemap: &mut UnifiedTilemap) {
        tilemap.resume_all_animations();
    }

    /// Stop and reset all animations in a tilemap
    pub fn stop_tilemap_animations(tilemap: &mut UnifiedTilemap) {
        tilemap.stop_all_animations();
    }

    /// Set animation speed for a tilemap
    pub fn set_tilemap_animation_speed(tilemap: &mut UnifiedTilemap, speed: f32) {
        tilemap.animation_speed = speed.max(0.0); // Ensure non-negative speed
    }

    /// Enable or disable pixel alignment preservation for a tilemap
    pub fn set_pixel_alignment_preservation(tilemap: &mut UnifiedTilemap, preserve: bool) {
        tilemap.preserve_pixel_alignment = preserve;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{unified_rendering::AnimatedTileData, sprite_sheet::AnimationMode};

    #[test]
    fn test_animation_system_creation() {
        let system = TilemapAnimationSystem::new(30.0);
        assert_eq!(system.update_frequency, 30.0);
        assert_eq!(system.delta_accumulator, 0.0);
    }

    #[test]
    fn test_water_animation_creation() {
        let animation = TilemapAnimationSystem::create_water_animation(10, 4, 8.0, true);
        assert_eq!(animation.base_tile_id, 10);
        assert_eq!(animation.frame_sequence, vec![10, 11, 12, 13]);
        assert_eq!(animation.animation_mode, AnimationMode::Loop);
        assert!(animation.playing);
    }

    #[test]
    fn test_pixel_perfect_frame_duration() {
        let duration_8fps = UnifiedTilemap::calculate_pixel_perfect_frame_duration(8.0, true);
        let duration_12fps = UnifiedTilemap::calculate_pixel_perfect_frame_duration(12.0, true);
        
        // Should align to common frame rates
        assert!(duration_8fps > 0.0);
        assert!(duration_12fps > 0.0);
        
        // Perfect pixel should give different results than non-perfect
        let duration_non_perfect = UnifiedTilemap::calculate_pixel_perfect_frame_duration(8.0, false);
        assert_eq!(duration_non_perfect, 1.0 / 8.0);
    }

    #[test]
    fn test_tilemap_animation_updates() {
        let mut tilemap = UnifiedTilemap::new("test_tileset");
        
        // Add an animated tile
        let animation = TilemapAnimationSystem::create_water_animation(1, 3, 10.0, true);
        tilemap.add_animated_tile(0, 0, animation);
        
        // Initial state
        assert_eq!(tilemap.get_render_tile_id(0, 0), 1);
        
        // Update animations
        let perfect_pixel_settings = PerfectPixelSettings::default();
        TilemapAnimationSystem::update_tilemap_animations(&mut tilemap, 0.1, &perfect_pixel_settings);
        
        // Should still be playing
        assert!(tilemap.animated_tiles.get(&(0, 0)).unwrap().playing);
    }

    #[test]
    fn test_animation_speed_control() {
        let mut tilemap = UnifiedTilemap::new("test_tileset");
        
        // Test speed setting
        TilemapAnimationSystem::set_tilemap_animation_speed(&mut tilemap, 2.0);
        assert_eq!(tilemap.animation_speed, 2.0);
        
        // Test negative speed clamping
        TilemapAnimationSystem::set_tilemap_animation_speed(&mut tilemap, -1.0);
        assert_eq!(tilemap.animation_speed, 0.0);
    }

    #[test]
    fn test_animation_control() {
        let mut tilemap = UnifiedTilemap::new("test_tileset");
        
        // Add an animated tile
        let animation = TilemapAnimationSystem::create_flame_animation(5, 2, 15.0, false);
        tilemap.add_animated_tile(1, 1, animation);
        
        // Test pause
        TilemapAnimationSystem::pause_tilemap_animations(&mut tilemap);
        assert!(!tilemap.animated_tiles.get(&(1, 1)).unwrap().playing);
        
        // Test resume
        TilemapAnimationSystem::resume_tilemap_animations(&mut tilemap);
        assert!(tilemap.animated_tiles.get(&(1, 1)).unwrap().playing);
        
        // Test stop
        TilemapAnimationSystem::stop_tilemap_animations(&mut tilemap);
        let anim = tilemap.animated_tiles.get(&(1, 1)).unwrap();
        assert!(!anim.playing);
        assert_eq!(anim.current_frame, 0);
        assert_eq!(anim.elapsed_time, 0.0);
    }
}