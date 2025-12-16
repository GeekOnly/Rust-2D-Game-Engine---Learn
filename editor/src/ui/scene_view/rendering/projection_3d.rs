//! 3D Projection System
//!
//! Provides 3D transformation, projection, and ray casting functionality
//! for rendering sprites and tilemaps in 3D space.

use glam::{Vec2, Vec3, Vec4, Mat4, Vec4Swizzles};

use crate::SceneCamera;

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
    pub fov: f32,      // Field of view in radians (perspective) or size (orthographic)
    pub aspect: f32,   // Aspect ratio (width / height)
    pub near: f32,     // Near clipping plane
    pub far: f32,      // Far clipping plane
    pub is_orthographic: bool, // True for orthographic, false for perspective
}

impl ProjectionMatrix {
    /// Create perspective projection matrix
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            aspect,
            near,
            far,
            is_orthographic: false,
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
    
    /// Create orthographic projection matrix
    pub fn orthographic(size: f32, aspect: f32, near: f32, far: f32) -> Self {
        // For orthographic projection, we store the size instead of FOV
        // The size represents the height of the orthographic view volume
        Self {
            fov: size,      // Store orthographic size in fov field
            aspect,
            near,
            far,
            is_orthographic: true,
        }
    }
    
    /// Create default orthographic projection
    pub fn default_orthographic(size: f32, aspect: f32) -> Self {
        Self::orthographic(
            size,       // Orthographic size (height of view volume)
            aspect,
            0.1,        // Near plane
            10000.0,    // Far plane
        )
    }
    
    /// Get the projection matrix as Mat4
    pub fn to_matrix(&self) -> Mat4 {
        if self.is_orthographic {
            // Orthographic projection - fov field contains the size
            let height = self.fov;
            let width = height * self.aspect;
            Mat4::orthographic_rh(-width/2.0, width/2.0, -height/2.0, height/2.0, self.near, self.far)
        } else {
            // Perspective projection
            Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
        }
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
        if !point.is_finite() {
            eprintln!("Warning: Invalid point in projection");
            return None;
        }
        
        if !viewport_size.is_finite() {
            eprintln!("Warning: Invalid viewport size in projection");
            return None;
        }
        
        // Check for zero or negative viewport dimensions
        if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
            eprintln!("Warning: Invalid viewport dimensions in projection");
            return None;
        }
        
        // Check for extreme viewport sizes (likely overflow)
        if viewport_size.x > 100000.0 || viewport_size.y > 100000.0 {
            return None;
        }
        
        // Validate projection parameters
        if !self.fov.is_finite() || !self.aspect.is_finite() || 
           !self.near.is_finite() || !self.far.is_finite() {
            eprintln!("Warning: Invalid projection parameters");
            return None;
        }
        
        // Check for invalid projection parameters
        if self.fov <= 0.0 || self.aspect <= 0.0 || 
           self.near <= 0.0 || self.far <= self.near {
            eprintln!("Warning: Invalid projection parameter values");
            return None;
        }
        
        // Create projection matrix
        let proj_matrix = self.to_matrix();
        
        // Validate matrices
        if !view_matrix.is_finite() || !proj_matrix.is_finite() {
            eprintln!("Warning: Invalid matrices in projection");
            return None;
        }
        
        // Transform point to clip space
        let clip_space = proj_matrix * (*view_matrix) * Vec4::from((point, 1.0));
        
        // Validate clip space
        if !clip_space.is_finite() {
            return None;
        }
        
        // Check if point is behind camera (negative W)
        if clip_space.w <= 0.0 {
            return None;
        }
        
        // Check for extreme W values (likely overflow)
        if clip_space.w.abs() > 1000000.0 {
            return None;
        }
        
        // Perspective divide to get NDC coordinates
        let ndc = clip_space.xyz() / clip_space.w;
        
        // Check if point is within NDC bounds [-1, 1]
        if !ndc.is_finite() {
            return None;
        }
        
        // Check for extreme NDC values (likely overflow)
        if ndc.x.abs() > 100.0 || ndc.y.abs() > 100.0 || ndc.z.abs() > 100.0 {
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
        
        // Check for extreme screen positions (likely overflow)
        if screen_pos.x.abs() > 1000000.0 || screen_pos.y.abs() > 1000000.0 {
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
            eprintln!("Warning: Invalid AABB bounds in intersection test");
            return None;
        }
        
        // Validate ray
        if !self.origin.is_finite() || !self.direction.is_finite() {
            eprintln!("Warning: Invalid ray in AABB intersection test");
            return None;
        }
        
        // Check for degenerate AABB (min >= max)
        if min.x >= max.x || min.y >= max.y || min.z >= max.z {
            return None;
        }
        
        // Slab method for ray-AABB intersection
        let inv_dir = Vec3::new(
            if self.direction.x != 0.0 { 1.0 / self.direction.x } else { f32::INFINITY },
            if self.direction.y != 0.0 { 1.0 / self.direction.y } else { f32::INFINITY },
            if self.direction.z != 0.0 { 1.0 / self.direction.z } else { f32::INFINITY },
        );
        
        // Note: inv_dir can contain INFINITY values, which is correct for the slab method
        // when the ray is parallel to an axis. We don't validate is_finite() here.
        
        let t1 = (min - self.origin) * inv_dir;
        let t2 = (max - self.origin) * inv_dir;
        
        // Note: t1 and t2 can contain INFINITY values when inv_dir contains INFINITY
        // This is correct for the slab method. We don't validate is_finite() here.
        
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        
        let t_near = tmin.x.max(tmin.y).max(tmin.z);
        let t_far = tmax.x.min(tmax.y).min(tmax.z);
        
        // Validate final t values
        if !t_near.is_finite() || !t_far.is_finite() {
            return None;
        }
        
        // Check if ray intersects AABB
        if t_near > t_far || t_far < 0.0 {
            return None;
        }
        
        // Return nearest intersection point
        let result = if t_near < 0.0 { t_far } else { t_near };
        
        // Validate result
        if !result.is_finite() || result < 0.0 {
            return None;
        }
        
        Some(result)
    }
    
