// Property-based tests for render queue depth sorting
// These tests validate the correctness properties defined in the scene-view-improvements design document

use proptest::prelude::*;
use ecs::Entity;
use glam::{Vec2, Vec3};
use engine::editor::ui::scene_view::rendering::{RenderQueue, RenderObject};
use engine::editor::ui::scene_view::rendering::sprite_3d::SpriteRenderData;
use engine::editor::ui::scene_view::rendering::tilemap_3d::TilemapLayer;
use engine::editor::SceneCamera;

// Helper to create sprite render data at specific depth
fn create_sprite_at_depth(entity: Entity, z: f32) -> SpriteRenderData {
    SpriteRenderData {
        entity,
        position: Vec3::new(0.0, 0.0, z),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        texture_id: format!("sprite_{}", entity),
        sprite_rect: None,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: false,
        width: 32.0,
        height: 32.0,
    }
}

// Helper to create tilemap layer at specific depth
fn create_tilemap_at_depth(entity: Entity, z: f32) -> TilemapLayer {
    TilemapLayer {
        entity,
        z_depth: z,
        tiles: Vec::new(),
        bounds: egui::Rect::NOTHING,
        name: format!("layer_{}", entity),
        opacity: 1.0,
        visible: true,
    }
}

// Helper to extract depths from sorted render queue
fn get_sorted_depths(queue: &RenderQueue, camera: &SceneCamera) -> Vec<f32> {
    queue.get_sorted().iter().map(|obj| {
        match obj {
            RenderObject::Grid => f32::MAX,
            RenderObject::Sprite(sprite) => {
                // Calculate depth from camera
                let relative_pos = Vec3::new(
                    sprite.position.x - camera.position.x,
                    sprite.position.y,
                    sprite.position.z - camera.position.y,
                );
                
                let yaw = camera.rotation.to_radians();
                let pitch = camera.pitch.to_radians();
                
                let cos_yaw = yaw.cos();
                let sin_yaw = yaw.sin();
                let rotated_z = -relative_pos.x * sin_yaw + relative_pos.z * cos_yaw;
                
                let cos_pitch = pitch.cos();
                let sin_pitch = pitch.sin();
                relative_pos.y * sin_pitch + rotated_z * cos_pitch
            }
            RenderObject::Tilemap(layer) => layer.z_depth,
            RenderObject::Gizmo(_) => f32::MIN,
        }
    }).collect()
}

// Helper to check if depths are sorted in descending order (farther first)
fn is_sorted_descending(values: &[f32]) -> bool {
    values.windows(2).all(|w| {
        // Handle NaN/Inf gracefully
        if w[0].is_nan() || w[1].is_nan() {
            true
        } else {
            w[0] >= w[1]
        }
    })
}

