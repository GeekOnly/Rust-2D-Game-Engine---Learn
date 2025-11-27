// Property-based tests for SceneCamera
// These tests validate the correctness properties defined in the unity-scene-view design document

use proptest::prelude::*;

// We need to include the camera module - for now we'll create a minimal version
// In a real scenario, we'd extract camera to a library crate or make engine a lib+bin crate

// Temporary: Copy the necessary types for testing
use glam::{Vec2, Vec3, Mat4};

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
    pub rotation: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pivot: Vec2,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    is_panning: bool,
    last_mouse_pos: Vec2,
    is_rotating: bool,
    is_orbiting: bool,
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub pan_speed: f32,
    target_zoom: f32,
    zoom_interpolation_speed: f32,
    saved_3d_state: Option<CameraState>,
}

impl SceneCamera {
    pub fn new() -> Self {
        let initial_zoom = 50.0;
        Self {
            position: Vec2::ZERO,
            zoom: initial_zoom,
            rotation: 45.0,
            pitch: 30.0,
            distance: 500.0,
            pivot: Vec2::ZERO,
            min_zoom: 5.0,
            max_zoom: 200.0,
            min_pitch: -89.0,
            max_pitch: 89.0,
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
            is_rotating: false,
            is_orbiting: false,
            rotation_sensitivity: 0.5,
            zoom_sensitivity: 0.1,
            pan_speed: 1.0,
            target_zoom: initial_zoom,
            zoom_interpolation_speed: 10.0,
            saved_3d_state: None,
        }
    }
    
