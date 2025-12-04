use crate::{World, Entity, Tilemap, Transform};
use serde_json::Value;
use std::path::Path;

/// LDTK file loader
/// 
/// Loads LDtk JSON files directly using serde_json
/// Compatible with LDtk 1.5.3+
pub struct LdtkLoader;

impl LdtkLoader {
    /// Load an LDTK project file and spawn entities into the world
    pub fn load_project(path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        // Load the project JSON
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: Value = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        let mut entities = Vec::new();

        // Get levels array
        let levels = project["levels"]
            .as_array()
            .ok_or("No levels found in LDTK file")?;

        // Load each level
        for level in levels {
            // Get layer instances
            let empty_vec = vec![];
            let layer_instances = level["layerInstances"]
                .as_array()
                .unwrap_or(&empty_vec);

            // Process each layer
            for layer in layer_instances {
                // Create a basic tilemap entity for each layer
                let entity = world.spawn();

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

                // Create tilemap
                let tilemap = Tilemap::new(
                    identifier,
                    format!("tileset_{}", layer_def_uid),
                    width,
                    height,
                );

                world.tilemaps.insert(entity, tilemap);
                world.names.insert(entity, format!("LDTK Layer: {}", identifier));

                // Add transform at layer offset
                let transform = Transform::with_position(
                    px_offset_x,
                    px_offset_y,
                    0.0,
                );
                world.transforms.insert(entity, transform);

                entities.push(entity);
            }
        }

        if entities.is_empty() {
            log::warn!("No entities loaded from LDTK file. Check if levels have layers with data.");
        }

        Ok(entities)
    }
}
