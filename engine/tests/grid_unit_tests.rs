// Unit tests for SceneGrid calculations
// These tests validate specific examples and edge cases for grid operations

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

// ============================================================================
// Unit Tests for Grid Line Generation
// Requirements: 2.1, 2.2, 2.3, 2.5
// ============================================================================

#[cfg(test)]
mod grid_line_generation_tests {
    use super::*;
    
    #[test]
    fn test_2d_grid_lines_at_origin() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::ZERO;
        let viewport_size = Vec2::new(800.0, 600.0);
        let zoom = 50.0;
        
        let (horizontal, vertical) = grid.generate_2d_grid_lines(camera_pos, viewport_size, zoom);
        
        // Should generate some lines
        assert!(horizontal.len() > 0, "Should generate horizontal lines");
        assert!(vertical.len() > 0, "Should generate vertical lines");
        
        // Verify horizontal lines are actually horizontal (constant Y)
        for (start, end) in &horizontal {
            assert!((start.y - end.y).abs() < 0.01, "Horizontal lines should have constant Y");
        }
        
        // Verify vertical lines are actually vertical (constant X)
        for (start, end) in &vertical {
            assert!((start.x - end.x).abs() < 0.01, "Vertical lines should have constant X");
        }
    }
    
    #[test]
    fn test_2d_grid_lines_orthogonal() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::new(10.0, 20.0);
        let viewport_size = Vec2::new(800.0, 600.0);
        let zoom = 50.0;
        
        let (horizontal, vertical) = grid.generate_2d_grid_lines(camera_pos, viewport_size, zoom);
        
        // All horizontal lines should be parallel to X axis (Y constant)
        for (start, end) in &horizontal {
            let direction = *end - *start;
            assert!(direction.y.abs() < 0.01, "Horizontal lines should be parallel to X axis");
            assert!(direction.x.abs() > 0.1, "Horizontal lines should extend along X");
        }
        
        // All vertical lines should be parallel to Y axis (X constant)
        for (start, end) in &vertical {
            let direction = *end - *start;
            assert!(direction.x.abs() < 0.01, "Vertical lines should be parallel to Y axis");
            assert!(direction.y.abs() > 0.1, "Vertical lines should extend along Y");
        }
    }
    
    #[test]
    fn test_2d_grid_lines_cover_viewport() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::ZERO;
        let viewport_size = Vec2::new(800.0, 600.0);
        let zoom = 50.0;
        
        let (horizontal, vertical) = grid.generate_2d_grid_lines(camera_pos, viewport_size, zoom);
        
        // Calculate visible world bounds
        let half_width = viewport_size.x / (2.0 * zoom);
        let half_height = viewport_size.y / (2.0 * zoom);
        
        // Check that lines cover the visible area
        // Horizontal lines should span the width
        for (start, end) in &horizontal {
            assert!(start.x <= -half_width + 0.1, "Horizontal lines should start at left edge");
            assert!(end.x >= half_width - 0.1, "Horizontal lines should end at right edge");
        }
        
        // Vertical lines should span the height
        for (start, end) in &vertical {
            assert!(start.y <= -half_height + 0.1, "Vertical lines should start at bottom edge");
            assert!(end.y >= half_height - 0.1, "Vertical lines should end at top edge");
        }
    }
    
    #[test]
    fn test_2d_grid_spacing_consistent() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::ZERO;
        let viewport_size = Vec2::new(800.0, 600.0);
        let zoom = 50.0;
        
        let (horizontal, _vertical) = grid.generate_2d_grid_lines(camera_pos, viewport_size, zoom);
        
        // Check that spacing between horizontal lines is consistent
        if horizontal.len() >= 2 {
            let spacing1 = (horizontal[1].0.y - horizontal[0].0.y).abs();
            for i in 1..horizontal.len()-1 {
                let spacing = (horizontal[i+1].0.y - horizontal[i].0.y).abs();
                assert!((spacing - spacing1).abs() < 0.01, "Grid spacing should be consistent");
            }
        }
    }
    
    #[test]
    fn test_3d_grid_lines_on_ground_plane() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::ZERO;
        let zoom = 50.0;
        let view_distance = 500.0;
        
        let lines = grid.generate_3d_grid_lines(camera_pos, zoom, view_distance);
        
        // Should generate some lines
        assert!(lines.len() > 0, "Should generate 3D grid lines");
        
        // All lines should be on the ground plane (Y = 0)
        for (start, end, _distance) in &lines {
            assert!((start.y).abs() < 0.01, "Grid lines should be on ground plane (Y=0)");
            assert!((end.y).abs() < 0.01, "Grid lines should be on ground plane (Y=0)");
        }
    }
    
    #[test]
    fn test_3d_grid_lines_perpendicular() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::ZERO;
        let zoom = 50.0;
        let view_distance = 500.0;
        
        let lines = grid.generate_3d_grid_lines(camera_pos, zoom, view_distance);
        
        // Separate lines into X-parallel and Z-parallel
        let mut x_parallel = Vec::new();
        let mut z_parallel = Vec::new();
        
        for (start, end, _) in &lines {
            let direction = *end - *start;
            if direction.x.abs() > direction.z.abs() {
                x_parallel.push((start, end));
            } else {
                z_parallel.push((start, end));
            }
        }
        
        // Should have both types of lines
        assert!(x_parallel.len() > 0, "Should have X-parallel lines");
        assert!(z_parallel.len() > 0, "Should have Z-parallel lines");
        
        // X-parallel lines should have constant Z
        for (start, end) in &x_parallel {
            assert!((start.z - end.z).abs() < 0.01, "X-parallel lines should have constant Z");
        }
        
        // Z-parallel lines should have constant X
        for (start, end) in &z_parallel {
            assert!((start.x - end.x).abs() < 0.01, "Z-parallel lines should have constant X");
        }
    }
    
    #[test]
    fn test_3d_grid_includes_distance() {
        let grid = SceneGrid::new();
        let camera_pos = Vec2::new(100.0, 200.0);
        let zoom = 50.0;
        let view_distance = 500.0;
        
        let lines = grid.generate_3d_grid_lines(camera_pos, zoom, view_distance);
        
        // All lines should have a valid distance value
        for (_start, _end, distance) in &lines {
            assert!(*distance >= 0.0, "Distance should be non-negative");
            assert!(distance.is_finite(), "Distance should be finite");
        }
    }
}

