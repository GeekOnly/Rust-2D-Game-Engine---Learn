# Sprite/Tilemap System Usage Examples

## Overview

This document provides examples of how to use the new sprite sheet and tilemap system in the Rust 2D Game Engine.

## Sprite Sheets

### Creating a Sprite Sheet from a Grid

```rust
use ecs::{World, SpriteSheet, AnimatedSprite, AnimationMode};

let mut world = World::new();

// Create a sprite sheet entity
let sprite_sheet_entity = world.spawn();

// Create a grid-based sprite sheet (e.g., 8x8 tiles, 32x32 pixels each)
let sprite_sheet = SpriteSheet::from_grid(
    "assets/character_sheet.png",  // Texture path
    "character_sheet",              // Texture ID
    256,                            // Sheet width (pixels)
    256,                            // Sheet height (pixels)
    32,                             // Frame width (pixels)
    32,                             // Frame height (pixels)
    0,                              // Spacing between frames
    0,                              // Margin around sheet
);

world.sprite_sheets.insert(sprite_sheet_entity, sprite_sheet);
world.names.insert(sprite_sheet_entity, "Character Sprite Sheet".to_string());
```

### Creating an Animated Sprite

```rust
// Create an entity with an animated sprite
let character_entity = world.spawn();

// Create animated sprite component
let mut animated_sprite = AnimatedSprite::new("character_sheet", 0.1); // 10 FPS
animated_sprite.mode = AnimationMode::Loop;
animated_sprite.frame_sequence = vec![0, 1, 2, 3]; // Walk cycle frames

world.animated_sprites.insert(character_entity, animated_sprite);
world.names.insert(character_entity, "Player Character".to_string());

// Add transform
let transform = Transform::with_position(100.0, 100.0, 0.0);
world.transforms.insert(character_entity, transform);
```

### Updating Animations

```rust
// In your game loop
let delta_time = 0.016; // 60 FPS

for (entity, animated_sprite) in &mut world.animated_sprites {
    if let Some(sprite_sheet) = world.sprite_sheets.get(&entity) {
        animated_sprite.update(delta_time, sprite_sheet.frames.len());
    }
}
```

## Tilemaps

### Creating a Tilemap Manually

```rust
use ecs::{Tilemap, TileSet, Tile};

// Create a tileset entity
let tileset_entity = world.spawn();

let tileset = TileSet::new(
    "Dungeon Tiles",                // Name
    "assets/dungeon_tileset.png",   // Texture path
    "dungeon_tileset",              // Texture ID
    16,                             // Tile width
    16,                             // Tile height
    8,                              // Columns
    64,                             // Total tiles
);

world.tilesets.insert(tileset_entity, tileset);
world.names.insert(tileset_entity, "Dungeon TileSet".to_string());

// Create a tilemap entity
let tilemap_entity = world.spawn();

let mut tilemap = Tilemap::new(
    "Ground Layer",      // Layer name
    "dungeon_tileset",   // Tileset ID
    20,                  // Width (tiles)
    15,                  // Height (tiles)
);

// Set some tiles
tilemap.set_tile_id(0, 0, 1);  // Top-left corner
tilemap.set_tile_id(1, 0, 2);  // Ground tile
tilemap.set_tile_id(2, 0, 3);  // Another tile

// Configure layer properties
tilemap.z_order = 0;
tilemap.visible = true;
tilemap.opacity = 1.0;

world.tilemaps.insert(tilemap_entity, tilemap);
world.names.insert(tilemap_entity, "Ground Layer".to_string());

// Add transform
let transform = Transform::default();
world.transforms.insert(tilemap_entity, transform);
```

### Loading LDTK Files

```rust
use ecs::loaders::LdtkLoader;

// Load an LDTK project
let entities = LdtkLoader::load_project("assets/levels/dungeon.ldtk", &mut world)
    .expect("Failed to load LDTK project");

println!("Loaded {} entities from LDTK", entities.len());
```

### Loading Tiled/TMX Files

```rust
use ecs::loaders::TiledLoader;

// Load a Tiled map
let entities = TiledLoader::load_map("assets/levels/level1.tmx", &mut world)
    .expect("Failed to load Tiled map");

println!("Loaded {} entities from Tiled", entities.len());
```

## Advanced Features

### Parallax Scrolling

```rust
// Create a background layer with parallax
let background_entity = world.spawn();

let mut background_tilemap = Tilemap::new(
    "Background",
    "background_tileset",
    30,
    20,
);

// Set parallax factor (0.5 = moves at half speed)
background_tilemap.parallax_factor = (0.5, 0.5);
background_tilemap.z_order = -1; // Behind other layers

world.tilemaps.insert(background_entity, background_tilemap);
```

### Tile Flipping

```rust
// Create a tile with horizontal flip
let mut tile = Tile::new(5);
tile.flip_h = true;  // Flip horizontally
tile.flip_v = false; // Don't flip vertically

tilemap.set_tile(10, 5, tile);
```

### Animation Modes

```rust
// Play once and stop
let mut anim = AnimatedSprite::new("explosion", 0.05);
anim.mode = AnimationMode::Once;

// Loop continuously
let mut anim = AnimatedSprite::new("idle", 0.15);
anim.mode = AnimationMode::Loop;

// Ping-pong (forward then backward)
let mut anim = AnimatedSprite::new("breathing", 0.2);
anim.mode = AnimationMode::PingPong;
```

### World to Tile Conversion

```rust
// Convert world coordinates to tile coordinates
let (tile_x, tile_y) = tilemap.world_to_tile(
    player_x,
    player_y,
    16,  // Tile width
    16,  // Tile height
);

// Get the tile at that position
if let Some(tile) = tilemap.get_tile(tile_x, tile_y) {
    println!("Player is standing on tile ID: {}", tile.tile_id);
}

// Convert tile coordinates to world position
let (world_x, world_y) = tilemap.tile_to_world(tile_x, tile_y, 16, 16);
```

## Serialization

The tilemap and sprite sheet components are fully serializable:

```rust
// Save the world (includes all tilemaps and sprite sheets)
let json = world.save_to_json().expect("Failed to serialize");
std::fs::write("save_game.json", json).expect("Failed to write file");

// Load the world
let json = std::fs::read_to_string("save_game.json").expect("Failed to read file");
world.load_from_json(&json).expect("Failed to deserialize");
```

## Next Steps

- Implement rendering support in the `render` crate
- Add editor UI for tilemap editing
- Create tilemap painting tools
- Add collision detection for tilemaps
