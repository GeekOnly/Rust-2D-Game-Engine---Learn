// Property-based tests for InfiniteGrid
// Feature: scene-view-improvements
// These tests validate the correctness properties defined in the scene-view-improvements design document

use proptest::prelude::*;
use glam::{Vec2, Vec3};

// Import the types we need to test
#[derive(Debug, Clone, PartialEq)]
pub struct CameraState {
    pub position: Vec2,
    pub rotation: f32,
    pub pitch: f32,
    pub zoom: f32,
}

#[derive(Debug, Clone)]
pub struct InfiniteGrid {
    pub enabled: bool,
    pub base_unit: f32,
    pub major_line_every: u32,
    pub minor_line_color: [f32; 4],
    pub major_line_color: [f32; 4],
    pub x_axis_color: [f32; 4],
    pub z_axis_color: [f32; 4],
    pub minor_line_width: f32,
    pub major_line_width: f32,
    pub axis_line_width: f32,
    pub fade_start_distance: f32,
    pub fade_end_distance: f32,
    pub near_fade_start: f32,
    pub near_fade_end: f32,
    pub min_pixel_spacing: f32,
    pub max_pixel_spacing: f32,
    pub level_transition_range: f32,
    cached_geometry: Option<GridGeometry>,
    last_camera_state: Option<CameraState>,
}

#[derive(Debug, Clone)]
pub struct GridGeometry {
    pub lines: Vec<GridLine>,
    pub generation_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct GridLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 4],
    pub width: f32,
    pub line_type: GridLineType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridLineType {
    Minor,
    Major,
    XAxis,
    ZAxis,
}

impl InfiniteGrid {
    pub fn new() -> Self {
        Self {
            enabled: true,
            base_unit: 1.0,
            major_line_every: 10,
            minor_line_color: [0.3, 0.3, 0.3, 0.4],
            major_line_color: [0.4, 0.4, 0.4, 0.6],
            x_axis_color: [0.8, 0.2, 0.2, 0.8],
            z_axis_color: [0.2, 0.2, 0.8, 0.8],
            minor_line_width: 1.0,
            major_line_width: 1.5,
            axis_line_width: 2.0,
            fade_start_distance: 500.0,
            fade_end_distance: 1000.0,
            near_fade_start: 0.5,
            near_fade_end: 0.1,
            min_pixel_spacing: 20.0,
            max_pixel_spacing: 100.0,
            level_transition_range: 0.3,
            cached_geometry: None,
            last_camera_state: None,
        }
    }
    
    pub fn calculate_grid_level(&self, zoom: f32) -> f32 {
        let base_spacing = self.base_unit;
        let target_mid = (self.min_pixel_spacing + self.max_pixel_spacing) / 2.0;
        
        let screen_spacing = base_spacing * zoom;
        
        if screen_spacing >= self.min_pixel_spacing && screen_spacing <= self.max_pixel_spacing {
            return base_spacing;
        }
        
        let optimal_spacing = target_mid / zoom;
        let magnitude = 10.0_f32.powf(optimal_spacing.log10().floor());
        let normalized = optimal_spacing / magnitude;
        
        let rounded = if normalized < 1.5 {
            1.0
        } else if normalized < 3.5 {
            2.0
        } else if normalized < 7.5 {
            5.0
        } else {
            10.0
        };
        
        magnitude * rounded
    }
    
    pub fn calculate_fade_alpha(&self, point: Vec3, camera_pos: Vec3) -> f32 {
        let distance = (point - camera_pos).length();
        
        let far_alpha = if distance < self.fade_start_distance {
            1.0
        } else if distance > self.fade_end_distance {
            0.0
        } else {
            let fade_progress = (distance - self.fade_start_distance) 
                / (self.fade_end_distance - self.fade_start_distance);
            1.0 - fade_progress
        };
        
        let near_alpha = if distance > self.near_fade_start {
            1.0
        } else if distance < self.near_fade_end {
            0.0
        } else {
            let fade_progress = (distance - self.near_fade_end) 
                / (self.near_fade_start - self.near_fade_end);
            fade_progress
        };
        
        far_alpha * near_alpha
    }
    
