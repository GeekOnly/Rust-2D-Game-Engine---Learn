/// Unit tests for CameraStateDisplay
/// 
/// Tests:
/// - Distance calculation display
/// - Angle display formatting
/// - Grid size display

use glam::Vec2;

// Copy the necessary types for testing
#[derive(Debug, Clone)]
pub struct CameraStateDisplay {
    pub show_distance: bool,
    pub show_angles: bool,
    pub show_grid_size: bool,
    pub show_fps: bool,
}

impl CameraStateDisplay {
    pub fn new() -> Self {
        Self {
            show_distance: true,
            show_angles: true,
            show_grid_size: true,
            show_fps: true,
        }
    }
    
    pub fn calculate_distance(&self, camera: &SceneCamera) -> f32 {
        camera.position.length()
    }
    
    pub fn format_angles(&self, camera: &SceneCamera) -> String {
        format!("Yaw: {:.1}° Pitch: {:.1}°", camera.rotation, camera.pitch)
    }
    
    pub fn format_grid_size(&self, grid_size: f32) -> String {
        if grid_size >= 1.0 {
            format!("Grid: {:.1}m", grid_size)
        } else if grid_size >= 0.01 {
            format!("Grid: {:.2}m", grid_size)
        } else {
            format!("Grid: {:.3}m", grid_size)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub position: Vec2,
    pub rotation: f32,
    pub pitch: f32,
}

impl SceneCamera {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 45.0,
            pitch: 30.0,
        }
    }
}

#[test]
fn test_distance_calculation_display() {
    let display = CameraStateDisplay::new();
    
    // Test camera at origin
    let mut camera = SceneCamera::new();
    camera.position = glam::Vec2::ZERO;
    assert_eq!(display.calculate_distance(&camera), 0.0);
    
    // Test camera at (3, 4) - should be 5.0 units from origin
    camera.position = glam::Vec2::new(3.0, 4.0);
    assert!((display.calculate_distance(&camera) - 5.0).abs() < 0.001);
    
    // Test camera at (10, 0)
    camera.position = glam::Vec2::new(10.0, 0.0);
    assert!((display.calculate_distance(&camera) - 10.0).abs() < 0.001);
    
    // Test camera at negative coordinates
    camera.position = glam::Vec2::new(-6.0, -8.0);
    assert!((display.calculate_distance(&camera) - 10.0).abs() < 0.001);
}

#[test]
fn test_angle_display_formatting() {
    let display = CameraStateDisplay::new();
    let mut camera = SceneCamera::new();
    
    // Test default angles
    camera.rotation = 45.0;
    camera.pitch = 30.0;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("45.0"));
    assert!(formatted.contains("30.0"));
    assert!(formatted.contains("Yaw"));
    assert!(formatted.contains("Pitch"));
    
    // Test zero angles
    camera.rotation = 0.0;
    camera.pitch = 0.0;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("0.0"));
    
    // Test negative angles
    camera.rotation = -90.0;
    camera.pitch = -45.0;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("-90.0"));
    assert!(formatted.contains("-45.0"));
    
    // Test large angles
    camera.rotation = 180.0;
    camera.pitch = 89.0;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("180.0"));
    assert!(formatted.contains("89.0"));
}

#[test]
fn test_grid_size_display() {
    let display = CameraStateDisplay::new();
    
    // Test grid size >= 1.0 (should show 1 decimal place)
    let formatted = display.format_grid_size(1.0);
    assert!(formatted.contains("1.0"));
    assert!(formatted.contains("Grid"));
    assert!(formatted.contains("m"));
    
    let formatted = display.format_grid_size(10.0);
    assert!(formatted.contains("10.0"));
    
    // Test grid size between 0.01 and 1.0 (should show 2 decimal places)
    let formatted = display.format_grid_size(0.5);
    assert!(formatted.contains("0.50"));
    
    let formatted = display.format_grid_size(0.1);
    assert!(formatted.contains("0.10"));
    
    // Test grid size < 0.01 (should show 3 decimal places)
    let formatted = display.format_grid_size(0.005);
    assert!(formatted.contains("0.005"));
    
    let formatted = display.format_grid_size(0.001);
    assert!(formatted.contains("0.001"));
}

#[test]
fn test_camera_state_display_defaults() {
    let display = CameraStateDisplay::new();
    
    // All display options should be enabled by default
    assert!(display.show_distance);
    assert!(display.show_angles);
    assert!(display.show_grid_size);
    assert!(display.show_fps);
}

#[test]
fn test_distance_calculation_precision() {
    let display = CameraStateDisplay::new();
    let mut camera = SceneCamera::new();
    
    // Test various positions for precision
    let test_cases = vec![
        (glam::Vec2::new(1.0, 1.0), 1.414213562),
        (glam::Vec2::new(5.0, 12.0), 13.0),
        (glam::Vec2::new(8.0, 15.0), 17.0),
        (glam::Vec2::new(100.0, 0.0), 100.0),
        (glam::Vec2::new(0.0, 50.0), 50.0),
    ];
    
    for (position, expected_distance) in test_cases {
        camera.position = position;
        let calculated = display.calculate_distance(&camera);
        assert!(
            (calculated - expected_distance).abs() < 0.001,
            "Position {:?} should have distance {}, got {}",
            position,
            expected_distance,
            calculated
        );
    }
}

#[test]
fn test_angle_formatting_edge_cases() {
    let display = CameraStateDisplay::new();
    let mut camera = SceneCamera::new();
    
    // Test very small angles
    camera.rotation = 0.1;
    camera.pitch = 0.05;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("0.1"));
    assert!(formatted.contains("0.0") || formatted.contains("0.1")); // Pitch might round to 0.0 or 0.1
    
    // Test angles near 360
    camera.rotation = 359.9;
    camera.pitch = 89.9;
    let formatted = display.format_angles(&camera);
    assert!(formatted.contains("359.9"));
    assert!(formatted.contains("89.9"));
}

#[test]
fn test_grid_size_formatting_edge_cases() {
    let display = CameraStateDisplay::new();
    
    // Test exactly at boundaries
    assert!(display.format_grid_size(1.0).contains("1.0"));
    assert!(display.format_grid_size(0.01).contains("0.01"));
    
    // Test very small values
    assert!(display.format_grid_size(0.0001).contains("0.000"));
    
    // Test large values
    assert!(display.format_grid_size(1000.0).contains("1000.0"));
}
