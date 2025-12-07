//! Grid Rendering
//!
//! 2D and 3D grid rendering functions for the scene view.

use egui;
use glam::Mat4;
use crate::editor::{SceneCamera, SceneGrid};
use crate::editor::grid::{InfiniteGrid, CameraState};

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
    // Y axis is inverted (world Y up = screen Y down)
    let center = rect.center();
    let offset_x = (-scene_camera.position.x * scene_camera.zoom) % grid_size;
    let offset_y = (scene_camera.position.y * scene_camera.zoom) % grid_size;

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

/// Render 3D grid using InfiniteGrid system
pub fn render_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    // Use the old grid rendering for now (will be replaced with InfiniteGrid)
    render_grid_3d_legacy(painter, rect, scene_camera, scene_grid);
}

/// Render 3D grid using enhanced InfiniteGrid system
/// OPTIMIZED: Uses cached geometry, aggressive culling, and efficient projection
pub fn render_infinite_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    infinite_grid: &mut InfiniteGrid,
) {
    // For now, use the legacy grid rendering which is known to work
    // TODO: Debug and fix the infinite grid system
    let scene_grid = SceneGrid::new();
    render_grid_3d_legacy(painter, rect, scene_camera, &scene_grid);
    
    /* DISABLED: Infinite grid system needs debugging
    let center = rect.center();
    let viewport_size = glam::Vec2::new(rect.width(), rect.height());
    
    // Create camera state for grid generation
    let camera_state = CameraState {
        position: scene_camera.position,
        rotation: scene_camera.rotation,
        pitch: scene_camera.pitch,
        zoom: scene_camera.zoom,
    };
    
    // Generate grid geometry (uses cache when possible)
    let geometry = infinite_grid.generate_geometry(&camera_state, viewport_size);
    
    // Get view and projection matrices
    let view_matrix = scene_camera.get_view_matrix();
    let aspect = rect.width() / rect.height();
    let projection_matrix = scene_camera.get_projection_matrix(aspect, crate::editor::camera::ProjectionMode::Perspective);
    let view_proj = projection_matrix * view_matrix;
    
    // OPTIMIZED: Project and render each line with spatial culling
    for line in &geometry.lines {
        // Project start and end points
        let start_screen = project_point_to_screen(line.start, &view_proj, center, viewport_size);
        let end_screen = project_point_to_screen(line.end, &view_proj, center, viewport_size);
        
        // Skip lines that are behind the camera or off-screen
        if let (Some(start), Some(end)) = (start_screen, end_screen) {
            // OPTIMIZED: Tighter bounds checking for better culling
            let margin = 50.0; // Reduced margin for more aggressive culling
            let in_bounds = (start.x >= rect.min.x - margin && start.x <= rect.max.x + margin &&
                            start.y >= rect.min.y - margin && start.y <= rect.max.y + margin) ||
                           (end.x >= rect.min.x - margin && end.x <= rect.max.x + margin &&
                            end.y >= rect.min.y - margin && end.y <= rect.max.y + margin);
            
            if in_bounds {
                let color = egui::Color32::from_rgba_premultiplied(
                    (line.color[0] * 255.0) as u8,
                    (line.color[1] * 255.0) as u8,
                    (line.color[2] * 255.0) as u8,
                    (line.color[3] * 255.0) as u8,
                );
                
                painter.line_segment(
                    [start, end],
                    egui::Stroke::new(line.width, color),
                );
            }
        }
    }
    */
}

/// Project a 3D point to screen space
fn project_point_to_screen(
    point: glam::Vec3,
    view_proj: &Mat4,
    center: egui::Pos2,
    viewport_size: glam::Vec2,
) -> Option<egui::Pos2> {
    // Transform point to clip space
    let clip_space = *view_proj * glam::Vec4::new(point.x, point.y, point.z, 1.0);
    
    // Check if point is behind camera
    if clip_space.w <= 0.0 {
        return None;
    }
    
    // Perspective divide
    let ndc = glam::Vec3::new(
        clip_space.x / clip_space.w,
        clip_space.y / clip_space.w,
        clip_space.z / clip_space.w,
    );
    
    // Check if point is within NDC bounds (with some tolerance)
    if ndc.x < -2.0 || ndc.x > 2.0 || ndc.y < -2.0 || ndc.y > 2.0 {
        return None;
    }
    
    // Convert NDC to screen space
    let screen_x = center.x + (ndc.x * viewport_size.x * 0.5);
    let screen_y = center.y - (ndc.y * viewport_size.y * 0.5); // Flip Y
    
    Some(egui::pos2(screen_x, screen_y))
}

/// Legacy 3D grid rendering (fallback)
fn render_grid_3d_legacy(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    let center = rect.center();
    let grid_world_size = scene_grid.size;

    // Unity-like subtle grid colors
    let grid_color = egui::Color32::from_rgba_premultiplied(64, 64, 64, 76);  // Subtle gray
    let x_axis_color = egui::Color32::from_rgba_premultiplied(217, 64, 64, 230);  // Bright red
    let z_axis_color = egui::Color32::from_rgba_premultiplied(64, 115, 217, 230);  // Bright blue

    let yaw = scene_camera.rotation.to_radians();
    let pitch = scene_camera.pitch.to_radians();
    let zoom = scene_camera.zoom;

    let grid_range = 50;  // Wider grid like Unity
    let fade_distance = 40.0 * grid_world_size;  // Longer fade distance

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

                let width = if is_x_axis { 2.0 } else { 0.8 };  // Thinner lines like Unity
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

                let width = if is_z_axis { 2.0 } else { 0.8 };  // Thinner lines like Unity
                painter.line_segment(
                    [points[j], points[j + 1]],
                    egui::Stroke::new(width, color),
                );
            }
        }
    }
}
