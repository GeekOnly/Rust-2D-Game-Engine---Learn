/// Unit tests for error handling and validation
/// Tests invalid sensitivity values, NaN/Inf handling, extreme zoom levels, and projection edge cases

use glam::Vec2;

// Import the camera module
#[path = "../src/editor/camera.rs"]
mod camera;
use camera::{SceneCamera, CameraSettings};

// Import the grid module
#[path = "../src/editor/grid.rs"]
mod grid;
use grid::{InfiniteGrid, SceneGrid, CameraState};

// ============================================================================
// CAMERA SETTINGS VALIDATION TESTS
// ============================================================================

#[test]
fn test_sensitivity_clamping_min() {
    let mut settings = CameraSettings::default();
    
    // Set values below minimum
    settings.pan_sensitivity = -1.0;
    settings.rotation_sensitivity = 0.0;
    settings.zoom_sensitivity = 0.001;
    
    // Validate should clamp to minimum
    settings.validate();
    
    assert!(settings.pan_sensitivity >= CameraSettings::MIN_SENSITIVITY);
    assert!(settings.rotation_sensitivity >= CameraSettings::MIN_SENSITIVITY);
    assert!(settings.zoom_sensitivity >= CameraSettings::MIN_SENSITIVITY);
}

#[test]
fn test_sensitivity_clamping_max() {
    let mut settings = CameraSettings::default();
    
    // Set values above maximum
    settings.pan_sensitivity = 100.0;
    settings.rotation_sensitivity = 50.0;
    settings.zoom_sensitivity = 20.0;
    
    // Validate should clamp to maximum
    settings.validate();
    
    assert!(settings.pan_sensitivity <= CameraSettings::MAX_SENSITIVITY);
    assert!(settings.rotation_sensitivity <= CameraSettings::MAX_SENSITIVITY);
    assert!(settings.zoom_sensitivity <= CameraSettings::MAX_SENSITIVITY);
}

#[test]
fn test_damping_clamping() {
    let mut settings = CameraSettings::default();
    
    // Set damping values outside [0, 1]
    settings.pan_damping = -0.5;
    settings.rotation_damping = 1.5;
    settings.zoom_damping = 2.0;
    
    settings.validate();
    
    assert!(settings.pan_damping >= 0.0 && settings.pan_damping <= 1.0);
    assert!(settings.rotation_damping >= 0.0 && settings.rotation_damping <= 1.0);
    assert!(settings.zoom_damping >= 0.0 && settings.zoom_damping <= 1.0);
}

#[test]
fn test_inertia_decay_clamping() {
    let mut settings = CameraSettings::default();
    
    // Set inertia decay outside [0, 1]
    settings.inertia_decay = -0.5;
    settings.validate();
    assert!(settings.inertia_decay >= 0.0 && settings.inertia_decay <= 1.0);
    
    settings.inertia_decay = 1.5;
    settings.validate();
    assert!(settings.inertia_decay >= 0.0 && settings.inertia_decay <= 1.0);
}

#[test]
fn test_zoom_speed_clamping() {
    let mut settings = CameraSettings::default();
    
    // Set zoom speed outside reasonable range
    settings.zoom_speed = 0.5;
    settings.validate();
    assert!(settings.zoom_speed >= 1.0);
    
    settings.zoom_speed = 200.0;
    settings.validate();
    assert!(settings.zoom_speed <= 100.0);
}

// ============================================================================
// NaN/INF HANDLING TESTS
// ============================================================================

#[test]
fn test_nan_sensitivity_detection() {
    let mut settings = CameraSettings::default();
    settings.pan_sensitivity = f32::NAN;
    
    assert!(!settings.is_valid());
}

#[test]
fn test_inf_sensitivity_detection() {
    let mut settings = CameraSettings::default();
    settings.rotation_sensitivity = f32::INFINITY;
    
    assert!(!settings.is_valid());
}

#[test]
fn test_nan_in_zoom() {
    let mut camera = SceneCamera::new();
    let initial_zoom = camera.zoom;
    
    // Try to zoom with NaN delta
    camera.zoom(f32::NAN, Vec2::ZERO);
    
    // Zoom should remain unchanged
    assert_eq!(camera.zoom, initial_zoom);
}

