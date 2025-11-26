use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::{SceneCamera, SceneGrid};

/// Scene view mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneViewMode {
    Mode2D,
    Mode3D,
}

/// Renders the Scene view panel (editor view only, no game view)
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
) {
    // Unity-like toolbar
    render_scene_toolbar(ui, current_tool, is_playing, play_request, stop_request);

    // Main scene view
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::click_and_drag(),
    );
    let rect = response.rect;

    // === CAMERA CONTROLS ===
    handle_camera_controls(&response, scene_camera, rect);

    // === BACKGROUND ===
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 50));

    // === GRID ===
    if scene_grid.enabled {
        render_grid(&painter, rect, scene_camera, scene_grid);
    }

    // === ENTITIES ===
    let center = rect.center();
    let mut hovered_entity: Option<Entity> = None;
    
    for (&entity, transform) in &world.transforms {
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        let screen_x = center.x + screen_pos.x;
        let screen_y = center.y + screen_pos.y;

        // Get entity bounds for click detection
        let entity_rect = if let Some(sprite) = world.sprites.get(&entity) {
            let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
        } else {
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(10.0, 10.0))
        };

        // Check if mouse is hovering this entity
        if let Some(hover_pos) = response.hover_pos() {
            if entity_rect.contains(hover_pos) {
                hovered_entity = Some(entity);
            }
        }

        // Draw entity
        render_entity(&painter, entity, transform, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity));
        
        // Draw gizmos
        if *show_colliders {
            render_collider_gizmo(&painter, entity, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity));
        }
        
        if *show_velocities {
            render_velocity_gizmo(&painter, entity, world, screen_x, screen_y);
        }
    }

    // === SELECTION ===
    if response.clicked() && !response.dragged() {
        if let Some(entity) = hovered_entity {
            *selected_entity = Some(entity);
        } else {
            *selected_entity = None;
        }
    }

    // === TRANSFORM GIZMO ===
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;

            render_transform_gizmo(&painter, screen_x, screen_y, current_tool);
            
            // Handle gizmo interaction (smooth dragging with state)
            handle_gizmo_interaction_stateful(&response, sel_entity, world, screen_x, screen_y, current_tool, scene_camera, dragging_entity, drag_axis);
        }
    }

    // Clear drag state when not dragging
    if !response.dragged() {
        *dragging_entity = None;
        *drag_axis = None;
    }
}

fn render_scene_toolbar(
    ui: &mut egui::Ui,
    current_tool: &mut TransformTool,
    is_playing: bool,
    play_request: &mut bool,
    stop_request: &mut bool,
) {
    ui.horizontal(|ui| {
        // Transform tools
        ui.selectable_value(current_tool, TransformTool::View, "🖐 View (Q)");
        ui.selectable_value(current_tool, TransformTool::Move, "➕ Move (W)");
        ui.selectable_value(current_tool, TransformTool::Rotate, "🔄 Rotate (E)");
        ui.selectable_value(current_tool, TransformTool::Scale, "📏 Scale (R)");
        
        ui.separator();
        
        // 2D/3D toggle (placeholder)
        ui.label("2D");
        
        ui.separator();
        
        // Pivot/Center toggle
        ui.label("Pivot: Center");
        
        ui.separator();
        
        // Space: Local/Global
        ui.label("Space: Local");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Play/Stop buttons
            if !is_playing {
                if ui.button("▶ Play").clicked() {
                    *play_request = true;
                }
            } else {
                if ui.button("⏹ Stop").clicked() {
                    *stop_request = true;
                }
            }
        });
    });
    ui.separator();
}

