//! Game Runtime Renderer
//!
//! Renders the game scene using Camera components from the ECS.
//! This is separate from the editor's scene view.

use ecs::{World, Entity, Camera, CameraProjection};
use egui;
use crate::texture_manager::TextureManager;

/// Render the game view using the main camera
pub fn render_game_view(
    ui: &mut egui::Ui,
    world: &World,
    texture_manager: &mut TextureManager,
) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);

    // Find the main camera (first active camera with lowest depth)
    let main_camera = find_main_camera(world);

    if let Some((camera_entity, camera, transform)) = main_camera {
        // Clear background
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                (camera.background_color[0] * 255.0) as u8,
                (camera.background_color[1] * 255.0) as u8,
                (camera.background_color[2] * 255.0) as u8,
                (camera.background_color[3] * 255.0) as u8,
            ),
        );

        // Render all entities
        render_entities(ui, world, camera, transform, rect, texture_manager);
    } else {
        // No camera found - show default view
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(30, 30, 35),
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Camera Found\n\nAdd a Camera component to an entity",
            egui::FontId::proportional(16.0),
            egui::Color32::from_rgb(150, 150, 150),
        );
    }
}

/// Find the main camera (lowest depth, active)
fn find_main_camera(world: &World) -> Option<(Entity, &Camera, &ecs::Transform)> {
    let mut cameras: Vec<_> = world.cameras.iter()
        .filter_map(|(entity, camera)| {
            // Check if entity is active
            if world.active.get(entity).copied().unwrap_or(true) {
                world.transforms.get(entity).map(|transform| (*entity, camera, transform))
            } else {
                None
            }
        })
        .collect();

    // Sort by depth (lowest first)
    cameras.sort_by_key(|(_, camera, _)| camera.depth);

    cameras.into_iter().next()
}

/// Render all entities visible to the camera
fn render_entities(
    ui: &mut egui::Ui,
    world: &World,
    camera: &Camera,
    camera_transform: &ecs::Transform,
    rect: egui::Rect,
    texture_manager: &mut TextureManager,
) {
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let ctx = ui.ctx().clone();

    // Get camera position
    let cam_pos = camera_transform.position;

    // Render based on projection mode
    match camera.projection {
        CameraProjection::Orthographic => {
            render_orthographic(world, &painter, &ctx, camera, cam_pos, center, texture_manager);
        }
        CameraProjection::Perspective => {
            render_perspective(world, &painter, &ctx, camera, cam_pos, center, texture_manager);
        }
    }
}

