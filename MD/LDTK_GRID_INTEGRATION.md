# LDtk + Grid System Integration

## 🎯 Overview

Grid component ทำงานร่วมกับ LDtk โดยอัตโนมัติ โดย LDtk loader จะอ่านค่า grid settings จาก LDtk project และสร้าง Grid component ให้

## 🔄 Integration Flow

```
LDtk Project (.ldtk)
    ↓
LdtkLoader.load_project()
    ↓
1. อ่าน defaultGridSize จาก LDtk
2. สร้าง Grid entity (parent)
3. สร้าง Tilemap entities (children)
4. ตั้งค่า Transform ตาม Grid
    ↓
Grid + Tilemaps ใน World
```

## 📐 LDtk Grid Settings

### LDtk Project Structure

```json
{
  "defaultGridSize": 8,
  "defaultPivotX": 0,
  "defaultPivotY": 0,
  "levels": [
    {
      "worldX": 0,
      "worldY": 0,
      "layerInstances": [
        {
          "__gridSize": 8,
          "__cWid": 37,
          "__cHei": 26,
          "gridTiles": [...]
        }
      ]
    }
  ]
}
```

### Mapping to Grid Component

| LDtk | Grid Component |
|------|----------------|
| `defaultGridSize` | `cell_size` |
| `worldX`, `worldY` | `Transform.position` |
| `__gridSize` | `cell_size` (per layer) |
| Rectangle layout | `GridLayout::Rectangle` |

## 🔧 Enhanced LDtk Loader

ให้ผมแก้ LDtk loader เพื่อสร้าง Grid component:

```rust
impl LdtkLoader {
    /// Load LDtk project with Grid support
    pub fn load_project_with_grid(
        path: impl AsRef<Path>, 
        world: &mut World
    ) -> Result<(Entity, Vec<Entity>), String> {
        // Load project JSON
        let project_data = std::fs::read_to_string(path.as_ref())?;
        let project: Value = serde_json::from_str(&project_data)?;
        
        // Create Grid entity (parent)
        let grid_entity = world.spawn();
        world.names.insert(grid_entity, "LDtk Grid".to_string());
        
        // Get grid size from LDtk
        let grid_size = project["defaultGridSize"]
            .as_i64()
            .unwrap_or(8) as f32;
        
        // Create Grid component
        let grid = Grid {
            cell_size: (grid_size / 8.0, grid_size / 8.0), // Convert to world units
            cell_gap: (0.0, 0.0),
            layout: GridLayout::Rectangle,
            swizzle: CellSwizzle::XYZ,
        };
        world.grids.insert(grid_entity, grid);
        
        // Position grid
        world.transforms.insert(
            grid_entity, 
            Transform::with_position(0.0, 0.0, 0.0)
        );
        
        // Load levels as children
        let mut tilemap_entities = Vec::new();
        let levels = project["levels"].as_array()?;
        
        for level in levels {
            let entities = Self::load_level(level, world, grid_entity)?;
            tilemap_entities.extend(entities);
        }
        
        Ok((grid_entity, tilemap_entities))
    }
    
    /// Load a single level as Grid children
    fn load_level(
        level: &Value,
        world: &mut World,
        grid_parent: Entity,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();
        
        // Get level position
        let level_x = level["worldX"].as_i64().unwrap_or(0) as f32;
        let level_y = level["worldY"].as_i64().unwrap_or(0) as f32;
        
        // Load layers
        let layers = level["layerInstances"].as_array()?;
        
        for layer in layers {
            let entity = world.spawn();
            
            // Set parent to Grid
            world.set_parent(entity, Some(grid_parent));
            
            // Create tilemap...
            let tilemap = Tilemap::new(...);
            world.tilemaps.insert(entity, tilemap);
            
            // Position relative to Grid
            let transform = Transform::with_position(
                level_x / 8.0,
                -level_y / 8.0,
                0.0,
            );
            world.transforms.insert(entity, transform);
            
            entities.push(entity);
        }
        
        Ok(entities)
    }
}
```

## 🎮 Usage Examples

### Load LDtk with Grid

```rust
use ecs::{World, loaders::LdtkLoader};

let mut world = World::new();

// Load with Grid support
let (grid, tilemaps) = LdtkLoader::load_project_with_grid(
    "levels/Level_01.ldtk",
    &mut world
)?;

println!("Grid entity: {}", grid);
println!("Loaded {} tilemap layers", tilemaps.len());

// Access Grid component
if let Some(grid_comp) = world.grids.get(&grid) {
    println!("Cell size: {:?}", grid_comp.cell_size);
    println!("Layout: {:?}", grid_comp.layout);
}
```

### Convert LDtk Coordinates

```rust
// Get Grid component
let grid = world.grids.get(&grid_entity).unwrap();

// LDtk uses pixel coordinates
let ldtk_x = 64; // pixels
let ldtk_y = 32; // pixels

// Convert to cell coordinates
let cell_x = ldtk_x / 8; // 8 = grid size
let cell_y = ldtk_y / 8;

// Convert to world coordinates using Grid
let (world_x, world_y) = grid.cell_to_world(cell_x, cell_y);

println!("LDtk ({}, {}) -> Cell ({}, {}) -> World ({}, {})",
    ldtk_x, ldtk_y, cell_x, cell_y, world_x, world_y);
```

