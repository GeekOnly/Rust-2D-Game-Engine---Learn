use ecs::World;
use crate::tilemap_error::TilemapError;
use std::path::PathBuf;

pub struct ColliderGenerator;

impl ColliderGenerator {
    /// Regenerate colliders for a map
    pub fn regenerate_colliders(
        load_map: &mut crate::map_manager::LoadedMap,
        world: &mut World,
        collision_value: i64,
        path: &PathBuf,
    ) -> Result<usize, TilemapError> {
        let grid_entity = load_map.grid_entity;
        
        // Remove old colliders
        for &collider in &load_map.collider_entities {
            world.despawn(collider);
        }
        load_map.collider_entities.clear();
        
        // Generate new colliders
        let colliders = match ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
            path,
            world,
            collision_value,
            None,
            None,
        ) {
            Ok(colliders) => colliders,
            Err(e) => {
                let error = TilemapError::ColliderGenerationFailed(e);
                error.log_error();
                return Err(error);
            }
        };
        
        // Set as children of Grid
        for &collider in &colliders {
            world.set_parent(collider, Some(grid_entity));
        }
        
        // Update tracking
        load_map.collider_entities = colliders.clone();
        
        Ok(colliders.len())
    }
    
     /// Clean up colliders
    pub fn clean_up_colliders(
        load_map: &mut crate::map_manager::LoadedMap,
        world: &mut World
    ) -> usize {
        let count = load_map.collider_entities.len();
        
        // Remove colliders
        for &collider in &load_map.collider_entities {
            world.despawn(collider);
        }
        
        // Clear tracking
        load_map.collider_entities.clear();
        
        count
    }
}
