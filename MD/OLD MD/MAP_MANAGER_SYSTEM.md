# Map Manager System - Editor UI

## 🎯 Overview

ระบบจัดการ Map แบบ production-ready ใน Editor ที่แสดง:
- LDtk Files list
- Loaded maps with Grid hierarchy
- Layer management
- Actions (Reload, Generate Colliders, Clean Up)
- Statistics

## 🎨 UI Design

```
┌─────────────────────────────────────┐
│ 🗺️  Maps                      [↻]  │
├─────────────────────────────────────┤
│ ▼ 📁 LDtk Files                     │
│   ├─ 🗺️ Level_01.ldtk         [↻]  │
│   └─ 🗺️ simple_level.ldtk     [↻]  │
│                                     │
│ ▼ 🎯 Level_01.ldtk                  │
│   │ Loaded Tilemaps: 1              │
│   │                                 │
│   └─ ▼ 📐 LDtk Grid                 │
│      ├─ 🎨 IntGrid_layer (42x26)    │
│      ├─ 🎨 Tiles (42x26)            │
│      └─ 🎨 Entities (42x26)         │
│                                     │
│ ▼ ⚙️  Actions                       │
│   ├─ [🔄 Reload Map]                │
│   ├─ [🔨 Generate Colliders]        │
│   └─ [🧹 Clean Up Colliders (28)]   │
│                                     │
│ ▼ 📊 Statistics                     │
│   ├─ Entities: 32                   │
│   ├─ Tilemaps: 1                    │
│   ├─ Colliders: 29                  │
│   └─ Sprites: 1                     │
└─────────────────────────────────────┘
```

## 🏗️ Architecture

### Map Manager Component

```rust
pub struct MapManager {
    /// Loaded LDtk files
    pub loaded_maps: HashMap<PathBuf, LoadedMap>,
    
    /// Available LDtk files in project
    pub available_files: Vec<PathBuf>,
    
    /// Selected map for actions
    pub selected_map: Option<PathBuf>,
    
    /// Hot-reload runtime
    pub ldtk_runtime: LdtkRuntime,
}

pub struct LoadedMap {
    /// Grid entity (parent)
    pub grid_entity: Entity,
    
    /// Tilemap layer entities (children)
    pub layer_entities: Vec<LayerInfo>,
    
    /// Collider entities
    pub collider_entities: Vec<Entity>,
    
    /// File path
    pub file_path: PathBuf,
    
    /// Last modified time
    pub last_modified: SystemTime,
}

pub struct LayerInfo {
    pub entity: Entity,
    pub name: String,
    pub size: (u32, u32),
    pub visible: bool,
    pub z_order: i32,
}
```

## 🎮 Features

### 1. LDtk Files List

```rust
fn render_ldtk_files_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.collapsing("📁 LDtk Files", |ui| {
        for file_path in &map_manager.available_files {
            ui.horizontal(|ui| {
                // File icon and name
                ui.label("🗺️");
                
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                // Selectable file
                if ui.selectable_label(
                    map_manager.selected_map.as_ref() == Some(file_path),
                    file_name
                ).clicked() {
                    map_manager.selected_map = Some(file_path.clone());
                }
                
                // Reload button
                if ui.small_button("↻").clicked() {
                    reload_map(file_path, map_manager, world);
                }
            });
        }
        
        // Add new map button
        if ui.button("➕ Add LDtk File").clicked() {
            // Open file dialog
            if let Some(path) = open_file_dialog() {
                load_new_map(&path, map_manager, world);
            }
        }
    });
}
```

### 2. Loaded Map Hierarchy

