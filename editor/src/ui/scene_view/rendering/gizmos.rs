//! Gizmo Rendering
//!
//! Functions for rendering various gizmos (scene gizmo, transform gizmo, colliders, etc).

use ecs::{World, Entity};
use egui;
use crate::ui::TransformTool;
use crate::SceneCamera;
use super::super::types::*;
use super::projection_3d;

/// Render scene gizmo (XYZ axes in top-right corner)
pub fn render_scene_gizmo_visual(
    painter: &egui::Painter,
    center_x: f32,
    center_y: f32,
    gizmo_size: f32,
    scene_camera: &SceneCamera,
) {
    let gizmo_center = egui::pos2(center_x, center_y);
    
    // Background circle
    painter.circle_filled(gizmo_center, gizmo_size / 2.0, egui::Color32::from_rgba_premultiplied(30, 30, 35, 200));
    painter.circle_stroke(gizmo_center, gizmo_size / 2.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)));
    
    // Calculate proper 3D axis directions based on camera view
    let axis_len = gizmo_size * 0.35;
    
    // Get camera rotation for proper axis calculation
    // Get camera rotation for proper axis calculation
    // yaw_rad and pitch_rad were unused here
    // let yaw_rad = scene_camera.get_rotation_radians();
    // let pitch_rad = scene_camera.get_pitch_radians();
    
    // Calculate view matrix to get proper axis directions
    let view_matrix = scene_camera.get_view_matrix();
    
    // Extract right, up, and forward vectors from view matrix (inverted for gizmo display)
    let right = glam::Vec3::new(view_matrix.x_axis.x, view_matrix.y_axis.x, view_matrix.z_axis.x);
    let up = glam::Vec3::new(view_matrix.x_axis.y, view_matrix.y_axis.y, view_matrix.z_axis.y);
    let _forward = glam::Vec3::new(view_matrix.x_axis.z, view_matrix.y_axis.z, view_matrix.z_axis.z);
    
    // For scene gizmo, we want to show world axes as they appear from camera view
    // X = Right (Red), Y = Up (Green), Z = Forward (Blue)
    let project_axis_simple = |direction: glam::Vec3| -> egui::Pos2 {
        // Project 3D direction to 2D screen space
        let screen_x = direction.dot(right);
        let screen_y = -direction.dot(up); // Flip Y for screen coordinates
        
        egui::pos2(
            gizmo_center.x + screen_x * axis_len,
            gizmo_center.y + screen_y * axis_len,
        )
    };
    
    // Render X axis (Red) - World X direction
    let x_end = project_axis_simple(glam::Vec3::X);
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
    
    // Render Y axis (Green) - World Y direction
    let y_end = project_axis_simple(glam::Vec3::Y);
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
    
    // Render Z axis (Blue) - World Z direction
    let z_end = project_axis_simple(glam::Vec3::Z);
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
    
    // Display projection mode and rotation angles below gizmo
    let projection_mode = match scene_camera.projection_mode {
        crate::SceneProjectionMode::Perspective => "Persp",
        crate::SceneProjectionMode::Isometric => "Iso",
    };
    let rotation_text = format!("{} | Yaw: {:.0}° Pitch: {:.0}°", projection_mode, scene_camera.rotation, scene_camera.pitch);
    painter.text(
        egui::pos2(gizmo_center.x, gizmo_center.y + gizmo_size / 2.0 + 15.0),
        egui::Align2::CENTER_TOP,
        rotation_text,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(180, 180, 180),
    );
}