// ============================================================================
// Unit Tests for Fade Alpha Calculations
// Requirements: 2.3
// ============================================================================

#[cfg(test)]
mod fade_alpha_tests {
    use super::*;
    
    #[test]
    fn test_fade_alpha_before_fade_distance() {
        let grid = SceneGrid::new();
        
        // Distance less than fade_distance should return full alpha
        let alpha = grid.calculate_fade_alpha(100.0);
        assert!((alpha - 1.0).abs() < 0.01, "Alpha should be 1.0 before fade distance");
    }
    
    #[test]
    fn test_fade_alpha_at_fade_distance() {
        let grid = SceneGrid::new();
        
        // At exactly fade_distance, should still be full alpha
        let alpha = grid.calculate_fade_alpha(grid.fade_distance);
        assert!((alpha - 1.0).abs() < 0.01, "Alpha should be 1.0 at fade distance");
    }
    
    #[test]
    fn test_fade_alpha_after_fade_range() {
        let grid = SceneGrid::new();
        
        // Distance beyond fade_distance + fade_range should return zero alpha
        let alpha = grid.calculate_fade_alpha(grid.fade_distance + grid.fade_range + 100.0);
        assert!(alpha.abs() < 0.01, "Alpha should be 0.0 after fade range");
    }
    
    #[test]
    fn test_fade_alpha_within_fade_range() {
        let grid = SceneGrid::new();
        
        // Halfway through fade range should be approximately 0.5 alpha
        let mid_distance = grid.fade_distance + grid.fade_range / 2.0;
        let alpha = grid.calculate_fade_alpha(mid_distance);
        assert!(alpha > 0.4 && alpha < 0.6, "Alpha should be ~0.5 at midpoint of fade range");
    }
    
    #[test]
    fn test_fade_alpha_decreases_monotonically() {
        let grid = SceneGrid::new();
        
        // Alpha should decrease as distance increases within fade range
        let d1 = grid.fade_distance + 10.0;
        let d2 = grid.fade_distance + 50.0;
        let d3 = grid.fade_distance + 100.0;
        
        let alpha1 = grid.calculate_fade_alpha(d1);
        let alpha2 = grid.calculate_fade_alpha(d2);
        let alpha3 = grid.calculate_fade_alpha(d3);
        
        assert!(alpha1 > alpha2, "Alpha should decrease with distance");
        assert!(alpha2 > alpha3, "Alpha should decrease with distance");
    }
    
    #[test]
    fn test_fade_alpha_linear_interpolation() {
        let grid = SceneGrid::new();
        
        // Test that fade is linear within the fade range
        let start_distance = grid.fade_distance;
        let end_distance = grid.fade_distance + grid.fade_range;
        
        let alpha_start = grid.calculate_fade_alpha(start_distance);
        let alpha_end = grid.calculate_fade_alpha(end_distance);
        
        // Quarter way through
        let quarter_distance = start_distance + grid.fade_range * 0.25;
        let alpha_quarter = grid.calculate_fade_alpha(quarter_distance);
        
        assert!((alpha_start - 1.0).abs() < 0.01, "Should start at 1.0");
        assert!(alpha_end.abs() < 0.01, "Should end at 0.0");
        assert!((alpha_quarter - 0.75).abs() < 0.01, "Should be 0.75 at quarter point");
    }
}

