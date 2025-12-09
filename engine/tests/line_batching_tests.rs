// Unit tests for line batching system
// These tests validate line grouping, spatial culling, and draw call minimization
// Requirements: 10.1

use glam::{Vec2, Vec3};

// Copy the necessary structures for testing
#[derive(Debug, Clone)]
pub struct LineBatch {
    pub color: [f32; 4],
    pub width: f32,
    pub lines: Vec<(Vec3, Vec3)>,
}

#[derive(Debug, Clone)]
pub struct LineBatcher {
    batches: Vec<LineBatch>,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraState {
    pub position: Vec2,
    pub rotation: f32,
    pub pitch: f32,
    pub zoom: f32,
}

impl LineBatcher {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
        }
    }
    
    pub fn add_line(&mut self, start: Vec3, end: Vec3, color: [f32; 4], width: f32) {
        for batch in &mut self.batches {
            if Self::colors_match(batch.color, color) && (batch.width - width).abs() < 0.01 {
                batch.lines.push((start, end));
                return;
            }
        }
        
        self.batches.push(LineBatch {
            color,
            width,
            lines: vec![(start, end)],
        });
    }
    
    fn colors_match(c1: [f32; 4], c2: [f32; 4]) -> bool {
        (c1[0] - c2[0]).abs() < 0.01 &&
        (c1[1] - c2[1]).abs() < 0.01 &&
        (c1[2] - c2[2]).abs() < 0.01 &&
        (c1[3] - c2[3]).abs() < 0.01
    }
    
    pub fn get_batches(&self) -> &[LineBatch] {
        &self.batches
    }
    
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
    
    pub fn line_count(&self) -> usize {
        self.batches.iter().map(|b| b.lines.len()).sum()
    }
    
    pub fn clear(&mut self) {
        self.batches.clear();
    }
    
    pub fn cull_offscreen_lines(&mut self, viewport_bounds: ViewportBounds) {
        for batch in &mut self.batches {
            batch.lines.retain(|(start, end)| {
                Self::line_intersects_viewport(*start, *end, &viewport_bounds)
            });
        }
        
        self.batches.retain(|b| !b.lines.is_empty());
    }
    
    fn line_intersects_viewport(start: Vec3, end: Vec3, bounds: &ViewportBounds) -> bool {
        if bounds.contains_point(start) || bounds.contains_point(end) {
            return true;
        }
        
        let line_min_x = start.x.min(end.x);
        let line_max_x = start.x.max(end.x);
        let line_min_z = start.z.min(end.z);
        let line_max_z = start.z.max(end.z);
        
        let x_overlap = line_max_x >= bounds.min_x && line_min_x <= bounds.max_x;
        let z_overlap = line_max_z >= bounds.min_z && line_min_z <= bounds.max_z;
        
        x_overlap && z_overlap
    }
}

impl ViewportBounds {
    pub fn new(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_z,
            max_z,
        }
    }
    
    pub fn from_camera(camera: &CameraState, viewport_size: Vec2, margin: f32) -> Self {
        let half_width = (viewport_size.x / (2.0 * camera.zoom)) + margin;
        let half_height = (viewport_size.y / (2.0 * camera.zoom)) + margin;
        
        Self {
            min_x: camera.position.x - half_width,
            max_x: camera.position.x + half_width,
            min_z: camera.position.y - half_height,
            max_z: camera.position.y + half_height,
        }
    }
    
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min_x && point.x <= self.max_x &&
        point.z >= self.min_z && point.z <= self.max_z
    }
}

// ============================================================================
// Unit Tests for Line Grouping by Properties
// Requirements: 10.1
// ============================================================================

#[cfg(test)]
mod line_grouping_tests {
    use super::*;
    
    #[test]
    fn test_same_properties_grouped_together() {
        let mut batcher = LineBatcher::new();
        
        let color = [1.0, 0.0, 0.0, 1.0];
        let width = 2.0;
        
        // Add multiple lines with same properties
        batcher.add_line(Vec3::ZERO, Vec3::X, color, width);
        batcher.add_line(Vec3::ZERO, Vec3::Y, color, width);
        batcher.add_line(Vec3::ZERO, Vec3::Z, color, width);
        
        // Should be grouped into a single batch
        assert_eq!(batcher.batch_count(), 1, "Lines with same properties should be in one batch");
        assert_eq!(batcher.line_count(), 3, "Should have 3 lines total");
        
        let batches = batcher.get_batches();
        assert_eq!(batches[0].lines.len(), 3, "Batch should contain all 3 lines");
    }
    
