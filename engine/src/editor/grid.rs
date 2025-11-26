/// Grid system for scene view
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct SceneGrid {
    pub enabled: bool,
    pub size: f32,
    pub snap_enabled: bool,
    pub color: [f32; 4],
    pub axis_color: [f32; 4],
}

impl SceneGrid {
    pub fn new() -> Self {
        Self {
            enabled: true,
            size: 1.0,  // 1 unit per grid cell (like Blender: 1 unit = 1 meter)
            snap_enabled: false,
            color: [0.3, 0.3, 0.3, 0.5],  // Gray grid lines
            axis_color: [0.5, 0.5, 0.5, 0.8],  // Brighter axis lines
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
        self.size = size.max(1.0);
    }
}

impl Default for SceneGrid {
    fn default() -> Self {
        Self::new()
    }
}