```rust
fn render_loaded_map_section(
    ui: &mut egui::Ui,
    file_path: &PathBuf,
    loaded_map: &LoadedMap,
    world: &World,
) {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");
    
    ui.collapsing(format!("🎯 {}", file_name), |ui| {
        // Info
        ui.label(format!("Loaded Tilemaps: {}", loaded_map.layer_entities.len()));
        ui.separator();
        
        // Grid entity
        if let Some(grid_name) = world.names.get(&loaded_map.grid_entity) {
            ui.collapsing(format!("📐 {}", grid_name), |ui| {
                // Grid info
                if let Some(grid) = world.grids.get(&loaded_map.grid_entity) {
                    ui.label(format!("Cell Size: {:?}", grid.cell_size));
                    ui.label(format!("Layout: {:?}", grid.layout));
                }
                
                ui.separator();
                
                // Layers
                for layer in &loaded_map.layer_entities {
                    render_layer_item(ui, layer, world);
                }
            });
        }
    });
}

fn render_layer_item(
    ui: &mut egui::Ui,
    layer: &LayerInfo,
    world: &World,
) {
    ui.horizontal(|ui| {
        // Layer icon
        ui.label("🎨");
        
        // Layer name and size
        ui.label(format!("{} ({}x{})", 
            layer.name, 
            layer.size.0, 
            layer.size.1
        ));
        
        // Visibility toggle
        if ui.small_button(if layer.visible { "👁" } else { "👁‍🗨" }).clicked() {
            // Toggle visibility
        }
    });
}
```

### 3. Actions Section

```rust
fn render_actions_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.collapsing("⚙️ Actions", |ui| {
        // Reload Map
        if ui.button("🔄 Reload Map").clicked() {
            if let Some(path) = &map_manager.selected_map {
                reload_map(path, map_manager, world);
            }
        }
        
        // Generate Colliders
        if ui.button("🔨 Generate Colliders").clicked() {
            if let Some(path) = &map_manager.selected_map {
                generate_colliders(path, map_manager, world);
            }
        }
        
        // Clean Up Colliders
        let collider_count = count_ldtk_colliders(world);
        if ui.button(format!("🧹 Clean Up Colliders ({})", collider_count)).clicked() {
            clean_up_colliders(world);
        }
    });
}

fn reload_map(
    path: &PathBuf,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    // Remove old map
    if let Some(old_map) = map_manager.loaded_maps.remove(path) {
        world.despawn(old_map.grid_entity);
    }
    
    // Load new map
    match LdtkLoader::load_project_with_grid(path, world) {
        Ok((grid_entity, layer_entities)) => {
            let loaded_map = LoadedMap {
                grid_entity,
                layer_entities: layer_entities.iter().map(|&entity| {
                    LayerInfo {
                        entity,
                        name: world.names.get(&entity)
                            .cloned()
                            .unwrap_or_else(|| "Unknown".to_string()),
                        size: world.tilemaps.get(&entity)
                            .map(|t| (t.width, t.height))
                            .unwrap_or((0, 0)),
                        visible: true,
                        z_order: 0,
                    }
                }).collect(),
                collider_entities: Vec::new(),
                file_path: path.clone(),
                last_modified: SystemTime::now(),
            };
            
            map_manager.loaded_maps.insert(path.clone(), loaded_map);
            log::info!("Reloaded map: {:?}", path);
        }
        Err(e) => {
            log::error!("Failed to reload map: {}", e);
        }
    }
}

fn generate_colliders(
    path: &PathBuf,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    match LdtkLoader::generate_composite_colliders_from_intgrid(
        path,
        world,
        1, // collision_value
    ) {
        Ok(colliders) => {
            // Store collider entities
            if let Some(loaded_map) = map_manager.loaded_maps.get_mut(path) {
                loaded_map.collider_entities.extend(colliders.iter());
            }
            log::info!("Generated {} colliders", colliders.len());
        }
        Err(e) => {
            log::error!("Failed to generate colliders: {}", e);
        }
    }
}

fn clean_up_colliders(world: &mut World) {
    let mut colliders_to_remove = Vec::new();
    
    for (entity, name) in &world.names {
        if name.starts_with("CompositeCollider") || name.starts_with("Collider_") {
            colliders_to_remove.push(*entity);
        }
    }
    
    for entity in colliders_to_remove {
        world.despawn(entity);
    }
    
    log::info!("Cleaned up colliders");
}

fn count_ldtk_colliders(world: &World) -> usize {
    world.names.iter()
        .filter(|(_, name)| {
            name.starts_with("CompositeCollider") || name.starts_with("Collider_")
        })
        .count()
}
```

