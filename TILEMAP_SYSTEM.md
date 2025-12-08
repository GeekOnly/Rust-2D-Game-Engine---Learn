# Unity-Style Tilemap System

## Overview
ระบบ Tilemap แบบ Unity ที่สมบูรณ์ พร้อม Grid component, Tilemap component, และ TilemapRenderer component

## Architecture

### 1. Grid Entity (Parent)
GameObject ที่มี Grid component สำหรับกำหนด cell layout

**Components:**
- `Transform`: ตำแหน่งของ Grid ใน world space
- `Grid`: กำหนด cell size, layout, และ plane orientation

**Grid Component Properties:**
```rust
pub struct Grid {
    pub cell_size: (f32, f32, f32),  // (width, height, depth) in world units
    pub cell_gap: (f32, f32),         // Gap between cells
    pub layout: GridLayout,           // Rectangle, Hexagon, Isometric
    pub swizzle: CellSwizzle,         // XYZ, XZY, YXZ, etc.
    pub plane: GridPlane,             // XY, XZ, YZ
}
```

**Grid Planes:**
- `GridPlane::XY` - Horizontal plane (default for 2D games)
- `GridPlane::XZ` - Vertical plane (for walls)
- `GridPlane::YZ` - Side plane (for side view)

**Helper Methods:**
```rust
Grid::new()              // Default horizontal grid
Grid::vertical()         // Vertical grid (XZ plane)
Grid::side()            // Side grid (YZ plane)
Grid::with_cell_size_3d(w, h, d)  // Custom 3D grid
```

### 2. Tilemap Entity (Child of Grid)
GameObject ที่เก็บ tile data และ rendering settings

**Components:**
- `Transform`: ตำแหน่งของ Tilemap (relative to Grid)
- `Tilemap`: เก็บ tile data (width, height, tiles array)
- `TileSet`: texture และ tile properties
- `TilemapRenderer`: rendering settings และ optimization

**Tilemap Component Properties:**
```rust
pub struct Tilemap {
    pub name: String,
    pub tileset_id: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: f32,
    pub parallax_factor: (f32, f32),
}
```

**TileSet Component Properties:**
```rust
pub struct TileSet {
    pub name: String,
    pub texture_path: String,
    pub texture_id: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub tile_count: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tiles: HashMap<u32, TileData>,
}
```

**TilemapRenderer Component Properties:**
```rust
pub struct TilemapRenderer {
    pub mode: TilemapRenderMode,        // Individual or Chunk
    pub sorting_layer: String,
    pub order_in_layer: i32,
    pub material: Option<String>,
    pub color: [f32; 4],                // Tint color
    pub chunk_size: u32,                // Chunk size for optimization
    pub detect_chunk_culling: bool,     // Cull off-screen chunks
    pub mask_interaction: MaskInteraction,
}
```

## Unit Scale System (Unity Standard)

**Pixels Per Unit (PPU) = 100.0**
- 100 pixels = 1 world unit (1 meter)
- Consistent across Sprites, Tilemaps, and Grid
- Example: 8x8 pixel tile = 0.08 x 0.08 world units

**Grid Cell Size:**
- LDtk default: 8 pixels
- World units: 8 / 100 = 0.08 units
- Grid component: `cell_size: (0.08, 0.08, 0.0)`

## Scene Structure Example

```
Scene: main
├── Main Camera (Entity 0)
├── Player (Entity 11)
└── Grid (Entity 314)
    ├── Grid Component
    │   ├── cell_size: (0.08, 0.08, 0.0)
    │   ├── plane: XY (horizontal)
    │   └── layout: Rectangle
    │
    └── Tilemap (Entity 315) - Child
        ├── Tilemap Component
        │   ├── width: 10, height: 10
        │   ├── tiles: [100 tiles]
        │   └── z_order: 0
        │
        ├── TileSet Component
        │   ├── texture: Cavernas tileset
        │   ├── tile_size: 8x8 pixels
        │   └── columns: 12
        │
        └── TilemapRenderer Component
            ├── mode: Chunk
            ├── chunk_size: 16
            └── detect_chunk_culling: true
```

## Usage

### Creating a Grid with Tilemap

```rust
// 1. Create Grid entity
let grid_entity = world.spawn();
world.transforms.insert(grid_entity, Transform::default());
world.grids.insert(grid_entity, Grid::new());  // or Grid::vertical()
world.names.insert(grid_entity, "Grid".to_string());

// 2. Create Tilemap entity (child of Grid)
let tilemap_entity = world.spawn();
world.transforms.insert(tilemap_entity, Transform::default());
world.set_parent(tilemap_entity, Some(grid_entity));

// 3. Add Tilemap components
world.tilemaps.insert(tilemap_entity, Tilemap::new("Layer 1", "tileset_1", 10, 10));
world.tilesets.insert(tilemap_entity, TileSet::new(...));
world.tilemap_renderers.insert(tilemap_entity, TilemapRenderer::default());
world.names.insert(tilemap_entity, "Tilemap".to_string());
```

### Creating a Vertical Grid (for walls)

```rust
let grid_entity = world.spawn();
world.transforms.insert(grid_entity, Transform::default());
world.grids.insert(grid_entity, Grid::vertical());  // XZ plane
world.names.insert(grid_entity, "Wall Grid".to_string());
```

### Customizing TilemapRenderer

```rust
let renderer = TilemapRenderer::new()
    .with_color(1.0, 0.5, 0.0, 0.8)  // Orange tint
    .with_chunk_culling(true);

world.tilemap_renderers.insert(tilemap_entity, renderer);
```

## Scene File Format

Grid entity (314) in `main.json`:
```json
{
  "transforms": [[314, { "position": [0, 0, 0], ... }]],
  "names": [[314, "Grid"]],
  "grids": [[314, {
    "cell_size": [0.08, 0.08, 0.0],
    "cell_gap": [0.0, 0.0],
    "layout": "Rectangle",
    "swizzle": "XYZ",
    "plane": "XY"
  }]],
  "parents": [[315, 314]],  // Tilemap is child of Grid
  "tilemaps": [[315, { ... }]],
  "tilesets": [[315, { ... }]],
  "tilemap_renderers": [[315, { ... }]]
}
```

## Features

✅ Unity-style Grid component with plane support (XY, XZ, YZ)
✅ Parent-child hierarchy (Grid → Tilemap)
✅ Consistent unit scale (100 PPU)
✅ TilemapRenderer for rendering optimization
✅ Chunk-based rendering with culling
✅ Parallax scrolling support
✅ Z-order sorting
✅ Multiple layout types (Rectangle, Hexagon, Isometric)
✅ Cell swizzle for 3D grids

## Troubleshooting

### Grid not showing in Hierarchy
1. Check if scene is loaded: Look for entity 314 in scene file
2. Reload scene: Close and reopen the scene
3. Check entity_names: Grid should have name "Grid"

### Tilemap not rendering
1. Check TilemapRenderer component exists
2. Verify tileset texture path is correct
3. Check z_order and visible flags
4. Ensure parent-child relationship is set

### Scale issues
1. Verify PPU = 100.0 in all components
2. Check Grid cell_size matches tile size / 100
3. Example: 8px tile → 0.08 world units

## Next Steps

- [ ] Implement Tilemap painting tools in editor
- [ ] Add Tilemap Collider component
- [ ] Support for animated tiles
- [ ] Tile palette UI
- [ ] Brush tools (paint, erase, fill)
- [ ] Layer management UI
