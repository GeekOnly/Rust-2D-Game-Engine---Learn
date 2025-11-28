//! 3D Scene View Rendering
//!
//! Handles rendering of the scene in 3D mode (meshes, billboards, grid, gizmos).

use ecs::{World, Entity, MeshType};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_collider_gizmo, render_velocity_gizmo};

/// Render the scene in 3D mode
pub fn render_scene_3d(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    projection_mode: &ProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // Collect and sort entities by Z position for proper depth rendering
    let mut entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .map(|(&e, t)| (e, t))
        .collect();
    
    // Sort by Z position (far to near) for painter's algorithm
    entities.sort_by(|a, b| a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal));
    
    // Separate entities into opaque and transparent
    let (opaque_entities, transparent_entities): (Vec<_>, Vec<_>) = entities.into_iter()
        .partition(|(entity, _)| {
            if let Some(sprite) = world.sprites.get(entity) {
                sprite.color[3] >= 1.0
            } else if let Some(mesh) = world.meshes.get(entity) {
                mesh.color[3] >= 1.0
            } else {
                true
            }
        });
    
    // Render opaque entities first
    for (entity, transform) in opaque_entities.iter() {
        render_entity_3d(
            painter, 
            *entity, 
            transform, 
            world, 
            scene_camera, 
            projection_mode, 
            center, 
            selected_entity, 
            show_colliders, 
            show_velocities, 
            hovered_entity, 
            response
        );
    }
    
    // Render transparent entities after
    for (entity, transform) in transparent_entities.iter() {
        render_entity_3d(
            painter, 
            *entity, 
            transform, 
            world, 
            scene_camera, 
            projection_mode, 
            center, 
            selected_entity, 
            show_colliders, 
            show_velocities, 
            hovered_entity, 
            response
        );
    }

    // Render selection outline on top
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let (screen_x, screen_y) = project_point_3d(transform, scene_camera, center);
            
            // Draw selection outline
            if let Some(sprite) = world.sprites.get(&sel_entity) {
                let scale = calculate_perspective_scale(transform, scene_camera);
                let size = egui::vec2(sprite.width * scale, sprite.height * scale);
                
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size + egui::vec2(4.0, 4.0)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            } else if world.meshes.contains_key(&sel_entity) {
                let scale_vec = glam::Vec3::from(transform.scale);
                let world_size = 2.0;
                let scale_factor = calculate_perspective_scale(transform, scene_camera);
                let base_size = world_size * scale_factor * scale_vec.x.max(scale_vec.y).max(scale_vec.z);
                let selection_size = base_size + 8.0;
                
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(selection_size, selection_size)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            }
            
            // Draw selected entity's collider gizmo on top
            if *show_colliders {
                render_collider_gizmo(painter, sel_entity, world, screen_x, screen_y, scene_camera, true);
            }
        }
    }
}

fn render_entity_3d(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    projection_mode: &ProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    let (screen_x, screen_y) = project_point_3d(transform, scene_camera, center);

    // Get entity bounds for click detection
    let entity_rect = if let Some(sprite) = world.sprites.get(&entity) {
        let scale = calculate_perspective_scale(transform, scene_camera);
        let size = egui::vec2(sprite.width * scale, sprite.height * scale);
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
    } else if world.meshes.contains_key(&entity) {
        let scale_vec = glam::Vec3::from(transform.scale);
        let world_size = 2.0;
        let scale_factor = calculate_perspective_scale(transform, scene_camera);
        let base_size = world_size * scale_factor * scale_vec.x.max(scale_vec.y).max(scale_vec.z);
        calculate_3d_cube_bounds(screen_x, screen_y, base_size, transform, scene_camera, projection_mode)
    } else {
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(10.0, 10.0))
    };

    // Check hover
    if let Some(hover_pos) = response.hover_pos() {
        if entity_rect.contains(hover_pos) {
            *hovered_entity = Some(entity);
        }
    }

    // Render Sprite
    if let Some(sprite) = world.sprites.get(&entity) {
        let scale = calculate_perspective_scale(transform, scene_camera);
        let size = egui::vec2(sprite.width * scale, sprite.height * scale);
        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );

        if sprite.billboard {
            // Billboard mode
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                color,
            );
            painter.rect_stroke(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 50)),
            );
        } else {
            // Rotated sprite in 3D (simplified as flat polygon)
            // For now, just render as billboard for simplicity in this refactor, 
            // or implement full 3D quad rendering if needed.
            // Let's stick to billboard behavior for now as per original code behavior for 3D mode.
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                color,
            );
        }
    } else if let Some(mesh) = world.meshes.get(&entity) {
        render_mesh_entity_3d(painter, entity, transform, mesh, screen_x, screen_y, scene_camera, projection_mode);
    } else {
         // Default placeholder
        let is_camera = world.names.get(&entity)
            .map(|name| name.contains("Camera") || name.contains("camera"))
            .unwrap_or(false);
        
        if is_camera {
            render_camera_gizmo(painter, screen_x, screen_y, scene_camera, &SceneViewMode::Mode3D);
        } else {
            painter.circle_filled(egui::pos2(screen_x, screen_y), 5.0 * scene_camera.zoom, egui::Color32::from_rgb(150, 150, 150));
        }
    }

    // Gizmos
    if *show_colliders && *selected_entity != Some(entity) {
        render_collider_gizmo(painter, entity, world, screen_x, screen_y, scene_camera, false);
    }
    
    if *show_velocities {
        render_velocity_gizmo(painter, entity, world, screen_x, screen_y);
    }
}

