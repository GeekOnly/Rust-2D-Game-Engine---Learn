use crate::{World, Entity};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::collections::HashMap;
use log::{info, warn, error};

/// Hot-reload system for LDtk files
/// 
/// Usage:
/// ```
/// let mut reloader = LdtkHotReloader::new();
/// reloader.watch("path/to/level.ldtk", &mut world)?;
/// 
/// // In game loop:
/// if let Some(updated_entities) = reloader.check_updates(&mut world) {
///     println!("Reloaded {} entities", updated_entities.len());
/// }
/// ```
pub struct LdtkHotReloader {
    watcher: Option<notify::RecommendedWatcher>,
    receiver: Receiver<notify::Result<Event>>,
    sender: Sender<notify::Result<Event>>,
    watched_files: HashMap<PathBuf, Vec<Entity>>,
}

impl LdtkHotReloader {
    /// Create a new hot-reloader
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        
        Self {
            watcher: None,
            receiver,
            sender,
            watched_files: HashMap::new(),
        }
    }

    /// Watch an LDtk file and load it into the world
    /// Returns the entities created from the file
    pub fn watch(&mut self, path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        let path = path.as_ref();
        let canonical_path = path.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        // Initialize watcher if not already created
        if self.watcher.is_none() {
            let sender = self.sender.clone();
            let watcher = notify::recommended_watcher(move |res| {
                let _ = sender.send(res);
            }).map_err(|e| format!("Failed to create watcher: {}", e))?;
            
            self.watcher = Some(watcher);
        }

        // Watch the file
        if let Some(watcher) = &mut self.watcher {
            watcher.watch(&canonical_path, RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch file: {}", e))?;
        }

        // Remove any existing LDTK entities before loading
        // This handles the case where scene was saved with LDTK entities
        self.remove_existing_ldtk_entities(world);

        // Load the file with Grid support and auto-generated colliders
        let (grid_entity, tilemap_entities, collider_entities) = 
            super::LdtkLoader::load_project_with_grid_and_colliders(
                path,
                world,
                true,  // auto_generate_colliders
                1,     // collision_value
                None,  // layer_filter
                None,
            )?;
        
        // Store all entities (Grid + Tilemaps + Colliders) for this file
        let mut all_entities = vec![grid_entity];
        all_entities.extend(tilemap_entities);
        all_entities.extend(collider_entities);
        
        self.watched_files.insert(canonical_path.clone(), all_entities.clone());
        
        info!("Watching LDtk file: {:?} ({} entities including Grid and Colliders)", canonical_path, all_entities.len());
        
        Ok(all_entities)
    }

    /// Stop watching a file
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        let canonical_path = path.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

        if let Some(watcher) = &mut self.watcher {
            watcher.unwatch(&canonical_path)
                .map_err(|e| format!("Failed to unwatch file: {}", e))?;
        }

        self.watched_files.remove(&canonical_path);
        info!("Stopped watching: {:?}", canonical_path);
        
        Ok(())
    }

    /// Check for file updates and reload if necessary
    /// Returns Some(entities) if files were reloaded, None otherwise
    pub fn check_updates(&mut self, world: &mut World) -> Option<Vec<Entity>> {
        let mut updated_files = Vec::new();

        // Collect all events
        while let Ok(event_result) = self.receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    // Check for modify, create, or remove events
                    // Some editors (like LDtk) save by creating temp file then renaming
                    // So we need to catch Create and Remove events too
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            for path in event.paths {
                                if self.watched_files.contains_key(&path) {
                                    if !updated_files.contains(&path) {
                                        updated_files.push(path);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    warn!("File watcher error: {}", e);
                }
            }
        }

        if updated_files.is_empty() {
            return None;
        }

        // Reload updated files
        let mut all_entities = Vec::new();
        
        for path in updated_files {
            info!("Reloading LDtk file: {:?}", path);
            
            // Remove ALL existing LDTK entities (not just tracked ones)
            // This ensures we clean up entities that were saved in the scene
            self.remove_existing_ldtk_entities(world);

            // Re-watch the file (in case it was removed/renamed during save)
            if let Some(watcher) = &mut self.watcher {
                // Unwatch first (ignore errors)
                let _ = watcher.unwatch(&path);
                
                // Re-watch (ignore errors if file doesn't exist yet)
                if path.exists() {
                    if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                        warn!("Failed to re-watch file {:?}: {}", path, e);
                    }
                }
            }

            // Wait a bit for file to be fully written (some editors write in chunks)
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Reload the file with Grid support and auto-generated colliders
            match super::LdtkLoader::load_project_with_grid_and_colliders(&path, world, true, 1, None, None) {
                Ok((grid_entity, tilemap_entities, collider_entities)) => {
                    let mut entities = vec![grid_entity];
                    entities.extend(tilemap_entities);
                    entities.extend(collider_entities);
                    
                    info!("Successfully reloaded {} entities (Grid + Layers + Colliders) from {:?}", entities.len(), path);
                    self.watched_files.insert(path.clone(), entities.clone());
                    all_entities.extend(entities);
                }
                Err(e) => {
                    error!("Failed to reload {:?}: {}", path, e);
                }
            }
        }

        if all_entities.is_empty() {
            None
        } else {
            Some(all_entities)
        }
    }

    /// Get all watched files
    pub fn watched_files(&self) -> Vec<PathBuf> {
        self.watched_files.keys().cloned().collect()
    }

    /// Get entities for a specific file
    pub fn get_entities(&self, path: impl AsRef<Path>) -> Option<&[Entity]> {
        let canonical_path = path.as_ref().canonicalize().ok()?;
        self.watched_files.get(&canonical_path).map(|v| v.as_slice())
    }

    /// Remove all existing LDTK entities from the world
    /// This is used to clean up before loading/reloading
    /// Remove all existing LDTK entities from the world
    /// This is used to clean up before loading/reloading
    fn remove_existing_ldtk_entities(&self, world: &mut World) {
        #[cfg(not(feature = "hecs"))]
        {
            // Collect all entities with names starting with "LDTK Layer:" or "LDtk Grid"
            let mut ldtk_entities = Vec::new();
            
            for (entity, name) in &world.names {
                if name.starts_with("LDTK Layer:") || name.starts_with("LDtk Grid") {
                    ldtk_entities.push(*entity);
                }
            }
            
            // Also collect entities with names starting with "CompositeCollider" or "Collider_"
            // These are generated by LDTK collider generation
            for (entity, name) in &world.names {
                if name.starts_with("CompositeCollider") || name.starts_with("Collider_") {
                    if !ldtk_entities.contains(entity) {
                        ldtk_entities.push(*entity);
                    }
                }
            }
            
            // Despawn all LDTK entities (this will also despawn children)
            if !ldtk_entities.is_empty() {

                info!("Removing {} existing LDTK entities before reload", ldtk_entities.len());
                for entity in ldtk_entities {
                    let _ = world.despawn(entity);
                }
            }
        }

        #[cfg(feature = "hecs")]
        {
            log::warn!("LdtkHotReloader: remove_existing_ldtk_entities not supported in HECS/Generic backend mode.");
        }
    }
}

impl Drop for LdtkHotReloader {
    fn drop(&mut self) {
        // Watcher will be automatically dropped and stop watching
        info!("LdtkHotReloader dropped, stopped watching {} files", self.watched_files.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reloader_creation() {
        let reloader = LdtkHotReloader::new();
        assert_eq!(reloader.watched_files().len(), 0);
    }
}
