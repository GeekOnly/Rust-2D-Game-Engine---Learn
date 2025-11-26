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

/// Projection mode for 3D view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionMode {
    Isometric,
    Perspective,
}

/// Transform space mode (Local or World)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformSpace {
    Local,
    World,
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
    scene_view_mode: &mut SceneViewMode,
    projection_mode: &mut ProjectionMode,
    transform_space: &mut TransformSpace,
) {
    // Unity-like toolbar
    render_scene_toolbar(ui, current_tool, is_playing, play_request, stop_request, scene_view_mode, transform_space);

    // Main scene view
    let (response, painter) = ui.allocate_painter(
        ui.available_size(),
        egui::Sense::click_and_drag(),
    );
    let rect = response.rect;

    // === KEYBOARD SHORTCUTS ===
    let focus_pressed = ui.input(|i| i.key_pressed(egui::Key::F));
    
    // === CAMERA CONTROLS ===
    handle_camera_controls(&response, scene_camera, rect, scene_view_mode, selected_entity, world);

    // === BACKGROUND ===
    let bg_color = match scene_view_mode {
        SceneViewMode::Mode2D => egui::Color32::from_rgb(40, 40, 50),
        SceneViewMode::Mode3D => egui::Color32::from_rgb(50, 55, 65), // Slightly different for 3D
    };
    painter.rect_filled(rect, 0.0, bg_color);

    // === GRID ===
    if scene_grid.enabled {
        match scene_view_mode {
            SceneViewMode::Mode2D => render_grid_2d(&painter, rect, scene_camera, scene_grid),
            SceneViewMode::Mode3D => render_grid_3d(&painter, rect, scene_camera, scene_grid),
        }
    }
    
    // === 3D SCENE GIZMO (top-right corner) ===
    if *scene_view_mode == SceneViewMode::Mode3D {
        // Render gizmo with UI for clickable button
        let gizmo_size = 80.0;
        let margin = 20.0;
        let gizmo_center_x = rect.max.x - margin - gizmo_size / 2.0;
        let gizmo_center_y = rect.min.y + margin + gizmo_size / 2.0;
        
        render_scene_gizmo_visual(&painter, gizmo_center_x, gizmo_center_y, gizmo_size, scene_camera);
        
        // Projection mode button (using UI for interaction)
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

    // === ENTITIES ===
    let center = rect.center();
    let mut hovered_entity: Option<Entity> = None;
    
    // Collect and sort entities by Z position for proper depth rendering in 3D mode
    let mut entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .map(|(&e, t)| (e, t))
        .collect();
    
    if *scene_view_mode == SceneViewMode::Mode3D {
        // Sort by Z position (far to near) for painter's algorithm
        entities.sort_by(|a, b| a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    for (entity, transform) in entities {
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        let screen_x = center.x + screen_pos.x;
        let screen_y = center.y + screen_pos.y;

        // Get entity bounds for click detection
        let entity_rect = if let Some(sprite) = world.sprites.get(&entity) {
            let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
        } else if world.meshes.contains_key(&entity) && *scene_view_mode == SceneViewMode::Mode3D {
            // Calculate proper 3D bounds for meshes in 3D mode
            let scale = glam::Vec3::from(transform.scale);
            let world_size = 2.0; // Default cube is 2x2x2 units (like Blender)
            let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
            calculate_3d_cube_bounds(screen_x, screen_y, base_size, transform, scene_camera, projection_mode)
        } else if world.meshes.contains_key(&entity) {
            // 2D mode - simple square bounds
            let scale = glam::Vec3::from(transform.scale);
            let world_size = 2.0; // Default cube is 2x2x2 units
            let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(base_size, base_size))
        } else {
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(10.0, 10.0))
        };

        // Check if mouse is hovering this entity
        if let Some(hover_pos) = response.hover_pos() {
            if entity_rect.contains(hover_pos) {
                hovered_entity = Some(entity);
            }
        }

        // Draw entity (sprite or mesh)
        if world.meshes.contains_key(&entity) {
            render_mesh_entity(&painter, entity, transform, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity), scene_view_mode, projection_mode);
        } else {
            render_entity(&painter, entity, transform, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity));
        }
        
        // Draw gizmos
        if *show_colliders {
            render_collider_gizmo(&painter, entity, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity));
        }
        
        if *show_velocities {
            render_velocity_gizmo(&painter, entity, world, screen_x, screen_y);
        }
    }

    // === FOCUS ON SELECTED (F key) ===
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
                scene_camera.focus_on(pos, size);
            }
        }
    }
    
    // === SELECTION === (only if not controlling camera)
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

    // === TRANSFORM GIZMO === (only if not controlling camera)
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;

            // Clone transform data to avoid borrow issues
            let transform_copy = transform.clone();
            
            render_transform_gizmo(&painter, screen_x, screen_y, current_tool, scene_camera, scene_view_mode, transform_space, &transform_copy);
            
            // Handle gizmo interaction only if not controlling camera
            if !is_camera_control {
                handle_gizmo_interaction_stateful(&response, sel_entity, world, screen_x, screen_y, current_tool, scene_camera, dragging_entity, drag_axis, transform_space, &transform_copy);
            }
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
    scene_view_mode: &mut SceneViewMode,
    transform_space: &mut TransformSpace,
) {
    ui.horizontal(|ui| {
        // Transform tools
        ui.selectable_value(current_tool, TransformTool::View, "🖐 View (Q)");
        ui.selectable_value(current_tool, TransformTool::Move, "➕ Move (W)");
        ui.selectable_value(current_tool, TransformTool::Rotate, "� Roteate (E)");
        ui.selectable_value(current_tool, TransformTool::Scale, "📏 Scale (R)");
        
        ui.separator();
        
        // 2D/3D toggle (Unity-like)
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode2D, "2D");
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode3D, "3D");
        
        ui.separator();
        
        // Pivot/Center toggle
        ui.label("Pivot: Center");
        
        ui.separator();
        
        // Space: Local/World toggle
        ui.label("Space:");
        ui.selectable_value(transform_space, TransformSpace::Local, "Local");
        ui.selectable_value(transform_space, TransformSpace::World, "World");
        
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

fn handle_camera_controls(
    response: &egui::Response, 
    scene_camera: &mut SceneCamera, 
    rect: egui::Rect, 
    scene_view_mode: &SceneViewMode,
    selected_entity: &Option<ecs::Entity>,
    world: &ecs::World,
) {
    let is_alt_pressed = response.ctx.input(|i| i.modifiers.alt);
    
    // Alt + Left mouse button - Orbit around selected object (3D mode)
    if *scene_view_mode == SceneViewMode::Mode3D && is_alt_pressed {
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if response.drag_started_by(egui::PointerButton::Primary) {
                    // Get pivot point from selected entity or use camera position
                    let pivot = if let Some(entity) = selected_entity {
                        if let Some(transform) = world.transforms.get(entity) {
                            glam::Vec2::new(transform.x(), transform.y())
                        } else {
                            scene_camera.position
                        }
                    } else {
                        scene_camera.position
                    };
                    scene_camera.start_orbit(glam::Vec2::new(mouse_pos.x, mouse_pos.y), pivot);
                } else {
                    scene_camera.update_orbit(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            }
        } else {
            scene_camera.stop_orbit();
        }
    }
    
    // Right mouse button - Free look rotate (3D mode only)
    if *scene_view_mode == SceneViewMode::Mode3D && !is_alt_pressed {
        if response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    scene_camera.start_rotate(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                } else {
                    scene_camera.update_rotate(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            }
        } else {
            scene_camera.stop_rotate();
        }
    }
    
    // Middle mouse button - Pan camera
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

    // Scroll wheel - Zoom (smooth and responsive)
    let scroll_delta = response.ctx.input(|i| {
        // Use smooth scroll if available, otherwise use raw but scaled down
        if i.smooth_scroll_delta.y.abs() > 0.1 {
            i.smooth_scroll_delta.y
        } else {
            i.raw_scroll_delta.y * 0.1 // Scale down raw scroll
        }
    });
    
    if scroll_delta.abs() > 0.1 {
        let mouse_pos = response.hover_pos().unwrap_or(rect.center());
        let zoom_direction = if scroll_delta > 0.0 { 1.0 } else { -1.0 };
        scene_camera.zoom(zoom_direction, glam::Vec2::new(mouse_pos.x, mouse_pos.y));
    }
}

fn render_grid_2d(painter: &egui::Painter, rect: egui::Rect, scene_camera: &SceneCamera, scene_grid: &SceneGrid) {
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

fn render_grid_3d(painter: &egui::Painter, rect: egui::Rect, scene_camera: &SceneCamera, scene_grid: &SceneGrid) {
    let center = rect.center();
    let grid_world_size = scene_grid.size; // World space grid size
    
    // Grid colors - make more visible
    let grid_color = egui::Color32::from_rgba_premultiplied(
        (scene_grid.color[0] * 255.0) as u8,
        (scene_grid.color[1] * 255.0) as u8,
        (scene_grid.color[2] * 255.0) as u8,
        150, // More opaque for visibility
    );
    
    // Axis colors (X=Red, Z=Blue for 3D)
    let x_axis_color = egui::Color32::from_rgba_premultiplied(200, 50, 50, 200);
    let z_axis_color = egui::Color32::from_rgba_premultiplied(50, 100, 200, 200);
    
    // Camera parameters
    let yaw = scene_camera.rotation.to_radians();
    let pitch = scene_camera.pitch.to_radians();
    let zoom = scene_camera.zoom;
    
    // Grid range
    let grid_range = 20; // Number of grid lines in each direction (reduced for better performance)
    let fade_distance = 15.0 * grid_world_size; // Distance at which lines start to fade (in world units)
    
    // Helper function to project 3D point to 2D screen
    let project_3d = |x: f32, z: f32| -> egui::Pos2 {
        // Apply camera rotation
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = x * cos_yaw - z * sin_yaw;
        let rotated_z = x * sin_yaw + z * cos_yaw;
        
        // Apply pitch (vertical rotation)
        let y = 0.0; // Grid is on Y=0 plane
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let rotated_y = y * cos_pitch - rotated_z * sin_pitch;
        let final_z = y * sin_pitch + rotated_z * cos_pitch;
        
        // Perspective projection
        let distance = 500.0;
        let perspective_z = final_z + distance;
        let scale = if perspective_z > 10.0 {
            (distance / perspective_z) * zoom
        } else {
            zoom
        };
        
        egui::pos2(
            center.x + rotated_x * scale,
            center.y + rotated_y * scale,
        )
    };
    
    // Calculate fade based on distance from camera
    let calc_alpha = |x: f32, z: f32| -> u8 {
        let dist = (x * x + z * z).sqrt();
        if dist > fade_distance {
            let fade = 1.0 - ((dist - fade_distance) / fade_distance).min(1.0);
            (fade * 150.0) as u8 // Increased base alpha
        } else {
            150 // More opaque
        }
    };
    
    // Draw grid lines parallel to X axis (running along Z)
    for i in -grid_range..=grid_range {
        let x = i as f32 * grid_world_size;
        let is_axis = i == 0;
        
        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let z = j as f32 * grid_world_size;
            points.push(project_3d(x, z));
        }
        
        // Draw line segments
        for j in 0..points.len() - 1 {
            let z1 = ((j as i32) - grid_range) as f32 * grid_world_size;
            let alpha = calc_alpha(x, z1);
            
            if alpha > 10 {
                let color = if is_axis {
                    egui::Color32::from_rgba_premultiplied(
                        z_axis_color.r(),
                        z_axis_color.g(),
                        z_axis_color.b(),
                        alpha,
                    )
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        grid_color.r(),
                        grid_color.g(),
                        grid_color.b(),
                        alpha,
                    )
                };
                
                let width = if is_axis { 2.0 } else { 1.0 };
                painter.line_segment(
                    [points[j], points[j + 1]],
                    egui::Stroke::new(width, color),
                );
            }
        }
    }
    
    // Draw grid lines parallel to Z axis (running along X)
    for i in -grid_range..=grid_range {
        let z = i as f32 * grid_world_size;
        let is_axis = i == 0;
        
        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let x = j as f32 * grid_world_size;
            points.push(project_3d(x, z));
        }
        
        // Draw line segments
        for j in 0..points.len() - 1 {
            let x1 = ((j as i32) - grid_range) as f32 * grid_world_size;
            let alpha = calc_alpha(x1, z);
            
            if alpha > 10 {
                let color = if is_axis {
                    egui::Color32::from_rgba_premultiplied(
                        x_axis_color.r(),
                        x_axis_color.g(),
                        x_axis_color.b(),
                        alpha,
                    )
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        grid_color.r(),
                        grid_color.g(),
                        grid_color.b(),
                        alpha,
                    )
                };
                
                let width = if is_axis { 2.0 } else { 1.0 };
                painter.line_segment(
                    [points[j], points[j + 1]],
                    egui::Stroke::new(width, color),
                );
            }
        }
    }
}