    #[test]
    fn test_different_colors_separate_batches() {
        let mut batcher = LineBatcher::new();
        
        let red = [1.0, 0.0, 0.0, 1.0];
        let blue = [0.0, 0.0, 1.0, 1.0];
        let width = 2.0;
        
        batcher.add_line(Vec3::ZERO, Vec3::X, red, width);
        batcher.add_line(Vec3::ZERO, Vec3::Y, blue, width);
        
        // Should create two separate batches
        assert_eq!(batcher.batch_count(), 2, "Different colors should create separate batches");
        assert_eq!(batcher.line_count(), 2, "Should have 2 lines total");
    }
    
    #[test]
    fn test_different_widths_separate_batches() {
        let mut batcher = LineBatcher::new();
        
        let color = [1.0, 0.0, 0.0, 1.0];
        
        batcher.add_line(Vec3::ZERO, Vec3::X, color, 1.0);
        batcher.add_line(Vec3::ZERO, Vec3::Y, color, 2.0);
        
        // Should create two separate batches
        assert_eq!(batcher.batch_count(), 2, "Different widths should create separate batches");
    }
    
    #[test]
    fn test_multiple_property_combinations() {
        let mut batcher = LineBatcher::new();
        
        let red = [1.0, 0.0, 0.0, 1.0];
        let blue = [0.0, 0.0, 1.0, 1.0];
        let green = [0.0, 1.0, 0.0, 1.0];
        
        // Add lines with various property combinations
        batcher.add_line(Vec3::ZERO, Vec3::X, red, 1.0);
        batcher.add_line(Vec3::X, Vec3::Y, red, 1.0);
        batcher.add_line(Vec3::Y, Vec3::Z, blue, 2.0);
        batcher.add_line(Vec3::Z, Vec3::ZERO, blue, 2.0);
        batcher.add_line(Vec3::ONE, Vec3::ZERO, green, 1.0);
        
        // Should have 3 batches: (red, 1.0), (blue, 2.0), (green, 1.0)
        assert_eq!(batcher.batch_count(), 3, "Should have 3 distinct batches");
        assert_eq!(batcher.line_count(), 5, "Should have 5 lines total");
    }
    
    #[test]
    fn test_batch_preserves_line_order_within_batch() {
        let mut batcher = LineBatcher::new();
        
        let color = [1.0, 0.0, 0.0, 1.0];
        let width = 2.0;
        
        let line1 = (Vec3::ZERO, Vec3::X);
        let line2 = (Vec3::X, Vec3::Y);
        let line3 = (Vec3::Y, Vec3::Z);
        
        batcher.add_line(line1.0, line1.1, color, width);
        batcher.add_line(line2.0, line2.1, color, width);
        batcher.add_line(line3.0, line3.1, color, width);
        
        let batches = batcher.get_batches();
        assert_eq!(batches[0].lines[0], line1, "First line should be preserved");
        assert_eq!(batches[0].lines[1], line2, "Second line should be preserved");
        assert_eq!(batches[0].lines[2], line3, "Third line should be preserved");
    }
    
    #[test]
    fn test_clear_removes_all_batches() {
        let mut batcher = LineBatcher::new();
        
        let color = [1.0, 0.0, 0.0, 1.0];
        batcher.add_line(Vec3::ZERO, Vec3::X, color, 1.0);
        batcher.add_line(Vec3::ZERO, Vec3::Y, color, 2.0);
        
        assert_eq!(batcher.batch_count(), 2, "Should have 2 batches before clear");
        
        batcher.clear();
        
        assert_eq!(batcher.batch_count(), 0, "Should have 0 batches after clear");
        assert_eq!(batcher.line_count(), 0, "Should have 0 lines after clear");
    }
}

// ============================================================================
// Unit Tests for Spatial Culling Logic
// Requirements: 10.1
// ============================================================================

#[cfg(test)]
mod spatial_culling_tests {
    use super::*;
    
