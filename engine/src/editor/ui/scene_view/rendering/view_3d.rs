//! 3D Scene View Rendering
//!
//! Handles rendering of the scene in 3D mode (meshes, billboards, grid, gizmos).

use ecs::{World, Entity, MeshType};
use egui;
use glam::{Vec2, Vec3};
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_camera_frustum_3d, render_collider_gizmo, render_velocity_gizmo};

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
use super::sprite_3d::Sprite3DRenderer;
use super::tilemap_3d::Tilemap3DRenderer;
use super::render_queue::{RenderQueue, RenderObject};
use super::projection_3d::{self, Transform3D, ProjectionMatrix};

/// Render the scene in 3D mode
pub fn render_scene_3d(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    projection_mode: &SceneProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    _show_debug_lines: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
    texture_manager: &mut crate::texture_manager::TextureManager,
    ctx: &egui::Context,
    tilemap_settings: Option<&crate::editor::tilemap_settings::TilemapSettings>,
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
    let mut tilemap_layers = tilemap_renderer.collect_tilemaps(world, tilemap_settings);
    
    // Depth sort tilemap layers
    tilemap_renderer.depth_sort_layers(&mut tilemap_layers);
    
    // Add tilemaps to render queue
    for layer in tilemap_layers {
        render_queue.push(RenderObject::Tilemap(layer));
    }
    
    // Collect mesh entities and camera entities (not handled by sprite/tilemap renderers)
    let mut mesh_entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .filter(|(entity, _)| {
            // Include entities with meshes (but not sprites) OR entities with cameras
            (world.meshes.contains_key(entity) && !world.sprites.contains_key(entity)) ||
            world.cameras.contains_key(entity)
        })
        .map(|(&e, t)| (e, t))
        .collect();
    
    // Sort mesh entities by Z position (simple sort, should use distance from camera)
    mesh_entities.sort_by(|a, b| {
        // Calculate distance from camera for better sorting
        let dist_a = (Vec3::from(a.1.position) - Vec3::new(scene_camera.position.x, 0.0, scene_camera.position.y)).length_squared();
        let dist_b = (Vec3::from(b.1.position) - Vec3::new(scene_camera.position.x, 0.0, scene_camera.position.y)).length_squared();
        dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Sort render queue by depth
    render_queue.sort_by_depth(scene_camera);
    
    // Get viewport rect for rendering
    let viewport_rect = response.rect;
    let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
    
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
                    viewport_rect,
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
                tilemap_renderer.render(painter, &[layer.clone()], scene_camera, viewport_rect, texture_manager, ctx);
                
                // Check for hover on tilemap bounds
                if let Some(hover_pos) = response.hover_pos() {
                    // Project bounds to screen space for hover detection
                    let screen_tiles = tilemap_renderer.project_tilemap_to_screen(layer, scene_camera, viewport_rect);
                    
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
    // But skip camera entities - they will be rendered on top later
    for (entity, transform) in mesh_entities.iter() {
        // Skip camera entities - render them on top later
        if world.cameras.contains_key(entity) {
            continue;
        }
        
        render_entity_3d(
            painter,
            *entity,
            transform,
            world,
            scene_camera,
            projection_mode,
            viewport_size,
            &viewport_rect,
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
                        viewport_rect,
                        egui::Color32::from_rgb(255, 200, 0),
                    );
                }
            }
            // Render tilemap bounds
            else if world.tilemaps.contains_key(&sel_entity) {
                let tilemap_layers = tilemap_renderer.collect_tilemaps(world, tilemap_settings);
                let layer = tilemap_layers.into_iter().find(|l| l.entity == sel_entity);
                
                if let Some(layer) = layer {
                    tilemap_renderer.render_bounds(
                        painter,
                        &layer,
                        scene_camera,
                        viewport_rect,
                        egui::Color32::from_rgb(255, 200, 0),
                    );
                }
            }
            // Render mesh bounds (legacy)
            else if world.meshes.contains_key(&sel_entity) {
                let world_pos = Vec3::from(transform.position);
                if let Some(screen_pos) = projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
                    let scale_vec = glam::Vec3::from(transform.scale);
                    let world_size = 1.0;
                    
                    let base_size = world_size * scale_vec.x.max(scale_vec.y).max(scale_vec.z);
                    let selection_size = base_size; // Selection bounds matching object size
                    
                    // We need to project the bounds properly because 'base_size' is in world units,
                    // but we are drawing a 2D rect here using screen coordinates.
                    // Using calculate_3d_cube_bounds is better but we are inside render_scene_3d.
                    // For now, let's use the calculate_3d_cube_bounds logic which projects vertices.
                    
                    let bounds = calculate_3d_cube_bounds(screen_pos.x, screen_pos.y, base_size, transform, scene_camera, projection_mode, viewport_size, &viewport_rect);
                    
                    painter.rect_stroke(
                        bounds,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                    );
                }
            }
            
            // Draw selected entity's collider gizmo on top
            if *show_colliders {
                let world_pos = Vec3::from(transform.position);
                if let Some(screen_pos) = projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
                    render_collider_gizmo(painter, sel_entity, world, screen_pos.x, screen_pos.y, scene_camera, true);
                }
            }
        }
    }
    
    // Render camera gizmos on top of everything else
    // Collect ALL camera entities (both with and without meshes)
    let mut camera_entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
        .filter(|(entity, _)| world.cameras.contains_key(entity))
        .map(|(&e, t)| (e, t))
        .collect();
    
    // Always render camera gizmos, even if projection fails
    for (entity, transform) in camera_entities.iter() {
        let world_pos = Vec3::from(transform.position);
        
        // Try multiple projection methods to ensure visibility
        let screen_result = projection_3d::world_to_screen(world_pos, scene_camera, viewport_size)
            .or_else(|| projection_3d::world_to_screen_allow_behind(world_pos, scene_camera, viewport_size));
        
        match screen_result {
            Some(screen_pos) => {
                let screen_x = viewport_rect.min.x + screen_pos.x;
                let screen_y = viewport_rect.min.y + screen_pos.y;
                
                // Render camera gizmo
                render_camera_gizmo(painter, screen_x, screen_y, *entity, world, scene_camera, &SceneViewMode::Mode3D);
                
                // Render camera frustum (pyramid showing FOV)
                render_camera_frustum_3d(painter, *entity, world, scene_camera, viewport_rect, egui::pos2(screen_x, screen_y));
            }
            None => {
                // Fallback: render at fixed position to ensure visibility
                let fallback_x = viewport_rect.min.x + 100.0 + (*entity as f32 * 60.0); // Offset multiple cameras
                let fallback_y = viewport_rect.min.y + 100.0;
                
                // Render camera gizmo with fallback position
                render_camera_gizmo(painter, fallback_x, fallback_y, *entity, world, scene_camera, &SceneViewMode::Mode3D);
                
                // Add a label to indicate this is a fallback position
                painter.text(
                    egui::pos2(fallback_x + 60.0, fallback_y),
                    egui::Align2::LEFT_CENTER,
                    format!("Cam {} (off-screen)", entity),
                    egui::FontId::proportional(12.0),
                    egui::Color32::from_rgb(255, 100, 100),
                );
                
                // Still try to render frustum with fallback position
                render_camera_frustum_3d(painter, *entity, world, scene_camera, viewport_rect, egui::pos2(fallback_x, fallback_y));
            }
        }
    }
    
    // Debug: Always render at least one test camera gizmo at center of screen
    if camera_entities.is_empty() {
        let center_x = viewport_rect.center().x;
        let center_y = viewport_rect.center().y;
        
        painter.text(
            egui::pos2(center_x, center_y - 50.0),
            egui::Align2::CENTER_CENTER,
            "No cameras found in scene",
            egui::FontId::proportional(16.0),
            egui::Color32::from_rgb(255, 100, 100),
        );
        
        // Render a test camera gizmo
        render_default_camera_gizmo(painter, center_x, center_y, &SceneViewMode::Mode3D);
    }
}

