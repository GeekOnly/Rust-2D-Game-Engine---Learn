use crate::{World, Entity, Tilemap, Transform, Collider, Rigidbody2D};
use serde_json::Value;
use std::path::Path;

/// Normalize texture path - extract filename from absolute paths
fn normalize_texture_path(path: &str) -> String {
    // Check if it's an absolute path (Windows or Unix style)
    if path.contains(":\\") || path.starts_with('/') {
        // Extract just the filename
        if let Some(filename) = Path::new(path).file_name() {
            if let Some(name) = filename.to_str() {
                // Return as assets/filename
                log::info!("LdtkLoader: Normalized absolute path '{}' to 'assets/{}'", path, name);
                return format!("assets/{}", name);
            }
        }
    }

    // Already relative, return as-is
    path.to_string()
}

/// LDTK file loader
/// 
/// Loads LDtk JSON files directly using serde_json
/// Compatible with LDtk 1.5.3+
pub struct LdtkLoader;

impl LdtkLoader {
    /// Load an LDTK project file with Grid component and auto-generated colliders
    /// Returns (grid_entity, tilemap_entities, collider_entities)
    pub fn load_project_with_grid_and_colliders(
        path: impl AsRef<Path>,
        world: &mut World,
        auto_generate_colliders: bool,
        collision_value: i64,
    ) -> Result<(Entity, Vec<Entity>, Vec<Entity>), String> {
        // Load grid and tilemaps
        let (grid_entity, tilemap_entities) = Self::load_project_with_grid(path.as_ref(), world)?;
        
        let mut collider_entities = Vec::new();
        
        if auto_generate_colliders {
            // Generate colliders
            match Self::generate_composite_colliders_from_intgrid(
                path.as_ref(),
                world,
                collision_value,
            ) {
                Ok(colliders) => {
                    // Set colliders as children of Grid
                    for &collider in &colliders {
                        world.set_parent(collider, Some(grid_entity));
                    }
                    
                    collider_entities = colliders;
                    log::info!("Auto-generated {} colliders for map", collider_entities.len());
                }
                Err(e) => {
                    log::warn!("Failed to auto-generate colliders: {}", e);
                }
            }
        }
        
        Ok((grid_entity, tilemap_entities, collider_entities))
    }

    /// Load an LDTK project file with Grid component
    /// Returns (grid_entity, tilemap_entities)
    pub fn load_project_with_grid(
        path: impl AsRef<Path>,
        world: &mut World,
    ) -> Result<(Entity, Vec<Entity>), String> {
        // Load the project JSON
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: Value = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        // Create Grid entity (parent)
        let grid_entity = world.spawn();
        
        // Set name with file name for clarity
        let file_name = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");
        world.names.insert(grid_entity, format!("LDtk Grid - {}", file_name));
        
        // Get grid size from LDtk
        let grid_size = project["defaultGridSize"]
            .as_i64()
            .unwrap_or(8) as f32;
        
        // Create Grid component
        // Unity standard: 100 pixels = 1 world unit (1 meter)
        let pixels_per_unit = 100.0;
        let cell_world_size = grid_size / pixels_per_unit;
        
        let grid = crate::Grid {
            cell_size: (cell_world_size, cell_world_size, 0.0),  // 2D grid (no depth)
            cell_gap: (0.0, 0.0),
            layout: crate::GridLayout::Rectangle,
            swizzle: crate::CellSwizzle::XYZ,
            plane: crate::GridPlane::XY,  // Default horizontal plane - can be changed to XZ for 3D
        };
        world.grids.insert(grid_entity, grid);
        
        // Position grid at origin
        world.transforms.insert(
            grid_entity,
            crate::Transform::with_position(0.0, 0.0, 0.0),
        );
        
        log::info!("Created LDtk Grid with cell size: {}x{}", 
            grid_size / pixels_per_unit, grid_size / pixels_per_unit);
        
        // Load levels as children of Grid
        let mut tilemap_entities = Vec::new();
        let levels = project["levels"]
            .as_array()
            .ok_or("No levels found in LDTK file")?;

        for level in levels {
            let entities = Self::load_level_as_children(
                level,
                &project,
                world,
                grid_entity,
                path.as_ref(),
            )?;
            tilemap_entities.extend(entities);
        }

        if tilemap_entities.is_empty() {
            log::warn!("No entities loaded from LDTK file. Check if levels have layers with data.");
        }

        Ok((grid_entity, tilemap_entities))
    }

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
                                        let tileset_path_abs = tileset_path.to_string_lossy().to_string();
                                        let tileset_path_str = normalize_texture_path(&tileset_path_abs);