#[test]
fn test_nan_in_mouse_position() {
    let mut camera = SceneCamera::new();
    let initial_position = camera.position;
    
    // Try to pan with NaN mouse position
    camera.start_pan(Vec2::new(f32::NAN, 0.0));
    camera.update_pan(Vec2::new(100.0, 100.0));
    
    // Position should remain unchanged (pan should not have started)
    assert_eq!(camera.position, initial_position);
}

#[test]
fn test_inf_in_zoom_delta() {
    let mut camera = SceneCamera::new();
    let initial_zoom = camera.zoom;
    
    // Try to zoom with infinite delta
    camera.zoom(f32::INFINITY, Vec2::ZERO);
    
    // Zoom should remain unchanged
    assert_eq!(camera.zoom, initial_zoom);
}

#[test]
fn test_nan_in_update_delta_time() {
    let mut camera = SceneCamera::new();
    let initial_position = camera.position;
    
    // Start panning to create some velocity
    camera.start_pan(Vec2::ZERO);
    camera.update_pan(Vec2::new(100.0, 100.0));
    camera.stop_pan();
    
    // Update with NaN delta time
    camera.update(f32::NAN);
    
    // Position should remain valid (not NaN)
    assert!(camera.position.is_finite());
}

#[test]
fn test_negative_delta_time() {
    let mut camera = SceneCamera::new();
    let initial_position = camera.position;
    
    // Start panning to create some velocity
    camera.start_pan(Vec2::ZERO);
    camera.update_pan(Vec2::new(100.0, 100.0));
    camera.stop_pan();
    
    // Update with negative delta time
    camera.update(-0.016);
    
    // Position should remain valid (not changed by negative delta)
    assert!(camera.position.is_finite());
}

// ============================================================================
// EXTREME ZOOM LEVEL TESTS
// ============================================================================

#[test]
fn test_zoom_at_minimum_limit() {
    let mut camera = SceneCamera::new();
    camera.zoom = camera.min_zoom;
    
    // Try to zoom out further
    camera.zoom(-1.0, Vec2::ZERO);
    
    // Should remain at minimum
    assert_eq!(camera.zoom, camera.min_zoom);
}

#[test]
fn test_zoom_at_maximum_limit() {
    let mut camera = SceneCamera::new();
    camera.zoom = camera.max_zoom;
    
    // Try to zoom in further
    camera.zoom(1.0, Vec2::ZERO);
    
    // Should remain at maximum
    assert_eq!(camera.zoom, camera.max_zoom);
}

#[test]
fn test_extreme_zoom_out() {
    let mut camera = SceneCamera::new();
    
    // Zoom out many times
    for _ in 0..1000 {
        camera.zoom(-1.0, Vec2::ZERO);
    }
    
    // Should be clamped to minimum
    assert!(camera.zoom >= camera.min_zoom);
    assert!(camera.zoom.is_finite());
}

#[test]
fn test_extreme_zoom_in() {
    let mut camera = SceneCamera::new();
    
    // Zoom in many times
    for _ in 0..1000 {
        camera.zoom(1.0, Vec2::ZERO);
    }
    
    // Should be clamped to maximum
    assert!(camera.zoom <= camera.max_zoom);
    assert!(camera.zoom.is_finite());
}

#[test]
fn test_zoom_to_cursor_at_limits() {
    let mut camera = SceneCamera::new();
    camera.zoom = camera.min_zoom;
    
    let cursor_pos = Vec2::new(100.0, 100.0);
    let viewport_center = Vec2::ZERO;
    
    // Try to zoom out at minimum
    camera.zoom_to_cursor(-1.0, cursor_pos, viewport_center);
    assert_eq!(camera.zoom, camera.min_zoom);
    
    // Zoom to maximum
    camera.zoom = camera.max_zoom;
    
    // Try to zoom in at maximum
    camera.zoom_to_cursor(1.0, cursor_pos, viewport_center);
    assert_eq!(camera.zoom, camera.max_zoom);
}

// ============================================================================
// GRID SPACING BOUNDS TESTS
// ============================================================================

