/// Unit tests for error handling and validation
/// Tests invalid sensitivity values, NaN/Inf handling, extreme zoom levels, and projection edge cases

use glam::Vec2;

// Note: Camera tests moved to editor crate where the camera module exists
// Note: Grid tests moved to editor crate where the grid module exists

// ============================================================================
// BASIC ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_vec2_nan_detection() {
    let nan_vec = Vec2::new(f32::NAN, 0.0);
    assert!(!nan_vec.is_finite());
}

#[test]
fn test_vec2_inf_detection() {
    let inf_vec = Vec2::new(f32::INFINITY, 0.0);
    assert!(!inf_vec.is_finite());
}

#[test]
fn test_vec2_valid() {
    let valid_vec = Vec2::new(1.0, 2.0);
    assert!(valid_vec.is_finite());
}

// Additional error handling tests can be added here as needed