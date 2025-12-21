# LDtk Workflow Redesign

## 🎯 Current Problems

### Problem 1: Colliders Not Tracked
เมื่อ generate colliders แล้ว clean up แล้ว generate ใหม่:
- Colliders ใหม่ไม่ถูก track โดย MapManager
- แสดงใน Hierarchy แทนที่จะซ่อน
- ต้อง reload map ทั้งหมดเพื่อ fix

### Problem 2: Manual Workflow
User ต้อง:
1. Load map
2. Generate colliders (manual)
3. Clean up colliders (manual)
4. Generate ใหม่ (manual)

### Problem 3: Colliders Separate from Map
- Colliders ไม่ใช่ส่วนหนึ่งของ Grid hierarchy
- ไม่ถูกลบเมื่อ reload map
- ต้อง clean up แยก

## ✅ Proposed Solution

### Design 1: Auto-Generate Colliders (Recommended)

```
Load LDtk Map
    ↓
Create Grid Entity
    ↓
Create Tilemap Layers (as children)
    ↓
Auto-Generate Colliders (as children)
    ↓
Done!
```

**Benefits:**
- ✅ Automatic - ไม่ต้อง manual generate
- ✅ Colliders เป็น children ของ Grid
- ✅ Reload map = ลบ colliders อัตโนมัติ
- ✅ ไม่ต้อง clean up แยก

**Implementation:**

```rust
impl LdtkLoader {
    pub fn load_project_with_grid(
        path: impl AsRef<Path>,
        world: &mut World,
    ) -> Result<(Entity, Vec<Entity>, Vec<Entity>), String> {
        // Load grid and tilemaps
        let (grid_entity, tilemap_entities) = /* ... */;
        
        // Auto-generate colliders
        let collider_entities = Self::generate_composite_colliders_from_intgrid(
            path,
            world,
            1, // collision_value
        )?;
        
        // Set colliders as children of Grid
        for &collider in &collider_entities {
            world.set_parent(collider, Some(grid_entity));
        }
        
        Ok((grid_entity, tilemap_entities, collider_entities))
    }
}
```

### Design 2: Colliders as Grid Children

```
Grid Entity
├── Tilemap Layer 1
├── Tilemap Layer 2
└── Colliders (Group)
    ├── CompositeCollider_1
    ├── CompositeCollider_2
    └── ...
```

**Benefits:**
- ✅ Organized hierarchy
- ✅ Despawn Grid = despawn all children (including colliders)
- ✅ Easy to toggle colliders visibility

**Implementation:**

```rust
// Create colliders group
let colliders_group = world.spawn();
world.names.insert(colliders_group, "Colliders".to_string());
world.transforms.insert(colliders_group, Transform::default());
world.set_parent(colliders_group, Some(grid_entity));

// Generate colliders as children of group
for collider in generate_colliders() {
    world.set_parent(collider, Some(colliders_group));
}
```

### Design 3: Collider Layer Component

```rust
pub struct ColliderLayer {
    pub source_layer: String,  // "IntGrid_layer"
    pub collision_value: i64,  // 1
    pub collider_type: ColliderType,  // Composite, Individual, Polygon
    pub colliders: Vec<Entity>,
}

// Add to Grid entity
world.collider_layers.insert(grid_entity, ColliderLayer {
    source_layer: "IntGrid_layer".to_string(),
    collision_value: 1,
    collider_type: ColliderType::Composite,
    colliders: vec![],
});
```

**Benefits:**
- ✅ Metadata about colliders
- ✅ Easy to regenerate
- ✅ Can have multiple collision layers

## 🎨 Recommended Workflow

### Option A: Fully Automatic (Best for Most Cases)

```rust
// Load map with auto-generated colliders
let (grid, layers, colliders) = LdtkLoader::load_project_with_grid_and_colliders(
    "levels/Level_01.ldtk",
    world,
    LdtkColliderConfig {
        auto_generate: true,
        collision_value: 1,
        collider_type: ColliderType::Composite,
    }
)?;
```

**UI:**
```
Maps Panel
└── Level_01.ldtk
    ├── Settings
    │   ├── ☑ Auto-generate colliders
    │   ├── Collision value: [1]
    │   └── Type: [Composite ▼]
    └── Grid
        ├── Layers (2)
        └── Colliders (28)
```

### Option B: Manual Control (Advanced Users)

```rust
// Load map without colliders
let (grid, layers) = LdtkLoader::load_project_with_grid(path, world)?;

// Generate colliders manually
let colliders = LdtkLoader::generate_colliders_for_grid(
    grid,
    world,
    ColliderConfig { /* ... */ }
)?;
```

**UI:**
```
Maps Panel
└── Level_01.ldtk
    ├── Grid
    │   ├── Layers (2)
    │   └── Colliders (0)
    └── Actions
        ├── [Generate Colliders]
        └── [Regenerate Colliders]
```

## 🔧 Implementation Plan

### Phase 1: Auto-Generate Colliders

