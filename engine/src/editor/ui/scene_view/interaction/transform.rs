//! Transform Interaction
//!
//! Transform gizmo interaction handlers (move, rotate, scale).

use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
use super::super::types::*;

/// Handle transform gizmo interaction
pub fn handle_gizmo_interaction_stateful(
    response: &egui::Response,
    entity: Entity,
    world: &mut World,
    screen_x: f32,
    screen_y: f32,
    current_tool: &TransformTool,
    scene_camera: &SceneCamera,
    dragging_entity: &mut Option<Entity>,
    drag_axis: &mut Option<u8>,
    transform_space: &TransformSpace,
    transform: &ecs::Transform,
) {
    if *current_tool == TransformTool::View {
        return;
    }

    // Calculate rotation for gizmo handles
    let rotation_rad = match transform_space {
        TransformSpace::Local => {
            scene_camera.get_rotation_radians() + transform.rotation[2].to_radians()
        }
        TransformSpace::World => {
            scene_camera.get_rotation_radians()
        }
    };

    // Start dragging - determine which handle
    if response.drag_started() {
        if let Some(hover_pos) = response.hover_pos() {
            let gizmo_size = 50.0;
            let handle_size = 8.0;
            let center = egui::pos2(screen_x, screen_y);
            
            match current_tool {
                TransformTool::Move => {
                    // Calculate rotated handle positions
                    let x_dir = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
                    let y_dir = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos()); // Perpendicular to X
                    
                    let x_handle = egui::pos2(
                        screen_x + x_dir.x * gizmo_size,
                        screen_y + x_dir.y * gizmo_size
                    );
                    let y_handle = egui::pos2(
                        screen_x - gizmo_size * rotation_rad.sin(),
                        screen_y - gizmo_size * rotation_rad.cos()
                    );
                    
                    let dist_x = hover_pos.distance(x_handle);
                    let dist_y = hover_pos.distance(y_handle);
                    let dist_center = hover_pos.distance(center);
                    
                    if dist_center < handle_size * 1.5 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(2); // Both axes
                    } else if dist_x < handle_size * 1.5 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(0); // X only
                    } else if dist_y < handle_size * 1.5 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(1); // Y only
                    }
                }
                TransformTool::Rotate => {
                    // Check if mouse is near the rotation circle
                    let radius = gizmo_size * 0.8;
                    let dist_from_center = hover_pos.distance(center);
                    let dist_from_circle = (dist_from_center - radius).abs();
                    
                    // If mouse is near the circle (within 25 pixels for easier clicking)
                    // OR if mouse is anywhere inside the gizmo area
                    if dist_from_circle < 25.0 || dist_from_center < radius {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(0); // Use axis 0 for rotation
                    }
                }
                TransformTool::Scale => {
                    // Check if mouse is near any corner handle
                    let box_size = gizmo_size * 0.7;
                    let corners = [
                        egui::pos2(screen_x - box_size, screen_y - box_size), // Top-left
                        egui::pos2(screen_x + box_size, screen_y - box_size), // Top-right
                        egui::pos2(screen_x - box_size, screen_y + box_size), // Bottom-left
                        egui::pos2(screen_x + box_size, screen_y + box_size), // Bottom-right
                    ];
                    
                    for corner in &corners {
                        if hover_pos.distance(*corner) < handle_size * 1.5 {
                            *dragging_entity = Some(entity);
                            *drag_axis = Some(0); // Use axis 0 for uniform scale
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Continue dragging
    if response.dragged() && *dragging_entity == Some(entity) {
        let delta = response.drag_delta();

        if let Some(transform_mut) = world.transforms.get_mut(&entity) {
            match current_tool {
                TransformTool::Move => {
                    // Convert screen delta to world delta (accounting for zoom)
                    let screen_delta = glam::Vec2::new(delta.x, delta.y);
                    
                    // Calculate rotation for axis-aligned movement
                    let rotation_rad = match transform_space {
                        TransformSpace::Local => {
                            // In 2D mode, only use object rotation (no camera rotation)
                            transform.rotation[2].to_radians()
                        }
                        TransformSpace::World => {
                            // World space: no rotation
                            0.0
                        }
                    };
                    
                    // Rotate screen delta to world space
                    let cos_r = rotation_rad.cos();
                    let sin_r = rotation_rad.sin();
                    let world_delta_x = (screen_delta.x * cos_r + screen_delta.y * sin_r) / scene_camera.zoom;
                    let world_delta_y = (-screen_delta.x * sin_r + screen_delta.y * cos_r) / scene_camera.zoom;
                    
                    if let Some(axis) = *drag_axis {
                        match axis {
                            0 => {
                                // X axis only - project delta onto X axis
                                transform_mut.position[0] += world_delta_x;
                            }
                            1 => {
                                // Y axis only - project delta onto Y axis
                                // Fixed: Remove negative sign for correct Y movement
                                transform_mut.position[1] += world_delta_y;
                            }
                            2 => {
                                // Both axes - free movement
                                transform_mut.position[0] += world_delta_x;
                                transform_mut.position[1] += world_delta_y;
                            }
                            _ => {}
                        }
                    }
                }
                TransformTool::Rotate => {
                    // Unity-style rotation: use horizontal drag only
                    // Positive delta.x = rotate counter-clockwise (standard math convention)
                    let rotation_speed = 0.5;
                    transform_mut.rotation[2] += delta.x * rotation_speed;
                }
                TransformTool::Scale => {
                    // Increased scale speed for better control
                    let scale_speed = 0.005;
                    let scale_delta = (delta.x + delta.y) * scale_speed;
                    transform_mut.scale[0] += scale_delta;
                    transform_mut.scale[1] += scale_delta;
                    transform_mut.scale[0] = transform_mut.scale[0].max(0.1);
                    transform_mut.scale[1] = transform_mut.scale[1].max(0.1);
                }
                _ => {}
            }
        }
    }
}
