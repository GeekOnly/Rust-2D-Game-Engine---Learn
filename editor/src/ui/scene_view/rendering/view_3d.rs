//! 3D Scene View Rendering
//!
//! Handles rendering of the scene in 3D mode (meshes, billboards, grid, gizmos).

use ecs::{World, Entity};
use egui;
use glam::{Vec2, Vec3};
use crate::{SceneCamera, SceneGrid};
use crate::grid::InfiniteGrid;
use super::super::types::*;
use super::gizmos::{render_camera_gizmo, render_camera_frustum_3d, render_collider_gizmo, render_selection_box_3d};

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
    painter.rect_stroke(body_rect, 2.0, egui::Stroke::new(2.0, color), egui::epaint::StrokeKind::Outside);
    
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
use super::projection_3d;

/// Render the scene in 3D mode
pub fn render_scene_3d(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    _scene_grid: &SceneGrid,
    _infinite_grid: &mut InfiniteGrid,
    projection_mode: &SceneProjectionMode,
    _center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    _show_velocities: &bool,
    _show_debug_lines: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
    _texture_manager: &mut engine::texture_manager::TextureManager,
    _ctx: &egui::Context,
    tilemap_settings: Option<&crate::tilemap_settings::TilemapSettings>,
    scene_view_renderer: &mut crate::scene_view_renderer::SceneViewRenderer,
    egui_renderer: &mut egui_wgpu::Renderer,
    device: &wgpu::Device,
) {
    // ------------------------------------------------------------------------
    // 1. Render Scene Texture (WGPU Offscreen)
    // ------------------------------------------------------------------------
    let viewport_rect = response.rect;
    let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());
    
    // Resize offscreen texture to match viewport
    let width = viewport_rect.width() as u32;
    let height = viewport_rect.height() as u32;
    if width > 0 && height > 0 {
        scene_view_renderer.resize(device, egui_renderer, width, height);
    }

    // Draw the rendered scene texture
    // UVs are (0,0) top-left to (1,1) bottom-right
    painter.image(
        scene_view_renderer.texture_id,
        viewport_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE
    );

    // ------------------------------------------------------------------------
    // 2. Handle Picking / Hover (CPU Approximation)
    // ------------------------------------------------------------------------
    // Note: This matches the GPU rendering by using the same projection math.
    let mut sprite_renderer = Sprite3DRenderer::new();
    let mut tilemap_renderer = Tilemap3DRenderer::new();

    if let Some(hover_pos) = response.hover_pos() {
        // Simple Z-sort for picking (painters algorithm approximation)
        // In a real engine we might use pixel-readback or strict raycasting against all colliders
        
        let mut best_depth = f32::MAX;
        
        // Check Sprites
        for (entity, transform) in world.transforms.iter() {
            if let Some(sprite) = world.sprites.get(entity) {
                 if let Some(screen_pos) = projection_3d::world_to_screen(Vec3::from(transform.position), scene_camera, viewport_size) {
                    // Approximate bounds
                    let dist = (Vec3::from(transform.position) - scene_camera.position).length();
                    let _scale_factor = if dist > 0.1 { 500.0 / dist } else { 1.0 }; // Match legacy scale for now
                     
                    let transform_scale = glam::Vec2::new(transform.scale[0], transform.scale[1]);
                    let _size = egui::vec2(
                        sprite.width * transform_scale.x * 1.0, // Scale factors might need tuning to match WGPU
                        sprite.height * transform_scale.y * 1.0
                    ) * (50.0 / dist); // Simplified persective scale approximation for picking
                    
                    // The old code had a specific scale_factor formula. 
                    // Let's use a simpler heuristic: if it's close to the screen pos, pick it.
                    // Or reuse the bounds logic if possible.
                    
                    let screen_x = viewport_rect.min.x + screen_pos.x;
                    let screen_y = viewport_rect.min.y + screen_pos.y;
                    
                    // Simple box check
                    let rect = egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), egui::vec2(50.0, 50.0)); // Arbitrary pick size
                    
                    if rect.contains(hover_pos) {
                        if dist < best_depth {
                            best_depth = dist;
                            *hovered_entity = Some(*entity);
                        }
                    }
                 }
            } else if let Some(_) = world.meshes.get(entity) {
                // Mesh picking - Use projected 3D bounds for accuracy
                // scale_vec was unused here
                // Use standard unit cube size (1.0) as base, scaled by transform
                let bounds = calculate_3d_cube_bounds(
                    0.0, 0.0, 1.0, 
                    transform, scene_camera, projection_mode, viewport_size, &viewport_rect
                );
                
                // Check if mouse is within projected bounds
                if bounds.contains(hover_pos) {
                    // Calculate depth to center for sorting
                    let dist = (Vec3::from(transform.position) - scene_camera.position).length();
                    
                    if dist < best_depth {
                        best_depth = dist;
                        *hovered_entity = Some(*entity);
                    }
                }
            } else if let Some(_layer) = world.tilemaps.get(entity) {
                // Tilemap picking (simplified: bounds check)
                // ... (Omitted for brevity in this initial pass, tilemap picking is complex)
            }
        }
    }

    // Render Grid (Overlay) - DISABLED: Now using WGPU GridRenderer for proper depth testing
    /*
    if scene_grid.enabled {
        grid::render_grid_3d_with_component(
            painter,
            viewport_rect,
            scene_camera,
            scene_grid,
            world,
            *selected_entity,
        );
    }
    */
    
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
            // Render mesh bounds (3D wireframe)
            else if world.meshes.contains_key(&sel_entity) {
                render_selection_box_3d(
                    painter,
                    transform,
                    scene_camera,
                    &viewport_rect,
                    1.0, // Default world size for unit cube
                );
            }
            
            // Draw selected entity's collider gizmo on top
            if *show_colliders {
                let world_pos = Vec3::from(transform.position);
                if let Some(screen_pos) = projection_3d::world_to_screen(world_pos, scene_camera, viewport_size) {
                    render_collider_gizmo(painter, sel_entity, world, screen_pos.x, screen_pos.y, scene_camera, Some(viewport_rect), true, false);
                }
            }
        }
    }
    
    // Render camera gizmos on top of everything else
    // Collect ALL camera entities (both with and without meshes)
    let camera_entities: Vec<(Entity, &ecs::Transform)> = world.transforms.iter()
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