### Hot-Reload with Grid

```rust
use engine::runtime::LdtkRuntime;

let mut ldtk = LdtkRuntime::new();

// Load with Grid
ldtk.load_with_grid("levels/Level_01.ldtk", &mut world)?;

// Game loop
loop {
    if ldtk.update(&mut world) {
        println!("Map reloaded with Grid!");
        
        // Grid entity is preserved
        // Tilemap children are recreated
    }
}
```

## 🔍 LDtk Grid Features

### Supported

✅ Rectangle grid  
✅ Grid size (cell size)  
✅ Level positioning (worldX, worldY)  
✅ Layer offsets  
✅ Multiple levels  
✅ Hot-reload  

### Planned

🔲 Hexagon grid (if LDtk adds support)  
🔲 Isometric grid (custom layout)  
🔲 Grid pivot point  
🔲 Grid rotation  

## 📊 Coordinate Systems

### LDtk Coordinates

```
Origin: Top-Left
X: Right →
Y: Down ↓
Units: Pixels
```

### Grid Coordinates

```
Origin: Bottom-Left (Unity-style)
X: Right →
Y: Up ↑
Units: Cells
```

### World Coordinates

```
Origin: Center (0, 0)
X: Right →
Y: Up ↑
Units: World units (meters)
```

### Conversion Formula

```rust
// LDtk pixel → Cell
cell_x = ldtk_x / grid_size
cell_y = ldtk_y / grid_size

// Cell → World (Rectangle grid)
world_x = cell_x * cell_size.0
world_y = -cell_y * cell_size.1  // Flip Y axis

// World → Cell
cell_x = floor(world_x / cell_size.0)
cell_y = floor(-world_y / cell_size.1)  // Flip Y axis
```

## 🎨 Editor Integration

### Hierarchy View

```
Scene
├── LDtk Grid (Grid component)
│   ├── Level_01 (Transform)
│   │   ├── IntGrid_layer (Tilemap)
│   │   ├── Tiles (Tilemap)
│   │   └── Entities (Tilemap)
│   └── Level_02 (Transform)
│       ├── Ground (Tilemap)
│       └── Walls (Tilemap)
└── Player
```

### Inspector View

```
Grid Component
├── Cell Size: (1.0, 1.0)
├── Cell Gap: (0.0, 0.0)
├── Layout: Rectangle
└── Swizzle: XYZ

Tilemap Component
├── Name: "IntGrid_layer"
├── Tileset: "tileset_72"
├── Size: 37 x 26
├── Parent: LDtk Grid
└── Transform: (0, 0, 0)
```

## 🚀 Performance

### Grid Benefits

- **Spatial Queries**: ใช้ Grid แทน pixel coordinates
- **Collision Detection**: ตรวจสอบเฉพาะ cells ใกล้เคียง
- **Chunk Culling**: แบ่ง Grid เป็น chunks
- **Pathfinding**: ใช้ Grid-based A*

### Example: Spatial Query

```rust
// Find entities in cell
fn get_entities_in_cell(
    world: &World,
    grid: &Grid,
    cell_x: i32,
    cell_y: i32,
) -> Vec<Entity> {
    let (world_x, world_y) = grid.cell_to_world(cell_x, cell_y);
    let cell_size = grid.cell_size;
    
    world.transforms.iter()
        .filter(|(_, transform)| {
            let x = transform.position[0];
            let y = transform.position[1];
            
            x >= world_x && x < world_x + cell_size.0 &&
            y >= world_y && y < world_y + cell_size.1
        })
        .map(|(entity, _)| *entity)
        .collect()
}
```

## 🔧 Advanced Usage

### Custom Grid Layout for LDtk

```rust
// Load LDtk with custom grid
let (grid, tilemaps) = LdtkLoader::load_project_with_grid(
    "levels/isometric.ldtk",
    &mut world
)?;

// Override to Isometric
if let Some(grid_comp) = world.grids.get_mut(&grid) {
    grid_comp.layout = GridLayout::Isometric;
}
```

### Multiple LDtk Files

```rust
// Load multiple LDtk files, each with own Grid
let (grid1, _) = LdtkLoader::load_project_with_grid(
    "levels/world_1.ldtk",
    &mut world
)?;

let (grid2, _) = LdtkLoader::load_project_with_grid(
    "levels/world_2.ldtk",
    &mut world
)?;

// Position grids
world.transforms.get_mut(&grid1).unwrap()
    .set_position(0.0, 0.0, 0.0);

world.transforms.get_mut(&grid2).unwrap()
    .set_position(100.0, 0.0, 0.0);
```

## 📚 References

- [LDtk JSON Format](https://ldtk.io/json/)
- [Unity Grid](https://docs.unity3d.com/Manual/class-Grid.html)
- [Godot TileMap](https://docs.godotengine.org/en/stable/classes/class_tilemap.html)

## ✅ Checklist

- [x] Grid component
- [x] LDtk integration design
- [ ] Implement `load_project_with_grid()`
- [ ] Update hot-reload to preserve Grid
- [ ] Add Grid to editor UI
- [ ] Add coordinate conversion helpers
- [ ] Add spatial query system
- [ ] Write tests
- [ ] Update documentation

