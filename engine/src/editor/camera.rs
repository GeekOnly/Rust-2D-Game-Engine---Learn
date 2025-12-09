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

impl CameraSettings {
    /// Minimum allowed sensitivity value
    pub const MIN_SENSITIVITY: f32 = 0.01;
    /// Maximum allowed sensitivity value
    pub const MAX_SENSITIVITY: f32 = 10.0;
    
    /// Validate and clamp sensitivity values to safe range
    pub fn validate(&mut self) {
        // Handle NaN/Inf by replacing with default values
        if !self.pan_sensitivity.is_finite() {
            self.pan_sensitivity = 0.5;
        }
        if !self.rotation_sensitivity.is_finite() {
            self.rotation_sensitivity = 0.5;
        }
        if !self.zoom_sensitivity.is_finite() {
            self.zoom_sensitivity = 0.01;
        }
        if !self.pan_damping.is_finite() {
            self.pan_damping = 0.08;
        }
        if !self.rotation_damping.is_finite() {
            self.rotation_damping = 0.12;
        }
        if !self.zoom_damping.is_finite() {
            self.zoom_damping = 0.08;
        }
        if !self.inertia_decay.is_finite() {
            self.inertia_decay = 0.92;
        }
        if !self.zoom_speed.is_finite() {
            self.zoom_speed = 20.0;
        }
        
        // Now clamp to valid ranges
        self.pan_sensitivity = self.pan_sensitivity.clamp(Self::MIN_SENSITIVITY, Self::MAX_SENSITIVITY);
        self.rotation_sensitivity = self.rotation_sensitivity.clamp(Self::MIN_SENSITIVITY, Self::MAX_SENSITIVITY);
        self.zoom_sensitivity = self.zoom_sensitivity.clamp(Self::MIN_SENSITIVITY, Self::MAX_SENSITIVITY);
        
        // Clamp damping values to [0.0, 1.0]
        self.pan_damping = self.pan_damping.clamp(0.0, 1.0);
        self.rotation_damping = self.rotation_damping.clamp(0.0, 1.0);
        self.zoom_damping = self.zoom_damping.clamp(0.0, 1.0);
        
        // Clamp inertia decay to [0.0, 1.0]
        self.inertia_decay = self.inertia_decay.clamp(0.0, 1.0);
        
        // Clamp zoom speed to reasonable range
        self.zoom_speed = self.zoom_speed.clamp(1.0, 100.0);
    }
    