                                        log::info!("  Tileset: {} -> {} (normalized to: {})", tileset_rel_path, tileset_path_abs, tileset_path_str);
                                        
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
                    // Unity standard: 100 pixels = 1 world unit (consistent with tilemap rendering)
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
    
    /// Generate colliders from IntGrid layer
    /// Creates static collider entities for each solid tile
    pub fn generate_colliders_from_intgrid(
        path: impl AsRef<Path>,
        world: &mut World,
        collision_value: i64,
    ) -> Result<Vec<Entity>, String> {
        // Load the project JSON
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: Value = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        let mut collider_entities = Vec::new();
        let pixels_per_unit = 100.0; // Unity standard: 100 pixels = 1 world unit

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
                // Only process IntGrid layers
                let layer_type = layer["__type"].as_str().unwrap_or("");
                if layer_type != "IntGrid" {
                    continue;
                }
                
                // Get layer properties
                let identifier = layer["__identifier"].as_str().unwrap_or("Unknown");
                let width = layer["__cWid"].as_i64().unwrap_or(0) as u32;
                let height = layer["__cHei"].as_i64().unwrap_or(0) as u32;
                let grid_size = layer["__gridSize"].as_i64().unwrap_or(8) as f32;
                
                let px_offset_x = layer["__pxTotalOffsetX"].as_i64().unwrap_or(0) as f32;
                let px_offset_y = layer["__pxTotalOffsetY"].as_i64().unwrap_or(0) as f32;
                
                // Get IntGrid CSV data
                let intgrid_csv = layer["intGridCsv"].as_array();
                
                if let Some(intgrid) = intgrid_csv {
                    log::info!("Generating colliders for IntGrid layer '{}' ({}x{})", identifier, width, height);
                    
                    let mut collider_count = 0;
                    
                    // Create collider for each solid tile
                    for y in 0..height {
                        for x in 0..width {
                            let index = (y * width + x) as usize;
                            if index >= intgrid.len() {
                                continue;
                            }
                            
                            let value = intgrid[index].as_i64().unwrap_or(0);
                            
                            // Check if this tile should have collision
                            if value == collision_value {
                                // Create collider entity
                                let entity = world.spawn();
                                
                                // Calculate world position
                                let total_px_x = level_world_x + px_offset_x + (x as f32 * grid_size);
                                let total_px_y = level_world_y + px_offset_y + (y as f32 * grid_size);
                                let world_x = total_px_x / pixels_per_unit;
                                let world_y = -total_px_y / pixels_per_unit;
                                
                                // Position at tile center
                                let tile_size = grid_size / pixels_per_unit;
                                let center_x = world_x + tile_size / 2.0;
                                let center_y = world_y - tile_size / 2.0;
                                
                                let transform = Transform::with_position(center_x, center_y, 0.0);
                                world.transforms.insert(entity, transform);
                                
                                // Add collider (size = 1 world unit = 1 tile)
                                let collider = Collider::new(tile_size, tile_size);
                                world.colliders.insert(entity, collider);
                                
                                // Add kinematic rigidbody (static, doesn't move)
                                let rigidbody = Rigidbody2D {
                                    velocity: (0.0, 0.0),
                                    gravity_scale: 0.0,
                                    mass: 1.0,
                                    is_kinematic: true,
                                    freeze_rotation: true,
                                    enable_ccd: false, // Static objects don't need CCD
                                };
                                world.rigidbodies.insert(entity, rigidbody);
                                
                                // Add name for debugging
                                world.names.insert(entity, format!("Collider_{}_{}", x, y));
                                
                                collider_entities.push(entity);
                                collider_count += 1;
                            }
                        }
                    }
                    
                    log::info!("Created {} colliders for layer '{}'", collider_count, identifier);
                }
            }
        }

        Ok(collider_entities)
    }
    
    /// Load a level as children of Grid entity
    fn load_level_as_children(
        level: &Value,
        project: &Value,
        world: &mut World,
        grid_parent: Entity,
        ldtk_path: &Path,
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
                let mut tilemap = crate::Tilemap::new(
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
                                    let ldtk_dir = ldtk_path.parent().unwrap_or(std::path::Path::new("."));
                                    let tileset_path = ldtk_dir.join(tileset_rel_path);
                                    let tileset_path_abs = tileset_path.to_string_lossy().to_string();
                                    let tileset_path_str = normalize_texture_path(&tileset_path_abs);

                                    log::info!("  Tileset: {} -> {} (normalized to: {})", tileset_rel_path, tileset_path_abs, tileset_path_str);
                                    
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
                // Unity standard: 100 pixels = 1 world unit (consistent with tilemap rendering)
                let pixels_per_unit = 100.0;
                let total_px_x = level_world_x + px_offset_x;
                let total_px_y = level_world_y + px_offset_y;
                let world_x = total_px_x / pixels_per_unit;
                let world_y = -total_px_y / pixels_per_unit;
                
                let transform = crate::Transform::with_position(
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

        Ok(entities)
    }

    /// Generate optimized composite colliders from IntGrid layer
    /// Merges adjacent tiles into larger rectangles for better performance
    pub fn generate_composite_colliders_from_intgrid(
        path: impl AsRef<Path>,
        world: &mut World,
        collision_value: i64,
    ) -> Result<Vec<Entity>, String> {
        // Load the project JSON
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK JSON: {}", e))?;
        
        let project: Value = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        let mut collider_entities = Vec::new();
        let pixels_per_unit = 100.0; // Unity standard: 100 pixels = 1 world unit

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
                // Only process IntGrid layers
                let layer_type = layer["__type"].as_str().unwrap_or("");
                if layer_type != "IntGrid" {
                    continue;
                }
                
                // Get layer properties
                let identifier = layer["__identifier"].as_str().unwrap_or("Unknown");
                let width = layer["__cWid"].as_i64().unwrap_or(0) as u32;
                let height = layer["__cHei"].as_i64().unwrap_or(0) as u32;
                let grid_size = layer["__gridSize"].as_i64().unwrap_or(8) as f32;
                
                let px_offset_x = layer["__pxTotalOffsetX"].as_i64().unwrap_or(0) as f32;
                let px_offset_y = layer["__pxTotalOffsetY"].as_i64().unwrap_or(0) as f32;
                
                // Get IntGrid CSV data
                let intgrid_csv = layer["intGridCsv"].as_array();
                
                if let Some(intgrid) = intgrid_csv {
                    log::info!("Generating composite colliders for IntGrid layer '{}' ({}x{})", identifier, width, height);
                    
                    // Convert to 2D grid
                    let mut grid = vec![vec![false; width as usize]; height as usize];
                    for y in 0..height {
                        for x in 0..width {
                            let index = (y * width + x) as usize;
                            if index < intgrid.len() {
                                let value = intgrid[index].as_i64().unwrap_or(0);
                                grid[y as usize][x as usize] = value == collision_value;
                            }
                        }
                    }
                    
                    // Find rectangles using greedy algorithm
                    let rectangles = find_rectangles(&mut grid, width, height);
                    
                    log::info!("Found {} composite rectangles (reduced from {} tiles)", 
                        rectangles.len(), 
                        grid.iter().flatten().filter(|&&v| v).count()
                    );
                    
                    // Create collider for each rectangle
                    for rect in rectangles {
                        let entity = world.spawn();
                        
                        // Calculate world position (center of rectangle)
                        let total_px_x = level_world_x + px_offset_x + (rect.x as f32 * grid_size);
                        let total_px_y = level_world_y + px_offset_y + (rect.y as f32 * grid_size);
                        let world_x = total_px_x / pixels_per_unit;
                        let world_y = -total_px_y / pixels_per_unit;
                        
                        // Calculate size in world units
                        let rect_width = rect.width as f32 * grid_size / pixels_per_unit;
                        let rect_height = rect.height as f32 * grid_size / pixels_per_unit;
                        
                        // Position at rectangle center
                        let center_x = world_x + rect_width / 2.0;
                        let center_y = world_y - rect_height / 2.0;
                        
                        let transform = Transform::with_position(center_x, center_y, 0.0);
                        world.transforms.insert(entity, transform);
                        
                        // Add collider with rectangle size
                        let collider = Collider::new(rect_width, rect_height);
                        world.colliders.insert(entity, collider);
                        
                        // Add kinematic rigidbody (static, doesn't move)
                        let rigidbody = Rigidbody2D {
                            velocity: (0.0, 0.0),
                            gravity_scale: 0.0,
                            mass: 1.0,
                            is_kinematic: true,
                            freeze_rotation: true,
                            enable_ccd: false, // Static objects don't need CCD
                        };
                        world.rigidbodies.insert(entity, rigidbody);
                        
                        // Add name for debugging
                        world.names.insert(entity, format!("CompositeCollider_{}x{}", rect.width, rect.height));
                        
                        collider_entities.push(entity);
                    }
                    
                    log::info!("Created {} composite colliders for layer '{}'", collider_entities.len(), identifier);
                }
            }
        }

        Ok(collider_entities)
    }
}

