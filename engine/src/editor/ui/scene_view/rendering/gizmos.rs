//! Gizmo Rendering
//!
//! Functions for rendering various gizmos (scene gizmo, transform gizmo, colliders, etc).

use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
use super::super::types::*;

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
) {
    let gizmo_size = 60.0; // Increased for easier clicking
    let handle_size = 10.0; // Increased for easier clicking
    
    // Get rotation angle based on space mode
    let rotation_rad = match transform_space {
        TransformSpace::Local => {
            // Local space: rotate gizmo with object
            if *scene_view_mode == SceneViewMode::Mode3D {
                // 3D: combine object rotation with camera rotation
                scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
            } else {
                // 2D: only object rotation (Z axis)
                transform.rotation[2].to_radians()
            }
        }
        TransformSpace::World => {
            // World space: gizmo aligned with world axes
            if *scene_view_mode == SceneViewMode::Mode3D {
                // In 3D, still need camera rotation for proper axis display
                scene_camera.get_rotation_radians()
            } else {
                // In 2D, no rotation
                0.0
            }
        }
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));
            
            // Label
            painter.text(
                egui::pos2(y_end.x, y_end.y - 12.0),
                egui::Align2::CENTER_BOTTOM,
                "Y",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(0, 255, 0),
            );
            
            // Z axis (Blue) - perpendicular to X in 3D mode
            if *scene_view_mode == SceneViewMode::Mode3D {
                let z_dir = glam::Vec2::new(rotation_rad.sin(), rotation_rad.cos()); // Perpendicular to X
                let z_end = egui::pos2(
                    screen_x + z_dir.x * gizmo_size, 
                    screen_y + z_dir.y * gizmo_size
                );
                painter.line_segment(
                    [egui::pos2(screen_x, screen_y), z_end],
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 0, 255)),
                );
                painter.circle_filled(z_end, handle_size, egui::Color32::from_rgb(0, 0, 255));
                
                // Label
                painter.text(
                    egui::pos2(z_end.x + 12.0, z_end.y),
                    egui::Align2::LEFT_CENTER,
                    "Z",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(0, 0, 255),
                );
            }

            // Center handle for free movement (Yellow)
            painter.circle_filled(egui::pos2(screen_x, screen_y), handle_size * 1.2, egui::Color32::from_rgb(255, 255, 0));
        }
        TransformTool::Rotate => {
            let radius = gizmo_size * 0.8;
            
            // Draw rotation circle with thicker stroke for easier clicking
            painter.circle_stroke(
                egui::pos2(screen_x, screen_y),
                radius,
                egui::Stroke::new(5.0, egui::Color32::from_rgb(0, 150, 255)),
            );
            
            // Draw center dot
            painter.circle_filled(
                egui::pos2(screen_x, screen_y),
                3.0,
                egui::Color32::from_rgb(0, 150, 255),
            );
            
            // Draw rotation indicators (4 dots on circle)
            for i in 0..4 {
                let angle = (i as f32) * std::f32::consts::PI / 2.0;
                let dot_x = screen_x + radius * angle.cos();
                let dot_y = screen_y + radius * angle.sin();
                painter.circle_filled(
                    egui::pos2(dot_x, dot_y),
                    4.0,
                    egui::Color32::from_rgb(0, 150, 255),
                );
            }
        }
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.rect_filled(
                egui::Rect::from_center_size(y_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)),
                0.0,
                egui::Color32::from_rgb(0, 255, 0)
            );
            
            // Center handle for uniform scale (White)
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(handle_size * 2.2, handle_size * 2.2)
                ),
                0.0,
                egui::Color32::from_rgb(255, 255, 255)
            );
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
) {
    if let Some(collider) = world.colliders.get(&entity) {
        let size = egui::vec2(collider.width * scene_camera.zoom, collider.height * scene_camera.zoom);
        
        // Get entity rotation if available
        let rotation_rad = world.transforms.get(&entity)
            .map(|t| t.rotation[2].to_radians())
            .unwrap_or(0.0);
        
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
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
) {
    let zoom = scene_camera.zoom;
    
    // Camera gizmo size (scales with zoom)
    let base_size = 40.0;
    let size = base_size * (zoom / 50.0).clamp(0.5, 2.0);
    
    // Unity-style camera colors
    let camera_color = egui::Color32::from_rgb(255, 220, 0); // Yellow
    let outline_color = egui::Color32::from_rgb(200, 170, 0); // Darker yellow
    
    if *scene_view_mode == SceneViewMode::Mode2D {
        // 2D mode: Draw trapezoid (camera frustum shape)
        // Front (small) rectangle
        let front_width = size * 0.4;
        let front_height = size * 0.3;
        
        // Back (large) rectangle
        let back_width = size * 0.8;
        let back_height = size * 0.6;
        let back_offset = size * 0.5;
        
        // Define trapezoid points (camera pointing right)
        let points = vec![
            // Front face (left side - small)
            egui::pos2(screen_x - front_width / 2.0, screen_y - front_height / 2.0),
            egui::pos2(screen_x - front_width / 2.0, screen_y + front_height / 2.0),
            // Back face (right side - large)
            egui::pos2(screen_x + back_offset, screen_y + back_height / 2.0),
            egui::pos2(screen_x + back_offset, screen_y - back_height / 2.0),
        ];
        
        // Fill trapezoid
        painter.add(egui::Shape::convex_polygon(
            points.clone(),
            egui::Color32::from_rgba_premultiplied(255, 220, 0, 100), // Semi-transparent yellow
            egui::Stroke::new(2.0, camera_color),
        ));
        
        // Draw lens (small rectangle at front)
        let lens_rect = egui::Rect::from_center_size(
            egui::pos2(screen_x - front_width / 2.0, screen_y),
            egui::vec2(3.0, front_height * 0.8),
        );
        painter.rect_filled(lens_rect, 0.0, egui::Color32::from_rgb(100, 100, 150));
        
        // Draw center dot
        painter.circle_filled(
            egui::pos2(screen_x, screen_y),
            3.0,
            camera_color,
        );
        
    } else {
        // 3D mode: Draw simplified camera icon
        // Body (rectangle)
        let body_width = size * 0.6;
        let body_height = size * 0.4;
        let body_rect = egui::Rect::from_center_size(
            egui::pos2(screen_x, screen_y),
            egui::vec2(body_width, body_height),
        );
        painter.rect_filled(body_rect, 2.0, egui::Color32::from_rgba_premultiplied(255, 220, 0, 150));
        painter.rect_stroke(body_rect, 2.0, egui::Stroke::new(2.0, camera_color));
        
        // Lens (circle)
        let lens_radius = size * 0.15;
        painter.circle_filled(
            egui::pos2(screen_x, screen_y),
            lens_radius,
            egui::Color32::from_rgb(100, 100, 150),
        );
        painter.circle_stroke(
            egui::pos2(screen_x, screen_y),
            lens_radius,
            egui::Stroke::new(1.5, outline_color),
        );
        
        // Viewfinder lines
        let line_len = size * 0.15;
        painter.line_segment(
            [
                egui::pos2(screen_x - line_len, screen_y),
                egui::pos2(screen_x + line_len, screen_y),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150)),
        );
        painter.line_segment(
            [
                egui::pos2(screen_x, screen_y - line_len),
                egui::pos2(screen_x, screen_y + line_len),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150)),
        );
    }
}
