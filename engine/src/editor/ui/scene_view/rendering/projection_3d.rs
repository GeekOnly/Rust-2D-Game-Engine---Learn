//! 3D Projection System
//!
//! Provides 3D transformation, projection, and ray casting functionality
//! for rendering sprites and tilemaps in 3D space.

use glam::{Vec2, Vec3, Vec4, Mat4, Vec4Swizzles};

use crate::editor::camera::SceneCamera;

// ============================================================================
// TRANSFORM 3D
// ============================================================================

/// Represents a 3D transform for rendering
#[derive(Clone, Copy, Debug)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: f32,  // Rotation around Y axis (yaw) in radians
    pub scale: Vec2,
}

impl Transform3D {
    /// Create a new Transform3D
    pub fn new(position: Vec3, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
    
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
    
    /// Convert to 4x4 transformation matrix
    pub fn to_matrix(&self) -> Mat4 {
        // Create transformation matrix: Translation * Rotation * Scale
        let translation = Mat4::from_translation(self.position);
        let rotation = Mat4::from_rotation_y(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.0));
        
        translation * rotation * scale
    }
    
    /// Calculate depth from camera (distance along camera's forward direction)
    pub fn depth_from_camera(&self, camera: &SceneCamera) -> f32 {
        // Get camera position in 3D space
        let yaw_rad = camera.rotation.to_radians();
        let pitch_rad = camera.pitch.to_radians();
        
        let cam_x = camera.position.x + camera.distance * yaw_rad.cos() * pitch_rad.cos();
        let cam_y = camera.distance * pitch_rad.sin();
        let cam_z = camera.position.y + camera.distance * yaw_rad.sin() * pitch_rad.cos();
        
        let camera_pos = Vec3::new(cam_x, cam_y, cam_z);
        
        // Calculate distance from camera to object
        (self.position - camera_pos).length()
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

// ============================================================================
// PROJECTION MATRIX
// ============================================================================

/// Projection matrix for 3D rendering
#[derive(Clone, Copy, Debug)]
pub struct ProjectionMatrix {
    pub fov: f32,      // Field of view in radians
    pub aspect: f32,   // Aspect ratio (width / height)
    pub near: f32,     // Near clipping plane
    pub far: f32,      // Far clipping plane
}

impl ProjectionMatrix {
    /// Create perspective projection matrix
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
        }
    }
    
    /// Create default perspective projection
    pub fn default_perspective(aspect: f32) -> Self {
        Self::perspective(
            60.0_f32.to_radians(),  // 60 degree FOV
            aspect,
            0.1,                     // Near plane
            10000.0,                 // Far plane
        )
    }
    
    /// Get the projection matrix as Mat4
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
    
    /// Project 3D point to screen space
    /// Returns None if point is behind camera or projection fails
    pub fn project(
        &self,
        point: Vec3,
        view_matrix: &Mat4,
        viewport_size: Vec2,
    ) -> Option<Vec2> {
        // Validate inputs
        if !point.is_finite() || !viewport_size.is_finite() {
            return None;
        }
        
        // Create projection matrix
        let proj_matrix = self.to_matrix();
        
        // Transform point to clip space
        let clip_space = proj_matrix * (*view_matrix) * Vec4::from((point, 1.0));
        
        // Check if point is behind camera (negative W)
        if clip_space.w <= 0.0 {
            return None;
        }
        
        // Perspective divide to get NDC coordinates
        let ndc = clip_space.xyz() / clip_space.w;
        
        // Check if point is within NDC bounds [-1, 1]
        if !ndc.is_finite() {
            return None;
        }
        
        // Convert NDC to screen space
        // NDC: (-1, -1) = bottom-left, (1, 1) = top-right
        // Screen: (0, 0) = top-left, (width, height) = bottom-right
        let screen_x = (ndc.x + 1.0) * 0.5 * viewport_size.x;
        let screen_y = (1.0 - ndc.y) * 0.5 * viewport_size.y;
        
        let screen_pos = Vec2::new(screen_x, screen_y);
        
        // Validate screen position
        if !screen_pos.is_finite() {
            return None;
        }
        
        Some(screen_pos)
    }
    
    /// Unproject screen point to 3D ray
    pub fn unproject(
        &self,
        screen_pos: Vec2,
        view_matrix: &Mat4,
        viewport_size: Vec2,
    ) -> Ray3D {
        // Convert screen space to NDC
        let ndc_x = (screen_pos.x / viewport_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / viewport_size.y) * 2.0;
        
        // Create ray in clip space (near and far points)
        let near_clip = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_clip = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        
        // Get inverse matrices
        let proj_matrix = self.to_matrix();
        let inv_proj = proj_matrix.inverse();
        let inv_view = view_matrix.inverse();
        
        // Transform to world space
        let near_world = inv_view * (inv_proj * near_clip);
        let far_world = inv_view * (inv_proj * far_clip);
        
        // Perspective divide
        let near_world = near_world.xyz() / near_world.w;
        let far_world = far_world.xyz() / far_world.w;
        
        // Create ray
        let origin = near_world;
        let direction = (far_world - near_world).normalize();
        
        Ray3D { origin, direction }
    }
}

impl Default for ProjectionMatrix {
    fn default() -> Self {
        Self::default_perspective(16.0 / 9.0)
    }
}

// ============================================================================
// RAY 3D
// ============================================================================

/// 3D ray for picking and intersection tests
#[derive(Clone, Copy, Debug)]
pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3,  // Should be normalized
}

impl Ray3D {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
    
    /// Get point along ray at distance t
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
    
