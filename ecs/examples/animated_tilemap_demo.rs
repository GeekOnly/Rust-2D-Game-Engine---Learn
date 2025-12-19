//! Animated Tilemap Demo
//!
//! This example demonstrates the animated tile support with perfect pixel preservation
//! for the unified 2D/3D rendering system.

use ecs::{
    UnifiedTilemap, TilemapAnimationSystem, PerfectPixelSettings,
    components::{unified_rendering::AnimatedTileData, sprite_sheet::AnimationMode}
};

fn main() {
    println!("Animated Tilemap Demo");
    println!("====================");

    // Create a new unified tilemap
    let mut tilemap = UnifiedTilemap::new("water_tileset");
    tilemap.pixel_perfect = true;
    tilemap.preserve_pixel_alignment = true;

    // Create perfect pixel settings
    let perfect_pixel_settings = PerfectPixelSettings::pixel_art();

    // Create animated water tiles
    let water_animation = TilemapAnimationSystem::create_water_animation(
        10, // base_tile_id
        4,  // frame_count (tiles 10, 11, 12, 13)
        8.0, // fps
        true // perfect_pixel_enabled
    );

    // Add animated water tiles to the tilemap
    tilemap.add_animated_tile(0, 0, water_animation.clone());
    tilemap.add_animated_tile(1, 0, water_animation.clone());
    tilemap.add_animated_tile(2, 0, water_animation);

    // Create animated flame tiles
    let flame_animation = TilemapAnimationSystem::create_flame_animation(
        20, // base_tile_id
        3,  // frame_count (tiles 20, 21, 22)
        15.0, // fps
        true // perfect_pixel_enabled
    );

    // Add animated flame tile
    tilemap.add_animated_tile(5, 0, flame_animation);

    // Create conveyor belt animation
    let conveyor_animation = TilemapAnimationSystem::create_conveyor_animation(
        30, // base_tile_id
        2,  // frame_count (tiles 30, 31)
        6.0, // fps
        true // perfect_pixel_enabled
    );

    // Add conveyor belt tiles
    tilemap.add_animated_tile(10, 0, conveyor_animation.clone());
    tilemap.add_animated_tile(11, 0, conveyor_animation);

    println!("Created tilemap with {} animated tiles", tilemap.animated_tiles.len());

    // Simulate animation updates
    let delta_time = 1.0 / 60.0; // 60 FPS
    let mut total_time = 0.0;

    println!("\nSimulating animation updates:");
    for frame in 0..120 { // 2 seconds at 60 FPS
        // Update animations
        TilemapAnimationSystem::update_tilemap_animations(
            &mut tilemap,
            delta_time,
            &perfect_pixel_settings,
        );

        total_time += delta_time;

        // Print animation state every 30 frames (0.5 seconds)
        if frame % 30 == 0 {
            println!("Frame {}: Time {:.2}s", frame, total_time);
            
            // Check water animation at (0, 0)
            let water_tile_id = tilemap.get_render_tile_id(0, 0);
            println!("  Water tile (0,0): tile_id = {}", water_tile_id);
            
            // Check flame animation at (5, 0)
            let flame_tile_id = tilemap.get_render_tile_id(5, 0);
            println!("  Flame tile (5,0): tile_id = {}", flame_tile_id);
            
            // Check conveyor animation at (10, 0)
            let conveyor_tile_id = tilemap.get_render_tile_id(10, 0);
            println!("  Conveyor tile (10,0): tile_id = {}", conveyor_tile_id);
        }
    }

    // Test animation control
    println!("\nTesting animation control:");
    
    // Pause all animations
    TilemapAnimationSystem::pause_tilemap_animations(&mut tilemap);
    println!("Paused all animations");
    
    // Update (should not change)
    let water_before = tilemap.get_render_tile_id(0, 0);
    TilemapAnimationSystem::update_tilemap_animations(
        &mut tilemap,
        delta_time,
        &perfect_pixel_settings,
    );
    let water_after = tilemap.get_render_tile_id(0, 0);
    println!("Water tile before/after pause: {} -> {} (should be same)", water_before, water_after);
    
    // Resume animations
    TilemapAnimationSystem::resume_tilemap_animations(&mut tilemap);
    println!("Resumed all animations");
    
    // Test speed control
    TilemapAnimationSystem::set_tilemap_animation_speed(&mut tilemap, 2.0);
    println!("Set animation speed to 2x");
    
    // Test pixel perfect frame duration calculation
    let duration_8fps = UnifiedTilemap::calculate_pixel_perfect_frame_duration(8.0, true);
    let duration_12fps = UnifiedTilemap::calculate_pixel_perfect_frame_duration(12.0, true);
    let duration_non_perfect = UnifiedTilemap::calculate_pixel_perfect_frame_duration(8.0, false);
    
    println!("\nPixel-perfect frame duration calculations:");
    println!("  8 FPS (pixel-perfect): {:.4}s", duration_8fps);
    println!("  12 FPS (pixel-perfect): {:.4}s", duration_12fps);
    println!("  8 FPS (non-pixel-perfect): {:.4}s", duration_non_perfect);

    println!("\nDemo completed successfully!");
}