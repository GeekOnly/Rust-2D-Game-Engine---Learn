//! Types and Enums for Scene View
//!
//! This module contains all the type definitions, enums, and helper structures
//! used throughout the scene view system.

// ============================================================================
// ENUMS
// ============================================================================

/// Scene view mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneViewMode {
    Mode2D,
    Mode3D,
}

/// Projection mode for 3D view (Re-exported from camera)
pub use crate::SceneProjectionMode;

/// Transform space mode (Local or World)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformSpace {
    Local,
    World,
}

/// Snap mode (Crown Engine inspired)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapMode {
    Relative,  // Snap relative to drag start position
    Absolute,  // Snap to absolute grid positions
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Snap settings for transform operations
#[derive(Debug, Clone)]
pub struct SnapSettings {
    pub enabled: bool,
    pub mode: SnapMode,
    pub position_snap: f32,    // Grid size for position (e.g., 1.0)
    pub rotation_snap: f32,    // Degrees for rotation (e.g., 15.0)
    pub scale_snap: f32,       // Increment for scale (e.g., 0.1)
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: SnapMode::Absolute,
            position_snap: 1.0,
            rotation_snap: 15.0,
            scale_snap: 0.1,
        }
    }
}

/// 3D point structure for 3D transformations
#[derive(Clone, Copy, Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Rotate around X axis
    pub fn rotate_x(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Y axis
    pub fn rotate_y(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Z axis
    pub fn rotate_z(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }
    
    /// Apply full 3D rotation (XYZ order)
    pub fn rotate(&self, rotation: &[f32; 3]) -> Self {
        self.rotate_x(rotation[0].to_radians())
            .rotate_y(rotation[1].to_radians())
            .rotate_z(rotation[2].to_radians())
    }
    
    /// Project to 2D screen space with perspective
    pub fn project_perspective(&self, fov: f32, distance: f32) -> (f32, f32) {
        let z_offset = self.z + distance;
        if z_offset <= 10.0 {
            return (self.x, self.y);
        }
        
        let scale = fov / z_offset;
        (self.x * scale, self.y * scale)
    }
    
    /// Project to 2D screen space (isometric) - proper isometric angles
    pub fn project_isometric(&self) -> (f32, f32) {
        // True isometric projection (120° angles)
        // X and Z axes at 30° from horizontal
        let iso_x = (self.x - self.z) * 0.866; // cos(30°) ≈ 0.866
        let iso_y = self.y + (self.x + self.z) * 0.5; // sin(30°) = 0.5
        (iso_x, iso_y)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Snap value to grid based on snap mode
pub fn snap_to_grid(value: f32, grid_size: f32, mode: SnapMode, original: f32) -> f32 {
    if grid_size <= 0.0 {
        return value;
    }
    
    match mode {
        SnapMode::Absolute => {
            // Snap to absolute grid positions
            (value / grid_size).round() * grid_size
        }
        SnapMode::Relative => {
            // Snap relative to original position
            let delta = value - original;
            let snapped_delta = (delta / grid_size).round() * grid_size;
            original + snapped_delta
        }
    }
}

/// Helper function to rotate a 2D point around origin
pub fn rotate_point_2d(x: f32, y: f32, angle_rad: f32) -> (f32, f32) {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    (x * cos_a - y * sin_a, x * sin_a + y * cos_a)
}
