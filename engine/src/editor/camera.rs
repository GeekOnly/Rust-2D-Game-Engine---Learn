/// Scene camera controller for Unity-like editor
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32, // In degrees (for 3D view)
    
    // Camera bounds
    pub min_zoom: f32,
    pub max_zoom: f32,
    
    // Pan state
    is_panning: bool,
    last_mouse_pos: Vec2,
    
    // Rotation state (for 3D view)
    is_rotating: bool,
    rotation_sensitivity: f32,
}

impl SceneCamera {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            is_rotating: false,
            rotation_sensitivity: 0.5,
        }
    }
    
    /// Start panning (middle mouse button pressed)
    pub fn start_pan(&mut self, mouse_pos: Vec2) {
        self.is_panning = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    /// Update pan (middle mouse button held)
    pub fn update_pan(&mut self, mouse_pos: Vec2) {
        if self.is_panning {
            let delta = mouse_pos - self.last_mouse_pos;
            self.position -= delta / self.zoom;
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop panning (middle mouse button released)
    pub fn stop_pan(&mut self) {
        self.is_panning = false;
    }
    
    /// Zoom in/out (scroll wheel) - improved version
    pub fn zoom(&mut self, delta: f32, _mouse_pos: Vec2) {
        // Smooth exponential zoom
        let zoom_speed = 0.15;
        let zoom_factor = if delta > 0.0 {
            1.0 + zoom_speed
        } else {
            1.0 - zoom_speed
        };
        
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
    }
    
    /// Start rotation (right mouse button pressed)
    pub fn start_rotate(&mut self, mouse_pos: Vec2) {
        self.is_rotating = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    /// Update rotation (right mouse button held)
    pub fn update_rotate(&mut self, mouse_pos: Vec2) {
        if self.is_rotating {
            let delta = mouse_pos - self.last_mouse_pos;
            // Horizontal movement rotates around Y axis
            self.rotation += delta.x * self.rotation_sensitivity;
            // Keep rotation in 0-360 range
            self.rotation = self.rotation.rem_euclid(360.0);
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop rotation (right mouse button released)
    pub fn stop_rotate(&mut self) {
        self.is_rotating = false;
    }
    
    /// Frame selected object (F key)
    pub fn frame_object(&mut self, object_pos: Vec2, object_size: Vec2, viewport_size: Vec2) {
        self.position = object_pos;
        
        // Calculate zoom to fit object in view
        let zoom_x = viewport_size.x / (object_size.x * 1.5);
        let zoom_y = viewport_size.y / (object_size.y * 1.5);
        self.zoom = zoom_x.min(zoom_y).clamp(self.min_zoom, self.max_zoom);
    }
    
    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.position + screen_pos / self.zoom
    }
    
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos - self.position) * self.zoom
    }
    
    /// Reset camera to default
    pub fn reset(&mut self) {
        self.position = Vec2::ZERO;
        self.zoom = 1.0;
        self.rotation = 0.0;
        self.is_panning = false;
        self.is_rotating = false;
    }
    
    /// Check if camera is being controlled
    pub fn is_controlling(&self) -> bool {
        self.is_panning || self.is_rotating
    }
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self::new()
    }
}