/// Render transform gizmo for selected entity
pub fn render_transform_gizmo(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    current_tool: &TransformTool,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    transform_space: &TransformSpace,
    transform: &ecs::Transform,
    viewport_rect: Option<egui::Rect>,
    highlight_axis: Option<u8>,
) {
    let gizmo_size = 80.0;
    let handle_size = 10.0;
    
    // Choose rendering mode based on View Mode
    match scene_view_mode {
        SceneViewMode::Mode3D => {
            // --------------------------------------------------------
            // 3D MODE: True 3D projection
            // --------------------------------------------------------
            if let Some(rect) = viewport_rect {
                let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                let world_pos = glam::Vec3::from(transform.position);

                // Re-project origin to ensure consistency
                let origin_screen = match projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
                    Some(pos) => egui::pos2(rect.min.x + pos.x, rect.min.y + pos.y),
                    None => egui::pos2(screen_x, screen_y), // Fallback
                };

                // Determine Basis Vectors
                let (right, up, forward) = match transform_space {
                    TransformSpace::Local => {
                        // Create rotation from Euler (XYZ)
                        let rot = glam::Quat::from_euler(
                            glam::EulerRot::XYZ, 
                            transform.rotation[0].to_radians(), 
                            transform.rotation[1].to_radians(), 
                            transform.rotation[2].to_radians()
                        );
                        // Unity/Engine conventions: 
                        // Assuming Right-Handed Y-Up, but let's stick to standard axes rotated.
                        (rot * glam::Vec3::X, rot * glam::Vec3::Y, rot * -glam::Vec3::Z) 
                    },
                    TransformSpace::World => (glam::Vec3::X, glam::Vec3::Y, -glam::Vec3::Z),
                };

                // Calculate Visual Scale Factor
                // We want the gizmo to appear roughly constant size on screen.
                let transform3d_temp = projection_3d::Transform3D::new(world_pos, 0.0, glam::Vec2::ONE);
                let cam_dist = transform3d_temp.depth_from_camera(scene_camera);
                let safe_dist = cam_dist.max(0.1);
                let scale = safe_dist * 0.15; // Tuned constant

                let project = |pos: glam::Vec3| -> Option<egui::Pos2> {
                    projection_3d::world_to_screen(pos, scene_camera, viewport_size)
                        .map(|p| egui::pos2(rect.min.x + p.x, rect.min.y + p.y))
                };
                
                let p_origin = origin_screen;
                let p_right = project(world_pos + right * scale);
                let p_up = project(world_pos + up * scale);
                let p_fwd = project(world_pos + forward * scale);

                let is_highlighted = |axis: u8| -> bool {
                    highlight_axis == Some(axis) || highlight_axis == Some(3) // 3 usually implies all/free/uniform for Move
                };

                match current_tool {
                    TransformTool::Move => {
                        // X Axis (Red)
                        let col_x = if is_highlighted(0) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(255, 50, 50) };
                        if let Some(end) = p_right {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_x));
                            painter.circle_filled(end, handle_size, col_x);
                            painter.text(egui::pos2(end.x + 5.0, end.y), egui::Align2::LEFT_CENTER, "X", egui::FontId::proportional(14.0), col_x);
                        }
                        // Y Axis (Green)
                        let col_y = if is_highlighted(1) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 255, 50) };
                        if let Some(end) = p_up {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_y));
                            painter.circle_filled(end, handle_size, col_y);
                            painter.text(egui::pos2(end.x, end.y - 12.0), egui::Align2::CENTER_BOTTOM, "Y", egui::FontId::proportional(14.0), col_y);
                        }
                        // Z Axis (Blue)
                        let col_z = if is_highlighted(2) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 100, 255) };
                        if let Some(end) = p_fwd {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_z));
                            painter.circle_filled(end, handle_size, col_z);
                            painter.text(egui::pos2(end.x - 5.0, end.y), egui::Align2::RIGHT_CENTER, "Z", egui::FontId::proportional(14.0), col_z);
                        }
                        // Center (Free Move)
                        let col_c = if highlight_axis == Some(3) { egui::Color32::WHITE } else { egui::Color32::YELLOW };
                        painter.circle_filled(p_origin, handle_size * 0.8, col_c);
                    }
                    TransformTool::Rotate => {
                        // 3D Rotation Gizmo: Render 3 rings
                        let radius_world = scale * 4.0; // Rough visual size
                        let segments = 32;

                        // Function to draw a ring in a plane defined by two vectors
                        let draw_ring = |axis_u: glam::Vec3, axis_v: glam::Vec3, color: egui::Color32, label: &str, label_pos_factor: (f32, f32)| {
                            let mut points = Vec::with_capacity(segments + 1);
                            for i in 0..=segments {
                                let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                                let offset = axis_u * angle.cos() * radius_world + axis_v * angle.sin() * radius_world;
                                if let Some(p) = project(world_pos + offset) {
                                    points.push(p);
                                }
                            }
                            if points.len() > 1 {
                                painter.add(egui::Shape::line(points, egui::Stroke::new(2.5, color)));
                            }
                            
                            // Draw Axis Label (X/Y/Z) at specific angle
                            let _angle = std::f32::consts::PI / 4.0; // 45 degrees
                            let label_offset = axis_u * label_pos_factor.0 * radius_world + axis_v * label_pos_factor.1 * radius_world;
                             if let Some(p) = project(world_pos + label_offset * 1.1) {
                                 painter.text(p, egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(12.0), color);
                             }
                        };

                        // X-Axis Ring (Rotates around X -> lies in Y/Z plane) -> Up/Forward
                        let col_x = if highlight_axis == Some(0) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(255, 50, 50) };
                        draw_ring(up, forward, col_x, "X", (0.0, 1.0));

                        // Y-Axis Ring (Rotates around Y -> lies in X/Z plane) -> Right/Forward
                        // Note: X/Z plane
                        let col_y = if highlight_axis == Some(1) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 255, 50) };
                        draw_ring(right, forward, col_y, "Y", (1.0, 0.0));

                        // Z-Axis Ring (Rotates around Z -> lies in X/Y plane) -> Right/Up
                        let col_z = if highlight_axis == Some(2) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 100, 255) };
                        draw_ring(right, up, col_z, "Z", (0.7, 0.7));

                        // Outer white circle (Screen space Billboarding)
                        let radius_screen = gizmo_size * 0.8;
                        painter.circle_stroke(p_origin, radius_screen, egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 100)));
                    }
                    TransformTool::Scale => {
                        // X Axis (Red)
                        let col_x = if highlight_axis == Some(0) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(255, 50, 50) };
                        if let Some(end) = p_right {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_x));
                            painter.rect_filled(egui::Rect::from_center_size(end, egui::vec2(handle_size*1.5, handle_size*1.5)), 0.0, col_x);
                        }
                        // Y Axis (Green)
                        let col_y = if highlight_axis == Some(1) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 255, 50) };
                        if let Some(end) = p_up {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_y));
                            painter.rect_filled(egui::Rect::from_center_size(end, egui::vec2(handle_size*1.5, handle_size*1.5)), 0.0, col_y);
                        }
                        // Z Axis (Blue)
                        let col_z = if highlight_axis == Some(2) { egui::Color32::YELLOW } else { egui::Color32::from_rgb(50, 100, 255) };
                        if let Some(end) = p_fwd {
                            painter.line_segment([p_origin, end], egui::Stroke::new(4.0, col_z));
                            painter.rect_filled(egui::Rect::from_center_size(end, egui::vec2(handle_size*1.5, handle_size*1.5)), 0.0, col_z);
                        }
                        // Center
                        let col_c = if highlight_axis == Some(3) { egui::Color32::YELLOW } else { egui::Color32::WHITE };
                        painter.rect_filled(egui::Rect::from_center_size(p_origin, egui::vec2(handle_size*1.5, handle_size*1.5)), 0.0, col_c);
                    }
                    _ => {}
                }
            } else {
                // Fallback if no viewport rect (shouldn't happen with correct calls)
            }
        }
        SceneViewMode::Mode2D => {
            // --------------------------------------------------------
            // 2D MODE (Legacy logic)
            // --------------------------------------------------------
            let rotation_rad = match transform_space {
                TransformSpace::Local => transform.rotation[2].to_radians(),
                TransformSpace::World => 0.0,
            };

            match current_tool {
                TransformTool::View => {}
                TransformTool::Move => {
                    // X Axis (Red)
                    let x_dir = (rotation_rad.cos(), rotation_rad.sin());
                    // Invert Y for screen space (World Up = Screen Up/Negative)
                    let x_end = egui::pos2(screen_x + x_dir.0 * gizmo_size, screen_y - x_dir.1 * gizmo_size);
                    painter.line_segment([egui::pos2(screen_x, screen_y), x_end], egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 0, 0)));
                    painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));
                    painter.text(egui::pos2(x_end.x + 12.0, x_end.y), egui::Align2::LEFT_CENTER, "X", egui::FontId::proportional(14.0), egui::Color32::from_rgb(255, 0, 0));

                    // Y Axis (Green)
                    let y_angle = rotation_rad + std::f32::consts::PI / 2.0; // +90 degrees for Y axis
                    let y_dir = (y_angle.cos(), y_angle.sin());
                    // Invert Y for screen space
                    let y_end = egui::pos2(screen_x + y_dir.0 * gizmo_size, screen_y - y_dir.1 * gizmo_size);
                    painter.line_segment([egui::pos2(screen_x, screen_y), y_end], egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)));
                    painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));
                    painter.text(egui::pos2(y_end.x, y_end.y - 12.0), egui::Align2::CENTER_BOTTOM, "Y", egui::FontId::proportional(14.0), egui::Color32::from_rgb(0, 255, 0));
                    
                    painter.circle_filled(egui::pos2(screen_x, screen_y), handle_size * 1.2, egui::Color32::from_rgb(255, 255, 0));
                }
                TransformTool::Rotate => {
                    let radius = gizmo_size * 0.8;
                    painter.circle_stroke(egui::pos2(screen_x, screen_y), radius, egui::Stroke::new(5.0, egui::Color32::from_rgb(0, 150, 255)));
                    painter.circle_filled(egui::pos2(screen_x, screen_y), 3.0, egui::Color32::from_rgb(0, 150, 255));
                    for i in 0..4 {
                        let angle = (i as f32) * std::f32::consts::PI / 2.0 + rotation_rad;
                        let dot_x = screen_x + radius * angle.cos();
                        let dot_y = screen_y - radius * angle.sin(); // Invert Y
                        painter.circle_filled(egui::pos2(dot_x, dot_y), 4.0, egui::Color32::from_rgb(0, 150, 255));
                    }
                }
                TransformTool::Scale => {
                    // X Axis (Red)
                    let x_dir = (rotation_rad.cos(), rotation_rad.sin());
                    // Invert Y
                    let x_end = egui::pos2(screen_x + x_dir.0 * gizmo_size, screen_y - x_dir.1 * gizmo_size);
                    painter.line_segment([egui::pos2(screen_x, screen_y), x_end], egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 0, 0)));
                    painter.rect_filled(egui::Rect::from_center_size(x_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)), 0.0, egui::Color32::from_rgb(255, 0, 0));

                    // Y Axis (Green)
                    let y_angle = rotation_rad + std::f32::consts::PI / 2.0;
                    let y_dir = (y_angle.cos(), y_angle.sin());
                    // Invert Y
                    let y_end = egui::pos2(screen_x + y_dir.0 * gizmo_size, screen_y - y_dir.1 * gizmo_size);
                    painter.line_segment([egui::pos2(screen_x, screen_y), y_end], egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)));
                    painter.rect_filled(egui::Rect::from_center_size(y_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)), 0.0, egui::Color32::from_rgb(0, 255, 0));
                    
                    painter.rect_filled(egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(handle_size * 2.2, handle_size * 2.2)), 0.0, egui::Color32::from_rgb(255, 255, 255));
                }
            }
        }
    }
}

