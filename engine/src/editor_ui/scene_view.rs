use ecs::{World, Entity};
use egui;
use crate::editor_ui::TransformTool;

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
pub fn render_scene_view(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    scene_view_tab: &mut usize,
    is_playing: bool,
    show_colliders: &bool,
    show_velocities: &bool,
    current_tool: &TransformTool,
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

            // Draw grid background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 50));

            // Draw grid lines
            let grid_size = 50.0;
            for i in 0..((rect.width() / grid_size) as usize) {
                let x = rect.min.x + i as f32 * grid_size;
                painter.line_segment(
                    [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
                );
            }
            for i in 0..((rect.height() / grid_size) as usize) {
                let y = rect.min.y + i as f32 * grid_size;
                painter.line_segment(
                    [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
                );
            }

            // Draw entities
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
