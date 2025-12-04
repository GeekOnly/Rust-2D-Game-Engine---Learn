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
use crate::editor::ui::TransformTool;
use crate::editor::{SceneCamera, SceneGrid, DragDropState};

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
    current_tool: &mut TransformTool,
    scene_camera: &mut SceneCamera,
    scene_grid: &SceneGrid,
    play_request: &mut bool,
    stop_request: &mut bool,
    dragging_entity: &mut Option<Entity>,
    drag_axis: &mut Option<u8>,
    scene_view_mode: &mut SceneViewMode,
    projection_mode: &mut ProjectionMode,
    transform_space: &mut TransformSpace,
    texture_manager: &mut crate::texture_manager::TextureManager,
    drag_drop: &mut DragDropState,
) {
    // Track previous mode to detect changes
    let previous_mode = *scene_view_mode;
    
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
    
    let focus_pressed = ui.input(|i| i.key_pressed(egui::Key::F));
    
    // Handle camera controls
    interaction::camera::handle_camera_controls(
        &response,
        scene_camera,
        rect,
        scene_view_mode,
        selected_entity,
        world,
    );

    // Background
    let bg_color = match scene_view_mode {
        SceneViewMode::Mode2D => egui::Color32::from_rgb(40, 40, 50),
        SceneViewMode::Mode3D => egui::Color32::from_rgb(50, 55, 65),
    };
    painter.rect_filled(rect, 0.0, bg_color);

    // Render grid
    if scene_grid.enabled {
        match scene_view_mode {
            SceneViewMode::Mode2D => rendering::grid::render_grid_2d(&painter, rect, scene_camera, scene_grid),
            SceneViewMode::Mode3D => rendering::grid::render_grid_3d(&painter, rect, scene_camera, scene_grid),
        }
    }
    
    // Render 3D scene gizmo (top-right corner)
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
                    ProjectionMode::Perspective => "⬜ Persp",
                    ProjectionMode::Isometric => "◇ Iso",
                };

                if ui.button(button_text).clicked() {
                    *projection_mode = match projection_mode {
                        ProjectionMode::Perspective => ProjectionMode::Isometric,
                        ProjectionMode::Isometric => ProjectionMode::Perspective,
                    };
                }
            }
        );
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
                &mut hovered_entity,
                &response,
                texture_manager,
                &ctx,
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
                projection_mode,
                center,
                selected_entity,
                show_colliders,
                show_velocities,
                &mut hovered_entity,
                &response,
            );
        }
    }

    // Focus on selected entity (F key)
    if focus_pressed {
        if let Some(entity) = *selected_entity {
            if let Some(transform) = world.transforms.get(&entity) {
                let pos = glam::Vec2::new(transform.x(), transform.y());
                let size = if let Some(sprite) = world.sprites.get(&entity) {
                    sprite.width.max(sprite.height)
                } else if world.meshes.contains_key(&entity) {
                    50.0
                } else {
                    10.0
                };
                let viewport_size = glam::Vec2::new(rect.width(), rect.height());
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
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;

            let transform_copy = transform.clone();
            
            rendering::gizmos::render_transform_gizmo(
                &painter,
                screen_x,
                screen_y,
                current_tool,
                scene_camera,
                scene_view_mode,
                transform_space,
                &transform_copy,
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
        if response.drag_released() {
            if let Some(asset) = drag_drop.get_dragged_asset() {
                // Check if it's a sprite file
                if asset.path.extension().and_then(|s| s.to_str()) == Some("sprite") {
                    // Load sprite metadata
                    if let Ok(metadata) = crate::editor::sprite_editor::SpriteMetadata::load(&asset.path) {
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
                    crate::editor::ui::camera_settings::render_scene_view_controls(ui, scene_camera);
                });
        }
    );
}
