//! Game Runtime Renderer
//!
//! Renders the game scene using Camera components from the ECS.
//! This is separate from the editor's scene view.

use ecs::{World, Entity, Camera, CameraProjection};
use egui;

/// Render the game view using the main camera
pub fn render_game_view(
    ui: &mut egui::Ui,
    world: &World,
) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);

    // Find the main camera (first active camera with lowest depth)
    let main_camera = find_main_camera(world);

    if let Some((camera_entity, camera, transform)) = main_camera {
        // Clear background
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                (camera.background_color[0] * 255.0) as u8,
                (camera.background_color[1] * 255.0) as u8,
                (camera.background_color[2] * 255.0) as u8,
                (camera.background_color[3] * 255.0) as u8,
            ),
        );

        // Render all entities
        render_entities(ui, world, camera, transform, rect);
    } else {
        // No camera found - show default view
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(30, 30, 35),
        );

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Camera Found\n\nAdd a Camera component to an entity",
            egui::FontId::proportional(16.0),
            egui::Color32::from_rgb(150, 150, 150),
        );
    }
}

/// Find the main camera (lowest depth, active)
fn find_main_camera(world: &World) -> Option<(Entity, &Camera, &ecs::Transform)> {
    let mut cameras: Vec<_> = world.cameras.iter()
        .filter_map(|(entity, camera)| {
            // Check if entity is active
            if world.active.get(entity).copied().unwrap_or(true) {
                world.transforms.get(entity).map(|transform| (*entity, camera, transform))
            } else {
                None
            }
        })
        .collect();

    // Sort by depth (lowest first)
    cameras.sort_by_key(|(_, camera, _)| camera.depth);

    cameras.into_iter().next()
}

/// Render all entities visible to the camera
fn render_entities(
    ui: &mut egui::Ui,
    world: &World,
    camera: &Camera,
    camera_transform: &ecs::Transform,
    rect: egui::Rect,
) {
    let painter = ui.painter_at(rect);
    let center = rect.center();

    // Get camera position
    let cam_pos = camera_transform.position;

    // Render based on projection mode
    match camera.projection {
        CameraProjection::Orthographic => {
            render_orthographic(world, &painter, camera, cam_pos, center);
        }
        CameraProjection::Perspective => {
            render_perspective(world, &painter, camera, cam_pos, center);
        }
    }
}

/// Render in orthographic mode (2D)
fn render_orthographic(
    world: &World,
    painter: &egui::Painter,
    camera: &Camera,
    cam_pos: [f32; 3],
    center: egui::Pos2,
) {
    // Calculate zoom from orthographic size
    let zoom = 100.0 / camera.orthographic_size;

    // Render all entities
    for (entity, transform) in &world.transforms {
        // Skip if not active
        if !world.active.get(entity).copied().unwrap_or(true) {
            continue;
        }

        // Calculate screen position (simple orthographic projection)
        let world_x = transform.position[0] - cam_pos[0];
        let world_y = transform.position[1] - cam_pos[1];

        let screen_x = center.x + world_x * zoom;
        let screen_y = center.y - world_y * zoom; // Flip Y axis

        // Render sprite if exists
        if let Some(sprite) = world.sprites.get(entity) {
            let size = egui::vec2(sprite.width * zoom, sprite.height * zoom);
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
        }

        // Render mesh if exists (simple placeholder for now)
        if let Some(mesh) = world.meshes.get(entity) {
            let size = 50.0 * zoom;
            let color = egui::Color32::from_rgba_unmultiplied(
                (mesh.color[0] * 255.0) as u8,
                (mesh.color[1] * 255.0) as u8,
                (mesh.color[2] * 255.0) as u8,
                (mesh.color[3] * 255.0) as u8,
            );

            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(size, size)),
                2.0,
                color,
            );
        }
    }
}

/// Render in perspective mode (3D)
fn render_perspective(
    world: &World,
    painter: &egui::Painter,
    camera: &Camera,
    cam_pos: [f32; 3],
    center: egui::Pos2,
) {
    // Simple 3D projection
    let fov_scale = 500.0 / camera.fov;

    // TODO: Implement proper 3D perspective projection
    // For now, just use orthographic as fallback
    render_orthographic(world, painter, camera, cam_pos, center);
}
