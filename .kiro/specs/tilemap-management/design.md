# Design Document

## Overview

The Tilemap Management System is a comprehensive editor feature that provides professional-grade tools for managing LDtk tilemap files. The system follows a hierarchical entity-component architecture where each loaded map creates a Grid Entity as the root, with tilemap layers and colliders as children. This design ensures clean organization, automatic cleanup, and efficient memory management.

The system integrates seamlessly with the existing ECS (Entity Component System) architecture and provides multiple UI panels for different aspects of tilemap management: the Maps Panel for file operations, the Layer Properties Panel for detailed editing, and the Layer Ordering Panel for visual reordering.

Key design principles:
- **Automatic Management**: Colliders are generated automatically when maps load
- **Hierarchical Organization**: Grid → Layers + Colliders parent-child structure
- **Hot-Reload Support**: Automatic detection and reloading of changed files
- **Performance Optimization**: Composite colliders merge adjacent tiles
- **Clean Separation**: UI, business logic, and ECS components are clearly separated

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Editor Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Maps Panel   │  │ Layer Props  │  │ Layer Order  │      │
│  │              │  │ Panel        │  │ Panel        │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
│                    ┌───────▼────────┐                        │
│                    │  MapManager    │                        │
│                    │  (Business     │                        │
│                    │   Logic)       │                        │
│                    └───────┬────────┘                        │
└────────────────────────────┼──────────────────────────────────┘
                             │
┌────────────────────────────▼──────────────────────────────────┐
│                      ECS Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ LdtkLoader   │  │ World        │  │ Components   │       │
│  │              │  │              │  │ - Grid       │       │
│  │              │  │              │  │ - Tilemap    │       │
│  │              │  │              │  │ - Transform  │       │
│  │              │  │              │  │ - Collider   │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└───────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Action (UI)
    ↓
MapManager (validates, coordinates)
    ↓
LdtkLoader (loads file, creates entities)
    ↓
World (stores components)
    ↓
UI Update (reflects changes)
```

### Entity Hierarchy

```
Grid Entity (root)
├── Transform Component
├── Grid Component
├── Name: "LDtk Grid"
│
├── Layer Entity 1 (child)
│   ├── Transform Component
│   ├── Tilemap Component
│   ├── TileSet Component
│   └── Name: "LDTK Layer: IntGrid_layer"
│
├── Layer Entity 2 (child)
│   ├── Transform Component
│   ├── Tilemap Component
│   ├── TileSet Component
│   └── Name: "LDTK Layer: Tiles"
│
└── Collider Entities (children)
    ├── Transform Component
    ├── Collider Component
    ├── Rigidbody2D Component
    └── Name: "CompositeCollider_4x2"
```

## Components and Interfaces

### Core Components

#### Grid Component
```rust
pub struct Grid {
    pub cell_size: (f32, f32),      // Cell size in world units
    pub cell_gap: (f32, f32),       // Gap between cells
    pub layout: GridLayout,          // Rectangle, Hexagon, Isometric
    pub swizzle: CellSwizzle,       // Axis mapping for 3D
}

impl Grid {
    pub fn cell_to_world(&self, cell_x: i32, cell_y: i32) -> (f32, f32);
    pub fn world_to_cell(&self, world_x: f32, world_y: f32) -> (i32, i32);
    pub fn get_cell_center(&self, cell_x: i32, cell_y: i32) -> (f32, f32);
}
```

#### Tilemap Component
```rust
pub struct Tilemap {
    pub name: String,
    pub tileset_id: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub z_order: i32,
}

impl Tilemap {
    pub fn new(name: &str, tileset_id: String, width: u32, height: u32) -> Self;
    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) -> bool;
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile>;
}
```

#### Collider Component
```rust
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub offset: (f32, f32),
    pub is_trigger: bool,
}

impl Collider {
    pub fn new(width: f32, height: f32) -> Self;
}
```

### Manager Classes

#### MapManager
```rust
pub struct MapManager {
    pub loaded_maps: HashMap<PathBuf, LoadedMap>,
    pub available_files: Vec<PathBuf>,
    pub selected_map: Option<PathBuf>,
    pub project_path: Option<PathBuf>,
}

