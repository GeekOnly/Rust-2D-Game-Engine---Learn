use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider3D {
    /// Offset from entity center (in local space)
    pub offset: [f32; 3],
    /// Size of collider (Width, Height, Depth)
    pub size: [f32; 3],
    /// Shape type
    pub shape: ColliderShape3D,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ColliderShape3D {
    Box,
    Sphere,
    Capsule,
}

impl Default for Collider3D {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            shape: ColliderShape3D::Box,
        }
    }
}

impl Collider3D {
    pub fn new_box(size: [f32; 3]) -> Self {
        Self {
            offset: [0.0, 0.0, 0.0],
            size,
            shape: ColliderShape3D::Box,
        }
    }

    pub fn new_box_with_offset(size: [f32; 3], offset: [f32; 3]) -> Self {
        Self {
            offset,
            size,
            shape: ColliderShape3D::Box,
        }
    }
}