    #[test]
    fn test_lines_inside_viewport_not_culled() {
        let mut batcher = LineBatcher::new();
        let color = [1.0, 0.0, 0.0, 1.0];
        
        // Add lines inside viewport
        batcher.add_line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0), color, 1.0);
        batcher.add_line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 5.0), color, 1.0);
        
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        batcher.cull_offscreen_lines(bounds);
        
        assert_eq!(batcher.line_count(), 2, "Lines inside viewport should not be culled");
    }
    
    #[test]
    fn test_lines_outside_viewport_culled() {
        let mut batcher = LineBatcher::new();
        let color = [1.0, 0.0, 0.0, 1.0];
        
        // Add lines completely outside viewport
        batcher.add_line(Vec3::new(100.0, 0.0, 100.0), Vec3::new(110.0, 0.0, 100.0), color, 1.0);
        batcher.add_line(Vec3::new(-100.0, 0.0, -100.0), Vec3::new(-90.0, 0.0, -100.0), color, 1.0);
        
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        batcher.cull_offscreen_lines(bounds);
        
        assert_eq!(batcher.line_count(), 0, "Lines outside viewport should be culled");
    }
    
    #[test]
    fn test_lines_crossing_viewport_boundary_not_culled() {
        let mut batcher = LineBatcher::new();
        let color = [1.0, 0.0, 0.0, 1.0];
        
        // Add line that crosses viewport boundary
        batcher.add_line(Vec3::new(-20.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0), color, 1.0);
        
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        batcher.cull_offscreen_lines(bounds);
        
        assert_eq!(batcher.line_count(), 1, "Lines crossing viewport should not be culled");
    }
    
    #[test]
    fn test_lines_with_one_endpoint_inside_not_culled() {
        let mut batcher = LineBatcher::new();
        let color = [1.0, 0.0, 0.0, 1.0];
        
        // Add line with one endpoint inside, one outside
        batcher.add_line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 0.0), color, 1.0);
        
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        batcher.cull_offscreen_lines(bounds);
        
        assert_eq!(batcher.line_count(), 1, "Lines with one endpoint inside should not be culled");
    }
    
    #[test]
    fn test_culling_removes_empty_batches() {
        let mut batcher = LineBatcher::new();
        
        let red = [1.0, 0.0, 0.0, 1.0];
        let blue = [0.0, 0.0, 1.0, 1.0];
        
        // Add lines: some inside, some outside
        batcher.add_line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0), red, 1.0);
        batcher.add_line(Vec3::new(100.0, 0.0, 100.0), Vec3::new(110.0, 0.0, 100.0), blue, 1.0);
        
        assert_eq!(batcher.batch_count(), 2, "Should have 2 batches before culling");
        
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        batcher.cull_offscreen_lines(bounds);
        
        assert_eq!(batcher.batch_count(), 1, "Should have 1 batch after culling (empty batch removed)");
        assert_eq!(batcher.line_count(), 1, "Should have 1 line remaining");
    }
    
    #[test]
    fn test_viewport_bounds_from_camera() {
        let camera = CameraState {
            position: Vec2::new(100.0, 200.0),
            rotation: 0.0,
            pitch: 45.0,
            zoom: 50.0,
        };
        
        let viewport_size = Vec2::new(800.0, 600.0);
        let margin = 10.0;
        
        let bounds = ViewportBounds::from_camera(&camera, viewport_size, margin);
        
        // Check that bounds are centered on camera position
        let center_x = (bounds.min_x + bounds.max_x) / 2.0;
        let center_z = (bounds.min_z + bounds.max_z) / 2.0;
        
        assert!((center_x - camera.position.x).abs() < 0.1, "Bounds should be centered on camera X");
        assert!((center_z - camera.position.y).abs() < 0.1, "Bounds should be centered on camera Z");
        
        // Check that bounds include margin
        let expected_half_width = (viewport_size.x / (2.0 * camera.zoom)) + margin;
        let actual_half_width = (bounds.max_x - bounds.min_x) / 2.0;
        
        assert!((actual_half_width - expected_half_width).abs() < 0.1, "Bounds should include margin");
    }
    
    #[test]
    fn test_viewport_contains_point() {
        let bounds = ViewportBounds::new(-10.0, 10.0, -10.0, 10.0);
        
        assert!(bounds.contains_point(Vec3::ZERO), "Origin should be inside bounds");
        assert!(bounds.contains_point(Vec3::new(5.0, 0.0, 5.0)), "Point inside should be contained");
        assert!(!bounds.contains_point(Vec3::new(20.0, 0.0, 0.0)), "Point outside X should not be contained");
        assert!(!bounds.contains_point(Vec3::new(0.0, 0.0, 20.0)), "Point outside Z should not be contained");
    }
}