impl MapManager {
    pub fn new() -> Self;
    pub fn scan_ldtk_files(&mut self);
    pub fn load_map(&mut self, path: &PathBuf, world: &mut World) -> Result<(), String>;
    pub fn reload_map(&mut self, path: &PathBuf, world: &mut World) -> Result<(), String>;
    pub fn unload_map(&mut self, path: &PathBuf, world: &mut World);
    pub fn regenerate_colliders(&mut self, path: &PathBuf, world: &mut World) -> Result<usize, String>;
    pub fn toggle_layer_visibility(&mut self, entity: Entity, world: &mut World);
    pub fn is_map_entity(&self, entity: Entity) -> bool;
}
```

#### LoadedMap
```rust
pub struct LoadedMap {
    pub grid_entity: Entity,
    pub layer_entities: Vec<LayerInfo>,
    pub collider_entities: Vec<Entity>,
    pub file_path: PathBuf,
    pub last_modified: SystemTime,
}
```

#### LayerInfo
```rust
pub struct LayerInfo {
    pub entity: Entity,
    pub name: String,
    pub size: (u32, u32),
    pub visible: bool,
    pub z_order: i32,
}
```

### Loader Interface

#### LdtkLoader
```rust
pub struct LdtkLoader;

impl LdtkLoader {
    // Load with Grid and auto-generated colliders
    pub fn load_project_with_grid_and_colliders(
        path: impl AsRef<Path>,
        world: &mut World,
        auto_generate_colliders: bool,
        collision_value: i64,
    ) -> Result<(Entity, Vec<Entity>, Vec<Entity>), String>;
    
    // Load with Grid only
    pub fn load_project_with_grid(
        path: impl AsRef<Path>,
        world: &mut World,
    ) -> Result<(Entity, Vec<Entity>), String>;
    
    // Generate composite colliders
    pub fn generate_composite_colliders_from_intgrid(
        path: impl AsRef<Path>,
        world: &mut World,
        collision_value: i64,
    ) -> Result<Vec<Entity>, String>;
}
```

## Data Models

### File Format

The system works with LDtk JSON files (version 1.5.3+):

```json
{
  "defaultGridSize": 8,
  "levels": [
    {
      "identifier": "Level_01",
      "worldX": 0,
      "worldY": 0,
      "layerInstances": [
        {
          "__identifier": "IntGrid_layer",
          "__type": "IntGrid",
          "__cWid": 42,
          "__cHei": 26,
          "__gridSize": 8,
          "__pxTotalOffsetX": 0,
          "__pxTotalOffsetY": 0,
          "intGridCsv": [0, 0, 1, 1, 0, ...],
          "autoLayerTiles": [...]
        }
      ]
    }
  ],
  "defs": {
    "tilesets": [...]
  }
}
```

### Internal Data Structures

#### Rectangle (for collider optimization)
```rust
struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
```

Used by the greedy meshing algorithm to merge adjacent tiles into larger colliders.

### Coordinate Systems

The system handles multiple coordinate systems:

1. **LDtk Pixel Coordinates**: Top-left origin, pixels
2. **Grid Cell Coordinates**: Integer grid positions
3. **World Coordinates**: Bottom-left origin, world units (8 pixels = 1 unit)

Conversion formula:
```rust
let pixels_per_unit = 8.0;
let world_x = pixel_x / pixels_per_unit;
let world_y = -pixel_y / pixels_per_unit;  // Flip Y axis
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Map Loading Creates Hierarchy
*For any* valid .ldtk file, loading the file should create exactly one Grid Entity with all layers as children.
**Validates: Requirements 1.2, 7.1, 7.2**

### Property 2: Reload Preserves Grid Entity
*For any* loaded map, reloading the map should preserve the Grid Entity reference (same entity ID).
**Validates: Requirements 1.3**

### Property 3: Unload Cleans Up Hierarchy
*For any* loaded map, unloading should despawn the Grid Entity and all its children (layers and colliders).
**Validates: Requirements 1.4, 2.5**

### Property 4: Multiple Maps Display Correctly
*For any* N loaded maps, the Maps Panel should display all N file names with status indicators.
**Validates: Requirements 1.5**

### Property 5: Auto-Generate Colliders
*For any* map with IntGrid layers containing collision value 1, loading should automatically generate colliders for those tiles.
**Validates: Requirements 2.1**

### Property 6: Colliders Are Grid Children
*For any* generated colliders, all colliders should have the Grid Entity as their parent.
**Validates: Requirements 2.2, 7.3**