fn project_point_3d(transform: &ecs::Transform, scene_camera: &SceneCamera, center: egui::Pos2) -> (f32, f32) {
    let pos_3d = Point3D::new(
        transform.x() - scene_camera.position.x,
        transform.y(),
        transform.position[2] - scene_camera.position.y,
    );

    let yaw = scene_camera.rotation.to_radians();
    let pitch = scene_camera.pitch.to_radians();
    let rotated = pos_3d
        .rotate_y(-yaw)
        .rotate_x(pitch);

    let distance = 500.0;
    let perspective_z = rotated.z + distance;
    let scale = if perspective_z > 10.0 {
        (distance / perspective_z) * scene_camera.zoom
    } else {
        scene_camera.zoom
    };

    (
        center.x + rotated.x * scale,
        center.y + rotated.y * scale,
    )
}

fn calculate_perspective_scale(transform: &ecs::Transform, scene_camera: &SceneCamera) -> f32 {
    let pos_3d = Point3D::new(
        transform.x() - scene_camera.position.x,
        transform.y(),
        transform.position[2] - scene_camera.position.y,
    );

    let yaw = scene_camera.rotation.to_radians();
    let pitch = scene_camera.pitch.to_radians();
    let rotated = pos_3d.rotate_y(-yaw).rotate_x(pitch);

    let distance = 500.0;
    let perspective_z = rotated.z + distance;
    
    if perspective_z > 10.0 {
        (distance / perspective_z) * scene_camera.zoom
    } else {
        scene_camera.zoom
    }
}