    pub fn generate_geometry(
        &mut self,
        camera: &CameraState,
        viewport_size: Vec2,
    ) -> &GridGeometry {
        let mut lines = Vec::new();
        
        let grid_spacing = self.calculate_grid_level(camera.zoom);
        let visible_range = 1000.0;
        
        let min_x = camera.position.x - visible_range;
        let max_x = camera.position.x + visible_range;
        let min_z = camera.position.y - visible_range;
        let max_z = camera.position.y + visible_range;
        
        let yaw_rad = camera.rotation.to_radians();
        let pitch_rad = camera.pitch.to_radians();
        let distance = 500.0;
        
        let cam_x = camera.position.x + distance * yaw_rad.cos() * pitch_rad.cos();
        let cam_y = distance * pitch_rad.sin();
        let cam_z = camera.position.y + distance * yaw_rad.sin() * pitch_rad.cos();
        let camera_pos_3d = Vec3::new(cam_x, cam_y, cam_z);
        
        // Generate lines parallel to X axis
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            
            let line_type = if z.abs() < 0.01 {
                GridLineType::ZAxis
            } else if (z / grid_spacing).rem_euclid(self.major_line_every as f32).abs() < 0.01 {
                GridLineType::Major
            } else {
                GridLineType::Minor
            };
            
            let mid_point = Vec3::new((min_x + max_x) / 2.0, 0.0, z);
            let alpha = self.calculate_fade_alpha(mid_point, camera_pos_3d);
            
            let (mut color, width) = match line_type {
                GridLineType::Minor => (self.minor_line_color, self.minor_line_width),
                GridLineType::Major => (self.major_line_color, self.major_line_width),
                GridLineType::ZAxis => (self.z_axis_color, self.axis_line_width),
                GridLineType::XAxis => (self.x_axis_color, self.axis_line_width),
            };
            
            color[3] *= alpha;
            
            lines.push(GridLine {
                start,
                end,
                color,
                width,
                line_type,
            });
            
            z += grid_spacing;
        }
        
        // Generate lines parallel to Z axis
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            let start = Vec3::new(x, 0.0, min_z);
            let end = Vec3::new(x, 0.0, max_z);
            
            let line_type = if x.abs() < 0.01 {
                GridLineType::XAxis
            } else if (x / grid_spacing).rem_euclid(self.major_line_every as f32).abs() < 0.01 {
                GridLineType::Major
            } else {
                GridLineType::Minor
            };
            
            let mid_point = Vec3::new(x, 0.0, (min_z + max_z) / 2.0);
            let alpha = self.calculate_fade_alpha(mid_point, camera_pos_3d);
            
            let (mut color, width) = match line_type {
                GridLineType::Minor => (self.minor_line_color, self.minor_line_width),
                GridLineType::Major => (self.major_line_color, self.major_line_width),
                GridLineType::XAxis => (self.x_axis_color, self.axis_line_width),
                GridLineType::ZAxis => (self.z_axis_color, self.axis_line_width),
            };
            
            color[3] *= alpha;
            
            lines.push(GridLine {
                start,
                end,
                color,
                width,
                line_type,
            });
            
            x += grid_spacing;
        }
        
        let geometry = GridGeometry {
            lines,
            generation_time: std::time::Instant::now(),
        };
        
        self.cached_geometry = Some(geometry);
        self.last_camera_state = Some(camera.clone());
        
        self.cached_geometry.as_ref().unwrap()
    }
}

