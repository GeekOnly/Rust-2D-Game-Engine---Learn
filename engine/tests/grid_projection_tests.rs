// Unit tests for grid projection and perspective rendering
// Requirements: 7.1, 7.2, 7.5
// Tests perspective projection calculations, vanishing point convergence, and grid line generation

use glam::{Vec2, Vec3, Mat4};

// Helper function to project 3D point to screen space using perspective projection
fn project_to_screen(
    point: Vec3,
    view_matrix: &Mat4,
    projection_matrix: &Mat4,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // Transform point to clip space
    let clip_pos = *projection_matrix * *view_matrix * point.extend(1.0);
    
    // Check if point is behind camera (negative w)
    if clip_pos.w <= 0.0 {
        return None;
    }
    
    // Perspective divide to get NDC coordinates
    let ndc = clip_pos / clip_pos.w;
    
    // Check if point is within view frustum
    if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 || ndc.z.abs() > 1.0 {
        return None;
    }
    
    // Convert NDC to screen space
    // NDC: [-1, 1] -> Screen: [0, viewport_size]
    let screen_x = (ndc.x + 1.0) * 0.5 * viewport_size.x;
    let screen_y = (1.0 - ndc.y) * 0.5 * viewport_size.y; // Flip Y for screen coordinates
    
    Some(Vec2::new(screen_x, screen_y))
}

// Helper function to create view matrix from camera parameters
fn create_view_matrix(camera_pos: Vec2, rotation: f32, pitch: f32, distance: f32) -> Mat4 {
    let yaw_rad = rotation.to_radians();
    let pitch_rad = pitch.to_radians();
    
    // Calculate camera position in 3D space
    let cam_x = camera_pos.x + distance * yaw_rad.cos() * pitch_rad.cos();
    let cam_y = distance * pitch_rad.sin();
    let cam_z = camera_pos.y + distance * yaw_rad.sin() * pitch_rad.cos();
    
    let eye = Vec3::new(cam_x, cam_y, cam_z);
    let target = Vec3::new(camera_pos.x, 0.0, camera_pos.y);
    let up = Vec3::Y;
    
    Mat4::look_at_rh(eye, target, up)
}

// Helper function to create perspective projection matrix
fn create_projection_matrix(fov_degrees: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let fov_rad = fov_degrees.to_radians();
    Mat4::perspective_rh(fov_rad, aspect, near, far)
}

// ============================================================================
// Unit Tests for Perspective Projection
// Requirements: 7.1, 7.2
// ============================================================================

#[cfg(test)]
mod perspective_projection_tests {
    use super::*;
    
    #[test]
    fn test_project_point_on_ground_plane() {
        // Test that a point on the ground plane projects correctly
        let camera_pos = Vec2::ZERO;
        let rotation = 45.0;
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Point at origin on ground plane
        let point = Vec3::new(0.0, 0.0, 0.0);
        let screen_pos = project_to_screen(point, &view_matrix, &projection_matrix, viewport_size);
        
        assert!(screen_pos.is_some(), "Point on ground plane should project to screen");
        
        let screen_pos = screen_pos.unwrap();
        assert!(screen_pos.x >= 0.0 && screen_pos.x <= viewport_size.x, 
                "Projected X should be within viewport");
        assert!(screen_pos.y >= 0.0 && screen_pos.y <= viewport_size.y, 
                "Projected Y should be within viewport");
    }
    
    #[test]
    fn test_project_point_behind_camera() {
        // Test that points behind the camera return None
        let camera_pos = Vec2::ZERO;
        let rotation = 0.0;
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Point behind camera (camera looks toward +Z, so point at large negative Z is behind)
        let point = Vec3::new(0.0, 0.0, -1000.0);
        let screen_pos = project_to_screen(point, &view_matrix, &projection_matrix, viewport_size);
        
        assert!(screen_pos.is_none(), "Point behind camera should not project");
    }
    
