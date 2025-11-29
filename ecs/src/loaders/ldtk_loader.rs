use crate::{World, Entity, Tilemap, Transform};
use ldtk_rust::Project;
use std::path::Path;

/// LDTK file loader
/// 
/// Note: This is a simplified loader that creates basic tilemap entities from LDTK files.
/// Full tile data loading requires more complex API interaction with ldtk_rust 0.6.
pub struct LdtkLoader;

impl LdtkLoader {
    /// Load an LDTK project file and spawn entities into the world
    pub fn load_project(path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        // Load the project - ldtk_rust 0.6 uses new_from_file
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: Project = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        let mut entities = Vec::new();

        // Load each level in the project
        for level in &project.levels {
            // Process each layer in the level
            if let Some(layer_instances) = &level.layer_instances {
                for layer_instance in layer_instances {
                    // Create a basic tilemap entity for each layer
                    let entity = world.spawn();

                    let width = layer_instance.c_wid as u32;
                    let height = layer_instance.c_hei as u32;

                    let tilemap = Tilemap::new(
                        &layer_instance.identifier,
                        format!("tileset_{}", layer_instance.layer_def_uid),
                        width,
                        height,
                    );

                    world.tilemaps.insert(entity, tilemap);
                    world.names.insert(entity, format!("LDTK Layer: {}", layer_instance.identifier));

                    // Add transform at layer offset
                    let transform = Transform::with_position(
                        layer_instance.px_total_offset_x as f32,
                        layer_instance.px_total_offset_y as f32,
                        0.0,
                    );
                    world.transforms.insert(entity, transform);

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}
