//! 2D Scene View Rendering
//!
//! Handles rendering of the scene in 2D mode (sprites, grid, gizmos).

use ecs::{World, Entity, MeshType};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_collider_gizmo, render_velocity_gizmo};

/// Render the scene in 2D mode
pub fn render_scene_2d(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // In 2D, we just iterate through all entities. 
    // Z-ordering is usually determined by entity order or a specific Z-index component (if we had one).
    // For now, we'll just use the order in the HashMap (arbitrary) or sorted by ID.
    // To be consistent, let's sort by ID for stability.
    let mut entities: Vec<Entity> = world.transforms.keys().cloned().collect();
    entities.sort();

    for entity in entities {
        if let Some(transform) = world.transforms.get(&entity) {
            render_entity_2d(
                painter,
                entity,
                transform,
                world,
                scene_camera,
                center,
                selected_entity,
                show_colliders,
                show_velocities,
                hovered_entity,
                response,
            );
        }
    }

    // Render selection outline on top
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;

            // Draw selection outline
            if let Some(sprite) = world.sprites.get(&sel_entity) {
                let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
                painter.rect_stroke(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size + egui::vec2(4.0, 4.0)),
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                );
            } else if world.meshes.contains_key(&sel_entity) {
                let scale = glam::Vec3::from(transform.scale);
                let world_size = 2.0;
                let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
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

/// Render transform gizmo for selected entity in 2D
pub fn render_transform_gizmo_2d(
    painter: &egui::Painter,
    entity: Entity,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    current_tool: &super::super::super::TransformTool,
    transform_space: &super::super::types::TransformSpace,
) {
    if let Some(transform) = world.transforms.get(&entity) {
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        let screen_x = center.x + screen_pos.x;
        let screen_y = center.y + screen_pos.y;
        
        super::gizmos::render_transform_gizmo(
            painter,
            screen_x,
            screen_y,
            current_tool,
            scene_camera,
            &super::super::types::SceneViewMode::Mode2D,
            transform_space,
            transform,
        );
    }
}

fn render_entity_2d(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // 2D Projection
    let world_pos = glam::Vec2::new(transform.x(), transform.y());
    let screen_pos = scene_camera.world_to_screen(world_pos);
    let screen_x = center.x + screen_pos.x;
    let screen_y = center.y + screen_pos.y;

    // Get entity bounds for click detection
    let entity_rect = if let Some(sprite) = world.sprites.get(&entity) {
        let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
    } else if world.meshes.contains_key(&entity) {
        let scale = glam::Vec3::from(transform.scale);
        let world_size = 2.0;
        let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(base_size, base_size))
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
        let size = egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom);
        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );

        let rotation_rad = transform.rotation[2].to_radians();
        
        if rotation_rad.abs() < 0.01 {
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                color,
            );
        } else {
            // Rotated sprite
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
            
            painter.add(egui::Shape::convex_polygon(
                corners.to_vec(),
                color,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
            ));
        }
    } else if let Some(mesh) = world.meshes.get(&entity) {
        // Render Mesh (Simplified for 2D)
        let color = egui::Color32::from_rgba_unmultiplied(
            (mesh.color[0] * 255.0) as u8,
            (mesh.color[1] * 255.0) as u8,
            (mesh.color[2] * 255.0) as u8,
            (mesh.color[3] * 255.0) as u8,
        );
        
        let scale = glam::Vec3::from(transform.scale);
        let world_size = 2.0;
        let base_size = world_size * scene_camera.zoom * scale.x.max(scale.y).max(scale.z);

        match mesh.mesh_type {
            MeshType::Sphere | MeshType::Cylinder => {
                 painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
                 painter.circle_stroke(egui::pos2(screen_x, screen_y), base_size / 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            },
            _ => {
                let rect = egui::Rect::from_center_size(
                    egui::pos2(screen_x, screen_y),
                    egui::vec2(base_size, base_size),
                );
                painter.rect_filled(rect, 2.0, color);
                painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
    } else {
        // Default placeholder - only for non-camera entities
        // Camera entities should not be rendered in the scene view
        let is_camera = world.names.get(&entity)
            .map(|name| name.contains("Camera") || name.contains("camera"))
            .unwrap_or(false);
        
        if !is_camera {
            // Only render placeholder for non-camera entities
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
