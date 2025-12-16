//! Game Runtime Renderer
//!
//! Renders the game scene using Camera components from the ECS.
//! This is separate from the editor's scene view.

use ecs::{World, Entity, Camera, CameraProjection};
use egui;
use crate::texture_manager::TextureManager;
use glam::{Vec3, Mat4, Quat, EulerRot};

/// Render the game view using the main camera
pub fn render_game_view(
    ui: &mut egui::Ui,
    world: &World,
    texture_manager: &mut TextureManager,
    ui_manager: Option<&mut crate::ui_manager::UIManager>,
    game_view_settings: Option<&crate::runtime::GameViewSettings>,
) {
    let available_rect = ui.available_rect_before_wrap();
    
    // Calculate game view rect based on settings
    let game_rect = if let Some(settings) = game_view_settings {
        settings.calculate_game_rect(available_rect)
    } else {
        available_rect
    };
    
    // Fill background outside game view
    if let Some(settings) = game_view_settings {
        if !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
            let bg_color = egui::Color32::from_rgba_unmultiplied(
                (settings.background_color[0] * 255.0) as u8,
                (settings.background_color[1] * 255.0) as u8,
                (settings.background_color[2] * 255.0) as u8,
                (settings.background_color[3] * 255.0) as u8,
            );
            ui.painter().rect_filled(available_rect, 0.0, bg_color);
        }
    }
    
    let rect = game_rect;
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
        
        // Render UI system on top
        if let Some(ui_mgr) = ui_manager {
            ui_mgr.render(ui, world, rect);
        }
        
        // Render game view overlays (resolution info, safe area)
        if let Some(settings) = game_view_settings {
            render_game_view_overlays(ui, rect, settings);
        }
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