/// Render a tilemap in 2D
fn render_tilemap_2d(
    tilemap: &ecs::Tilemap,
    transform: &ecs::Transform,
    painter: &egui::Painter,
    ctx: &egui::Context,
    camera: &Camera,
    cam_pos: [f32; 3],
    center: egui::Pos2,
    zoom: f32,
    texture_manager: &mut TextureManager,
    world: &World,
    entity: ecs::Entity,
) {
    // Get tileset component
    let tileset = world.tilesets.get(&entity);
    
    // Get tile size (default to 8x8 if no tileset)
    let (tile_width, tile_height, texture_opt) = if let Some(ts) = tileset {
        // Normalize path separators (convert / to \ on Windows)
        let normalized_path = ts.texture_path.replace('/', std::path::MAIN_SEPARATOR_STR);
        let tex_path = std::path::Path::new(&normalized_path);
        let texture = texture_manager.load_texture(ctx, &ts.texture_id, tex_path);
        (ts.tile_width as f32, ts.tile_height as f32, texture)
    } else {
        (8.0, 8.0, None)
    };

    // Get tilemap position
    let tilemap_x = transform.position[0];
    let tilemap_y = transform.position[1];

    // Render each tile
    for y in 0..tilemap.height {
        for x in 0..tilemap.width {
            if let Some(tile) = tilemap.get_tile(x, y) {
                // Skip empty tiles
                if tile.is_empty() {
                    continue;
                }

                // Calculate world position (tile position in world space)
                // Tiles are positioned in world units (pixels / pixels_per_unit)
                // Use 8.0 to match LDtk cell size (8x8 pixels = 1x1 world units)
                let pixels_per_unit = 8.0;
                let tile_world_x = tilemap_x + (x as f32 * tile_width / pixels_per_unit);
                // Flip Y: LDtk uses top-left origin (Y down), engine uses bottom-left (Y up)
                let tile_world_y = tilemap_y - (y as f32 * tile_height / pixels_per_unit);
                
                // Convert to screen space
                let world_x = tile_world_x - cam_pos[0];
                let world_y = tile_world_y - cam_pos[1];

                let screen_x = center.x + world_x * zoom;
                let screen_y = center.y - world_y * zoom;

                // Calculate size in world units then convert to screen pixels
                let tile_world_width = tile_width / pixels_per_unit;
                let tile_world_height = tile_height / pixels_per_unit;
                let size = egui::vec2(tile_world_width * zoom, tile_world_height * zoom);
                let rect = egui::Rect::from_min_size(
                    egui::pos2(screen_x, screen_y),
                    size
                );

                // Render with texture if available
                if let (Some(texture), Some(ts)) = (texture_opt.as_ref(), tileset) {
                    // Get tile coordinates in tileset
                    if let Some((src_x, src_y)) = ts.get_tile_coords(tile.tile_id) {
                        // Calculate UV coordinates
                        let tex_size = texture.size();
                        let tex_width = tex_size[0] as f32;
                        let tex_height = tex_size[1] as f32;
                        
                        let u0 = src_x as f32 / tex_width;
                        let v0 = src_y as f32 / tex_height;
                        let u1 = u0 + (tile_width / tex_width);
                        let v1 = v0 + (tile_height / tex_height);

                        // Handle flip flags
                        let (u0, u1) = if tile.flip_h { (u1, u0) } else { (u0, u1) };
                        let (v0, v1) = if tile.flip_v { (v1, v0) } else { (v0, v1) };

                        // Create textured mesh
                        let mut mesh = egui::Mesh::with_texture(texture.id());
                        mesh.add_rect_with_uv(
                            rect,
                            egui::Rect::from_min_max(
                                egui::pos2(u0, v0),
                                egui::pos2(u1, v1)
                            ),
                            egui::Color32::WHITE,
                        );
                        painter.add(egui::Shape::mesh(mesh));
                    } else {
                        // Fallback to colored rectangle
                        let color = egui::Color32::from_rgb(
                            ((tile.tile_id * 37) % 255) as u8,
                            ((tile.tile_id * 73) % 255) as u8,
                            ((tile.tile_id * 131) % 255) as u8,
                        );
                        painter.rect_filled(rect, 0.0, color);
                    }
                } else {
                    // No texture - render as colored rectangles
                    let color = egui::Color32::from_rgb(
                        ((tile.tile_id * 37) % 255) as u8,
                        ((tile.tile_id * 73) % 255) as u8,
                        ((tile.tile_id * 131) % 255) as u8,
                    );
                    painter.rect_filled(rect, 0.0, color);
                }
            }
        }
    }
}

