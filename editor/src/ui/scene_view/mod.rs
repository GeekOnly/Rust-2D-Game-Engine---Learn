//! Scene View Module
//!
//! Refactored scene view system with better organization.
//! 
//! ## Structure:
//! - `types`: Type definitions and enums
//! - `rendering`: All rendering functions (grid, entities, gizmos)
//! - `interaction`: User interaction (camera, selection, transforms)
//! - `toolbar`: Toolbar UI
//! - `shortcuts`: Keyboard shortcuts

// Module declarations
pub mod types;
pub mod rendering;
pub mod interaction;
pub mod toolbar;
pub mod shortcuts;

// Re-exports for backward compatibility
pub use types::*;

use ecs::{World, Entity};
use egui;
use crate::ui::TransformTool;
use crate::{SceneCamera, SceneGrid, DragDropState};

/// Main scene view render function
/// 
/// This is the entry point for rendering the scene view panel.
/// It coordinates all submodules to render the complete scene.
pub fn render_scene_view(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    _scene_view_tab: &mut usize,
    is_playing: bool,
    show_colliders: &bool,
    show_velocities: &bool,
    show_debug_lines: &bool,
    debug_draw: &mut crate::debug_draw::DebugDrawManager,
    current_tool: &mut TransformTool,
    scene_camera: &mut SceneCamera,
    scene_grid: &SceneGrid,
    infinite_grid: &mut crate::grid::InfiniteGrid,
    camera_state_display: &crate::ui::camera_settings::CameraStateDisplay,
    play_request: &mut bool,
    stop_request: &mut bool,
    dragging_entity: &mut Option<Entity>,
    drag_axis: &mut Option<u8>,
    scene_view_mode: &mut SceneViewMode,
    projection_mode: &mut SceneProjectionMode,
    transform_space: &mut TransformSpace,
    texture_manager: &mut engine::texture_manager::TextureManager,
    drag_drop: &mut DragDropState,
    delta_time: f32,
    map_manager: &crate::map_manager::MapManager,
    scene_view_renderer: &mut crate::scene_view_renderer::SceneViewRenderer,
    egui_renderer: &mut egui_wgpu::Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    // Sync camera projection mode with editor state
    scene_camera.projection_mode = *projection_mode;
    
    // Track previous mode to detect changes
    let previous_mode = *scene_view_mode;
    
    // Update camera (for smooth interpolation and damping)
    scene_camera.update(delta_time);
    
    // Render toolbar
    toolbar::render_scene_toolbar(
        ui,
        current_tool,
        is_playing,
        play_request,
        stop_request,
        scene_view_mode,
        transform_space,
    );

    // Handle mode switching
    if previous_mode != *scene_view_mode {
        match scene_view_mode {
            SceneViewMode::Mode2D => scene_camera.switch_to_2d(),
            SceneViewMode::Mode3D => scene_camera.switch_to_3d(),
        }
    }

    // Main scene view
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::click_and_drag(),
    );
    let rect = response.rect;

    // Handle keyboard shortcuts
    shortcuts::handle_keyboard_shortcuts(ui, current_tool, scene_camera, scene_view_mode);
    
    // Check for F key press (focus on selected entity)
    // Use ctx.input instead of ui.input to ensure we catch the key press even if UI doesn't have focus
    let focus_pressed = ui.ctx().input(|i| i.key_pressed(egui::Key::F) && !i.modifiers.ctrl && !i.modifiers.shift && !i.modifiers.alt);
    
    // Handle camera controls
    interaction::camera::handle_camera_controls(
        &response,
        scene_camera,
        rect,
        scene_view_mode,
        selected_entity,
        world,
    );

    // Background - Unity-like colors
    let bg_color = match scene_view_mode {
        SceneViewMode::Mode2D => egui::Color32::from_rgb(40, 40, 50),
        SceneViewMode::Mode3D => egui::Color32::from_rgb(48, 48, 48),  // Unity-like dark gray
    };
    painter.rect_filled(rect, 0.0, bg_color);

    // Render grid
    if scene_grid.enabled {
        match scene_view_mode {
            SceneViewMode::Mode2D => rendering::grid::render_grid_2d(&painter, rect, scene_camera, scene_grid),
            SceneViewMode::Mode3D => {
                // Grid is rendered inside render_scene_3d to handle overlay/depth correctly
            }
        }
    }
    


    // Render entities
    let center = rect.center();
    let mut hovered_entity: Option<Entity> = None;
    
    // Render entities based on mode
    let ctx = ui.ctx().clone();
    match scene_view_mode {
        SceneViewMode::Mode2D => {
            rendering::view_2d::render_scene_2d(
                &painter,
                world,
                scene_camera,
                center,
                selected_entity,
                show_colliders,
                show_velocities,
                show_debug_lines,
                debug_draw,
                &mut hovered_entity,
                &response,
                texture_manager,
                &ctx,
                rect,
            );
            
            // Render transform gizmo for selected entity
            if let Some(entity) = *selected_entity {
                rendering::view_2d::render_transform_gizmo_2d(
                    &painter,
                    entity,
                    world,
                    scene_camera,
                    center,
                    current_tool,
                    transform_space,
                );
            }
        }
        SceneViewMode::Mode3D => {
            rendering::view_3d::render_scene_3d(
                &painter,
                world,
                scene_camera,
                scene_grid,
                infinite_grid,
                projection_mode,
                center,
                selected_entity,
                show_colliders,
                show_velocities,
                show_debug_lines,
                &mut hovered_entity,
                &response,
                texture_manager,
                &ctx,
                Some(&map_manager.settings),
                scene_view_renderer,
                egui_renderer,
                device,
            );
        }
    }

    // Render 3D scene gizmo (top-right corner) - Rendered AFTER scene to be on top
    if *scene_view_mode == SceneViewMode::Mode3D {
        let gizmo_size = 80.0;
        let margin = 20.0;
        let gizmo_center_x = rect.max.x - margin - gizmo_size / 2.0;
        let gizmo_center_y = rect.min.y + margin + gizmo_size / 2.0;

        interaction::camera::handle_gizmo_axis_clicks(ui, gizmo_center_x, gizmo_center_y, gizmo_size, scene_camera);
        rendering::gizmos::render_scene_gizmo_visual(&painter, gizmo_center_x, gizmo_center_y, gizmo_size, scene_camera);

        // Projection mode button
        let button_y = gizmo_center_y + gizmo_size / 2.0 + 35.0;
        let button_pos = egui::pos2(gizmo_center_x - 40.0, button_y - 10.0);

        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(button_pos, egui::vec2(80.0, 20.0)),
            |ui| {
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgba_premultiplied(50, 50, 55, 200);
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgba_premultiplied(60, 60, 65, 220);
                ui.style_mut().visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgba_premultiplied(70, 70, 75, 240);

                let button_text = match projection_mode {
                    SceneProjectionMode::Perspective => "⬜ Persp",
                    SceneProjectionMode::Isometric => "◇ Iso",
                };

                if ui.button(button_text).clicked() {
                    *projection_mode = match projection_mode {
                        SceneProjectionMode::Perspective => SceneProjectionMode::Isometric,
                        SceneProjectionMode::Isometric => SceneProjectionMode::Perspective,
                    };
                }
            }
        );
    }

    // Focus on selected entity (F key)
    if focus_pressed {
        if let Some(entity) = *selected_entity {
            if let Some(transform) = world.transforms.get(&entity) {
                // Calculate center position of the entity
                let mut pos = glam::Vec3::new(transform.x(), transform.y(), 0.0);
                let size = if let Some(sprite) = world.sprites.get(&entity) {
                    sprite.width.max(sprite.height)
                } else if let Some(tilemap) = world.tilemaps.get(&entity) {
                    // For tilemaps, calculate the center position
                    // Unity standard: 100 pixels = 1 world unit
                    let pixels_per_unit = 100.0;
                    let tile_size_pixels = 16.0; // Assume 16px tiles
                    let tile_size_world = tile_size_pixels / pixels_per_unit; // 0.16 world units per tile
                    
                    let width_world = tilemap.width as f32 * tile_size_world;
                    let height_world = tilemap.height as f32 * tile_size_world;
                    
                    // Adjust position to center of tilemap
                    pos.x += width_world / 2.0;
                    pos.y += height_world / 2.0;
                    
                    // Use small representative size for close focus
                    let tile_count = (tilemap.width * tilemap.height) as f32;
                    if tile_count < 100.0 {
                        0.5
                    } else if tile_count < 1000.0 {
                        1.0
                    } else {
                        2.0
                    }
                } else if world.meshes.contains_key(&entity) {
                    2.0 // Reasonable size for meshes in world units
                } else if world.cameras.contains_key(&entity) {
                    2.0 // Reasonable size for camera gizmos
                } else {
                    1.0 // Default size in world units
                };
                let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                
                // Debug logging for focus
                log::info!("Focus request: pos=({:.2}, {:.2}), size={:.2}", pos.x, pos.y, size);
                
                scene_camera.focus_on(pos, size, viewport_size);
            }
        }
    }
    
    // Handle entity selection
    let is_camera_control = response.dragged_by(egui::PointerButton::Middle) || 
                           response.dragged_by(egui::PointerButton::Secondary) ||
                           (ui.input(|i| i.modifiers.alt) && response.dragged_by(egui::PointerButton::Primary));
    
    if response.clicked() && !response.dragged() && !is_camera_control {
        if let Some(entity) = hovered_entity {
            *selected_entity = Some(entity);
        } else {
            *selected_entity = None;
        }
    }

    // Handle transform gizmo interaction
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            // Calculate screen position based on view mode
            let (screen_x, screen_y) = match scene_view_mode {
                SceneViewMode::Mode2D => {
                    // 2D mode: use simple world_to_screen
                    let world_pos = glam::Vec3::new(transform.x(), transform.y(), 0.0);
                    let screen_pos = scene_camera.world_to_screen(world_pos);
                    (center.x + screen_pos.x, center.y + screen_pos.y)
                }
                SceneViewMode::Mode3D => {
                    // 3D mode: use 3D projection
                    // Use same viewport calculation as view_3d.rs for consistency
                    let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                    let world_pos = glam::Vec3::from(transform.position);
                    
                    match rendering::projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
                        Some(pos) => {
                            // pos is relative to viewport (0,0 is top-left of viewport)
                            // Convert to screen coordinates by adding rect.min
                            (rect.min.x + pos.x, rect.min.y + pos.y)
                        },
                        None => (-10000.0, -10000.0), // Off-screen
                    }
                }
            };

            let transform_copy = transform.clone();
            
            let hovered_axis = if let Some(hover_pos) = response.hover_pos() {
                interaction::transform::hit_test_gizmo(
                    screen_x,
                    screen_y,
                    hover_pos,
                    current_tool,
                    scene_camera,
                    scene_view_mode,
                    transform_space,
                    &transform_copy,
                    Some(rect),
                )
            } else {
                None
            };
            
            let highlight_axis = drag_axis.or(hovered_axis);

            rendering::gizmos::render_transform_gizmo(
                &painter,
                screen_x,
                screen_y,
                current_tool,
                scene_camera,
                scene_view_mode,
                transform_space,
                &transform_copy,
                Some(rect),
                highlight_axis,
            );
            
            if !is_camera_control {
                interaction::transform::handle_gizmo_interaction_stateful(
                    &response,
                    sel_entity,
                    world,
                    screen_x,
                    screen_y,
                    current_tool,
                    scene_camera,
                    dragging_entity,
                    drag_axis,
                    transform_space,
                    &transform_copy,
                    scene_view_mode,
                    Some(rect),
                );
            }
        }
    }

    // Clear drag state when not dragging
    if !response.dragged() {
        *dragging_entity = None;
        *drag_axis = None;
    }
    
    // Handle drag-drop from asset browser
    if drag_drop.is_dragging() {
        // Update drop position
        if let Some(hover_pos) = response.hover_pos() {
            drag_drop.set_drop_position(hover_pos);
        }
        
        // Handle drop
        if response.drag_stopped() {
            if let Some(asset) = drag_drop.get_dragged_asset() {
                // Check if it's a sprite file
                if asset.path.extension().and_then(|s| s.to_str()) == Some("sprite") {
                    // Load sprite metadata
                    if let Ok(metadata) = sprite_editor::SpriteMetadata::load(&asset.path) {
                        // Get drop position in world coordinates
                        if let Some(screen_pos) = drag_drop.drop_position {
                            let center = rect.center();
                            let relative_x = screen_pos.x - center.x;
                            let relative_y = screen_pos.y - center.y;
                            let world_pos = scene_camera.screen_to_world(glam::Vec2::new(relative_x, relative_y));
                            
                            // Create entity with sprite
                            let entity = world.spawn();
                            
                            // Add Transform component
                            world.transforms.insert(entity, ecs::Transform {
                                position: [world_pos.x, world_pos.y, 0.0],
                                rotation: [0.0, 0.0, 0.0],
                                scale: [1.0, 1.0, 1.0],
                            });
                            
                            // Add name
                            let entity_name = if !metadata.sprites.is_empty() {
                                format!("{} ({})", asset.name, metadata.sprites[0].name)
                            } else {
                                asset.name.clone()
                            };
                            world.names.insert(entity, entity_name);
                            
                            // Add Sprite component (for rendering)
                            if let Some(first_sprite) = metadata.sprites.first() {
                                world.sprites.insert(entity, ecs::Sprite {
                                    texture_id: metadata.texture_path.clone(),
                                    width: first_sprite.width as f32,
                                    height: first_sprite.height as f32,
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    billboard: false,
                                    flip_x: false,
                                    flip_y: false,
                                    sprite_rect: Some([first_sprite.x, first_sprite.y, first_sprite.width, first_sprite.height]),
                                    pixels_per_unit: 100.0,
                                });
                            }
                            
                            // Add SpriteSheet component
                            let mut sprite_sheet = ecs::SpriteSheet::new(
                                metadata.texture_path.clone(),
                                metadata.texture_path.clone(),
                                metadata.texture_width,
                                metadata.texture_height,
                            );
                            
                            // Add all sprite frames
                            for sprite_def in &metadata.sprites {
                                sprite_sheet.add_frame(ecs::SpriteFrame {
                                    x: sprite_def.x,
                                    y: sprite_def.y,
                                    width: sprite_def.width,
                                    height: sprite_def.height,
                                    name: Some(sprite_def.name.clone()),
                                });
                            }
                            
                            world.sprite_sheets.insert(entity, sprite_sheet);
                            
                            // Add AnimatedSprite component (default to first frame, not playing)
                            let mut animated_sprite = ecs::AnimatedSprite::new(
                                metadata.texture_path.clone(),
                                0.1, // 10 FPS default
                            );
                            animated_sprite.current_frame = 0;
                            animated_sprite.playing = false; // Don't auto-play
                            world.animated_sprites.insert(entity, animated_sprite);
                            
                            // Select the newly created entity
                            *selected_entity = Some(entity);
                            
                            log::info!("Created sprite entity from {}", asset.name);
                        }
                    } else {
                        log::error!("Failed to load sprite metadata from {:?}", asset.path);
                    }
                } 
                // Handle XSG files
                else if asset.path.extension().and_then(|s| s.to_str()) == Some("xsg") {
                    if let Ok(xsg) = engine::assets::xsg_importer::XsgImporter::load_from_file(&asset.path) {
                         if let Some(screen_pos) = drag_drop.drop_position {
                            let center = rect.center();
                            let relative_x = screen_pos.x - center.x;
                            let relative_y = screen_pos.y - center.y;
                            
                             // Determine world position based on mode
                            let world_pos = match scene_view_mode {
                                SceneViewMode::Mode2D => {
                                     let p = scene_camera.screen_to_world(glam::Vec2::new(relative_x, relative_y));
                                     glam::Vec3::new(p.x, p.y, 0.0)
                                },
                                SceneViewMode::Mode3D => {
                                    // For 3D, we raycast against the grid plane (Y=0 usually, or Z=0?)
                                    // Our engine seems to use Y-up?
                                    // scene_camera typically looks at 0,0,0.
                                    // Let's assume we drop at Z=0 for now or raycast to ground plane.
                                    // Simple approximation: Unproject to a point at some distance or intersection with plane.
                                    // For now, let's just spawn at (0,0,0) offset by camera look?
                                    // Or better, use `screen_to_world` logic if available for 3D.
                                    // `scene_camera.screen_to_ray` would be ideal.
                                    // Let's spawn at (0,0,0) + some offset for now, or just (0,0,0).
                                    // Or try to interpret 2D screen pos on the grid.
                                    glam::Vec3::new(0.0, 0.0, 0.0) 
                                }
                            };

                            let path_id = asset.path.to_string_lossy().to_string();
                            match engine::assets::xsg_loader::XsgLoader::load_into_world(
                                &xsg,
                                world,
                                device,
                                queue, 
                                texture_manager,
                                &path_id,
                            ) {
                                Ok(entities) => {
                                    if !entities.is_empty() {
                                        // Select the first root
                                        if let Some(root_idx) = xsg.root_nodes.first() {
                                            if let Some(root_entity) = entities.get(*root_idx as usize) {
                                                *selected_entity = Some(*root_entity);
                                                
                                                // Move root(s) to drop position logic could go here
                                                // For now, they spawn at their XSG defined position (which might be 0,0,0)
                                            }
                                        }
                                        log::info!("Loaded XSG asset: {}", asset.name);
                                    }
                                },
                                Err(e) => log::error!("Failed to load XSG: {}", e),
                            }
                         }
                    } else {
                        log::error!("Failed to parse XSG file: {:?}", asset.path);
                    }
                }
                
                // Stop drag
                drag_drop.stop_drag();
            }
        }
    }
    
    // Camera controls overlay (bottom-left corner)
    let overlay_margin = 10.0;
    let overlay_pos = egui::pos2(rect.min.x + overlay_margin, rect.max.y - 60.0);
    
    ui.allocate_ui_at_rect(
        egui::Rect::from_min_size(overlay_pos, egui::vec2(300.0, 50.0)),
        |ui| {
            // Semi-transparent background
            ui.style_mut().visuals.window_fill = egui::Color32::from_rgba_premultiplied(30, 30, 35, 200);
            ui.style_mut().visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(80, 80, 90, 200));
            
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 35, 200))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(80, 80, 90, 200)))
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    crate::ui::camera_settings::render_scene_view_controls(ui, scene_camera);
                });
        }
    );
    
    // Camera state display overlay (top-left corner, only in 3D mode)
    if *scene_view_mode == SceneViewMode::Mode3D {
        let state_overlay_pos = egui::pos2(rect.min.x + overlay_margin, rect.min.y + overlay_margin);
        
        ui.allocate_ui_at_rect(
            egui::Rect::from_min_size(state_overlay_pos, egui::vec2(200.0, 100.0)),
            |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(30, 30, 35, 200))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(80, 80, 90, 200)))
                    .rounding(4.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        // Calculate FPS from delta_time
                        let fps = if delta_time > 0.0 { 1.0 / delta_time } else { 0.0 };
                        
                        // Get grid size for display
                        let _grid_size = if infinite_grid.enabled {
                            infinite_grid.calculate_grid_level(scene_camera.zoom)
                        } else {
                            scene_grid.size
                        };
                        
                        // Render camera state display
                        camera_state_display.render(ui, scene_camera, Some(infinite_grid), fps);
                    });
            }
        );
    }
}
