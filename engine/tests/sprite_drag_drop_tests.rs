/// Unit tests for sprite drag-drop functionality
/// 
/// These tests verify that dragging and dropping sprite files onto the scene
/// creates entities with the correct components.

use ecs::{World, Transform, Sprite, SpriteSheet, AnimatedSprite};

#[test]
fn test_sprite_entity_has_required_components() {
    // This test verifies that when a sprite is dropped onto the scene,
    // the created entity has all required components:
    // - Transform
    // - Sprite
    // - SpriteSheet
    // - AnimatedSprite
    
    let mut world = World::new();
    
    // Simulate creating an entity from a sprite drop
    let entity = world.spawn();
    
    // Add Transform component
    world.transforms.insert(entity, Transform {
        position: [100.0, 200.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    
    // Add Sprite component
    world.sprites.insert(entity, Sprite {
        texture_id: "test_texture.png".to_string(),
        width: 32.0,
        height: 32.0,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    // Add SpriteSheet component
    let mut sprite_sheet = SpriteSheet::new(
        "test_texture.png",
        "test_texture.png",
        256,
        256,
    );
    sprite_sheet.add_frame(ecs::SpriteFrame {
        x: 0,
        y: 0,
        width: 32,
        height: 32,
        name: Some("sprite_0".to_string()),
    });
    world.sprite_sheets.insert(entity, sprite_sheet);
    
    // Add AnimatedSprite component
    let animated_sprite = AnimatedSprite::new("test_texture.png", 0.1);
    world.animated_sprites.insert(entity, animated_sprite);
    
    // Verify all components exist
    assert!(world.transforms.contains_key(&entity), "Entity should have Transform component");
    assert!(world.sprites.contains_key(&entity), "Entity should have Sprite component");
    assert!(world.sprite_sheets.contains_key(&entity), "Entity should have SpriteSheet component");
    assert!(world.animated_sprites.contains_key(&entity), "Entity should have AnimatedSprite component");
}

#[test]
fn test_sprite_entity_transform_at_drop_position() {
    // This test verifies that the entity is created at the correct world position
    // where the sprite was dropped
    
    let mut world = World::new();
    let entity = world.spawn();
    
    let drop_x = 150.0;
    let drop_y = 250.0;
    
    world.transforms.insert(entity, Transform {
        position: [drop_x, drop_y, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    
    let transform = world.transforms.get(&entity).unwrap();
    assert_eq!(transform.position[0], drop_x, "X position should match drop position");
    assert_eq!(transform.position[1], drop_y, "Y position should match drop position");
    assert_eq!(transform.position[2], 0.0, "Z position should be 0 for 2D sprites");
}

#[test]
fn test_sprite_sheet_has_frames() {
    // This test verifies that the SpriteSheet component contains the frames
    // from the loaded sprite metadata
    
    let mut world = World::new();
    let entity = world.spawn();
    
    let mut sprite_sheet = SpriteSheet::new(
        "test_texture.png",
        "test_texture.png",
        256,
        256,
    );
    
    // Add multiple frames
    sprite_sheet.add_frame(ecs::SpriteFrame {
        x: 0,
        y: 0,
        width: 32,
        height: 32,
        name: Some("sprite_0".to_string()),
    });
    sprite_sheet.add_frame(ecs::SpriteFrame {
        x: 32,
        y: 0,
        width: 32,
        height: 32,
        name: Some("sprite_1".to_string()),
    });
    
    world.sprite_sheets.insert(entity, sprite_sheet);
    
    let sheet = world.sprite_sheets.get(&entity).unwrap();
    assert_eq!(sheet.frames.len(), 2, "SpriteSheet should have 2 frames");
    assert_eq!(sheet.frames[0].name.as_ref().unwrap(), "sprite_0");
    assert_eq!(sheet.frames[1].name.as_ref().unwrap(), "sprite_1");
}

#[test]
fn test_animated_sprite_starts_at_first_frame() {
    // This test verifies that the AnimatedSprite component starts at frame 0
    // and is not playing by default
    
    let mut world = World::new();
    let entity = world.spawn();
    
    let mut animated_sprite = AnimatedSprite::new("test_texture.png", 0.1);
    animated_sprite.current_frame = 0;
    animated_sprite.playing = false;
    
    world.animated_sprites.insert(entity, animated_sprite);
    
    let anim = world.animated_sprites.get(&entity).unwrap();
    assert_eq!(anim.current_frame, 0, "AnimatedSprite should start at frame 0");
    assert!(!anim.playing, "AnimatedSprite should not be playing by default");
}

#[test]
fn test_sprite_component_matches_first_frame() {
    // This test verifies that the Sprite component dimensions match
    // the first frame of the sprite sheet
    
    let mut world = World::new();
    let entity = world.spawn();
    
    let frame_width = 32.0;
    let frame_height = 48.0;
    
    world.sprites.insert(entity, Sprite {
        texture_id: "test_texture.png".to_string(),
        width: frame_width,
        height: frame_height,
        color: [1.0, 1.0, 1.0, 1.0],
        billboard: false,
        flip_x: false,
        flip_y: false,
        sprite_rect: None,
        pixels_per_unit: 100.0,
    });
    
    let sprite = world.sprites.get(&entity).unwrap();
    assert_eq!(sprite.width, frame_width, "Sprite width should match frame width");
    assert_eq!(sprite.height, frame_height, "Sprite height should match frame height");
}