fn render_entity_3d(
    painter: &egui::Painter,
    entity: Entity,
    transform: &ecs::Transform,
    world: &World,
    scene_camera: &SceneCamera,
    projection_mode: &SceneProjectionMode,
    viewport_size: Vec2,
    viewport_rect: &egui::Rect,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    let world_pos = Vec3::from(transform.position);
    
    // Project to screen (returns position relative to viewport 0,0)
    let screen_pos = match projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
        Some(pos) => pos,
        None => return, // Behind camera or invalid
    };
    
    // Convert viewport-relative position to absolute screen position
    let screen_x = viewport_rect.min.x + screen_pos.x;
    let screen_y = viewport_rect.min.y + screen_pos.y;

    // Calculate scale factor based on distance
    let dist = (world_pos - Vec3::new(scene_camera.position.x, 0.0, scene_camera.position.y)).length();
    let scale_factor = if dist > 0.1 { 500.0 / dist } else { 1.0 };

    // Get entity bounds for click detection
    let entity_rect = if let Some(_sprite) = world.sprites.get(&entity) {
        let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        let size = egui::vec2(transform_scale.x * scale_factor, transform_scale.y * scale_factor);
        egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size)
    } else if world.meshes.contains_key(&entity) {
        let scale_vec = glam::Vec3::from(transform.scale);
        let world_size = 1.0;
        let base_size = world_size * scale_vec.x.max(scale_vec.y).max(scale_vec.z);
        calculate_3d_cube_bounds(screen_x, screen_y, base_size, transform, scene_camera, projection_mode, viewport_size, viewport_rect)
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
        let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
        let size = egui::vec2(transform_scale.x * scale_factor, transform_scale.y * scale_factor);
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
            painter.rect_filled(
                egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size),
                2.0,
                color,
            );
        }
    } else if let Some(mesh) = world.meshes.get(&entity) {
        render_mesh_entity_3d(painter, entity, transform, mesh, screen_x, screen_y, scene_camera, projection_mode, viewport_size, viewport_rect);
    } else {
         // Default placeholder
        // Check if entity has Camera component
        let is_camera = world.cameras.contains_key(&entity);
        
        if is_camera {
            render_camera_gizmo(painter, screen_x, screen_y, entity, world, scene_camera, &SceneViewMode::Mode3D);
            // Render camera frustum (pyramid showing FOV)
            render_camera_frustum_3d(painter, entity, world, scene_camera, *viewport_rect, egui::pos2(screen_x, screen_y));
        } else {
            painter.circle_filled(egui::pos2(screen_x, screen_y), 5.0 * scale_factor, egui::Color32::from_rgb(150, 150, 150));
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



fn render_mesh_entity_3d(
    painter: &egui::Painter,
    _entity: Entity,
    transform: &ecs::Transform,
    mesh: &ecs::Mesh,
    screen_x: f32,
    screen_y: f32,
    scene_camera: &SceneCamera,
    projection_mode: &SceneProjectionMode,
    viewport_size: Vec2,
    viewport_rect: &egui::Rect,
) {
    let color = egui::Color32::from_rgba_unmultiplied(
        (mesh.color[0] * 255.0) as u8,
        (mesh.color[1] * 255.0) as u8,
        (mesh.color[2] * 255.0) as u8,
        (mesh.color[3] * 255.0) as u8,
    );
    
    let scale = glam::Vec3::from(transform.scale);
    let world_size = 1.0;
    
    // We do NOT use scale_factor for 3D meshes anymore. They should scale naturally with perspective.
    let base_size = world_size * scale.x.max(scale.y).max(scale.z);
    
    match mesh.mesh_type {
        MeshType::Cube => {
            render_3d_cube(painter, screen_x, screen_y, base_size, transform, color, scene_camera, projection_mode, viewport_size, viewport_rect);
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
    _projection_mode: &SceneProjectionMode,
    viewport_size: Vec2,
    viewport_rect: &egui::Rect,
) -> egui::Rect {
    let half = size / 2.0;
    let scale = transform.scale;
    
    // Define vertices in local space
    let vertices = [
        Vec3::new(-half * scale[0], -half * scale[1], -half * scale[2]),
        Vec3::new(half * scale[0], -half * scale[1], -half * scale[2]),
        Vec3::new(half * scale[0], half * scale[1], -half * scale[2]),
        Vec3::new(-half * scale[0], half * scale[1], -half * scale[2]),
        Vec3::new(-half * scale[0], -half * scale[1], half * scale[2]),
        Vec3::new(half * scale[0], -half * scale[1], half * scale[2]),
        Vec3::new(half * scale[0], half * scale[1], half * scale[2]),
        Vec3::new(-half * scale[0], half * scale[1], half * scale[2]),
    ];
    
    let world_pos = Vec3::from(transform.position);
    
    // Project vertices to screen space
    let projected: Vec<Vec2> = vertices.iter()
        .filter_map(|v| {
            // Apply rotation
            // Note: This is a simplified rotation, ideally we'd use Quaternions or Mat4
            // For now, just adding to world position
            let v_world = world_pos + *v;
            projection_3d::world_to_screen(v_world, scene_camera, viewport_size)
        })
        .collect();
    
    if projected.is_empty() {
        return egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(size, size));
    }
    
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    
    for p in projected {
        min_x = min_x.min(p.x + viewport_rect.min.x);
        max_x = max_x.max(p.x + viewport_rect.min.x);
        min_y = min_y.min(p.y + viewport_rect.min.y);
        max_y = max_y.max(p.y + viewport_rect.min.y);
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
    _projection_mode: &SceneProjectionMode,
    viewport_size: Vec2,
    viewport_rect: &egui::Rect,
) {
    let half = size / 2.0;
    let scale = transform.scale;
    
    // Define vertices in local space
    let vertices = [
        Vec3::new(-half * scale[0], -half * scale[1], -half * scale[2]),
        Vec3::new(half * scale[0], -half * scale[1], -half * scale[2]),
        Vec3::new(half * scale[0], half * scale[1], -half * scale[2]),
        Vec3::new(-half * scale[0], half * scale[1], -half * scale[2]),
        Vec3::new(-half * scale[0], -half * scale[1], half * scale[2]),
        Vec3::new(half * scale[0], -half * scale[1], half * scale[2]),
        Vec3::new(half * scale[0], half * scale[1], half * scale[2]),
        Vec3::new(-half * scale[0], half * scale[1], half * scale[2]),
    ];
    
    let world_pos = Vec3::from(transform.position);
    
    // Project vertices to screen space
    let projected: Vec<Option<Vec2>> = vertices.iter()
        .map(|v| {
            // Apply rotation (simplified)
            let v_world = world_pos + *v;
            projection_3d::world_to_screen(v_world, scene_camera, viewport_size)
        })
        .collect();
    
    // Define faces (indices)
    let faces = [
        [0, 1, 2, 3], // Front
        [5, 4, 7, 6], // Back
        [1, 0, 4, 5], // Bottom
        [3, 2, 6, 7], // Top
        [0, 3, 7, 4], // Left
        [2, 1, 5, 6], // Right
    ];
    
    // Render faces
    for face_indices in faces {
        let mut points = Vec::new();
        let mut valid = true;
        
        for &i in &face_indices {
            if let Some(p) = projected[i] {
                points.push(egui::pos2(p.x + viewport_rect.min.x, p.y + viewport_rect.min.y));
            } else {
                valid = false;
                break;
            }
        }
        
        if valid && points.len() == 4 {
            // Simple backface culling (check winding order)
            let v1 = points[1] - points[0];
            let v2 = points[2] - points[0];
            let cross = v1.x * v2.y - v1.y * v2.x;
            
            if cross > 0.0 {
                painter.add(egui::Shape::convex_polygon(
                    points,
                    base_color,
                    egui::Stroke::new(1.5, egui::Color32::from_gray(40)),
                ));
            }
        }
    }
}
