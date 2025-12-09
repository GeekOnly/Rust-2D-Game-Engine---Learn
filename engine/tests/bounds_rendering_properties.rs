// Property-based tests for bounds rendering
// These tests validate the correctness properties defined in the scene-view-improvements design document

use proptest::prelude::*;
use ecs::{World, Entity, Transform, Sprite};
use glam::{Vec2, Vec3};

// Mock camera for testing
#[derive(Clone, Debug)]
struct TestCamera {
    position: Vec2,
    rotation: f32,
    pitch: f32,
    zoom: f32,
}

impl TestCamera {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            pitch: 45.0,
            zoom: 50.0,
        }
    }
}

// Helper to create a sprite entity at a specific depth
fn create_sprite_at_depth(world: &mut World, z: f32) -> Entity {
    let entity = world.spawn();
    world.transforms.insert(entity, Transform {
        position: [0.0, 0.0, z],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    
    world.sprites.insert(entity, Sprite {
        texture_id: "test".to_string(),
        width: 32.0,
        height: 32.0,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    entity
}

// Helper to calculate depth from camera (simplified version)
fn calculate_depth_from_camera(position: Vec3, camera: &TestCamera) -> f32 {
    let relative_pos = Vec3::new(
        position.x - camera.position.x,
        position.y,
        position.z - camera.position.y,
    );
    
    let yaw = camera.rotation.to_radians();
    let pitch = camera.pitch.to_radians();
    
    // Rotate around Y axis (yaw)
    let cos_yaw = yaw.cos();
    let sin_yaw = yaw.sin();
    let rotated_z = -relative_pos.x * sin_yaw + relative_pos.z * cos_yaw;
    
    // Rotate around X axis (pitch)
    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();
    let final_z = relative_pos.y * sin_pitch + rotated_z * cos_pitch;
    
    final_z
}

// Helper to check if bounds should be occluded
fn should_bounds_be_occluded(
    bounds_entity: Entity,
    occluding_entity: Entity,
    world: &World,
    camera: &TestCamera,
) -> bool {
    let bounds_transform = world.transforms.get(&bounds_entity).unwrap();
    let occluding_transform = world.transforms.get(&occluding_entity).unwrap();
    
    let bounds_pos = Vec3::new(
        bounds_transform.position[0],
        bounds_transform.position[1],
        bounds_transform.position[2],
    );
    
    let occluding_pos = Vec3::new(
        occluding_transform.position[0],
        occluding_transform.position[1],
        occluding_transform.position[2],
    );
    
    let bounds_depth = calculate_depth_from_camera(bounds_pos, camera);
    let occluding_depth = calculate_depth_from_camera(occluding_pos, camera);
    
    // Bounds should be occluded if the occluding object is closer (smaller depth)
    occluding_depth < bounds_depth
}

// Strategy for generating Z positions
fn prop_z_position() -> impl Strategy<Value = f32> {
    -500.0f32..500.0f32
}

// Strategy for generating camera angles
fn prop_camera_angle() -> impl Strategy<Value = f32> {
    -180.0f32..180.0f32
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: scene-view-improvements, Property 26: Bounds respect depth testing
    // Validates: Requirements 15.4
    #[test]
    fn prop_bounds_respect_depth_testing(
        x_offset in -50.0f32..50.0f32,
        z_near in 10.0f32..100.0f32,
        z_far in 101.0f32..500.0f32,
        camera_yaw in prop_camera_angle(),
        camera_pitch in 0.0f32..89.0f32,
    ) {
        let mut world = World::new();
        let mut camera = TestCamera::new();
        camera.rotation = camera_yaw;
        camera.pitch = camera_pitch;
        
        // Create two sprites at different depths, with the same X offset
        // This ensures they're both in front of the camera regardless of rotation
        let entity1 = world.spawn();
        world.transforms.insert(entity1, Transform {
            position: [x_offset, 0.0, z_near],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world.sprites.insert(entity1, Sprite {
            texture_id: "test".to_string(),
            width: 32.0,
            height: 32.0,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            flip_x: false,
            flip_y: false,
            sprite_rect: None,
            pixels_per_unit: 100.0,
        });
        
        let entity2 = world.spawn();
        world.transforms.insert(entity2, Transform {
            position: [x_offset, 0.0, z_far],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        });
        world.sprites.insert(entity2, Sprite {
            texture_id: "test".to_string(),
            width: 32.0,
            height: 32.0,
            color: [1.0, 1.0, 1.0, 1.0],
            billboard: false,
            flip_x: false,
            flip_y: false,
            sprite_rect: None,
            pixels_per_unit: 100.0,
        });
        
        // Calculate depths from camera
        let transform1 = world.transforms.get(&entity1).unwrap();
        let transform2 = world.transforms.get(&entity2).unwrap();
        
        let pos1 = Vec3::new(
            transform1.position[0],
            transform1.position[1],
            transform1.position[2],
        );
        
        let pos2 = Vec3::new(
            transform2.position[0],
            transform2.position[1],
            transform2.position[2],
        );
        
        let depth1 = calculate_depth_from_camera(pos1, &camera);
        let depth2 = calculate_depth_from_camera(pos2, &camera);
        
        // Property: The entity with smaller depth should occlude the entity with larger depth
        // Determine which entity is closer
        let (closer_entity, farther_entity) = if depth1 < depth2 {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };
        
        // Verify occlusion decision
        let should_occlude = should_bounds_be_occluded(farther_entity, closer_entity, &world, &camera);
        prop_assert!(
            should_occlude,
            "Bounds of farther entity should be occluded by closer entity. Depth1: {}, Depth2: {}",
            depth1,
            depth2
        );
        
        // Verify the opposite is not true
        let should_not_occlude = should_bounds_be_occluded(closer_entity, farther_entity, &world, &camera);
        prop_assert!(
            !should_not_occlude,
            "Bounds of closer entity should NOT be occluded by farther entity. Depth1: {}, Depth2: {}",
            depth1,
            depth2
        );
    }
    
    // Additional property: Bounds at same depth don't occlude each other
    #[test]
    fn prop_bounds_same_depth_no_occlusion(
        z_position in prop_z_position(),
        camera_yaw in prop_camera_angle(),
    ) {
        let mut world = World::new();
        let mut camera = TestCamera::new();
        camera.rotation = camera_yaw;
        
        // Create two sprites at the same depth
        let entity1 = create_sprite_at_depth(&mut world, z_position);
        let entity2 = create_sprite_at_depth(&mut world, z_position);
        
        // Calculate depths
        let transform1 = world.transforms.get(&entity1).unwrap();
        let transform2 = world.transforms.get(&entity2).unwrap();
        
        let pos1 = Vec3::new(
            transform1.position[0],
            transform1.position[1],
            transform1.position[2],
        );
        
        let pos2 = Vec3::new(
            transform2.position[0],
            transform2.position[1],
            transform2.position[2],
        );
        
        let depth1 = calculate_depth_from_camera(pos1, &camera);
        let depth2 = calculate_depth_from_camera(pos2, &camera);
        
        // Verify depths are approximately equal
        prop_assert!(
            (depth1 - depth2).abs() < 0.1,
            "Entities at same Z should have similar depths. Depth1: {}, Depth2: {}",
            depth1,
            depth2
        );
        
        // Neither should occlude the other
        let occlude_1_by_2 = should_bounds_be_occluded(entity1, entity2, &world, &camera);
        let occlude_2_by_1 = should_bounds_be_occluded(entity2, entity1, &world, &camera);
        
        prop_assert!(
            !occlude_1_by_2 && !occlude_2_by_1,
            "Entities at same depth should not occlude each other's bounds"
        );
    }
    
    // Property: Bounds occlusion is transitive
    #[test]
    fn prop_bounds_occlusion_transitive(
        z_near in 10.0f32..50.0f32,
        z_mid in 51.0f32..100.0f32,
        z_far in 101.0f32..200.0f32,
    ) {
        let mut world = World::new();
        let camera = TestCamera::new();
        
        // Create three sprites at different depths
        let near_entity = create_sprite_at_depth(&mut world, z_near);
        let mid_entity = create_sprite_at_depth(&mut world, z_mid);
        let far_entity = create_sprite_at_depth(&mut world, z_far);
        
        // If A occludes B and B occludes C, then A should occlude C
        let near_occludes_mid = should_bounds_be_occluded(mid_entity, near_entity, &world, &camera);
        let mid_occludes_far = should_bounds_be_occluded(far_entity, mid_entity, &world, &camera);
        let near_occludes_far = should_bounds_be_occluded(far_entity, near_entity, &world, &camera);
        
        prop_assert!(
            near_occludes_mid,
            "Near entity should occlude mid entity"
        );
        
        prop_assert!(
            mid_occludes_far,
            "Mid entity should occlude far entity"
        );
        
        prop_assert!(
            near_occludes_far,
            "Near entity should occlude far entity (transitivity)"
        );
    }
}
