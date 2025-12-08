//! Gizmo Rendering
//!
//! Functions for rendering various gizmos (scene gizmo, transform gizmo, colliders, etc).

use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
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
    };

    match current_tool {
        TransformTool::View => {
            // No gizmo
        }
        TransformTool::Move => {
            // X Axis (Red)
            let x_dir = (rotation_rad.cos(), rotation_rad.sin());
            let x_end = egui::pos2(
                screen_x + x_dir.0 * gizmo_size,
                screen_y + x_dir.1 * gizmo_size,
            );
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), x_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 0, 0)),
            );
            painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));
            painter.text(
                egui::pos2(x_end.x + 12.0, x_end.y),
                egui::Align2::LEFT_CENTER,
                "X",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(255, 0, 0),
            );

            // Y Axis (Green) - Perpendicular to X (Up in 2D usually means negative Y in screen space)
            // We rotate -90 degrees (PI/2) from X to get Y pointing "Up" relative to X
            let y_angle = rotation_rad - std::f32::consts::PI / 2.0;
            let y_dir = (y_angle.cos(), y_angle.sin());
            let y_end = egui::pos2(
                screen_x + y_dir.0 * gizmo_size,
                screen_y + y_dir.1 * gizmo_size,
            );
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));
            painter.text(
                egui::pos2(y_end.x, y_end.y - 12.0),
                egui::Align2::CENTER_BOTTOM,
                "Y",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(0, 255, 0),
            );

            // Z Axis (Blue) - Only in 3D mode
            if *scene_view_mode == SceneViewMode::Mode3D {
                // Z axis points perpendicular to X (opposite direction from Y)
                let z_angle = rotation_rad + std::f32::consts::PI / 2.0;
                let z_dir = (z_angle.cos(), z_angle.sin());
                let z_end = egui::pos2(
                    screen_x + z_dir.0 * gizmo_size,
                    screen_y + z_dir.1 * gizmo_size,
                );
                painter.line_segment(
                    [egui::pos2(screen_x, screen_y), z_end],
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 100, 255)),
                );
                painter.circle_filled(z_end, handle_size, egui::Color32::from_rgb(0, 100, 255));
                painter.text(
                    egui::pos2(z_end.x - 12.0, z_end.y),
                    egui::Align2::RIGHT_CENTER,
                    "Z",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(0, 100, 255),
                );
            }

            // Center handle (Yellow)
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
                let angle = (i as f32) * std::f32::consts::PI / 2.0 + rotation_rad;
                let dot_x = screen_x + radius * angle.cos();
                let dot_y = screen_y + radius * angle.sin();
                painter.circle_filled(
                    egui::pos2(dot_x, dot_y),
                    4.0,
                    egui::Color32::from_rgb(0, 150, 255),
                );
            }
        }
        TransformTool::Scale => {
            // X Axis (Red)
            let x_dir = (rotation_rad.cos(), rotation_rad.sin());
            let x_end = egui::pos2(
                screen_x + x_dir.0 * gizmo_size,
                screen_y + x_dir.1 * gizmo_size,
            );
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), x_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(255, 0, 0)),
            );
            painter.rect_filled(
                egui::Rect::from_center_size(x_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)),
                0.0,
                egui::Color32::from_rgb(255, 0, 0)
            );

            // Y Axis (Green)
            let y_angle = rotation_rad - std::f32::consts::PI / 2.0;
            let y_dir = (y_angle.cos(), y_angle.sin());
            let y_end = egui::pos2(
                screen_x + y_dir.0 * gizmo_size,
                screen_y + y_dir.1 * gizmo_size,
            );
            painter.line_segment(
                [egui::pos2(screen_x, screen_y), y_end],
                egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 255, 0)),
            );
            painter.rect_filled(
                egui::Rect::from_center_size(y_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)),
                0.0,
                egui::Color32::from_rgb(0, 255, 0)
            );

            // Z Axis (Blue) - Only in 3D mode
            if *scene_view_mode == SceneViewMode::Mode3D {
                let z_angle = rotation_rad + std::f32::consts::PI / 2.0;
                let z_dir = (z_angle.cos(), z_angle.sin());
                let z_end = egui::pos2(
                    screen_x + z_dir.0 * gizmo_size,
                    screen_y + z_dir.1 * gizmo_size,
                );
                painter.line_segment(
                    [egui::pos2(screen_x, screen_y), z_end],
                    egui::Stroke::new(4.0, egui::Color32::from_rgb(0, 100, 255)),
                );
                painter.rect_filled(
                    egui::Rect::from_center_size(z_end, egui::vec2(handle_size * 1.8, handle_size * 1.8)),
                    0.0,
                    egui::Color32::from_rgb(0, 100, 255)
                );
            }
            
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
        let screen_y = screen_y - offset_screen.y; // Flip Y
        
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

