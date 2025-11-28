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
            // Local space: rotate with object
            transform.rotation[2].to_radians()
        }
        TransformSpace::World => {
            // World space: no rotation (aligned with world axes)
            0.0
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
                    // Calculate handle positions
                    let axis_length = gizmo_size;
                    let x_dir = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
                    let y_dir = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos());
                    
                    let x_handle = egui::pos2(
                        screen_x + x_dir.x * axis_length,
                        screen_y + x_dir.y * axis_length
                    );
                    let y_handle = egui::pos2(
                        screen_x + y_dir.x * axis_length,
                        screen_y + y_dir.y * axis_length
                    );
                    
                    let dist_x = hover_pos.distance(x_handle);
                    let dist_y = hover_pos.distance(y_handle);
                    let dist_center = hover_pos.distance(center);
                    
                    if dist_center < handle_size * 2.0 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(2); // Uniform scale
                    } else if dist_x < handle_size * 2.0 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(0); // X axis scale
                    } else if dist_y < handle_size * 2.0 {
                        *dragging_entity = Some(entity);
                        *drag_axis = Some(1); // Y axis scale
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
                    // Step 1: Convert screen delta to world space (no rotation yet)
                    let screen_delta = glam::Vec2::new(delta.x, delta.y);
                    let world_delta = screen_delta / scene_camera.zoom;
                    
                    if let Some(axis) = *drag_axis {
                        match axis {
                            0 | 1 => {
                                // Single axis movement - project onto the axis
                                match transform_space {
                                    TransformSpace::Local => {
                                        // Local space: use object rotation
                                        let rotation_rad = transform.rotation[2].to_radians();
                                        
                                        // Calculate local axis direction in world space
                                        let local_axis = if axis == 0 {
                                            // X axis
                                            glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin())
                                        } else {
                                            // Y axis (perpendicular to X)
                                            glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos())
                                        };
                                        
                                        // Project world delta onto local axis
                                        let projection = world_delta.dot(local_axis);
                                        let movement = local_axis * projection;
                                        
                                        transform_mut.position[0] += movement.x;
                                        transform_mut.position[1] += movement.y;
                                    }
                                    TransformSpace::World => {
                                        // World space: move along world axes
                                        if axis == 0 {
                                            transform_mut.position[0] += world_delta.x;
                                        } else {
                                            transform_mut.position[1] += world_delta.y;
                                        }
                                    }
                                }
                            }
                            2 => {
                                // Both axes - free movement (no projection needed)
                                transform_mut.position[0] += world_delta.x;
                                transform_mut.position[1] += world_delta.y;
                            }
                            _ => {}
                        }
                    }
                }
                TransformTool::Rotate => {
                    // Unity-style rotation: rotate based on angle change around center
                    if let Some(mouse_pos) = response.interact_pointer_pos() {
                        let center = egui::pos2(screen_x, screen_y);
                        let current_pos = mouse_pos;
                        let prev_pos = current_pos - delta;
                        
                        let current_angle = (current_pos.y - center.y).atan2(current_pos.x - center.x);
                        let prev_angle = (prev_pos.y - center.y).atan2(prev_pos.x - center.x);
                        
                        let mut angle_delta = current_angle - prev_angle;
                        
                        // Handle wrapping around PI/-PI
                        if angle_delta > std::f32::consts::PI {
                            angle_delta -= 2.0 * std::f32::consts::PI;
                        } else if angle_delta < -std::f32::consts::PI {
                            angle_delta += 2.0 * std::f32::consts::PI;
                        }
                        
                        // Convert radians to degrees and apply
                        transform_mut.rotation[2] += angle_delta.to_degrees();
                    }
                }
                TransformTool::Scale => {
                    // Convert screen delta to world space
                    let screen_delta = glam::Vec2::new(delta.x, delta.y);
                    let world_delta = screen_delta / scene_camera.zoom;
                    
                    if let Some(axis) = *drag_axis {
                        let scale_speed = 0.01;
                        
                        match axis {
                            0 => {
                                // X axis scale only
                                let rotation_rad = match transform_space {
                                    TransformSpace::Local => transform.rotation[2].to_radians(),
                                    TransformSpace::World => 0.0,
                                };
                                let x_axis = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
                                let scale_delta = world_delta.dot(x_axis) * scale_speed;
                                transform_mut.scale[0] = (transform_mut.scale[0] + scale_delta).max(0.1);
                            }
                            1 => {
                                // Y axis scale only
                                let rotation_rad = match transform_space {
                                    TransformSpace::Local => transform.rotation[2].to_radians(),
                                    TransformSpace::World => 0.0,
                                };
                                let y_axis = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos());
                                let scale_delta = world_delta.dot(y_axis) * scale_speed;
                                transform_mut.scale[1] = (transform_mut.scale[1] + scale_delta).max(0.1);
                            }
                            2 => {
                                // Uniform scale
                                let scale_delta = (world_delta.x + world_delta.y) * scale_speed;
                                transform_mut.scale[0] = (transform_mut.scale[0] + scale_delta).max(0.1);
                                transform_mut.scale[1] = (transform_mut.scale[1] + scale_delta).max(0.1);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
