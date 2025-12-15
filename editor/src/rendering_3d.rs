/// 3D rendering utilities for scene view
use glam::Vec3;

/// A 3D point with rotation and projection methods
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    /// Create a new 3D point
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Convert to glam Vec3
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    
    /// Create from glam Vec3
    pub fn from_vec3(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
    
    /// Rotate around X axis (pitch)
    pub fn rotate_x(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Y axis (yaw)
    pub fn rotate_y(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Z axis (roll)
    pub fn rotate_z(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }
    
    /// Rotate by Euler angles [yaw, pitch, roll] in radians
    pub fn rotate(&self, rotation: &[f32; 3]) -> Self {
        self.rotate_y(rotation[0])
            .rotate_x(rotation[1])
            .rotate_z(rotation[2])
    }
    
    /// Project using perspective projection
    /// Returns (screen_x, screen_y) coordinates
    /// fov: field of view in radians
    /// distance: distance from camera to projection plane
    pub fn project_perspective(&self, fov: f32, distance: f32) -> (f32, f32) {
        // Prevent division by zero or negative depth
        if self.z <= 0.01 {
            return (self.x * 1000.0, self.y * 1000.0); // Far away point
        }
        
        // Perspective projection: scale inversely with depth
        let scale = distance / self.z;
        let fov_scale = (fov / 2.0).tan();
        
        (self.x * scale * fov_scale, self.y * scale * fov_scale)
    }
    
    /// Project using isometric projection
    /// Returns (screen_x, screen_y) coordinates
    pub fn project_isometric(&self) -> (f32, f32) {
        // Standard isometric projection (dimetric 2:1)
        // X and Z contribute to horizontal position
        // Y contributes to vertical position
        let screen_x = (self.x - self.z) * 0.866; // cos(30°) ≈ 0.866
        let screen_y = self.y - (self.x + self.z) * 0.5;
        
        (screen_x, screen_y)
    }
}

/// A 3D face (triangle) with vertices
#[derive(Debug, Clone)]
pub struct Face3D {
    pub vertices: [Point3D; 3],
    pub normal: Vec3,
}

impl Face3D {
    /// Create a new face from three vertices
    pub fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        let normal = Self::calculate_normal(v0, v1, v2);
        Self {
            vertices: [v0, v1, v2],
            normal,
        }
    }
    
    /// Calculate face normal using cross product
    fn calculate_normal(v0: Point3D, v1: Point3D, v2: Point3D) -> Vec3 {
        let edge1 = Vec3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let edge2 = Vec3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
        edge1.cross(edge2).normalize_or_zero()
    }
    
    /// Check if face should be culled (back-face culling)
    /// Returns true if face is facing away from camera
    pub fn should_cull(&self, camera_direction: Vec3) -> bool {
        // If dot product > 0, face is pointing away from camera
        self.normal.dot(camera_direction) > 0.0
    }
    
    /// Get average depth of face (for depth sorting)
    pub fn average_depth(&self) -> f32 {
        (self.vertices[0].z + self.vertices[1].z + self.vertices[2].z) / 3.0
    }
    
    /// Transform face by rotation
    pub fn rotate(&self, rotation: &[f32; 3]) -> Self {
        Self {
            vertices: [
                self.vertices[0].rotate(rotation),
                self.vertices[1].rotate(rotation),
                self.vertices[2].rotate(rotation),
            ],
            normal: {
                let rotated_normal = Point3D::from_vec3(self.normal).rotate(rotation);
                rotated_normal.to_vec3().normalize_or_zero()
            },
        }
    }
}

/// Sort faces by depth (painter's algorithm)
/// Faces are sorted back-to-front (farthest first)
pub fn depth_sort_faces(faces: &mut [Face3D]) {
    faces.sort_by(|a, b| {
        // Sort in descending order (farthest first)
        b.average_depth().partial_cmp(&a.average_depth()).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_point3d_creation() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
    }
    
    #[test]
    fn test_rotation_x() {
        let p = Point3D::new(0.0, 1.0, 0.0);
        let rotated = p.rotate_x(std::f32::consts::PI / 2.0);
        assert!((rotated.x - 0.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
        assert!((rotated.z - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_rotation_y() {
        let p = Point3D::new(1.0, 0.0, 0.0);
        let rotated = p.rotate_y(std::f32::consts::PI / 2.0);
        assert!((rotated.x - 0.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
        assert!((rotated.z - -1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_perspective_projection() {
        let p = Point3D::new(10.0, 10.0, 100.0);
        let (x, y) = p.project_perspective(1.0, 100.0);
        // Point should be scaled down due to depth
        assert!(x.abs() < 10.0);
        assert!(y.abs() < 10.0);
    }
    
    #[test]
    fn test_back_face_culling() {
        // Face pointing toward camera (normal pointing at -Z)
        let v0 = Point3D::new(0.0, 0.0, 10.0);
        let v1 = Point3D::new(1.0, 0.0, 10.0);
        let v2 = Point3D::new(0.0, 1.0, 10.0);
        let face = Face3D::new(v0, v1, v2);
        
        // Camera looking down -Z axis
        let camera_dir = Vec3::new(0.0, 0.0, -1.0);
        assert!(!face.should_cull(camera_dir), "Front face should not be culled");
        
        // Camera looking down +Z axis (behind face)
        let camera_dir = Vec3::new(0.0, 0.0, 1.0);
        assert!(face.should_cull(camera_dir), "Back face should be culled");
    }
    
    #[test]
    fn test_depth_sorting() {
        let v0 = Point3D::new(0.0, 0.0, 10.0);
        let v1 = Point3D::new(1.0, 0.0, 10.0);
        let v2 = Point3D::new(0.0, 1.0, 10.0);
        let face1 = Face3D::new(v0, v1, v2);
        
        let v3 = Point3D::new(0.0, 0.0, 20.0);
        let v4 = Point3D::new(1.0, 0.0, 20.0);
        let v5 = Point3D::new(0.0, 1.0, 20.0);
        let face2 = Face3D::new(v3, v4, v5);
        
        let mut faces = vec![face1.clone(), face2.clone()];
        depth_sort_faces(&mut faces);
        
        // After sorting, farther face should be first
        assert!(faces[0].average_depth() > faces[1].average_depth());
    }
}
