//! Transform Interaction
//!
//! Transform gizmo interaction handlers (move, rotate, scale).

use ecs::{World, Entity};
use egui;
use crate::ui::TransformTool;
use crate::SceneCamera;
use super::super::types::*;
use super::super::rendering::projection_3d;
use glam;

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
    scene_view_mode: &SceneViewMode,
    viewport_rect: Option<egui::Rect>,
) {
    if *current_tool == TransformTool::View {
        return;
    }

    // Start dragging - determine which handle
    if response.drag_started() {
        if let Some(hover_pos) = response.hover_pos() {
            let gizmo_size = 80.0; // Match rendering size base (was 60, should be 80)
            let handle_size = 10.0; // Match rendering size
            let center = egui::pos2(screen_x, screen_y);
            
            // Check based on mode
            match scene_view_mode {
                SceneViewMode::Mode3D => {
                    if let Some(rect) = viewport_rect {
                        let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                        let world_pos = glam::Vec3::from(transform.position);

                        // Calculate visual scale (same as rendering)
                        let transform3d_temp = projection_3d::Transform3D::new(world_pos, 0.0, glam::Vec2::ONE);
                        let cam_dist = transform3d_temp.depth_from_camera(scene_camera);
                        let safe_dist = cam_dist.max(0.1);
                        let scale = safe_dist * 0.15; // Tuned constant matching rendering

                        // Projection helper
                        let project = |pos: glam::Vec3| -> Option<egui::Pos2> {
                            projection_3d::world_to_screen(pos, scene_camera, viewport_size)
                                .map(|p| egui::pos2(rect.min.x + p.x, rect.min.y + p.y))
                        };

                        // Calculate Basis Vectors
                        let (right, up, forward) = match transform_space {
                            TransformSpace::Local => {
                                let rot = glam::Quat::from_euler(
                                    glam::EulerRot::XYZ, 
                                    transform.rotation[0].to_radians(), 
                                    transform.rotation[1].to_radians(), 
                                    transform.rotation[2].to_radians()
                                );
                                (rot * glam::Vec3::X, rot * glam::Vec3::Y, rot * -glam::Vec3::Z) 
                            },
                            TransformSpace::World => (glam::Vec3::X, glam::Vec3::Y, -glam::Vec3::Z),
                        };

                        // Project handles
                        // Note origin might be slightly different than screen_x/y due to re-projection, use projected one
                        let p_origin = project(world_pos).unwrap_or(center);
                        let p_right = project(world_pos + right * scale);
                        let p_up = project(world_pos + up * scale);
                        let p_fwd = project(world_pos + forward * scale);

                        // Debug: Print coordinates for troubleshooting
                        println!("3D Gizmo Debug - Hover: {:?}, Origin: {:?}, Scale: {:.3}", hover_pos, p_origin, scale);
                        if let Some(pr) = p_right { println!("  Right handle: {:?}", pr); }
                        if let Some(pu) = p_up { println!("  Up handle: {:?}", pu); }
                        if let Some(pf) = p_fwd { println!("  Forward handle: {:?}", pf); }

                        let dist_center = hover_pos.distance(p_origin);

                        match current_tool {
                            TransformTool::Move => {
                                let dist_x = p_right.map_or(f32::MAX, |p| hover_pos.distance(p));
                                let dist_y = p_up.map_or(f32::MAX, |p| hover_pos.distance(p));
                                let dist_z = p_fwd.map_or(f32::MAX, |p| hover_pos.distance(p));

                                // Much larger hit radius for easier clicking in 3D
                                let hit_radius = handle_size * 3.0;

                                if dist_center < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(3); // All axes (free movement)
                                } else if dist_x < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(0); // X only
                                } else if dist_y < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(1); // Y only
                                } else if dist_z < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(2); // Z only
                                }
                            }
                            TransformTool::Rotate => {
                                // For 3D rotation, check if mouse is near any of the rotation rings
                                let radius = scale * 4.0; // Match the radius_world from rendering
                                let dist = hover_pos.distance(p_origin);
                                // Much more generous hit detection for rotation rings
                                if dist < radius * 1.5 || (dist - radius).abs() < 30.0 {
                                     *dragging_entity = Some(entity);
                                     *drag_axis = Some(0); // TODO: improved 3D rotation axes
                                }
                            }
                            TransformTool::Scale => {
                                let dist_x = p_right.map_or(f32::MAX, |p| hover_pos.distance(p));
                                let dist_y = p_up.map_or(f32::MAX, |p| hover_pos.distance(p));
                                let dist_z = p_fwd.map_or(f32::MAX, |p| hover_pos.distance(p));
                                let hit_radius = handle_size * 4.0; // Much larger hit radius for 3D

                                if dist_center < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(3); // Uniform (Move logic uses 3 for free, Scale uses 2 usually, check logic below)
                                    // Logic below uses 2 for uniform scale
                                    *drag_axis = Some(2); 
                                } else if dist_x < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(0); // X only
                                } else if dist_y < hit_radius {
                                    *dragging_entity = Some(entity);
                                    *drag_axis = Some(1); // Y only
                                } else if dist_z < hit_radius {
                                    *dragging_entity = Some(entity);
                                    // Logic below didn't distinguish Z scale well, reusing Z handle for Z scale
                                    // But wait, the existing code for scale maps 2 to uniform.
                                    // I should probably map 3 to Z scale if supported, but existing logic might not support it.
                                    // Let's check existing logic: TransformTool::Scale...
                                    // axis 2 => Uniform.
                                    // So Z axis handle currently likely maps to uniform or nothing.
                                    // Let's map Z handle to axis 4 (Z scale) if we implement it, or keep consistent.
                                    // Existing code only handles 0, 1, 2 (Uniform).
                                    // Let's interpret Z handle as Uniform for now to avoid breaking things, or add Z support.
                                    // Adding Z support requires updating the drag logic below too.
                                    // For now, let's map Z handle to Uniform scaling as well to be safe, or just ignore it if unsupported.
                                    // Actually, let's modify the drag logic to support Z scale (index 4) or just map to uniform for now.
                                    *drag_axis = Some(2); // Uniform
                                }
                            }
                            _ => {}
                        }
                    }
                }
                SceneViewMode::Mode2D => {
                     // Calculate rotation for gizmo handles (must match rendering)
                    let rotation_rad = match transform_space {
                        TransformSpace::Local => {
                            // Local space: rotate with object (Z rotation only in 2D)
                            transform.rotation[2].to_radians()
                        }
                        TransformSpace::World => {
                            // World space: no rotation (aligned with world axes)
                            0.0
                        }
                    };

                    match current_tool {
                        TransformTool::Move => {
                            // Calculate rotated handle positions (MUST MATCH RENDERING)
                            // X axis direction (inverted Y for screen space)
                            let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
                            // Y axis direction (perpendicular to X, inverted Y for screen space)
                            let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());
                            
                            let x_handle = egui::pos2(
                                screen_x + x_dir.x * gizmo_size,
                                screen_y + x_dir.y * gizmo_size
                            );
                            let y_handle = egui::pos2(
                                screen_x + y_dir.x * gizmo_size,
                                screen_y + y_dir.y * gizmo_size
                            );
                            
                            let dist_x = hover_pos.distance(x_handle);
                            let dist_y = hover_pos.distance(y_handle);
                            let dist_center = hover_pos.distance(center);
                            
                            // Increased hit detection radius for easier clicking
                            if dist_center < handle_size * 1.8 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(3); // All axes (free movement)
                            } else if dist_x < handle_size * 1.8 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(0); // X only
                            } else if dist_y < handle_size * 1.8 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(1); // Y only
                            }
                        }
                        TransformTool::Rotate => {
                            // Check if mouse is near the rotation circle
                            let radius = gizmo_size * 0.8;
                            let dist_from_center = hover_pos.distance(center);
                            let dist_from_circle = (dist_from_center - radius).abs();
                            
                            // If mouse is near the circle (within 30 pixels for easier clicking)
                            // OR if mouse is anywhere inside the gizmo area
                            if dist_from_circle < 30.0 || dist_from_center < radius {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(0); // Use axis 0 for rotation
                            }
                        }
                        TransformTool::Scale => {
                            // Calculate handle positions (MUST MATCH RENDERING)
                            let axis_length = gizmo_size;
                            // X axis direction (inverted Y for screen space)
                            let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
                            // Y axis direction (perpendicular to X, inverted Y for screen space)
                            let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());
                            
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
                            
                            // Increased hit detection radius for easier clicking
                            if dist_center < handle_size * 2.5 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(2); // Uniform scale
                            } else if dist_x < handle_size * 2.5 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(0); // X axis scale
                            } else if dist_y < handle_size * 2.5 {
                                *dragging_entity = Some(entity);
                                *drag_axis = Some(1); // Y axis scale
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Continue dragging
    if response.dragged() && *dragging_entity == Some(entity) {
        let delta = response.drag_delta();

        if let Some(transform_mut) = world.transforms.get_mut(&entity) {
            match scene_view_mode {
                SceneViewMode::Mode3D => {
                    // --------------------------------------------------------
                    // 3D MODE INTERACTION
                    // --------------------------------------------------------
                    if let Some(rect) = viewport_rect {
                        let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                        // Use current position for projection
                        let world_pos = glam::Vec3::from(transform_mut.position);

                        // Recalculate Screen Origin based on current View
                        let project = |pos: glam::Vec3| -> Option<glam::Vec2> {
                             projection_3d::world_to_screen(pos, scene_camera, viewport_size)
                                 .map(|p| glam::Vec2::new(rect.min.x + p.x, rect.min.y + p.y))
                        };

                        if let Some(p_origin) = project(world_pos) {
                             // Recalculate Basis Vectors (Same as Hit Test)
                            let (right, up, forward) = match transform_space {
                                TransformSpace::Local => {
                                    let rot = glam::Quat::from_euler(
                                        glam::EulerRot::XYZ, 
                                        transform_mut.rotation[0].to_radians(), 
                                        transform_mut.rotation[1].to_radians(), 
                                        transform_mut.rotation[2].to_radians()
                                    );
                                    (rot * glam::Vec3::X, rot * glam::Vec3::Y, rot * -glam::Vec3::Z) 
                                },
                                TransformSpace::World => (glam::Vec3::X, glam::Vec3::Y, -glam::Vec3::Z),
                            };

                            // Calculate scale for projection mapping
                            let transform3d_temp = projection_3d::Transform3D::new(world_pos, 0.0, glam::Vec2::ONE);
                            let cam_dist = transform3d_temp.depth_from_camera(scene_camera);
                            let safe_dist = cam_dist.max(0.1);
                            let world_scale = safe_dist * 0.15; 

                            match current_tool {
                                TransformTool::Move => {
                                    if let Some(axis) = *drag_axis {
                                        let (move_axis, is_explicit_axis) = match axis {
                                            0 => (right, true),
                                            1 => (up, true),
                                            2 => (forward, true),
                                            _ => (glam::Vec3::ZERO, false),
                                        };

                                        if is_explicit_axis {
                                            // Project Axis End to Screen
                                            if let Some(p_axis_end) = project(world_pos + move_axis * world_scale) {
                                                // Axis Impulse Vector in Screen Space
                                                let axis_vec = p_axis_end - p_origin; 
                                                let axis_len_sq = axis_vec.length_squared();
                                                
                                                if axis_len_sq > 0.001 {
                                                    let mouse_delta = glam::Vec2::new(delta.x, delta.y);
                                                    
                                                    // Project mouse delta onto screen axis vector
                                                    // t is the proportion of 'world_scale' moved
                                                    let t = mouse_delta.dot(axis_vec) / axis_len_sq;
                                                    let world_move = t * world_scale;
                                                    
                                                    let displacement = move_axis * world_move;
                                                    transform_mut.position[0] += displacement.x;
                                                    transform_mut.position[1] += displacement.y;
                                                    transform_mut.position[2] += displacement.z;
                                                }
                                            }
                                        } else if axis == 3 {
                                            // Free Move (Screen Plane approximation)
                                            // Move parallel to camera view plane
                                            let pixel_conv = safe_dist * 0.001; // Approximate units per pixel
                                            
                                            // Camera Basis
                                            let yaw = scene_camera.rotation.to_radians();
                                            let pitch = scene_camera.pitch.to_radians();

                                            // Construct Right and Up relative to camera view
                                            // Right is perpendicular to Forward and World Up
                                            let forward = glam::Vec3::new(
                                                yaw.cos() * pitch.cos(),
                                                pitch.sin(),
                                                yaw.sin() * pitch.cos()
                                            ).normalize();
                                            let world_up = glam::Vec3::Y;
                                            let right = forward.cross(world_up).normalize();
                                            // Correction: if looking straight up/down, cross layout fails.
                                            // Fallback: simplified
                                            let right_simple = glam::Vec3::new(yaw.sin(), 0.0, -yaw.cos());
                                            
                                            // Screen Y is inverted, so -delta.y moves Up
                                            let displacement = right_simple * delta.x * pixel_conv + glam::Vec3::Y * -delta.y * pixel_conv;
                                            
                                            transform_mut.position[0] += displacement.x;
                                            transform_mut.position[1] += displacement.y;
                                            transform_mut.position[2] += displacement.z;
                                        }
                                    }
                                }
                                TransformTool::Rotate => {
                                    if let Some(mouse_pos) = response.interact_pointer_pos() {
                                        // Use projected center
                                        let center = egui::pos2(p_origin.x, p_origin.y);
                                        let current_pos = mouse_pos;
                                        let prev_pos = current_pos - delta;
                                        
                                        let current_angle = (current_pos.y - center.y).atan2(current_pos.x - center.x);
                                        let prev_angle = (prev_pos.y - center.y).atan2(prev_pos.x - center.x);
                                        
                                        let mut angle_delta = current_angle - prev_angle;
                                        
                                        // Wrap logic
                                        if angle_delta > std::f32::consts::PI { angle_delta -= 2.0 * std::f32::consts::PI; }
                                        else if angle_delta < -std::f32::consts::PI { angle_delta += 2.0 * std::f32::consts::PI; }
                                        
                                        // Rotate Z axis (Roll/2D Spin) is standard for this tool currently
                                        transform_mut.rotation[2] += angle_delta.to_degrees();
                                    }
                                }
                                TransformTool::Scale => {
                                    if let Some(axis) = *drag_axis {
                                        let mouse_delta = glam::Vec2::new(delta.x, delta.y);
                                        
                                        match axis {
                                            0 => { // X Axis
                                                if let Some(p_end) = project(world_pos + right * world_scale) {
                                                    let axis_vec = p_end - p_origin;
                                                    if axis_vec.length_squared() > 0.001 {
                                                        let t = mouse_delta.dot(axis_vec) / axis_vec.length_squared();
                                                        transform_mut.scale[0] = (transform_mut.scale[0] + t).max(0.1);
                                                    }
                                                }
                                            }
                                            1 => { // Y Axis
                                                if let Some(p_end) = project(world_pos + up * world_scale) {
                                                    let axis_vec = p_end - p_origin;
                                                    if axis_vec.length_squared() > 0.001 {
                                                        let t = mouse_delta.dot(axis_vec) / axis_vec.length_squared();
                                                        transform_mut.scale[1] = (transform_mut.scale[1] + t).max(0.1);
                                                    }
                                                }
                                            }
                                            2 => { // Uniform (Center Handle or Z Handle)
                                                // Simple uniform scaling by dragging right/up
                                                let scale_delta = (delta.x - delta.y) * 0.01;
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
                SceneViewMode::Mode2D => {
                    // --------------------------------------------------------
                    // 2D MODE (Legacy logic)
                    // --------------------------------------------------------
                    let move_scale = 1.0 / scene_camera.zoom;
                    // Convert screen delta to world space (invert Y)
                    let screen_delta = glam::Vec2::new(delta.x, -delta.y);
                    let world_delta = screen_delta * move_scale;

                    match current_tool {
                        TransformTool::Move => {
                            if let Some(axis) = *drag_axis {
                                match axis {
                                    0 | 1 => {
                                        // Single axis movement - project onto the axis
                                        match transform_space {
                                            TransformSpace::Local => {
                                                // Local space: use object rotation (Z-axis only in 2D)
                                                let rotation_rad = transform_mut.rotation[2].to_radians();
                                                
                                                // Calculate local axis direction in world space
                                                let local_axis = if axis == 0 {
                                                    // X axis in world space
                                                    glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin())
                                                } else {
                                                    // Y axis in world space (perpendicular to X, rotated 90 deg CCW)
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
                                                    // X axis only
                                                    transform_mut.position[0] += world_delta.x;
                                                } else {
                                                    // Y axis only
                                                    transform_mut.position[1] += world_delta.y;
                                                }
                                            }
                                        }
                                    }
                                    3 => {
                                        // Free movement
                                        transform_mut.position[0] += world_delta.x;
                                        transform_mut.position[1] += world_delta.y;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        TransformTool::Rotate => {
                            if let Some(mouse_pos) = response.interact_pointer_pos() {
                                let center = egui::pos2(screen_x, screen_y);
                                let current_pos = mouse_pos;
                                let prev_pos = current_pos - delta;
                                
                                let current_angle = (current_pos.y - center.y).atan2(current_pos.x - center.x);
                                let prev_angle = (prev_pos.y - center.y).atan2(prev_pos.x - center.x);
                                
                                let mut angle_delta = current_angle - prev_angle;
                                
                                if angle_delta > std::f32::consts::PI { angle_delta -= 2.0 * std::f32::consts::PI; }
                                else if angle_delta < -std::f32::consts::PI { angle_delta += 2.0 * std::f32::consts::PI; }
                                
                                transform_mut.rotation[2] += angle_delta.to_degrees();
                            }
                        }
                        TransformTool::Scale => {
                            let scale_speed = 1.0;
                            if let Some(axis) = *drag_axis {
                                match axis {
                                    0 => { // X axis scale
                                        let rotation_rad = match transform_space {
                                            TransformSpace::Local => transform_mut.rotation[2].to_radians(),
                                            TransformSpace::World => 0.0,
                                        };
                                        let x_axis = glam::Vec2::new(rotation_rad.cos(), rotation_rad.sin());
                                        let scale_delta = world_delta.dot(x_axis) * scale_speed;
                                        transform_mut.scale[0] = (transform_mut.scale[0] + scale_delta).max(0.1);
                                    }
                                    1 => { // Y axis scale
                                        let rotation_rad = match transform_space {
                                            TransformSpace::Local => transform_mut.rotation[2].to_radians(),
                                            TransformSpace::World => 0.0,
                                        };
                                        let y_axis = glam::Vec2::new(-rotation_rad.sin(), rotation_rad.cos());
                                        let scale_delta = world_delta.dot(y_axis) * scale_speed;
                                        transform_mut.scale[1] = (transform_mut.scale[1] + scale_delta).max(0.1);
                                    }
                                    2 => { // Uniform scale
                                        let scale_delta = (world_delta.x + world_delta.y) * 0.5 * scale_speed;
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
    }
}