fn render_scene_gizmo_visual(painter: &egui::Painter, center_x: f32, center_y: f32, gizmo_size: f32, scene_camera: &SceneCamera) {
    let gizmo_center = egui::pos2(center_x, center_y);
    
    // Background circle
    painter.circle_filled(gizmo_center, gizmo_size / 2.0, egui::Color32::from_rgba_premultiplied(30, 30, 35, 200));
    painter.circle_stroke(gizmo_center, gizmo_size / 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)));
    
    // Axis length
    let axis_len = gizmo_size * 0.35;
    
    // Get camera rotation
    let yaw_rad = scene_camera.get_rotation_radians();
    let pitch_rad = scene_camera.get_pitch_radians();
    
    // Calculate 3D axis directions and project to 2D
    // X axis (Red) - rotated by yaw
    let x_dir = (yaw_rad.cos(), yaw_rad.sin());
    let x_end = egui::pos2(
        gizmo_center.x + x_dir.0 * axis_len,
        gizmo_center.y + x_dir.1 * axis_len,
    );
    painter.line_segment(
        [gizmo_center, x_end],
        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 80, 80)),
    );
    painter.circle_filled(x_end, 6.0, egui::Color32::from_rgb(255, 80, 80));
    painter.text(
        egui::pos2(x_end.x + 12.0, x_end.y),
        egui::Align2::LEFT_CENTER,
        "X",
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(255, 80, 80),
    );
    
    // Y axis (Green) - Up (affected by pitch)
    let y_offset = pitch_rad.cos() * axis_len;
    let y_end = egui::pos2(gizmo_center.x, gizmo_center.y - y_offset);
    painter.line_segment(
        [gizmo_center, y_end],
        egui::Stroke::new(3.0, egui::Color32::from_rgb(80, 255, 80)),
    );
    painter.circle_filled(y_end, 6.0, egui::Color32::from_rgb(80, 255, 80));
    painter.text(
        egui::pos2(y_end.x, y_end.y - 12.0),
        egui::Align2::CENTER_BOTTOM,
        "Y",
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(80, 255, 80),
    );
    
    // Z axis (Blue) - perpendicular to X, affected by yaw
    let z_dir = (-yaw_rad.sin(), yaw_rad.cos());
    let z_end = egui::pos2(
        gizmo_center.x + z_dir.0 * axis_len,
        gizmo_center.y + z_dir.1 * axis_len,
    );
    painter.line_segment(
        [gizmo_center, z_end],
        egui::Stroke::new(3.0, egui::Color32::from_rgb(80, 80, 255)),
    );
    painter.circle_filled(z_end, 6.0, egui::Color32::from_rgb(80, 80, 255));
    painter.text(
        egui::pos2(z_end.x - 12.0, z_end.y),
        egui::Align2::RIGHT_CENTER,
        "Z",
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(80, 80, 255),
    );
    
    // Display rotation angles below gizmo
    let rotation_text = format!("Yaw: {:.0}° Pitch: {:.0}°", scene_camera.rotation, scene_camera.pitch);
    painter.text(
        egui::pos2(gizmo_center.x, gizmo_center.y + gizmo_size / 2.0 + 15.0),
        egui::Align2::CENTER_TOP,
        rotation_text,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(180, 180, 180),
    );
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