fn render_mesh_entity_3d(
    painter: &egui::Painter,
    _entity: Entity,
    transform: &ecs::Transform,
    mesh: &ecs::Mesh,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    projection_mode: &ProjectionMode,
) {
    let color = egui::Color32::from_rgba_unmultiplied(
        (mesh.color[0] * 255.0) as u8,
        (mesh.color[1] * 255.0) as u8,
        (mesh.color[2] * 255.0) as u8,
        (mesh.color[3] * 255.0) as u8,
    );
    
    let scale = glam::Vec3::from(transform.scale);
    let world_size = 2.0;
    let scale_factor = calculate_perspective_scale(transform, scene_camera);
    let base_size = world_size * scale_factor * scale.x.max(scale.y).max(scale.z);
    
    match mesh.mesh_type {
        MeshType::Cube => {
            render_3d_cube(painter, screen_x, screen_y, base_size, transform, color, scene_camera, projection_mode);
        }
        MeshType::Sphere => {
            painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
            painter.circle_stroke(egui::pos2(screen_x, screen_y), base_size / 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
        MeshType::Cylinder => {
            let width = base_size;
            let height = base_size * 1.5;
            let ellipse_height = base_size * 0.3;
            
            let body_rect = egui::Rect::from_center_size(
                egui::pos2(screen_x, screen_y),
                egui::vec2(width, height),
            );
            painter.rect_filled(body_rect, 0.0, color);
            
            let top_rect = egui::Rect::from_center_size(
                egui::pos2(screen_x, screen_y - height/2.0),
                egui::vec2(width, ellipse_height),
            );
            painter.rect_filled(top_rect, width/2.0, color);
            
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
        }
        MeshType::Plane => {
            let size = base_size * 1.5;
            let rect = egui::Rect::from_center_size(
                egui::pos2(screen_x, screen_y),
                egui::vec2(size, size * 0.1),
            );
            painter.rect_filled(rect, 0.0, color);
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
        MeshType::Capsule => {
            let width = base_size * 0.6;
            let height = base_size * 1.5;
            let radius = width / 2.0;
            
            let body_rect = egui::Rect::from_center_size(
                egui::pos2(screen_x, screen_y),
                egui::vec2(width, height - width),
            );
            painter.rect_filled(body_rect, 0.0, color);
            
            painter.circle_filled(egui::pos2(screen_x, screen_y - (height - width)/2.0), radius, color);
            painter.circle_filled(egui::pos2(screen_x, screen_y + (height - width)/2.0), radius, color);
            
            painter.circle_stroke(egui::pos2(screen_x, screen_y - (height - width)/2.0), radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
            painter.circle_stroke(egui::pos2(screen_x, screen_y + (height - width)/2.0), radius, egui::Stroke::new(1.0, egui::Color32::BLACK));
        }
    }
}

// Reuse the cube rendering logic
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
    
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| {
            let obj_rotated = v.rotate(&transform.rotation);
            obj_rotated
                .rotate_y(-scene_camera.rotation.to_radians())
                .rotate_x(scene_camera.pitch.to_radians())
        })
        .collect();
    
    let projected: Vec<(f32, f32)> = rotated.iter()
        .map(|v| match projection_mode {
            ProjectionMode::Isometric => v.project_isometric(),
            ProjectionMode::Perspective => v.project_perspective(500.0, 300.0),
        })
        .collect();
    
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
    
    let rotated: Vec<Point3D> = vertices.iter()
        .map(|v| {
            let obj_rotated = v.rotate(&transform.rotation);
            obj_rotated
                .rotate_y(-scene_camera.rotation.to_radians())
                .rotate_x(scene_camera.pitch.to_radians())
        })
        .collect();
    
    let projected: Vec<(f32, f32)> = rotated.iter()
        .map(|v| match projection_mode {
            ProjectionMode::Isometric => v.project_isometric(),
            ProjectionMode::Perspective => v.project_perspective(500.0, 300.0),
        })
        .collect();
    
    let mut faces_with_depth: Vec<(Vec<usize>, f32, f32)> = vec![
        (vec![0, 1, 2, 3], 1.0, 0.0),
        (vec![5, 4, 7, 6], 0.6, 0.0),
        (vec![1, 0, 4, 5], 0.7, 0.0),
        (vec![3, 2, 6, 7], 0.9, 0.0),
        (vec![0, 3, 7, 4], 0.75, 0.0),
        (vec![2, 1, 5, 6], 0.85, 0.0),
    ];
    
    for face_data in &mut faces_with_depth {
        let avg_z: f32 = face_data.0.iter()
            .map(|&i| rotated[i].z)
            .sum::<f32>() / face_data.0.len() as f32;
        face_data.2 = avg_z;
    }
    
    faces_with_depth.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    
    for (face_indices, brightness, _depth) in faces_with_depth {
        if face_indices.len() >= 3 {
            let v0 = &rotated[face_indices[0]];
            let v1 = &rotated[face_indices[1]];
            let v2 = &rotated[face_indices[2]];
            
            let edge1 = (v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
            let edge2 = (v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
            
            let normal = (
                edge1.1 * edge2.2 - edge1.2 * edge2.1,
                edge1.2 * edge2.0 - edge1.0 * edge2.2,
                edge1.0 * edge2.1 - edge1.1 * edge2.0,
            );
            
            let view_dir = (0.0, 0.0, -1.0);
            let dot = normal.0 * view_dir.0 + normal.1 * view_dir.1 + normal.2 * view_dir.2;
            
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
