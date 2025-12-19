//! Unified 2D/3D Rendering Demo
//!
//! This example demonstrates the core unified rendering infrastructure,
//! showing how to create cameras with 2D/3D mode switching capabilities.

use ecs::{
    Camera, CameraProjection, UnifiedRenderingHelpers,
    components::{ViewMode, PerfectPixelSettings, UnifiedSprite, UnifiedTilemap, FilterMode}
};

fn main() {
    println!("=== Unified 2D/3D Rendering Demo ===\n");
    
    // Demo 1: Create a unified camera that can switch between 2D and 3D modes
    println!("1. Creating unified camera...");
    let mut camera = UnifiedRenderingHelpers::create_unified_camera();
    println!("   Initial mode: {:?}", camera.get_view_mode());
    println!("   Has unified rendering: {}", camera.has_unified_rendering());
    
    // Demo 2: Switch between modes
    println!("\n2. Switching between 2D and 3D modes...");
    camera.toggle_view_mode();
    println!("   After toggle: {:?}", camera.get_view_mode());
    camera.toggle_view_mode();
    println!("   After second toggle: {:?}", camera.get_view_mode());
    
    // Demo 3: Create specialized cameras
    println!("\n3. Creating specialized cameras...");
    
    let camera_2d = UnifiedRenderingHelpers::create_2d_camera(100.0, 5.0);
    println!("   2D Camera mode: {:?}", camera_2d.get_view_mode());
    println!("   2D Camera projection: {:?}", camera_2d.projection);
    println!("   2D Camera pixels per unit: {}", camera_2d.get_effective_pixels_per_unit());
    
    let camera_3d = UnifiedRenderingHelpers::create_3d_camera(60.0, 0.1, 1000.0);
    println!("   3D Camera mode: {:?}", camera_3d.get_view_mode());
    println!("   3D Camera projection: {:?}", camera_3d.projection);
    println!("   3D Camera FOV: {}", camera_3d.fov);
    
    // Demo 4: Perfect pixel settings
    println!("\n4. Perfect pixel settings...");
    
    let pixel_art_settings = UnifiedRenderingHelpers::create_pixel_art_settings(32.0);
    println!("   Pixel art settings:");
    println!("     Enabled: {}", pixel_art_settings.enabled);
    println!("     Snap to pixel: {}", pixel_art_settings.snap_to_pixel);
    println!("     Filter mode: {:?}", pixel_art_settings.filter_mode);
    println!("     Pixels per unit: {}", pixel_art_settings.pixels_per_unit);
    
    let hd_settings = UnifiedRenderingHelpers::create_hd_sprite_settings(100.0);
    println!("   HD sprite settings:");
    println!("     Enabled: {}", hd_settings.enabled);
    println!("     Snap to pixel: {}", hd_settings.snap_to_pixel);
    println!("     Filter mode: {:?}", hd_settings.filter_mode);
    println!("     Pixels per unit: {}", hd_settings.pixels_per_unit);
    
    // Demo 5: Unified sprite component
    println!("\n5. Unified sprite component...");
    
    let mut sprite = UnifiedSprite::default();
    sprite.texture_id = "player_sprite.png".to_string();
    sprite.width = 1.0;
    sprite.height = 1.0;
    sprite.billboard = false; // World-space quad in 3D
    sprite.pixel_perfect = true;
    sprite.pixels_per_unit = Some(64.0); // Override global setting
    
    println!("   Sprite texture: {}", sprite.texture_id);
    println!("   Sprite size: {}x{}", sprite.width, sprite.height);
    println!("   Billboard mode: {}", sprite.billboard);
    println!("   Pixel perfect: {}", sprite.pixel_perfect);
    println!("   Custom pixels per unit: {:?}", sprite.pixels_per_unit);
    
    // Demo 6: Unified tilemap component
    println!("\n6. Unified tilemap component...");
    
    let mut tilemap = UnifiedTilemap::default();
    tilemap.tileset_id = "dungeon_tiles.png".to_string();
    tilemap.tile_size = (16, 16);
    tilemap.chunk_size = (32, 32);
    tilemap.pixel_perfect = true;
    tilemap.layer_depth = -1.0; // Behind sprites
    
    // Add some tiles
    tilemap.tiles.insert((0, 0), 1); // Tile ID 1 at position (0,0)
    tilemap.tiles.insert((1, 0), 2); // Tile ID 2 at position (1,0)
    tilemap.tiles.insert((0, 1), 3); // Tile ID 3 at position (0,1)
    
    println!("   Tilemap tileset: {}", tilemap.tileset_id);
    println!("   Tile size: {:?}", tilemap.tile_size);
    println!("   Chunk size: {:?}", tilemap.chunk_size);
    println!("   Number of tiles: {}", tilemap.tiles.len());
    println!("   Layer depth: {}", tilemap.layer_depth);
    
    // Demo 7: Mode switching workflow
    println!("\n7. Mode switching workflow...");
    
    let mut workflow_camera = Camera::default();
    println!("   Initial state: unified rendering = {}", workflow_camera.has_unified_rendering());
    
    // Enable unified rendering and switch to 2D
    UnifiedRenderingHelpers::switch_to_2d_mode(&mut workflow_camera, Some(100.0));
    println!("   After enabling 2D mode: {:?}", workflow_camera.get_view_mode());
    println!("   Projection: {:?}", workflow_camera.projection);
    
    // Switch to 3D
    UnifiedRenderingHelpers::switch_to_3d_mode(&mut workflow_camera, Some(75.0));
    println!("   After switching to 3D mode: {:?}", workflow_camera.get_view_mode());
    println!("   Projection: {:?}", workflow_camera.projection);
    
    // Demo 8: Helper utility functions
    println!("\n8. Helper utility functions...");
    
    println!("   Is 2D mode: {}", UnifiedRenderingHelpers::is_2d_mode(&camera_2d));
    println!("   Is 3D mode: {}", UnifiedRenderingHelpers::is_3d_mode(&camera_2d));
    println!("   Is 2D mode: {}", UnifiedRenderingHelpers::is_2d_mode(&camera_3d));
    println!("   Is 3D mode: {}", UnifiedRenderingHelpers::is_3d_mode(&camera_3d));
    
    println!("   2D Camera pixels per unit: {}", UnifiedRenderingHelpers::get_pixels_per_unit(&camera_2d));
    println!("   3D Camera pixels per unit: {}", UnifiedRenderingHelpers::get_pixels_per_unit(&camera_3d));
    
    println!("\n=== Demo Complete ===");
    println!("The unified 2D/3D rendering infrastructure is now set up and ready to use!");
    println!("Key features implemented:");
    println!("  ✓ ViewMode enum for 2D/3D switching");
    println!("  ✓ UnifiedCamera component with mode-specific settings");
    println!("  ✓ PerfectPixelSettings for pixel-perfect 2D rendering");
    println!("  ✓ UnifiedSprite component for 2D/3D sprite rendering");
    println!("  ✓ UnifiedTilemap component for 2D/3D tilemap rendering");
    println!("  ✓ WGPU pipeline modifications (UnifiedRenderer)");
    println!("  ✓ Helper functions for easy camera setup and mode switching");
}