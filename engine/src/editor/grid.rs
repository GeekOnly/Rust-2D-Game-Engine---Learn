/// Grid system for scene view
use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct SceneGrid {
    pub enabled: bool,
    pub size: f32,
    pub snap_enabled: bool,
    
    // Visual settings
    pub color: [f32; 4],
    pub axis_color_x: [f32; 4],  // Red for X axis
    pub axis_color_z: [f32; 4],  // Blue for Z axis
    pub fade_distance: f32,       // Distance at which grid starts fading
    pub fade_range: f32,          // Range over which fade occurs
    
    // Adaptive grid
    pub subdivision_levels: Vec<f32>,  // [1.0, 0.1, 0.01] for multi-scale grid
    pub min_line_spacing: f32,         // Minimum pixels between lines
}

impl SceneGrid {
    pub fn new() -> Self {
        Self {
            enabled: true,
            size: 1.0,  // 1 unit per grid cell (like Blender: 1 unit = 1 meter)
            snap_enabled: false,
            color: [0.3, 0.3, 0.3, 0.5],  // Gray grid lines
            axis_color_x: [0.8, 0.2, 0.2, 0.8],  // Red for X axis
            axis_color_z: [0.2, 0.2, 0.8, 0.8],  // Blue for Z axis
            fade_distance: 500.0,
            fade_range: 200.0,
            subdivision_levels: vec![1.0, 0.1, 0.01],
            min_line_spacing: 20.0,  // Minimum 20 pixels between grid lines
        }
    }
    
    /// Snap position to grid
    pub fn snap(&self, position: Vec2) -> Vec2 {
        if self.snap_enabled {
            Vec2::new(
                (position.x / self.size).round() * self.size,
                (position.y / self.size).round() * self.size,
            )
        } else {
            position
        }
    }
    
    /// Toggle grid visibility
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    
    /// Toggle snap
    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
    }
    
    /// Set grid size
    pub fn set_size(&mut self, size: f32) {
        self.size = size.max(0.01);
    }
    
    /// Select appropriate grid level based on zoom
    /// Returns the grid spacing that maintains visual density
    pub fn select_grid_level(&self, zoom: f32) -> f32 {
        let base_spacing = self.size;
        let target_min = self.min_line_spacing;
        let target_max = self.min_line_spacing * 5.0;
        let target_mid = (target_min + target_max) / 2.0;
        
        // Calculate screen-space spacing for base grid
        let screen_spacing = base_spacing * zoom;
        
        // If base spacing is in the sweet spot, use it
        if screen_spacing >= target_min && screen_spacing <= target_max {
            return base_spacing;
        }
        
        // Try predefined subdivision levels first (for nice round numbers)
        let mut best_spacing = base_spacing;
        let mut best_distance = (screen_spacing - target_mid).abs();
        
        // Check scaling up (for when base is too small)
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
        
        // Check scaling down (for when base is too large)
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
        
        // If we found a good subdivision level, use it
        if best_distance < (target_max - target_min) / 2.0 {
            return best_spacing;
        }
        
        // Otherwise, calculate the optimal spacing to hit the target range
        // Aim for the middle of the target range
        let optimal_spacing = target_mid / zoom;
        
        // Round to a nice number (power of 10)
        let magnitude = 10.0_f32.powf(optimal_spacing.log10().floor());
        let normalized = optimal_spacing / magnitude;
        
        // Round to nearest 1, 2, or 5
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
    
    /// Calculate fade alpha based on distance from camera
    /// Returns alpha value in range [0.0, 1.0]
    pub fn calculate_fade_alpha(&self, distance: f32) -> f32 {
        if distance < self.fade_distance {
            1.0
        } else if distance > self.fade_distance + self.fade_range {
            0.0
        } else {
            // Linear fade within fade range
            let fade_progress = (distance - self.fade_distance) / self.fade_range;
            1.0 - fade_progress
        }
    }
    
    /// Generate 2D grid lines (orthogonal to world axes)
    /// Returns (horizontal_lines, vertical_lines) where each line is (start, end)
    pub fn generate_2d_grid_lines(
        &self,
        camera_pos: Vec2,
        viewport_size: Vec2,
        zoom: f32,
    ) -> (Vec<(Vec2, Vec2)>, Vec<(Vec2, Vec2)>) {
        let grid_spacing = self.select_grid_level(zoom);
        
        // Calculate visible world space bounds
        let half_width = viewport_size.x / (2.0 * zoom);
        let half_height = viewport_size.y / (2.0 * zoom);
        
        let min_x = camera_pos.x - half_width;
        let max_x = camera_pos.x + half_width;
        let min_y = camera_pos.y - half_height;
        let max_y = camera_pos.y + half_height;
        
        let mut horizontal_lines = Vec::new();
        let mut vertical_lines = Vec::new();
        
        // Generate vertical lines (parallel to Y axis)
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            vertical_lines.push((
                Vec2::new(x, min_y),
                Vec2::new(x, max_y),
            ));
            x += grid_spacing;
        }
        
        // Generate horizontal lines (parallel to X axis)
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
    
    /// Generate 3D grid lines with perspective projection
    /// Returns lines in 3D space that need to be projected to screen
    pub fn generate_3d_grid_lines(
        &self,
        camera_pos: Vec2,
        zoom: f32,
        view_distance: f32,
    ) -> Vec<(Vec3, Vec3, f32)> {
        let grid_spacing = self.select_grid_level(zoom);
        
        // Calculate visible range based on view distance
        let visible_range = view_distance * 2.0;
        
        let min_x = camera_pos.x - visible_range;
        let max_x = camera_pos.x + visible_range;
        let min_z = camera_pos.y - visible_range;
        let max_z = camera_pos.y + visible_range;
        
        let mut lines = Vec::new();
        
        // Generate lines parallel to X axis (running along X, constant Z)
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            let distance = (Vec2::new(camera_pos.x, z) - camera_pos).length();
            lines.push((start, end, distance));
            z += grid_spacing;
        }
        
        // Generate lines parallel to Z axis (running along Z, constant X)
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
    
    /// Check if a line is an axis line (X or Z)
    pub fn is_x_axis(&self, line_start: Vec3, line_end: Vec3) -> bool {
        // X axis line: runs along X (varying x), constant z near 0
        line_start.z.abs() < 0.01 && line_end.z.abs() < 0.01 && (line_start.x - line_end.x).abs() > 0.01
    }
    
    pub fn is_z_axis(&self, line_start: Vec3, line_end: Vec3) -> bool {
        // Z axis line: runs along Z (varying z), constant x near 0
        line_start.x.abs() < 0.01 && line_end.x.abs() < 0.01 && (line_start.z - line_end.z).abs() > 0.01
    }
}

impl Default for SceneGrid {
    fn default() -> Self {
        Self::new()
    }
}
