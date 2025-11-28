use crate::{World, Entity, Tilemap, TileSet, Tile, Transform};
use ldtk_rust::{Project, LayerInstance, TileInstance};
use std::collections::HashMap;
use std::path::Path;

/// LDTK file loader
pub struct LdtkLoader;

impl LdtkLoader {
    /// Load an LDTK project file and spawn entities into the world
    pub fn load_project(path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        let project = Project::load_project(path.as_ref())
            .map_err(|e| format!("Failed to load LDTK project: {}", e))?;

        let mut entities = Vec::new();

        // Load each level in the project
        for level in &project.levels {
            let level_entities = Self::load_level(&project, level, world)?;
            entities.extend(level_entities);
        }

        Ok(entities)
    }

    /// Load a single LDTK level
    fn load_level(
        project: &Project,
        level: &ldtk_rust::Level,
        world: &mut World,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        // Process each layer in the level
        for layer_instance in &level.layer_instances {
            if let Some(entity) = Self::load_layer(project, layer_instance, world)? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Load a single layer instance
    fn load_layer(
        project: &Project,
        layer: &LayerInstance,
        world: &mut World,
    ) -> Result<Option<Entity>, String> {
        // Only process tile layers for now
        if layer.layer_instance_type != "Tiles" && layer.layer_instance_type != "IntGrid" {
            return Ok(None);
        }

        let entity = world.spawn();

        // Create tilemap component
        let width = (layer.c_wid) as u32;
        let height = (layer.c_hei) as u32;
        let mut tilemap = Tilemap::new(
            &layer.identifier,
            format!("tileset_{}", layer.layer_def_uid),
            width,
            height,
        );

        // Set layer properties
        tilemap.z_order = layer.layer_def_uid as i32;
        tilemap.visible = layer.visible;
        tilemap.opacity = layer.opacity;

        // Load tiles from grid tiles
        for tile_instance in &layer.grid_tiles {
            let tile_x = (tile_instance.px[0] / layer.grid_size) as u32;
            let tile_y = (tile_instance.px[1] / layer.grid_size) as u32;

            let tile = Tile {
                tile_id: tile_instance.t as u32,
                flip_h: tile_instance.f & 1 != 0,
                flip_v: tile_instance.f & 2 != 0,
                flip_d: false,
            };

            tilemap.set_tile(tile_x, tile_y, tile);
        }

        // Create tileset if needed
        if let Some(tileset_def_uid) = layer.tileset_def_uid {
            if let Some(tileset_def) = project.defs.tilesets.iter().find(|ts| ts.uid == tileset_def_uid) {
                let tileset_entity = world.spawn();
                
                let tileset = TileSet::new(
                    &tileset_def.identifier,
                    tileset_def.rel_path.as_deref().unwrap_or(""),
                    format!("tileset_{}", tileset_def.uid),
                    tileset_def.tile_grid_size as u32,
                    tileset_def.tile_grid_size as u32,
                    (tileset_def.px_wid / tileset_def.tile_grid_size) as u32,
                    tileset_def.c_hei as u32 * (tileset_def.px_wid / tileset_def.tile_grid_size) as u32,
                );

                world.tilesets.insert(tileset_entity, tileset);
                world.names.insert(tileset_entity, format!("TileSet: {}", tileset_def.identifier));
            }
        }

        // Add components to entity
        world.tilemaps.insert(entity, tilemap);
        world.names.insert(entity, format!("Layer: {}", layer.identifier));
        
        // Add transform at layer offset
        let transform = Transform::with_position(
            layer.px_total_offset_x as f32,
            layer.px_total_offset_y as f32,
            0.0,
        );
        world.transforms.insert(entity, transform);

        Ok(Some(entity))
    }

    /// Load LDTK entities (not tile layers)
    pub fn load_entities(
        project: &Project,
        level: &ldtk_rust::Level,
        world: &mut World,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        for layer_instance in &level.layer_instances {
            if layer_instance.layer_instance_type == "Entities" {
                for entity_instance in &layer_instance.entity_instances {
                    let entity = world.spawn();
                    
                    // Add transform
                    let transform = Transform::with_position(
                        entity_instance.px[0] as f32,
                        entity_instance.px[1] as f32,
                        0.0,
                    );
                    world.transforms.insert(entity, transform);
                    world.names.insert(entity, entity_instance.identifier.clone());

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}