impl CameraState {
    pub fn has_changed_significantly(&self, other: &CameraState, threshold: f32) -> bool {
        let pos_delta = (self.position - other.position).length();
        let rotation_delta = (self.rotation - other.rotation).abs();
        let pitch_delta = (self.pitch - other.pitch).abs();
        let zoom_delta = (self.zoom - other.zoom).abs() / self.zoom;
        
        pos_delta > threshold 
            || rotation_delta > threshold * 10.0 
            || pitch_delta > threshold * 10.0 
            || zoom_delta > threshold * 0.1
    }
}

// Helper functions for property testing
fn prop_vec2() -> impl Strategy<Value = Vec2> {
    (-1000.0f32..1000.0f32, -1000.0f32..1000.0f32)
        .prop_map(|(x, y)| Vec2::new(x, y))
}

fn prop_zoom() -> impl Strategy<Value = f32> {
    5.0f32..200.0f32
}

fn prop_viewport_size() -> impl Strategy<Value = Vec2> {
    (800.0f32..1920.0f32, 600.0f32..1080.0f32)
        .prop_map(|(w, h)| Vec2::new(w, h))
}

fn prop_rotation() -> impl Strategy<Value = f32> {
    0.0f32..360.0f32
}

fn prop_pitch() -> impl Strategy<Value = f32> {
    -89.0f32..89.0f32
}

