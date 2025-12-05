use ecs::{World, Entity};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Map Manager for handling LDtk files in the editor
pub struct MapManager {
    /// Loaded LDtk files
    pub loaded_maps: HashMap<PathBuf, LoadedMap>,
    
    /// Available LDtk files in project
    pub available_files: Vec<PathBuf>,
    
    /// Selected map for actions
    pub selected_map: Option<PathBuf>,
    
    /// Project path for scanning files
    pub project_path: Option<PathBuf>,
}

/// Information about a loaded map
#[derive(Clone)]
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

/// Information about a tilemap layer
#[derive(Clone)]
pub struct LayerInfo {
    pub entity: Entity,
    pub name: String,
    pub size: (u32, u32),
    pub visible: bool,
    pub z_order: i32,
}

impl MapManager {
    /// Create a new MapManager
    pub fn new() -> Self {
        Self {
            loaded_maps: HashMap::new(),
            available_files: Vec::new(),
            selected_map: None,
            project_path: None,
        }
    }
    
    /// Set project path and scan for LDtk files
    pub fn set_project_path(&mut self, path: PathBuf) {
        self.project_path = Some(path.clone());
        self.scan_ldtk_files();
    }
    
    /// Scan project directory for LDtk files
    pub fn scan_ldtk_files(&mut self) {
        self.available_files.clear();
        
        if let Some(project_path) = &self.project_path {
            if let Ok(entries) = std::fs::read_dir(project_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    // Check subdirectories
                    if path.is_dir() {
                        self.scan_directory(&path);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("ldtk") {
                        self.available_files.push(path);
                    }
                }
            }
        }
        
        log::info!("Found {} LDtk files", self.available_files.len());
    }
    
