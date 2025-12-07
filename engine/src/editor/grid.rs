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

/// Infinite grid system with multi-level support and smooth fading
#[derive(Debug, Clone)]
pub struct InfiniteGrid {
    pub enabled: bool,
    
    // Grid appearance
    pub base_unit: f32,  // Base grid unit (e.g., 1.0 meter)
    pub major_line_every: u32,  // Major line every N units (e.g., 10)
    
    // Colors
    pub minor_line_color: [f32; 4],
    pub major_line_color: [f32; 4],
    pub x_axis_color: [f32; 4],
    pub z_axis_color: [f32; 4],
    
    // Line widths
    pub minor_line_width: f32,
    pub major_line_width: f32,
    pub axis_line_width: f32,
    
    // Fade settings
    pub fade_start_distance: f32,  // Distance where fade begins
    pub fade_end_distance: f32,    // Distance where grid disappears
    pub near_fade_start: f32,      // Distance where near fade begins
    pub near_fade_end: f32,        // Distance where near fade completes
    
    // Adaptive grid
    pub min_pixel_spacing: f32,    // Minimum pixels between lines
    pub max_pixel_spacing: f32,    // Maximum pixels between lines
    pub level_transition_range: f32, // Range for smooth level transitions
    
    // Performance
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

#[derive(Debug, Clone, PartialEq)]
pub struct CameraState {
    pub position: Vec2,
    pub rotation: f32,
    pub pitch: f32,
    pub zoom: f32,
}

/// Grid level for adaptive multi-level grid
#[derive(Debug, Clone)]
pub struct GridLevel {
    pub unit_size: f32,      // Size of one grid cell at this level
    pub alpha: f32,          // Current alpha for smooth transitions
    pub is_active: bool,     // Whether this level is currently visible
}

/// Adaptive grid levels manager
#[derive(Debug, Clone)]
pub struct AdaptiveGridLevels {
    pub levels: Vec<GridLevel>,
    pub current_primary: usize,
    pub transition_progress: f32,
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

// ============================================================================
// INFINITE GRID IMPLEMENTATION
// ============================================================================

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
    
