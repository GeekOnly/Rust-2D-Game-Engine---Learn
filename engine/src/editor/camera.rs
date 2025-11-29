/// Scene camera controller for Unity-like editor
use glam::{Vec2, Vec3, Mat4};
use serde::{Deserialize, Serialize};

/// Camera settings for sensitivity and damping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    // Sensitivity settings
    pub pan_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    
    // Damping settings (0.0 = no damping, 1.0 = maximum damping)
    pub pan_damping: f32,
    pub rotation_damping: f32,
    pub zoom_damping: f32,
    
    // Inertia settings
    pub enable_inertia: bool,
    pub inertia_decay: f32,  // How quickly momentum decays (0.0-1.0)
    
    // Zoom settings
    pub zoom_to_cursor: bool,
    pub zoom_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pan_sensitivity: 0.5,   // Reduced for slower, more precise panning
            rotation_sensitivity: 0.5,
            zoom_sensitivity: 0.12, // Optimized for 2D mode
            pan_damping: 0.08,      // Reduced for more responsive panning
            rotation_damping: 0.12,
            zoom_damping: 0.08,     // Further reduced for instant zoom response
            enable_inertia: false,  // Disabled by default for more predictable behavior
            inertia_decay: 0.92,    // Faster decay when enabled
            zoom_to_cursor: true,   // Zoom to cursor (better for precise editing)
            zoom_speed: 20.0,       // Increased for faster zoom response in 2D
        }
    }
}

/// Camera velocity for tracking movement momentum
#[derive(Debug, Clone)]
pub struct CameraVelocity {
    pub pan_velocity: Vec2,
    pub rotation_velocity: Vec2,  // (yaw_velocity, pitch_velocity)
    pub zoom_velocity: f32,
}

impl Default for CameraVelocity {
    fn default() -> Self {
        Self {
            pan_velocity: Vec2::ZERO,
            rotation_velocity: Vec2::ZERO,
            zoom_velocity: 0.0,
        }
    }
}

/// Camera state for mode switching
#[derive(Debug, Clone)]
pub struct CameraState {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub pitch: f32,
}

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
    
    // Settings
    pub settings: CameraSettings,
    pub rotation_sensitivity: f32,  // Kept for backward compatibility
    pub zoom_sensitivity: f32,      // Kept for backward compatibility
    pub pan_speed: f32,
    
    // Velocity tracking for inertia
    velocity: CameraVelocity,
    
    // Target values for smooth interpolation
    target_position: Vec2,
    target_rotation: f32,
    target_pitch: f32,
    target_zoom: f32,
    
    // Smooth interpolation
    zoom_interpolation_speed: f32,
    
    // Mode switching state
    saved_3d_state: Option<CameraState>,
    
    // Cursor tracking for zoom
    last_cursor_world_pos: Option<Vec2>,
}

impl SceneCamera {
    pub fn new() -> Self {
        let initial_zoom = 50.0;
        let settings = CameraSettings::default();
        Self {
            position: Vec2::ZERO,
            zoom: initial_zoom,       // Zoom to convert world units to screen pixels (50 pixels per unit)
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
            rotation_sensitivity: settings.rotation_sensitivity,
            zoom_sensitivity: settings.zoom_sensitivity,
            pan_speed: 1.0,
            settings,
            velocity: CameraVelocity::default(),
            target_position: Vec2::ZERO,
            target_rotation: 45.0,
            target_pitch: 30.0,
            target_zoom: initial_zoom,
            zoom_interpolation_speed: 10.0,
            saved_3d_state: None,
            last_cursor_world_pos: None,
        }
    }
    
