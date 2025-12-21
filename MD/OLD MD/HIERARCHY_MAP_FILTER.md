# Hierarchy Map Entity Filter

## 🎯 Problem

เมื่อ load LDtk map ใน Hierarchy จะแสดง:
- Grid entity
- Tilemap layers (IntGrid_layer, Tiles, Entities)
- Colliders หลายสิบ/ร้อย entities (CompositeCollider_1x1, CompositeCollider_2x3, etc.)

ทำให้ Hierarchy รก ยากต่อการหา GameObject อื่นๆ

## ✅ Solution

แยก Map entities ออกจาก Hierarchy และแสดงใน Maps panel แทน

### Before (รก)
```
Hierarchy
├── Main Camera
├── Player
├── LDtk Grid                    ← Map entity
│   └── LDTK Layer: IntGrid_layer ← Tilemap
├── CompositeCollider_37x1        ← Collider
├── CompositeCollider_2x10        ← Collider
├── CompositeCollider_3x9         ← Collider
├── CompositeCollider_6x9         ← Collider
├── ... (28 more colliders)
```

### After (สะอาด)
```
Hierarchy
├── Main Camera
├── Player

Maps Panel
└── 🎯 Level_01.ldtk
    └── 📐 LDtk Grid
        ├── 🎨 IntGrid_layer (42x26)
        └── 28 colliders
```

## 🔧 Implementation

### 1. Filter Function

```rust
fn is_map_entity(
    entity: Entity,
    world: &World,
    map_manager: &MapManager,
) -> bool {
    // Check if it's a Grid entity
    if world.grids.contains_key(&entity) {
        return true;
    }
    
    // Check if name starts with map-related prefixes
    if let Some(name) = world.names.get(&entity) {
        if name.starts_with("LDtk Grid") 
            || name.starts_with("LDTK Layer:") 
            || name.starts_with("CompositeCollider")
            || name.starts_with("Collider_") 
        {
            return true;
        }
    }
    
    // Check if it's tracked by map_manager
    map_manager.is_map_entity(entity)
}
```

### 2. Filter in Hierarchy

```rust
// Filter roots
let mut roots: Vec<Entity> = entity_names.keys()
    .filter(|&e| {
        // Filter out entities with parent
        if world.parents.get(e).is_some() {
            return false;
        }
        
        // Filter out map entities
        if let Some(manager) = map_manager {
            if is_map_entity(*e, world, manager) {
                return false;
            }
        }
        
        true
    })
    .cloned()
    .collect();
```

### 3. Filter Children

```rust
// Draw children
for &child in children {
    // Skip map entities
    if let Some(manager) = map_manager {
        if is_map_entity(child, world, manager) {
            continue;
        }
    }
    
    draw_entity_node(ui, child, world, ...);
}
```

## 🎨 Map Entities Detection

### Grid Entity
```rust
world.grids.contains_key(&entity)
```

### Tilemap Layer
```rust
name.starts_with("LDTK Layer:")
// Examples:
// - "LDTK Layer: IntGrid_layer"
// - "LDTK Layer: Tiles"
// - "LDTK Layer: Entities"
```

### Colliders
```rust
name.starts_with("CompositeCollider") || name.starts_with("Collider_")
// Examples:
// - "CompositeCollider_37x1"
// - "CompositeCollider_2x10"
// - "Collider_5_3"
```

### Grid Name
```rust
name.starts_with("LDtk Grid")
// Example: "LDtk Grid"
```

### Tracked by MapManager
```rust
map_manager.is_map_entity(entity)
// Checks if entity is in:
// - loaded_maps[].grid_entity
// - loaded_maps[].layer_entities
// - loaded_maps[].collider_entities
```

## 📊 Benefits

✅ **Clean Hierarchy** - แสดงเฉพาะ GameObjects ที่สำคัญ  
✅ **Easy Navigation** - หา Player, Camera ง่ายขึ้น  
✅ **Organized Maps** - Map entities อยู่ใน Maps panel  
✅ **Better Performance** - Hierarchy render เร็วขึ้น (น้อย entities)  
✅ **Clear Separation** - แยก Scene objects กับ Map data  

## 🎮 Usage

### Enable Filtering

```rust
// In dock_layout.rs or main editor UI
render_hierarchy_with_filter(
    ui,
    world,
    entity_names,
    selected_entity,
    // ... other params
    Some(&editor_state.map_manager), // Pass map_manager
);
```

### Disable Filtering (Show All)

```rust
render_hierarchy(
    ui,
    world,
    entity_names,
    selected_entity,
    // ... other params
    // No map_manager = show all entities
);
```

## 🔍 Edge Cases

### Selecting Map Entity

ถ้า user select map entity (เช่น click ใน Scene View):
- Hierarchy จะไม่แสดง entity นั้น
- แต่ Inspector จะแสดง properties ปกติ
- Maps panel จะ highlight entity นั้น

### Deleting Map Entity

ถ้า user ลบ map entity:
- ควรลบผ่าน Maps panel (Clean Up Colliders)
- ถ้าลบผ่าน Inspector จะลบได้ แต่ MapManager ไม่รู้

### Parenting Map Entity

ถ้า user ลาก GameObject เป็น child ของ Grid:
- GameObject จะไม่ถูก filter (ไม่ใช่ map entity)
- แสดงใน Hierarchy ปกติ

## 🚀 Future Improvements

- [ ] Toggle "Show Map Entities" in Hierarchy
- [ ] Highlight selected map entity in Maps panel
- [ ] Sync selection between Hierarchy and Maps panel
- [ ] Context menu "Show in Maps Panel"
- [ ] Filter by layer (show/hide specific layers)

## 📚 Related Files

- `engine/src/editor/ui/hierarchy.rs` - Hierarchy rendering with filter
- `engine/src/editor/ui/maps_panel.rs` - Maps panel UI
- `engine/src/editor/map_manager.rs` - MapManager implementation
- `ecs/src/loaders/ldtk_loader.rs` - LDtk loader with Grid