    /// Recursively scan directory for LDtk files
    fn scan_directory(&mut self, dir: &PathBuf) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    self.scan_directory(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("ldtk") {
                    self.available_files.push(path);
                }
            }
        }
    }
    
    /// Load a map file with auto-generated colliders
    pub fn load_map(&mut self, path: &PathBuf, world: &mut World) -> Result<(), String> {
        // Remove old map if exists
        if let Some(old_map) = self.loaded_maps.remove(path) {
            world.despawn(old_map.grid_entity);
        }
        
        // Load new map with Grid and auto-generated colliders
        let (grid_entity, layer_entities, collider_entities) = 
            ecs::loaders::LdtkLoader::load_project_with_grid_and_colliders(
                path,
                world,
                true,  // auto_generate_colliders
                1,     // collision_value
            )?;
        
        // Create LayerInfo for each layer
        let layer_infos: Vec<LayerInfo> = layer_entities.iter().map(|&entity| {
            LayerInfo {
                entity,
                name: world.names.get(&entity)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                size: world.tilemaps.get(&entity)
                    .map(|t| (t.width, t.height))
                    .unwrap_or((0, 0)),
                visible: world.active.get(&entity).copied().unwrap_or(true),
                z_order: world.tilemaps.get(&entity)
                    .map(|t| t.z_order)
                    .unwrap_or(0),
            }
        }).collect();
        
        // Get last modified time
        let last_modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());
        
        // Store loaded map
        let loaded_map = LoadedMap {
            grid_entity,
            layer_entities: layer_infos,
            collider_entities,  // Now includes auto-generated colliders
            file_path: path.clone(),
            last_modified,
        };
        
        self.loaded_maps.insert(path.clone(), loaded_map);
        self.selected_map = Some(path.clone());
        
        log::info!("Loaded map: {:?}", path);
        Ok(())
    }
    
    /// Reload a map file while preserving Grid Entity ID
    pub fn reload_map(&mut self, path: &PathBuf, world: &mut World) -> Result<(), String> {
        // Get existing map info
        let existing_map = self.loaded_maps.get(path)
            .ok_or_else(|| format!("Map not loaded: {:?}", path))?;
        
        let grid_entity = existing_map.grid_entity;
        
        // Store visibility states before reload
        let mut visibility_states: HashMap<String, bool> = HashMap::new();
        for layer in &existing_map.layer_entities {
            visibility_states.insert(layer.name.clone(), layer.visible);
        }
        
        // Despawn all children (layers and colliders) but keep Grid Entity
        let children: Vec<Entity> = world.get_children(grid_entity).to_vec();
        for child in children {
            world.despawn(child);
        }
        
        // Load the project JSON
        let project_data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: serde_json::Value = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;
        
        // Update Grid component
        let grid_size = project["defaultGridSize"]
            .as_i64()
            .unwrap_or(8) as f32;
        
        let pixels_per_unit = 8.0;
        let grid = ecs::Grid {
            cell_size: (grid_size / pixels_per_unit, grid_size / pixels_per_unit),
            cell_gap: (0.0, 0.0),
            layout: ecs::GridLayout::Rectangle,
            swizzle: ecs::CellSwizzle::XYZ,
        };
        world.grids.insert(grid_entity, grid);
        
        // Load levels as children of existing Grid
        let mut layer_entities = Vec::new();
        let levels = project["levels"]
            .as_array()
            .ok_or("No levels found in LDTK file")?;

        for level in levels {
            let entities = self.load_level_as_children(
                level,
                &project,
                world,
                grid_entity,
                path,
            )?;
            layer_entities.extend(entities);
        }
        
        // Generate new colliders
        let collider_entities = ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
            path,
            world,
            1, // collision_value
        )?;
        
        // Set colliders as children of Grid
        for &collider in &collider_entities {
            world.set_parent(collider, Some(grid_entity));
        }
        
        // Create LayerInfo and restore visibility states
        let layer_infos: Vec<LayerInfo> = layer_entities.iter().map(|&entity| {
            let name = world.names.get(&entity)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            
            // Restore visibility state if it existed
            let visible = visibility_states.get(&name).copied().unwrap_or(true);
            if let Some(active) = world.active.get_mut(&entity) {
                *active = visible;
            }
            
            LayerInfo {
                entity,
                name,
                size: world.tilemaps.get(&entity)
                    .map(|t| (t.width, t.height))
                    .unwrap_or((0, 0)),
                visible,
                z_order: world.tilemaps.get(&entity)
                    .map(|t| t.z_order)
                    .unwrap_or(0),
            }
        }).collect();
        
        // Get last modified time
        let last_modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());
        
        // Update loaded map (preserving grid_entity)
        let loaded_map = LoadedMap {
            grid_entity,  // Same entity ID
            layer_entities: layer_infos,
            collider_entities,
            file_path: path.clone(),
            last_modified,
        };
        
        self.loaded_maps.insert(path.clone(), loaded_map);
        
        log::info!("Reloaded map: {:?} (preserved Grid Entity {})", path, grid_entity);
        Ok(())
    }
    
    /// Load a level as children of Grid entity (helper for reload)
    fn load_level_as_children(
        &self,
        level: &serde_json::Value,
        project: &serde_json::Value,
        world: &mut World,
        grid_parent: Entity,
        ldtk_path: &PathBuf,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        // Get level world position
        let level_world_x = level["worldX"].as_i64().unwrap_or(0) as f32;
        let level_world_y = level["worldY"].as_i64().unwrap_or(0) as f32;
        
        // Get layer instances
        let empty_vec = vec![];
        let layer_instances = level["layerInstances"]
            .as_array()
            .unwrap_or(&empty_vec);

        // Process each layer
        for layer in layer_instances {
            // Get layer properties
            let identifier = layer["__identifier"]
                .as_str()
                .unwrap_or("Unknown");
            
            let width = layer["__cWid"]
                .as_i64()
                .unwrap_or(0) as u32;
            
            let height = layer["__cHei"]
                .as_i64()
                .unwrap_or(0) as u32;
            
            let layer_def_uid = layer["layerDefUid"]
                .as_i64()
                .unwrap_or(0);
            
            let px_offset_x = layer["__pxTotalOffsetX"]
                .as_i64()
                .unwrap_or(0) as f32;
            
            let px_offset_y = layer["__pxTotalOffsetY"]
                .as_i64()
                .unwrap_or(0) as f32;

            // Check if layer has tiles
            let has_grid_tiles = layer["gridTiles"]
                .as_array()
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);
            
            let has_auto_tiles = layer["autoLayerTiles"]
                .as_array()
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);

            // Only create entity if layer has tiles
            if has_grid_tiles || has_auto_tiles {
                let entity = world.spawn();

                // Set parent to Grid
                world.set_parent(entity, Some(grid_parent));

                // Create tilemap
                let mut tilemap = ecs::Tilemap::new(
                    identifier,
                    format!("tileset_{}", layer_def_uid),
                    width,
                    height,
                );

                // Get grid size for tile positioning
                let grid_size = layer["__gridSize"]
                    .as_i64()
                    .unwrap_or(8) as u32;

                // Parse tiles (prefer autoLayerTiles, fallback to gridTiles)
                let tiles_array = if has_auto_tiles {
                    layer["autoLayerTiles"].as_array()
                } else {
                    layer["gridTiles"].as_array()
                };

                if let Some(tiles) = tiles_array {
                    for tile_data in tiles {
                        // Get tile position in pixels
                        let px = tile_data["px"].as_array();
                        
                        // Get tile ID
                        let tile_id = tile_data["t"]
                            .as_i64()
                            .unwrap_or(0) as u32;
                        
                        // Get flip flags (f: 0=none, 1=flipX, 2=flipY, 3=both)
                        let flip_flags = tile_data["f"]
                            .as_i64()
                            .unwrap_or(0);
                        
                        if let Some(px_array) = px {
                            if px_array.len() >= 2 {
                                let tile_x = px_array[0].as_i64().unwrap_or(0) as u32;
                                let tile_y = px_array[1].as_i64().unwrap_or(0) as u32;
                                
                                // Convert pixel position to tile coordinates
                                let grid_x = tile_x / grid_size;
                                let grid_y = tile_y / grid_size;
                                
                                // Create tile
                                let tile = ecs::Tile {
                                    tile_id,
                                    flip_h: (flip_flags & 1) != 0,
                                    flip_v: (flip_flags & 2) != 0,
                                    flip_d: false,
                                };
                                
                                // Set tile in tilemap
                                tilemap.set_tile(grid_x, grid_y, tile);
                            }
                        }
                    }
                }

                // Get tileset info and create TileSet component
                let tileset_uid = layer["__tilesetDefUid"]
                    .as_i64()
                    .unwrap_or(0);
                
                if tileset_uid > 0 {
                    // Find tileset definition in project
                    if let Some(tilesets) = project["defs"]["tilesets"].as_array() {
                        for tileset_def in tilesets {
                            if tileset_def["uid"].as_i64().unwrap_or(0) == tileset_uid {
                                if let Some(tileset_rel_path) = tileset_def["relPath"].as_str() {
                                    // Convert relative path to absolute path
                                    let ldtk_dir = ldtk_path.parent().unwrap_or(std::path::Path::new("."));
                                    let tileset_path = ldtk_dir.join(tileset_rel_path);
                                    let tileset_path_str = tileset_path.to_string_lossy().to_string();
                                    
                                    // Get tileset dimensions
                                    let tileset_width = tileset_def["pxWid"].as_i64().unwrap_or(256) as u32;
                                    let tileset_height = tileset_def["pxHei"].as_i64().unwrap_or(256) as u32;
                                    let columns = tileset_width / grid_size;
                                    let rows = tileset_height / grid_size;
                                    let tile_count = columns * rows;
                                    
                                    // Create TileSet component
                                    let tileset = ecs::TileSet::new(
                                        format!("tileset_{}", tileset_uid),
                                        tileset_path_str.clone(),
                                        format!("tileset_{}", tileset_uid),
                                        grid_size,
                                        grid_size,
                                        columns,
                                        tile_count,
                                    );
                                    
                                    world.tilesets.insert(entity, tileset);
                                }
                                break;
                            }
                        }
                    }
                }

                world.tilemaps.insert(entity, tilemap);
                world.names.insert(entity, format!("LDTK Layer: {}", identifier));

                // Add transform at layer offset (relative to Grid parent)
                let pixels_per_unit = 8.0;
                let total_px_x = level_world_x + px_offset_x;
                let total_px_y = level_world_y + px_offset_y;
                let world_x = total_px_x / pixels_per_unit;
                let world_y = -total_px_y / pixels_per_unit;
                
                let transform = ecs::Transform::with_position(
                    world_x,
                    world_y,
                    0.0,
                );
                world.transforms.insert(entity, transform);

                entities.push(entity);
            }
        }

        Ok(entities)
    }
    
    /// Unload a map file
    pub fn unload_map(&mut self, path: &PathBuf, world: &mut World) {
        if let Some(loaded_map) = self.loaded_maps.remove(path) {
            world.despawn(loaded_map.grid_entity);
            log::info!("Unloaded map: {:?}", path);
        }
    }
    
    /// Regenerate colliders for a map
    pub fn regenerate_colliders(&mut self, path: &PathBuf, world: &mut World) -> Result<usize, String> {
        if let Some(loaded_map) = self.loaded_maps.get_mut(path) {
            // Remove old colliders
            for &collider in &loaded_map.collider_entities {
                world.despawn(collider);
            }
            loaded_map.collider_entities.clear();
            
            // Generate new colliders
            let colliders = ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
                path,
                world,
                1, // collision_value
            )?;
            
            // Set as children of Grid
            for &collider in &colliders {
                world.set_parent(collider, Some(loaded_map.grid_entity));
            }
            
            // Update tracking
            loaded_map.collider_entities = colliders.clone();
            
            log::info!("Regenerated {} colliders for {:?}", colliders.len(), path);
            Ok(colliders.len())
        } else {
            Err(format!("Map not loaded: {:?}", path))
        }
    }
    
    /// Clean up colliders for a specific map
    pub fn clean_up_colliders(&mut self, path: &PathBuf, world: &mut World) -> usize {
        if let Some(loaded_map) = self.loaded_maps.get_mut(path) {
            let count = loaded_map.collider_entities.len();
            
            // Remove colliders
            for &collider in &loaded_map.collider_entities {
                world.despawn(collider);
            }
            
            // Clear tracking
            loaded_map.collider_entities.clear();
            
            log::info!("Cleaned up {} colliders for {:?}", count, path);
            count
        } else {
            0
        }
    }
    
    /// Clean up all LDtk colliders (all maps)
    pub fn clean_up_all_colliders(&mut self, world: &mut World) -> usize {
        let mut total = 0;
        
        for loaded_map in self.loaded_maps.values_mut() {
            for &collider in &loaded_map.collider_entities {
                world.despawn(collider);
            }
            total += loaded_map.collider_entities.len();
            loaded_map.collider_entities.clear();
        }
        
        log::info!("Cleaned up {} colliders (all maps)", total);
        total
    }
    
    /// Count LDtk colliders in world
    pub fn count_colliders(&self, world: &World) -> usize {
        world.names.iter()
            .filter(|(_, name)| {
                name.starts_with("CompositeCollider") || name.starts_with("Collider_")
            })
            .count()
    }
    
    /// Toggle layer visibility
    pub fn toggle_layer_visibility(&mut self, entity: Entity, world: &mut World) {
        if let Some(active) = world.active.get_mut(&entity) {
            *active = !*active;
            
            // Update LayerInfo
            for loaded_map in self.loaded_maps.values_mut() {
                if let Some(layer) = loaded_map.layer_entities.iter_mut().find(|l| l.entity == entity) {
                    layer.visible = *active;
                }
            }
        }
    }
    
    /// Check if entity is a map entity
    pub fn is_map_entity(&self, entity: Entity) -> bool {
        for loaded_map in self.loaded_maps.values() {
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
    
    /// Get loaded map by entity
    pub fn get_map_by_entity(&self, entity: Entity) -> Option<&LoadedMap> {
        self.loaded_maps.values().find(|map| {
            map.grid_entity == entity ||
            map.layer_entities.iter().any(|l| l.entity == entity) ||
            map.collider_entities.contains(&entity)
        })
    }
}

impl Default for MapManager {
    fn default() -> Self {
        Self::new()
    }
}