fn handle_camera_controls(response: &egui::Response, scene_camera: &mut SceneCamera, rect: egui::Rect) {
    // Middle mouse pan
    if response.dragged_by(egui::PointerButton::Middle) {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            if response.drag_started_by(egui::PointerButton::Middle) {
                scene_camera.start_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
            } else {
                scene_camera.update_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
            }
        }
    } else {
        scene_camera.stop_pan();
    }

    // Scroll wheel zoom
    let scroll_delta = response.ctx.input(|i| i.smooth_scroll_delta.y + i.raw_scroll_delta.y);
    if scroll_delta.abs() > 0.01 {
        let mouse_pos = response.hover_pos().unwrap_or(rect.center());
        scene_camera.zoom(scroll_delta * 0.01, glam::Vec2::new(mouse_pos.x, mouse_pos.y));
    }
}

fn render_grid(painter: &egui::Painter, rect: egui::Rect, scene_camera: &SceneCamera, scene_grid: &SceneGrid) {
    let grid_size = scene_grid.size * scene_camera.zoom;
    let grid_color = egui::Color32::from_rgba_premultiplied(
        (scene_grid.color[0] * 255.0) as u8,
        (scene_grid.color[1] * 255.0) as u8,
        (scene_grid.color[2] * 255.0) as u8,
        (scene_grid.color[3] * 255.0) as u8,
    );

    // Vertical lines
    let start_x = ((rect.min.x - scene_camera.position.x * scene_camera.zoom) / grid_size).floor() * grid_size;
    let mut x = start_x;
    while x < rect.max.x {
        painter.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.0, grid_color),
        );
        x += grid_size;
    }

    // Horizontal lines
    let start_y = ((rect.min.y - scene_camera.position.y * scene_camera.zoom) / grid_size).floor() * grid_size;
    let mut y = start_y;
    while y < rect.max.y {
        painter.line_segment(
            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
            egui::Stroke::new(1.0, grid_color),
        );
        y += grid_size;
    }
}

fn render_entity(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    is_selected: bool,
) {
    if let Some(sprite) = world.sprites.get(&entity) {
        let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );

        painter.rect_filled(
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
            2.0,
            color,
        );

        if is_selected {
            painter.rect_stroke(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size + egui::vec2(4.0, 4.0)),
                2.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
            );
        }
    } else {
        painter.circle_filled(egui::pos2(screen_x, screen_y), 5.0 * scene_camera.zoom, egui::Color32::from_rgb(150, 150, 150));
        
        if is_selected {
            painter.circle_stroke(
                egui::pos2(screen_x, screen_y),
                8.0 * scene_camera.zoom,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
            );
        }
    }
}

fn render_collider_gizmo(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    _is_selected: bool,
) {
    if let Some(collider) = world.colliders.get(&entity) {
        let size = egui::vec2(collider.width * scene_camera.zoom, collider.height * scene_camera.zoom);
        painter.rect_stroke(
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
            0.0,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
        );
    }
}

fn render_velocity_gizmo(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    screen_x: f32,
    screen_y: f32,
) {
    if let Some((vx, vy)) = world.velocities.get(&entity) {
        if vx.abs() > 0.1 || vy.abs() > 0.1 {
            let arrow_scale = 0.5;
            let end_x = screen_x + vx * arrow_scale;
            let end_y = screen_y + vy * arrow_scale;

            painter.line_segment(
                [egui::pos2(screen_x, screen_y), egui::pos2(end_x, end_y)],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0)),
            );
            painter.circle_filled(egui::pos2(end_x, end_y), 5.0, egui::Color32::from_rgb(255, 255, 0));
        }
    }
}

