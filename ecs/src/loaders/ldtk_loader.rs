use crate::{World, Entity, Tilemap, TileSet, Transform, Collider, Rigidbody2D};
use std::path::Path;
use crate::traits::{EcsWorld, ComponentAccess};
use crate::components::ldtk_map::LdtkJson;

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
/// Loads LDtk JSON files directly using typed structs
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
        layer_filter: Option<&str>,
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
                layer_filter,
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
        
        let project: LdtkJson = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        // Create Grid entity (parent)
        let grid_entity = world.spawn();
        
        // Set name with file name for clarity
        let file_name = path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");
        let _ = ComponentAccess::<String>::insert(world, grid_entity, format!("LDtk Grid - {}", file_name));
        
        // Use default grid size from LDtk for world scale
        let grid_size = project.default_grid_size as f32;
        let pixels_per_unit = grid_size; // 1 tile = 1 world unit
        
        let grid = crate::Grid {
            cell_size: (1.0, 1.0, 0.0),  // 1 world unit per cell
            cell_gap: (0.0, 0.0),
            layout: crate::GridLayout::Rectangle,
            swizzle: crate::CellSwizzle::XYZ,
            plane: crate::GridPlane::XY, 
        };
        let _ = ComponentAccess::<crate::Grid>::insert(world, grid_entity, grid);
        
        // Position grid at origin
        let _ = ComponentAccess::<crate::Transform>::insert(
            world,
            grid_entity,
            crate::Transform::with_position(0.0, 0.0, 0.0),
        );
        
        log::info!("Created LDtk Grid with cell size 1.0x1.0 world units (tilemap tiles: {}px at {:.0} PPU)", 
            grid_size, pixels_per_unit);
        
        // Load levels as children of Grid
        let mut tilemap_entities = Vec::new();

        for level in &project.levels {
            // Get layer instances
            if let Some(layer_instances) = &level.layer_instances {
                // Reverse to render bottom layers first (LDtk stores top-to-bottom)
                // Actually, engine usually renders based on Z or draw order. 
                // LDtk layers: [0] is TOP layer.
                // We should iterate in reverse if we want [0] to be last spawned (on top)?
                // Or just rely on Z-index. Let's spawn in order but adjust Z if needed.
                // For now, standard iteration.
                
                for layer in layer_instances.iter().rev() {
                     // Get grid size for this layer
                    let layer_grid_size = layer.__grid_size as u32;
                    let width = layer.__c_wid as u32;
                    let height = layer.__c_hei as u32;
                    
                    match layer.__type.as_str() {
                        "Tiles" | "AutoLayer" => {
                            let tiles = if !layer.auto_layer_tiles.is_empty() {
                                &layer.auto_layer_tiles
                            } else {
                                &layer.grid_tiles
                            };

                            if tiles.is_empty() {
                                continue;
                            }

                            let entity = world.spawn();
                            world.set_parent(entity, Some(grid_entity));

                            // Create tilemap
                            let mut tilemap = Tilemap::new(
                                &layer.__identifier,
                                format!("tileset_{}", layer.__tileset_def_uid.unwrap_or(0)),
                                width,
                                height,
                            );

                            for tile_data in tiles {
                                // tile.px is [x, y] in pixels
                                let px = tile_data.px;
                                let tile_id = tile_data.t as u32;
                                let flip_bits = tile_data.f; // 0=none, 1=flipX, 2=flipY

                                // Convert to grid coords
                                let grid_x = (px[0] as u32) / layer_grid_size;
                                let grid_y = (px[1] as u32) / layer_grid_size;
                                
                                let tile = crate::Tile {
                                    tile_id,
                                    flip_h: (flip_bits & 1) != 0,
                                    flip_v: (flip_bits & 2) != 0,
                                    flip_d: false, // LDtk doesn't support diagonal flip commonly
                                };
                                
                                tilemap.set_tile(grid_x, grid_y, tile);
                            }

                            // Find and setup tileset
                            if let Some(uid) = layer.__tileset_def_uid {
                                if let Some(tileset_def) = project.defs.tilesets.iter().find(|t| t.uid == uid) {
                                     if let Some(rel_path) = &tileset_def.rel_path {
                                        // Path logic
                                        let ldtk_dir = path.as_ref().parent().unwrap_or(Path::new("."));
                                        let tileset_path = ldtk_dir.join(rel_path);
                                        let tileset_path_abs = tileset_path.to_string_lossy().to_string();
                                        let tileset_path_str = normalize_texture_path(&tileset_path_abs);

                                        let columns = (tileset_def.px_wid / tileset_def.tile_grid_size) as u32;
                                        // Calculate total tiles
                                        let rows = (tileset_def.px_hei / tileset_def.tile_grid_size) as u32;
                                        let tile_count = columns * rows;

                                        let tileset = TileSet::new(
                                            format!("tileset_{}", uid),
                                            tileset_path_str,
                                            format!("tileset_{}", uid),
                                            layer_grid_size,
                                            layer_grid_size,
                                            columns,
                                            tile_count,
                                        );
                                        let _ = ComponentAccess::<TileSet>::insert(world, entity, tileset);
                                    }
                                }
                            }

                            let _ = ComponentAccess::<Tilemap>::insert(world, entity, tilemap);
                            let _ = ComponentAccess::<String>::insert(world, entity, format!("LDTK Layer: {}", layer.__identifier));

                            // Calculate Transform
                            // Level world pos + Layer offset
                            let total_px_x = (level.world_x + layer.__px_total_offset_x) as f32;
                            let total_px_y = (level.world_y + layer.__px_total_offset_y) as f32;
                            
                            // To map LDtk (Top-Left) to Engine (Bottom-Left assumed, OR 2D Y-Up)
                            // In engine 2D usually Y is UP. LDtk is Y DOWN.
                            // So Y = -total_px_y / PPU.
                            let world_x = total_px_x / pixels_per_unit;
                            let world_y = -total_px_y / pixels_per_unit;

                            let transform = Transform::with_position(world_x, world_y, 0.0);
                            let _ = ComponentAccess::<Transform>::insert(world, entity, transform);

                            tilemap_entities.push(entity);
                        }
                        "Entities" => {
                            for entity_instance in &layer.entity_instances {
                                let entity = world.spawn();
                                world.set_parent(entity, Some(grid_entity));

                                // Calculate position
                                let entity_px_x = entity_instance.px[0] as f32;
                                let entity_px_y = entity_instance.px[1] as f32;
                                
                                let total_px_x = (level.world_x + layer.__px_total_offset_x) as f32 + entity_px_x;
                                let total_px_y = (level.world_y + layer.__px_total_offset_y) as f32 + entity_px_y;
                                
                                let world_x = total_px_x / pixels_per_unit;
                                let world_y = -total_px_y / pixels_per_unit;
                                
                                let mut transform = Transform::with_position(world_x, world_y, 0.0);
                                let _ = ComponentAccess::<Transform>::insert(world, entity, transform);
                                
                                // Name
                                let _ = ComponentAccess::<String>::insert(world, entity, entity_instance.__identifier.clone());

                                // Add LdtkEntity component with raw data
                                use crate::components::ldtk_entity::LdtkEntity; // Import here or top
                                use std::collections::HashMap;

                                let mut fields = HashMap::new();
                                for field in &entity_instance.field_instances {
                                    fields.insert(field.__identifier.clone(), field.__value.clone());
                                }

                                let ldtk_entity = LdtkEntity {
                                    identifier: entity_instance.__identifier.clone(),
                                    iid: entity_instance.iid.clone(),
                                    width: entity_instance.width,
                                    height: entity_instance.height,
                                    tags: entity_instance.__tags.clone(),
                                    fields,
                                };
                                let _ = ComponentAccess::<LdtkEntity>::insert(world, entity, ldtk_entity);
                                
                                tilemap_entities.push(entity);
                                
                                log::info!("Spawned Entity: {} at ({:.2}, {:.2})", entity_instance.__identifier, world_x, world_y);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok((grid_entity, tilemap_entities))
    }

    /// Load an LDTK project file and spawn entities into the world (Legacy/Simple wrapper)
    pub fn load_project(path: impl AsRef<Path>, world: &mut World) -> Result<Vec<Entity>, String> {
        let (grid, children) = Self::load_project_with_grid(path, world)?;
        let mut all = vec![grid];
        all.extend(children);
        Ok(all)
    }
    
    /// Generate optimized composite colliders from IntGrid layer
    /// 
    /// # Arguments
    /// * `path` - Path to LDtk file
    /// * `world` - ECS World
    /// * `collision_value` - IntGrid value that represents collision
    /// * `layer_filter` - Optional layer name filter. If None, processes all IntGrid layers.
    pub fn generate_composite_colliders_from_intgrid(
        path: impl AsRef<Path>,
        world: &mut World,
        collision_value: i64,
        layer_filter: Option<&str>,
    ) -> Result<Vec<Entity>, String> {
        let project_data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read LDTK file: {}", e))?;
        
        let project: LdtkJson = serde_json::from_str(&project_data)
            .map_err(|e| format!("Failed to parse LDTK JSON: {}", e))?;

        let mut collider_entities = Vec::new();
        let grid_size = project.default_grid_size as f32;
        let pixels_per_unit = grid_size; 

        for level in &project.levels {
             if let Some(layer_instances) = &level.layer_instances {
                for layer in layer_instances {
                    if layer.__type != "IntGrid" {
                        continue;
                    }
                    
                    // Filter by layer identifier if provided
                    if let Some(filter) = layer_filter {
                        if layer.__identifier != filter {
                            continue;
                        }
                    } else {
                        // Default behavior: ignore layers explicitly named "Visual" or "Art" if no filter?
                        // Or just process all? Let's check if collision value matches first.
                        // Actually, users might use IntGrid for other things. 
                        // Plan suggested "Safe default".
                        // Let's iterate all, assuming collision_value check is enough for basic usage.
                    }

                    // Check if IntGrid has values
                    if layer.int_grid_csv.is_empty() {
                        continue;
                    }
                    
                    let width = layer.__c_wid as u32;
                    let height = layer.__c_hei as u32;
                    let layer_grid_size = layer.__grid_size as f32;
                    
                    // Pre-scan to see if this layer actually contains the collision_value
                    // to avoid doing heavy lifting for non-collision layers
                    let has_collision_value = layer.int_grid_csv.iter().any(|&v| v as i64 == collision_value);
                    if !has_collision_value {
                        continue;
                    }

                    log::info!("Generating composite colliders for IntGrid layer '{}' ({}x{})", layer.__identifier, width, height);

                    // Convert to 2D grid
                    let mut grid = vec![vec![false; width as usize]; height as usize];
                    for y in 0..height {
                        for x in 0..width {
                            let index = (y * width + x) as usize;
                            if index < layer.int_grid_csv.len() {
                                let value = layer.int_grid_csv[index] as i64;
                                grid[y as usize][x as usize] = value == collision_value;
                            }
                        }
                    }

                    let rectangles = find_rectangles(&mut grid, width, height);
                    
                    log::info!("Found {} composite rectangles", rectangles.len());

                    for rect in rectangles {
                        let entity = world.spawn();
                        
                        // Calculate position
                        let total_px_x = (level.world_x + layer.__px_total_offset_x) as f32 + (rect.x as f32 * layer_grid_size);
                        let total_px_y = (level.world_y + layer.__px_total_offset_y) as f32 + (rect.y as f32 * layer_grid_size);
                        
                        let world_x = total_px_x / pixels_per_unit;
                        let world_y = -total_px_y / pixels_per_unit;
                        
                        let rect_width = rect.width as f32 * layer_grid_size / pixels_per_unit;
                        let rect_height = rect.height as f32 * layer_grid_size / pixels_per_unit;
                        
                        // Center
                        let center_x = world_x + rect_width / 2.0;
                        let center_y = world_y - rect_height / 2.0;
                        
                        let transform = Transform::with_position(center_x, center_y, 0.0);
                        let _ = ComponentAccess::<Transform>::insert(world, entity, transform);
                        
                        let collider = Collider::new(rect_width, rect_height);
                        let _ = ComponentAccess::<Collider>::insert(world, entity, collider);
                        
                        let rigidbody = Rigidbody2D {
                            velocity: (0.0, 0.0),
                            gravity_scale: 0.0,
                            mass: 1.0,
                            is_kinematic: true,
                            freeze_rotation: true,
                            enable_ccd: false,
                        };
                        let _ = ComponentAccess::<Rigidbody2D>::insert(world, entity, rigidbody);
                        let _ = ComponentAccess::<String>::insert(world, entity, format!("CompositeCollider_{}_{}", rect.x, rect.y)); // Use coords for unique name check? or just rect size
                        
                        collider_entities.push(entity);
                    }
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

fn find_largest_rectangle(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    let rect1 = expand_horizontal_first(grid, x, y, width, height);
    let rect2 = expand_vertical_first(grid, x, y, width, height);
    if (rect1.width * rect1.height) >= (rect2.width * rect2.height) { rect1 } else { rect2 }
}

fn expand_horizontal_first(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    let mut rect_width = 1;
    while x + rect_width < width && grid[y as usize][(x + rect_width) as usize] {
        rect_width += 1;
    }
    let mut rect_height = 1;
    'height_loop: while y + rect_height < height {
        for dx in 0..rect_width {
            if !grid[(y + rect_height) as usize][(x + dx) as usize] {
                break 'height_loop;
            }
        }
        rect_height += 1;
    }
    Rectangle { x, y, width: rect_width, height: rect_height }
}

fn expand_vertical_first(grid: &Vec<Vec<bool>>, x: u32, y: u32, width: u32, height: u32) -> Rectangle {
    let mut rect_height = 1;
    while y + rect_height < height && grid[(y + rect_height) as usize][x as usize] {
        rect_height += 1;
    }
    let mut rect_width = 1;
    'width_loop: while x + rect_width < width {
        for dy in 0..rect_height {
            if !grid[(y + dy) as usize][(x + rect_width) as usize] {
                break 'width_loop;
            }
        }
        rect_width += 1;
    }
    Rectangle { x, y, width: rect_width, height: rect_height }
}