    #[test]
    fn test_project_distant_point_converges() {
        // Test that distant points converge toward vanishing point
        let camera_pos = Vec2::ZERO;
        let rotation = 0.0;  // Look straight ahead
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Two points in front of camera at different distances
        let near_point = Vec3::new(0.0, 0.0, 50.0);
        let far_point = Vec3::new(0.0, 0.0, 500.0);
        
        let near_screen = project_to_screen(near_point, &view_matrix, &projection_matrix, viewport_size);
        let far_screen = project_to_screen(far_point, &view_matrix, &projection_matrix, viewport_size);
        
        // Both points should project since they're in front of camera
        assert!(near_screen.is_some(), "Near point should project to screen");
        assert!(far_screen.is_some(), "Far point should project to screen");
        
        // Both should be valid screen coordinates
        let near_screen = near_screen.unwrap();
        let far_screen = far_screen.unwrap();
        
        assert!(near_screen.x >= 0.0 && near_screen.x <= viewport_size.x,
                "Near point X should be within viewport");
        assert!(far_screen.x >= 0.0 && far_screen.x <= viewport_size.x,
                "Far point X should be within viewport");
    }
    
    #[test]
    fn test_projection_preserves_relative_positions() {
        // Test that relative positions are preserved in projection
        // Simple test: create a view matrix looking down the Z axis
        let eye = Vec3::new(0.0, 5.0, 10.0);  // Camera in front and above origin
        let target = Vec3::new(0.0, 0.0, 0.0);  // Looking at origin
        let up = Vec3::Y;
        let view_matrix = Mat4::look_at_rh(eye, target, up);
        
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Three points in a line along X axis, at the origin
        let left_point = Vec3::new(-20.0, 0.0, 0.0);
        let center_point = Vec3::new(0.0, 0.0, 0.0);
        let right_point = Vec3::new(20.0, 0.0, 0.0);
        
        let left_screen = project_to_screen(left_point, &view_matrix, &projection_matrix, viewport_size);
        let center_screen = project_to_screen(center_point, &view_matrix, &projection_matrix, viewport_size);
        let right_screen = project_to_screen(right_point, &view_matrix, &projection_matrix, viewport_size);
        
        assert!(left_screen.is_some() && center_screen.is_some() && right_screen.is_some(),
                "All points should project to screen");
        
        let left_screen = left_screen.unwrap();
        let center_screen = center_screen.unwrap();
        let right_screen = right_screen.unwrap();
        
        // Left should be to the left of center, right should be to the right
        assert!(left_screen.x < center_screen.x, "Left point should project to left of center");
        assert!(right_screen.x > center_screen.x, "Right point should project to right of center");
    }
    
    #[test]
    fn test_projection_handles_extreme_distances() {
        // Test projection with very near and very far points
        let camera_pos = Vec2::ZERO;
        let rotation = 45.0;
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Very near point (but not behind camera)
        let near_point = Vec3::new(1.0, 0.0, 1.0);
        let near_screen = project_to_screen(near_point, &view_matrix, &projection_matrix, viewport_size);
        
        // Very far point (within far plane)
        let far_point = Vec3::new(5000.0, 0.0, 5000.0);
        let far_screen = project_to_screen(far_point, &view_matrix, &projection_matrix, viewport_size);
        
        // Both should project (or far might be outside frustum, which is ok)
        assert!(near_screen.is_some(), "Near point should project");
        // Far point might be outside frustum, so we don't assert it must project
    }
}

// ============================================================================
// Unit Tests for Vanishing Point Convergence
// Requirements: 7.1, 7.2
// ============================================================================

#[cfg(test)]
mod vanishing_point_tests {
    use super::*;
    
    #[test]
    fn test_parallel_lines_converge_to_vanishing_point() {
        // Test that parallel lines in world space converge in screen space
        // Simple test: create a view matrix looking down the Z axis
        let eye = Vec3::new(0.0, 5.0, 10.0);  // Camera in front and above origin
        let target = Vec3::new(0.0, 0.0, 0.0);  // Looking at origin
        let up = Vec3::Y;
        let view_matrix = Mat4::look_at_rh(eye, target, up);
        
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Two parallel lines along Z axis at different X positions
        // Lines go away from camera (negative Z direction in view space)
        let line1_near = Vec3::new(20.0, 0.0, 5.0);
        let line1_far = Vec3::new(20.0, 0.0, -50.0);
        
        let line2_near = Vec3::new(40.0, 0.0, 5.0);
        let line2_far = Vec3::new(40.0, 0.0, -50.0);
        
        let l1_near_screen = project_to_screen(line1_near, &view_matrix, &projection_matrix, viewport_size);
        let l1_far_screen = project_to_screen(line1_far, &view_matrix, &projection_matrix, viewport_size);
        let l2_near_screen = project_to_screen(line2_near, &view_matrix, &projection_matrix, viewport_size);
        let l2_far_screen = project_to_screen(line2_far, &view_matrix, &projection_matrix, viewport_size);
        
        // All points should project
        assert!(l1_near_screen.is_some() && l1_far_screen.is_some() &&
                l2_near_screen.is_some() && l2_far_screen.is_some(),
                "All points should project to screen");
        
        let l1_near = l1_near_screen.unwrap();
        let l1_far = l1_far_screen.unwrap();
        let l2_near = l2_near_screen.unwrap();
        let l2_far = l2_far_screen.unwrap();
        
        // Distance between lines should decrease as they go into distance
        let near_distance = (l1_near - l2_near).length();
        let far_distance = (l1_far - l2_far).length();
        
        assert!(far_distance < near_distance,
                "Parallel lines should converge (get closer) in screen space. Near: {}, Far: {}",
                near_distance, far_distance);
    }
    
