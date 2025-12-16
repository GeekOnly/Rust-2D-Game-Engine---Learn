//! Grid Rendering
//!
//! 2D and 3D grid rendering functions for the scene view.
//! Supports Unity-style Grid component with different plane orientations (XY, XZ, YZ).

use egui;
use glam::{Vec2, Vec3, Mat4};
use crate::{SceneCamera, SceneGrid};
use crate::grid::InfiniteGrid;
use super::projection_3d;

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
/// If Grid components exist in the scene, render grids based on their plane settings
pub fn render_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    // Use the old grid rendering for now (will be replaced with InfiniteGrid)
    render_grid_3d_legacy(painter, rect, scene_camera, scene_grid, None);
}

/// Render 3D grid with Grid component support (Unity-style)
/// - Always renders default 3D space grid for navigation
/// - If a Grid entity is selected, renders ONLY that Grid's component grid
/// - If no Grid entity is selected, doesn't render any Grid component grids
pub fn render_grid_3d_with_component(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
    world: &ecs::World,
    selected_entity: Option<ecs::Entity>,
) {
    // ALWAYS render default 3D space grid first (for navigation)
    render_grid_3d_legacy(painter, rect, scene_camera, scene_grid, None);
    
    // Unity-style: Render Grid component ONLY if a Grid entity is selected
    if let Some(entity) = selected_entity {
        // Get entity name for debugging
        let entity_name = world.names.get(&entity)
            .map(|s| s.as_str())
            .unwrap_or("Unnamed");
            
        if let Some(grid) = world.grids.get(&entity) {
            // Get transform if available
            let transform = world.transforms.get(&entity);
            
            log::info!("✓ Rendering selected Grid component '{}' (entity {}): plane={:?}, cell_size=({:.3}, {:.3}, {:.3})", 
                entity_name, entity, grid.plane, grid.cell_size.0, grid.cell_size.1, grid.cell_size.2);
            
            // Render Grid component grid with different visual style
            render_grid_component_3d(painter, rect, scene_camera, scene_grid, grid, transform);
        } else {
            log::debug!("Selected entity '{}' (entity {}) does not have Grid component", entity_name, entity);
        }
    } else {
        log::debug!("No entity selected - Grid component grid hidden");
    }
}

/// Render 3D grid using enhanced InfiniteGrid system
/// OPTIMIZED: Uses cached geometry, aggressive culling, and efficient projection
pub fn render_infinite_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    _infinite_grid: &mut InfiniteGrid,
) {
    // For now, use the legacy grid rendering which is known to work
    // TODO: Debug and fix the infinite grid system
    let scene_grid = SceneGrid::new();
    render_grid_3d_legacy(painter, rect, scene_camera, &scene_grid, None);
    
    /* DISABLED: Infinite grid system needs debugging
    let center = rect.center();
    // ...
    */
}

/// Project a 3D point to screen space
#[allow(dead_code)]
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

