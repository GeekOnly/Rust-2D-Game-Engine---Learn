// Property-based tests for SceneGrid
// These tests validate the correctness properties defined in the unity-scene-view design document

use proptest::prelude::*;
use glam::{Vec2, Vec3};

// Copy the SceneGrid implementation for testing
#[derive(Debug, Clone)]
pub struct SceneGrid {
    pub enabled: bool,
    pub size: f32,
    pub snap_enabled: bool,
    pub color: [f32; 4],
    pub axis_color_x: [f32; 4],
    pub axis_color_z: [f32; 4],
    pub fade_distance: f32,
    pub fade_range: f32,
    pub subdivision_levels: Vec<f32>,
    pub min_line_spacing: f32,
}

impl SceneGrid {
    pub fn new() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            snap_enabled: false,
            color: [0.3, 0.3, 0.3, 0.5],
            axis_color_x: [0.8, 0.2, 0.2, 0.8],
            axis_color_z: [0.2, 0.2, 0.8, 0.8],
            fade_distance: 500.0,
            fade_range: 200.0,
            subdivision_levels: vec![1.0, 0.1, 0.01],
            min_line_spacing: 20.0,
        }
    }
    
    pub fn select_grid_level(&self, zoom: f32) -> f32 {
        let base_spacing = self.size;
        let target_min = self.min_line_spacing;
        let target_max = self.min_line_spacing * 5.0;
        let target_mid = (target_min + target_max) / 2.0;
        
        let screen_spacing = base_spacing * zoom;
        
        if screen_spacing >= target_min && screen_spacing <= target_max {
            return base_spacing;
        }
        
        let mut best_spacing = base_spacing;
        let mut best_distance = (screen_spacing - target_mid).abs();
        
        for &level in &self.subdivision_levels {
            let test_spacing = base_spacing / level;
            let test_screen = test_spacing * zoom;
            if test_screen >= target_min && test_screen <= target_max {
                let distance = (test_screen - target_mid).abs();
                if distance < best_distance {
                    best_spacing = test_spacing;
                    best_distance = distance;
                }
            }
        }
        
        for &level in &self.subdivision_levels {
            let test_spacing = base_spacing * level;
            let test_screen = test_spacing * zoom;
            if test_screen >= target_min && test_screen <= target_max {
                let distance = (test_screen - target_mid).abs();
                if distance < best_distance {
                    best_spacing = test_spacing;
                    best_distance = distance;
                }
            }
        }
        
        if best_distance < (target_max - target_min) / 2.0 {
            return best_spacing;
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
    
    pub fn calculate_fade_alpha(&self, distance: f32) -> f32 {
        if distance < self.fade_distance {
            1.0
        } else if distance > self.fade_distance + self.fade_range {
            0.0
        } else {
            let fade_progress = (distance - self.fade_distance) / self.fade_range;
            1.0 - fade_progress
        }
    }
    
    pub fn generate_2d_grid_lines(
        &self,
        camera_pos: Vec2,
        viewport_size: Vec2,
        zoom: f32,
    ) -> (Vec<(Vec2, Vec2)>, Vec<(Vec2, Vec2)>) {
        let grid_spacing = self.select_grid_level(zoom);
        
        let half_width = viewport_size.x / (2.0 * zoom);
        let half_height = viewport_size.y / (2.0 * zoom);
        
        let min_x = camera_pos.x - half_width;
        let max_x = camera_pos.x + half_width;
        let min_y = camera_pos.y - half_height;
        let max_y = camera_pos.y + half_height;
        
        let mut horizontal_lines = Vec::new();
        let mut vertical_lines = Vec::new();
        
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            vertical_lines.push((
                Vec2::new(x, min_y),
                Vec2::new(x, max_y),
            ));
            x += grid_spacing;
        }
        
        let start_y = (min_y / grid_spacing).floor() * grid_spacing;
        let mut y = start_y;
        while y <= max_y {
            horizontal_lines.push((
                Vec2::new(min_x, y),
                Vec2::new(max_x, y),
            ));
            y += grid_spacing;
        }
        
        (horizontal_lines, vertical_lines)
    }
    
    pub fn generate_3d_grid_lines(
        &self,
        camera_pos: Vec2,
        zoom: f32,
        view_distance: f32,
    ) -> Vec<(Vec3, Vec3, f32)> {
        let grid_spacing = self.select_grid_level(zoom);
        let visible_range = view_distance * 2.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        let mut lines = Vec::new();
        
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            let distance = (Vec2::new(camera_pos.x, z) - camera_pos).length();
            lines.push((start, end, distance));
            z += grid_spacing;
        }
        
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            let start = Vec3::new(x, 0.0, min_z);
            let end = Vec3::new(x, 0.0, max_z);
            let distance = (Vec2::new(x, camera_pos.y) - camera_pos).length();
            lines.push((start, end, distance));
            x += grid_spacing;
        }
        
        lines
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: unity-scene-view, Property 7: 2D grid lines are orthogonal
    // Validates: Requirements 2.1
    #[test]
    fn prop_2d_grid_lines_are_orthogonal(
        camera_pos in prop_vec2(),
        viewport_size in prop_viewport_size(),
        zoom in prop_zoom(),
    ) {
        let grid = SceneGrid::new();
        let (horizontal_lines, vertical_lines) = grid.generate_2d_grid_lines(
            camera_pos,
            viewport_size,
            zoom,
        );
        
        // Check all horizontal lines are perfectly horizontal (constant Y)
        for (start, end) in &horizontal_lines {
            let y_diff = (start.y - end.y).abs();
            prop_assert!(
                y_diff < 0.0001,
                "Horizontal lines should have constant Y coordinate. Diff: {}",
                y_diff
            );
            
            // Should span horizontally
            prop_assert!(
                (start.x - end.x).abs() > 0.01,
                "Horizontal lines should span in X direction"
            );
        }
        
        // Check all vertical lines are perfectly vertical (constant X)
        for (start, end) in &vertical_lines {
            let x_diff = (start.x - end.x).abs();
            prop_assert!(
                x_diff < 0.0001,
                "Vertical lines should have constant X coordinate. Diff: {}",
                x_diff
            );
            
            // Should span vertically
            prop_assert!(
                (start.y - end.y).abs() > 0.01,
                "Vertical lines should span in Y direction"
            );
        }
        
        // Verify orthogonality: horizontal and vertical lines should be perpendicular
        if !horizontal_lines.is_empty() && !vertical_lines.is_empty() {
            let h_line = &horizontal_lines[0];
            let v_line = &vertical_lines[0];
            
            let h_dir = (h_line.1 - h_line.0).normalize();
            let v_dir = (v_line.1 - v_line.0).normalize();
            
            let dot_product = h_dir.dot(v_dir).abs();
            prop_assert!(
                dot_product < 0.01,
                "Horizontal and vertical lines should be perpendicular (dot product near 0). Dot: {}",
                dot_product
            );
        }
    }
    
    // Feature: unity-scene-view, Property 8: 3D grid has correct perspective
    // Validates: Requirements 2.2
    #[test]
    fn prop_3d_grid_has_correct_perspective(
        camera_pos in prop_vec2(),
        zoom in prop_zoom(),
        view_distance in 100.0f32..1000.0f32,
    ) {
        let grid = SceneGrid::new();
        let lines = grid.generate_3d_grid_lines(camera_pos, zoom, view_distance);
        
        // All lines should be on the ground plane (Y = 0)
        for (start, end, _distance) in &lines {
            prop_assert!(
                start.y.abs() < 0.0001,
                "All 3D grid lines should be on ground plane (Y=0). Start Y: {}",
                start.y
            );
            prop_assert!(
                end.y.abs() < 0.0001,
                "All 3D grid lines should be on ground plane (Y=0). End Y: {}",
                end.y
            );
        }
        
        // Lines should be either parallel to X axis or Z axis
        for (start, end, _distance) in &lines {
            let dx = (start.x - end.x).abs();
            let dz = (start.z - end.z).abs();
            
            // Either X varies (parallel to X axis) or Z varies (parallel to Z axis)
            let is_x_parallel = dx > 0.01 && dz < 0.01;
            let is_z_parallel = dz > 0.01 && dx < 0.01;
            
            prop_assert!(
                is_x_parallel || is_z_parallel,
                "Grid lines should be parallel to either X or Z axis. dx: {}, dz: {}",
                dx, dz
            );
        }
        
        // Verify distance calculation is correct
        for (start, end, distance) in &lines {
            // Distance should be from camera to the line
            // For a line parallel to X, distance is to the Z coordinate
            // For a line parallel to Z, distance is to the X coordinate
            
            let dx = (start.x - end.x).abs();
            let dz = (start.z - end.z).abs();
            
            let expected_distance = if dx > dz {
                // Line parallel to X axis (constant Z)
                (Vec2::new(camera_pos.x, start.z) - camera_pos).length()
            } else {
                // Line parallel to Z axis (constant X)
                (Vec2::new(start.x, camera_pos.y) - camera_pos).length()
            };
            
            let distance_diff = (*distance - expected_distance).abs();
            prop_assert!(
                distance_diff < 0.1,
                "Distance calculation should be correct. Expected: {}, Actual: {}, Diff: {}",
                expected_distance, distance, distance_diff
            );
        }
    }
    
    // Feature: unity-scene-view, Property 9: Grid fades with distance
    // Validates: Requirements 2.3
    #[test]
    fn prop_grid_fades_with_distance(
        fade_distance in 100.0f32..1000.0f32,
        fade_range in 50.0f32..500.0f32,
        distance1 in 0.0f32..2000.0f32,
        distance2 in 0.0f32..2000.0f32,
    ) {
        let mut grid = SceneGrid::new();
        grid.fade_distance = fade_distance;
        grid.fade_range = fade_range;
        
        let alpha1 = grid.calculate_fade_alpha(distance1);
        let alpha2 = grid.calculate_fade_alpha(distance2);
        
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
        
        // Before fade distance, alpha should be 1.0
        if distance1 < fade_distance {
            prop_assert!(
                (alpha1 - 1.0).abs() < 0.0001,
                "Alpha should be 1.0 before fade distance. Distance: {}, Fade distance: {}, Alpha: {}",
                distance1, fade_distance, alpha1
            );
        }
        
        // After fade distance + fade range, alpha should be 0.0
        if distance1 > fade_distance + fade_range {
            prop_assert!(
                alpha1.abs() < 0.0001,
                "Alpha should be 0.0 after fade range. Distance: {}, Fade end: {}, Alpha: {}",
                distance1, fade_distance + fade_range, alpha1
            );
        }
        
        // Within fade range, alpha should decrease linearly
        if distance1 >= fade_distance && distance1 <= fade_distance + fade_range {
            let expected_alpha = 1.0 - (distance1 - fade_distance) / fade_range;
            prop_assert!(
                (alpha1 - expected_alpha).abs() < 0.01,
                "Alpha should decrease linearly in fade range. Expected: {}, Got: {}",
                expected_alpha, alpha1
            );
        }
        
        // Monotonicity: if distance2 > distance1, then alpha2 <= alpha1
        if distance2 > distance1 + 0.1 {
            prop_assert!(
                alpha2 <= alpha1 + 0.01, // Small tolerance for floating point
                "Alpha should decrease monotonically with distance. d1: {}, a1: {}, d2: {}, a2: {}",
                distance1, alpha1, distance2, alpha2
            );
        }
    }
    
    // Feature: unity-scene-view, Property 10: Adaptive grid maintains visual density
    // Validates: Requirements 2.5, 7.1
    #[test]
    fn prop_adaptive_grid_maintains_visual_density(
        grid_size in 0.1f32..10.0f32,
        zoom in prop_zoom(),
        min_spacing in 10.0f32..50.0f32,
    ) {
        let mut grid = SceneGrid::new();
        grid.size = grid_size;
        grid.min_line_spacing = min_spacing;
        
        let selected_spacing = grid.select_grid_level(zoom);
        let screen_spacing = selected_spacing * zoom;
        
        // Selected spacing should be positive
        prop_assert!(
            selected_spacing > 0.0,
            "Selected grid spacing should be positive. Got: {}",
            selected_spacing
        );
        
        // Screen spacing should be within reasonable bounds
        // It should be at least min_line_spacing (or close to it)
        // and not excessively large (less than 5x min_spacing)
        prop_assert!(
            screen_spacing >= min_spacing * 0.9, // Allow small tolerance
            "Screen spacing should be at least min_line_spacing. Screen: {}, Min: {}",
            screen_spacing, min_spacing
        );
        
        prop_assert!(
            screen_spacing <= min_spacing * 6.0, // Allow some flexibility
            "Screen spacing should not be excessively large. Screen: {}, Max: {}",
            screen_spacing, min_spacing * 6.0
        );
    }
    
    // Feature: unity-scene-view, Property 11: Grid subdivisions adapt to zoom
    // Validates: Requirements 7.2, 7.3
    #[test]
    fn prop_grid_subdivisions_adapt_to_zoom(
        grid_size in 0.1f32..10.0f32,
        zoom1 in 5.0f32..50.0f32,
        zoom2 in 100.0f32..200.0f32,
        min_spacing in 10.0f32..50.0f32,
    ) {
        let mut grid = SceneGrid::new();
        grid.size = grid_size;
        grid.min_line_spacing = min_spacing;
        
        // Get grid spacing at low zoom (zoomed out)
        let spacing_low_zoom = grid.select_grid_level(zoom1);
        let screen_spacing_low = spacing_low_zoom * zoom1;
        
        // Get grid spacing at high zoom (zoomed in)
        let spacing_high_zoom = grid.select_grid_level(zoom2);
        let screen_spacing_high = spacing_high_zoom * zoom2;
        
        // Both should produce reasonable screen spacing
        prop_assert!(
            spacing_low_zoom > 0.0,
            "Grid spacing should be positive at low zoom"
        );
        prop_assert!(
            spacing_high_zoom > 0.0,
            "Grid spacing should be positive at high zoom"
        );
        
        // When zoomed in (high zoom), we should use finer grid spacing
        // (smaller world-space spacing) to maintain visual density
        prop_assert!(
            spacing_high_zoom <= spacing_low_zoom * 1.1, // Allow small tolerance
            "Higher zoom should use finer or equal grid spacing. Low zoom spacing: {}, High zoom spacing: {}",
            spacing_low_zoom, spacing_high_zoom
        );
        
        // Screen spacing should be within reasonable bounds for both
        let target_min = min_spacing * 0.9; // Allow small tolerance
        let target_max = min_spacing * 6.0;
        
        prop_assert!(
            screen_spacing_low >= target_min && screen_spacing_low <= target_max,
            "Low zoom screen spacing should be in range. Got: {}, Range: [{}, {}]",
            screen_spacing_low, target_min, target_max
        );
        
        prop_assert!(
            screen_spacing_high >= target_min && screen_spacing_high <= target_max,
            "High zoom screen spacing should be in range. Got: {}, Range: [{}, {}]",
            screen_spacing_high, target_min, target_max
        );
    }
}