fn render_transform_gizmo(painter: &egui::Painter, screen_x: f32, screen_y: f32, current_tool: &TransformTool) {
    let gizmo_size = 50.0;
    let handle_size = 8.0;

    match current_tool {
        TransformTool::View => {}
        TransformTool::Move => {
            // X axis (Red)
            let x_end = egui::pos2(screen_x + gizmo_size, screen_y);
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), x_end],
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)),
            );
            painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));

            // Y axis (Green)
            let y_end = egui::pos2(screen_x, screen_y + gizmo_size);
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));

            // Center (Yellow)
            painter.circle_filled(egui::pos2(screen_x, screen_y), handle_size, egui::Color32::from_rgb(255, 255, 0));
        }
        TransformTool::Rotate => {
            let radius = gizmo_size * 0.8;
            painter.circle_stroke(
                egui::pos2(screen_x, screen_y),
                radius,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 150, 255)),
            );
        }
        TransformTool::Scale => {
            let box_size = gizmo_size * 0.7;
            let rect = egui::Rect::from_center_size(
                egui::pos2(screen_x, screen_y),
                egui::vec2(box_size * 2.0, box_size * 2.0),
            );
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
            
            // Corner handles
            painter.circle_filled(rect.left_top(), handle_size, egui::Color32::from_rgb(255, 150, 0));
            painter.circle_filled(rect.right_top(), handle_size, egui::Color32::from_rgb(255, 150, 0));
            painter.circle_filled(rect.left_bottom(), handle_size, egui::Color32::from_rgb(255, 150, 0));
            painter.circle_filled(rect.right_bottom(), handle_size, egui::Color32::from_rgb(255, 150, 0));
        }
    }
}

fn handle_gizmo_interaction_stateful(
    response: &egui::Response,
    entity: Entity,
    world: &mut World,
    screen_x: f32,
    screen_y: f32,
    current_tool: &TransformTool,
    scene_camera: &SceneCamera,
    dragging_entity: &mut Option<Entity>,
    drag_axis: &mut Option<u8>,
) {
    if *current_tool == TransformTool::View {
        return;
    }

    // Start dragging - determine which handle
    if response.drag_started() {
        if let Some(hover_pos) = response.hover_pos() {
            let gizmo_size = 50.0;
            let handle_size = 8.0;
            
            let x_handle = egui::pos2(screen_x + gizmo_size, screen_y);
            let y_handle = egui::pos2(screen_x, screen_y + gizmo_size);
            let center = egui::pos2(screen_x, screen_y);
            
            let dist_x = hover_pos.distance(x_handle);
            let dist_y = hover_pos.distance(y_handle);
            let dist_center = hover_pos.distance(center);
            
            if dist_center < handle_size * 1.5 {
                *dragging_entity = Some(entity);
                *drag_axis = Some(2); // Both axes
            } else if dist_x < handle_size * 1.5 {
                *dragging_entity = Some(entity);
                *drag_axis = Some(0); // X only
            } else if dist_y < handle_size * 1.5 {
                *dragging_entity = Some(entity);
                *drag_axis = Some(1); // Y only
            }
        }
    }

    // Continue dragging
    if response.dragged() && *dragging_entity == Some(entity) {
        let delta = response.drag_delta();
        
        // Convert screen delta to world delta (accounting for zoom)
        let world_delta_x = delta.x / scene_camera.zoom;
        let world_delta_y = delta.y / scene_camera.zoom;

        if let Some(transform) = world.transforms.get_mut(&entity) {
            match current_tool {
                TransformTool::Move => {
                    if let Some(axis) = *drag_axis {
                        match axis {
                            0 => transform.position[0] += world_delta_x, // X only
                            1 => transform.position[1] += world_delta_y, // Y only
                            2 => {
                                // Both axes
                                transform.position[0] += world_delta_x;
                                transform.position[1] += world_delta_y;
                            }
                            _ => {}
                        }
                    }
                }
                TransformTool::Rotate => {
                    let rotation_speed = 0.01;
                    transform.rotation[2] += (delta.x - delta.y) * rotation_speed;
                }
                TransformTool::Scale => {
                    let scale_speed = 0.01;
                    let scale_delta = (delta.x + delta.y) * scale_speed;
                    transform.scale[0] += scale_delta;
                    transform.scale[1] += scale_delta;
                    transform.scale[0] = transform.scale[0].max(0.1);
                    transform.scale[1] = transform.scale[1].max(0.1);
                }
                _ => {}
            }
        }
    }
}