### 4. Statistics Section

```rust
fn render_statistics_section(
    ui: &mut egui::Ui,
    world: &World,
) {
    ui.collapsing("📊 Statistics", |ui| {
        ui.horizontal(|ui| {
            ui.label("Entities:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", world.transforms.len()));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Tilemaps:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", world.tilemaps.len()));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Colliders:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", world.colliders.len()));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Sprites:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}", world.sprites.len()));
            });
        });
    });
}
```

## 🔧 Integration with Hierarchy

### Show Map Entities in Hierarchy

```rust
fn render_hierarchy_with_maps(
    ui: &mut egui::Ui,
    world: &World,
    map_manager: &MapManager,
) {
    // Show map entities grouped
    for (path, loaded_map) in &map_manager.loaded_maps {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        
        ui.collapsing(format!("🗺️ {}", file_name), |ui| {
            // Grid entity
            render_entity_tree(ui, loaded_map.grid_entity, world, 0);
        });
    }
    
    ui.separator();
    
    // Show other entities
    for (entity, _) in &world.transforms {
        // Skip map entities
        if is_map_entity(*entity, map_manager) {
            continue;
        }
        
        // Only show root entities (no parent)
        if !world.parents.contains_key(entity) {
            render_entity_tree(ui, *entity, world, 0);
        }
    }
}

fn is_map_entity(entity: Entity, map_manager: &MapManager) -> bool {
    for loaded_map in map_manager.loaded_maps.values() {
        if entity == loaded_map.grid_entity {
            return true;
        }
        if loaded_map.layer_entities.iter().any(|l| l.entity == entity) {
            return true;
        }
        if loaded_map.collider_entities.contains(&entity) {
            return true;
        }
    }
    false
}
```

## 📁 File Structure

```
engine/src/editor/
├── ui/
│   ├── mod.rs
│   ├── hierarchy.rs          (existing)
│   ├── inspector.rs          (existing)
│   ├── map_manager.rs        (NEW)
│   └── map_panel.rs          (NEW)
└── states.rs
    └── MapManager field
```

## 🎯 Implementation Steps

1. ✅ Create MapManager struct
2. ✅ Add to EditorState
3. ✅ Scan project for LDtk files
4. ✅ Render Maps panel UI
5. ✅ Implement Reload action
6. ✅ Implement Generate Colliders action
7. ✅ Implement Clean Up action
8. ✅ Show statistics
9. ✅ Integrate with Hierarchy
10. ✅ Add keyboard shortcuts

## 🎮 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+R` | Reload selected map |
| `Ctrl+G` | Generate colliders |
| `Ctrl+Shift+C` | Clean up colliders |

## 🔍 Context Menu

Right-click on map file:
- 🔄 Reload
- 🔨 Generate Colliders
- 👁 Show/Hide Layers
- 📋 Copy Path
- 🗑️ Unload Map

Right-click on layer:
- 👁 Toggle Visibility
- 🔼 Move Up
- 🔽 Move Down
- 📋 Copy Name
- 🗑️ Delete Layer

## 📊 Benefits

✅ **Better Organization** - Maps grouped by file  
✅ **Easy Management** - One-click reload/generate  
✅ **Visual Hierarchy** - Grid → Layers structure  
✅ **Statistics** - Quick overview of scene  
✅ **Hot-Reload** - Auto-update on file change  
✅ **Layer Control** - Show/hide individual layers  