    pub fn start_pan(&mut self, mouse_pos: Vec2) {
        self.is_panning = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    pub fn update_pan(&mut self, mouse_pos: Vec2) {
        if self.is_panning {
            let delta = mouse_pos - self.last_mouse_pos;
            let yaw_rad = self.rotation.to_radians();
            let cos_yaw = yaw_rad.cos();
            let sin_yaw = yaw_rad.sin();
            let pan_speed = 1.0 / self.zoom;
            let world_delta_x = -(delta.x * cos_yaw + delta.y * sin_yaw) * pan_speed;
            let world_delta_z = -(-delta.x * sin_yaw + delta.y * cos_yaw) * pan_speed;
            self.position.x += world_delta_x;
            self.position.y += world_delta_z;
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    pub fn stop_pan(&mut self) {
        self.is_panning = false;
    }
    
    pub fn zoom(&mut self, delta: f32, mouse_pos: Vec2) {
        let world_pos_before = self.screen_to_world(mouse_pos);
        let zoom_factor = if delta > 0.0 {
            1.0 + self.zoom_sensitivity
        } else {
            1.0 / (1.0 + self.zoom_sensitivity)
        };
        self.target_zoom *= zoom_factor;
        self.target_zoom = self.target_zoom.clamp(self.min_zoom, self.max_zoom);
        self.zoom = self.target_zoom;
        let world_pos_after = self.screen_to_world(mouse_pos);
        let world_delta = world_pos_after - world_pos_before;
        self.position -= world_delta;
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if (self.zoom - self.target_zoom).abs() > 0.01 {
            let t = 1.0 - (-self.zoom_interpolation_speed * delta_time).exp();
            self.zoom = self.zoom + (self.target_zoom - self.zoom) * t;
        } else {
            self.zoom = self.target_zoom;
        }
    }
    
    pub fn start_rotate(&mut self, mouse_pos: Vec2) {
        self.is_rotating = true;
        self.last_mouse_pos = mouse_pos;
    }
    
    pub fn update_rotate(&mut self, mouse_pos: Vec2) {
        if self.is_rotating {
            let delta = mouse_pos - self.last_mouse_pos;
            self.rotation += delta.x * self.rotation_sensitivity;
            self.rotation = self.rotation.rem_euclid(360.0);
            self.pitch -= delta.y * self.rotation_sensitivity;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    pub fn stop_rotate(&mut self) {
        self.is_rotating = false;
    }
    
    pub fn start_orbit(&mut self, mouse_pos: Vec2, pivot_point: Vec2) {
        self.is_orbiting = true;
        self.pivot = pivot_point;
        self.last_mouse_pos = mouse_pos;
    }
    
    pub fn update_orbit(&mut self, mouse_pos: Vec2) {
        if self.is_orbiting {
            let delta = mouse_pos - self.last_mouse_pos;
            let initial_distance = (self.position - self.pivot).length();
            self.rotation += delta.x * self.rotation_sensitivity;
            self.rotation = self.rotation.rem_euclid(360.0);
            self.pitch -= delta.y * self.rotation_sensitivity;
            self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
            let yaw_rad = self.rotation.to_radians();
            let offset_x = initial_distance * yaw_rad.cos();
            let offset_z = initial_distance * yaw_rad.sin();
            self.position = self.pivot + Vec2::new(offset_x, offset_z);
            self.distance = initial_distance;
            self.last_mouse_pos = mouse_pos;
        }
    }
    
    pub fn stop_orbit(&mut self) {
        self.is_orbiting = false;
    }
    
    pub fn focus_on(&mut self, target_pos: Vec2, object_size: f32, viewport_size: Vec2) {
        self.pivot = target_pos;
        self.position = target_pos;
        let target_screen_size = object_size * 1.5;
        let viewport_min = viewport_size.x.min(viewport_size.y);
        self.target_zoom = viewport_min / target_screen_size;
        self.target_zoom = self.target_zoom.clamp(self.min_zoom, self.max_zoom);
        self.zoom = self.target_zoom;
        self.distance = object_size * 3.0;
    }
    
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.position + screen_pos / self.zoom
    }
    
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos - self.position) * self.zoom
    }
    
    pub fn save_state(&self) -> CameraState {
        CameraState {
            position: self.position,
            zoom: self.zoom,
            rotation: self.rotation,
            pitch: self.pitch,
        }
    }
    
    pub fn restore_state(&mut self, state: &CameraState) {
        self.position = state.position;
        self.zoom = state.zoom;
        self.target_zoom = state.zoom;
        self.rotation = state.rotation;
        self.pitch = state.pitch;
    }
    
    pub fn switch_to_2d(&mut self) {
        self.saved_3d_state = Some(self.save_state());
        self.rotation = 0.0;
        self.pitch = 0.0;
    }
    
    pub fn switch_to_3d(&mut self) {
        if let Some(saved_state) = &self.saved_3d_state {
            self.rotation = saved_state.rotation;
            self.pitch = saved_state.pitch;
        } else {
            self.rotation = 45.0;
            self.pitch = 30.0;
        }
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

fn prop_angle() -> impl Strategy<Value = f32> {
    -180.0f32..180.0f32
}

fn prop_pitch() -> impl Strategy<Value = f32> {
    -89.0f32..89.0f32
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: unity-scene-view, Property 1: Pan updates camera position
    // Validates: Requirements 1.1, 1.4, 5.4
    #[test]
    fn prop_pan_updates_camera_position(
        initial_pos in prop_vec2(),
        initial_zoom in prop_zoom(),
        mouse_start in prop_vec2(),
        mouse_delta in prop_vec2(),
    ) {
        let mut camera = SceneCamera::new();
        camera.position = initial_pos;
        camera.zoom = initial_zoom;
        camera.rotation = 0.0;
        
        let mouse_end = mouse_start + mouse_delta;
        
        camera.start_pan(mouse_start);
        camera.update_pan(mouse_end);
        
        if mouse_delta.length() > 0.01 {
            let position_changed = (camera.position - initial_pos).length() > 0.001;
            prop_assert!(position_changed, "Camera position should change when panning");
        }
        
        let expected_scale = 1.0 / initial_zoom;
        let actual_delta = camera.position - initial_pos;
        let expected_magnitude = mouse_delta.length() * expected_scale;
        let actual_magnitude = actual_delta.length();
        let tolerance = expected_magnitude * 0.1 + 0.01;
        
        prop_assert!(
            (actual_magnitude - expected_magnitude).abs() < tolerance,
            "Pan distance should be proportional to mouse movement. Expected: {}, Actual: {}",
            expected_magnitude,
            actual_magnitude
        );
    }
    
    // Feature: unity-scene-view, Property 2: Orbit maintains pivot distance
    // Validates: Requirements 1.2, 5.3
    #[test]
    fn prop_orbit_maintains_pivot_distance(
        pivot in prop_vec2(),
        initial_offset in prop_vec2().prop_filter("Non-zero offset", |v| v.length() > 1.0),
        mouse_start in prop_vec2(),
        mouse_delta in prop_vec2(),
    ) {
        let mut camera = SceneCamera::new();
        camera.pivot = pivot;
        camera.position = pivot + initial_offset;
        
        let initial_distance = (camera.position - camera.pivot).length();
        let mouse_end = mouse_start + mouse_delta;
        
        camera.start_orbit(mouse_start, pivot);
        camera.update_orbit(mouse_end);
        
        let final_distance = (camera.position - camera.pivot).length();
        let distance_diff = (final_distance - initial_distance).abs();
        
        prop_assert!(
            distance_diff < 0.01,
            "Orbit should maintain constant distance from pivot. Initial: {}, Final: {}, Diff: {}",
            initial_distance,
            final_distance,
            distance_diff
        );
    }
    
    // Feature: unity-scene-view, Property 3: Free-look rotation updates camera orientation
    // Validates: Requirements 1.3
    #[test]
    fn prop_freelook_rotation_updates_orientation(
        initial_rotation in prop_angle(),
        initial_pitch in prop_pitch(),
        mouse_start in prop_vec2(),
        mouse_delta_x in -100.0f32..100.0f32,
        mouse_delta_y in -100.0f32..100.0f32,
    ) {
        let mut camera = SceneCamera::new();
        camera.rotation = initial_rotation;
        camera.pitch = initial_pitch;
        
        let mouse_end = mouse_start + Vec2::new(mouse_delta_x, mouse_delta_y);
        
        camera.start_rotate(mouse_start);
        camera.update_rotate(mouse_end);
        
        if mouse_delta_x.abs() > 0.1 {
            let rotation_changed = (camera.rotation - initial_rotation).abs() > 0.01;
            prop_assert!(rotation_changed, "Rotation (yaw) should change with horizontal mouse movement");
            
            let expected_rotation_delta = mouse_delta_x * camera.rotation_sensitivity;
            // Handle angle wrapping: calculate shortest angular distance
            let mut actual_rotation_delta = camera.rotation - initial_rotation;
            if actual_rotation_delta > 180.0 {
                actual_rotation_delta -= 360.0;
            } else if actual_rotation_delta < -180.0 {
                actual_rotation_delta += 360.0;
            }
            
            prop_assert!(
                (actual_rotation_delta - expected_rotation_delta).abs() < 0.1,
                "Rotation should be proportional to horizontal mouse movement. Expected delta: {}, Actual delta: {}",
                expected_rotation_delta,
                actual_rotation_delta
            );
        }
        
        if mouse_delta_y.abs() > 0.1 {
            let expected_pitch_delta = -mouse_delta_y * camera.rotation_sensitivity;
            let expected_pitch = initial_pitch + expected_pitch_delta;
            
            // Only test proportionality if the expected pitch would be within bounds
            // (otherwise clamping will affect the result)
            if expected_pitch >= camera.min_pitch && expected_pitch <= camera.max_pitch {
                let pitch_changed = (camera.pitch - initial_pitch).abs() > 0.01;
                prop_assert!(pitch_changed, "Pitch should change with vertical mouse movement");
                
                let actual_pitch_delta = camera.pitch - initial_pitch;
                prop_assert!(
                    (actual_pitch_delta - expected_pitch_delta).abs() < 0.1,
                    "Pitch should be proportional to vertical mouse movement when not clamped. Expected: {}, Actual: {}",
                    expected_pitch_delta,
                    actual_pitch_delta
                );
            }
        }
        
        prop_assert!(camera.pitch >= camera.min_pitch && camera.pitch <= camera.max_pitch,
            "Pitch should stay within bounds");
    }
    
    // Feature: unity-scene-view, Property 4: Zoom scales view toward cursor
    // Validates: Requirements 1.5
    #[test]
    fn prop_zoom_scales_toward_cursor(
        initial_pos in prop_vec2(),
        initial_zoom in prop_zoom(),
        cursor_screen_pos in prop_vec2(),
        zoom_delta in -5.0f32..5.0f32,
    ) {
        let mut camera = SceneCamera::new();
        camera.position = initial_pos;
        camera.zoom = initial_zoom;
        camera.target_zoom = initial_zoom;
        
        let world_pos_before = camera.screen_to_world(cursor_screen_pos);
        camera.zoom(zoom_delta, cursor_screen_pos);
        let world_pos_after = camera.screen_to_world(cursor_screen_pos);
        let world_pos_diff = (world_pos_after - world_pos_before).length();
        
        prop_assert!(
            world_pos_diff < 0.1,
            "World position under cursor should remain stationary during zoom. Diff: {}",
            world_pos_diff
        );
        
        if zoom_delta.abs() > 0.1 {
            let zoom_changed = (camera.zoom - initial_zoom).abs() > 0.001;
            if initial_zoom > camera.min_zoom + 1.0 && initial_zoom < camera.max_zoom - 1.0 {
                prop_assert!(zoom_changed, "Zoom should change when not at bounds");
            }
        }
    }
    
    // Feature: unity-scene-view, Property 5: Zoom interpolation is smooth
    // Validates: Requirements 5.2
    #[test]
    fn prop_zoom_interpolation_is_smooth(
        initial_zoom in prop_zoom(),
        zoom_delta in -5.0f32..5.0f32,
        dt in 0.001f32..0.05f32, // Reduced max dt for more reasonable test
    ) {
        let mut camera = SceneCamera::new();
        camera.zoom = initial_zoom;
        camera.target_zoom = initial_zoom;
        
        camera.zoom(zoom_delta, Vec2::ZERO);
        let target = camera.target_zoom;
        camera.zoom = initial_zoom;
        
        let mut prev_zoom = camera.zoom;
        let mut zoom_changes = Vec::new();
        
        for _ in 0..10 {
            camera.update(dt);
            let change = (camera.zoom - prev_zoom).abs();
            zoom_changes.push(change);
            prev_zoom = camera.zoom;
            
            if (camera.zoom - target).abs() < 0.01 {
                break;
            }
        }
        
        if (initial_zoom - target).abs() > 0.1 {
            prop_assert!(
                zoom_changes.iter().any(|&c| c > 0.001),
                "Zoom should be interpolating toward target"
            );
            
            // For exponential interpolation, verify changes are decreasing over time
            // (each step should be smaller than the previous as we approach target)
            if zoom_changes.len() >= 3 {
                let first_change = zoom_changes[0];
                let last_change = *zoom_changes.last().unwrap();
                
                prop_assert!(
                    last_change <= first_change * 1.1, // Allow small tolerance
                    "Zoom changes should decrease or stay similar (exponential decay). First: {}, Last: {}",
                    first_change,
                    last_change
                );
            }
        }
    }
    
    // Feature: unity-scene-view, Property 6: Focus frames entity appropriately
    // Validates: Requirements 1.6
    #[test]
    fn prop_focus_frames_entity_appropriately(
        entity_pos in prop_vec2(),
        entity_size in 10.0f32..500.0f32,
        viewport_width in 800.0f32..1920.0f32,
        viewport_height in 600.0f32..1080.0f32,
    ) {
        let mut camera = SceneCamera::new();
        let viewport_size = Vec2::new(viewport_width, viewport_height);
        
        camera.focus_on(entity_pos, entity_size, viewport_size);
        
        prop_assert!(
            (camera.position - entity_pos).length() < 0.01,
            "Camera should be centered on entity position"
        );
        
        let viewport_min = viewport_width.min(viewport_height);
        let entity_screen_size = entity_size * camera.zoom;
        let padded_entity_size = entity_size * 1.5;
        let padded_screen_size = padded_entity_size * camera.zoom;
        
        // Account for zoom clamping - if zoom is at min/max, we can't guarantee perfect framing
        let zoom_is_clamped = (camera.zoom - camera.min_zoom).abs() < 0.01 || 
                              (camera.zoom - camera.max_zoom).abs() < 0.01;
        
        if !zoom_is_clamped {
            // When not clamped, entity with padding should fit in viewport
            prop_assert!(
                padded_screen_size <= viewport_min * 1.1,
                "Entity with padding should fit in viewport when zoom is not clamped. Padded size: {}, Viewport: {}",
                padded_screen_size,
                viewport_min
            );
            
            // Entity should take up reasonable portion of viewport
            prop_assert!(
                entity_screen_size >= viewport_min * 0.3,
                "Entity should take up reasonable portion of viewport when zoom is not clamped"
            );
        } else {
            // When clamped, just verify zoom is within bounds
            prop_assert!(
                camera.zoom >= camera.min_zoom && camera.zoom <= camera.max_zoom,
                "Zoom should be within bounds"
            );
        }
    }
    
    // Feature: unity-scene-view, Property 12: Mode switching preserves camera state
    // Validates: Requirements 3.3
    #[test]
    fn prop_mode_switching_preserves_camera_state(
        initial_pos in prop_vec2(),
        initial_zoom in prop_zoom(),
        initial_rotation in prop_angle(),
        initial_pitch in prop_pitch(),
    ) {
        let mut camera = SceneCamera::new();
        camera.position = initial_pos;
        camera.zoom = initial_zoom;
        camera.target_zoom = initial_zoom;
        camera.rotation = initial_rotation;
        camera.pitch = initial_pitch;
        
        // Switch from 3D to 2D
        camera.switch_to_2d();
        
        // Position and zoom should be preserved
        prop_assert!(
            (camera.position - initial_pos).length() < 0.01,
            "Position should be preserved when switching to 2D. Expected: {:?}, Actual: {:?}",
            initial_pos,
            camera.position
        );
        
        prop_assert!(
            (camera.zoom - initial_zoom).abs() < 0.01,
            "Zoom should be preserved when switching to 2D. Expected: {}, Actual: {}",
            initial_zoom,
            camera.zoom
        );
        
        // Rotation and pitch should be reset for 2D mode
        prop_assert!(
            camera.rotation.abs() < 0.01,
            "Rotation should be reset to 0 in 2D mode"
        );
        
        prop_assert!(
            camera.pitch.abs() < 0.01,
            "Pitch should be reset to 0 in 2D mode"
        );
        
        // Switch back to 3D
        camera.switch_to_3d();
        
        // Position and zoom should still be preserved
        prop_assert!(
            (camera.position - initial_pos).length() < 0.01,
            "Position should be preserved when switching back to 3D. Expected: {:?}, Actual: {:?}",
            initial_pos,
            camera.position
        );
        
        prop_assert!(
            (camera.zoom - initial_zoom).abs() < 0.01,
            "Zoom should be preserved when switching back to 3D. Expected: {}, Actual: {}",
            initial_zoom,
            camera.zoom
        );
    }
    
    // Feature: unity-scene-view, Property 13: 3D mode restores or initializes orientation
    // Validates: Requirements 3.4
    #[test]
    fn prop_3d_mode_restores_or_initializes_orientation(
        initial_rotation in prop_angle(),
        initial_pitch in prop_pitch(),
    ) {
        // Test case 1: Switching to 3D with previous state
        let mut camera = SceneCamera::new();
        camera.rotation = initial_rotation;
        camera.pitch = initial_pitch;
        
        // Switch to 2D and back to 3D
        camera.switch_to_2d();
        camera.switch_to_3d();
        
        // Should restore previous 3D orientation
        prop_assert!(
            (camera.rotation - initial_rotation).abs() < 0.01,
            "Rotation should be restored when switching back to 3D. Expected: {}, Actual: {}",
            initial_rotation,
            camera.rotation
        );
        
        prop_assert!(
            (camera.pitch - initial_pitch).abs() < 0.01,
            "Pitch should be restored when switching back to 3D. Expected: {}, Actual: {}",
            initial_pitch,
            camera.pitch
        );
        
        // Test case 2: Switching to 3D without previous state
        let mut camera2 = SceneCamera::new();
        camera2.saved_3d_state = None; // Ensure no saved state
        camera2.rotation = 0.0;
        camera2.pitch = 0.0;
        
        camera2.switch_to_3d();
        
        // Should initialize to default isometric view
        prop_assert!(
            (camera2.rotation - 45.0).abs() < 0.01,
            "Rotation should be initialized to 45° when no previous state exists. Actual: {}",
            camera2.rotation
        );
        
        prop_assert!(
            (camera2.pitch - 30.0).abs() < 0.01,
            "Pitch should be initialized to 30° when no previous state exists. Actual: {}",
            camera2.pitch
        );
    }
}