/// Render game view overlays (resolution info, safe area guides)
fn render_game_view_overlays(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    settings: &crate::runtime::GameViewSettings,
) {
    let painter = ui.painter();
    
    // Show resolution info
    if settings.show_resolution_info && !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
        let (w, h) = settings.resolution.get_size();
        let info_text = format!(
            "{}\n{}x{} ({}%)",
            settings.resolution.get_name(),
            w, h,
            (settings.scale * 100.0) as i32
        );
        
        // Background for text
        let text_pos = rect.left_top() + egui::vec2(8.0, 8.0);
        let text_galley = painter.layout_no_wrap(
            info_text.clone(),
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
        let text_rect = egui::Rect::from_min_size(
            text_pos,
            text_galley.size() + egui::vec2(8.0, 4.0),
        );
        painter.rect_filled(
            text_rect,
            2.0,
            egui::Color32::from_black_alpha(180),
        );
        painter.text(
            text_pos + egui::vec2(4.0, 2.0),
            egui::Align2::LEFT_TOP,
            info_text,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }
    
    // Show safe area guides
    if settings.show_safe_area {
        let safe_margin = 0.05; // 5% margin
        let safe_rect = rect.shrink2(egui::vec2(
            rect.width() * safe_margin,
            rect.height() * safe_margin,
        ));
        
        // Draw safe area border
        painter.rect_stroke(
            safe_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 0)),
        );
        
        // Draw corner markers
        let marker_size = 10.0;
        let corners = [
            safe_rect.left_top(),
            safe_rect.right_top(),
            safe_rect.left_bottom(),
            safe_rect.right_bottom(),
        ];
        
        for corner in corners {
            painter.line_segment(
                [corner, corner + egui::vec2(marker_size, 0.0)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.line_segment(
                [corner, corner + egui::vec2(0.0, marker_size)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
            );
        }
    }
    
    // Draw border around game view
    if !matches!(settings.resolution, crate::runtime::GameViewResolution::Free) {
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
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
            render_orthographic(world, &painter, &ctx, camera, camera_transform, center, texture_manager);
        }
        CameraProjection::Perspective => {
            render_perspective(world, &painter, &ctx, camera, camera_transform, center, texture_manager);
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
                // Make tiles match grid cell size (1 world unit per tile)
                // This ensures tilemap aligns perfectly with grid for consistent sizing
                let tile_world_size = 1.0;  // 1 tile = 1 grid cell = 1 world unit
                let tile_world_x = tilemap_x + (x as f32 * tile_world_size);
                // Flip Y: LDtk uses top-left origin (Y down), engine uses bottom-left (Y up)
                let tile_world_y = tilemap_y - (y as f32 * tile_world_size);
                
                // Convert to screen space
                let world_x = tile_world_x - cam_pos[0];
                let world_y = tile_world_y - cam_pos[1];

                let screen_x = center.x + world_x * zoom;
                let screen_y = center.y - world_y * zoom;

                // Calculate size in world units (match grid cell size)
                let tile_world_width = tile_world_size;
                let tile_world_height = tile_world_size;
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
    camera_transform: &ecs::Transform,
    center: egui::Pos2,
    texture_manager: &mut TextureManager,
) {
    let cam_pos = camera_transform.position;
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
            let width = transform.scale[0] * zoom;
            let height = transform.scale[1] * zoom;
            let color = egui::Color32::from_rgba_unmultiplied(
                (mesh.color[0] * 255.0) as u8,
                (mesh.color[1] * 255.0) as u8,
                (mesh.color[2] * 255.0) as u8,
                (mesh.color[3] * 255.0) as u8,
            );

            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(width, height)),
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
    camera_transform: &ecs::Transform,
    center: egui::Pos2,
    texture_manager: &mut TextureManager,
) {
    // 1. Construct View Matrix (Camera World -> View Space)
    // Camera transform is Local -> World. View Matrix is Inverse(Camera World).
    
    // Convert Euler angles (degrees) to Quat
    // Assuming rotation order YXZ (Yaw, Pitch, Roll) which is standard for game cams
    let rot_rad = Vec3::new(
        camera_transform.rotation[0].to_radians(),
        camera_transform.rotation[1].to_radians(),
        camera_transform.rotation[2].to_radians(),
    );
    
    let cam_rotation = Quat::from_euler(
        EulerRot::YXZ, 
        rot_rad.y, // Yaw 
        rot_rad.x, // Pitch 
        rot_rad.z  // Roll
    );
    let cam_translation = Vec3::from(camera_transform.position);
    
    // View Matrix = Inverse of Camera World Matrix
    // World Matrix = T * R
    // View Matrix = (T * R)^-1 = R^-1 * T^-1
    let view_matrix = Mat4::from_rotation_translation(cam_rotation, cam_translation).inverse();

    // 2. Construct Projection Matrix (View Space -> Clip Space)
    let rect = painter.clip_rect();
    let aspect_ratio = rect.width() / rect.height();
    
    // Use Right-Handed perspective (standard for glam/wgpu)
    // Z range [0, 1]
    let proj_matrix = Mat4::perspective_rh(
        camera.fov.to_radians(), 
        aspect_ratio, 
        camera.near_clip, 
        camera.far_clip
    );

    // Combined View-Projection Matrix
    let view_proj = proj_matrix * view_matrix;

    // Render all entities
    for (entity, transform) in &world.transforms {
        // Skip if not active
        if !world.active.get(entity).copied().unwrap_or(true) {
            continue;
        }

        // Get entity position
        let world_pos = Vec3::from(transform.position);

        // Project world position to NDC (Normalized Device Coordinates)
        let ndc_pos = view_proj.project_point3(world_pos);

        // Check visibility (Frustum Culling)
        // NDC range: x: [-1, 1], y: [-1, 1], z: [0, 1] (for perspective_rh)
        if ndc_pos.z < 0.0 || ndc_pos.z > 1.0 || 
           ndc_pos.x < -1.2 || ndc_pos.x > 1.2 || 
           ndc_pos.y < -1.2 || ndc_pos.y > 1.2 {
            continue;
        }

        // Convert NDC to Screen Coordinates
        // NDC Y is up, Screen Y is down (egui) -> flip Y
        let screen_x = center.x + ndc_pos.x * (rect.width() * 0.5);
        let screen_y = center.y - ndc_pos.y * (rect.height() * 0.5);

        // Calculate Scale/Size on Screen
        // Project a second point offset by scale to estimate screen size
        // Offset by UP vector * scale.y
        let top_world_pos = world_pos + (cam_rotation * Vec3::Y) * transform.scale[1]; // Use camera up or world up? World up generally.
        // Let's use camera-facing plane size approximation for simplicity
        // Or simply: world_size / depth * constant
        
        // Better: Project a point at the corner of the object bounds
        let world_scale = Vec3::from(transform.scale);
        let corner_world_pos = world_pos + (view_matrix.inverse().transform_vector3(Vec3::X)) * world_scale.x; 
        // Use camera right vector for width estimation to avoid rotation issues
        let corner_ndc = view_proj.project_point3(corner_world_pos);
        let screen_width = (corner_ndc.x - ndc_pos.x).abs() * rect.width() * 0.5 * 2.0;
        let screen_scale = screen_width / world_scale.x; // approximate pixels per unit at this depth

        // Render contents...
        
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
            // Calculate size based on sprite dimensions and transform scale
            let size = egui::vec2(
                sprite.width * transform_scale.x * screen_scale, // Use calculated screen_scale
                sprite.height * transform_scale.y * screen_scale
            );
            
            // Note: This logic assumes billboard behavior or 2D sprites in 3D world
            // Ideally, we should check sprite.billboard
             
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
                    let mut mesh = egui::Mesh::with_texture(texture.id());
                    let rect = egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size);

                    // UVs and flipping...
                    let (u_min_base, u_max_base, v_min_base, v_max_base) = if let Some(sprite_rect) = sprite.sprite_rect {
                        let tex_size = texture.size();
                        let tex_width = tex_size[0] as f32;
                        let tex_height = tex_size[1] as f32;
                        (
                            sprite_rect[0] as f32 / tex_width,
                            (sprite_rect[0] + sprite_rect[2]) as f32 / tex_width,
                            sprite_rect[1] as f32 / tex_height,
                            (sprite_rect[1] + sprite_rect[3]) as f32 / tex_height
                        )
                    } else {
                        (0.0, 1.0, 0.0, 1.0)
                    };

                    let (u_min, u_max) = if sprite.flip_x { (u_max_base, u_min_base) } else { (u_min_base, u_max_base) };
                    let (v_min, v_max) = if sprite.flip_y { (v_max_base, v_min_base) } else { (v_min_base, v_max_base) };

                    mesh.add_rect_with_uv(
                        rect,
                        egui::Rect::from_min_max(egui::pos2(u_min, v_min), egui::pos2(u_max, v_max)),
                        color,
                    );
                    painter.add(egui::Shape::mesh(mesh));
                } else {
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                        2.0,
                        color,
                    );
                }
            } else {
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                    2.0,
                    color,
                );
            }
        }

        // Render mesh if exists
        if let Some(mesh) = world.meshes.get(entity) {
            let width = transform.scale[0] * screen_scale; // Use calculated screen_scale
            let height = transform.scale[1] * screen_scale;
            let color = egui::Color32::from_rgba_unmultiplied(
                (mesh.color[0] * 255.0) as u8,
                (mesh.color[1] * 255.0) as u8,
                (mesh.color[2] * 255.0) as u8,
                (mesh.color[3] * 255.0) as u8,
            );

            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(width, height)),
                2.0,
                color,
            );
        }
    }
}
