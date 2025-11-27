//! Entity Rendering
//!
//! Functions for rendering entities (sprites, meshes, cameras).

use ecs::{World, Entity, MeshType};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_collider_gizmo, render_velocity_gizmo};

/// Render all entities in the scene
pub fn render_all_entities(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    projection_mode: &ProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // Collect and sort entities by Z position for proper depth rendering in 3D mode
    let mut entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .map(|(&e, t)| (e, t))
        .collect();
    
    if *scene_view_mode == SceneViewMode::Mode3D {
        // Sort by Z position (far to near) for painter's algorithm
        // This ensures proper depth ordering as per Requirements 8.1
        entities.sort_by(|a, b| a.1.position[2].partial_cmp(&b.1.position[2]).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    // Separate entities into opaque and transparent for proper rendering order
    // Transparent objects need special handling for alpha blending (Requirements 8.2)
    let (opaque_entities, transparent_entities): (Vec<_>, Vec<_>) = entities.into_iter()
        .partition(|(entity, _)| {
            // Check if entity has transparency
            let is_opaque = if let Some(sprite) = world.sprites.get(entity) {
                sprite.color[3] >= 1.0
            } else if let Some(mesh) = world.meshes.get(entity) {
                mesh.color[3] >= 1.0
            } else {
                true // Default to opaque
            };
            is_opaque
        });
    
    // Render opaque entities first (back-to-front)
    for (entity, transform) in opaque_entities.iter() {
        render_single_entity(
            painter, 
            *entity, 
            transform, 
            world, 
            scene_camera, 
            scene_view_mode, 
            projection_mode, 
            center, 
            selected_entity, 
            show_colliders, 
            show_velocities, 
            hovered_entity, 
            response
        );
    }
    
    // Render transparent entities after opaque ones (Requirements 8.2)
    // Transparent objects must be rendered back-to-front for correct alpha blending
    for (entity, transform) in transparent_entities.iter() {
        render_single_entity(
            painter, 
            *entity, 
            transform, 
            world, 
            scene_camera, 
            scene_view_mode, 
            projection_mode, 
            center, 
            selected_entity, 
            show_colliders, 
            show_velocities, 
            hovered_entity, 
            response
        );
    }
    
    // === SELECTION OUTLINES AND GIZMOS === (Requirements 8.3, 8.4)
    // Render selection outlines and transform gizmos on top of all entities
    if let Some(sel_entity) = *selected_entity {
        if let Some(transform) = world.transforms.get(&sel_entity) {
            let (screen_x, screen_y) = if *scene_view_mode == SceneViewMode::Mode3D {
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
            } else {
                let world_pos = glam::Vec2::new(transform.x(), transform.y());
                let screen_pos = scene_camera.world_to_screen(world_pos);
                (center.x + screen_pos.x, center.y + screen_pos.y)
            };
            
            // Draw selection outline on top
            if let Some(sprite) = world.sprites.get(&sel_entity) {
                let size = if *scene_view_mode == SceneViewMode::Mode3D {
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
                    let scale = if perspective_z > 10.0 {
                        (distance / perspective_z) * scene_camera.zoom
                    } else {
                        scene_camera.zoom
                    };
                    egui::vec2(sprite.width * scale, sprite.height * scale)
                } else {
                    egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom)
                };
                
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

fn render_single_entity(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    projection_mode: &ProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // Calculate screen position with proper 3D projection for Z-axis
    let (screen_x, screen_y) = if *scene_view_mode == SceneViewMode::Mode3D {
        // 3D mode: Project 3D position (X, Y, Z) to 2D screen
        // Apply camera position offset (for panning)
        let pos_3d = Point3D::new(
            transform.x() - scene_camera.position.x,
            transform.y(),
            transform.position[2] - scene_camera.position.y, // camera.position.y maps to world Z
        );

        // Apply camera rotation
        let yaw = scene_camera.rotation.to_radians();
        let pitch = scene_camera.pitch.to_radians();
        let rotated = pos_3d
            .rotate_y(-yaw)
            .rotate_x(pitch);

        // Perspective projection
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
    } else {
        // 2D mode: Simple X, Y projection
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        (center.x + screen_pos.x, center.y + screen_pos.y)
    };

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
            *hovered_entity = Some(entity);
        }
    }

    // Draw entity (sprite or mesh)
    if world.meshes.contains_key(&entity) {
        render_mesh_entity(painter, entity, transform, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity), scene_view_mode, projection_mode);
    } else {
        render_entity(painter, entity, transform, world, screen_x, screen_y, scene_camera, *selected_entity == Some(entity), scene_view_mode);
    }
    
    // Draw gizmos (but not selection outlines yet - those render on top)
    if *show_colliders && *selected_entity != Some(entity) {
        render_collider_gizmo(painter, entity, world, screen_x, screen_y, scene_camera, false);
    }
    
    if *show_velocities {
        render_velocity_gizmo(painter, entity, world, screen_x, screen_y);
    }
}

fn render_entity(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    _is_selected: bool,
    scene_view_mode: &SceneViewMode,
) {
    if let Some(sprite) = world.sprites.get(&entity) {
        // Calculate size based on view mode
        let size = if *scene_view_mode == SceneViewMode::Mode3D {
            // In 3D mode, calculate perspective-correct size
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

            // Calculate perspective scale based on distance
            let distance = 500.0;
            let perspective_z = rotated.z + distance;
            let scale = if perspective_z > 10.0 {
                (distance / perspective_z) * scene_camera.zoom
            } else {
                scene_camera.zoom
            };

            egui::vec2(sprite.width * scale, sprite.height * scale)
        } else {
            // 2D mode - simple zoom-based size
            egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom)
        };

        let color = egui::Color32::from_rgba_unmultiplied(
            (sprite.color[0] * 255.0) as u8,
            (sprite.color[1] * 255.0) as u8,
            (sprite.color[2] * 255.0) as u8,
            (sprite.color[3] * 255.0) as u8,
        );

        // Draw sprite
        // In 3D mode with billboard enabled: sprite always faces camera (no rotation)
        // In 2D mode or 3D mode with billboard disabled: sprite can be rotated
        if *scene_view_mode == SceneViewMode::Mode3D && sprite.billboard {
            // Billboard mode: draw sprite facing camera (no rotation applied)
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                color,
            );

            // Draw subtle outline for billboard sprites in 3D
            painter.rect_stroke(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 50)),
            );
        } else {
            // Normal mode: draw sprite with rotation
            // Apply sprite rotation based on transform.rotation[2] (Z-axis rotation)
            let rotation_rad = transform.rotation[2].to_radians();
            
            if rotation_rad.abs() < 0.01 {
                // No rotation - use simple rect for better performance
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                    2.0,
                    color,
                );
            } else {
                // Has rotation - draw as rotated polygon
                let half_width = size.x / 2.0;
                let half_height = size.y / 2.0;
                
                // Calculate 4 corners of rotated rectangle
                let cos_r = rotation_rad.cos();
                let sin_r = rotation_rad.sin();
                
                let corners = [
                    // Top-left
                    egui::pos2(
                        screen_x + (-half_width * cos_r - (-half_height) * sin_r),
                        screen_y + (-half_width * sin_r + (-half_height) * cos_r),
                    ),
                    // Top-right
                    egui::pos2(
                        screen_x + (half_width * cos_r - (-half_height) * sin_r),
                        screen_y + (half_width * sin_r + (-half_height) * cos_r),
                    ),
                    // Bottom-right
                    egui::pos2(
                        screen_x + (half_width * cos_r - half_height * sin_r),
                        screen_y + (half_width * sin_r + half_height * cos_r),
                    ),
                    // Bottom-left
                    egui::pos2(
                        screen_x + (-half_width * cos_r - half_height * sin_r),
                        screen_y + (-half_width * sin_r + half_height * cos_r),
                    ),
                ];
                
                // Draw rotated sprite as polygon
                painter.add(egui::Shape::convex_polygon(
                    corners.to_vec(),
                    color,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 30)),
                ));
            }
        }

        // Selection outline removed - now rendered separately on top (Requirements 8.3)
    } else {
        // Check if this is a camera entity (has "Camera" in name or specific tag)
        let is_camera = world.names.get(&entity)
            .map(|name| name.contains("Camera") || name.contains("camera"))
            .unwrap_or(false);
        
        if is_camera {
            // Render Unity-style camera gizmo (trapezoid shape)
            render_camera_gizmo(painter, screen_x, screen_y, scene_camera, scene_view_mode);
        } else {
            // Default: render as gray circle
            painter.circle_filled(egui::pos2(screen_x, screen_y), 5.0 * scene_camera.zoom, egui::Color32::from_rgb(150, 150, 150));
        }
        // Selection outline removed - now rendered separately on top (Requirements 8.3)
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
    _is_selected: bool,
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
        
        match &mesh.mesh_type {
            MeshType::Cube => {
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
            MeshType::Sphere => {
                painter.circle_filled(egui::pos2(screen_x, screen_y), base_size / 2.0, color);
                painter.circle_stroke(egui::pos2(screen_x, screen_y), base_size / 2.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
            MeshType::Cylinder => {
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