    /// Test intersection with axis-aligned bounding box (AABB)
    /// Returns Some(t) where t is the distance along the ray to the intersection point
    pub fn intersect_aabb(&self, min: Vec3, max: Vec3) -> Option<f32> {
        // Validate inputs
        if !min.is_finite() || !max.is_finite() {
            return None;
        }
        
        // Slab method for ray-AABB intersection
        let inv_dir = Vec3::new(
            if self.direction.x != 0.0 { 1.0 / self.direction.x } else { f32::INFINITY },
            if self.direction.y != 0.0 { 1.0 / self.direction.y } else { f32::INFINITY },
            if self.direction.z != 0.0 { 1.0 / self.direction.z } else { f32::INFINITY },
        );
        
        let t1 = (min - self.origin) * inv_dir;
        let t2 = (max - self.origin) * inv_dir;
        
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        
        let t_near = tmin.x.max(tmin.y).max(tmin.z);
        let t_far = tmax.x.min(tmax.y).min(tmax.z);
        
        // Check if ray intersects AABB
        if t_near > t_far || t_far < 0.0 {
            return None;
        }
        
        // Return nearest intersection point
        Some(if t_near < 0.0 { t_far } else { t_near })
    }
    
    /// Test intersection with a plane defined by a point and normal
    /// Returns Some(t) where t is the distance along the ray to the intersection point
    pub fn intersect_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
        // Validate inputs
        if !plane_point.is_finite() || !plane_normal.is_finite() {
            return None;
        }
        
        let denom = self.direction.dot(plane_normal);
        
        // Check if ray is parallel to plane
        if denom.abs() < 1e-6 {
            return None;
        }
        
        let t = (plane_point - self.origin).dot(plane_normal) / denom;
        
        // Check if intersection is behind ray origin
        if t < 0.0 {
            return None;
        }
        
        Some(t)
    }
    
    /// Test intersection with a sphere
    /// Returns Some(t) where t is the distance along the ray to the nearest intersection point
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        // Validate inputs
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
            return None;
        }
        
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_discriminant = discriminant.sqrt();
        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);
        
        // Return nearest positive intersection
        if t1 > 0.0 {
            Some(t1)
        } else if t2 > 0.0 {
            Some(t2)
        } else {
            None
        }
    }
}

impl Default for Ray3D {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        }
    }
}

// ============================================================================
// VIEW MATRIX CALCULATION
// ============================================================================

/// Calculate view matrix from camera
pub fn calculate_view_matrix(camera: &SceneCamera) -> Mat4 {
    camera.get_view_matrix()
}

/// Calculate projection matrix from camera and viewport
pub fn calculate_projection_matrix(
    camera: &SceneCamera,
    viewport_size: Vec2,
    perspective: bool,
) -> Mat4 {
    let aspect = viewport_size.x / viewport_size.y;
    
    if perspective {
        camera.get_projection_matrix(aspect, crate::editor::camera::ProjectionMode::Perspective)
    } else {
        camera.get_projection_matrix(aspect, crate::editor::camera::ProjectionMode::Isometric)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Project world position to screen space
pub fn world_to_screen(
    world_pos: Vec3,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Option<Vec2> {
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;
    let projection = ProjectionMatrix::default_perspective(aspect);
    
    projection.project(world_pos, &view_matrix, viewport_size)
}

/// Unproject screen position to 3D ray
pub fn screen_to_ray(
    screen_pos: Vec2,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Ray3D {
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;
    let projection = ProjectionMatrix::default_perspective(aspect);
    
    projection.unproject(screen_pos, &view_matrix, viewport_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_identity() {
        let transform = Transform3D::identity();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }
    
    #[test]
    fn test_transform_to_matrix() {
        let transform = Transform3D::new(
            Vec3::new(1.0, 2.0, 3.0),
            0.0,
            Vec2::ONE,
        );
        let matrix = transform.to_matrix();
        
        // Check that translation is correct
        let translated = matrix * Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((translated.x - 1.0).abs() < 0.001);
        assert!((translated.y - 2.0).abs() < 0.001);
        assert!((translated.z - 3.0).abs() < 0.001);
    }
    
    #[test]
    fn test_ray_point_at() {
        let ray = Ray3D::new(Vec3::ZERO, Vec3::Z);
        let point = ray.point_at(5.0);
        assert_eq!(point, Vec3::new(0.0, 0.0, 5.0));
    }
    
    #[test]
    fn test_ray_intersect_plane() {
        let ray = Ray3D::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let plane_point = Vec3::ZERO;
        let plane_normal = Vec3::Y;
        
        let t = ray.intersect_plane(plane_point, plane_normal);
        assert!(t.is_some());
        assert!((t.unwrap() - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_ray_intersect_sphere() {
        let ray = Ray3D::new(Vec3::new(0.0, 0.0, -10.0), Vec3::Z);
        let center = Vec3::ZERO;
        let radius = 1.0;
        
        let t = ray.intersect_sphere(center, radius);
        assert!(t.is_some());
        assert!((t.unwrap() - 9.0).abs() < 0.001);
    }
    
    #[test]
    fn test_ray_intersect_aabb() {
        let ray = Ray3D::new(Vec3::new(0.0, 0.0, -10.0), Vec3::Z);
        let min = Vec3::new(-1.0, -1.0, -1.0);
        let max = Vec3::new(1.0, 1.0, 1.0);
        
        let t = ray.intersect_aabb(min, max);
        assert!(t.is_some());
        assert!((t.unwrap() - 9.0).abs() < 0.001);
    }
    
    #[test]
    fn test_projection_matrix_default() {
        let proj = ProjectionMatrix::default();
        assert!(proj.fov > 0.0);
        assert!(proj.aspect > 0.0);
        assert!(proj.near > 0.0);
        assert!(proj.far > proj.near);
    }
}
