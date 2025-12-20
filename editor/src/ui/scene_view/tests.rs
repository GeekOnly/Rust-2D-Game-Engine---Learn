//! Tests for Scene View Mode and Unified Camera Integration

#[cfg(test)]
mod tests {
    use crate::ui::scene_view::{sync_scene_view_mode_with_camera, types::SceneViewMode};
    use ecs::World;

    #[test]
    fn test_scene_view_mode_sync_with_camera() {
        let mut world = World::new();
        
        // Create a camera entity
        let camera_entity = world.spawn();
        let camera = ecs::Camera::default();
        
        // Add camera component using the correct method
        world.cameras.insert(camera_entity, camera);
        
        // Test syncing 2D mode
        sync_scene_view_mode_with_camera(&mut world, SceneViewMode::Mode2D, Some(camera_entity));
        
        // Verify the camera has unified rendering enabled and is in 2D mode
        if let Some(camera) = world.cameras.get(&camera_entity) {
            assert!(camera.has_unified_rendering(), "Camera should have unified rendering enabled");
            assert_eq!(camera.get_view_mode(), Some(ecs::components::ViewMode::Mode2D), "Camera should be in 2D mode");
        } else {
            panic!("Camera component not found");
        }
        
        // Test syncing 3D mode
        sync_scene_view_mode_with_camera(&mut world, SceneViewMode::Mode3D, Some(camera_entity));
        
        // Verify the camera is now in 3D mode
        if let Some(camera) = world.cameras.get(&camera_entity) {
            assert_eq!(camera.get_view_mode(), Some(ecs::components::ViewMode::Mode3D), "Camera should be in 3D mode");
        } else {
            panic!("Camera component not found");
        }
    }

    #[test]
    fn test_scene_view_mode_sync_all_cameras() {
        let mut world = World::new();
        
        // Create multiple camera entities
        let camera1 = world.spawn();
        let camera2 = world.spawn();
        
        world.cameras.insert(camera1, ecs::Camera::default());
        world.cameras.insert(camera2, ecs::Camera::default());
        
        // Test syncing all cameras to 3D mode (no specific entity selected)
        sync_scene_view_mode_with_camera(&mut world, SceneViewMode::Mode3D, None);
        
        // Verify both cameras are in 3D mode
        for &entity in &[camera1, camera2] {
            if let Some(camera) = world.cameras.get(&entity) {
                assert!(camera.has_unified_rendering(), "Camera should have unified rendering enabled");
                assert_eq!(camera.get_view_mode(), Some(ecs::components::ViewMode::Mode3D), "Camera should be in 3D mode");
            } else {
                panic!("Camera component not found for entity {:?}", entity);
            }
        }
    }

    #[test]
    fn test_smooth_camera_transitions() {
        use crate::systems::camera::SceneCamera;
        
        let mut camera = SceneCamera::new();
        
        // Start in 2D mode
        assert_eq!(camera.pitch, 0.0);
        assert_eq!(camera.rotation, 0.0);
        
        // Switch to 3D mode with smooth transition
        camera.switch_to_3d_smooth();
        
        // Should be transitioning
        assert!(camera.is_transitioning());
        
        // Simulate some time passing
        camera.update(0.016); // 16ms frame
        
        // Should still be transitioning (1 second duration)
        assert!(camera.is_transitioning());
        
        // Simulate transition completion
        for _ in 0..70 { // ~1.1 seconds at 16ms per frame
            camera.update(0.016);
        }
        
        // Should no longer be transitioning
        assert!(!camera.is_transitioning());
        
        // Should be in 3D position
        assert!(camera.pitch > 0.0);
        assert!(camera.rotation > 0.0);
        
        // Switch back to 2D
        camera.switch_to_2d_smooth();
        assert!(camera.is_transitioning());
        
        // Complete transition
        for _ in 0..70 {
            camera.update(0.016);
        }
        
        // Should be back in 2D position
        assert!(!camera.is_transitioning());
        assert_eq!(camera.pitch, 0.0);
        assert_eq!(camera.rotation, 0.0);
    }

    #[test]
    fn test_camera_state_preservation() {
        use crate::systems::camera::SceneCamera;
        
        let mut camera = SceneCamera::new();
        
        // Set up a custom 2D state
        camera.position = glam::Vec3::new(10.0, 20.0, 0.0);
        camera.zoom = 3.0;
        
        // Switch to 3D
        camera.switch_to_3d_smooth();
        
        // Complete transition
        for _ in 0..70 {
            camera.update(0.016);
        }
        
        // Modify 3D state
        camera.position = glam::Vec3::new(5.0, 15.0, 25.0);
        
        // Switch back to 2D
        camera.switch_to_2d_smooth();
        
        // Complete transition
        for _ in 0..70 {
            camera.update(0.016);
        }
        
        // Should restore the original 2D state
        assert_eq!(camera.position.x, 10.0);
        assert_eq!(camera.position.y, 20.0);
        assert_eq!(camera.zoom, 3.0);
        assert_eq!(camera.pitch, 0.0);
        assert_eq!(camera.rotation, 0.0);
    }

    #[test]
    fn test_transition_duration_setting() {
        use crate::systems::camera::SceneCamera;
        
        let mut camera = SceneCamera::new();
        
        // Set a shorter transition duration
        camera.set_transition_duration(0.5); // 0.5 seconds
        
        // Switch to 3D mode
        camera.switch_to_3d_smooth();
        assert!(camera.is_transitioning());
        
        // Simulate transition completion with shorter duration
        for _ in 0..35 { // ~0.56 seconds at 16ms per frame
            camera.update(0.016);
        }
        
        // Should be complete
        assert!(!camera.is_transitioning());
        assert!(camera.pitch > 0.0);
        assert!(camera.rotation > 0.0);
    }
}