/// Render collider gizmo
pub fn render_collider_gizmo(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    _is_selected: bool,
    is_2d_mode: bool,
) {
    if let Some(collider) = world.colliders.get(&entity) {
        // Get entity transform for rotation and scale
        let (rotation_rad, scale) = world.transforms.get(&entity)
            .map(|t| (
                t.rotation[2].to_radians(),
                glam::Vec2::new(t.scale[0], t.scale[1])
            ))
            .unwrap_or((0.0, glam::Vec2::ONE));
        
        // Apply transform.scale to collider size (Unity-like: size * scale)
        let world_width = collider.get_world_width(scale.x);
        let world_height = collider.get_world_height(scale.y);
        let size = egui::vec2(
            world_width * scene_camera.zoom,
            world_height * scene_camera.zoom
        );
        
        // Apply offset
        let world_offset = collider.get_world_offset(scale.x, scale.y);
        let offset_screen = egui::vec2(
            world_offset[0] * scene_camera.zoom,
            world_offset[1] * scene_camera.zoom
        );
        let screen_x = screen_x + offset_screen.x;
        // Only flip Y in 3D mode because 3D projection usually flips Y
        // But in our Y-Up 2D mode, +WorldY means -ScreenY (Up), so we subtract offset in both cases
        let screen_y = if is_2d_mode {
            screen_y - offset_screen.y  // Y-Up 2D: Subtract offset to go Up on screen
        } else {
            screen_y - offset_screen.y  // 3D mode: Subtract offset (already handled by projection?)
        };
        
        if rotation_rad.abs() < 0.01 {
            // No rotation - use simple rect
            painter.rect_stroke(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                0.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
            );
        } else {
            // Has rotation - draw as rotated polygon outline
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
            
            // Draw rotated collider outline
            painter.add(egui::Shape::closed_line(
                corners.to_vec(),
                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
            ));
        }
    }
}