```rust
// ecs/src/loaders/ldtk_loader.rs

impl LdtkLoader {
    /// Load with auto-generated colliders
    pub fn load_project_with_grid_and_colliders(
        path: impl AsRef<Path>,
        world: &mut World,
        config: LdtkColliderConfig,
    ) -> Result<(Entity, Vec<Entity>, Vec<Entity>), String> {
        // Load grid and tilemaps
        let (grid_entity, tilemap_entities) = Self::load_project_with_grid_internal(path.as_ref(), world)?;
        
        let mut collider_entities = Vec::new();
        
        if config.auto_generate {
            // Generate colliders
            let colliders = Self::generate_composite_colliders_from_intgrid(
                path,
                world,
                config.collision_value,
            )?;
            
            // Set as children of Grid
            for &collider in &colliders {
                world.set_parent(collider, Some(grid_entity));
            }
            
            collider_entities = colliders;
        }
        
        Ok((grid_entity, tilemap_entities, collider_entities))
    }
}

pub struct LdtkColliderConfig {
    pub auto_generate: bool,
    pub collision_value: i64,
    pub collider_type: ColliderType,
}

impl Default for LdtkColliderConfig {
    fn default() -> Self {
        Self {
            auto_generate: true,
            collision_value: 1,
            collider_type: ColliderType::Composite,
        }
    }
}

pub enum ColliderType {
    Individual,
    Composite,
    Polygon,
}
```

### Phase 2: Update MapManager

```rust
// engine/src/editor/map_manager.rs

impl MapManager {
    pub fn load_map_with_config(
        &mut self,
        path: &PathBuf,
        world: &mut World,
        config: LdtkColliderConfig,
    ) -> Result<(), String> {
        // Load with colliders
        let (grid, layers, colliders) = 
            ecs::loaders::LdtkLoader::load_project_with_grid_and_colliders(
                path,
                world,
                config,
            )?;
        
        // Store in LoadedMap
        let loaded_map = LoadedMap {
            grid_entity: grid,
            layer_entities: /* ... */,
            collider_entities: colliders,  // Now tracked!
            /* ... */
        };
        
        self.loaded_maps.insert(path.clone(), loaded_map);
        Ok(())
    }
    
    /// Regenerate colliders for a map
    pub fn regenerate_colliders(
        &mut self,
        path: &PathBuf,
        world: &mut World,
    ) -> Result<(), String> {
        if let Some(loaded_map) = self.loaded_maps.get_mut(path) {
            // Remove old colliders
            for &collider in &loaded_map.collider_entities {
                world.despawn(collider);
            }
            
            // Generate new colliders
            let colliders = ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
                path,
                world,
                1,
            )?;
            
            // Set as children of Grid
            for &collider in &colliders {
                world.set_parent(collider, Some(loaded_map.grid_entity));
            }
            
            // Update tracking
            loaded_map.collider_entities = colliders;
        }
        
        Ok(())
    }
}
```

### Phase 3: Update UI

```rust
// engine/src/editor/ui/maps_panel.rs

fn render_actions_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.collapsing("⚙️ Actions", |ui| {
        // Reload Map
        if ui.button("🔄 Reload Map").clicked() {
            if let Some(path) = &map_manager.selected_map.clone() {
                let config = LdtkColliderConfig::default();
                if let Err(e) = map_manager.load_map_with_config(path, world, config) {
                    log::error!("Failed to reload: {}", e);
                }
            }
        }
        
        // Regenerate Colliders (replaces Generate + Clean Up)
        if ui.button("🔨 Regenerate Colliders").clicked() {
            if let Some(path) = &map_manager.selected_map.clone() {
                match map_manager.regenerate_colliders(path, world) {
                    Ok(_) => log::info!("Colliders regenerated"),
                    Err(e) => log::error!("Failed: {}", e),
                }
            }
        }
    });
}
```

## 📊 Comparison

| Feature | Current | Design 1 (Auto) | Design 2 (Children) | Design 3 (Component) |
|---------|---------|-----------------|---------------------|----------------------|
| Auto-generate | ❌ Manual | ✅ Automatic | ✅ Automatic | ✅ Automatic |
| Hierarchy clean | ❌ Separate | ✅ Children | ✅ Grouped | ✅ Children |
| Reload clean | ❌ Manual | ✅ Automatic | ✅ Automatic | ✅ Automatic |
| Regenerate | ❌ Clean + Gen | ✅ One click | ✅ One click | ✅ One click |
| Tracking | ❌ Broken | ✅ Always | ✅ Always | ✅ Always |
| Complexity | Simple | Simple | Medium | Complex |

## 🎯 Recommendation

**Use Design 1 + Design 2 Combined:**

1. **Auto-generate colliders** when loading map
2. **Set colliders as children** of Grid entity
3. **Track in MapManager** for easy management
4. **One-click regenerate** instead of clean + generate

**Benefits:**
- ✅ Zero manual work
- ✅ Clean hierarchy
- ✅ Reliable tracking
- ✅ Easy to use

**Migration:**
```rust
// Old way (manual)
map_manager.load_map(path, world)?;
map_manager.generate_colliders(path, world)?;

// New way (automatic)
map_manager.load_map(path, world)?;  // Colliders auto-generated!
```

## 🚀 Next Steps

1. ✅ Implement `load_project_with_grid_and_colliders()`
2. ✅ Update MapManager to use new loader
3. ✅ Set colliders as Grid children
4. ✅ Add "Regenerate Colliders" button
5. ✅ Remove "Generate" and "Clean Up" buttons
6. ✅ Update documentation