/// Render in orthographic mode (2D)
fn render_orthographic(
    world: &World,
    painter: &egui::Painter,
    ctx: &egui::Context,
    camera: &Camera,
    cam_pos: [f32; 3],
    center: egui::Pos2,
    texture_manager: &mut TextureManager,
) {
    // Calculate zoom based on orthographic_size and screen height
    // orthographic_size = half of the height the camera sees (in world units)
    // zoom = screen_height / (orthographic_size * 2)
    let screen_height = painter.clip_rect().height();
    let zoom = screen_height / (camera.orthographic_size * 2.0);

    // Render tilemaps first (background layers)
    for (&entity, tilemap) in &world.tilemaps {
        // Skip if not active or not visible
        if !world.active.get(&entity).copied().unwrap_or(true) || !tilemap.visible {
            continue;
        }

        // Get transform
        if let Some(transform) = world.transforms.get(&entity) {
            render_tilemap_2d(
                tilemap,
                transform,
                painter,
                ctx,
                camera,
                cam_pos,
                center,
                zoom,
                texture_manager,
                world,
                entity,
            );
        }
    }

    // Render all entities
    for (entity, transform) in &world.transforms {
        // Skip if not active
        if !world.active.get(entity).copied().unwrap_or(true) {
            continue;
        }

        // Calculate screen position (simple orthographic projection)
        let world_x = transform.position[0] - cam_pos[0];
        let world_y = transform.position[1] - cam_pos[1];

        let screen_x = center.x + world_x * zoom;
        let screen_y = center.y - world_y * zoom; // Flip Y axis

        // Check if entity has animated sprite (priority over regular sprite)
        let has_animated_sprite = world.animated_sprites.contains_key(entity);
        
        if has_animated_sprite {
            log::info!("Entity {} has animated sprite", entity);
            // Render animated sprite from sprite sheet
            if let (Some(animated_sprite), Some(sprite_sheet)) = 
                (world.animated_sprites.get(entity), world.sprite_sheets.get(entity)) {
                
                let frame_index = animated_sprite.get_frame_index();
                if let Some(frame) = sprite_sheet.get_frame(frame_index) {
                    let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
                    
                    // Calculate size based on frame dimensions
                    let aspect_ratio = frame.width as f32 / frame.height as f32;
                    let size = egui::vec2(
                        transform_scale.x * zoom * aspect_ratio,
                        transform_scale.y * zoom
                    );
                    
                    // Get sprite color if exists, otherwise white
                    let color = if let Some(sprite) = world.sprites.get(entity) {
                        egui::Color32::from_rgba_unmultiplied(
                            (sprite.color[0] * 255.0) as u8,
                            (sprite.color[1] * 255.0) as u8,
                            (sprite.color[2] * 255.0) as u8,
                            (sprite.color[3] * 255.0) as u8,
                        )
                    } else {
                        egui::Color32::WHITE
                    };

                    // Try to load and render texture
                    let texture_path = std::path::Path::new(&sprite_sheet.texture_path);
                    if let Some(texture) = texture_manager.load_texture(ctx, &sprite_sheet.texture_id, texture_path) {
                        // Calculate UV coordinates for the frame
                        let uv_min = egui::pos2(
                            frame.x as f32 / sprite_sheet.sheet_width as f32,
                            frame.y as f32 / sprite_sheet.sheet_height as f32,
                        );
                        let uv_max = egui::pos2(
                            (frame.x + frame.width) as f32 / sprite_sheet.sheet_width as f32,
                            (frame.y + frame.height) as f32 / sprite_sheet.sheet_height as f32,
                        );
                        
                        let rect = egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size);
                        
                        // Draw the texture with UV coordinates
                        painter.image(
                            texture.id(),
                            rect,
                            egui::Rect::from_min_max(uv_min, uv_max),
                            color,
                        );
                    } else {
                        // Fallback: draw colored rectangle
                        painter.rect_filled(
                            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                            2.0,
                            color,
                        );
                        
                        // Draw frame number for debugging
                        painter.text(
                            egui::pos2(screen_x, screen_y),
                            egui::Align2::CENTER_CENTER,
                            format!("F{}", frame_index),
                            egui::FontId::proportional(12.0),
                            egui::Color32::BLACK,
                        );
                    }
                }
            }
        } else if let Some(sprite) = world.sprites.get(entity) {
            // Render regular sprite
            let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
            // Unity-style: sprite size in world units = pixels / pixels_per_unit
            let world_width = sprite.width / sprite.pixels_per_unit;
            let world_height = sprite.height / sprite.pixels_per_unit;
            let size = egui::vec2(
                world_width * transform_scale.x * zoom,
                world_height * transform_scale.y * zoom
            );
            let color = egui::Color32::from_rgba_unmultiplied(
                (sprite.color[0] * 255.0) as u8,
                (sprite.color[1] * 255.0) as u8,
                (sprite.color[2] * 255.0) as u8,
                (sprite.color[3] * 255.0) as u8,
            );

            // Try to load and render texture
            if !sprite.texture_id.is_empty() {
                let texture_path = std::path::Path::new(&sprite.texture_id);
                if let Some(texture) = texture_manager.load_texture(ctx, &sprite.texture_id, texture_path) {
                    // Render texture with color tint and flipping
                    let mut mesh = egui::Mesh::with_texture(texture.id());

                    let rect = egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size);

                    // Calculate UV coordinates based on sprite_rect (Unity-style)
                    let (u_min_base, u_max_base, v_min_base, v_max_base) = if let Some(sprite_rect) = sprite.sprite_rect {
                        // Use sprite rect to calculate UV coordinates
                        let tex_size = texture.size();
                        let tex_width = tex_size[0] as f32;
                        let tex_height = tex_size[1] as f32;
                        
                        let u_min = sprite_rect[0] as f32 / tex_width;
                        let v_min = sprite_rect[1] as f32 / tex_height;
                        let u_max = (sprite_rect[0] + sprite_rect[2]) as f32 / tex_width;
                        let v_max = (sprite_rect[1] + sprite_rect[3]) as f32 / tex_height;
                        
                        (u_min, u_max, v_min, v_max)
                    } else {
                        // Use full texture
                        (0.0, 1.0, 0.0, 1.0)
                    };

                    // Apply flipping
                    let (u_min, u_max) = if sprite.flip_x { (u_max_base, u_min_base) } else { (u_min_base, u_max_base) };
                    let (v_min, v_max) = if sprite.flip_y { (v_max_base, v_min_base) } else { (v_min_base, v_max_base) };

                    mesh.add_rect_with_uv(
                        rect,
                        egui::Rect::from_min_max(
                            egui::pos2(u_min, v_min),
                            egui::pos2(u_max, v_max),
                        ),
                        color,
                    );

                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    // Fallback to colored rectangle if texture load fails
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                        2.0,
                        color,
                    );
                }
            } else {
                // No texture specified, render colored rectangle
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                    2.0,
                    color,
                );
            }
        }

        // Render mesh if exists (simple placeholder for now)
        if let Some(mesh) = world.meshes.get(entity) {
            let size = 50.0 * zoom;
            let color = egui::Color32::from_rgba_unmultiplied(
                (mesh.color[0] * 255.0) as u8,
                (mesh.color[1] * 255.0) as u8,
                (mesh.color[2] * 255.0) as u8,
                (mesh.color[3] * 255.0) as u8,
            );

            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(size, size)),
                2.0,
                color,
            );
        }
    }
}