/// Render velocity gizmo
pub fn render_velocity_gizmo(
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

/// Render Unity-style camera gizmo (trapezoid shape in yellow)
pub fn render_camera_gizmo(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    camera_entity: Entity,
    world: &World,
    _scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
) {
    // Get camera component and transform from entity
    let (camera_component, camera_transform) = match (world.cameras.get(&camera_entity), world.transforms.get(&camera_entity)) {
        (Some(cam), Some(transform)) => (cam, transform),
        _ => {
            // No camera component or transform, render a default camera gizmo
            render_default_camera_gizmo(painter, screen_x, screen_y, scene_view_mode);
            return;
        }
    };
    
    // Fixed size for better visibility
    let size = 50.0; // Larger, fixed size
    
    // Bright, highly visible colors
    let camera_color = egui::Color32::from_rgb(255, 255, 0); // Bright yellow
    let outline_color = egui::Color32::from_rgb(255, 200, 0); // Slightly darker yellow
    
    // Get camera rotation from transform (Z rotation = yaw in Unity)
    // Unity convention: rotation[2] is yaw (rotation around Y axis)
    // When rotation = 0°, camera looks down -Z axis (toward scene at Z=0)
    let camera_rotation_rad = camera_transform.rotation[2].to_radians();
    
    // Determine gizmo style based on camera projection type
    let use_2d_style = match camera_component.projection {
        ecs::CameraProjection::Orthographic => true,  // Orthographic cameras use 2D style
        ecs::CameraProjection::Perspective => false,  // Perspective cameras use 3D style
    };
    
    if use_2d_style {
        // Orthographic camera: Draw wireframe cube with proper rotation
        render_rotated_camera_trapezoid(painter, screen_x, screen_y, size, camera_rotation_rad, camera_color, camera_component);
    } else {
        // Perspective camera: Draw 3D camera icon with proper rotation
        render_rotated_camera_3d_icon(painter, screen_x, screen_y, size, camera_rotation_rad, camera_color, outline_color, camera_component);
    }
    
    // Draw camera information based on component values
    let camera_info = match camera_component.projection {
        ecs::CameraProjection::Orthographic => {
            format!("Cam {} (Ortho {:.1})", camera_entity, camera_component.orthographic_size)
        }
        ecs::CameraProjection::Perspective => {
            format!("Cam {} (Persp {:.0}°)", camera_entity, camera_component.fov)
        }
    };
    
    painter.text(
        egui::pos2(screen_x + size * 0.6, screen_y - size * 0.6),
        egui::Align2::LEFT_TOP,
        camera_info,
        egui::FontId::proportional(12.0),
        camera_color,
    );
    
    // Get camera transform to check Z position
    if let Some(transform) = world.transforms.get(&camera_entity) {
        let z_pos = transform.position[2];
        
        // Show warning if camera is not positioned correctly for Unity-style setup
        if z_pos < 10.0 {
            painter.text(
                egui::pos2(screen_x + size * 0.6, screen_y - size * 0.3),
                egui::Align2::LEFT_TOP,
                format!("Z={:.1} (suggest Z=+20)", z_pos),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(255, 150, 100), // Orange warning
            );
        }
    }
}

/// Render clean Unity-style camera gizmo with Game View preview
/// NOTE: rotation_rad is the CAMERA's rotation in world space, NOT affected by Scene Camera
fn render_rotated_camera_trapezoid(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    size: f32,
    rotation_rad: f32,
    _camera_color: egui::Color32,
    camera_component: &ecs::Camera,
) {
    // IMPORTANT: This gizmo is drawn in SCREEN SPACE using the camera's WORLD SPACE rotation
    // It should NOT be affected by Scene Camera rotation - it shows the camera's actual direction

    // Unity camera convention: rotation_rad = 0° means camera looks down -Z (toward scene)
    // In 2D screen space: we want to show this direction clearly
    // rotation_rad is already in world space from camera_transform.rotation[2]

    let cos_r = rotation_rad.cos();
    let sin_r = rotation_rad.sin();

    // Helper function to rotate a point around the center (in screen space)
    let rotate_point = |x: f32, y: f32| -> egui::Pos2 {
        let rotated_x = x * cos_r - y * sin_r;
        let rotated_y = x * sin_r + y * cos_r;
        egui::pos2(screen_x + rotated_x, screen_y + rotated_y)
    };

    // Clean, simple camera gizmo
    let gizmo_width = size * 0.8;
    let gizmo_height = size * 0.5;

    // Draw main Game View preview rectangle
    render_clean_game_view_preview(painter, screen_x, screen_y, gizmo_width, gizmo_height, rotation_rad, camera_component);

    // Draw simple direction indicator (just a small arrow)
    // Arrow points in the direction camera is looking (down -Z in world space)
    let arrow_length = size * 0.3;
    let arrow_end = rotate_point(-arrow_length, 0.0); // Point toward scene (down -Z)
    painter.line_segment(
        [egui::pos2(screen_x, screen_y), arrow_end],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)), // Soft red arrow
    );

    // Small arrow head
    let arrow_head_size = 6.0;
    let arrow_left = rotate_point(-arrow_length + arrow_head_size, -arrow_head_size * 0.4);
    let arrow_right = rotate_point(-arrow_length + arrow_head_size, arrow_head_size * 0.4);
    painter.line_segment([arrow_end, arrow_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)));
    painter.line_segment([arrow_end, arrow_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)));
}

