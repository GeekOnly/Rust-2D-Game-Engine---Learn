//! 3D Scene View Rendering
//!
//! Handles rendering of the scene in 3D mode (meshes, billboards, grid, gizmos).

use ecs::{World, Entity, MeshType};
use egui;
use glam::Vec2;
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_collider_gizmo, render_velocity_gizmo};
use super::sprite_3d::Sprite3DRenderer;
use super::tilemap_3d::Tilemap3DRenderer;
use super::render_queue::{RenderQueue, RenderObject, GizmoData, GizmoType};

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
    _show_debug_lines: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
    texture_manager: &mut crate::texture_manager::TextureManager,
    ctx: &egui::Context,
) {
    // Create render queue for proper depth sorting
    let mut render_queue = RenderQueue::new();
    
    // Create sprite and tilemap renderers
    let mut sprite_renderer = Sprite3DRenderer::new();
    let mut tilemap_renderer = Tilemap3DRenderer::new();
    
    // Collect sprites from world
    let mut sprites = sprite_renderer.collect_sprites(world);
    
    // Depth sort sprites
    sprite_renderer.depth_sort(&mut sprites, scene_camera);
    
    // Add sprites to render queue
    for sprite in sprites {
        render_queue.push(RenderObject::Sprite(sprite));
    }
    
    // Collect tilemaps from world
    let mut tilemap_layers = tilemap_renderer.collect_tilemaps(world);
    
    // Depth sort tilemap layers
    tilemap_renderer.depth_sort_layers(&mut tilemap_layers);
    
    // Add tilemaps to render queue
    for layer in tilemap_layers {
        render_queue.push(RenderObject::Tilemap(layer));
    }
    
    // Collect mesh entities (not handled by sprite/tilemap renderers)
    let mut mesh_entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .filter(|(entity, _)| {
            world.meshes.contains_key(entity) && !world.sprites.contains_key(entity)
        })
        .map(|(&e, t)| (e, t))
        .collect();
    
    // Sort mesh entities by Z position
    mesh_entities.sort_by(|a, b| {
        b.1.position[2].partial_cmp(&a.1.position[2]).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Sort render queue by depth
    render_queue.sort_by_depth(scene_camera);
    
    // Get viewport rect for rendering
    let viewport_rect = egui::Rect::from_center_size(
        center,
        egui::vec2(painter.clip_rect().width(), painter.clip_rect().height()),
    );
    
    // Render all objects in sorted order
    for render_object in render_queue.get_sorted() {
        match render_object {
            RenderObject::Grid => {
                // Grid is rendered separately in the main scene view
            }
            RenderObject::Sprite(sprite_data) => {
                // Render sprite using sprite renderer
                sprite_renderer.render(painter, &[sprite_data.clone()], scene_camera, viewport_rect, texture_manager, ctx);
                
                // Check for hover
                if let Some(screen_sprite) = sprite_renderer.project_sprite_to_screen(
                    sprite_data,
                    scene_camera,
                    Vec2::new(center.x, center.y),
                ) {
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(screen_sprite.screen_pos.x, screen_sprite.screen_pos.y),
                        egui::vec2(screen_sprite.screen_size.x, screen_sprite.screen_size.y),
                    );
                    
                    if let Some(hover_pos) = response.hover_pos() {
                        if rect.contains(hover_pos) {
                            *hovered_entity = Some(sprite_data.entity);
                        }
                    }
                }
            }
            RenderObject::Tilemap(layer) => {
                // Render tilemap using tilemap renderer
                tilemap_renderer.render(painter, &[layer.clone()], scene_camera, viewport_rect);
                
                // Check for hover on tilemap bounds
                if let Some(hover_pos) = response.hover_pos() {
                    // Project bounds to screen space for hover detection
                    let viewport_center = Vec2::new(center.x, center.y);
                    let screen_tiles = tilemap_renderer.project_tilemap_to_screen(layer, scene_camera, viewport_center);
                    
                    for screen_tile in screen_tiles {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(screen_tile.screen_pos.x, screen_tile.screen_pos.y),
                            egui::vec2(screen_tile.screen_size.x, screen_tile.screen_size.y),
                        );
                        
                        if rect.contains(hover_pos) {
                            *hovered_entity = Some(layer.entity);
                            break;
                        }
                    }
                }
            }
            RenderObject::Gizmo(_) => {
                // Gizmos are rendered separately
            }
        }
    }
    
    // Render mesh entities (legacy rendering for non-sprite/tilemap entities)
    for (entity, transform) in mesh_entities.iter() {
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
            response,
        );
    }
    
    // Render selection outline and bounds on top
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let viewport_center = Vec2::new(center.x, center.y);
            
            // Render sprite bounds
            if world.sprites.contains_key(&sel_entity) {
                // Find sprite in collected sprites
                let sprite_data = sprite_renderer.collect_sprites(world)
                    .into_iter()
                    .find(|s| s.entity == sel_entity);
                
                if let Some(sprite) = sprite_data {
                    sprite_renderer.render_bounds(
                        painter,
                        &sprite,
                        scene_camera,
                        viewport_center,
                        egui::Color32::from_rgb(255, 200, 0),
                    );
                }
            }
            // Render tilemap bounds
            else if world.tilemaps.contains_key(&sel_entity) {
                let tilemap_layers = tilemap_renderer.collect_tilemaps(world);
                let layer = tilemap_layers.into_iter().find(|l| l.entity == sel_entity);
                
                if let Some(layer) = layer {
                    tilemap_renderer.render_bounds(
                        painter,
                        &layer,
                        scene_camera,
                        viewport_center,
                        egui::Color32::from_rgb(255, 200, 0),
                    );
                }
            }
            // Render mesh bounds (legacy)
            else if world.meshes.contains_key(&sel_entity) {
                let (screen_x, screen_y) = project_point_3d(transform, scene_camera, center);
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
                let (screen_x, screen_y) = project_point_3d(transform, scene_camera, center);
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
    let entity_rect = if let Some(_sprite) = world.sprites.get(&entity) {
        let scale = calculate_perspective_scale(transform, scene_camera);
        let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        let size = egui::vec2(transform_scale.x * scale, transform_scale.y * scale);
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
        let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        let size = egui::vec2(transform_scale.x * scale, transform_scale.y * scale);
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
