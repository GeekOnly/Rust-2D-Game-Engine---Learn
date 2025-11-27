// Property-based tests for 3D rendering
// These tests validate the correctness properties defined in the unity-scene-view design document

use proptest::prelude::*;
use glam::Vec3;

// Copy the necessary types for testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
    
    pub fn from_vec3(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
    
    pub fn rotate_x(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }
    
    pub fn rotate_y(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }
    
    pub fn rotate_z(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }
    
    pub fn rotate(&self, rotation: &[f32; 3]) -> Self {
        self.rotate_y(rotation[0])
            .rotate_x(rotation[1])
            .rotate_z(rotation[2])
    }
    
    pub fn project_perspective(&self, fov: f32, distance: f32) -> (f32, f32) {
        if self.z <= 0.01 {
            return (self.x * 1000.0, self.y * 1000.0);
        }
        
        let scale = distance / self.z;
        let fov_scale = (fov / 2.0).tan();
        
        (self.x * scale * fov_scale, self.y * scale * fov_scale)
    }
    
    pub fn project_isometric(&self) -> (f32, f32) {
        let screen_x = (self.x - self.z) * 0.866;
        let screen_y = self.y - (self.x + self.z) * 0.5;
        
        (screen_x, screen_y)
    }
}

#[derive(Debug, Clone)]
pub struct Face3D {
    pub vertices: [Point3D; 3],
    pub normal: Vec3,
}

impl Face3D {
    pub fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        let normal = Self::calculate_normal(v0, v1, v2);
        Self {
            vertices: [v0, v1, v2],
            normal,
        }
    }
    
    fn calculate_normal(v0: Point3D, v1: Point3D, v2: Point3D) -> Vec3 {
        let edge1 = Vec3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let edge2 = Vec3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
        edge1.cross(edge2).normalize_or_zero()
    }
    
    pub fn should_cull(&self, camera_direction: Vec3) -> bool {
        self.normal.dot(camera_direction) > 0.0
    }
    
    pub fn average_depth(&self) -> f32 {
        (self.vertices[0].z + self.vertices[1].z + self.vertices[2].z) / 3.0
    }
    
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

pub fn depth_sort_faces(faces: &mut [Face3D]) {
    faces.sort_by(|a, b| {
        b.average_depth().partial_cmp(&a.average_depth()).unwrap_or(std::cmp::Ordering::Equal)
    });
}

// Helper functions for property testing
fn prop_point3d() -> impl Strategy<Value = Point3D> {
    (-1000.0f32..1000.0f32, -1000.0f32..1000.0f32, -1000.0f32..1000.0f32)
        .prop_map(|(x, y, z)| Point3D::new(x, y, z))
}

fn prop_point3d_positive_z() -> impl Strategy<Value = Point3D> {
    (-1000.0f32..1000.0f32, -1000.0f32..1000.0f32, 1.0f32..1000.0f32)
        .prop_map(|(x, y, z)| Point3D::new(x, y, z))
}

fn prop_fov() -> impl Strategy<Value = f32> {
    0.5f32..2.0f32 // Reasonable FOV range in radians
}

fn prop_distance() -> impl Strategy<Value = f32> {
    10.0f32..1000.0f32
}

fn prop_angle() -> impl Strategy<Value = f32> {
    -std::f32::consts::PI..std::f32::consts::PI
}

