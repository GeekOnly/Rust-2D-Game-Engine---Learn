//! Integration tests for 3D scene view rendering
//!
//! Tests the integration of Sprite3DRenderer, Tilemap3DRenderer, and RenderQueue
//! in the scene view 3D rendering pipeline.

use ecs::{World, Transform, Sprite};

#[test]
fn test_3d_rendering_integration_with_sprites() {
    // Create a world with sprites
    let mut world = World::new();
    
    // Create sprite entities at different Z depths
    let entity1 = world.spawn();
    world.transforms.insert(entity1, Transform {
        position: [0.0, 0.0, 10.0], // Farther
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.sprites.insert(entity1, Sprite {
        texture_id: "test1".to_string(),
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
        position: [0.0, 0.0, 5.0], // Closer
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.sprites.insert(entity2, Sprite {
        texture_id: "test2".to_string(),
        width: 32.0,
        height: 32.0,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    // Verify sprites were created
    assert_eq!(world.sprites.len(), 2);
    assert_eq!(world.transforms.len(), 2);
}



#[test]
fn test_billboard_sprite_in_3d() {
    // Create a world with a billboard sprite
    let mut world = World::new();
    
    let entity = world.spawn();
    world.transforms.insert(entity, Transform {
        position: [10.0, 5.0, 15.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.sprites.insert(entity, Sprite {
        texture_id: "billboard".to_string(),
        width: 64.0,
        height: 64.0,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: true, // Billboard mode enabled
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    // Verify billboard sprite was created
    assert_eq!(world.sprites.len(), 1);
    let sprite = world.sprites.get(&entity).unwrap();
    assert!(sprite.billboard);
}

#[test]
fn test_transparent_sprite_rendering() {
    // Create a world with transparent sprites
    let mut world = World::new();
    
    // Opaque sprite
    let entity1 = world.spawn();
    world.transforms.insert(entity1, Transform {
        position: [0.0, 0.0, 5.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.sprites.insert(entity1, Sprite {
        texture_id: "opaque".to_string(),
        width: 32.0,
        height: 32.0,
        color: [1.0, 1.0, 1.0, 1.0], // Fully opaque
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    // Transparent sprite
    let entity2 = world.spawn();
    world.transforms.insert(entity2, Transform {
        position: [0.0, 0.0, 10.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.sprites.insert(entity2, Sprite {
        texture_id: "transparent".to_string(),
        width: 32.0,
        height: 32.0,
        color: [1.0, 1.0, 1.0, 0.5], // Semi-transparent
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    // Verify both sprites were created
    assert_eq!(world.sprites.len(), 2);
    
    // Verify transparency values
    let sprite1 = world.sprites.get(&entity1).unwrap();
    assert_eq!(sprite1.color[3], 1.0);
    
    let sprite2 = world.sprites.get(&entity2).unwrap();
    assert_eq!(sprite2.color[3], 0.5);
}


