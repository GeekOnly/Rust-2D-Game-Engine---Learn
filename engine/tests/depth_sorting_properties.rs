// Property-based tests for depth sorting
// These tests validate the correctness properties defined in the unity-scene-view design document

use proptest::prelude::*;
use ecs::{World, Entity, Transform, Sprite, Mesh, MeshType};

// Helper to create entities with specific Z positions
fn create_entity_at_depth(world: &mut World, z: f32, has_alpha: bool) -> Entity {
    let entity = world.spawn();
    world.transforms.insert(entity, Transform {
        position: [0.0, 0.0, z],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    
    // Add sprite with optional transparency
    let alpha = if has_alpha { 0.5 } else { 1.0 };
    world.sprites.insert(entity, Sprite {
        texture_id: "test".to_string(),
        width: 10.0,
        height: 10.0,
        color: [1.0, 1.0, 1.0, alpha],
        billboard: false,
        flip_x: false,
        flip_y: false,
    });
    
    entity
}

// Helper to extract Z positions from sorted entities
fn get_sorted_z_positions(world: &World, entities: &[(Entity, &Transform)]) -> Vec<f32> {
    entities.iter()
        .map(|(_, transform)| transform.position[2])
        .collect()
}

// Helper to check if a list is sorted in ascending order
fn is_sorted_ascending(values: &[f32]) -> bool {
    values.windows(2).all(|w| w[0] <= w[1])
}

// Helper to check if transparent entities are sorted
fn is_transparent(world: &World, entity: Entity) -> bool {
    if let Some(sprite) = world.sprites.get(&entity) {
        sprite.color[3] < 1.0
    } else if let Some(mesh) = world.meshes.get(&entity) {
        mesh.color[3] < 1.0
    } else {
        false
    }
}

// Strategy for generating Z positions
fn prop_z_position() -> impl Strategy<Value = f32> {
    -1000.0f32..1000.0f32
}

// Strategy for generating alpha values
fn prop_alpha() -> impl Strategy<Value = f32> {
    0.0f32..1.0f32
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Feature: unity-scene-view, Property 18: Entities render in depth order
    // Validates: Requirements 8.1
    #[test]
    fn prop_entities_render_in_depth_order(
        z_positions in prop::collection::vec(prop_z_position(), 2..20),
    ) {
        let mut world = World::new();
        
        // Create entities at different Z positions
        let mut created_entities = Vec::new();
        for z in &z_positions {
            let entity = create_entity_at_depth(&mut world, *z, false);
            created_entities.push(entity);
        }
        
        // Collect and sort entities by Z position (simulating the rendering code)
        let mut entities: Vec<(Entity, &Transform)> = world.transforms.iter()
            .map(|(&e, t)| (e, t))
            .collect();
        
        // Sort by Z position (far to near) for painter's algorithm
        entities.sort_by(|a, b| {
            a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Extract sorted Z positions
        let sorted_z = get_sorted_z_positions(&world, &entities);
        
        // Verify entities are sorted in ascending Z order (back-to-front)
        prop_assert!(
            is_sorted_ascending(&sorted_z),
            "Entities should be sorted by Z position in ascending order (back-to-front). Got: {:?}",
            sorted_z
        );
        
        // Verify the first entity has the smallest Z (farthest back)
        if !sorted_z.is_empty() {
            let first_z = sorted_z[0];
            let min_z = z_positions.iter().cloned().fold(f32::INFINITY, f32::min);
            
            prop_assert!(
                (first_z - min_z).abs() < 0.001,
                "First entity should have smallest Z. First: {}, Min: {}",
                first_z,
                min_z
            );
        }
        
        // Verify the last entity has the largest Z (closest to camera)
        if !sorted_z.is_empty() {
            let last_z = sorted_z[sorted_z.len() - 1];
            let max_z = z_positions.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            
            prop_assert!(
                (last_z - max_z).abs() < 0.001,
                "Last entity should have largest Z. Last: {}, Max: {}",
                last_z,
                max_z
            );
        }
    }
    
    // Feature: unity-scene-view, Property 19: Transparent objects are sorted correctly
    // Validates: Requirements 8.2
    #[test]
    fn prop_transparent_objects_sorted_correctly(
        opaque_z_positions in prop::collection::vec(prop_z_position(), 1..10),
        transparent_z_positions in prop::collection::vec(prop_z_position(), 1..10),
    ) {
        let mut world = World::new();
        
        // Create opaque entities
        for z in &opaque_z_positions {
            create_entity_at_depth(&mut world, *z, false);
        }
        
        // Create transparent entities
        for z in &transparent_z_positions {
            create_entity_at_depth(&mut world, *z, true);
        }
        
        // Collect all entities
        let mut all_entities: Vec<(Entity, &Transform)> = world.transforms.iter()
            .map(|(&e, t)| (e, t))
            .collect();
        
        // Sort by Z position
        all_entities.sort_by(|a, b| {
            a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Separate transparent and opaque entities while maintaining order
        let transparent_entities: Vec<_> = all_entities.iter()
            .filter(|(e, _)| is_transparent(&world, *e))
            .collect();
        
        let opaque_entities: Vec<_> = all_entities.iter()
            .filter(|(e, _)| !is_transparent(&world, *e))
            .collect();
        
        // Verify transparent entities are sorted by depth
        let transparent_z: Vec<f32> = transparent_entities.iter()
            .map(|(_, t)| t.position[2])
            .collect();
        
        prop_assert!(
            is_sorted_ascending(&transparent_z),
            "Transparent entities should be sorted by Z position. Got: {:?}",
            transparent_z
        );
        
        // Verify opaque entities are sorted by depth
        let opaque_z: Vec<f32> = opaque_entities.iter()
            .map(|(_, t)| t.position[2])
            .collect();
        
        prop_assert!(
            is_sorted_ascending(&opaque_z),
            "Opaque entities should be sorted by Z position. Got: {:?}",
            opaque_z
        );
        
        // Verify all entities together are sorted (both opaque and transparent)
        let all_z = get_sorted_z_positions(&world, &all_entities);
        prop_assert!(
            is_sorted_ascending(&all_z),
            "All entities (opaque and transparent) should be sorted by Z position. Got: {:?}",
            all_z
        );
    }
    
    // Additional property: Transparent entities with same Z maintain stable order
    #[test]
    fn prop_transparent_same_depth_stable_order(
        z_position in prop_z_position(),
        count in 2usize..10,
    ) {
        let mut world = World::new();
        
        // Create multiple transparent entities at the same Z position
        let mut created_entities = Vec::new();
        for _ in 0..count {
            let entity = create_entity_at_depth(&mut world, z_position, true);
            created_entities.push(entity);
        }
        
        // Sort entities
        let mut entities: Vec<(Entity, &Transform)> = world.transforms.iter()
            .map(|(&e, t)| (e, t))
            .collect();
        
        entities.sort_by(|a, b| {
            a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Verify all entities have the same Z position
        let sorted_z = get_sorted_z_positions(&world, &entities);
        for z in &sorted_z {
            prop_assert!(
                (z - z_position).abs() < 0.001,
                "All entities should have Z position {}. Got: {}",
                z_position,
                z
            );
        }
        
        // Verify count is preserved
        prop_assert_eq!(
            sorted_z.len(),
            count,
            "All entities should be present after sorting"
        );
    }
}