### Property 7: Regenerate Colliders Round Trip
*For any* map with colliders, regenerating colliders should produce colliders that match the current IntGrid data.
**Validates: Requirements 2.3**

### Property 8: Composite Collider Optimization
*For any* pattern of N adjacent tiles, composite collider generation should produce fewer than N colliders.
**Validates: Requirements 2.4**

### Property 9: Visibility Toggle Idempotence
*For any* layer, toggling visibility twice should return to the original state.
**Validates: Requirements 3.1**

### Property 10: Visibility State Synchronization
*For any* layer, hiding the layer should set active=false, and showing should set active=true.
**Validates: Requirements 3.2, 3.3**

### Property 11: Visibility Persists Across Reload
*For any* map with custom visibility states, reloading should preserve those states.
**Validates: Requirements 3.5, 6.3**

### Property 12: Layer Properties Synchronization
*For any* layer, selecting the layer should display correct current property values in the Layer Properties Panel.
**Validates: Requirements 4.1**

### Property 13: Transform Updates Apply Immediately
*For any* layer, modifying transform properties should update the entity's transform component.
**Validates: Requirements 4.2**

### Property 14: Z-Order Affects Rendering
*For any* two layers, the layer with higher Z-Order should render on top.
**Validates: Requirements 4.3**

### Property 15: Reset Restores Defaults
*For any* modified transform property, clicking Reset should restore the default value.
**Validates: Requirements 4.5**

### Property 16: Layer Reordering Updates Z-Orders
*For any* layer moved from position A to position B, all Z-Order values should be updated to reflect the new ordering.
**Validates: Requirements 5.2**

### Property 17: Z-Order Monotonic Increase
*For any* reordered layer list, Z-Order values should be monotonically increasing from bottom to top.
**Validates: Requirements 5.3**

### Property 18: Move Up Increments Z-Order
*For any* layer, clicking "Move Up" should increment Z-Order by exactly 1.
**Validates: Requirements 5.4**

### Property 19: Move Down Decrements With Bounds
*For any* layer, clicking "Move Down" should decrement Z-Order by 1, but never below -100.
**Validates: Requirements 5.5**

### Property 20: Hot-Reload Regenerates Entities
*For any* modified .ldtk file, hot-reload should regenerate all entities from the new file data.
**Validates: Requirements 6.2**

### Property 21: Hot-Reload Regenerates Colliders
*For any* hot-reloaded map, colliders should be regenerated automatically.
**Validates: Requirements 6.4**

### Property 22: Hot-Reload Error Recovery
*For any* corrupted file during hot-reload, the system should preserve the last valid state and display an error.
**Validates: Requirements 6.5**

### Property 23: Hierarchy Filtering
*For any* loaded map, the Hierarchy Panel should show the Grid and layers but hide collider entities.
**Validates: Requirements 7.4**

### Property 24: Performance Metrics Display
*For any* loaded maps, the Performance Panel should display non-negative draw call, triangle, and vertex counts.
**Validates: Requirements 8.1, 8.2**

### Property 25: Memory Usage Increases With Maps
*For any* N loaded maps, memory usage should be greater than or equal to memory usage with N-1 maps.
**Validates: Requirements 8.3**

### Property 26: Performance Warning Indicators
*For any* performance metrics exceeding thresholds, visual warning indicators should appear.
**Validates: Requirements 8.5**

### Property 27: Collider Settings Display
*For any* map, opening Collider Settings should display the current configuration values.
**Validates: Requirements 9.1**

### Property 28: Composite Type Merges Tiles
*For any* adjacent tiles with Composite collider type, the number of colliders should be less than the number of tiles.
**Validates: Requirements 9.2**

### Property 29: Individual Type One Per Tile
*For any* solid tiles with Individual collider type, the number of colliders should equal the number of tiles.
**Validates: Requirements 9.3**

### Property 30: Collision Value Filtering
*For any* collision value V, generated colliders should only exist for IntGrid tiles with value V.
**Validates: Requirements 9.4**

### Property 31: Auto-Regenerate On Reload
*For any* map with "Auto-regenerate on reload" enabled, reloading should automatically regenerate colliders.
**Validates: Requirements 9.5**

### Property 32: Invalid File Error Handling
*For any* invalid .ldtk file, attempting to load should display an error message without crashing.
**Validates: Requirements 11.1**