/// Render clean 3D camera gizmo for perspective cameras
fn render_rotated_camera_3d_icon(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    size: f32,
    rotation_rad: f32,
    _camera_color: egui::Color32,
    _outline_color: egui::Color32,
    camera_component: &ecs::Camera,
) {
    // Calculate rotation
    let cos_r = rotation_rad.cos();
    let sin_r = rotation_rad.sin();
    
    // Helper function to rotate a point around the center
    let rotate_point = |x: f32, y: f32| -> egui::Pos2 {
        let rotated_x = x * cos_r - y * sin_r;
        let rotated_y = x * sin_r + y * cos_r;
        egui::pos2(screen_x + rotated_x, screen_y + rotated_y)
    };
    
    // Clean 3D camera gizmo
    let gizmo_width = size * 0.8;
    let gizmo_height = size * 0.5;
    
    // Draw main Game View preview rectangle
    render_clean_game_view_preview(painter, screen_x, screen_y, gizmo_width, gizmo_height, rotation_rad, camera_component);
    
    // Draw small lens indicator for perspective camera
    painter.circle_filled(
        egui::pos2(screen_x, screen_y),
        3.0,
        egui::Color32::from_rgb(100, 150, 255),
    );
    
    // Draw simple direction indicator
    let arrow_length = size * 0.3;
    let arrow_end = rotate_point(-arrow_length, 0.0); // Point toward scene
    painter.line_segment(
        [egui::pos2(screen_x, screen_y), arrow_end],
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)), // Soft red arrow
    );
    
    // Small arrow head
    let arrow_head_size = 6.0;
    let arrow_left = rotate_point(-arrow_length + arrow_head_size, -arrow_head_size * 0.4);
    let arrow_right = rotate_point(-arrow_length + arrow_head_size, arrow_head_size * 0.4);
    painter.line_segment([arrow_end, arrow_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)));
    painter.line_segment([arrow_end, arrow_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 150)));
}

/// Render a default camera gizmo when no camera component is found
fn render_default_camera_gizmo(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    _scene_view_mode: &SceneViewMode,
) {
    let size = 40.0;
    let color = egui::Color32::from_rgb(255, 100, 100); // Red for missing component
    
    // Draw a simple camera outline
    let body_rect = egui::Rect::from_center_size(
        egui::pos2(screen_x, screen_y),
        egui::vec2(size, size * 0.6),
    );
    painter.rect_stroke(body_rect, 2.0, egui::Stroke::new(2.0, color));
    
    // Draw X to indicate missing component
    let half_size = size * 0.3;
    painter.line_segment([
        egui::pos2(screen_x - half_size, screen_y - half_size),
        egui::pos2(screen_x + half_size, screen_y + half_size)
    ], egui::Stroke::new(2.0, color));
    painter.line_segment([
        egui::pos2(screen_x - half_size, screen_y + half_size),
        egui::pos2(screen_x + half_size, screen_y - half_size)
    ], egui::Stroke::new(2.0, color));
}

/// Render clean Game View preview inside camera gizmo
fn render_clean_game_view_preview(
    painter: &egui::Painter,
    screen_x: f32,
    screen_y: f32,
    gizmo_width: f32,
    gizmo_height: f32,
    rotation_rad: f32,
    camera_component: &ecs::Camera,
) {
    // Calculate rotation
    let cos_r = rotation_rad.cos();
    let sin_r = rotation_rad.sin();
    
    // Helper function to rotate a point around the center
    let rotate_point = |x: f32, y: f32| -> egui::Pos2 {
        let rotated_x = x * cos_r - y * sin_r;
        let rotated_y = x * sin_r + y * cos_r;
        egui::pos2(screen_x + rotated_x, screen_y + rotated_y)
    };
    
    // Use most of the gizmo area for Game View preview
    let preview_width = gizmo_width * 0.9;
    let preview_height = gizmo_height * 0.9;
    
    // Create preview rectangle (rotated)
    let preview_corners = [
        rotate_point(-preview_width / 2.0, -preview_height / 2.0), // Top-left
        rotate_point(preview_width / 2.0, -preview_height / 2.0),  // Top-right
        rotate_point(preview_width / 2.0, preview_height / 2.0),   // Bottom-right
        rotate_point(-preview_width / 2.0, preview_height / 2.0),  // Bottom-left
    ];
    
    // Draw Game View preview background (camera's background color)
    let bg_color = egui::Color32::from_rgba_unmultiplied(
        (camera_component.background_color[0] * 255.0) as u8,
        (camera_component.background_color[1] * 255.0) as u8,
        (camera_component.background_color[2] * 255.0) as u8,
        200, // Slightly transparent
    );
    
    painter.add(egui::Shape::convex_polygon(
        preview_corners.to_vec(),
        bg_color,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 100)), // Thin yellow border
    ));
    
    // Draw simple camera info in center
    let info_text = match camera_component.projection {
        ecs::CameraProjection::Orthographic => {
            format!("Ortho\n{:.1}", camera_component.orthographic_size)
        }
        ecs::CameraProjection::Perspective => {
            format!("Persp\n{:.0}°", camera_component.fov)
        }
    };
    
    painter.text(
        egui::pos2(screen_x, screen_y),
        egui::Align2::CENTER_CENTER,
        info_text,
        egui::FontId::proportional(9.0),
        egui::Color32::from_rgb(255, 255, 255),
    );
}