// ============================================================================
// Unit Tests for Draw Call Minimization
// Requirements: 10.1
// ============================================================================

#[cfg(test)]
mod draw_call_minimization_tests {
    use super::*;
    
    #[test]
    fn test_many_similar_lines_single_batch() {
        let mut batcher = LineBatcher::new();
        let color = [1.0, 0.0, 0.0, 1.0];
        let width = 2.0;
        
        // Add 100 lines with same properties
        for i in 0..100 {
            let start = Vec3::new(i as f32, 0.0, 0.0);
            let end = Vec3::new(i as f32, 0.0, 10.0);
            batcher.add_line(start, end, color, width);
        }
        
        // Should be in a single batch (1 draw call)
        assert_eq!(batcher.batch_count(), 1, "100 similar lines should be in 1 batch");
        assert_eq!(batcher.line_count(), 100, "Should have 100 lines");
    }
    
    #[test]
    fn test_grid_with_three_line_types() {
        let mut batcher = LineBatcher::new();
        
        let minor = [0.3, 0.3, 0.3, 0.4];
        let major = [0.4, 0.4, 0.4, 0.6];
        let axis = [0.8, 0.2, 0.2, 0.8];
        
        // Simulate a typical grid with minor, major, and axis lines
        for i in 0..50 {
            batcher.add_line(Vec3::new(i as f32, 0.0, 0.0), Vec3::new(i as f32, 0.0, 50.0), minor, 1.0);
        }
        
        for i in 0..5 {
            batcher.add_line(Vec3::new(i as f32 * 10.0, 0.0, 0.0), Vec3::new(i as f32 * 10.0, 0.0, 50.0), major, 1.5);
        }
        
        batcher.add_line(Vec3::new(0.0, 0.0, -50.0), Vec3::new(0.0, 0.0, 50.0), axis, 2.0);
        batcher.add_line(Vec3::new(-50.0, 0.0, 0.0), Vec3::new(50.0, 0.0, 0.0), axis, 2.0);
        
        // Should have exactly 3 batches (3 draw calls)
        assert_eq!(batcher.batch_count(), 3, "Grid should have 3 batches (minor, major, axis)");
        assert_eq!(batcher.line_count(), 57, "Should have 57 lines total");
    }
    
    #[test]
    fn test_batch_count_scales_with_unique_properties() {
        let mut batcher = LineBatcher::new();
        
        // Add lines with N unique color/width combinations
        let num_unique = 5;
        for i in 0..num_unique {
            let color = [i as f32 / 10.0, 0.0, 0.0, 1.0];
            let width = 1.0 + i as f32;
            
            // Add multiple lines with this property combination
            for j in 0..10 {
                batcher.add_line(
                    Vec3::new(j as f32, 0.0, i as f32),
                    Vec3::new(j as f32 + 1.0, 0.0, i as f32),
                    color,
                    width
                );
            }
        }
        
        // Should have exactly num_unique batches
        assert_eq!(batcher.batch_count(), num_unique, "Batch count should equal unique property combinations");
        assert_eq!(batcher.line_count(), num_unique * 10, "Should have all lines");
    }
    
    #[test]
    fn test_batch_efficiency_ratio() {
        let mut batcher = LineBatcher::new();
        
        let color1 = [1.0, 0.0, 0.0, 1.0];
        let color2 = [0.0, 1.0, 0.0, 1.0];
        
        // Add 50 lines of each color
        for i in 0..50 {
            batcher.add_line(Vec3::new(i as f32, 0.0, 0.0), Vec3::new(i as f32, 0.0, 10.0), color1, 1.0);
            batcher.add_line(Vec3::new(i as f32, 0.0, 20.0), Vec3::new(i as f32, 0.0, 30.0), color2, 1.0);
        }
        
        // Efficiency: 100 lines in 2 batches = 50:1 ratio
        let efficiency = batcher.line_count() as f32 / batcher.batch_count() as f32;
        assert!(efficiency >= 50.0, "Should have high batching efficiency (50:1 or better)");
    }
    
    #[test]
    fn test_empty_batcher_has_no_batches() {
        let batcher = LineBatcher::new();
        
        assert_eq!(batcher.batch_count(), 0, "Empty batcher should have 0 batches");
        assert_eq!(batcher.line_count(), 0, "Empty batcher should have 0 lines");
    }
}