### Property 33: Missing File Error Handling
*For any* non-existent file path, attempting to load should display a "file not found" error message.
**Validates: Requirements 11.2**

### Property 34: Collider Generation Error Recovery
*For any* collider generation failure, the system should maintain the previous collider state.
**Validates: Requirements 11.3**

### Property 35: Error Logging
*For any* error condition, detailed error information should be logged to the console.
**Validates: Requirements 11.5**

### Property 36: Visibility Persistence
*For any* modified layer visibility, saving and reopening the scene should restore the visibility state.
**Validates: Requirements 12.1, 12.4**

### Property 37: Z-Order Persistence
*For any* modified layer Z-Order, saving and reopening the scene should restore the Z-Order value.
**Validates: Requirements 12.2, 12.4**

### Property 38: Transform Persistence
*For any* modified layer transform, saving and reopening the scene should restore the transform values.
**Validates: Requirements 12.3, 12.4**

### Property 39: Collider Configuration Persistence
*For any* modified collider configuration, the configuration should be saved to project settings.
**Validates: Requirements 12.5**

## Error Handling

### Error Categories

1. **File Errors**
   - File not found
   - Invalid JSON format
   - Unsupported LDtk version
   - File access denied

2. **Validation Errors**
   - Missing required fields
   - Invalid layer data
   - Corrupted tileset references

3. **Runtime Errors**
   - Entity not found
   - Component missing
   - Memory allocation failure

### Error Handling Strategy

```rust
pub enum TilemapError {
    FileNotFound(PathBuf),
    InvalidFormat(String),
    ValidationError(String),
    EntityNotFound(Entity),
    ComponentMissing(String),
}

impl TilemapError {
    pub fn display_message(&self) -> String {
        match self {
            TilemapError::FileNotFound(path) => {
                format!("File not found: {:?}", path)
            }
            TilemapError::InvalidFormat(msg) => {
                format!("Invalid file format: {}", msg)
            }
            // ... other cases
        }
    }
    
    pub fn log_error(&self) {
        log::error!("{}", self.display_message());
    }
}
```

### Recovery Mechanisms

1. **Graceful Degradation**: If collider generation fails, continue with map loading
2. **State Preservation**: On hot-reload failure, maintain the last valid state
3. **User Notification**: Display error messages in UI with actionable information
4. **Detailed Logging**: Log full error context for debugging

## Testing Strategy

### Unit Testing

Unit tests verify specific functionality of individual components:

1. **Grid Component Tests**
   - Cell to world coordinate conversion
   - World to cell coordinate conversion
   - Different grid layouts (Rectangle, Hexagon, Isometric)

2. **Collider Optimization Tests**
   - Rectangle finding algorithm
   - Greedy meshing correctness
   - Edge cases (single tile, full grid, checkerboard pattern)

3. **MapManager Tests**
   - File scanning
   - Entity tracking
   - Cleanup operations

### Property-Based Testing

Property-based tests verify universal properties across many random inputs using the **QuickCheck** library for Rust.

**Configuration**: Each property test should run a minimum of 100 iterations.

**Test Tagging**: Each property-based test must be tagged with a comment explicitly referencing the correctness property:
```rust
// Feature: tilemap-management, Property 1: Map Loading Creates Hierarchy
#[quickcheck]
fn prop_map_loading_creates_hierarchy(/* ... */) -> bool {
    // ...
}
```

**Key Properties to Test**:

1. **Hierarchy Properties** (Properties 1-3, 6, 23)
   - Map loading creates correct hierarchy
   - Reload preserves Grid Entity
   - Unload cleans up all children
   - Colliders are Grid children
   - Hierarchy filtering works correctly

2. **Collider Properties** (Properties 5, 7, 8, 28-31)
   - Auto-generation from IntGrid
   - Regeneration produces correct results
   - Composite optimization reduces count
   - Collision value filtering

3. **State Management Properties** (Properties 9-11, 36-39)
   - Visibility toggle idempotence
   - State synchronization
   - Persistence across reload/save

4. **Ordering Properties** (Properties 16-19)
   - Reordering updates Z-Orders
   - Z-Order monotonic increase
   - Move up/down operations

5. **Error Handling Properties** (Properties 32-35)
   - Invalid file handling
   - Missing file handling
   - Error recovery
   - Error logging