/// Calculate 2D bounding box from 3D cube vertices after projection
fn calculate_3d_cube_bounds(
    screen_x: f32,
    screen_y: f32,
    size: f32,
    transform: &ecs::Transform,
    scene_camera: &SceneCamera,
    projection_mode: &ProjectionMode,
) -> egui::Rect {
    let half = size / 2.0;
    let scale = transform.scale;
    
    // Define 8 vertices of a cube in local space
    let vertices = [
        Point3D::new(-half * scale[0], -half * scale[1], -half * scale[2]),
        Point3D::new(half * scale[0], -half * scale[1], -half * scale[2]),
        Point3D::new(half * scale[0], half * scale[1], -half * scale[2]),
        Point3D::new(-half * scale[0], half * scale[1], -half * scale[2]),
        Point3D::new(-half * scale[0], -half * scale[1], half * scale[2]),
        Point3D::new(half * scale[0], -half * scale[1], half * scale[2]),
        Point3D::new(half * scale[0], half * scale[1], half * scale[2]),
        Point3D::new(-half * scale[0], half * scale[1], half * scale[2]),
    ];
    
    // Apply object rotation first, then camera rotation
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| {
            let obj_rotated = v.rotate(&transform.rotation);
            obj_rotated
                .rotate_y(-scene_camera.rotation.to_radians())
                .rotate_x(scene_camera.pitch.to_radians())
        })
        .collect();
    
    // Project to 2D based on projection mode
    let projected: Vec<(f32, f32)> = rotated.iter()
        .map(|v| match projection_mode {
            ProjectionMode::Isometric => v.project_isometric(),
            ProjectionMode::Perspective => v.project_perspective(500.0, 300.0),
        })
        .collect();
    
    // Find min/max bounds
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    
    for (x, y) in projected {
        let screen_px = screen_x + x;
        let screen_py = screen_y + y;
        min_x = min_x.min(screen_px);
        max_x = max_x.max(screen_px);
        min_y = min_y.min(screen_py);
        max_y = max_y.max(screen_py);
    }
    
    egui::Rect::from_min_max(
        egui::pos2(min_x, min_y),
        egui::pos2(max_x, max_y),
    )
}

