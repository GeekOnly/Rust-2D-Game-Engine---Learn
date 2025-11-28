//! Grid Rendering
//!
//! 2D and 3D grid rendering functions for the scene view.

use egui;
use crate::editor::{SceneCamera, SceneGrid};

/// Render 2D grid
pub fn render_grid_2d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    let grid_size = scene_grid.size * scene_camera.zoom;
    let grid_color = egui::Color32::from_rgba_premultiplied(
        (scene_grid.color[0] * 255.0) as u8,
        (scene_grid.color[1] * 255.0) as u8,
        (scene_grid.color[2] * 255.0) as u8,
        (scene_grid.color[3] * 255.0) as u8,
    );

    // Calculate grid offset based on camera position
    // The grid should move opposite to camera movement
    let center = rect.center();
    let offset_x = (-scene_camera.position.x * scene_camera.zoom) % grid_size;
    let offset_y = (-scene_camera.position.y * scene_camera.zoom) % grid_size;

    // Vertical lines
    let start_x = ((rect.min.x - center.x - offset_x) / grid_size).floor() * grid_size;
    let mut x = start_x;
    while x < rect.max.x - center.x + grid_size {
        let screen_x = center.x + x + offset_x;
        if screen_x >= rect.min.x && screen_x <= rect.max.x {
            painter.line_segment(
                [egui::pos2(screen_x, rect.min.y), egui::pos2(screen_x, rect.max.y)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        x += grid_size;
    }

    // Horizontal lines
    let start_y = ((rect.min.y - center.y - offset_y) / grid_size).floor() * grid_size;
    let mut y = start_y;
    while y < rect.max.y - center.y + grid_size {
        let screen_y = center.y + y + offset_y;
        if screen_y >= rect.min.y && screen_y <= rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, screen_y), egui::pos2(rect.max.x, screen_y)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        y += grid_size;
    }
}

/// Render 3D grid
pub fn render_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    let center = rect.center();
    let grid_world_size = scene_grid.size;

    let grid_color = egui::Color32::from_rgba_premultiplied(100, 100, 100, 100);
    let x_axis_color = egui::Color32::from_rgba_premultiplied(220, 60, 60, 200);
    let z_axis_color = egui::Color32::from_rgba_premultiplied(60, 120, 220, 200);

    let yaw = scene_camera.rotation.to_radians();
    let pitch = scene_camera.pitch.to_radians();
    let zoom = scene_camera.zoom;

    let grid_range = 25;
    let fade_distance = 20.0 * grid_world_size;

    let project_3d = |x: f32, z: f32| -> egui::Pos2 {
        let world_x = x - scene_camera.position.x;
        let world_z = z - scene_camera.position.y;
        let y = 0.0;

        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        let rotated_x = world_x * cos_yaw - world_z * sin_yaw;
        let rotated_z = world_x * sin_yaw + world_z * cos_yaw;

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let rotated_y = y * cos_pitch - rotated_z * sin_pitch;
        let final_z = y * sin_pitch + rotated_z * cos_pitch;

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
    
    let calc_alpha = |x: f32, z: f32| -> u8 {
        let dist = (x * x + z * z).sqrt();
        if dist > fade_distance {
            let fade = 1.0 - ((dist - fade_distance) / (fade_distance * 0.5)).min(1.0);
            (fade * 100.0) as u8
        } else {
            100
        }
    };

    // Draw grid lines along Z axis
    for i in -grid_range..=grid_range {
        let x = i as f32 * grid_world_size;
        let is_x_axis = i == 0;

        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let z = j as f32 * grid_world_size;
            points.push(project_3d(x, z));
        }

        for j in 0..points.len() - 1 {
            let z1 = ((j as i32) - grid_range) as f32 * grid_world_size;
            let alpha = calc_alpha(x, z1);

            if alpha > 5 {
                let color = if is_x_axis {
                    egui::Color32::from_rgba_premultiplied(
                        x_axis_color.r(),
                        x_axis_color.g(),
                        x_axis_color.b(),
                        alpha.max(150),
                    )
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        grid_color.r(),
                        grid_color.g(),
                        grid_color.b(),
                        alpha,
                    )
                };

                let width = if is_x_axis { 2.5 } else { 1.0 };
                painter.line_segment(
                    [points[j], points[j + 1]],
                    egui::Stroke::new(width, color),
                );
            }
        }
    }

    // Draw grid lines along X axis
    for i in -grid_range..=grid_range {
        let z = i as f32 * grid_world_size;
        let is_z_axis = i == 0;

        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let x = j as f32 * grid_world_size;
            points.push(project_3d(x, z));
        }

        for j in 0..points.len() - 1 {
            let x1 = ((j as i32) - grid_range) as f32 * grid_world_size;
            let alpha = calc_alpha(x1, z);

            if alpha > 5 {
                let color = if is_z_axis {
                    egui::Color32::from_rgba_premultiplied(
                        z_axis_color.r(),
                        z_axis_color.g(),
                        z_axis_color.b(),
                        alpha.max(150),
                    )
                } else {
                    egui::Color32::from_rgba_premultiplied(
                        grid_color.r(),
                        grid_color.g(),
                        grid_color.b(),
                        alpha,
                    )
                };

                let width = if is_z_axis { 2.5 } else { 1.0 };
                painter.line_segment(
                    [points[j], points[j + 1]],
                    egui::Stroke::new(width, color),
                );
            }
        }
    }
}