/// Render Grid component in 3D with distinct visual style
/// This grid represents tilemap layout, not the 3D space grid
fn render_grid_component_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
    grid: &ecs::Grid,
    transform: Option<&ecs::Transform>,
) {
    // Use cell_size from Grid component
    let cell_size = grid.cell_size.0.max(grid.cell_size.1).max(0.01);
    // Always use the Grid component's actual cell size for accurate tilemap alignment
    // The previous threshold of 0.5 was causing misalignment with small tiles (8px at 100 PPU = 0.08)
    let grid_world_size = cell_size;
    
    let grid_plane = grid.plane;
    let grid_offset = transform
        .map(|t| Vec3::new(t.position[0], t.position[1], t.position[2]))
        .unwrap_or(Vec3::ZERO);
    
    let viewport_size = Vec2::new(rect.width(), rect.height());
    
    // Grid component uses BRIGHTER colors to distinguish from space grid
    let grid_color = egui::Color32::from_rgba_premultiplied(100, 100, 100, 120);  // Brighter gray
    let x_axis_color = egui::Color32::from_rgba_premultiplied(255, 80, 80, 255);  // Bright red
    let z_axis_color = egui::Color32::from_rgba_premultiplied(80, 140, 255, 255);  // Bright blue
    let y_axis_color = egui::Color32::from_rgba_premultiplied(80, 255, 80, 255);  // Bright green
    
    let grid_range = 20;  // Smaller range for Grid component
    let fade_distance = 15.0 * grid_world_size;
    
    // Project function that respects grid plane
    let project_3d = |u: f32, v: f32| -> Option<egui::Pos2> {
        let world_pos = match grid_plane {
            ecs::GridPlane::XY => Vec3::new(u, v, 0.0) + grid_offset,
            ecs::GridPlane::XZ => Vec3::new(u, 0.0, v) + grid_offset,
            ecs::GridPlane::YZ => Vec3::new(0.0, u, v) + grid_offset,
        };
        projection_3d::world_to_screen(world_pos, scene_camera, viewport_size)
            .map(|v| egui::pos2(v.x + rect.min.x, v.y + rect.min.y))
    };
    
    let calc_alpha = |u: f32, v: f32| -> u8 {
        let dist = (u * u + v * v).sqrt();
        if dist > fade_distance {
            let fade = 1.0 - ((dist - fade_distance) / (fade_distance * 0.5)).min(1.0);
            (fade * 150.0) as u8
        } else {
            150
        }
    };
    
    // Determine axis colors based on grid plane
    let (axis1_color, axis2_color) = match grid_plane {
        ecs::GridPlane::XY => (x_axis_color, y_axis_color),
        ecs::GridPlane::XZ => (x_axis_color, z_axis_color),
        ecs::GridPlane::YZ => (y_axis_color, z_axis_color),
    };
    
    // Draw grid lines along second axis (V direction)
    for i in -grid_range..=grid_range {
        let u = i as f32 * grid_world_size;
        let is_axis1 = i == 0;
        
        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let v = j as f32 * grid_world_size;
            points.push(project_3d(u, v));
        }
        
        for j in 0..points.len() - 1 {
            if let (Some(p1), Some(p2)) = (points[j], points[j + 1]) {
                let v1 = ((j as i32) - grid_range) as f32 * grid_world_size;
                let alpha = calc_alpha(u, v1);
                
                if alpha > 10 {
                    let color = if is_axis1 {
                        egui::Color32::from_rgba_premultiplied(
                            axis1_color.r(),
                            axis1_color.g(),
                            axis1_color.b(),
                            alpha.max(180),
                        )
                    } else {
                        egui::Color32::from_rgba_premultiplied(
                            grid_color.r(),
                            grid_color.g(),
                            grid_color.b(),
                            alpha,
                        )
                    };
                    
                    let width = if is_axis1 { 2.5 } else { 1.2 };  // Thicker lines
                    painter.line_segment([p1, p2], egui::Stroke::new(width, color));
                }
            }
        }
    }
    
    // Draw grid lines along first axis (U direction)
    for i in -grid_range..=grid_range {
        let v = i as f32 * grid_world_size;
        let is_axis2 = i == 0;
        
        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let u = j as f32 * grid_world_size;
            points.push(project_3d(u, v));
        }
        
        for j in 0..points.len() - 1 {
            if let (Some(p1), Some(p2)) = (points[j], points[j + 1]) {
                let u1 = ((j as i32) - grid_range) as f32 * grid_world_size;
                let alpha = calc_alpha(u1, v);
                
                if alpha > 10 {
                    let color = if is_axis2 {
                        egui::Color32::from_rgba_premultiplied(
                            axis2_color.r(),
                            axis2_color.g(),
                            axis2_color.b(),
                            alpha.max(180),
                        )
                    } else {
                        egui::Color32::from_rgba_premultiplied(
                            grid_color.r(),
                            grid_color.g(),
                            grid_color.b(),
                            alpha,
                        )
                    };
                    
                    let width = if is_axis2 { 2.5 } else { 1.2 };  // Thicker lines
                    painter.line_segment([p1, p2], egui::Stroke::new(width, color));
                }
            }
        }
    }
}