/// Render camera frustum in 3D (pyramid shape showing FOV)
pub fn render_camera_frustum_3d(
    painter: &egui::Painter,
    camera_entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    viewport_size: glam::Vec2,
) {
    // Get camera component and transform
    if let (Some(camera), Some(transform)) = (world.cameras.get(&camera_entity), world.transforms.get(&camera_entity)) {
        // Camera position in world space
        let cam_pos = glam::Vec3::new(transform.position[0], transform.position[1], transform.position[2]);
        
        // Calculate frustum based on FOV and aspect ratio
        let fov_rad = camera.fov.to_radians();
        let aspect = 16.0 / 9.0; // Default aspect ratio
        let near = camera.near_clip;
        let far = camera.far_clip.min(1000.0); // Limit far plane for visibility
        
        // Calculate frustum dimensions at near and far planes
        let near_height = 2.0 * near * (fov_rad / 2.0).tan();
        let near_width = near_height * aspect;
        let far_height = 2.0 * far * (fov_rad / 2.0).tan();
        let far_width = far_height * aspect;
        
        // Camera forward direction based on camera rotation
        // In Unity/game engines, camera typically looks along -Z in local space
        // Apply camera rotation to get world-space directions
        let rotation_y = transform.rotation[1].to_radians(); // Yaw
        let rotation_x = transform.rotation[0].to_radians(); // Pitch
        
        // Calculate forward vector (camera looks along -Z by default)
        let forward = glam::Vec3::new(
            rotation_y.sin() * rotation_x.cos(),
            -rotation_x.sin(),
            -rotation_y.cos() * rotation_x.cos(),
        );
        
        // Calculate right vector
        let right = glam::Vec3::new(
            rotation_y.cos(),
            0.0,
            rotation_y.sin(),
        );
        
        // Calculate up vector (cross product of right and forward)
        let up = right.cross(forward).normalize();
        
        // Calculate 8 frustum corners in world space
        // Near plane corners
        let near_center = cam_pos + forward * near;
        let near_tl = near_center + up * (near_height / 2.0) - right * (near_width / 2.0);
        let near_tr = near_center + up * (near_height / 2.0) + right * (near_width / 2.0);
        let near_bl = near_center - up * (near_height / 2.0) - right * (near_width / 2.0);
        let near_br = near_center - up * (near_height / 2.0) + right * (near_width / 2.0);
        
        // Far plane corners
        let far_center = cam_pos + forward * far;
        let far_tl = far_center + up * (far_height / 2.0) - right * (far_width / 2.0);
        let far_tr = far_center + up * (far_height / 2.0) + right * (far_width / 2.0);
        let far_bl = far_center - up * (far_height / 2.0) - right * (far_width / 2.0);
        let far_br = far_center - up * (far_height / 2.0) + right * (far_width / 2.0);
        
        // Helper function to project 3D point to screen
        let project = |point: glam::Vec3| -> Option<egui::Pos2> {
            projection_3d::world_to_screen(point, scene_camera, viewport_size)
                .map(|v| egui::pos2(v.x, v.y))
        };
        
        let frustum_color = egui::Color32::from_rgb(255, 220, 0); // Yellow
        let stroke = egui::Stroke::new(2.0, frustum_color);
        
        // Draw near plane rectangle
        if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
            project(near_tl), project(near_tr), project(near_br), project(near_bl)
        ) {
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment([p3, p4], stroke);
            painter.line_segment([p4, p1], stroke);
        }
        
        // Draw far plane rectangle with thicker stroke (viewport bounds)
        let far_stroke = egui::Stroke::new(4.0, frustum_color);
        if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
            project(far_tl), project(far_tr), project(far_br), project(far_bl)
        ) {
            // Draw the rectangle outline
            painter.line_segment([p1, p2], far_stroke);
            painter.line_segment([p2, p3], far_stroke);
            painter.line_segment([p3, p4], far_stroke);
            painter.line_segment([p4, p1], far_stroke);
            
            // Fill the far plane with semi-transparent yellow (viewport bounds)
            // Increased alpha for better visibility
            let fill_color = egui::Color32::from_rgba_premultiplied(255, 220, 0, 60);
            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3, p4],
                fill_color,
                egui::Stroke::NONE,
            ));
            
            // Draw corner markers for better visibility
            let corner_size = 8.0;
            painter.circle_filled(p1, corner_size, frustum_color);
            painter.circle_filled(p2, corner_size, frustum_color);
            painter.circle_filled(p3, corner_size, frustum_color);
            painter.circle_filled(p4, corner_size, frustum_color);
        }
        
        // Draw connecting lines from camera to far corners
        if let Some(cam_screen) = project(cam_pos) {
            if let Some(p) = project(far_tl) { painter.line_segment([cam_screen, p], stroke); }
            if let Some(p) = project(far_tr) { painter.line_segment([cam_screen, p], stroke); }
            if let Some(p) = project(far_bl) { painter.line_segment([cam_screen, p], stroke); }
            if let Some(p) = project(far_br) { painter.line_segment([cam_screen, p], stroke); }
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
        let cam_world_pos = glam::Vec2::new(transform.x(), transform.y());
        
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
