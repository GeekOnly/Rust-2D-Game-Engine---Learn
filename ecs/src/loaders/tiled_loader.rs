use crate::{World, Entity, Tilemap, TileSet, Tile, TileData, Transform};
use tiled::{Loader, Map, TileLayer, PropertyValue};
use std::path::Path;
use std::collections::HashMap;

/// Tiled/TMX file loader
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
            if let Some(tile_layer) = layer.as_tile_layer() {
                if let Some(entity) = Self::load_tile_layer(&map, tile_layer, layer_index, world)? {
                    entities.push(entity);
                }
            }
        }

        // Load object layers
        for layer in map.layers() {
            if let Some(object_layer) = layer.as_object_layer() {
                let object_entities = Self::load_object_layer(object_layer, world)?;
                entities.extend(object_entities);
            }
        }

        entities.extend(tileset_entities);
        Ok(entities)
    }

    /// Load tilesets from the map
    fn load_tilesets(map: &Map, world: &mut World) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        for tileset in map.tilesets() {
            let entity = world.spawn();

            let mut tile_set = TileSet::new(
                tileset.name.as_str(),
                tileset.image.as_ref().map(|img| img.source.to_string_lossy().to_string()).unwrap_or_default(),
                format!("tileset_{}", tileset.first_gid),
                tileset.tile_width,
                tileset.tile_height,
                tileset.columns,
                tileset.tilecount.unwrap_or(0),
            );

            tile_set.spacing = tileset.spacing;
            tile_set.margin = tileset.margin;

            // Load tile-specific data
            for (tile_id, tile) in tileset.tiles() {
                let mut tile_data = TileData {
                    id: tile_id,
                    x: 0,
                    y: 0,
                    width: tileset.tile_width,
                    height: tileset.tile_height,
                    properties: HashMap::new(),
                };

                // Load custom properties
                for (key, value) in &tile.properties {
                    let value_str = match value {
                        PropertyValue::BoolValue(b) => b.to_string(),
                        PropertyValue::FloatValue(f) => f.to_string(),
                        PropertyValue::IntValue(i) => i.to_string(),
                        PropertyValue::StringValue(s) => s.clone(),
                        PropertyValue::ColorValue(c) => format!("#{:08X}", c),
                        PropertyValue::FileValue(f) => f.to_string_lossy().to_string(),
                        PropertyValue::ObjectValue(o) => o.to_string(),
                        PropertyValue::ClassValue { .. } => String::from("class"),
                    };
                    tile_data.properties.insert(key.clone(), value_str);
                }

                // Calculate tile coordinates in tileset
                if let Some((x, y)) = tile_set.get_tile_coords(tile_id) {
                    tile_data.x = x;
                    tile_data.y = y;
                }

                tile_set.tiles.insert(tile_id, tile_data);
            }

            world.tilesets.insert(entity, tile_set);
            world.names.insert(entity, format!("TileSet: {}", tileset.name));

            entities.push(entity);
        }

        Ok(entities)
    }

    /// Load a tile layer
    fn load_tile_layer(
        map: &Map,
        layer: TileLayer,
        layer_index: usize,
        world: &mut World,
    ) -> Result<Option<Entity>, String> {
        let entity = world.spawn();

        let width = layer.width().unwrap_or(0);
        let height = layer.height().unwrap_or(0);

        let mut tilemap = Tilemap::new(
            layer.name(),
            "tileset_0", // Will be updated based on actual tileset
            width,
            height,
        );

        tilemap.z_order = layer_index as i32;
        tilemap.visible = layer.visible;
        tilemap.opacity = layer.opacity;

        // Load tiles
        for y in 0..height {
            for x in 0..width {
                if let Some(tile_data) = layer.get_tile(x as i32, y as i32) {
                    let gid = tile_data.id();
                    
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

        // Add components
        world.tilemaps.insert(entity, tilemap);
        world.names.insert(entity, format!("Layer: {}", layer.name()));

        // Add transform at layer offset
        let transform = Transform::with_position(
            layer.offset_x(),
            layer.offset_y(),
            0.0,
        );
        world.transforms.insert(entity, transform);

        Ok(Some(entity))
    }

    /// Load object layer
    fn load_object_layer(
        layer: tiled::ObjectLayer,
        world: &mut World,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        for object in layer.objects() {
            let entity = world.spawn();

            // Add transform
            let transform = Transform::with_position(
                object.x,
                object.y,
                0.0,
            );
            world.transforms.insert(entity, transform);
            world.names.insert(entity, object.name.clone());

            entities.push(entity);
        }

        Ok(entities)
    }
}
