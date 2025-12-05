# Tilemap System Design - Production Ready

## 🎯 Overview

ระบบจัดการ Tilemap แบบ production-ready เหมือน Unity ที่รองรับ:
- Grid system (Cell size, Gap, Layout, Swizzle)
- Tilemap component (Renderer, Collider)
- Tile Palette (Selection, Move, Brush, Paint, Fill)
- Hot-reload และ optimization

## 📐 Architecture

```
Grid (GameObject)
├── Cell Configuration
│   ├── Cell Size (width, height)
│   ├── Cell Gap (spacing)
│   ├── Cell Layout (Rectangle, Hexagon, Isometric)
│   └── Cell Swizzle (XYZ, XZY, YXZ, etc.)
│
└── Tilemap Components (Children)
    ├── Tilemap 1 (Layer)
    │   ├── Tilemap Renderer
    │   ├── Tilemap Collider 2D
    │   └── Tile Data
    │
    ├── Tilemap 2 (Layer)
    └── Tilemap 3 (Layer)
```

## 🔧 Components

### 1. Grid Component

```rust
pub struct Grid {
    /// Cell size (width, height) in world units
    pub cell_size: (f32, f32),
    
    /// Gap between cells
    pub cell_gap: (f32, f32),
    
    /// Grid layout type
    pub layout: GridLayout,
    
    /// Axis swizzle for 3D grids
    pub swizzle: CellSwizzle,
}

pub enum GridLayout {
    Rectangle,
    Hexagon(HexagonOrientation),
    Isometric,
}

pub enum HexagonOrientation {
    FlatTop,
    PointyTop,
}

pub enum CellSwizzle {
    XYZ, // Default (X=right, Y=up, Z=forward)
    XZY, // X=right, Z=up, Y=forward
    YXZ, // Y=right, X=up, Z=forward
    YZX,
    ZXY,
    ZYX,
}
```

### 2. Tilemap Component (Enhanced)

```rust
pub struct Tilemap {
    // Existing fields...
    pub name: String,
    pub tileset_id: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    
    // New fields for production
    /// Rendering mode
    pub render_mode: TilemapRenderMode,
    
    /// Collider generation mode
    pub collider_mode: TilemapColliderMode,
    
    /// Chunk size for optimization
    pub chunk_size: u32,
    
    /// Dirty chunks (need re-render)
    pub dirty_chunks: HashSet<(u32, u32)>,
    
    /// Material/shader override
    pub material: Option<String>,
    
    /// Sorting layer
    pub sorting_layer: String,
    pub sorting_order: i32,
}

pub enum TilemapRenderMode {
    /// Render all tiles individually
    Individual,
    /// Batch render by chunks
    Chunk,
    /// Render to texture (best performance)
    Baked,
}

pub enum TilemapColliderMode {
    /// No collider
    None,
    /// Individual box collider per tile
    Individual,
    /// Composite collider (merged rectangles)
    Composite,
    /// Polygon collider (precise)
    Polygon,
}
```

### 3. Tile Palette System

```rust
pub struct TilePalette {
    /// Tileset reference
    pub tileset: Entity,
    
    /// Selected tiles
    pub selection: TileSelection,
    
    /// Current brush
    pub brush: TileBrush,
    
    /// Brush size
    pub brush_size: (u32, u32),
    
    /// Preview tiles
    pub preview: Vec<Tile>,
}

pub struct TileSelection {
    /// Selected tile IDs
    pub tiles: Vec<u32>,
    
    /// Selection rectangle
    pub rect: Option<(u32, u32, u32, u32)>,
}

pub enum TileBrush {
    /// Single tile brush
    Single,
    
    /// Rectangle brush
    Rectangle,
    
    /// Fill/bucket tool
    Fill,
    
    /// Line brush
    Line,
    
    /// Random brush (picks random tile from selection)
    Random,
    
    /// Pattern brush (repeating pattern)
    Pattern(Vec<Vec<u32>>),
}
```

## 🎨 Editor Features

### Tile Palette Window

```
┌─────────────────────────────┐
│ Tile Palette                │
├─────────────────────────────┤
│ Tileset: [platformer_tiles] │
├─────────────────────────────┤
│ ┌─┬─┬─┬─┬─┬─┬─┬─┐          │
│ │0│1│2│3│4│5│6│7│          │
│ ├─┼─┼─┼─┼─┼─┼─┼─┤          │
│ │8│9│A│B│C│D│E│F│          │
│ └─┴─┴─┴─┴─┴─┴─┴─┘          │
├─────────────────────────────┤
│ Brush: [●] Single           │
│        [ ] Rectangle        │
│        [ ] Fill             │
│        [ ] Random           │
├─────────────────────────────┤
│ Size: [1] x [1]             │
└─────────────────────────────┘
```

### Tilemap Editor Tools

```rust
pub enum TilemapTool {
    /// Select tiles
    Select,
    
    /// Move selection
    Move,
    
    /// Paint tiles
    Paint,
    
    /// Erase tiles
    Erase,
    
    /// Fill area
    Fill,
    
    /// Pick tile (eyedropper)
    Pick,
    
    /// Rectangle select
    RectSelect,
}
```

## 🚀 Implementation Plan

### Phase 1: Core Grid System
- [ ] Grid component
- [ ] Cell layout (Rectangle, Hexagon, Isometric)
- [ ] Cell swizzle
- [ ] Grid-to-world coordinate conversion