    /// Start panning (middle mouse button pressed)
    pub fn start_pan(&mut self, mouse_pos: Vec2) {
        // Validate mouse position
        if !mouse_pos.is_finite() {
            return;
        }
        self.is_panning = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    /// Update pan (middle mouse button held)
    pub fn update_pan(&mut self, mouse_pos: Vec2) {
        if self.is_panning {
            let delta = mouse_pos - self.last_mouse_pos;

            // Simple pan calculation for 2D mode
            // Convert screen space delta to world space delta
            let pan_speed = self.settings.pan_sensitivity / self.zoom;
            
            // Direct X/Y movement (inverted to match Unity: drag right = world moves left)
            let world_delta = Vec2::new(-delta.x, delta.y) * pan_speed;

            // Update position immediately for responsive panning
            self.position += world_delta;
            self.target_position = self.position;
            
            // Add to velocity for inertia (if enabled)
            if self.settings.enable_inertia {
                self.velocity.pan_velocity += world_delta * 0.3;
            }

            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop panning (middle mouse button released)
    pub fn stop_pan(&mut self) {
        self.is_panning = false;
        // Velocity will continue to move camera if inertia is enabled
    }
    
    /// Zoom in/out (scroll wheel) - improved version with cursor-based zooming
    pub fn zoom(&mut self, delta: f32, mouse_pos: Vec2) {
        // Calculate world position under cursor BEFORE zoom
        let world_pos_before = self.screen_to_world(mouse_pos);
        
        // Calculate zoom factor
        let zoom_factor = if delta > 0.0 {
            1.0 + self.settings.zoom_sensitivity
        } else {
            1.0 / (1.0 + self.settings.zoom_sensitivity)
        };
        
        let old_zoom = self.zoom;
        
        // Apply zoom immediately for responsive feel
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
        self.target_zoom = self.zoom;
        
        // Adjust camera position to zoom towards cursor (if enabled)
        if self.settings.zoom_to_cursor {
            // Calculate world position under cursor AFTER zoom
            let world_pos_after = self.screen_to_world(mouse_pos);
            
            // Adjust camera position to keep the same world point under cursor
            let world_offset = world_pos_before - world_pos_after;
            self.position += world_offset;
            self.target_position = self.position;
        }
        
        // Add to velocity for inertia (if enabled)
        if self.settings.enable_inertia {
            let zoom_delta = self.zoom - old_zoom;
            self.velocity.zoom_velocity += zoom_delta * 0.2;
        }
    }
    
    /// Update camera state (call each frame for smooth interpolation)
    pub fn update(&mut self, delta_time: f32) {
        // Apply damping to smooth out movements
        self.apply_damping(delta_time);
        
        // Apply inertia when input stops
        if !self.is_controlling() {
            self.apply_inertia(delta_time);
        }
        
        // Smooth interpolation toward target values
        self.interpolate_to_targets(delta_time);
        
        // Handle cursor-based zoom adjustment
        if let Some(world_pos_before) = self.last_cursor_world_pos {
            // This would need cursor position, so we'll handle it differently
            // For now, clear the stored position after zoom completes
            if (self.zoom - self.target_zoom).abs() < 0.01 {
                self.last_cursor_world_pos = None;
            }
        }
    }
    
    /// Apply damping to velocity
    fn apply_damping(&mut self, delta_time: f32) {
        // Exponential damping for smooth deceleration
        let pan_damping_factor = 1.0 - (-self.settings.pan_damping * 10.0 * delta_time).exp();
        let rotation_damping_factor = 1.0 - (-self.settings.rotation_damping * 10.0 * delta_time).exp();
        let zoom_damping_factor = 1.0 - (-self.settings.zoom_damping * 10.0 * delta_time).exp();
        
        // Apply damping to position
        if (self.position - self.target_position).length() > 0.001 {
            self.position = self.position + (self.target_position - self.position) * pan_damping_factor;
        } else {
            self.position = self.target_position;
        }
        
        // Apply damping to rotation
        if (self.rotation - self.target_rotation).abs() > 0.01 {
            self.rotation = self.rotation + (self.target_rotation - self.rotation) * rotation_damping_factor;
        } else {
            self.rotation = self.target_rotation;
        }
        
        // Apply damping to pitch
        if (self.pitch - self.target_pitch).abs() > 0.01 {
            self.pitch = self.pitch + (self.target_pitch - self.pitch) * rotation_damping_factor;
        } else {
            self.pitch = self.target_pitch;
        }
        
        // Apply damping to zoom
        if (self.zoom - self.target_zoom).abs() > 0.01 {
            self.zoom = self.zoom + (self.target_zoom - self.zoom) * zoom_damping_factor;
        } else {
            self.zoom = self.target_zoom;
        }
    }
    
    /// Apply inertia when input stops
    fn apply_inertia(&mut self, delta_time: f32) {
        if !self.settings.enable_inertia {
            // Clear velocities if inertia is disabled
            self.velocity = CameraVelocity::default();
            return;
        }
        
        // Apply pan velocity
        if self.velocity.pan_velocity.length() > 0.001 {
            self.target_position += self.velocity.pan_velocity * delta_time * 60.0;
            // Decay velocity exponentially
            self.velocity.pan_velocity *= self.settings.inertia_decay;
        } else {
            self.velocity.pan_velocity = Vec2::ZERO;
        }
        
        // Apply rotation velocity
        if self.velocity.rotation_velocity.length() > 0.001 {
            self.target_rotation += self.velocity.rotation_velocity.x * delta_time * 60.0;
            self.target_pitch += self.velocity.rotation_velocity.y * delta_time * 60.0;
            self.target_pitch = self.target_pitch.clamp(self.min_pitch, self.max_pitch);
            // Decay velocity exponentially
            self.velocity.rotation_velocity *= self.settings.inertia_decay;
        } else {
            self.velocity.rotation_velocity = Vec2::ZERO;
        }
        
        // Apply zoom velocity
        if self.velocity.zoom_velocity.abs() > 0.001 {
            self.target_zoom += self.velocity.zoom_velocity * delta_time * 60.0;
            self.target_zoom = self.target_zoom.clamp(self.min_zoom, self.max_zoom);
            // Decay velocity exponentially
            self.velocity.zoom_velocity *= self.settings.inertia_decay;
        } else {
            self.velocity.zoom_velocity = 0.0;
        }
    }
    
    /// Smooth interpolation toward target values
    fn interpolate_to_targets(&mut self, _delta_time: f32) {
        // This is now handled in apply_damping
        // Kept as separate method for clarity and future enhancements
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
            let yaw_delta = delta.x * self.settings.rotation_sensitivity;
            let pitch_delta = -delta.y * self.settings.rotation_sensitivity;
            
            self.target_rotation += yaw_delta;
            self.target_rotation = self.target_rotation.rem_euclid(360.0);
            
            // Vertical movement changes pitch
            self.target_pitch += pitch_delta;
            self.target_pitch = self.target_pitch.clamp(self.min_pitch, self.max_pitch);
            
            // Add to velocity for inertia
            if self.settings.enable_inertia {
                self.velocity.rotation_velocity += Vec2::new(yaw_delta, pitch_delta);
            }
            
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
        // Ensure target position matches current position to avoid damping drift
        self.target_position = self.position;
        // Calculate and store the distance and rotation based on current position
        let offset = self.position - self.pivot;
        self.distance = offset.length();
        // Calculate the rotation angle from the offset to maintain current orientation
        if self.distance > 0.001 {
            self.target_rotation = offset.y.atan2(offset.x).to_degrees();
            self.rotation = self.target_rotation;
        }
    }
    
    /// Update orbit (Alt + Left mouse button held)
    pub fn update_orbit(&mut self, mouse_pos: Vec2) {
        if self.is_orbiting {
            let delta = mouse_pos - self.last_mouse_pos;
            
            // Use the stored distance field to maintain consistent distance
            let orbit_distance = self.distance;
            
            // Rotate around pivot
            let yaw_delta = delta.x * self.settings.rotation_sensitivity;
            let pitch_delta = -delta.y * self.settings.rotation_sensitivity;
            
            self.target_rotation += yaw_delta;
            self.target_rotation = self.target_rotation.rem_euclid(360.0);
            
            self.target_pitch += pitch_delta;
            self.target_pitch = self.target_pitch.clamp(self.min_pitch, self.max_pitch);
            
            // Update camera position to maintain the stored distance from pivot
            let yaw_rad = self.target_rotation.to_radians();
            let offset_x = orbit_distance * yaw_rad.cos();
            let offset_z = orbit_distance * yaw_rad.sin();
            self.target_position = self.pivot + Vec2::new(offset_x, offset_z);
            
            // Add to velocity for inertia
            if self.settings.enable_inertia {
                self.velocity.rotation_velocity += Vec2::new(yaw_delta, pitch_delta);
            }
            
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    /// Stop orbit
    pub fn stop_orbit(&mut self) {
        self.is_orbiting = false;
    }
    
    /// Focus on object (F key) - frames entity appropriately in viewport
    pub fn focus_on(&mut self, target_pos: Vec2, object_size: f32, viewport_size: Vec2) {
        self.pivot = target_pos;
        self.position = target_pos;
        
        // Calculate zoom to frame object with some padding (1.5x the object size)
        let target_screen_size = object_size * 1.5;
        let viewport_min = viewport_size.x.min(viewport_size.y);
        
        // Zoom should make the object take up a reasonable portion of the viewport
        self.target_zoom = viewport_min / target_screen_size;
        self.target_zoom = self.target_zoom.clamp(self.min_zoom, self.max_zoom);
        self.zoom = self.target_zoom;
        
        // Set appropriate distance based on object size for 3D mode
        self.distance = object_size * 3.0;
    }
    
    /// Frame selected object (F key) - alternative with explicit size
    pub fn frame_object(&mut self, object_pos: Vec2, object_size: Vec2, viewport_size: Vec2) {
        self.position = object_pos;
        
        // Calculate zoom to fit object in view with padding
        let zoom_x = viewport_size.x / (object_size.x * 1.5);
        let zoom_y = viewport_size.y / (object_size.y * 1.5);
        self.target_zoom = zoom_x.min(zoom_y).clamp(self.min_zoom, self.max_zoom);
        self.zoom = self.target_zoom;
    }
    
    /// Frame all objects in scene - calculates bounding box and frames it
    pub fn frame_all(&mut self, objects: &[(Vec2, Vec2)], viewport_size: Vec2) {
        if objects.is_empty() {
            // No objects, reset to default view
            self.reset();
            return;
        }
        
        // Calculate bounding box of all objects
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        
        for (pos, size) in objects {
            let half_size = *size * 0.5;
            min = min.min(*pos - half_size);
            max = max.max(*pos + half_size);
        }
        
        // Calculate center and size of bounding box
        let center = (min + max) * 0.5;
        let size = max - min;
        
        // Frame the bounding box
        self.frame_object(center, size, viewport_size);
    }
    
    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // In 2D mode (rotation = 0), Y axis points up (standard convention)
        // Screen Y increases downward, so we need to invert it
        self.position + Vec2::new(screen_pos.x, -screen_pos.y) / self.zoom
    }
    
    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // Convert world to screen, inverting Y axis
        let world_delta = world_pos - self.position;
        Vec2::new(world_delta.x, -world_delta.y) * self.zoom
    }
    
    /// Reset camera to default
    pub fn reset(&mut self) {
        self.position = Vec2::ZERO;
        self.zoom = 50.0;
        self.rotation = 45.0;
        self.pitch = 30.0;
        self.distance = 500.0;
        self.pivot = Vec2::ZERO;
        self.target_position = Vec2::ZERO;
        self.target_zoom = 50.0;
        self.target_rotation = 45.0;
        self.target_pitch = 30.0;
        self.velocity = CameraVelocity::default();
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
    
    /// Get view matrix for 3D rendering
    pub fn get_view_matrix(&self) -> Mat4 {
        let yaw_rad = self.rotation.to_radians();
        let pitch_rad = self.pitch.to_radians();
        
        // Calculate camera position in 3D space
        let cam_x = self.position.x + self.distance * yaw_rad.cos() * pitch_rad.cos();
        let cam_y = self.distance * pitch_rad.sin();
        let cam_z = self.position.y + self.distance * yaw_rad.sin() * pitch_rad.cos();
        
        let eye = Vec3::new(cam_x, cam_y, cam_z);
        let target = Vec3::new(self.position.x, 0.0, self.position.y);
        let up = Vec3::Y;
        
        Mat4::look_at_rh(eye, target, up)
    }
    
    /// Get projection matrix for 3D rendering
    pub fn get_projection_matrix(&self, aspect: f32, mode: ProjectionMode) -> Mat4 {
        match mode {
            ProjectionMode::Perspective => {
                let fov = 60.0_f32.to_radians();
                let near = 0.1;
                let far = 10000.0;
                Mat4::perspective_rh(fov, aspect, near, far)
            }
            ProjectionMode::Isometric => {
                let height = 1000.0 / self.zoom;
                let width = height * aspect;
                let near = -1000.0;
                let far = 1000.0;
                Mat4::orthographic_rh(
                    -width / 2.0,
                    width / 2.0,
                    -height / 2.0,
                    height / 2.0,
                    near,
                    far,
                )
            }
        }
    }
    
    /// Save current camera state (for mode switching)
    pub fn save_state(&self) -> CameraState {
        CameraState {
            position: self.position,
            zoom: self.zoom,
            rotation: self.rotation,
            pitch: self.pitch,
        }
    }
    
    /// Restore camera state (for mode switching)
    pub fn restore_state(&mut self, state: &CameraState) {
        self.position = state.position;
        self.zoom = state.zoom;
        self.target_zoom = state.zoom;
        self.rotation = state.rotation;
        self.pitch = state.pitch;
    }
    
    /// Switch to 2D mode, preserving position and zoom
    pub fn switch_to_2d(&mut self) {
        // Save current 3D state
        self.saved_3d_state = Some(self.save_state());
        
        // Reset rotation and pitch for 2D mode
        self.rotation = 0.0;
        self.pitch = 0.0;
        
        // Position and zoom are preserved
    }
    
    /// Switch to 3D mode, restoring previous 3D orientation or using default
    pub fn switch_to_3d(&mut self) {
        if let Some(saved_state) = &self.saved_3d_state {
            // Restore previous 3D orientation
            self.rotation = saved_state.rotation;
            self.pitch = saved_state.pitch;
            self.target_rotation = saved_state.rotation;
            self.target_pitch = saved_state.pitch;
            // Position and zoom are already preserved
        } else {
            // Initialize to default isometric view
            self.rotation = 45.0;
            self.pitch = 30.0;
            self.target_rotation = 45.0;
            self.target_pitch = 30.0;
        }
    }
    
    /// Load camera settings from JSON file
    pub fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = std::path::Path::new(".kiro/settings/camera_settings.json");
        if settings_path.exists() {
            let contents = std::fs::read_to_string(settings_path)?;
            self.settings = serde_json::from_str(&contents)?;
            // Update backward compatibility fields
            self.rotation_sensitivity = self.settings.rotation_sensitivity;
            self.zoom_sensitivity = self.settings.zoom_sensitivity;
        }
        Ok(())
    }
    
    /// Save camera settings to JSON file
    pub fn save_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_dir = std::path::Path::new(".kiro/settings");
        std::fs::create_dir_all(settings_dir)?;
        
        let settings_path = settings_dir.join("camera_settings.json");
        let contents = serde_json::to_string_pretty(&self.settings)?;
        std::fs::write(settings_path, contents)?;
        Ok(())
    }
    
    /// Reset settings to default Unity-like values
    pub fn reset_settings_to_default(&mut self) {
        self.settings = CameraSettings::default();
        self.rotation_sensitivity = self.settings.rotation_sensitivity;
        self.zoom_sensitivity = self.settings.zoom_sensitivity;
    }
    
    // ============================================================================
    // PRESET CAMERA VIEWS (Crown Engine inspired)
    // ============================================================================
    
    /// Set camera to front view (looking along +Z axis)
    pub fn set_view_front(&mut self) {
        self.target_rotation = 0.0;
        self.target_pitch = 0.0;
        self.rotation = 0.0;
        self.pitch = 0.0;
    }
    
    /// Set camera to back view (looking along -Z axis)
    pub fn set_view_back(&mut self) {
        self.target_rotation = 180.0;
        self.target_pitch = 0.0;
        self.rotation = 180.0;
        self.pitch = 0.0;
    }
    
    /// Set camera to right view (looking along +X axis)
    pub fn set_view_right(&mut self) {
        self.target_rotation = 90.0;
        self.target_pitch = 0.0;
        self.rotation = 90.0;
        self.pitch = 0.0;
    }
    
    /// Set camera to left view (looking along -X axis)
    pub fn set_view_left(&mut self) {
        self.target_rotation = -90.0;
        self.target_pitch = 0.0;
        self.rotation = -90.0;
        self.pitch = 0.0;
    }
    
    /// Set camera to top view (looking down along -Y axis)
    pub fn set_view_top(&mut self) {
        self.target_rotation = 0.0;
        self.target_pitch = 90.0;
        self.rotation = 0.0;
        self.pitch = 90.0;
    }
    
    /// Set camera to bottom view (looking up along +Y axis)
    pub fn set_view_bottom(&mut self) {
        self.target_rotation = 0.0;
        self.target_pitch = -90.0;
        self.rotation = 0.0;
        self.pitch = -90.0;
    }
    
    // ============================================================================
    // ZOOM CONTROL METHODS
    // ============================================================================
    
    /// Set zoom sensitivity (0.01 - 0.5)
    pub fn set_zoom_sensitivity(&mut self, sensitivity: f32) {
        self.settings.zoom_sensitivity = sensitivity.clamp(0.01, 0.5);
    }
    
    /// Get current zoom sensitivity
    pub fn get_zoom_sensitivity(&self) -> f32 {
        self.settings.zoom_sensitivity
    }
    
    /// Increase zoom sensitivity
    pub fn increase_zoom_sensitivity(&mut self, amount: f32) {
        self.settings.zoom_sensitivity = (self.settings.zoom_sensitivity + amount).clamp(0.01, 0.5);
    }
    
    /// Decrease zoom sensitivity
    pub fn decrease_zoom_sensitivity(&mut self, amount: f32) {
        self.settings.zoom_sensitivity = (self.settings.zoom_sensitivity - amount).clamp(0.01, 0.5);
    }
    
    /// Set zoom speed (1.0 - 50.0)
    pub fn set_zoom_speed(&mut self, speed: f32) {
        self.settings.zoom_speed = speed.clamp(1.0, 50.0);
    }
    
    /// Get current zoom level
    pub fn get_zoom_level(&self) -> f32 {
        self.zoom
    }
    
    /// Set zoom level directly
    pub fn set_zoom_level(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
        self.target_zoom = self.zoom;
    }
    
    /// Set camera to default perspective view (isometric-like)
    pub fn set_view_perspective(&mut self) {
        self.target_rotation = 45.0;
        self.target_pitch = 30.0;
        self.rotation = 45.0;
        self.pitch = 30.0;
    }
}

/// Projection mode for 3D rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionMode {
    Perspective,
    Isometric,
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self::new()
    }
}
