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
    
    /// Reload a map file
    pub fn reload_map(&mut self, path: &PathBuf, world: &mut World) -> Result<(), String> {
        self.load_map(path, world)
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