/// Rectangle in grid coordinates
#[derive(Debug, Clone)]
struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// Find rectangles in a 2D grid using greedy meshing algorithm
/// This finds the largest possible rectangles to minimize collider count
fn find_rectangles(grid: &mut Vec<Vec<bool>>, width: u32, height: u32) -> Vec<Rectangle> {
    let mut rectangles = Vec::new();
    
    for y in 0..height {
        for x in 0..width {
            if grid[y as usize][x as usize] {
                // Found a solid tile, find the largest rectangle starting here
                let rect = find_largest_rectangle(grid, x, y, width, height);
                
                // Mark tiles as used
                for dy in 0..rect.height {
                    for dx in 0..rect.width {
                        grid[(rect.y + dy) as usize][(rect.x + dx) as usize] = false;
                    }
                }
                
                rectangles.push(rect);
            }
        }
    }
    
    rectangles
}

/// Find the largest rectangle starting at (x, y) using greedy approach
/// Tries both horizontal-first and vertical-first expansion and picks the larger one
fn find_largest_rectangle(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    // Strategy 1: Expand horizontally first, then vertically
    let rect1 = expand_horizontal_first(grid, x, y, width, height);
    
    // Strategy 2: Expand vertically first, then horizontally
    let rect2 = expand_vertical_first(grid, x, y, width, height);
    
    // Pick the rectangle with larger area
    let area1 = rect1.width * rect1.height;
    let area2 = rect2.width * rect2.height;
    
    if area1 >= area2 {
        rect1
    } else {
        rect2
    }
}