    /// Calculate appropriate grid level for current zoom
    /// Returns the grid unit size that maintains visual density
    pub fn calculate_grid_level(&self, zoom: f32) -> f32 {
        let base_spacing = self.base_unit;
        let target_mid = (self.min_pixel_spacing + self.max_pixel_spacing) / 2.0;
        
        // Calculate screen-space spacing for base grid
        let screen_spacing = base_spacing * zoom;
        
        // If base spacing is in the sweet spot, use it
        if screen_spacing >= self.min_pixel_spacing && screen_spacing <= self.max_pixel_spacing {
            return base_spacing;
        }
        
        // Calculate the optimal spacing to hit the target range
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
    
    /// Calculate fade alpha for a point based on distance from camera
    pub fn calculate_fade_alpha(&self, point: Vec3, camera_pos: Vec3) -> f32 {
        let distance = (point - camera_pos).length();
        
        // Far fade (distance from camera)
        let far_alpha = if distance < self.fade_start_distance {
            1.0
        } else if distance > self.fade_end_distance {
            0.0
        } else {
            let fade_progress = (distance - self.fade_start_distance) 
                / (self.fade_end_distance - self.fade_start_distance);
            1.0 - fade_progress
        };
        
        // Near fade (too close to camera)
        let near_alpha = if distance > self.near_fade_start {
            1.0
        } else if distance < self.near_fade_end {
            0.0
        } else {
            let fade_progress = (distance - self.near_fade_end) 
                / (self.near_fade_start - self.near_fade_end);
            fade_progress
        };
        
        // Combine both fades
        far_alpha * near_alpha
    }
    
    /// Check if grid geometry needs regeneration
    pub fn needs_regeneration(&self, camera: &CameraState) -> bool {
        if let Some(last_state) = &self.last_camera_state {
            last_state.has_changed_significantly(camera, 0.1)
        } else {
            true
        }
    }
    
    /// Generate grid geometry for current camera view
    pub fn generate_geometry(
        &mut self,
        camera: &CameraState,
        viewport_size: Vec2,
    ) -> &GridGeometry {
        // Check if we can use cached geometry
        if !self.needs_regeneration(camera) {
            if let Some(ref geometry) = self.cached_geometry {
                return geometry;
            }
        }
        
        // Generate new geometry
        let mut lines = Vec::new();
        
        // Calculate grid level
        let grid_spacing = self.calculate_grid_level(camera.zoom);
        
        // Calculate visible range based on camera position and zoom
        let visible_range = 1000.0; // Extend far into distance
        
        let min_x = camera.position.x - visible_range;
        let max_x = camera.position.x + visible_range;
        let min_z = camera.position.y - visible_range;
        let max_z = camera.position.y + visible_range;
        
        // Camera position in 3D
        let yaw_rad = camera.rotation.to_radians();
        let pitch_rad = camera.pitch.to_radians();
        let distance = 500.0; // Default distance
        
        let cam_x = camera.position.x + distance * yaw_rad.cos() * pitch_rad.cos();
        let cam_y = distance * pitch_rad.sin();
        let cam_z = camera.position.y + distance * yaw_rad.sin() * pitch_rad.cos();
        let camera_pos_3d = Vec3::new(cam_x, cam_y, cam_z);
        
        // Generate lines parallel to X axis (running along X, constant Z)
        let start_z = (min_z / grid_spacing).floor() * grid_spacing;
        let mut z = start_z;
        while z <= max_z {
            let start = Vec3::new(min_x, 0.0, z);
            let end = Vec3::new(max_x, 0.0, z);
            
            // Determine line type
            let line_type = if z.abs() < 0.01 {
                GridLineType::ZAxis
            } else if (z / grid_spacing).rem_euclid(self.major_line_every as f32).abs() < 0.01 {
                GridLineType::Major
            } else {
                GridLineType::Minor
            };
            
            // Calculate fade alpha
            let mid_point = Vec3::new((min_x + max_x) / 2.0, 0.0, z);
            let alpha = self.calculate_fade_alpha(mid_point, camera_pos_3d);
            
            // Get color and width based on line type
            let (mut color, width) = match line_type {
                GridLineType::Minor => (self.minor_line_color, self.minor_line_width),
                GridLineType::Major => (self.major_line_color, self.major_line_width),
                GridLineType::ZAxis => (self.z_axis_color, self.axis_line_width),
                GridLineType::XAxis => (self.x_axis_color, self.axis_line_width),
            };
            
            // Apply fade alpha
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
        
        // Generate lines parallel to Z axis (running along Z, constant X)
        let start_x = (min_x / grid_spacing).floor() * grid_spacing;
        let mut x = start_x;
        while x <= max_x {
            let start = Vec3::new(x, 0.0, min_z);
            let end = Vec3::new(x, 0.0, max_z);
            
            // Determine line type
            let line_type = if x.abs() < 0.01 {
                GridLineType::XAxis
            } else if (x / grid_spacing).rem_euclid(self.major_line_every as f32).abs() < 0.01 {
                GridLineType::Major
            } else {
                GridLineType::Minor
            };
            
            // Calculate fade alpha
            let mid_point = Vec3::new(x, 0.0, (min_z + max_z) / 2.0);
            let alpha = self.calculate_fade_alpha(mid_point, camera_pos_3d);
            
            // Get color and width based on line type
            let (mut color, width) = match line_type {
                GridLineType::Minor => (self.minor_line_color, self.minor_line_width),
                GridLineType::Major => (self.major_line_color, self.major_line_width),
                GridLineType::XAxis => (self.x_axis_color, self.axis_line_width),
                GridLineType::ZAxis => (self.z_axis_color, self.axis_line_width),
            };
            
            // Apply fade alpha
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
        
        // Cache the geometry
        let geometry = GridGeometry {
            lines,
            generation_time: std::time::Instant::now(),
        };
        
        self.cached_geometry = Some(geometry);
        self.last_camera_state = Some(camera.clone());
        
        self.cached_geometry.as_ref().unwrap()
    }
}

impl Default for InfiniteGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraState {
    /// Check if camera has moved significantly
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

impl AdaptiveGridLevels {
    pub fn new() -> Self {
        Self {
            levels: vec![
                GridLevel { unit_size: 1.0, alpha: 1.0, is_active: true },
                GridLevel { unit_size: 10.0, alpha: 0.0, is_active: false },
                GridLevel { unit_size: 100.0, alpha: 0.0, is_active: false },
            ],
            current_primary: 0,
            transition_progress: 0.0,
        }
    }
    
    /// Update grid levels based on camera zoom
    pub fn update(&mut self, camera: &CameraState, delta_time: f32) {
        // Calculate target level based on zoom
        let screen_spacing = self.levels[self.current_primary].unit_size * camera.zoom;
        
        // Determine if we need to transition to a different level
        let target_min = 20.0;
        let target_max = 100.0;
        
        if screen_spacing < target_min && self.current_primary > 0 {
            // Transition to finer level
            self.current_primary -= 1;
            self.transition_progress = 0.0;
        } else if screen_spacing > target_max && self.current_primary < self.levels.len() - 1 {
            // Transition to coarser level
            self.current_primary += 1;
            self.transition_progress = 0.0;
        }
        
        // Update transition progress
        if self.transition_progress < 1.0 {
            self.transition_progress += delta_time * 2.0; // 0.5 second transition
            self.transition_progress = self.transition_progress.min(1.0);
        }
        
        // Update alpha values for smooth transitions
        for (i, level) in self.levels.iter_mut().enumerate() {
            if i == self.current_primary {
                level.alpha = self.transition_progress;
                level.is_active = true;
            } else if i == self.current_primary + 1 || i == self.current_primary - 1 {
                level.alpha = 1.0 - self.transition_progress;
                level.is_active = level.alpha > 0.01;
            } else {
                level.alpha = 0.0;
                level.is_active = false;
            }
        }
    }
    
    /// Get all active levels with their alphas
    pub fn get_active_levels(&self) -> Vec<(f32, f32)> {
        self.levels
            .iter()
            .filter(|l| l.is_active)
            .map(|l| (l.unit_size, l.alpha))
            .collect()
    }
}

impl Default for AdaptiveGridLevels {
    fn default() -> Self {
        Self::new()
    }
}