    /// Test intersection with a plane defined by a point and normal
    /// Returns Some(t) where t is the distance along the ray to the intersection point
    pub fn intersect_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
        // Validate inputs
        if !plane_point.is_finite() || !plane_normal.is_finite() {
            eprintln!("Warning: Invalid plane parameters in intersection test");
            return None;
        }
        
        // Validate ray
        if !self.origin.is_finite() || !self.direction.is_finite() {
            eprintln!("Warning: Invalid ray in plane intersection test");
            return None;
        }
        
        // Check for degenerate plane normal (zero length)
        if plane_normal.length_squared() < 1e-10 {
            return None;
        }
        
        let denom = self.direction.dot(plane_normal);
        
        // Validate denom
        if !denom.is_finite() {
            return None;
        }
        
        // Check if ray is parallel to plane
        if denom.abs() < 1e-6 {
            return None;
        }
        
        let numerator = (plane_point - self.origin).dot(plane_normal);
        
        // Validate numerator
        if !numerator.is_finite() {
            return None;
        }
        
        let t = numerator / denom;
        
        // Validate t
        if !t.is_finite() {
            return None;
        }
        
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
        if !center.is_finite() {
            eprintln!("Warning: Invalid sphere center in intersection test");
            return None;
        }
        
        if !radius.is_finite() || radius <= 0.0 {
            eprintln!("Warning: Invalid sphere radius in intersection test");
            return None;
        }
        
        // Check for extreme radius values
        if radius > 1000000.0 {
            return None;
        }
        
        // Validate ray
        if !self.origin.is_finite() || !self.direction.is_finite() {
            eprintln!("Warning: Invalid ray in sphere intersection test");
            return None;
        }
        
        let oc = self.origin - center;
        
        // Validate oc
        if !oc.is_finite() {
            return None;
        }
        
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        
        // Validate coefficients
        if !a.is_finite() || !b.is_finite() || !c.is_finite() {
            return None;
        }
        
        // Check for degenerate ray direction (zero length)
        if a < 1e-10 {
            return None;
        }
        
        let discriminant = b * b - 4.0 * a * c;
        
        // Validate discriminant
        if !discriminant.is_finite() {
            return None;
        }
        
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_discriminant = discriminant.sqrt();
        
        // Validate sqrt
        if !sqrt_discriminant.is_finite() {
            return None;
        }
        
        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);
        
        // Validate t values
        if !t1.is_finite() || !t2.is_finite() {
            return None;
        }
        
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
        camera.get_projection_matrix(aspect)
    } else {
        camera.get_projection_matrix(aspect)
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

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

    projection.project(world_pos, &view_matrix, viewport_size)
}

/// Project world position to screen space, allowing points behind camera
/// This is useful for rendering gizmos that should always be visible
pub fn world_to_screen_allow_behind(
    world_pos: Vec3,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // Validate inputs
    if !world_pos.is_finite() || !viewport_size.is_finite() {
        return None;
    }

    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return None;
    }

    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

    // Create projection matrix
    let proj_matrix = projection.to_matrix();
    
    // Validate matrices
    if !view_matrix.is_finite() || !proj_matrix.is_finite() {
        return None;
    }
    
    // Transform point to clip space
    let clip_space = proj_matrix * view_matrix * Vec4::from((world_pos, 1.0));
    
    // Validate clip space
    if !clip_space.is_finite() {
        return None;
    }
    
    // Handle points behind camera more gracefully
    let w = if clip_space.w.abs() < 0.001 { 
        // Very close to camera or behind - use a small positive value
        if clip_space.w >= 0.0 { 0.001 } else { -0.001 }
    } else { 
        clip_space.w 
    };
    
    // Perspective divide
    let ndc = clip_space.xyz() / w;
    
    // For points behind camera (negative w), we might get inverted coordinates
    // Clamp NDC to reasonable bounds to prevent extreme values
    let clamped_ndc = Vec3::new(
        ndc.x.clamp(-10.0, 10.0),
        ndc.y.clamp(-10.0, 10.0),
        ndc.z.clamp(-10.0, 10.0),
    );
    
    // Convert NDC to screen space
    let screen_x = (clamped_ndc.x + 1.0) * 0.5 * viewport_size.x;
    let screen_y = (1.0 - clamped_ndc.y) * 0.5 * viewport_size.y;
    
    let screen_pos = Vec2::new(screen_x, screen_y);
    
    // Validate screen position
    if !screen_pos.is_finite() {
        return None;
    }
    
    // Allow wider range for gizmos (they can be off-screen)
    if screen_pos.x.abs() > 10000.0 || screen_pos.y.abs() > 10000.0 {
        return None;
    }
    
    Some(screen_pos)
}

/// Unproject screen position to 3D ray
pub fn screen_to_ray(
    screen_pos: Vec2,
    camera: &SceneCamera,
    viewport_size: Vec2,
) -> Ray3D {
    let view_matrix = calculate_view_matrix(camera);
    let aspect = viewport_size.x / viewport_size.y;

    // Use camera's projection mode
    let projection = match camera.projection_mode {
        crate::SceneProjectionMode::Perspective => {
            ProjectionMatrix::default_perspective(aspect)
        }
        crate::SceneProjectionMode::Isometric => {
            ProjectionMatrix::default_orthographic(camera.zoom, aspect)
        }
    };

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