/// Expand horizontally first, then vertically
fn expand_horizontal_first(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    // Find maximum width
    let mut rect_width = 1;
    while x + rect_width < width && grid[y as usize][(x + rect_width) as usize] {
        rect_width += 1;
    }
    
    // Find maximum height with this width
    let mut rect_height = 1;
    'height_loop: while y + rect_height < height {
        // Check if entire row is solid
        for dx in 0..rect_width {
            if !grid[(y + rect_height) as usize][(x + dx) as usize] {
                break 'height_loop;
            }
        }
        rect_height += 1;
    }
    
    Rectangle {
        x,
        y,
        width: rect_width,
        height: rect_height,
    }
}

/// Expand vertically first, then horizontally
fn expand_vertical_first(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    // Find maximum height
    let mut rect_height = 1;
    while y + rect_height < height && grid[(y + rect_height) as usize][x as usize] {
        rect_height += 1;
    }
    
    // Find maximum width with this height
    let mut rect_width = 1;
    'width_loop: while x + rect_width < width {
        // Check if entire column is solid
        for dy in 0..rect_height {
            if !grid[(y + dy) as usize][(x + rect_width) as usize] {
                break 'width_loop;
            }
        }
        rect_width += 1;
    }
    
    Rectangle {
        x,
        y,
        width: rect_width,
        height: rect_height,
    }
}