#[test]
fn test_grid_spacing_with_zero_zoom() {
    let grid = InfiniteGrid::new();
    
    // Calculate grid level with zero zoom
    let spacing = grid.calculate_grid_level(0.0);
    
    // Should return base unit (fallback)
    assert_eq!(spacing, grid.base_unit);
}

#[test]
fn test_grid_spacing_with_nan_zoom() {
    let grid = InfiniteGrid::new();
    
    // Calculate grid level with NaN zoom
    let spacing = grid.calculate_grid_level(f32::NAN);
    
    // Should return base unit (fallback)
    assert_eq!(spacing, grid.base_unit);
}

#[test]
fn test_grid_spacing_with_inf_zoom() {
    let grid = InfiniteGrid::new();
    
    // Calculate grid level with infinite zoom
    let spacing = grid.calculate_grid_level(f32::INFINITY);
    
    // Should return a valid finite value
    assert!(spacing.is_finite());
    assert!(spacing > 0.0);
}

#[test]
fn test_grid_spacing_with_extreme_zoom() {
    let grid = InfiniteGrid::new();
    
    // Test with very small zoom
    let spacing_small = grid.calculate_grid_level(0.0001);
    assert!(spacing_small.is_finite());
    assert!(spacing_small > 0.0);
    assert!(spacing_small <= 10000.0);
    
    // Test with very large zoom
    let spacing_large = grid.calculate_grid_level(10000.0);
    assert!(spacing_large.is_finite());
    assert!(spacing_large > 0.0);
    assert!(spacing_large >= 0.001);
}

#[test]
fn test_scene_grid_spacing_with_invalid_zoom() {
    let grid = SceneGrid::new();
    
    // Test with NaN
    let spacing = grid.select_grid_level(f32::NAN);
    assert_eq!(spacing, grid.size);
    
    // Test with negative
    let spacing = grid.select_grid_level(-1.0);
    assert_eq!(spacing, grid.size);
    
    // Test with zero
    let spacing = grid.select_grid_level(0.0);
    assert_eq!(spacing, grid.size);
}

// ============================================================================
// PROJECTION ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_fade_alpha_with_nan_point() {
    let grid = InfiniteGrid::new();
    let camera_pos = glam::Vec3::new(0.0, 10.0, 0.0);
    let invalid_point = glam::Vec3::new(f32::NAN, 0.0, 0.0);
    
    // Should return 0.0 for invalid point
    let alpha = grid.calculate_fade_alpha(invalid_point, camera_pos);
    assert_eq!(alpha, 0.0);
}

#[test]
fn test_fade_alpha_with_nan_camera() {
    let grid = InfiniteGrid::new();
    let point = glam::Vec3::new(0.0, 0.0, 0.0);
    let invalid_camera = glam::Vec3::new(f32::NAN, 0.0, 0.0);
    
    // Should return 0.0 for invalid camera position
    let alpha = grid.calculate_fade_alpha(point, invalid_camera);
    assert_eq!(alpha, 0.0);
}

#[test]
fn test_fade_alpha_with_inf_distance() {
    let grid = InfiniteGrid::new();
    let point = glam::Vec3::new(0.0, 0.0, 0.0);
    let far_camera = glam::Vec3::new(f32::INFINITY, 0.0, 0.0);
    
    // Should return 0.0 for infinite distance
    let alpha = grid.calculate_fade_alpha(point, far_camera);
    assert_eq!(alpha, 0.0);
}

#[test]
fn test_fade_alpha_bounds() {
    let grid = InfiniteGrid::new();
    let camera_pos = glam::Vec3::new(0.0, 10.0, 0.0);
    
    // Test various distances
    for distance in [0.0, 10.0, 100.0, 500.0, 1000.0, 2000.0] {
        let point = glam::Vec3::new(distance, 0.0, 0.0);
        let alpha = grid.calculate_fade_alpha(point, camera_pos);
        
        // Alpha should always be in [0, 1]
        assert!(alpha >= 0.0 && alpha <= 1.0, "Alpha {} out of bounds for distance {}", alpha, distance);
        assert!(alpha.is_finite());
    }
}