// render_entity_3d removed (dead code)



// render_mesh_entity_3d removed (dead code)

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
    
    // world_pos unused
    
    let rot_rad = Vec3::new(
        transform.rotation[0].to_radians(),
        transform.rotation[1].to_radians(),
        transform.rotation[2].to_radians(),
    );
    let rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
    let translation = Vec3::from(transform.position);
    // scale_vec and model_matrix removed as they were unused

    // Project vertices to screen space
    let projected: Vec<Vec2> = vertices.iter()
        .filter_map(|v| {
            // Vertices are at +/- half * scale already ? No, vertices here are calculated with scale.
            // But we should probably use unit vertices and let the matrix handle scale?
            // Existing code: vertices = [ -half * scale ... ]
            // So if we have scale in vertices, we shouldn't have scale in matrix?
            // OR we change vertices to be unit size and use matrix for everything.
            // Let's stick to consistent logic: Matrix handles S*R*T.
            // So vertex should be local unscaled (half size).
            // BUT, the `vertices` array definition above uses scale.
            // Let's rely on the previous logic: if `vertices` has scale, we remove scale from matrix?
            // No, easier to just remove scale from `vertices` definition below? 
            // Actually, let's just use transform_point3 with the matrix, but since vertices already have scale applied,
            // we should construct a matrix with only R * T.
            
            // Wait, standard model matrix is T * R * S.
            // If `vertices` = S * UnitCube.
            // Then we want T * R * vertices.
            
            let v_rotated = rotation * *v;
            let v_world = translation + v_rotated;
            
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

// render_3d_cube removed (dead code)