// Strategy for generating Z positions
fn prop_z_position() -> impl Strategy<Value = f32> {
    -1000.0f32..1000.0f32
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: scene-view-improvements, Property 24: Closer objects occlude farther objects
    // Validates: Requirements 14.2, 14.3, 14.4
    #[test]
    fn prop_closer_objects_occlude_farther_objects(
        z_positions in prop::collection::vec(prop_z_position(), 2..20),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Create sprites at different Z positions
        for (i, z) in z_positions.iter().enumerate() {
            let sprite = create_sprite_at_depth(i as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Sort by depth
        queue.sort_by_depth(&camera);
        
        // Get sorted depths
        let sorted_depths = get_sorted_depths(&queue, &camera);
        
        // Verify depths are sorted in descending order (farther first for painter's algorithm)
        prop_assert!(
            is_sorted_descending(&sorted_depths),
            "Objects should be sorted by depth in descending order (farther first). Got: {:?}",
            sorted_depths
        );
        
        // Verify the first object has the largest depth (farthest)
        if sorted_depths.len() >= 2 {
            let first_depth = sorted_depths[0];
            let second_depth = sorted_depths[1];
            
            prop_assert!(
                first_depth >= second_depth,
                "First object should be farther than second. First: {}, Second: {}",
                first_depth,
                second_depth
            );
        }
        
        // Verify the last object has the smallest depth (closest)
        if sorted_depths.len() >= 2 {
            let last_idx = sorted_depths.len() - 1;
            let last_depth = sorted_depths[last_idx];
            let second_last_depth = sorted_depths[last_idx - 1];
            
            prop_assert!(
                last_depth <= second_last_depth,
                "Last object should be closer than second-to-last. Last: {}, Second-to-last: {}",
                last_depth,
                second_last_depth
            );
        }
        
        // Verify that for any two objects A and B where A is closer (smaller Z),
        // A appears later in the render queue than B
        for i in 0..z_positions.len() {
            for j in (i + 1)..z_positions.len() {
                let z_i = z_positions[i];
                let z_j = z_positions[j];
                
                if z_i < z_j {
                    // Object i is closer, should appear later in queue
                    let sorted = queue.get_sorted();
                    let pos_i = sorted.iter().position(|obj| {
                        if let RenderObject::Sprite(s) = obj {
                            s.entity == i as u32
                        } else {
                            false
                        }
                    });
                    let pos_j = sorted.iter().position(|obj| {
                        if let RenderObject::Sprite(s) = obj {
                            s.entity == j as u32
                        } else {
                            false
                        }
                    });
                    
                    if let (Some(pi), Some(pj)) = (pos_i, pos_j) {
                        prop_assert!(
                            pi > pj,
                            "Closer object (entity {}, z={}) should appear after farther object (entity {}, z={}) in render queue. Positions: {} vs {}",
                            i, z_i, j, z_j, pi, pj
                        );
                    }
                }
            }
        }
    }
    
    // Feature: scene-view-improvements, Property 24: Closer objects occlude farther objects (mixed types)
    // Validates: Requirements 14.2, 14.3, 14.4
    #[test]
    fn prop_closer_objects_occlude_farther_objects_mixed_types(
        sprite_z_positions in prop::collection::vec(prop_z_position(), 1..10),
        tilemap_z_positions in prop::collection::vec(prop_z_position(), 1..10),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add sprites
        for (i, z) in sprite_z_positions.iter().enumerate() {
            let sprite = create_sprite_at_depth(i as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Add tilemaps
        for (i, z) in tilemap_z_positions.iter().enumerate() {
            let tilemap = create_tilemap_at_depth((i + 1000) as u32, *z);
            queue.push(RenderObject::Tilemap(tilemap));
        }
        
        // Sort by depth
        queue.sort_by_depth(&camera);
        
        // Get sorted depths
        let sorted_depths = get_sorted_depths(&queue, &camera);
        
        // Verify depths are sorted in descending order
        prop_assert!(
            is_sorted_descending(&sorted_depths),
            "Mixed objects should be sorted by depth in descending order. Got: {:?}",
            sorted_depths
        );
        
        // Verify that sprites and tilemaps are interleaved correctly by depth
        let sorted = queue.get_sorted();
        for i in 0..(sorted.len() - 1) {
            let depth_i = sorted_depths[i];
            let depth_j = sorted_depths[i + 1];
            
            prop_assert!(
                depth_i >= depth_j,
                "Object at position {} (depth {}) should be farther than object at position {} (depth {})",
                i, depth_i, i + 1, depth_j
            );
        }
    }
    
    // Feature: scene-view-improvements, Property 24: Grid always renders first
    // Validates: Requirements 14.2
    #[test]
    fn prop_grid_always_renders_first(
        z_positions in prop::collection::vec(prop_z_position(), 1..10),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add grid
        queue.push(RenderObject::Grid);
        
        // Add sprites at various depths
        for (i, z) in z_positions.iter().enumerate() {
            let sprite = create_sprite_at_depth(i as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Sort by depth
        queue.sort_by_depth(&camera);
        
        // Verify grid is first
        let sorted = queue.get_sorted();
        prop_assert!(
            !sorted.is_empty(),
            "Queue should not be empty"
        );
        
        prop_assert!(
            matches!(sorted[0], RenderObject::Grid),
            "Grid should always be first in render queue"
        );
    }
    
    // Feature: scene-view-improvements, Property 24: Gizmos always render last
    // Validates: Requirements 14.2
    #[test]
    fn prop_gizmos_always_render_last(
        z_positions in prop::collection::vec(prop_z_position(), 1..10),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add sprites at various depths
        for (i, z) in z_positions.iter().enumerate() {
            let sprite = create_sprite_at_depth(i as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Add gizmo
        queue.push(RenderObject::Gizmo(
            engine::editor::ui::scene_view::rendering::GizmoData {
                entity: 9999,
                depth: 0.0,
                gizmo_type: engine::editor::ui::scene_view::rendering::GizmoType::Transform,
            }
        ));
        
        // Sort by depth
        queue.sort_by_depth(&camera);
        
        // Verify gizmo is last
        let sorted = queue.get_sorted();
        prop_assert!(
            !sorted.is_empty(),
            "Queue should not be empty"
        );
        
        let last_idx = sorted.len() - 1;
        prop_assert!(
            matches!(sorted[last_idx], RenderObject::Gizmo(_)),
            "Gizmo should always be last in render queue"
        );
    }
    
    // Feature: scene-view-improvements, Property 24: Depth sorting handles edge cases
    // Validates: Requirements 14.2, 14.3, 14.4
    #[test]
    fn prop_depth_sorting_handles_edge_cases(
        z_positions in prop::collection::vec(prop_z_position(), 2..10),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add objects with same Z position
        let same_z = z_positions[0];
        for i in 0..3 {
            let sprite = create_sprite_at_depth(i, same_z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Add objects with different Z positions
        for (i, z) in z_positions.iter().skip(1).enumerate() {
            let sprite = create_sprite_at_depth((i + 100) as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Sort by depth (should not panic)
        queue.sort_by_depth(&camera);
        
        // Verify all objects are present
        prop_assert_eq!(
            queue.len(),
            z_positions.len() + 2,
            "All objects should be present after sorting"
        );
        
        // Verify depths are sorted
        let sorted_depths = get_sorted_depths(&queue, &camera);
        prop_assert!(
            is_sorted_descending(&sorted_depths),
            "Depths should be sorted even with duplicates. Got: {:?}",
            sorted_depths
        );
    }
    
    // Feature: scene-view-improvements, Property 25: Depth sorting is consistent across object types
    // Validates: Requirements 14.1, 14.4
    #[test]
    fn prop_depth_sorting_consistent_across_types(
        sprite_z_positions in prop::collection::vec(prop_z_position(), 2..10),
        tilemap_z_positions in prop::collection::vec(prop_z_position(), 2..10),
    ) {
        let mut queue = RenderQueue::new();
        let mut camera = SceneCamera::new();
        camera.position = Vec2::new(0.0, 0.0);
        camera.rotation = 0.0;
        camera.pitch = 0.0;
        
        // Add grid (should be first)
        queue.push(RenderObject::Grid);
        
        // Add sprites at various depths
        for (i, z) in sprite_z_positions.iter().enumerate() {
            let sprite = create_sprite_at_depth(i as u32, *z);
            queue.push(RenderObject::Sprite(sprite));
        }
        
        // Add tilemaps at various depths
        for (i, z) in tilemap_z_positions.iter().enumerate() {
            let tilemap = create_tilemap_at_depth((i + 1000) as u32, *z);
            queue.push(RenderObject::Tilemap(tilemap));
        }
        
        // Add gizmo (should be last)
        queue.push(RenderObject::Gizmo(
            engine::editor::ui::scene_view::rendering::GizmoData {
                entity: 9999,
                depth: 0.0,
                gizmo_type: engine::editor::ui::scene_view::rendering::GizmoType::Transform,
            }
        ));
        
        // Sort by depth
        queue.sort_by_depth(&camera);
        
        // Get sorted objects
        let sorted = queue.get_sorted();
        
        // Verify grid is first
        prop_assert!(
            matches!(sorted[0], RenderObject::Grid),
            "Grid should always be first"
        );
        
        // Verify gizmo is last
        let last_idx = sorted.len() - 1;
        prop_assert!(
            matches!(sorted[last_idx], RenderObject::Gizmo(_)),
            "Gizmo should always be last"
        );
        
        // Verify all objects between grid and gizmo are sorted by depth
        // Extract depths for sprites and tilemaps (excluding grid and gizmo)
        let mut object_depths = Vec::new();
        for i in 1..last_idx {
            match &sorted[i] {
                RenderObject::Sprite(sprite) => {
                    // Calculate sprite depth
                    let relative_pos = Vec3::new(
                        sprite.position.x - camera.position.x,
                        sprite.position.y,
                        sprite.position.z - camera.position.y,
                    );
                    
                    let yaw = camera.rotation.to_radians();
                    let pitch = camera.pitch.to_radians();
                    
                    let cos_yaw = yaw.cos();
                    let sin_yaw = yaw.sin();
                    let rotated_z = -relative_pos.x * sin_yaw + relative_pos.z * cos_yaw;
                    
                    let cos_pitch = pitch.cos();
                    let sin_pitch = pitch.sin();
                    let final_z = relative_pos.y * sin_pitch + rotated_z * cos_pitch;
                    
                    object_depths.push(final_z);
                }
                RenderObject::Tilemap(layer) => {
                    object_depths.push(layer.z_depth);
                }
                _ => {}
            }
        }
        
        // Verify depths are sorted in descending order (farther first)
        prop_assert!(
            is_sorted_descending(&object_depths),
            "All objects (sprites and tilemaps) should be sorted by depth consistently. Got: {:?}",
            object_depths
        );
        
        // Verify that the same comparison function is used for all object types
        // by checking that sprites and tilemaps are interleaved correctly
        for i in 0..(object_depths.len() - 1) {
            prop_assert!(
                object_depths[i] >= object_depths[i + 1],
                "Depth at position {} ({}) should be >= depth at position {} ({})",
                i, object_depths[i], i + 1, object_depths[i + 1]
            );
        }
    }
}