/// Render clean Unity-style camera frustum in 3D
/// - Perspective: Pyramid shape showing FOV from camera origin
/// - Orthographic: Rectangular box showing view volume
pub fn render_camera_frustum_3d(
    painter: &egui::Painter,
    camera_entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    viewport_rect: egui::Rect,
    camera_screen_pos: egui::Pos2,
) {
    let viewport_size = glam::Vec2::new(viewport_rect.width(), viewport_rect.height());

    // Get camera component and transform
    if let (Some(camera), Some(transform)) = (world.cameras.get(&camera_entity), world.transforms.get(&camera_entity)) {
        // Camera position in world space
        let cam_pos = glam::Vec3::new(transform.position[0], transform.position[1], transform.position[2]);

        // Use full 3D rotation for accurate frustum orientation
        let rot_x = transform.rotation[0].to_radians();
        let rot_y = transform.rotation[1].to_radians();
        let rot_z = transform.rotation[2].to_radians();

        // Match Render System's rotation order (YXZ)
        let rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, rot_y, rot_x, rot_z);

        // Calculate directional vectors from rotation
        let forward = rotation * glam::Vec3::Z; // +Z Forward
        let up = rotation * glam::Vec3::Y;      // +Y Up
        let right = rotation * glam::Vec3::X;   // +X Right

        // Standard aspect ratio for clean visualization
        let aspect = 16.0 / 9.0;

        // Unity-style frustum colors
        let frustum_color = egui::Color32::from_rgb(255, 255, 0); // Bright yellow like Unity
        let line_stroke = egui::Stroke::new(2.0, frustum_color); // Thicker lines

        // Use the centralized 3D projection system to ensure the frustum matches the 3D scene view
        // This correctly handles rotation, zoom, and perspective/orthographic modes of the Scene Camera
        let project = |point: glam::Vec3| -> Option<egui::Pos2> {
            projection_3d::world_to_screen_allow_behind(
                point,
                scene_camera,
                viewport_size
            ).map(|p| egui::pos2(p.x + viewport_rect.min.x, p.y + viewport_rect.min.y))
        };
        
        // Project camera origin to screen space for frustum lines
        let camera_origin_screen = project(cam_pos);
        
        // Debug: Draw camera origin and axis vectors (Unity-style debug)
        if let Some(origin_screen) = camera_origin_screen {
            painter.circle_filled(origin_screen, 8.0, egui::Color32::from_rgb(255, 0, 255)); // Magenta circle
            painter.text(
                egui::pos2(origin_screen.x + 10.0, origin_screen.y),
                egui::Align2::LEFT_CENTER,
                format!("Cam Origin ({:.1}, {:.1}, {:.1})", cam_pos.x, cam_pos.y, cam_pos.z),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(255, 0, 255),
            );
            
            // Debug: Draw camera axis vectors
            let axis_length = 3.0;
            if let Some(forward_end) = project(cam_pos + forward * axis_length) {
                painter.line_segment([origin_screen, forward_end], egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 100, 255))); // Blue forward
                painter.text(egui::pos2(forward_end.x + 5.0, forward_end.y), egui::Align2::LEFT_CENTER, "F", egui::FontId::proportional(12.0), egui::Color32::from_rgb(0, 100, 255));
            }
            if let Some(up_end) = project(cam_pos + up * axis_length) {
                painter.line_segment([origin_screen, up_end], egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 0))); // Green up
                painter.text(egui::pos2(up_end.x, up_end.y - 12.0), egui::Align2::CENTER_BOTTOM, "U", egui::FontId::proportional(12.0), egui::Color32::from_rgb(0, 255, 0));
            }
            if let Some(right_end) = project(cam_pos + right * axis_length) {
                painter.line_segment([origin_screen, right_end], egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0))); // Red right
                painter.text(egui::pos2(right_end.x + 5.0, right_end.y), egui::Align2::LEFT_CENTER, "R", egui::FontId::proportional(12.0), egui::Color32::from_rgb(255, 0, 0));
            }
        } else {
            // If projection fails, draw at fallback position
            painter.circle_filled(camera_screen_pos, 8.0, egui::Color32::from_rgb(255, 100, 100)); // Red circle
            painter.text(
                egui::pos2(camera_screen_pos.x + 10.0, camera_screen_pos.y),
                egui::Align2::LEFT_CENTER,
                format!("Cam Fallback ({:.1}, {:.1}, {:.1})", cam_pos.x, cam_pos.y, cam_pos.z),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(255, 100, 100),
            );
        }

        match camera.projection {
            ecs::CameraProjection::Perspective => {
                // PERSPECTIVE CAMERA: Draw pyramid frustum from camera origin
                // Using 3D world-space projection for accurate spatial representation

                // Unity-style frustum calculation
                let fov_y_rad = camera.fov.to_radians(); // Unity uses vertical FOV
                let near_distance = camera.near_clip.max(0.1);
                let far_distance = 10.0; // Distance to far plane in world units

                // Unity math: height first, then width = height * aspect
                let near_height = 2.0 * (fov_y_rad * 0.5).tan() * near_distance;
                let near_width = near_height * aspect;

                let far_height = 2.0 * (fov_y_rad * 0.5).tan() * far_distance;
                let far_width = far_height * aspect;

                // Unity-style frustum corners calculation
                let near_center = cam_pos + forward * near_distance;
                let far_center = cam_pos + forward * far_distance;
                
                let nh2 = near_height * 0.5;
                let nw2 = near_width * 0.5;
                let fh2 = far_height * 0.5;
                let fw2 = far_width * 0.5;
                
                // Near plane corners (Unity order: TL, TR, BL, BR)
                let near_tl = near_center + up * nh2 - right * nw2;
                let near_tr = near_center + up * nh2 + right * nw2;
                let near_bl = near_center - up * nh2 - right * nw2;
                let near_br = near_center - up * nh2 + right * nw2;

                // Far plane corners (Unity order: TL, TR, BL, BR)
                let far_tl = far_center + up * fh2 - right * fw2;
                let far_tr = far_center + up * fh2 + right * fw2;
                let far_bl = far_center - up * fh2 - right * fw2;
                let far_br = far_center - up * fh2 + right * fw2;

                // Project all corners to screen space
                let near_projections = [
                    project(near_tl),
                    project(near_tr),
                    project(near_br),
                    project(near_bl)
                ];

                let far_projections = [
                    project(far_tl),
                    project(far_tr),
                    project(far_br),
                    project(far_bl)
                ];

                // Draw near plane rectangle
                if near_projections.iter().all(|p| p.is_some()) {
                    let near_points: Vec<egui::Pos2> = near_projections.into_iter().map(|p| p.unwrap()).collect();
                    let near_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 150));
                    painter.line_segment([near_points[0], near_points[1]], near_stroke);
                    painter.line_segment([near_points[1], near_points[2]], near_stroke);
                    painter.line_segment([near_points[2], near_points[3]], near_stroke);
                    painter.line_segment([near_points[3], near_points[0]], near_stroke);
                }

                // Draw far plane rectangle
                if far_projections.iter().all(|p| p.is_some()) {
                    let far_points: Vec<egui::Pos2> = far_projections.into_iter().map(|p| p.unwrap()).collect();
                    let far_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0));
                    painter.line_segment([far_points[0], far_points[1]], far_stroke);
                    painter.line_segment([far_points[1], far_points[2]], far_stroke);
                    painter.line_segment([far_points[2], far_points[3]], far_stroke);
                    painter.line_segment([far_points[3], far_points[0]], far_stroke);

                    // Draw pyramid lines from camera origin to far plane corners
                    let origin_to_use = camera_origin_screen.unwrap_or(camera_screen_pos);
                    for far_point in far_points.iter() {
                        painter.line_segment([origin_to_use, *far_point], line_stroke);
                    }
                }

                // Draw connecting edges between near and far planes (frustum edges)
                if near_projections.iter().all(|p| p.is_some()) && far_projections.iter().all(|p| p.is_some()) {
                    let near_points: Vec<egui::Pos2> = near_projections.into_iter().map(|p| p.unwrap()).collect();
                    let far_points: Vec<egui::Pos2> = far_projections.into_iter().map(|p| p.unwrap()).collect();

                    for i in 0..4 {
                        painter.line_segment([near_points[i], far_points[i]], line_stroke);
                    }
                }
            }
            ecs::CameraProjection::Orthographic => {
                // ORTHOGRAPHIC CAMERA: Draw rectangular frustum box in world space
                // Using 3D projection for accurate spatial representation

                let near_distance = camera.near_clip.max(0.1);
                // For cameras at Z=0, ensure we have a reasonable far distance
                let far_distance = if cam_pos.z.abs() < 0.1 {
                    5.0 // Default distance when camera is at origin
                } else {
                    (cam_pos.z.abs() + 2.0).min(camera.far_clip)
                };

                // Calculate view volume dimensions based on orthographic size
                let view_height = camera.orthographic_size * 2.0;
                let view_width = view_height * aspect;

                // Unity-style orthographic frustum corners
                let near_center = cam_pos + forward * near_distance;
                let far_center = cam_pos + forward * far_distance;
                
                let vh2 = view_height * 0.5;
                let vw2 = view_width * 0.5;
                
                // Near plane corners (Unity order: TL, TR, BL, BR)
                let near_tl = near_center + up * vh2 - right * vw2;
                let near_tr = near_center + up * vh2 + right * vw2;
                let near_bl = near_center - up * vh2 - right * vw2;
                let near_br = near_center - up * vh2 + right * vw2;

                // Far plane corners (Unity order: TL, TR, BL, BR)
                let far_tl = far_center + up * vh2 - right * vw2;
                let far_tr = far_center + up * vh2 + right * vw2;
                let far_bl = far_center - up * vh2 - right * vw2;
                let far_br = far_center - up * vh2 + right * vw2;

                // Project all corners to screen space
                let near_corners = [project(near_tl), project(near_tr), project(near_br), project(near_bl)];
                let far_corners = [project(far_tl), project(far_tr), project(far_br), project(far_bl)];

                // Draw near plane (Unity-style)
                if near_corners.iter().all(|p| p.is_some()) {
                    let near_points: Vec<egui::Pos2> = near_corners.into_iter().map(|p| p.unwrap()).collect();
                    let near_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0)); // Bright yellow
                    painter.line_segment([near_points[0], near_points[1]], near_stroke);
                    painter.line_segment([near_points[1], near_points[2]], near_stroke);
                    painter.line_segment([near_points[2], near_points[3]], near_stroke);
                    painter.line_segment([near_points[3], near_points[0]], near_stroke);
                }

                // Draw far plane (Unity-style)
                if far_corners.iter().all(|p| p.is_some()) {
                    let far_points: Vec<egui::Pos2> = far_corners.into_iter().map(|p| p.unwrap()).collect();
                    let far_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0)); // Bright yellow
                    painter.line_segment([far_points[0], far_points[1]], far_stroke);
                    painter.line_segment([far_points[1], far_points[2]], far_stroke);
                    painter.line_segment([far_points[2], far_points[3]], far_stroke);
                    painter.line_segment([far_points[3], far_points[0]], far_stroke);
                }

                // Draw connecting edges (parallel lines between near and far planes)
                if near_corners.iter().all(|p| p.is_some()) && far_corners.iter().all(|p| p.is_some()) {
                    let near_points: Vec<egui::Pos2> = near_corners.into_iter().map(|p| p.unwrap()).collect();
                    let far_points: Vec<egui::Pos2> = far_corners.into_iter().map(|p| p.unwrap()).collect();

                    for i in 0..4 {
                        painter.line_segment([near_points[i], far_points[i]], line_stroke);
                    }
                }

                // Draw lines from camera origin to near plane corners (Unity-style)
                let origin_to_use = camera_origin_screen.unwrap_or(camera_screen_pos);
                if near_corners.iter().all(|p| p.is_some()) {
                    let near_points: Vec<egui::Pos2> = near_corners.into_iter().map(|p| p.unwrap()).collect();
                    for (i, near_point) in near_points.iter().enumerate() {
                        // Draw Unity-style yellow lines from camera origin to near plane corners
                        painter.line_segment([origin_to_use, *near_point], egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 255, 0)));
                        
                        // Debug: Draw small circles at near plane corners
                        painter.circle_filled(*near_point, 4.0, egui::Color32::from_rgb(100, 255, 100));
                        painter.text(
                            egui::pos2(near_point.x + 5.0, near_point.y),
                            egui::Align2::LEFT_CENTER,
                            format!("N{}", i),
                            egui::FontId::proportional(8.0),
                            egui::Color32::from_rgb(100, 255, 100),
                        );
                    }
                } else {
                    // Debug: Show which corners failed to project
                    painter.text(
                        egui::pos2(origin_to_use.x, origin_to_use.y - 20.0),
                        egui::Align2::CENTER_BOTTOM,
                        "Near corners projection failed",
                        egui::FontId::proportional(10.0),
                        egui::Color32::from_rgb(255, 100, 100),
                    );
                }
            }
        }
    }
}