### Phase 2: Enhanced Tilemap
- [ ] Render modes (Individual, Chunk, Baked)
- [ ] Collider modes (None, Individual, Composite, Polygon)
- [ ] Chunk system for large maps
- [ ] Dirty tracking for optimization

### Phase 3: Tile Palette
- [ ] Tile selection
- [ ] Brush system
- [ ] Paint/Erase/Fill tools
- [ ] Preview system

### Phase 4: Editor Integration
- [ ] Tile Palette window
- [ ] Tilemap editor tools
- [ ] Keyboard shortcuts
- [ ] Undo/Redo support

### Phase 5: Optimization
- [ ] Chunk-based rendering
- [ ] Frustum culling
- [ ] Texture atlasing
- [ ] Batch rendering

## 📊 Performance Targets

| Feature | Target | Notes |
|---------|--------|-------|
| Map Size | 1000x1000 tiles | With chunking |
| Render Time | < 16ms (60 FPS) | For visible chunks |
| Memory | < 100MB | For 1000x1000 map |
| Paint Tool | < 1ms | Per tile |
| Fill Tool | < 100ms | For 10,000 tiles |

## 🎮 Usage Examples

### Creating a Grid

```rust
// Create grid entity
let grid = world.spawn();
world.names.insert(grid, "Grid".to_string());

// Add grid component
let grid_comp = Grid {
    cell_size: (1.0, 1.0),
    cell_gap: (0.0, 0.0),
    layout: GridLayout::Rectangle,
    swizzle: CellSwizzle::XYZ,
};
world.grids.insert(grid, grid_comp);
```

### Creating a Tilemap Layer

```rust
// Create tilemap entity as child of grid
let tilemap = world.spawn();
world.names.insert(tilemap, "Ground Layer".to_string());
world.parents.insert(tilemap, grid);

// Add tilemap component
let mut tilemap_comp = Tilemap::new("Ground", "tileset_1", 100, 100);
tilemap_comp.render_mode = TilemapRenderMode::Chunk;
tilemap_comp.collider_mode = TilemapColliderMode::Composite;
tilemap_comp.chunk_size = 16;
tilemap_comp.sorting_layer = "Default".to_string();
tilemap_comp.sorting_order = 0;

world.tilemaps.insert(tilemap, tilemap_comp);
```

### Using Tile Palette

```rust
// Create palette
let mut palette = TilePalette::new(tileset_entity);

// Select tiles
palette.selection.tiles = vec![1, 2, 3];
palette.brush = TileBrush::Rectangle;
palette.brush_size = (3, 3);

// Paint tiles
palette.paint(&mut tilemap, mouse_x, mouse_y);

// Fill area
palette.fill(&mut tilemap, mouse_x, mouse_y);
```

### Lua API

```lua
-- Create grid
local grid = GameObject.new("Grid")
local grid_comp = grid:add_component("Grid")
grid_comp.cell_size = {1.0, 1.0}
grid_comp.layout = "Rectangle"

-- Create tilemap layer
local tilemap = GameObject.new("Ground Layer")
tilemap:set_parent(grid)
local tilemap_comp = tilemap:add_component("Tilemap")
tilemap_comp.width = 100
tilemap_comp.height = 100
tilemap_comp.render_mode = "Chunk"
tilemap_comp.collider_mode = "Composite"

-- Paint tiles
local palette = TilePalette.new(tileset)
palette:select_tile(5)
palette:paint(tilemap, 10, 10)
```

## 🔍 Technical Details

### Chunk System

```rust
pub struct TilemapChunkSystem {
    /// Chunk size (tiles per chunk)
    chunk_size: u32,
    
    /// Active chunks (visible)
    active_chunks: HashMap<(u32, u32), TilemapChunk>,
    
    /// Chunk cache (for quick reload)
    chunk_cache: HashMap<(u32, u32), TilemapChunk>,
}

impl TilemapChunkSystem {
    /// Get chunk at position
    pub fn get_chunk(&self, chunk_x: u32, chunk_y: u32) -> Option<&TilemapChunk>;
    
    /// Update chunk (mark as dirty)
    pub fn update_chunk(&mut self, chunk_x: u32, chunk_y: u32);
    
    /// Render visible chunks
    pub fn render_visible_chunks(&self, camera: &Camera);
    
    /// Cull invisible chunks
    pub fn cull_chunks(&mut self, camera: &Camera);
}
```

### Collider Generation

```rust
pub struct TilemapColliderGenerator {
    /// Generate individual colliders
    pub fn generate_individual(tilemap: &Tilemap) -> Vec<Collider>;
    
    /// Generate composite collider (merged rectangles)
    pub fn generate_composite(tilemap: &Tilemap) -> Vec<Collider>;
    
    /// Generate polygon collider
    pub fn generate_polygon(tilemap: &Tilemap) -> Vec<Collider>;
}
```

## 📚 References

- Unity Tilemap: https://docs.unity3d.com/Manual/class-Tilemap.html
- Godot TileMap: https://docs.godotengine.org/en/stable/classes/class_tilemap.html
- LDtk: https://ldtk.io/
- Tiled: https://www.mapeditor.org/

## 🎯 Next Steps

1. Implement Grid component
2. Enhance Tilemap component
3. Create Tile Palette system
4. Build editor tools
5. Optimize rendering
6. Add Lua bindings
7. Write documentation
8. Create examples