    /// Check if all values are finite (not NaN or Inf)
    pub fn is_valid(&self) -> bool {
        self.pan_sensitivity.is_finite() &&
        self.rotation_sensitivity.is_finite() &&
        self.zoom_sensitivity.is_finite() &&
        self.pan_damping.is_finite() &&
        self.rotation_damping.is_finite() &&
        self.zoom_damping.is_finite() &&
        self.inertia_decay.is_finite() &&
        self.zoom_speed.is_finite()
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            // OPTIMIZED: Unity-like feel (improved for better control)
            pan_sensitivity: 1.0,   // Unity-like pan speed
            rotation_sensitivity: 0.15, // Slower for precise control
            zoom_sensitivity: 0.08, // Smoother, more gradual zoom
            pan_damping: 0.0,      // No damping for immediate response
            rotation_damping: 0.0, // No damping for immediate response
            zoom_damping: 0.0,     // No damping for immediate response
            enable_inertia: false,  // Disabled by default for more predictable behavior
            inertia_decay: 0.90,    // Slightly faster decay when enabled
            zoom_to_cursor: true,   // Zoom to cursor (better for precise editing)
            zoom_speed: 15.0,       // Smoother zoom speed
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

/// Projection mode for 3D view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneProjectionMode {
    Isometric,
    Perspective,
}

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,    // Horizontal rotation (yaw) in degrees
    pub pitch: f32,       // Vertical rotation in degrees
    pub distance: f32,    // Distance from pivot point (for orbit)
    pub pivot: Vec2,      // Pivot point for orbit mode
    pub projection_mode: SceneProjectionMode, // Current projection mode
    
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
        let initial_zoom = 2.0;  // 2x zoom for better visibility in editor
        let settings = CameraSettings::default();
        Self {
            position: Vec2::ZERO,
            zoom: initial_zoom,       // Editor zoom (2x for comfortable editing)
            rotation: 0.0,    // Start in 2D mode (0° rotation)
            pitch: 0.0,       // Start in 2D mode (0° pitch)
            distance: 500.0,  // Default distance for 3D mode
            pivot: Vec2::ZERO,
            projection_mode: SceneProjectionMode::Perspective,
            min_zoom: 0.01,   // Min zoom (1% - very zoomed out, see entire level)
            max_zoom: 100.0,  // Max zoom (100x - very zoomed in, pixel-level editing)
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
            target_rotation: 0.0,  // Start in 2D mode
            target_pitch: 0.0,     // Start in 2D mode
            target_zoom: initial_zoom,  // Match initial_zoom
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

            // Check if we're in 3D mode (pitch != 0)
            if self.pitch.abs() > 0.1 {
                // 3D mode: pan in camera space
                // Better pan speed calculation: based on distance and zoom-like feel
                let base_speed = 0.5; // Reduced base speed for better control
                let distance_factor = (self.distance / 100.0).max(0.1); // Clamp minimum
                let pan_speed = self.settings.pan_sensitivity * base_speed * distance_factor;

                let yaw_rad = self.rotation.to_radians();

                // Right vector (perpendicular to view direction on XZ plane)
                let right_x = yaw_rad.sin();
                let right_z = -yaw_rad.cos();

                // Forward vector (for up/down panning in 3D)
                // Use pitch to determine vertical movement
                let pitch_rad = self.pitch.to_radians();
                let forward_x = yaw_rad.cos() * pitch_rad.cos();
                let forward_z = yaw_rad.sin() * pitch_rad.cos();

                // Apply delta in camera space (Unity-style: drag right = camera moves right)
                let world_delta_x = (delta.x * right_x + delta.y * forward_x) * pan_speed;
                let world_delta_z = (delta.x * right_z + delta.y * forward_z) * pan_speed;

                let world_delta = Vec2::new(world_delta_x, world_delta_z);

                // Update both position and pivot
                self.position += world_delta;
                self.pivot += world_delta;
                self.target_position = self.position;
            } else {
                // 2D mode: simple pan
                let pan_speed = self.settings.pan_sensitivity / self.zoom;

                // Direct X/Y movement (inverted to match Unity: drag right = world moves left)
                let world_delta = Vec2::new(-delta.x, delta.y) * pan_speed;

                // Update position immediately for responsive panning
                self.position += world_delta;
                self.target_position = self.position;
            }

            // Add to velocity for inertia (if enabled)
            if self.settings.enable_inertia {
                let world_delta = self.position - self.target_position;
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
    /// For 3D mode, this adjusts the distance from the pivot point
    pub fn zoom(&mut self, delta: f32, mouse_pos: Vec2) {
        // Validate inputs
        if !delta.is_finite() || !mouse_pos.is_finite() {
            return;
        }

        // In 3D Perspective mode (when pitch != 0), adjust distance (Dolly).
        // In 3D Isometric or 2D mode, adjust zoom (Scale).
        if self.pitch.abs() > 0.1 && self.projection_mode == SceneProjectionMode::Perspective {
            // 3D Perspective: adjust distance (smoother scaling)
            let zoom_factor = if delta > 0.0 {
                0.92  // Zoom in = decrease distance by 8% (smoother)
            } else {
                1.08  // Zoom out = increase distance by 8% (smoother)
            };

            let new_distance = self.distance * zoom_factor;
            self.distance = new_distance.clamp(0.5, 10000.0);  // Allow closer zoom

            // In 3D mode, we simply move closer/further from the target (self.position)
            // relative to the current viewing angle.
            // We do NOT update self.position here, as that would shift the look-at target.

            self.target_zoom = self.zoom; // Sync target for 2D if we switch back
            return;
        }
        
        // 2D mode: use zoom
        // Check for extreme zoom levels - graceful degradation
        if self.zoom <= self.min_zoom * 1.01 && delta < 0.0 {
            // Already at minimum zoom, don't zoom out further
            return;
        }
        if self.zoom >= self.max_zoom * 0.99 && delta > 0.0 {
            // Already at maximum zoom, don't zoom in further
            return;
        }
        
        // Calculate world position under cursor BEFORE zoom
        let world_pos_before = self.screen_to_world(mouse_pos);
        
        // Validate world position
        if !world_pos_before.is_finite() {
            return;
        }
        
        // Store for smooth interpolation if needed
        self.last_cursor_world_pos = Some(world_pos_before);
        
        // Calculate zoom factor
        let zoom_factor = if delta > 0.0 {
            1.0 + self.settings.zoom_sensitivity
        } else {
            1.0 / (1.0 + self.settings.zoom_sensitivity)
        };
        
        // Validate zoom factor
        if !zoom_factor.is_finite() || zoom_factor <= 0.0 {
            return;
        }
        
        let old_zoom = self.zoom;
        
        // Apply zoom immediately for responsive feel
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
        self.target_zoom = self.zoom;
        
        // Adjust camera position to zoom towards cursor (if enabled)
        if self.settings.zoom_to_cursor {
            // Calculate world position under cursor AFTER zoom
            let world_pos_after = self.screen_to_world(mouse_pos);
            
            // Validate world position after zoom
            if !world_pos_after.is_finite() {
                return;
            }
            
            // Adjust camera position to keep the same world point under cursor
            let world_offset = world_pos_before - world_pos_after;
            
            // Validate offset before applying
            if world_offset.is_finite() {
                self.position += world_offset;
                self.target_position = self.position;
            }
        }
        
        // Add to velocity for inertia (if enabled)
        if self.settings.enable_inertia {
            let zoom_delta = self.zoom - old_zoom;
            if zoom_delta.is_finite() {
                self.velocity.zoom_velocity += zoom_delta * 0.2;
            }
        }
    }
    
    /// Enhanced zoom with explicit viewport center for better control
    pub fn zoom_to_cursor(&mut self, delta: f32, cursor_screen_pos: Vec2, viewport_center: Vec2) {
        // Validate inputs
        if !delta.is_finite() || !cursor_screen_pos.is_finite() || !viewport_center.is_finite() {
            return;
        }
        
        // Check for extreme zoom levels - graceful degradation
        if self.zoom <= self.min_zoom * 1.01 && delta < 0.0 {
            // Already at minimum zoom, don't zoom out further
            return;
        }
        if self.zoom >= self.max_zoom * 0.99 && delta > 0.0 {
            // Already at maximum zoom, don't zoom in further
            return;
        }
        
        // Convert cursor position to screen space relative to viewport center
        let screen_pos = cursor_screen_pos - viewport_center;
        
        // Validate screen position
        if !screen_pos.is_finite() {
            return;
        }
        
        // Calculate world position under cursor BEFORE zoom
        let world_pos_before = self.screen_to_world(screen_pos);
        
        // Validate world position
        if !world_pos_before.is_finite() {
            return;
        }
        
        // Store for tracking
        self.last_cursor_world_pos = Some(world_pos_before);
        
        // Calculate zoom factor
        let zoom_factor = if delta > 0.0 {
            1.0 + self.settings.zoom_sensitivity
        } else {
            1.0 / (1.0 + self.settings.zoom_sensitivity)
        };
        
        // Validate zoom factor
        if !zoom_factor.is_finite() || zoom_factor <= 0.0 {
            return;
        }
        
        let old_zoom = self.zoom;
        
        // Apply zoom transformation
        self.zoom *= zoom_factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
        self.target_zoom = self.zoom;
        
        // Adjust camera position to keep cursor point stationary
        if self.settings.zoom_to_cursor {
            // Calculate world position under cursor AFTER zoom
            let world_pos_after = self.screen_to_world(screen_pos);
            
            // Validate world position after zoom
            if !world_pos_after.is_finite() {
                return;
            }
            
            // Calculate offset needed to keep world point stationary
            let world_offset = world_pos_before - world_pos_after;
            
            // Validate and apply offset
            if world_offset.is_finite() {
                self.position += world_offset;
                self.target_position = self.position;
            }
        }
        
        // Add smooth zoom interpolation velocity
        if self.settings.enable_inertia {
            let zoom_delta = self.zoom - old_zoom;
            if zoom_delta.is_finite() {
                self.velocity.zoom_velocity += zoom_delta * 0.2;
            }
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
        // Validate delta_time
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return;
        }
        
        // Exponential damping for smooth deceleration
        let pan_damping_factor = 1.0 - (-self.settings.pan_damping * 10.0 * delta_time).exp();
        let rotation_damping_factor = 1.0 - (-self.settings.rotation_damping * 10.0 * delta_time).exp();
        let zoom_damping_factor = 1.0 - (-self.settings.zoom_damping * 10.0 * delta_time).exp();
        
        // Validate damping factors
        if !pan_damping_factor.is_finite() || !rotation_damping_factor.is_finite() || !zoom_damping_factor.is_finite() {
            return;
        }
        
        // Apply damping to position
        if (self.position - self.target_position).length() > 0.001 {
            let new_position = self.position + (self.target_position - self.position) * pan_damping_factor;
            if new_position.is_finite() {
                self.position = new_position;
            }
        } else {
            self.position = self.target_position;
        }
        
        // Apply damping to rotation
        if (self.rotation - self.target_rotation).abs() > 0.01 {
            let new_rotation = self.rotation + (self.target_rotation - self.rotation) * rotation_damping_factor;
            if new_rotation.is_finite() {
                self.rotation = new_rotation;
            }
        } else {
            self.rotation = self.target_rotation;
        }
        
        // Apply damping to pitch
        if (self.pitch - self.target_pitch).abs() > 0.01 {
            let new_pitch = self.pitch + (self.target_pitch - self.pitch) * rotation_damping_factor;
            if new_pitch.is_finite() {
                self.pitch = new_pitch;
            }
        } else {
            self.pitch = self.target_pitch;
        }
        
        // Apply damping to zoom
        if (self.zoom - self.target_zoom).abs() > 0.01 {
            let new_zoom = self.zoom + (self.target_zoom - self.zoom) * zoom_damping_factor;
            if new_zoom.is_finite() && new_zoom > 0.0 {
                self.zoom = new_zoom;
            }
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
        
        // Validate delta_time
        if !delta_time.is_finite() || delta_time <= 0.0 {
            return;
        }
        
        // Apply pan velocity
        if self.velocity.pan_velocity.length() > 0.001 {
            let new_position = self.target_position + self.velocity.pan_velocity * delta_time * 60.0;
            if new_position.is_finite() {
                self.target_position = new_position;
            }
            // Decay velocity exponentially
            let new_velocity = self.velocity.pan_velocity * self.settings.inertia_decay;
            if new_velocity.is_finite() {
                self.velocity.pan_velocity = new_velocity;
            } else {
                self.velocity.pan_velocity = Vec2::ZERO;
            }
        } else {
            self.velocity.pan_velocity = Vec2::ZERO;
        }
        
        // Apply rotation velocity
        if self.velocity.rotation_velocity.length() > 0.001 {
            let new_rotation = self.target_rotation + self.velocity.rotation_velocity.x * delta_time * 60.0;
            let new_pitch = self.target_pitch + self.velocity.rotation_velocity.y * delta_time * 60.0;
            
            if new_rotation.is_finite() {
                self.target_rotation = new_rotation;
            }
            if new_pitch.is_finite() {
                self.target_pitch = new_pitch.clamp(self.min_pitch, self.max_pitch);
            }
            
            // Decay velocity exponentially
            let new_velocity = self.velocity.rotation_velocity * self.settings.inertia_decay;
            if new_velocity.is_finite() {
                self.velocity.rotation_velocity = new_velocity;
            } else {
                self.velocity.rotation_velocity = Vec2::ZERO;
            }
        } else {
            self.velocity.rotation_velocity = Vec2::ZERO;
        }
        
        // Apply zoom velocity
        if self.velocity.zoom_velocity.abs() > 0.001 {
            let new_zoom = self.target_zoom + self.velocity.zoom_velocity * delta_time * 60.0;
            if new_zoom.is_finite() && new_zoom > 0.0 {
                self.target_zoom = new_zoom.clamp(self.min_zoom, self.max_zoom);
            }
            // Decay velocity exponentially
            let new_velocity = self.velocity.zoom_velocity * self.settings.inertia_decay;
            if new_velocity.is_finite() {
                self.velocity.zoom_velocity = new_velocity;
            } else {
                self.velocity.zoom_velocity = 0.0;
            }
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
            
            // Update rotation and pitch immediately from current values
            self.rotation += yaw_delta;
            self.rotation = self.rotation.rem_euclid(360.0);
            
            // Vertical movement changes pitch
            self.pitch += pitch_delta;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            
            // Also update targets to match
            self.target_rotation = self.rotation;
            self.target_pitch = self.pitch;
            
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
        
        // Calculate current distance from pivot
        let offset = self.position - self.pivot;
        let horizontal_distance = offset.length();
        
        // Store distance (considering pitch for 3D orbiting)
        if self.pitch.abs() > 0.1 {
            // In 3D mode, calculate actual 3D distance
            let pitch_rad = self.pitch.to_radians();
            self.distance = horizontal_distance / pitch_rad.cos().max(0.01);
        } else {
            // In 2D mode, use horizontal distance
            self.distance = horizontal_distance.max(10.0);  // Minimum distance
        }
        
        // Ensure target values match current values to avoid damping drift
        self.target_position = self.position;
        self.target_rotation = self.rotation;
        self.target_pitch = self.pitch;
    }
    
    /// Update orbit (Alt + Left mouse button held)
    pub fn update_orbit(&mut self, mouse_pos: Vec2) {
        if self.is_orbiting {
            let delta = mouse_pos - self.last_mouse_pos;
            
            // Rotate around pivot
            let yaw_delta = delta.x * self.settings.rotation_sensitivity;
            let pitch_delta = -delta.y * self.settings.rotation_sensitivity;
            
            // Update rotation angles
            self.rotation += yaw_delta;
            self.rotation = self.rotation.rem_euclid(360.0);
            self.target_rotation = self.rotation;
            
            self.pitch += pitch_delta;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            self.target_pitch = self.pitch;
            
            // Calculate new camera position maintaining constant distance from pivot
            let yaw_rad = self.rotation.to_radians();
            let pitch_rad = self.pitch.to_radians();
            
            // Calculate position in spherical coordinates
            let horizontal_distance = self.distance * pitch_rad.cos();
            let offset_x = horizontal_distance * yaw_rad.cos();
            let offset_z = horizontal_distance * yaw_rad.sin();
            
            let new_position = self.pivot + Vec2::new(offset_x, offset_z);
            
            // Validate and update position
            if new_position.is_finite() {
                self.position = new_position;
                self.target_position = new_position;
            }
            
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
        // Set pivot to target position
        self.pivot = target_pos;
        
        // Check if we're in 3D mode (pitch != 0)
        if self.pitch.abs() > 0.1 {
            // 3D mode: maintain current rotation/pitch, adjust distance
            // Calculate appropriate distance to frame the object
            let fov = 60.0_f32.to_radians();
            let viewport_min = viewport_size.x.min(viewport_size.y);
            
            // Calculate distance needed to frame object with padding
            let padding_factor = 2.5; // Show object with some padding
            let target_screen_size = object_size * padding_factor;
            
            // Use FOV to calculate distance
            let half_fov = fov / 2.0;
            self.distance = (target_screen_size / 2.0) / half_fov.tan();
            self.distance = self.distance.clamp(10.0, 10000.0);
            
            // Update camera position based on new distance and current rotation/pitch
            let yaw_rad = self.rotation.to_radians();
            let pitch_rad = self.pitch.to_radians();
            
            let horizontal_distance = self.distance * pitch_rad.cos();
            let offset_x = horizontal_distance * yaw_rad.cos();
            let offset_z = horizontal_distance * yaw_rad.sin();
            
            self.position = self.pivot + Vec2::new(offset_x, offset_z);
            self.target_position = self.position;
        } else {
            // 2D mode: adjust position and zoom
            self.position = target_pos;
            self.target_position = target_pos;
            
            // Calculate zoom to frame object with padding
            let padding_factor = 1.5;
            let target_screen_size = object_size * padding_factor;
            let viewport_min = viewport_size.x.min(viewport_size.y);
            
            // Zoom calculation: we want object_size * zoom * padding = viewport_min
            // So: zoom = viewport_min / (object_size * padding)
            if target_screen_size > 0.0 {
                self.zoom = viewport_min / target_screen_size;
                self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
                self.target_zoom = self.zoom;
            }
        }
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
    /// In Unity-like system: screen is in pixels, world is in units (1 unit = 100 pixels by default)
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // Unity-style: 2D camera is orthographic looking down -Z axis
        // X: right, Y: up, Z: forward (out of screen)
        // Screen Y increases downward, so we need to invert it
        // Zoom affects the orthographic size (how many world units fit in view)
        self.position + Vec2::new(screen_pos.x, -screen_pos.y) / self.zoom
    }

    /// Convert world coordinates to screen coordinates
    /// In Unity-like system: world is in units, screen is in pixels
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // Unity-style: convert world units to screen pixels
        // World coordinates: X = right, Y = up
        // Screen coordinates: X = right, Y = down
        let world_delta = world_pos - self.position;
        Vec2::new(world_delta.x, -world_delta.y) * self.zoom
    }

    /// Get orthographic projection matrix for 2D rendering (Unity-style)
    /// This allows 2D and 3D to share the same world space
    pub fn get_orthographic_matrix_2d(&self, viewport_size: Vec2) -> Mat4 {
        // Calculate orthographic bounds based on zoom
        // zoom = pixels per world unit (higher zoom = more zoomed in)
        // viewport_size = size in pixels
        let half_width = (viewport_size.x / 2.0) / self.zoom;
        let half_height = (viewport_size.y / 2.0) / self.zoom;

        // Unity-style 2D: looking down -Z axis, X = right, Y = up
        Mat4::orthographic_rh(
            self.position.x - half_width,   // left
            self.position.x + half_width,   // right
            self.position.y - half_height,  // bottom
            self.position.y + half_height,  // top
            -1000.0,                        // near (see objects from Z=-1000 to Z=1000)
            1000.0,                         // far
        )
    }
    
    /// Reset camera to default
    pub fn reset(&mut self) {
        self.position = Vec2::ZERO;
        self.zoom = 2.0;  // 2x zoom for editor
        self.rotation = 0.0;  // Reset to 2D mode
        self.pitch = 0.0;     // Reset to 2D mode
        self.distance = 500.0;
        self.pivot = Vec2::ZERO;
        self.target_position = Vec2::ZERO;
        self.target_zoom = 2.0;  // 2x zoom for editor
        self.target_rotation = 0.0;  // Reset to 2D mode
        self.target_pitch = 0.0;     // Reset to 2D mode
        self.velocity = CameraVelocity::default();
        self.is_panning = false;
        self.is_rotating = false;
        self.is_orbiting = false;
        self.saved_3d_state = None;  // Clear saved 3D state
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
    pub fn get_projection_matrix(&self, aspect: f32, mode: SceneProjectionMode) -> Mat4 {
        match mode {
            SceneProjectionMode::Perspective => {
                let fov = 60.0_f32.to_radians();
                let near = 0.1;
                let far = 10000.0;
                Mat4::perspective_rh(fov, aspect, near, far)
            }
            SceneProjectionMode::Isometric => {
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
            
            // Set pivot to current position (where we're looking at)
            self.pivot = self.position;
            
            // Use a reasonable distance based on zoom level
            // In 2D, zoom represents how much we see
            // In 3D, distance should give similar view
            self.distance = 20.0 / self.zoom;  // Closer view for better editing
            
            // Calculate camera position based on distance and angles
            let yaw_rad = self.rotation.to_radians();
            let pitch_rad = self.pitch.to_radians();
            
            let horizontal_distance = self.distance * pitch_rad.cos();
            let offset_x = horizontal_distance * yaw_rad.cos();
            let offset_z = horizontal_distance * yaw_rad.sin();
            
            self.position = self.pivot + Vec2::new(offset_x, offset_z);
            self.target_position = self.position;
        }
    }
    
    /// Load camera settings from JSON file
    pub fn load_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = std::path::Path::new(".kiro/settings/camera_settings.json");
        if settings_path.exists() {
            let contents = std::fs::read_to_string(settings_path)?;
            let mut loaded_settings: CameraSettings = serde_json::from_str(&contents)?;
            
            // Validate loaded settings
            if !loaded_settings.is_valid() {
                // If settings contain NaN/Inf, use defaults
                loaded_settings = CameraSettings::default();
            } else {
                // Clamp values to safe ranges
                loaded_settings.validate();
            }
            
            self.settings = loaded_settings;
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



impl Default for SceneCamera {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CAMERA STATE DISPLAY
// ============================================================================

/// Camera state display for visual feedback
#[derive(Debug, Clone)]
pub struct CameraStateDisplay {
    pub show_distance: bool,
    pub show_angles: bool,
    pub show_grid_size: bool,
    pub show_fps: bool,
}

impl CameraStateDisplay {
    pub fn new() -> Self {
        Self {
            show_distance: true,
            show_angles: true,
            show_grid_size: true,
            show_fps: true,
        }
    }
    
    /// Calculate camera distance from origin
    pub fn calculate_distance(&self, camera: &SceneCamera) -> f32 {
        camera.position.length()
    }
    
    /// Format camera rotation angles for display
    pub fn format_angles(&self, camera: &SceneCamera) -> String {
        format!("Yaw: {:.1}° Pitch: {:.1}°", camera.rotation, camera.pitch)
    }
    
    /// Format grid size for display
    pub fn format_grid_size(&self, grid_size: f32) -> String {
        if grid_size >= 1.0 {
            format!("Grid: {:.1}m", grid_size)
        } else if grid_size >= 0.01 {
            format!("Grid: {:.2}m", grid_size)
        } else {
            format!("Grid: {:.3}m", grid_size)
        }
    }
    
    /// Render camera state display
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        camera: &SceneCamera,
        grid_size: f32,
        fps: f32,
    ) {
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 2.0);
            
            if self.show_distance {
                let distance = self.calculate_distance(camera);
                ui.label(format!("Distance: {:.1}m", distance));
            }
            
            if self.show_angles {
                ui.label(self.format_angles(camera));
            }
            
            if self.show_grid_size {
                ui.label(self.format_grid_size(grid_size));
            }
            
            if self.show_fps {
                ui.label(format!("FPS: {:.0}", fps));
            }
        });
    }
}

impl Default for CameraStateDisplay {
    fn default() -> Self {
        Self::new()
    }
}