/// Render camera viewport bounds (the yellow rectangle showing what the camera sees)
pub fn render_camera_viewport_bounds(
    painter: &egui::Painter,
    camera_entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
) {
    // Get camera component and transform
    if let (Some(camera), Some(transform)) = (world.cameras.get(&camera_entity), world.transforms.get(&camera_entity)) {
        // Get camera position in world space
        let cam_world_pos = glam::Vec3::new(transform.x(), transform.y(), 0.0);
        
        // Calculate viewport size in world units based on orthographic_size
        // orthographic_size is the half-height of the camera view
        let viewport_height = camera.orthographic_size * 2.0;
        
        // Calculate aspect ratio from viewport_rect
        // viewport_rect is [x, y, width, height] where width and height are normalized (0-1)
        let viewport_width_normalized = camera.viewport_rect[2];
        let viewport_height_normalized = camera.viewport_rect[3];
        
        // Calculate actual aspect ratio
        // For a typical game window, we assume 16:9 as base, but scale by viewport rect
        let base_aspect_ratio = 16.0 / 9.0;
        let aspect_ratio = if viewport_height_normalized > 0.0 {
            base_aspect_ratio * (viewport_width_normalized / viewport_height_normalized)
        } else {
            base_aspect_ratio
        };
        
        let viewport_width = viewport_height * aspect_ratio;
        
        // Convert camera world position to screen position
        let cam_screen_pos = scene_camera.world_to_screen(cam_world_pos);
        let cam_screen_x = center.x + cam_screen_pos.x;
        let cam_screen_y = center.y + cam_screen_pos.y;
        
        // Calculate viewport bounds in screen space
        let half_width = (viewport_width / 2.0) * scene_camera.zoom;
        let half_height = (viewport_height / 2.0) * scene_camera.zoom;
        
        // Draw viewport rectangle (yellow outline)
        let viewport_rect = egui::Rect::from_center_size(
            egui::pos2(cam_screen_x, cam_screen_y),
            egui::vec2(half_width * 2.0, half_height * 2.0),
        );
        
        // Draw yellow outline only (no fill)
        painter.rect_stroke(
            viewport_rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 0)),
        );
        
        // Draw corner markers (Unity style)
        let corner_size = 10.0;
        let corners = [
            (viewport_rect.min.x, viewport_rect.min.y), // Top-left
            (viewport_rect.max.x, viewport_rect.min.y), // Top-right
            (viewport_rect.min.x, viewport_rect.max.y), // Bottom-left
            (viewport_rect.max.x, viewport_rect.max.y), // Bottom-right
        ];
        
        for (x, y) in corners.iter() {
            // Horizontal line
            painter.line_segment(
                [
                    egui::pos2(x - corner_size, *y),
                    egui::pos2(x + corner_size, *y),
                ],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 0)),
            );
            // Vertical line
            painter.line_segment(
                [
                    egui::pos2(*x, y - corner_size),
                    egui::pos2(*x, y + corner_size),
                ],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 0)),
            );
        }
        
        // Draw aspect ratio label at the top
        let aspect_text = format!("{:.2}:1", aspect_ratio);
        painter.text(
            egui::pos2(cam_screen_x, viewport_rect.min.y - 15.0),
            egui::Align2::CENTER_BOTTOM,
            aspect_text,
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(255, 220, 0),
        );
    }
}
