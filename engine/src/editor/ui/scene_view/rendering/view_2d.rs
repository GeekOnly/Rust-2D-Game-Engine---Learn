//! 2D Scene View Rendering
//!
//! Handles rendering of the scene in 2D mode (sprites, grid, gizmos).

use ecs::{World, Entity, MeshType};
use egui;
use crate::editor::SceneCamera;
use crate::texture_manager::TextureManager;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_camera_viewport_bounds, render_collider_gizmo, render_velocity_gizmo};

/// Render a tilemap in the scene view
fn render_tilemap_in_scene(
    painter: &egui::Painter,
    entity: Entity,
    tilemap: &ecs::Tilemap,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    texture_manager: &mut TextureManager,
    ctx: &egui::Context,
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

                // Calculate world position
                // Make tiles match grid cell size (1 world unit per tile)
                // This ensures tilemap aligns perfectly with grid for consistent sizing
                let tile_world_size = 1.0;  // 1 tile = 1 grid cell = 1 world unit
                let tile_world_x = tilemap_x + (x as f32 * tile_world_size);
                let tile_world_y = tilemap_y - (y as f32 * tile_world_size); // Flip Y
                
                let world_pos = glam::Vec2::new(tile_world_x, tile_world_y);
                let screen_pos = scene_camera.world_to_screen(world_pos);
                let screen_x = center.x + screen_pos.x;
                let screen_y = center.y + screen_pos.y;

                // Calculate size in world units (match grid cell size)
                let tile_world_width = tile_world_size;
                let tile_world_height = tile_world_size;
                let size = egui::vec2(tile_world_width * scene_camera.zoom, tile_world_height * scene_camera.zoom);
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

/// Render the scene in 2D mode
pub fn render_scene_2d(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    show_debug_lines: &bool,
    debug_draw: &mut crate::editor::debug_draw::DebugDrawManager,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
    texture_manager: &mut TextureManager,
    ctx: &egui::Context,
    rect: egui::Rect,
) {
    // In 2D, we just iterate through all entities. 
    // Z-ordering is usually determined by entity order or a specific Z-index component (if we had one).
    // For now, we'll just use the order in the HashMap (arbitrary) or sorted by ID.
    // To be consistent, let's sort by ID for stability.
    let mut entities: Vec<Entity> = world.transforms.keys().cloned().collect();
    entities.sort();

    // First, render tilemaps (background)
    for entity in &entities {
        if let Some(tilemap) = world.tilemaps.get(entity) {
            if let Some(transform) = world.transforms.get(entity) {
                render_tilemap_in_scene(
                    painter,
                    *entity,
                    tilemap,
                    transform,
                    world,
                    scene_camera,
                    center,
                    texture_manager,
                    ctx,
                );
            }
        }
    }

    // Then render camera viewport bounds
    for entity in &entities {
        if world.cameras.contains_key(entity) {
            render_camera_viewport_bounds(
                painter,
                *entity,
                world,
                scene_camera,
                center,
            );
        }
    }
    
    // Then render all entities
    for entity in entities {
        if let Some(transform) = world.transforms.get(&entity) {
            render_entity_2d(
                painter,
                entity,
                transform,
                world,
                scene_camera,
                center,
                selected_entity,
                show_colliders,
                show_velocities,
                hovered_entity,
                response,
                texture_manager,
                ctx,
            );
        }
    }

    // Render selection outline on top
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;

            // Draw selection outline
            if let Some(sprite) = world.sprites.get(&sel_entity) {
                let scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
                // Use sprite dimensions with pixels_per_unit (same as rendering)
                let world_width = sprite.width / sprite.pixels_per_unit;
                let world_height = sprite.height / sprite.pixels_per_unit;
                let size = egui::vec2(
                    world_width * scale.x * scene_camera.zoom,
                    world_height * scale.y * scene_camera.zoom
                );
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size + egui::vec2(4.0, 4.0)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            } else if world.meshes.contains_key(&sel_entity) {
                let scale = glam::Vec3::from(transform.scale);
                let world_size = 2.0;
                let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
                let selection_size = base_size + 8.0;
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(selection_size, selection_size)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            }

            // Draw selected entity's collider gizmo on top
            if *show_colliders {
                render_collider_gizmo(painter, sel_entity, world, screen_x, screen_y, scene_camera, true);
            }
        }
    }

    // Render debug lines if enabled
    if *show_debug_lines {
        let camera_pos = [scene_camera.position.x, scene_camera.position.y, 0.0];
        debug_draw.render(painter, camera_pos, scene_camera.zoom, rect);
    }
}