/// Legacy 3D grid rendering (fallback)
/// This renders the default 3D space grid for navigation (always XZ plane - ground)
fn render_grid_3d_legacy(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
    _grid_component: Option<(&ecs::Grid, Option<&ecs::Transform>)>,
) {
    // Default 3D space grid settings (always ground plane)
    let grid_world_size = scene_grid.size;
    let grid_plane = ecs::GridPlane::XZ;  // Always ground plane for 3D space
    let grid_offset = Vec3::ZERO;
    let viewport_size = Vec2::new(rect.width(), rect.height());

    // Unity-like subtle grid colors
    let grid_color = egui::Color32::from_rgba_premultiplied(64, 64, 64, 76);  // Subtle gray
    let x_axis_color = egui::Color32::from_rgba_premultiplied(217, 64, 64, 230);  // Bright red
    let z_axis_color = egui::Color32::from_rgba_premultiplied(64, 115, 217, 230);  // Bright blue

    let grid_range = 50;  // Wider grid like Unity
    let fade_distance = 40.0 * grid_world_size;  // Longer fade distance

    // Project function that respects grid plane
    let project_3d = |u: f32, v: f32| -> Option<egui::Pos2> {
        let world_pos = match grid_plane {
            ecs::GridPlane::XY => Vec3::new(u, v, 0.0) + grid_offset,  // Horizontal (default 2D)
            ecs::GridPlane::XZ => Vec3::new(u, 0.0, v) + grid_offset,  // Vertical (wall)
            ecs::GridPlane::YZ => Vec3::new(0.0, u, v) + grid_offset,  // Side view
        };
        projection_3d::world_to_screen(world_pos, scene_camera, viewport_size)
            .map(|v| egui::pos2(v.x + rect.min.x, v.y + rect.min.y))
    };
    
    let calc_alpha = |u: f32, v: f32| -> u8 {
        let dist = (u * u + v * v).sqrt();
        if dist > fade_distance {
            let fade = 1.0 - ((dist - fade_distance) / (fade_distance * 0.5)).min(1.0);
            (fade * 100.0) as u8
        } else {
            100
        }
    };
    
    // 3D space grid always uses X (red) and Z (blue) axes
    let axis1_color = x_axis_color;  // X axis
    let axis2_color = z_axis_color;  // Z axis

    // Draw grid lines along second axis (V direction)
    for i in -grid_range..=grid_range {
        let u = i as f32 * grid_world_size;
        let is_axis1 = i == 0;

        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let v = j as f32 * grid_world_size;
            points.push(project_3d(u, v));
        }

        for j in 0..points.len() - 1 {
            if let (Some(p1), Some(p2)) = (points[j], points[j+1]) {
                let v1 = ((j as i32) - grid_range) as f32 * grid_world_size;
                let alpha = calc_alpha(u, v1);

                if alpha > 5 {
                    let color = if is_axis1 {
                        egui::Color32::from_rgba_premultiplied(
                            axis1_color.r(),
                            axis1_color.g(),
                            axis1_color.b(),
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

                    let width = if is_axis1 { 2.0 } else { 0.8 };  // Thinner lines like Unity
                    painter.line_segment(
                        [p1, p2],
                        egui::Stroke::new(width, color),
                    );
                }
            }
        }
    }

    // Draw grid lines along first axis (U direction)
    for i in -grid_range..=grid_range {
        let v = i as f32 * grid_world_size;
        let is_axis2 = i == 0;

        let mut points = Vec::new();
        for j in -grid_range..=grid_range {
            let u = j as f32 * grid_world_size;
            points.push(project_3d(u, v));
        }

        for j in 0..points.len() - 1 {
            if let (Some(p1), Some(p2)) = (points[j], points[j+1]) {
                let u1 = ((j as i32) - grid_range) as f32 * grid_world_size;
                let alpha = calc_alpha(u1, v);

                if alpha > 5 {
                    let color = if is_axis2 {
                        egui::Color32::from_rgba_premultiplied(
                            axis2_color.r(),
                            axis2_color.g(),
                            axis2_color.b(),
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

                    let width = if is_axis2 { 2.0 } else { 0.8 };  // Thinner lines like Unity
                    painter.line_segment(
                        [p1, p2],
                        egui::Stroke::new(width, color),
                    );
                }
            }
        }
    }
}