### Integration Testing

Integration tests verify the interaction between components:

1. **End-to-End Map Loading**
   - Load map → Verify hierarchy → Verify colliders → Verify UI update

2. **Hot-Reload Workflow**
   - Load map → Modify file → Detect change → Reload → Verify state

3. **Layer Management Workflow**
   - Load map → Toggle visibility → Reorder layers → Verify rendering

### Performance Testing

Performance tests verify the system meets performance requirements:

1. **Load Time**: Maps up to 100x100 tiles load within 1 second
2. **Collider Generation**: 1000 tiles generate colliders within 500ms
3. **Frame Rate**: 10 loaded maps maintain 60 FPS
4. **Memory Usage**: Track memory growth with multiple maps

### Test Utilities

```rust
// Test helper for creating mock LDtk data
pub fn create_test_ldtk_file(
    width: u32,
    height: u32,
    intgrid_pattern: Vec<i64>,
) -> PathBuf {
    // Generate temporary .ldtk file
}

// Test helper for verifying hierarchy
pub fn verify_hierarchy(
    world: &World,
    grid_entity: Entity,
    expected_layer_count: usize,
    expected_collider_count: usize,
) -> bool {
    // Verify parent-child relationships
}

// Test helper for generating random IntGrid patterns
pub fn generate_random_intgrid(
    width: u32,
    height: u32,
    density: f32,
) -> Vec<i64> {
    // Generate random collision data
}
```

## Performance Considerations

### Optimization Strategies

1. **Composite Colliders**: Merge adjacent tiles to reduce collider count by 70-90%
2. **Lazy Loading**: Only load visible layers initially
3. **Chunk-Based Rendering**: Divide large tilemaps into renderable chunks
4. **Dirty Flags**: Only update changed components
5. **Entity Pooling**: Reuse entities during reload instead of despawn/spawn

### Memory Management

1. **Automatic Cleanup**: Despawning Grid Entity automatically cleans up all children
2. **Texture Sharing**: Multiple tilemaps share the same tileset textures
3. **Sparse Storage**: Only store non-empty tiles in tilemap data

### Profiling Points

Key areas to monitor:
- Map loading time
- Collider generation time
- UI update frequency
- Memory usage per map
- Frame time with multiple maps

## Future Enhancements

### Phase 2 Enhancements

1. **Layer Groups**: Group related layers for batch operations
2. **Blend Modes**: Support different rendering blend modes
3. **Layer Effects**: Drop shadow, glow, outline effects
4. **Multi-Selection**: Select and modify multiple layers at once

### Phase 3 Enhancements

1. **Undo/Redo**: Full undo/redo support for all operations
2. **Keyboard Shortcuts**: Comprehensive keyboard shortcut system
3. **Context Menus**: Right-click menus for quick actions
4. **Drag & Drop**: Drag .ldtk files from file explorer

### Phase 4 Enhancements

1. **LOD System**: Level of detail for large maps
2. **Streaming**: Stream large maps in chunks
3. **Prefab System**: Save map configurations as prefabs
4. **Animation**: Animated tile support

## Dependencies

### External Libraries

- **serde_json**: JSON parsing for LDtk files
- **notify**: File system watching for hot-reload
- **egui**: UI framework for editor panels
- **quickcheck**: Property-based testing library

### Internal Dependencies

- **ecs**: Entity Component System
- **render**: Rendering system for tilemaps
- **physics**: Physics system for colliders
- **editor**: Editor framework

## Deployment Considerations

### File Organization

```
project/
├── levels/              # LDtk files
│   ├── Level_01.ldtk
│   └── Level_02.ldtk
├── tilesets/            # Tileset images
│   └── platformer.png
└── .kiro/
    └── settings/
        └── tilemap.json # Tilemap settings
```

### Configuration

Tilemap settings stored in `.kiro/settings/tilemap.json`:
```json
{
  "auto_generate_colliders": true,
  "collision_value": 1,
  "collider_type": "Composite",
  "hot_reload_enabled": true,
  "pixels_per_unit": 8.0
}
```

### Platform Considerations

- **Windows**: Use backslash path separators
- **macOS/Linux**: Use forward slash path separators
- **File Watching**: Platform-specific file system events
- **Performance**: Adjust chunk size based on platform capabilities
