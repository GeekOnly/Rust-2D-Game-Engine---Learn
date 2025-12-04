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

                // Check if layer has tiles (either gridTiles or autoLayerTiles)
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

                    // Create tilemap
                    let mut tilemap = Tilemap::new(
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
                        let mut parsed_count = 0;
                        
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
                                    let tile = crate::Tile {
                                        tile_id,
                                        flip_h: (flip_flags & 1) != 0,
                                        flip_v: (flip_flags & 2) != 0,
                                        flip_d: false,
                                    };
                                    
                                    // Set tile in tilemap
                                    if tilemap.set_tile(grid_x, grid_y, tile) {
                                        parsed_count += 1;
                                    }
                                }
                            }
                        }
                        
                        log::info!("Layer '{}': parsed {}/{} tiles ({}x{} grid, {}px tiles)", 
                            identifier, parsed_count, tiles.len(), width, height, grid_size);
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
                                        // LDtk uses "../" relative to the .ldtk file
                                        let ldtk_dir = path.as_ref().parent().unwrap_or(std::path::Path::new("."));
                                        let tileset_path = ldtk_dir.join(tileset_rel_path);
                                        let tileset_path_str = tileset_path.to_string_lossy().to_string();
                                        
                                        log::info!("  Tileset: {} -> {}", tileset_rel_path, tileset_path_str);
                                        
                                        // Get tileset dimensions
                                        let tileset_width = tileset_def["pxWid"].as_i64().unwrap_or(256) as u32;
                                        let tileset_height = tileset_def["pxHei"].as_i64().unwrap_or(256) as u32;
                                        let columns = tileset_width / grid_size;
                                        let rows = tileset_height / grid_size;
                                        let tile_count = columns * rows;
                                        
                                        // Create TileSet component
                                        let tileset = crate::TileSet::new(
                                            format!("tileset_{}", tileset_uid),
                                            tileset_path_str.clone(),
                                            format!("tileset_{}", tileset_uid),
                                            grid_size,  // tile_width (from LDtk grid)
                                            grid_size,  // tile_height (from LDtk grid)
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

                    // Add transform at layer offset
                    // Convert pixel coordinates to world units (pixels / pixels_per_unit)
                    // Default pixels_per_unit = 100.0 (Unity standard)
                    let pixels_per_unit = 100.0;
                    // Combine level world position with layer offset
                    let total_px_x = level_world_x + px_offset_x;
                    let total_px_y = level_world_y + px_offset_y;
                    let world_x = total_px_x / pixels_per_unit;
                    let world_y = -total_px_y / pixels_per_unit; // Flip Y (LDtk uses top-left origin, engine uses bottom-left)
                    
                    let transform = Transform::with_position(
                        world_x,
                        world_y,
                        0.0,
                    );
                    world.transforms.insert(entity, transform);

                    entities.push(entity);
                } else {
                    log::debug!("Skipping empty layer: {}", identifier);
                }
            }
        }

        if entities.is_empty() {
            log::warn!("No entities loaded from LDTK file. Check if levels have layers with data.");
        }

        Ok(entities)
    }
}