fn prop_camera_direction() -> impl Strategy<Value = Vec3> {
    (-1.0f32..1.0f32, -1.0f32..1.0f32, -1.0f32..1.0f32)
        .prop_map(|(x, y, z)| Vec3::new(x, y, z).normalize_or_zero())
        .prop_filter("Non-zero direction", |v| v.length() > 0.1)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: unity-scene-view, Property 14: Perspective projection scales with depth
    // Validates: Requirements 4.1
    #[test]
    fn prop_perspective_projection_scales_with_depth(
        x in -100.0f32..100.0f32,
        y in -100.0f32..100.0f32,
        z1 in 10.0f32..100.0f32,
        z2 in 101.0f32..500.0f32,
        fov in prop_fov(),
        distance in prop_distance(),
    ) {
        // Create two points at same X,Y but different depths
        let point_near = Point3D::new(x, y, z1);
        let point_far = Point3D::new(x, y, z2);
        
        let (screen_x1, screen_y1) = point_near.project_perspective(fov, distance);
        let (screen_x2, screen_y2) = point_far.project_perspective(fov, distance);
        
        // Calculate screen-space sizes
        let size_near = (screen_x1.powi(2) + screen_y1.powi(2)).sqrt();
        let size_far = (screen_x2.powi(2) + screen_y2.powi(2)).sqrt();
        
        // Objects farther away should appear smaller (or equal if at origin)
        if x.abs() > 0.1 || y.abs() > 0.1 {
            prop_assert!(
                size_far <= size_near * 1.01, // Small tolerance for floating point
                "Farther objects should appear smaller or equal. Near size: {}, Far size: {}",
                size_near,
                size_far
            );
            
            // Verify inverse relationship with depth
            // size should be proportional to 1/z
            let expected_ratio = z2 / z1;
            let actual_ratio = if size_far > 0.001 { size_near / size_far } else { 1.0 };
            
            let ratio_error = (actual_ratio - expected_ratio).abs() / expected_ratio;
            prop_assert!(
                ratio_error < 0.1,
                "Size should be inversely proportional to depth. Expected ratio: {}, Actual ratio: {}",
                expected_ratio,
                actual_ratio
            );
        }
    }
    
    // Feature: unity-scene-view, Property 15: Back-face culling hides non-visible faces
    // Validates: Requirements 4.2
    #[test]
    fn prop_backface_culling_hides_nonvisible_faces(
        v0 in prop_point3d_positive_z(),
        offset1_x in 0.1f32..10.0f32,
        offset2_y in 0.1f32..10.0f32,
        camera_dir in prop_camera_direction(),
    ) {
        // Create a face with vertices in counter-clockwise order (right-hand rule)
        let v1 = Point3D::new(v0.x + offset1_x, v0.y, v0.z);
        let v2 = Point3D::new(v0.x, v0.y + offset2_y, v0.z);
        
        let face = Face3D::new(v0, v1, v2);
        
        // Calculate if face is facing toward or away from camera
        let face_to_camera_dot = face.normal.dot(camera_dir);
        let should_be_culled = face.should_cull(camera_dir);
        
        // If dot product > 0, face normal points in same direction as camera (facing away)
        // and should be culled
        if face_to_camera_dot > 0.01 {
            prop_assert!(
                should_be_culled,
                "Face pointing away from camera should be culled. Dot product: {}",
                face_to_camera_dot
            );
        } else if face_to_camera_dot < -0.01 {
            prop_assert!(
                !should_be_culled,
                "Face pointing toward camera should not be culled. Dot product: {}",
                face_to_camera_dot
            );
        }
        
        // Verify consistency: culling decision should match dot product sign
        prop_assert_eq!(
            should_be_culled,
            face_to_camera_dot > 0.0,
            "Culling decision should be consistent with dot product"
        );
    }
    
    // Feature: unity-scene-view, Property 16: Faces are depth-sorted
    // Validates: Requirements 4.3
    #[test]
    fn prop_faces_are_depth_sorted(
        base_x in -100.0f32..100.0f32,
        base_y in -100.0f32..100.0f32,
        z_values in prop::collection::vec(10.0f32..1000.0f32, 3..10),
    ) {
        // Create faces at different depths
        let mut faces = Vec::new();
        for z in &z_values {
            let v0 = Point3D::new(base_x, base_y, *z);
            let v1 = Point3D::new(base_x + 1.0, base_y, *z);
            let v2 = Point3D::new(base_x, base_y + 1.0, *z);
            faces.push(Face3D::new(v0, v1, v2));
        }
        
        // Sort faces using depth sort
        depth_sort_faces(&mut faces);
        
        // Verify faces are sorted back-to-front (farthest first)
        for i in 0..faces.len() - 1 {
            let depth_current = faces[i].average_depth();
            let depth_next = faces[i + 1].average_depth();
            
            prop_assert!(
                depth_current >= depth_next - 0.001, // Small tolerance for floating point
                "Faces should be sorted back-to-front. Face {} depth: {}, Face {} depth: {}",
                i,
                depth_current,
                i + 1,
                depth_next
            );
        }
        
        // Verify the first face is the farthest
        if !faces.is_empty() {
            let first_depth = faces[0].average_depth();
            let max_depth = faces.iter().map(|f| f.average_depth()).fold(f32::NEG_INFINITY, f32::max);
            
            prop_assert!(
                (first_depth - max_depth).abs() < 0.001,
                "First face should be the farthest. First: {}, Max: {}",
                first_depth,
                max_depth
            );
        }
        
        // Verify the last face is the nearest
        if !faces.is_empty() {
            let last_depth = faces[faces.len() - 1].average_depth();
            let min_depth = faces.iter().map(|f| f.average_depth()).fold(f32::INFINITY, f32::min);
            
            prop_assert!(
                (last_depth - min_depth).abs() < 0.001,
                "Last face should be the nearest. Last: {}, Min: {}",
                last_depth,
                min_depth
            );
        }
    }
}