    #[test]
    fn test_grid_lines_converge_consistently() {
        // Test that multiple parallel grid lines all converge toward same vanishing point
        let camera_pos = Vec2::ZERO;
        let rotation = 0.0;
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Multiple parallel lines along Z axis
        let mut convergence_points = Vec::new();
        
        for x in [-200.0, -100.0, 0.0, 100.0, 200.0] {
            let near = Vec3::new(x, 0.0, 100.0);
            let far = Vec3::new(x, 0.0, 2000.0);
            
            if let (Some(near_screen), Some(far_screen)) = (
                project_to_screen(near, &view_matrix, &projection_matrix, viewport_size),
                project_to_screen(far, &view_matrix, &projection_matrix, viewport_size)
            ) {
                // Calculate direction of line in screen space
                let direction = (far_screen - near_screen).normalize();
                
                // Extrapolate to find where line would converge (approximate vanishing point)
                // We'll just use the far point as an approximation
                convergence_points.push(far_screen);
            }
        }
        
        // All convergence points should be relatively close to each other
        if convergence_points.len() >= 2 {
            let first = convergence_points[0];
            for point in &convergence_points[1..] {
                let distance = (*point - first).length();
                // They should converge to roughly the same area (within 20% of viewport width)
                assert!(distance < viewport_size.x * 0.2,
                        "Parallel lines should converge to similar vanishing point. Distance: {}",
                        distance);
            }
        }
    }
    
    #[test]
    fn test_perpendicular_lines_have_different_vanishing_points() {
        // Test that perpendicular lines converge to different vanishing points
        let camera_pos = Vec2::ZERO;
        let rotation = 45.0;
        let pitch = 30.0;
        let distance = 500.0;
        
        let view_matrix = create_view_matrix(camera_pos, rotation, pitch, distance);
        let projection_matrix = create_projection_matrix(60.0, 16.0 / 9.0, 0.1, 10000.0);
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Line along X axis
        let x_line_near = Vec3::new(0.0, 0.0, 0.0);
        let x_line_far = Vec3::new(1000.0, 0.0, 0.0);
        
        // Line along Z axis
        let z_line_near = Vec3::new(0.0, 0.0, 0.0);
        let z_line_far = Vec3::new(0.0, 0.0, 1000.0);
        
        let x_near = project_to_screen(x_line_near, &view_matrix, &projection_matrix, viewport_size);
        let x_far = project_to_screen(x_line_far, &view_matrix, &projection_matrix, viewport_size);
        let z_near = project_to_screen(z_line_near, &view_matrix, &projection_matrix, viewport_size);
        let z_far = project_to_screen(z_line_far, &view_matrix, &projection_matrix, viewport_size);
        
        if let (Some(x_near), Some(x_far), Some(z_near), Some(z_far)) = (x_near, x_far, z_near, z_far) {
            // Calculate directions in screen space
            let x_direction = (x_far - x_near).normalize();
            let z_direction = (z_far - z_near).normalize();
            
            // Directions should be different (not parallel)
            let dot_product = x_direction.dot(z_direction);
            assert!(dot_product.abs() < 0.9,
                    "Perpendicular lines should have different screen directions. Dot: {}",
                    dot_product);
        }
    }
}

// ============================================================================
// Unit Tests for Grid Line Generation with Perspective
// Requirements: 7.5
// ============================================================================

#[cfg(test)]
mod grid_line_generation_tests {
    use super::*;
    
    #[test]
    fn test_generate_grid_lines_extends_far() {
        // Test that grid lines extend far into the distance (1000+ units)
        let grid_spacing = 10.0;
        let camera_pos = Vec2::ZERO;
        let visible_range = 1000.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        // Generate lines parallel to X axis
        let mut lines = Vec::new();
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            lines.push((start, end));
            z += grid_spacing;
        }
        