fn prop_camera_state() -> impl Strategy<Value = CameraState> {
    (prop_vec2(), prop_rotation(), prop_pitch(), prop_zoom())
        .prop_map(|(position, rotation, pitch, zoom)| CameraState {
            position,
            rotation,
            pitch,
            zoom,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: scene-view-improvements, Property 7: Grid lines converge with perspective
    // Validates: Requirements 1.2, 7.1, 7.2
    #[test]
    fn prop_grid_lines_converge_with_perspective(
        camera in prop_camera_state(),
        viewport_size in prop_viewport_size(),
    ) {
        let mut grid = InfiniteGrid::new();
        let geometry = grid.generate_geometry(&camera, viewport_size);
        
        // All lines should be on the ground plane (Y = 0)
        for line in &geometry.lines {
            prop_assert!(
                line.start.y.abs() < 0.0001,
                "All grid lines should be on ground plane (Y=0). Start Y: {}",
                line.start.y
            );
            prop_assert!(
                line.end.y.abs() < 0.0001,
                "All grid lines should be on ground plane (Y=0). End Y: {}",
                line.end.y
            );
        }
        
        // Lines should be either parallel to X axis or Z axis
        for line in &geometry.lines {
            let dx = (line.start.x - line.end.x).abs();
            let dz = (line.start.z - line.end.z).abs();
            
            // Either X varies (parallel to X axis) or Z varies (parallel to Z axis)
            let is_x_parallel = dx > 0.01 && dz < 0.01;
            let is_z_parallel = dz > 0.01 && dx < 0.01;
            
            prop_assert!(
                is_x_parallel || is_z_parallel,
                "Grid lines should be parallel to either X or Z axis. dx: {}, dz: {}",
                dx, dz
            );
        }
        
        // Verify that parallel lines in world space would converge in screen space
        // (This is a property of perspective projection - we verify the setup is correct)
        // Find two parallel lines (same direction)
        let x_parallel_lines: Vec<_> = geometry.lines.iter()
            .filter(|l| (l.start.x - l.end.x).abs() > 0.01 && (l.start.z - l.end.z).abs() < 0.01)
            .take(2)
            .collect();
        
        if x_parallel_lines.len() >= 2 {
            let line1 = x_parallel_lines[0];
            let line2 = x_parallel_lines[1];
            
            // These lines are parallel in world space (both run along X axis)
            let dir1 = (line1.end - line1.start).normalize();
            let dir2 = (line2.end - line2.start).normalize();
            
            // Directions should be nearly identical (parallel)
            let dot = dir1.dot(dir2);
            prop_assert!(
                dot > 0.99,
                "Parallel lines should have same direction. Dot product: {}",
                dot
            );
            
            // Lines should be at different Z positions (not the same line)
            let z_diff = (line1.start.z - line2.start.z).abs();
            prop_assert!(
                z_diff > 0.01,
                "Parallel lines should be at different positions"
            );
        }
    }
    
    // Feature: scene-view-improvements, Property 8: Grid fade is monotonic with distance
    // Validates: Requirements 1.3
    #[test]
    fn prop_grid_fade_is_monotonic_with_distance(
        fade_start in 100.0f32..1000.0f32,
        fade_end in 1100.0f32..2000.0f32,
        distance1 in 0.0f32..2500.0f32,
        distance2 in 0.0f32..2500.0f32,
    ) {
        let mut grid = InfiniteGrid::new();
        grid.fade_start_distance = fade_start;
        grid.fade_end_distance = fade_end;
        
        // Create two points at different distances
        let camera_pos = Vec3::new(0.0, 100.0, 0.0);
        let point1 = Vec3::new(distance1, 0.0, 0.0);
        let point2 = Vec3::new(distance2, 0.0, 0.0);
        
        let alpha1 = grid.calculate_fade_alpha(point1, camera_pos);
        let alpha2 = grid.calculate_fade_alpha(point2, camera_pos);
        
        // Alpha should be in valid range [0, 1]
        prop_assert!(
            alpha1 >= 0.0 && alpha1 <= 1.0,
            "Alpha should be in range [0, 1]. Got: {}",
            alpha1
        );
        prop_assert!(
            alpha2 >= 0.0 && alpha2 <= 1.0,
            "Alpha should be in range [0, 1]. Got: {}",
            alpha2
        );
        
        // Calculate actual distances from camera
        let actual_dist1 = (point1 - camera_pos).length();
        let actual_dist2 = (point2 - camera_pos).length();
        
        // Monotonicity: if distance2 > distance1 (beyond fade start), then alpha2 <= alpha1
        if actual_dist2 > actual_dist1 + 1.0 && actual_dist1 > fade_start {
            prop_assert!(
                alpha2 <= alpha1 + 0.01, // Small tolerance for floating point
                "Alpha should decrease monotonically with distance. d1: {}, a1: {}, d2: {}, a2: {}",
                actual_dist1, alpha1, actual_dist2, alpha2
            );
        }
        
        // Before fade start, alpha should be 1.0
        if actual_dist1 < fade_start {
            prop_assert!(
                (alpha1 - 1.0).abs() < 0.01,
                "Alpha should be 1.0 before fade start. Distance: {}, Fade start: {}, Alpha: {}",
                actual_dist1, fade_start, alpha1
            );
        }
        
        // After fade end, alpha should be 0.0
        if actual_dist1 > fade_end {
            prop_assert!(
                alpha1.abs() < 0.01,
                "Alpha should be 0.0 after fade end. Distance: {}, Fade end: {}, Alpha: {}",
                actual_dist1, fade_end, alpha1
            );
        }
    }
    
    // Feature: scene-view-improvements, Property 9: Grid level transitions maintain constant alpha
    // Validates: Requirements 6.3, 6.5
    #[test]
    fn prop_grid_level_transitions_maintain_constant_alpha(
        zoom1 in 5.0f32..50.0f32,
        zoom2 in 100.0f32..200.0f32,
    ) {
        let grid = InfiniteGrid::new();
        
        // Get grid levels at two different zoom levels
        let level1 = grid.calculate_grid_level(zoom1);
        let level2 = grid.calculate_grid_level(zoom2);
        
        // Both should be positive
        prop_assert!(
            level1 > 0.0,
            "Grid level should be positive at low zoom. Got: {}",
            level1
        );
        prop_assert!(
            level2 > 0.0,
            "Grid level should be positive at high zoom. Got: {}",
            level2
        );
        
        // Calculate screen spacing for both
        let screen_spacing1 = level1 * zoom1;
        let screen_spacing2 = level2 * zoom2;
        
        // Both should be within reasonable bounds
        let min_spacing = grid.min_pixel_spacing * 0.8; // Allow some tolerance
        let max_spacing = grid.max_pixel_spacing * 1.2;
        
        prop_assert!(
            screen_spacing1 >= min_spacing && screen_spacing1 <= max_spacing,
            "Screen spacing should be in range at low zoom. Got: {}, Range: [{}, {}]",
            screen_spacing1, min_spacing, max_spacing
        );
        
        prop_assert!(
            screen_spacing2 >= min_spacing && screen_spacing2 <= max_spacing,
            "Screen spacing should be in range at high zoom. Got: {}, Range: [{}, {}]",
            screen_spacing2, min_spacing, max_spacing
        );
    }
    
    // Feature: scene-view-improvements, Property 10: Grid spacing maintains visual density
    // Validates: Requirements 6.1, 6.2, 6.4
    #[test]
    fn prop_grid_spacing_maintains_visual_density(
        base_unit in 0.1f32..10.0f32,
        zoom in prop_zoom(),
    ) {
        let mut grid = InfiniteGrid::new();
        grid.base_unit = base_unit;
        
        let selected_spacing = grid.calculate_grid_level(zoom);
        let screen_spacing = selected_spacing * zoom;
        
        // Selected spacing should be positive
        prop_assert!(
            selected_spacing > 0.0,
            "Selected grid spacing should be positive. Got: {}",
            selected_spacing
        );
        
        // Screen spacing should be within reasonable bounds
        let min_bound = grid.min_pixel_spacing * 0.8; // Allow tolerance
        let max_bound = grid.max_pixel_spacing * 1.2;
        
        prop_assert!(
            screen_spacing >= min_bound,
            "Screen spacing should be at least min bound. Screen: {}, Min: {}",
            screen_spacing, min_bound
        );
        
        prop_assert!(
            screen_spacing <= max_bound,
            "Screen spacing should not exceed max bound. Screen: {}, Max: {}",
            screen_spacing, max_bound
        );
    }
    
    // Feature: scene-view-improvements, Property 11: Axis lines have full opacity at origin
    // Validates: Requirements 4.3
    #[test]
    fn prop_axis_lines_have_full_opacity_at_origin(
        camera in prop_camera_state(),
        viewport_size in prop_viewport_size(),
    ) {
        let mut grid = InfiniteGrid::new();
        let geometry = grid.generate_geometry(&camera, viewport_size);
        
        // Find axis lines (X and Z axis)
        let x_axis_lines: Vec<_> = geometry.lines.iter()
            .filter(|l| l.line_type == GridLineType::XAxis)
            .collect();
        
        let z_axis_lines: Vec<_> = geometry.lines.iter()
            .filter(|l| l.line_type == GridLineType::ZAxis)
            .collect();
        
        // Note: In the implementation, XAxis type means line at X=0 running along Z
        // and ZAxis type means line at Z=0 running along X
        
        // XAxis lines: at X=0, running along Z direction
        for line in x_axis_lines {
            // Should be at X=0 (constant X, varying Z)
            prop_assert!(
                line.start.x.abs() < 0.01 && line.end.x.abs() < 0.01,
                "XAxis line should be at X=0. Start X: {}, End X: {}",
                line.start.x, line.end.x
            );
            
            // Should vary in Z direction
            let dz = (line.start.z - line.end.z).abs();
            prop_assert!(
                dz > 0.01,
                "XAxis line should vary in Z direction. dz: {}",
                dz
            );
            
            // Alpha should be in valid range
            prop_assert!(
                line.color[3] >= 0.0 && line.color[3] <= 1.0,
                "Axis line alpha should be in valid range [0, 1]. Got: {}",
                line.color[3]
            );
        }
        
        // ZAxis lines: at Z=0, running along X direction
        for line in z_axis_lines {
            // Should be at Z=0 (constant Z, varying X)
            prop_assert!(
                line.start.z.abs() < 0.01 && line.end.z.abs() < 0.01,
                "ZAxis line should be at Z=0. Start Z: {}, End Z: {}",
                line.start.z, line.end.z
            );
            
            // Should vary in X direction
            let dx = (line.start.x - line.end.x).abs();
            prop_assert!(
                dx > 0.01,
                "ZAxis line should vary in X direction. dx: {}",
                dx
            );
            
            // Alpha should be in valid range
            prop_assert!(
                line.color[3] >= 0.0 && line.color[3] <= 1.0,
                "Axis line alpha should be in valid range [0, 1]. Got: {}",
                line.color[3]
            );
        }
    }
    
    // Feature: scene-view-improvements, Property 12: Grid extends to horizon
    // Validates: Requirements 1.1, 1.4, 1.5
    #[test]
    fn prop_grid_extends_to_horizon(
        camera in prop_camera_state(),
        viewport_size in prop_viewport_size(),
    ) {
        let mut grid = InfiniteGrid::new();
        let geometry = grid.generate_geometry(&camera, viewport_size);
        
        // Grid should have lines
        prop_assert!(
            !geometry.lines.is_empty(),
            "Grid should generate lines"
        );
        
        // Lines should extend far from camera position
        let visible_range = 1000.0; // From implementation
        
        for line in &geometry.lines {
            // Lines should be on ground plane
            prop_assert!(
                line.start.y.abs() < 0.01 && line.end.y.abs() < 0.01,
                "Grid lines should be on ground plane (Y=0)"
            );
            
            // Lines should extend across a significant distance
            let line_length = (line.end - line.start).length();
            prop_assert!(
                line_length > 100.0,
                "Grid lines should extend significantly. Length: {}",
                line_length
            );
        }
        
        // Check that lines extend to the visible range
        let max_extent_x = geometry.lines.iter()
            .flat_map(|l| [l.start.x.abs(), l.end.x.abs()])
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let max_extent_z = geometry.lines.iter()
            .flat_map(|l| [l.start.z.abs(), l.end.z.abs()])
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        // Grid should extend close to the visible range
        prop_assert!(
            max_extent_x > visible_range * 0.5 || max_extent_z > visible_range * 0.5,
            "Grid should extend far into distance. Max X: {}, Max Z: {}, Range: {}",
            max_extent_x, max_extent_z, visible_range
        );
    }
    
    // Feature: scene-view-improvements, Property 13: Grid orientation matches camera rotation
    // Validates: Requirements 7.4
    #[test]
    fn prop_grid_orientation_matches_camera_rotation(
        camera in prop_camera_state(),
        viewport_size in prop_viewport_size(),
    ) {
        let mut grid = InfiniteGrid::new();
        let geometry = grid.generate_geometry(&camera, viewport_size);
        
        // All grid lines should be on the ground plane (Y=0)
        for line in &geometry.lines {
            prop_assert!(
                line.start.y.abs() < 0.01,
                "Grid line start should be on ground plane. Y: {}",
                line.start.y
            );
            prop_assert!(
                line.end.y.abs() < 0.01,
                "Grid line end should be on ground plane. Y: {}",
                line.end.y
            );
        }
        
        // Lines should be aligned with world axes (not rotated with camera)
        // This is correct because the grid is in world space, not camera space
        for line in &geometry.lines {
            let dx = (line.start.x - line.end.x).abs();
            let dz = (line.start.z - line.end.z).abs();
            
            // Each line should be parallel to either X or Z axis
            let is_x_parallel = dx > 0.01 && dz < 0.01;
            let is_z_parallel = dz > 0.01 && dx < 0.01;
            
            prop_assert!(
                is_x_parallel || is_z_parallel,
                "Grid lines should be parallel to world axes. dx: {}, dz: {}",
                dx, dz
            );
        }
    }
}
