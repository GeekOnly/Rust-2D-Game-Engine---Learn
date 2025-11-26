/// Scene camera controller for Unity-like editor
use glam::Vec2;

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,    // Horizontal rotation (yaw) in degrees
    pub pitch: f32,       // Vertical rotation in degrees
    pub distance: f32,    // Distance from pivot point (for orbit)
    pub pivot: Vec2,      // Pivot point for orbit mode
    
    // Camera bounds
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    
    // Pan state
    is_panning: bool,
    last_mouse_pos: Vec2,
    
    // Rotation state (for 3D view)
    is_rotating: bool,
    is_orbiting: bool,
    rotation_sensitivity: f32,
}

impl SceneCamera {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 50.0,       // Zoom to convert world units to screen pixels (50 pixels per unit)
            rotation: 45.0,   // Default 45° angle
            pitch: 30.0,      // Default 30° pitch
            distance: 500.0,  // Default distance
            pivot: Vec2::ZERO,
            min_zoom: 5.0,    // Min zoom adjusted for world units
            max_zoom: 200.0,  // Max zoom adjusted for world units
            min_pitch: -89.0,
            max_pitch: 89.0,
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            is_rotating: false,
            is_orbiting: false,
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

            // In 3D mode, pan should respect camera rotation
            // Convert screen space delta to world space delta
            let yaw_rad = self.rotation.to_radians();
            let cos_yaw = yaw_rad.cos();
            let sin_yaw = yaw_rad.sin();

            // Pan along camera's local X and Z axes
            // Inverted to match Unity behavior (drag right = move camera right = world moves left)
            let pan_speed = 1.0 / self.zoom;
            let world_delta_x = -(delta.x * cos_yaw + delta.y * sin_yaw) * pan_speed;
            let world_delta_z = -(-delta.x * sin_yaw + delta.y * cos_yaw) * pan_speed;

            self.position.x += world_delta_x;
            self.position.y += world_delta_z; // position.y maps to world Z

            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop panning (middle mouse button released)
    pub fn stop_pan(&mut self) {
        self.is_panning = false;
    }
    
    /// Zoom in/out (scroll wheel) - improved version
    pub fn zoom(&mut self, delta: f32, _mouse_pos: Vec2) {
        // Smooth exponential zoom with better sensitivity
        let zoom_speed = 0.1; // Reduced from 0.15 for smoother control
        let zoom_factor = if delta > 0.0 {
            1.0 + zoom_speed
        } else {
            1.0 / (1.0 + zoom_speed) // Use division for symmetric zoom
        };
        
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
    }
    
    /// Start rotation (right mouse button pressed)
    pub fn start_rotate(&mut self, mouse_pos: Vec2) {
        self.is_rotating = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    /// Update rotation (right mouse button held) - Free look
    pub fn update_rotate(&mut self, mouse_pos: Vec2) {
        if self.is_rotating {
            let delta = mouse_pos - self.last_mouse_pos;
            // Horizontal movement rotates around Y axis (yaw)
            self.rotation += delta.x * self.rotation_sensitivity;
            self.rotation = self.rotation.rem_euclid(360.0);
            
            // Vertical movement changes pitch
            self.pitch -= delta.y * self.rotation_sensitivity;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop rotation (right mouse button released)
    pub fn stop_rotate(&mut self) {
        self.is_rotating = false;
    }
    
    /// Start orbit (Alt + Left mouse button)
    pub fn start_orbit(&mut self, mouse_pos: Vec2, pivot_point: Vec2) {
        self.is_orbiting = true;
        self.pivot = pivot_point;
        self.last_mouse_pos = mouse_pos;
    }
    
    /// Update orbit (Alt + Left mouse button held)
    pub fn update_orbit(&mut self, mouse_pos: Vec2) {
        if self.is_orbiting {
            let delta = mouse_pos - self.last_mouse_pos;
            
            // Rotate around pivot
            self.rotation += delta.x * self.rotation_sensitivity;
            self.rotation = self.rotation.rem_euclid(360.0);
            
            self.pitch -= delta.y * self.rotation_sensitivity;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop orbit
    pub fn stop_orbit(&mut self) {
        self.is_orbiting = false;
    }
    
    /// Focus on object (F key)
    pub fn focus_on(&mut self, target_pos: Vec2, object_size: f32) {
        self.pivot = target_pos;
        self.position = target_pos;
        
        // Set appropriate distance based on object size
        self.distance = object_size * 3.0;
        
        // Adjust zoom to frame object nicely
        self.zoom = 1.0;
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
        self.rotation = 45.0;
        self.pitch = 30.0;
        self.distance = 500.0;
        self.pivot = Vec2::ZERO;
        self.is_panning = false;
        self.is_rotating = false;
        self.is_orbiting = false;
    }
    
    /// Check if camera is being controlled
    pub fn is_controlling(&self) -> bool {
        self.is_panning || self.is_rotating || self.is_orbiting
    }
    
    /// Get rotation matrix for transforming gizmo
    pub fn get_rotation_radians(&self) -> f32 {
        self.rotation.to_radians()
    }
    
    /// Get pitch in radians
    pub fn get_pitch_radians(&self) -> f32 {
        self.pitch.to_radians()
    }
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self::new()
    }
}