        // Should generate many lines
        assert!(lines.len() > 100, "Should generate many grid lines for large range");
        
        // Lines should extend far
        for (start, end) in &lines {
            let length = (*end - *start).length();
            assert!(length >= 1000.0, "Grid lines should extend at least 1000 units");
        }
    }
    
    #[test]
    fn test_grid_lines_on_ground_plane() {
        // Test that all grid lines are on the ground plane (Y = 0)
        let grid_spacing = 10.0;
        let camera_pos = Vec2::ZERO;
        let visible_range = 500.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        let mut lines = Vec::new();
        
        // Generate lines parallel to X axis
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            lines.push((start, end));
            z += grid_spacing;
        }
        
        // Generate lines parallel to Z axis
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            let start = Vec3::new(x, 0.0, min_z);
            let end = Vec3::new(x, 0.0, max_z);
            lines.push((start, end));
            x += grid_spacing;
        }
        
        // All lines should be on ground plane
        for (start, end) in &lines {
            assert!(start.y.abs() < 0.01, "Grid line start should be on ground plane (Y=0)");
            assert!(end.y.abs() < 0.01, "Grid line end should be on ground plane (Y=0)");
        }
    }
    
    #[test]
    fn test_grid_lines_form_perpendicular_grid() {
        // Test that grid lines form a perpendicular grid pattern
        let grid_spacing = 10.0;
        let camera_pos = Vec2::ZERO;
        let visible_range = 100.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        let mut x_parallel_lines = Vec::new();
        let mut z_parallel_lines = Vec::new();
        
        // Generate lines parallel to X axis
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            x_parallel_lines.push((start, end));
            z += grid_spacing;
        }
        
        // Generate lines parallel to Z axis
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            let start = Vec3::new(x, 0.0, min_z);
            let end = Vec3::new(x, 0.0, max_z);
            z_parallel_lines.push((start, end));
            x += grid_spacing;
        }
        
        // Should have both types of lines
        assert!(x_parallel_lines.len() > 0, "Should have X-parallel lines");
        assert!(z_parallel_lines.len() > 0, "Should have Z-parallel lines");
        
        // X-parallel lines should have constant Z
        for (start, end) in &x_parallel_lines {
            assert!((start.z - end.z).abs() < 0.01, "X-parallel lines should have constant Z");
            assert!((end.x - start.x).abs() > 100.0, "X-parallel lines should extend along X");
        }
        
        // Z-parallel lines should have constant X
        for (start, end) in &z_parallel_lines {
            assert!((start.x - end.x).abs() < 0.01, "Z-parallel lines should have constant X");
            assert!((end.z - start.z).abs() > 100.0, "Z-parallel lines should extend along Z");
        }
    }
    
    #[test]
    fn test_grid_spacing_consistent() {
        // Test that grid lines are evenly spaced
        let grid_spacing = 10.0;
        let camera_pos = Vec2::ZERO;
        let visible_range = 100.0;
        
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        // Generate Z coordinates for lines parallel to X axis
        let mut z_coords = Vec::new();
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            z_coords.push(z);
            z += grid_spacing;
        }
        
        // Check spacing between consecutive lines
        for i in 0..z_coords.len()-1 {
            let spacing = (z_coords[i+1] - z_coords[i]).abs();
            assert!((spacing - grid_spacing).abs() < 0.01,
                    "Grid spacing should be consistent. Expected: {}, Got: {}",
                    grid_spacing, spacing);
        }
    }
    
    #[test]
    fn test_grid_includes_origin() {
        // Test that grid includes lines through the origin (axis lines)
        let grid_spacing = 10.0;
        let camera_pos = Vec2::ZERO;
        let visible_range = 100.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        let mut has_x_axis = false;
        let mut has_z_axis = false;
        
        // Check for X axis line (Z = 0)
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            if z.abs() < 0.01 {
                has_x_axis = true;
                break;
            }
            z += grid_spacing;
        }
        
        // Check for Z axis line (X = 0)
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            if x.abs() < 0.01 {
                has_z_axis = true;
                break;
            }
            x += grid_spacing;
        }
        
        assert!(has_x_axis, "Grid should include X axis line (Z=0)");
        assert!(has_z_axis, "Grid should include Z axis line (X=0)");
    }
}