/// Render transform gizmo for selected entity in 2D
pub fn render_transform_gizmo_2d(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    current_tool: &super::super::super::TransformTool,
    transform_space: &super::super::types::TransformSpace,
) {
    if let Some(transform) = world.transforms.get(&entity) {
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        let screen_x = center.x + screen_pos.x;
        let screen_y = center.y + screen_pos.y;
        
        super::gizmos::render_transform_gizmo(
            painter,
            screen_x,
            screen_y,
            current_tool,
            scene_camera,
            &super::super::types::SceneViewMode::Mode2D,
            transform_space,
            transform,
            None, // No viewport_rect needed for 2D mode
        );
    }
}

fn render_entity_2d(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
    texture_manager: &mut TextureManager,
    ctx: &egui::Context,
) {
    // 2D Projection
    let world_pos = glam::Vec2::new(transform.x(), transform.y());
    let screen_pos = scene_camera.world_to_screen(world_pos);
    let screen_x = center.x + screen_pos.x;
    let screen_y = center.y + screen_pos.y;

    // Get entity bounds for click detection
    // Use sprite dimensions with pixels_per_unit (matching rendering)
    let entity_rect = if let Some(sprite) = world.sprites.get(&entity) {
        let scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        // Use sprite dimensions with pixels_per_unit
        let world_width = sprite.width / sprite.pixels_per_unit;
        let world_height = sprite.height / sprite.pixels_per_unit;
        let size = egui::vec2(
            world_width * scale.x * scene_camera.zoom,
            world_height * scale.y * scene_camera.zoom
        );
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
    } else if world.meshes.contains_key(&entity) {
        let scale = glam::Vec3::from(transform.scale);
        let world_size = 2.0;
        let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(base_size, base_size))
    } else {
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(10.0, 10.0))
    };

    // Check hover
    if let Some(hover_pos) = response.hover_pos() {
        if entity_rect.contains(hover_pos) {
            *hovered_entity = Some(entity);
        }
    }

    // Check if entity has animated sprite (priority over regular sprite)
    let has_animated_sprite = world.animated_sprites.contains_key(&entity);
    
    if has_animated_sprite {
        // Render animated sprite from sprite sheet
        if let (Some(animated_sprite), Some(sprite_sheet)) = 
            (world.animated_sprites.get(&entity), world.sprite_sheets.get(&entity)) {
            
            let frame_index = animated_sprite.get_frame_index();
            if let Some(frame) = sprite_sheet.get_frame(frame_index) {
                let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
                
                // Calculate size based on frame dimensions
                let aspect_ratio = frame.width as f32 / frame.height as f32;
                let size = egui::vec2(
                    transform_scale.x * scene_camera.zoom * aspect_ratio,
                    transform_scale.y * scene_camera.zoom
                );
                
                // Get sprite color if exists, otherwise white
                let color = if let Some(sprite) = world.sprites.get(&entity) {
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
                }
            }
        }
    } else if let Some(sprite) = world.sprites.get(&entity) {
        // Render regular sprite
        let scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        // Unity-style: sprite size in world units = pixels / pixels_per_unit
        let world_width = sprite.width / sprite.pixels_per_unit;
        let world_height = sprite.height / sprite.pixels_per_unit;
        let size = egui::vec2(
            world_width * scale.x * scene_camera.zoom,
            world_height * scale.y * scene_camera.zoom
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
                let rotation_rad = transform.rotation[2].to_radians();

                if rotation_rad.abs() < 0.01 {
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                        2.0,
                        color,
                    );
                } else {
                    // Rotated sprite
                    let half_width = size.x / 2.0;
                    let half_height = size.y / 2.0;
                    let cos_r = rotation_rad.cos();
                    let sin_r = rotation_rad.sin();

                    let corners = [
                        egui::pos2(
                            screen_x + (-half_width * cos_r - (-half_height) * sin_r),
                            screen_y + (-half_width * sin_r + (-half_height) * cos_r),
                        ),
                        egui::pos2(
                            screen_x + (half_width * cos_r - (-half_height) * sin_r),
                            screen_y + (half_width * sin_r + (-half_height) * cos_r),
                        ),
                        egui::pos2(
                            screen_x + (half_width * cos_r - half_height * sin_r),
                            screen_y + (half_width * sin_r + half_height * cos_r),
                        ),
                        egui::pos2(
                            screen_x + (-half_width * cos_r - half_height * sin_r),
                            screen_y + (-half_width * sin_r + half_height * cos_r),
                        ),
                    ];

                    painter.add(egui::Shape::convex_polygon(
                        corners.to_vec(),
                        color,
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                    ));
                }
            }
        } else {
            // No texture specified, render colored rectangle
            let rotation_rad = transform.rotation[2].to_radians();

            if rotation_rad.abs() < 0.01 {
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                    2.0,
                    color,
                );
            } else {
                // Rotated sprite
                let half_width = size.x / 2.0;
                let half_height = size.y / 2.0;
                let cos_r = rotation_rad.cos();
                let sin_r = rotation_rad.sin();

                let corners = [
                    egui::pos2(
                        screen_x + (-half_width * cos_r - (-half_height) * sin_r),
                        screen_y + (-half_width * sin_r + (-half_height) * cos_r),
                    ),
                    egui::pos2(
                        screen_x + (half_width * cos_r - (-half_height) * sin_r),
                        screen_y + (half_width * sin_r + (-half_height) * cos_r),
                    ),
                    egui::pos2(
                        screen_x + (half_width * cos_r - half_height * sin_r),
                        screen_y + (half_width * sin_r + half_height * cos_r),
                    ),
                    egui::pos2(
                        screen_x + (-half_width * cos_r - half_height * sin_r),
                        screen_y + (-half_width * sin_r + half_height * cos_r),
                    ),
                ];

                painter.add(egui::Shape::convex_polygon(
                    corners.to_vec(),
                    color,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                ));
            }
        }
    } else if let Some(mesh) = world.meshes.get(&entity) {
        // Render Mesh (Simplified for 2D)
        let color = egui::Color32::from_rgba_unmultiplied(
            (mesh.color[0] * 255.0) as u8,
            (mesh.color[1] * 255.0) as u8,
            (mesh.color[2] * 255.0) as u8,
            (mesh.color[3] * 255.0) as u8,
        );
        
        let scale = glam::Vec3::from(transform.scale);
        let world_size = 2.0;
        let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);

        match mesh.mesh_type {
            MeshType::Sphere | MeshType::Cylinder => {
                 painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
                 painter.circle_stroke(egui::pos2(screen_x, screen_y), base_size / 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            },
            _ => {
                let rect = egui::Rect::from_center_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(base_size, base_size),
                );
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
    } else {
        // Check if this is a camera entity
        let is_camera = world.cameras.contains_key(&entity);
        
        if is_camera {
            // Render camera gizmo
            render_camera_gizmo(
                painter,
                screen_x,
                screen_y,
                entity,
                world,
                scene_camera,
                &SceneViewMode::Mode2D,
            );
        } else {
            // Default placeholder for other entities
            painter.circle_filled(egui::pos2(screen_x, screen_y), 5.0, egui::Color32::from_rgb(150, 150, 150));
        }
    }

    // Gizmos
    if *show_colliders && *selected_entity != Some(entity) {
        render_collider_gizmo(painter, entity, world, screen_x, screen_y, scene_camera, false);
    }
    
    if *show_velocities {
        render_velocity_gizmo(painter, entity, world, screen_x, screen_y);
    }
}