// ============================================================================
// Unit Tests for Adaptive Spacing Selection
// Requirements: 2.5
// ============================================================================

#[cfg(test)]
mod adaptive_spacing_tests {
    use super::*;
    
    #[test]
    fn test_select_grid_level_returns_positive() {
        let grid = SceneGrid::new();
        
        // Grid level should always be positive
        let level1 = grid.select_grid_level(10.0);
        let level2 = grid.select_grid_level(50.0);
        let level3 = grid.select_grid_level(100.0);
        
        assert!(level1 > 0.0, "Grid level should be positive");
        assert!(level2 > 0.0, "Grid level should be positive");
        assert!(level3 > 0.0, "Grid level should be positive");
    }
    
    #[test]
    fn test_select_grid_level_maintains_screen_spacing() {
        let mut grid = SceneGrid::new();
        grid.size = 1.0;
        grid.min_line_spacing = 20.0;
        
        let zoom = 50.0;
        let spacing = grid.select_grid_level(zoom);
        let screen_spacing = spacing * zoom;
        
        // Screen spacing should be within reasonable range
        // (between min_line_spacing and min_line_spacing * 5)
        assert!(
            screen_spacing >= grid.min_line_spacing * 0.9,
            "Screen spacing should be at least min_line_spacing. Got: {}",
            screen_spacing
        );
        assert!(
            screen_spacing <= grid.min_line_spacing * 5.5,
            "Screen spacing should be at most 5x min_line_spacing. Got: {}",
            screen_spacing
        );
    }
    
    #[test]
    fn test_select_grid_level_adapts_to_zoom() {
        let mut grid = SceneGrid::new();
        grid.size = 1.0;
        grid.min_line_spacing = 20.0;
        
        // At low zoom, should use larger spacing
        let spacing_low = grid.select_grid_level(5.0);
        
        // At high zoom, should use smaller spacing
        let spacing_high = grid.select_grid_level(100.0);
        
        assert!(
            spacing_low > spacing_high,
            "Lower zoom should result in larger grid spacing. Low: {}, High: {}",
            spacing_low,
            spacing_high
        );
    }
    
    #[test]
    fn test_select_grid_level_uses_nice_numbers() {
        let mut grid = SceneGrid::new();
        grid.size = 1.0;
        grid.min_line_spacing = 20.0;
        
        let spacing = grid.select_grid_level(50.0);
        
        // Spacing should be a "nice" number (power of 10, or 2x or 5x power of 10)
        // Check if it's close to a nice number
        let log = spacing.log10();
        let magnitude = 10.0_f32.powf(log.floor());
        let normalized = spacing / magnitude;
        
        // Should be close to 1, 2, 5, or 10
        let is_nice = (normalized - 1.0).abs() < 0.1 ||
                      (normalized - 2.0).abs() < 0.1 ||
                      (normalized - 5.0).abs() < 0.1 ||
                      (normalized - 10.0).abs() < 0.1;
        
        assert!(is_nice, "Grid spacing should be a nice number. Got: {}, normalized: {}", spacing, normalized);
    }
    
    #[test]
    fn test_select_grid_level_base_spacing_in_range() {
        let mut grid = SceneGrid::new();
        grid.size = 1.0;
        grid.min_line_spacing = 20.0;
        
        // When base spacing * zoom is in the target range, should return base spacing
        let zoom = 50.0; // 1.0 * 50.0 = 50 pixels, which is in range [20, 100]
        let spacing = grid.select_grid_level(zoom);
        
        assert!((spacing - grid.size).abs() < 0.01, "Should return base spacing when in range");
    }
    
    #[test]
    fn test_select_grid_level_subdivision() {
        let mut grid = SceneGrid::new();
        grid.size = 1.0;
        grid.min_line_spacing = 20.0;
        grid.subdivision_levels = vec![1.0, 0.1, 0.01];
        
        // At very high zoom, should use subdivision
        let zoom = 500.0; // Base spacing would be 500 pixels, too large
        let spacing = grid.select_grid_level(zoom);
        
        // Should be smaller than base spacing
        assert!(spacing < grid.size, "Should use subdivision at high zoom. Got: {}", spacing);
        
        // Should be one of the subdivision levels
        let is_subdivision = (spacing - 0.1).abs() < 0.01 || 
                            (spacing - 0.01).abs() < 0.01 ||
                            (spacing - 0.2).abs() < 0.01 ||
                            (spacing - 0.5).abs() < 0.01;
        
        assert!(is_subdivision, "Should use a subdivision level. Got: {}", spacing);
    }
}
