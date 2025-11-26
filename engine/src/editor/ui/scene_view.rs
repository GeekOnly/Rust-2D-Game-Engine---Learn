use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::{SceneCamera, SceneGrid};

/// Renders the Scene/Game view panel
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `world` - The ECS world containing entities and components
/// * `selected_entity` - The currently selected entity (if any)
/// * `scene_view_tab` - The active tab (0 = Scene, 1 = Game)
/// * `is_playing` - Whether the game is currently playing
/// * `show_colliders` - Whether to render collider boundaries
/// * `show_velocities` - Whether to render velocity vectors
/// * `current_tool` - The current transform tool (View, Move, Rotate, Scale)
/// * `scene_camera` - The scene camera for pan/zoom
/// * `scene_grid` - The grid system for snapping
pub fn render_scene_view(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    scene_view_tab: &mut usize,
    is_playing: bool,
    show_colliders: &bool,
    show_velocities: &bool,
    current_tool: &TransformTool,
    scene_camera: &mut SceneCamera,
    scene_grid: &SceneGrid,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(scene_view_tab, 0, "🎬 Scene");
        ui.selectable_value(scene_view_tab, 1, "🎮 Game");
    });
    ui.separator();

    match *scene_view_tab {
        0 => {
            // Scene View - Visual editor
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );
            let rect = response.rect;

            // === CAMERA CONTROLS ===
            // Handle middle mouse panning
            if response.dragged_by(egui::PointerButton::Middle) {
                let delta = response.drag_delta();
                let mouse_pos = response.interact_pointer_pos().unwrap_or(rect.center());
                
                if response.drag_started_by(egui::PointerButton::Middle) {
                    scene_camera.start_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                } else {
                    scene_camera.update_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            } else {
                scene_camera.stop_pan();
            }

            // Handle scroll wheel zooming
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y + i.raw_scroll_delta.y);
            if scroll_delta.abs() > 0.01 {
                let mouse_pos = response.hover_pos().unwrap_or(rect.center());
                scene_camera.zoom(scroll_delta * 0.01, glam::Vec2::new(mouse_pos.x, mouse_pos.y));
            }

            // Draw grid background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 50));

            // === GRID RENDERING ===
            if scene_grid.enabled {
                let grid_size = scene_grid.size * scene_camera.zoom;
                let grid_color = egui::Color32::from_rgba_premultiplied(
                    (scene_grid.color[0] * 255.0) as u8,
                    (scene_grid.color[1] * 255.0) as u8,
                    (scene_grid.color[2] * 255.0) as u8,
                    (scene_grid.color[3] * 255.0) as u8,
                );
                let axis_color = egui::Color32::from_rgba_premultiplied(
                    (scene_grid.axis_color[0] * 255.0) as u8,
                    (scene_grid.axis_color[1] * 255.0) as u8,
                    (scene_grid.axis_color[2] * 255.0) as u8,
                    (scene_grid.axis_color[3] * 255.0) as u8,
                );

                // Draw vertical grid lines
                let start_x = ((rect.min.x - scene_camera.position.x * scene_camera.zoom) / grid_size).floor() * grid_size;
                let mut x = start_x;
                while x < rect.max.x {
                    let world_x = (x - rect.center().x) / scene_camera.zoom + scene_camera.position.x;
                    let is_axis = (world_x / scene_grid.size).abs() % 5.0 < 0.1;
                    let color = if is_axis { axis_color } else { grid_color };
                    
                    painter.line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(1.0, color),
                    );
                    x += grid_size;
                }

                // Draw horizontal grid lines
                let start_y = ((rect.min.y - scene_camera.position.y * scene_camera.zoom) / grid_size).floor() * grid_size;
                let mut y = start_y;
                while y < rect.max.y {
                    let world_y = (y - rect.center().y) / scene_camera.zoom + scene_camera.position.y;
                    let is_axis = (world_y / scene_grid.size).abs() % 5.0 < 0.1;
                    let color = if is_axis { axis_color } else { grid_color };
                    
                    painter.line_segment(
                        [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                        egui::Stroke::new(1.0, color),
                    );
                    y += grid_size;
                }
            }

            // Draw entities (with camera transform)
            let center_x = rect.center().x;
            let center_y = rect.center().y;

            for (&entity, transform) in &world.transforms {
                // Apply camera transform
                let world_pos = glam::Vec2::new(transform.x(), transform.y());
                let screen_pos = scene_camera.world_to_screen(world_pos);
                let screen_x = center_x + screen_pos.x;
                let screen_y = center_y + screen_pos.y;

                if let Some(sprite) = world.sprites.get(&entity) {
                    let size = egui::vec2(sprite.width, sprite.height);
                    let color = egui::Color32::from_rgba_unmultiplied(
                        (sprite.color[0] * 255.0) as u8,
                        (sprite.color[1] * 255.0) as u8,
                        (sprite.color[2] * 255.0) as u8,
                        (sprite.color[3] * 255.0) as u8,
                    );

                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            egui::pos2(screen_x, screen_y),
                            size,
                        ),
                        2.0,
                        color,
                    );

                    // Draw selection outline
                    if *selected_entity == Some(entity) {
                        painter.rect_stroke(
                            egui::Rect::from_center_size(
                                egui::pos2(screen_x, screen_y),
                                size + egui::vec2(4.0, 4.0),
                            ),
                            2.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                        );
                    }
                } else {
                    // Draw as simple circle
                    painter.circle_filled(
                        egui::pos2(screen_x, screen_y),
                        5.0,
                        egui::Color32::from_rgb(150, 150, 150),
                    );

                    if *selected_entity == Some(entity) {
                        painter.circle_stroke(
                            egui::pos2(screen_x, screen_y),
                            8.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                        );
                    }
                }

                // GIZMOS: Draw collider boundaries (green wireframe)
                if *show_colliders {
                    if let Some(collider) = world.colliders.get(&entity) {
                        let collider_size = egui::vec2(collider.width, collider.height);
                        painter.rect_stroke(
                            egui::Rect::from_center_size(
                                egui::pos2(screen_x, screen_y),
                                collider_size,
                            ),
                            0.0,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
                        );

                        // Draw corner handles for selected entity
                        if *selected_entity == Some(entity) {
                            let half_w = collider.width / 2.0;
                            let half_h = collider.height / 2.0;
                            let handle_radius = 3.0;

                            // Four corners
                            let corners = [
                                (screen_x - half_w, screen_y - half_h),
                                (screen_x + half_w, screen_y - half_h),
                                (screen_x + half_w, screen_y + half_h),
                                (screen_x - half_w, screen_y + half_h),
                            ];

                            for (cx, cy) in corners {
                                painter.circle_filled(
                                    egui::pos2(cx, cy),
                                    handle_radius,
                                    egui::Color32::from_rgb(0, 255, 100),
                                );
                            }
                        }
                    }
                }

                // TRANSFORM GIZMO: Draw gizmo for selected entity based on current tool
                if *selected_entity == Some(entity) {
                    let gizmo_size = 50.0;
                    let handle_size = 8.0;

                    match current_tool {
                        TransformTool::View => {
                            // No gizmo in View mode (Q)
                        }
                        TransformTool::Move => {
                            // Move Tool (W) - Arrows for X/Y axes
                            // X axis arrow (Red)
                            let x_end = egui::pos2(screen_x + gizmo_size, screen_y);
                            painter.line_segment(
                                [egui::pos2(screen_x, screen_y), x_end],
                                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)),
                            );
                            painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));

                            // Y axis arrow (Green)
                            let y_end = egui::pos2(screen_x, screen_y + gizmo_size);
                            painter.line_segment(
                                [egui::pos2(screen_x, screen_y), y_end],
                                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 0)),
                            );
                            painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));

                            // Center handle (Both axes - Yellow)
                            painter.circle_filled(
                                egui::pos2(screen_x, screen_y),
                                handle_size,
                                egui::Color32::from_rgb(255, 255, 0),
                            );
                            painter.circle_stroke(
                                egui::pos2(screen_x, screen_y),
                                handle_size,
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 200, 0)),
                            );
                        }
                        TransformTool::Rotate => {
                            // Rotate Tool (E) - Circular ring
                            let radius = gizmo_size * 0.8;
                            painter.circle_stroke(
                                egui::pos2(screen_x, screen_y),
                                radius,
                                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 150, 255)),
                            );

                            // Rotation handles (4 points on circle)
                            let handle_positions = [
                                (radius, 0.0),           // Right
                                (0.0, radius),           // Bottom
                                (-radius, 0.0),          // Left
                                (0.0, -radius),          // Top
                            ];

                            for (dx, dy) in handle_positions.iter() {
                                painter.circle_filled(
                                    egui::pos2(screen_x + dx, screen_y + dy),
                                    handle_size,
                                    egui::Color32::from_rgb(0, 150, 255),
                                );
                            }

                            // Center dot
                            painter.circle_filled(
                                egui::pos2(screen_x, screen_y),
                                handle_size * 0.5,
                                egui::Color32::from_rgb(0, 150, 255),
                            );
                        }
                        TransformTool::Scale => {
                            // Scale Tool (R) - Box with corner handles
                            let box_size = gizmo_size * 0.7;

                            // Box outline
                            let top_left = egui::pos2(screen_x - box_size, screen_y - box_size);
                            let top_right = egui::pos2(screen_x + box_size, screen_y - box_size);
                            let bottom_left = egui::pos2(screen_x - box_size, screen_y + box_size);
                            let bottom_right = egui::pos2(screen_x + box_size, screen_y + box_size);

                            painter.line_segment([top_left, top_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                            painter.line_segment([top_right, bottom_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                            painter.line_segment([bottom_right, bottom_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                            painter.line_segment([bottom_left, top_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));

                            // Corner handles
                            painter.circle_filled(top_left, handle_size, egui::Color32::from_rgb(255, 150, 0));
                            painter.circle_filled(top_right, handle_size, egui::Color32::from_rgb(255, 150, 0));
                            painter.circle_filled(bottom_left, handle_size, egui::Color32::from_rgb(255, 150, 0));
                            painter.circle_filled(bottom_right, handle_size, egui::Color32::from_rgb(255, 150, 0));

                            // Center handle (uniform scale)
                            painter.circle_filled(
                                egui::pos2(screen_x, screen_y),
                                handle_size,
                                egui::Color32::from_rgb(255, 200, 100),
                            );
                        }
                    }
                }
            }

            // Draw velocity vectors as arrows (for debugging)
            if *show_velocities {
                for (&entity, velocity) in &world.velocities {
                    if let Some(transform) = world.transforms.get(&entity) {
                        let screen_x = center_x + transform.x();
                        let screen_y = center_y + transform.y();
                        let (vx, vy) = velocity;

                        // Only draw if velocity is non-zero
                        if vx.abs() > 0.1 || vy.abs() > 0.1 {
                            let arrow_scale = 0.5;
                            let end_x = screen_x + vx * arrow_scale;
                            let end_y = screen_y + vy * arrow_scale;

                            // Arrow line
                            painter.line_segment(
                                [egui::pos2(screen_x, screen_y), egui::pos2(end_x, end_y)],
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0)),
                            );

                            // Arrow head
                            let arrow_size = 5.0;
                            painter.circle_filled(
                                egui::pos2(end_x, end_y),
                                arrow_size,
                                egui::Color32::from_rgb(255, 255, 0),
                            );
                        }
                    }
                }
            }

            // INTERACTION: Handle transform gizmo dragging
            if let Some(sel_entity) = *selected_entity {
                if let Some(transform) = world.transforms.get(&sel_entity) {
                    let screen_x = center_x + transform.x();
                    let screen_y = center_y + transform.y();

                    if let Some(hover_pos) = response.hover_pos() {
                        let gizmo_size = 50.0;
                        let handle_size = 8.0;

                        // Check which handle is being hovered
                        let x_handle_pos = egui::pos2(screen_x + gizmo_size, screen_y);
                        let y_handle_pos = egui::pos2(screen_x, screen_y + gizmo_size);
                        let center_handle_pos = egui::pos2(screen_x, screen_y);

                        let dist_to_x = hover_pos.distance(x_handle_pos);
                        let dist_to_y = hover_pos.distance(y_handle_pos);
                        let dist_to_center = hover_pos.distance(center_handle_pos);

                        // Determine which axis to drag
                        let mut drag_axis = None;
                        if dist_to_center < handle_size * 1.5 {
                            drag_axis = Some(2); // Both axes
                        } else if dist_to_x < handle_size * 1.5 {
                            drag_axis = Some(0); // X axis
                        } else if dist_to_y < handle_size * 1.5 {
                            drag_axis = Some(1); // Y axis
                        }

                        // Handle dragging
                        if response.dragged() && drag_axis.is_some() {
                            let delta = response.drag_delta();

                            if let Some(transform) = world.transforms.get_mut(&sel_entity) {
                                match drag_axis.unwrap() {
                                    0 => transform.position[0] += delta.x, // X only
                                    1 => transform.position[1] += delta.y, // Y only
                                    2 => {
                                        // Both axes
                                        transform.position[0] += delta.x;
                                        transform.position[1] += delta.y;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        1 => {
            // Game View - show actual game when playing
            if is_playing {
                let (response, painter) = ui.allocate_painter(
                    ui.available_size(),
                    egui::Sense::hover(),
                );
                let rect = response.rect;

                // Draw game background
                painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 25, 35));

                // Draw entities (same as Scene but without grid)
                let center_x = rect.center().x;
                let center_y = rect.center().y;

                for (&entity, transform) in &world.transforms {
                    let screen_x = center_x + transform.x();
                    let screen_y = center_y + transform.y();

                    if let Some(sprite) = world.sprites.get(&entity) {
                        let size = egui::vec2(sprite.width, sprite.height);
                        let color = egui::Color32::from_rgba_unmultiplied(
                            (sprite.color[0] * 255.0) as u8,
                            (sprite.color[1] * 255.0) as u8,
                            (sprite.color[2] * 255.0) as u8,
                            (sprite.color[3] * 255.0) as u8,
                        );

                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                egui::pos2(screen_x, screen_y),
                                size,
                            ),
                            2.0,
                            color,
                        );
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Game View\n(Press Play to start)");
                });
            }
        }
        _ => {}
    }
}