#[test]
fn test_generate_geometry_with_invalid_camera_state() {
    let mut grid = InfiniteGrid::new();
    
    // Create invalid camera state with NaN
    let invalid_camera = CameraState {
        position: Vec2::new(f32::NAN, 0.0),
        rotation: 0.0,
        pitch: 0.0,
        zoom: 1.0,
    };
    
    let viewport_size = Vec2::new(800.0, 600.0);
    
    // Should return empty geometry without crashing
    let geometry = grid.generate_geometry(&invalid_camera, viewport_size);
    assert!(geometry.lines.is_empty());
}

#[test]
fn test_generate_geometry_with_invalid_zoom() {
    let mut grid = InfiniteGrid::new();
    
    // Create camera state with invalid zoom
    let invalid_camera = CameraState {
        position: Vec2::ZERO,
        rotation: 0.0,
        pitch: 0.0,
        zoom: f32::NAN,
    };
    
    let viewport_size = Vec2::new(800.0, 600.0);
    
    // Should return empty geometry without crashing
    let geometry = grid.generate_geometry(&invalid_camera, viewport_size);
    assert!(geometry.lines.is_empty());
}

#[test]
fn test_generate_geometry_with_invalid_angles() {
    let mut grid = InfiniteGrid::new();
    
    // Create camera state with invalid rotation
    let invalid_camera = CameraState {
        position: Vec2::ZERO,
        rotation: f32::INFINITY,
        pitch: 30.0,
        zoom: 1.0,
    };
    
    let viewport_size = Vec2::new(800.0, 600.0);
    
    // Should return empty geometry without crashing
    let geometry = grid.generate_geometry(&invalid_camera, viewport_size);
    assert!(geometry.lines.is_empty());
}

// ============================================================================
// CURSOR POSITION VALIDATION TESTS
// ============================================================================

#[test]
fn test_zoom_with_invalid_cursor_position() {
    let mut camera = SceneCamera::new();
    let initial_zoom = camera.zoom;
    
    // Try to zoom with invalid cursor positions
    camera.zoom(1.0, Vec2::new(f32::NAN, 100.0));
    assert_eq!(camera.zoom, initial_zoom);
    
    camera.zoom(1.0, Vec2::new(100.0, f32::INFINITY));
    assert_eq!(camera.zoom, initial_zoom);
}

#[test]
fn test_zoom_to_cursor_with_invalid_viewport() {
    let mut camera = SceneCamera::new();
    let initial_zoom = camera.zoom;
    
    let cursor_pos = Vec2::new(100.0, 100.0);
    let invalid_viewport = Vec2::new(f32::NAN, 600.0);
    
    // Should not crash and zoom should remain unchanged
    camera.zoom_to_cursor(1.0, cursor_pos, invalid_viewport);
    assert_eq!(camera.zoom, initial_zoom);
}

#[test]
fn test_screen_to_world_with_extreme_zoom() {
    let mut camera = SceneCamera::new();
    
    // Test with very small zoom
    camera.zoom = 0.001;
    let world_pos = camera.screen_to_world(Vec2::new(100.0, 100.0));
    assert!(world_pos.is_finite());
    
    // Test with very large zoom
    camera.zoom = 100.0;
    let world_pos = camera.screen_to_world(Vec2::new(100.0, 100.0));
    assert!(world_pos.is_finite());
}

// ============================================================================
// SETTINGS PERSISTENCE VALIDATION TESTS
// ============================================================================

#[test]
fn test_load_settings_validates_values() {
    // This test would require file I/O mocking, so we test the validation logic directly
    let mut settings = CameraSettings::default();
    
    // Simulate loading invalid settings
    settings.pan_sensitivity = 100.0;
    settings.rotation_sensitivity = -5.0;
    settings.zoom_sensitivity = f32::NAN;
    
    // Check if invalid
    assert!(!settings.is_valid());
    
    // Validate should fix it (NaN values get clamped to MIN_SENSITIVITY)
    settings.validate();
    assert!(settings.pan_sensitivity <= CameraSettings::MAX_SENSITIVITY);
    assert!(settings.rotation_sensitivity >= CameraSettings::MIN_SENSITIVITY);
    // NaN gets clamped to MIN_SENSITIVITY by clamp()
    assert!(settings.zoom_sensitivity.is_finite());
    assert!(settings.zoom_sensitivity >= CameraSettings::MIN_SENSITIVITY || settings.zoom_sensitivity.is_nan());
}