/// Render a 3D cube with full rotation support and proper face culling
fn render_3d_cube(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    size: f32,
    transform: &ecs::Transform,
    base_color: egui::Color32,
    scene_camera: &SceneCamera,
    projection_mode: &ProjectionMode,
) {
    let half = size / 2.0;
    let scale = transform.scale;
    
    // Define 8 vertices of a cube in local space
    let vertices = [
        Point3D::new(-half * scale[0], -half * scale[1], -half * scale[2]), // 0: front-bottom-left
        Point3D::new(half * scale[0], -half * scale[1], -half * scale[2]),  // 1: front-bottom-right
        Point3D::new(half * scale[0], half * scale[1], -half * scale[2]),   // 2: front-top-right
        Point3D::new(-half * scale[0], half * scale[1], -half * scale[2]),  // 3: front-top-left
        Point3D::new(-half * scale[0], -half * scale[1], half * scale[2]),  // 4: back-bottom-left
        Point3D::new(half * scale[0], -half * scale[1], half * scale[2]),   // 5: back-bottom-right
        Point3D::new(half * scale[0], half * scale[1], half * scale[2]),    // 6: back-top-right
        Point3D::new(-half * scale[0], half * scale[1], half * scale[2]),   // 7: back-top-left
    ];
    
    // Apply object rotation first, then camera rotation
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| {
            // Apply object rotation
            let obj_rotated = v.rotate(&transform.rotation);
            // Apply camera rotation for view
            obj_rotated
                .rotate_y(-scene_camera.rotation.to_radians())
                .rotate_x(scene_camera.pitch.to_radians())
        })
        .collect();
    
    // Project to 2D based on projection mode
    let projected: Vec<(f32, f32)> = rotated.iter()
        .map(|v| match projection_mode {
            ProjectionMode::Isometric => v.project_isometric(),
            ProjectionMode::Perspective => v.project_perspective(500.0, 300.0),
        })
        .collect();
    
    // Define faces with correct winding order (counter-clockwise from outside)
    let mut faces_with_depth: Vec<(Vec<usize>, f32, f32)> = vec![
        (vec![0, 1, 2, 3], 1.0, 0.0),   // Front face (Z-)
        (vec![5, 4, 7, 6], 0.6, 0.0),   // Back face (Z+)
        (vec![1, 0, 4, 5], 0.7, 0.0),   // Bottom face (Y-)
        (vec![3, 2, 6, 7], 0.9, 0.0),   // Top face (Y+)
        (vec![0, 3, 7, 4], 0.75, 0.0),  // Left face (X-)
        (vec![2, 1, 5, 6], 0.85, 0.0),  // Right face (X+)
    ];
    
    // Calculate average Z depth for each face
    for face_data in &mut faces_with_depth {
        let avg_z: f32 = face_data.0.iter()
            .map(|&i| rotated[i].z)
            .sum::<f32>() / face_data.0.len() as f32;
        face_data.2 = avg_z;
    }
    
    // Sort faces by depth (far to near) - painter's algorithm
    faces_with_depth.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    
    // Draw faces with depth-based shading and back-face culling
    for (face_indices, brightness, _depth) in faces_with_depth {
        // Back-face culling using normal vector
        if face_indices.len() >= 3 {
            // Get 3D positions for normal calculation
            let v0 = &rotated[face_indices[0]];
            let v1 = &rotated[face_indices[1]];
            let v2 = &rotated[face_indices[2]];
            
            // Calculate face normal using cross product
            let edge1 = (v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
            let edge2 = (v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
            
            let normal = (
                edge1.1 * edge2.2 - edge1.2 * edge2.1,
                edge1.2 * edge2.0 - edge1.0 * edge2.2,
                edge1.0 * edge2.1 - edge1.1 * edge2.0,
            );
            
            // View direction (camera looking at object)
            let view_dir = (0.0, 0.0, -1.0);
            
            // Dot product with view direction
            let dot = normal.0 * view_dir.0 + normal.1 * view_dir.1 + normal.2 * view_dir.2;
            
            // Skip back-facing polygons (dot product > 0 means facing away)
            if dot > 0.0 {
                continue;
            }
        }
        
        let points: Vec<egui::Pos2> = face_indices.iter()
            .map(|&i| {
                let (x, y) = projected[i];
                egui::pos2(screen_x + x, screen_y + y)
            })
            .collect();
        
        let face_color = egui::Color32::from_rgba_unmultiplied(
            (base_color.r() as f32 * brightness) as u8,
            (base_color.g() as f32 * brightness) as u8,
            (base_color.b() as f32 * brightness) as u8,
            base_color.a(),
        );
        
        painter.add(egui::Shape::convex_polygon(
            points,
            face_color,
            egui::Stroke::new(1.5, egui::Color32::from_gray(40)),
        ));
    }
}

/// Helper function to rotate a 2D point around origin
fn rotate_point_2d(x: f32, y: f32, angle_rad: f32) -> (f32, f32) {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    (x * cos_a - y * sin_a, x * sin_a + y * cos_a)
}

/// 3D point structure
#[derive(Clone, Copy, Debug)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Point3D {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Rotate around X axis
    fn rotate_x(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Y axis
    fn rotate_y(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Z axis
    fn rotate_z(&self, angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }
    
    /// Apply full 3D rotation (XYZ order)
    fn rotate(&self, rotation: &[f32; 3]) -> Self {
        self.rotate_x(rotation[0].to_radians())
            .rotate_y(rotation[1].to_radians())
            .rotate_z(rotation[2].to_radians())
    }
    
    /// Project to 2D screen space with perspective
    fn project_perspective(&self, fov: f32, distance: f32) -> (f32, f32) {
        let z_offset = self.z + distance;
        if z_offset <= 10.0 {
            return (self.x, self.y);
        }
        
        let scale = fov / z_offset;
        (self.x * scale, self.y * scale)
    }
    
    /// Project to 2D screen space (isometric) - proper isometric angles
    fn project_isometric(&self) -> (f32, f32) {
        // True isometric projection (120° angles)
        // X and Z axes at 30° from horizontal
        let iso_x = (self.x - self.z) * 0.866; // cos(30°) ≈ 0.866
        let iso_y = self.y + (self.x + self.z) * 0.5; // sin(30°) = 0.5
        (iso_x, iso_y)
    }
}

fn render_mesh_entity(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    is_selected: bool,
    scene_view_mode: &SceneViewMode,
    projection_mode: &ProjectionMode,
) {
    if let Some(mesh) = world.meshes.get(&entity) {
        let color = egui::Color32::from_rgba_unmultiplied(
            (mesh.color[0] * 255.0) as u8,
            (mesh.color[1] * 255.0) as u8,
            (mesh.color[2] * 255.0) as u8,
            (mesh.color[3] * 255.0) as u8,
        );
        
        // Apply object scale
        let scale = glam::Vec3::from(transform.scale);
        let world_size = 2.0; // Default cube is 2x2x2 units (like Blender)
        let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
        
        // Get camera rotation for 3D rendering
        let camera_rotation_rad = if *scene_view_mode == SceneViewMode::Mode3D {
            scene_camera.get_rotation_radians()
        } else {
            0.0
        };
        
        // Get object rotation (Z-axis rotation for now)
        let object_rotation_rad = transform.rotation[2].to_radians();
        let total_rotation = camera_rotation_rad + object_rotation_rad;
        
        // Store bounds for selection box
        let mesh_bounds = if *scene_view_mode == SceneViewMode::Mode3D && matches!(&mesh.mesh_type, ecs::MeshType::Cube) {
            Some(calculate_3d_cube_bounds(screen_x, screen_y, base_size, transform, scene_camera, projection_mode))
        } else {
            None
        };
        
        match &mesh.mesh_type {
            ecs::MeshType::Cube => {
                if *scene_view_mode == SceneViewMode::Mode3D {
                    render_3d_cube(painter, screen_x, screen_y, base_size, transform, color, scene_camera, projection_mode);
                } else {
                    // 2D view - simple square
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y),
                        egui::vec2(base_size, base_size),
                    );
                    painter.rect_filled(rect, 2.0, color);
                    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
                }
            }
            ecs::MeshType::Sphere => {
                painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
                painter.circle_stroke(egui::pos2(screen_x, screen_y), base_size / 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
            ecs::MeshType::Cylinder => {
                if *scene_view_mode == SceneViewMode::Mode3D {
                    // Draw cylinder (3D view)
                    let width = base_size;
                    let height = base_size * 1.5;
                    let ellipse_height = base_size * 0.3;
                    
                    // Body
                    let body_rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y),
                        egui::vec2(width, height),
                    );
                    painter.rect_filled(body_rect, 0.0, color);
                    
                    // Top cap (flattened circle)
                    let top_rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y - height/2.0),
                        egui::vec2(width, ellipse_height),
                    );
                    painter.rect_filled(top_rect, width/2.0, color);
                    
                    // Bottom cap (darker, flattened circle)
                    let bottom_color = egui::Color32::from_rgba_unmultiplied(
                        (mesh.color[0] * 200.0) as u8,
                        (mesh.color[1] * 200.0) as u8,
                        (mesh.color[2] * 200.0) as u8,
                        (mesh.color[3] * 255.0) as u8,
                    );
                    let bottom_rect = egui::Rect::from_center_size(
                        egui::pos2(screen_x, screen_y + height/2.0),
                        egui::vec2(width, ellipse_height),
                    );
                    painter.rect_filled(bottom_rect, width/2.0, bottom_color);
                } else {
                    // 2D view - circle
                    painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
                }
            }
            ecs::MeshType::Plane => {
                let size = base_size * 1.5;
                let rect = egui::Rect::from_center_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(size, size * 0.1),
                );
                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
            ecs::MeshType::Capsule => {
                let width = base_size * 0.6;
                let height = base_size * 1.5;
                let radius = width / 2.0;
                
                // Body rectangle
                let body_rect = egui::Rect::from_center_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(width, height - width),
                );
                painter.rect_filled(body_rect, 0.0, color);
                
                // Top cap
                painter.circle_filled(egui::pos2(screen_x, screen_y - (height - width)/2.0), radius, color);
                
                // Bottom cap
                painter.circle_filled(egui::pos2(screen_x, screen_y + (height - width)/2.0), radius, color);
                
                // Outline
                painter.circle_stroke(egui::pos2(screen_x, screen_y - (height - width)/2.0), radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
                painter.circle_stroke(egui::pos2(screen_x, screen_y + (height - width)/2.0), radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
        
        // Selection outline
        if is_selected {
            if let Some(bounds) = mesh_bounds {
                // Use calculated 3D bounds for selection box
                let expanded_bounds = bounds.expand(4.0);
                painter.rect_stroke(
                    expanded_bounds,
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            } else {
                // Fallback to simple square selection
                let selection_size = base_size + 8.0;
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(selection_size, selection_size)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            }
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

fn render_transform_gizmo(
    painter: &egui::Painter, 
    screen_x: f32, 
    screen_y: f32, 
    current_tool: &TransformTool,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    transform_space: &TransformSpace,
    transform: &ecs::Transform,
) {
    let gizmo_size = 50.0;
    let handle_size = 8.0;
    
    // Get rotation angle based on space mode
    let rotation_rad = if *scene_view_mode == SceneViewMode::Mode3D {
        match transform_space {
            TransformSpace::Local => {
                // Local space: combine object rotation with camera rotation
                scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
            }
            TransformSpace::World => {
                // World space: only camera rotation
                scene_camera.get_rotation_radians()
            }
        }
    } else {
        // 2D mode
        match transform_space {
            TransformSpace::Local => transform.rotation[2].to_radians(),
            TransformSpace::World => 0.0,
        }
    };

    match current_tool {
        TransformTool::View => {}
        TransformTool::Move => {
            // X axis (Red) - rotated
            let x_dir = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
            let x_end = egui::pos2(
                screen_x + x_dir.x * gizmo_size, 
                screen_y + x_dir.y * gizmo_size
            );
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), x_end],
                egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)),
            );
            painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));
            
            // Label
            painter.text(
                egui::pos2(x_end.x + 10.0, x_end.y),
                egui::Align2::LEFT_CENTER,
                "X",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(255, 0, 0),
            );

            // Y axis (Green) - always up in screen space
            let y_end = egui::pos2(screen_x, screen_y - gizmo_size);
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));
            
            // Label
            painter.text(
                egui::pos2(y_end.x, y_end.y - 10.0),
                egui::Align2::CENTER_BOTTOM,
                "Y",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(0, 255, 0),
            );
            
            // Z axis (Blue) - perpendicular to X in 3D mode
            if *scene_view_mode == SceneViewMode::Mode3D {
                let z_dir = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos());
                let z_end = egui::pos2(
                    screen_x + z_dir.x * gizmo_size, 
                    screen_y + z_dir.y * gizmo_size
                );
                painter.line_segment(
                    [egui::pos2(screen_x, screen_y), z_end],
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 0, 255)),
                );
                painter.circle_filled(z_end, handle_size, egui::Color32::from_rgb(0, 0, 255));
                
                // Label
                painter.text(
                    egui::pos2(z_end.x + 10.0, z_end.y),
                    egui::Align2::LEFT_CENTER,
                    "Z",
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(0, 0, 255),
                );
            }

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
    transform_space: &TransformSpace,
    transform: &ecs::Transform,
) {
    if *current_tool == TransformTool::View {
        return;
    }

    // Calculate rotation for gizmo handles
    let rotation_rad = match transform_space {
        TransformSpace::Local => {
            scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
        }
        TransformSpace::World => {
            scene_camera.get_rotation_radians()
        }
    };

    // Start dragging - determine which handle
    if response.drag_started() {
        if let Some(hover_pos) = response.hover_pos() {
            let gizmo_size = 50.0;
            let handle_size = 8.0;
            
            // Calculate rotated handle positions
            let x_dir = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
            let y_dir = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos()); // Perpendicular to X
            
            let x_handle = egui::pos2(
                screen_x + x_dir.x * gizmo_size,
                screen_y + x_dir.y * gizmo_size
            );
            let y_handle = egui::pos2(
                screen_x - gizmo_size * rotation_rad.sin(),
                screen_y - gizmo_size * rotation_rad.cos()
            );
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
        let screen_delta = glam::Vec2::new(delta.x, delta.y);
        
        // Calculate rotation for axis-aligned movement
        let rotation_rad = match transform_space {
            TransformSpace::Local => {
                scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
            }
            TransformSpace::World => {
                scene_camera.get_rotation_radians()
            }
        };
        
        // Rotate screen delta to world space
        let cos_r = rotation_rad.cos();
        let sin_r = rotation_rad.sin();
        let world_delta_x = (screen_delta.x * cos_r + screen_delta.y * sin_r) / scene_camera.zoom;
        let world_delta_y = (-screen_delta.x * sin_r + screen_delta.y * cos_r) / scene_camera.zoom;

        if let Some(transform_mut) = world.transforms.get_mut(&entity) {
            match current_tool {
                TransformTool::Move => {
                    if let Some(axis) = *drag_axis {
                        match axis {
                            0 => {
                                // X axis only - project delta onto X axis
                                transform_mut.position[0] += world_delta_x;
                            }
                            1 => {
                                // Y axis only - project delta onto Y axis  
                                transform_mut.position[1] -= world_delta_y;
                            }
                            2 => {
                                // Both axes - free movement
                                transform_mut.position[0] += world_delta_x;
                                transform_mut.position[1] -= world_delta_y;
                            }
                            _ => {}
                        }
                    }
                }
                TransformTool::Rotate => {
                    let rotation_speed = 0.01;
                    transform_mut.rotation[2] += (delta.x - delta.y) * rotation_speed;
                }
                TransformTool::Scale => {
                    let scale_speed = 0.01;
                    let scale_delta = (delta.x + delta.y) * scale_speed;
                    transform_mut.scale[0] += scale_delta;
                    transform_mut.scale[1] += scale_delta;
                    transform_mut.scale[0] = transform_mut.scale[0].max(0.1);
                    transform_mut.scale[1] = transform_mut.scale[1].max(0.1);
                }
                _ => {}
            }
        }
    }
}