/// Render in perspective mode (3D)
fn render_perspective(
    world: &World,
    painter: &egui::Painter,
    ctx: &egui::Context,
    camera: &Camera,
    cam_pos: [f32; 3],
    center: egui::Pos2,
    texture_manager: &mut TextureManager,
) {
    // Calculate FOV scale for perspective projection
    let fov_rad = camera.fov.to_radians();
    let fov_scale = 1.0 / (fov_rad / 2.0).tan();

    // Perspective distance
    let perspective_distance = 500.0;

    // Render all entities
    for (entity, transform) in &world.transforms {
        // Skip if not active
        if !world.active.get(entity).copied().unwrap_or(true) {
            continue;
        }

        // Calculate world position relative to camera
        let world_x = transform.position[0] - cam_pos[0];
        let world_y = transform.position[1] - cam_pos[1];
        let world_z = transform.position[2] - cam_pos[2];

        // Calculate perspective depth (Z distance from camera)
        let depth = world_z + perspective_distance;

        // Skip if behind camera or too close
        if depth <= camera.near_clip || depth > camera.far_clip {
            continue;
        }

        // Apply perspective division
        let perspective_scale = perspective_distance / depth;
        let screen_scale = fov_scale * perspective_scale * 100.0;

        let screen_x = center.x + world_x * screen_scale;
        let screen_y = center.y - world_y * screen_scale; // Flip Y axis

        // Check if entity has animated sprite
        let has_animated_sprite = world.animated_sprites.contains_key(entity);
        
        if has_animated_sprite {
            // Render animated sprite from sprite sheet
            if let (Some(animated_sprite), Some(sprite_sheet)) = 
                (world.animated_sprites.get(entity), world.sprite_sheets.get(entity)) {
                
                let frame_index = animated_sprite.get_frame_index();
                if let Some(frame) = sprite_sheet.get_frame(frame_index) {
                    let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
                    
                    let aspect_ratio = frame.width as f32 / frame.height as f32;
                    let size = egui::vec2(
                        transform_scale.x * screen_scale * aspect_ratio,
                        transform_scale.y * screen_scale
                    );
                    
                    let color = if let Some(sprite) = world.sprites.get(entity) {
                        egui::Color32::from_rgba_unmultiplied(
                            (sprite.color[0] * 255.0) as u8,
                            (sprite.color[1] * 255.0) as u8,
                            (sprite.color[2] * 255.0) as u8,
                            (sprite.color[3] * 255.0) as u8,
                        )
                    } else {
                        egui::Color32::WHITE
                    };

                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                        2.0,
                        color,
                    );
                    
                    painter.text(
                        egui::pos2(screen_x, screen_y),
                        egui::Align2::CENTER_CENTER,
                        format!("F{}", frame_index),
                        egui::FontId::proportional(12.0),
                        egui::Color32::BLACK,
                    );
                }
            }
        } else if let Some(sprite) = world.sprites.get(entity) {
            // Render regular sprite
            let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
            let size = egui::vec2(transform_scale.x * screen_scale, transform_scale.y * screen_scale);
            let color = egui::Color32::from_rgba_unmultiplied(
                (sprite.color[0] * 255.0) as u8,
                (sprite.color[1] * 255.0) as u8,
                (sprite.color[2] * 255.0) as u8,
                (sprite.color[3] * 255.0) as u8,
            );

            // Try to load and render texture
            if !sprite.texture_id.is_empty() {
                let texture_path = std::path::Path::new(&sprite.texture_id);
                if let Some(texture) = texture_manager.load_texture(ctx, &sprite.texture_id, texture_path) {
                    // Render texture with color tint and flipping
                    let mut mesh = egui::Mesh::with_texture(texture.id());

                    let rect = egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size);

                    // Calculate UV coordinates based on sprite_rect (Unity-style)
                    let (u_min_base, u_max_base, v_min_base, v_max_base) = if let Some(sprite_rect) = sprite.sprite_rect {
                        // Use sprite rect to calculate UV coordinates
                        let tex_size = texture.size();
                        let tex_width = tex_size[0] as f32;
                        let tex_height = tex_size[1] as f32;
                        
                        let u_min = sprite_rect[0] as f32 / tex_width;
                        let v_min = sprite_rect[1] as f32 / tex_height;
                        let u_max = (sprite_rect[0] + sprite_rect[2]) as f32 / tex_width;
                        let v_max = (sprite_rect[1] + sprite_rect[3]) as f32 / tex_height;
                        
                        (u_min, u_max, v_min, v_max)
                    } else {
                        // Use full texture
                        (0.0, 1.0, 0.0, 1.0)
                    };

                    // Apply flipping
                    let (u_min, u_max) = if sprite.flip_x { (u_max_base, u_min_base) } else { (u_min_base, u_max_base) };
                    let (v_min, v_max) = if sprite.flip_y { (v_max_base, v_min_base) } else { (v_min_base, v_max_base) };

                    mesh.add_rect_with_uv(
                        rect,
                        egui::Rect::from_min_max(
                            egui::pos2(u_min, v_min),
                            egui::pos2(u_max, v_max),
                        ),
                        color,
                    );

                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    // Fallback to colored rectangle if texture load fails
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                        2.0,
                        color,
                    );
                }
            } else {
                // No texture specified, render colored rectangle
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                    2.0,
                    color,
                );
            }
        }

        // Render mesh if exists
        if let Some(mesh) = world.meshes.get(entity) {
            let size = 50.0 * screen_scale;
            let color = egui::Color32::from_rgba_unmultiplied(
                (mesh.color[0] * 255.0) as u8,
                (mesh.color[1] * 255.0) as u8,
                (mesh.color[2] * 255.0) as u8,
                (mesh.color[3] * 255.0) as u8,
            );

            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(size, size)),
                2.0,
                color,
            );
        }
    }
}
