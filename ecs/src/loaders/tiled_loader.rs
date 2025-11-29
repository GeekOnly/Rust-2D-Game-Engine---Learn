use crate::{World, Entity, Tilemap, TileSet, Tile, Transform};
use tiled::{Loader, Map};
use std::path::Path;

/// Tiled/TMX file loader
/// 
/// Note: This is a simplified loader that creates basic tilemap entities from Tiled files.
/// It loads tile layers and creates tileset entities.
pub struct TiledLoader;

impl TiledLoader {
    /// Load a Tiled map file (.tmx) and spawn entities into the world
    pub fn load_map(path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        let mut loader = Loader::new();
        let map = loader
            .load_tmx_map(path.as_ref())
            .map_err(|e| format!("Failed to load TMX map: {}", e))?;

        let mut entities = Vec::new();

        // Load tilesets first
        let tileset_entities = Self::load_tilesets(&map, world)?;

        // Load tile layers
        for (layer_index, layer) in map.layers().enumerate() {
            let entity = world.spawn();

            let width = map.width;
            let height = map.height;

            let mut tilemap = Tilemap::new(
                format!("Layer {}", layer_index),
                "tileset_0",
                width,
                height,
            );

            tilemap.z_order = layer_index as i32;
            tilemap.visible = true;
            tilemap.opacity = 1.0;

            // Try to get tiles if this is a tile layer
            if let Some(tile_layer) = layer.as_tile_layer() {
                // Load tiles
                for y in 0..height {
                    for x in 0..width {
                        if let Some(layer_tile) = tile_layer.get_tile(x as i32, y as i32) {
                            let gid = layer_tile.id();
                            
                            // Extract flip flags from GID
                            const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
                            const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
                            const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;

                            let flip_h = (gid & FLIPPED_HORIZONTALLY_FLAG) != 0;
                            let flip_v = (gid & FLIPPED_VERTICALLY_FLAG) != 0;
                            let flip_d = (gid & FLIPPED_DIAGONALLY_FLAG) != 0;

                            // Remove flip flags to get actual tile ID
                            let tile_id = gid & !(FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG);

                            let tile = Tile {
                                tile_id,
                                flip_h,
                                flip_v,
                                flip_d,
                            };

                            tilemap.set_tile(x, y, tile);
                        }
                    }
                }
            }

            // Add components
            world.tilemaps.insert(entity, tilemap);
            world.names.insert(entity, format!("Tiled Layer {}", layer_index));

            // Add transform
            let transform = Transform::default();
            world.transforms.insert(entity, transform);

            entities.push(entity);
        }

        entities.extend(tileset_entities);
        Ok(entities)
    }

    /// Load tilesets from the map
    fn load_tilesets(map: &Map, world: &mut World) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        for (i, tileset) in map.tilesets().iter().enumerate() {
            let entity = world.spawn();

            let tile_set = TileSet::new(
                tileset.name.as_str(),
                tileset.image.as_ref().map(|img| img.source.to_string_lossy().to_string()).unwrap_or_default(),
                format!("tileset_{}_{}", i, tileset.name),
                tileset.tile_width,
                tileset.tile_height,
                tileset.columns,
                tileset.tilecount,
            );

            world.tilesets.insert(entity, tile_set);
            world.names.insert(entity, format!("TileSet: {}", tileset.name));

            entities.push(entity);
        }

        Ok(entities)
    }
}
