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
    // Start dragging - determine which handle
    if response.drag_started() {
        if let Some(hover_pos) = response.hover_pos() {
             if let Some(axis) = hit_test_gizmo(
                screen_x,
                screen_y,
                hover_pos,
                current_tool,
                scene_camera,
                scene_view_mode,
                transform_space,
                transform,
                viewport_rect,
            ) {
                *dragging_entity = Some(entity);
                *drag_axis = Some(axis);
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
                                            
                                            // Camera Basis Correction
                                            let yaw = scene_camera.rotation.to_radians();
                                            // Use simple camera basis for movement relative to screen
                                            let cam_right = glam::Vec3::new(yaw.sin(), 0.0, -yaw.cos());
                                            let cam_up = glam::Vec3::Y; // Simplified
                                            
                                            // Screen Y is inverted, so -delta.y moves Up
                                            let displacement = cam_right * delta.x * pixel_conv + cam_up * -delta.y * pixel_conv;
                                            
                                            transform_mut.position[0] += displacement.x;
                                            transform_mut.position[1] += displacement.y;
                                            transform_mut.position[2] += displacement.z;
                                        }
                                    }
                                }
                                TransformTool::Rotate => {
                                    if let Some(axis) = *drag_axis {
                                        let mouse_delta = glam::Vec2::new(delta.x, delta.y);
                                        let sensitivity = 0.5; // Degrees per pixel
                                        
                                        // Determine rotation axis vector
                                        let (rot_axis, _tangent_guess) = match axis {
                                            0 => (right, up),   // Rotate X
                                            1 => (up, right),   // Rotate Y
                                            2 => (forward, right), // Rotate Z
                                            _ => (glam::Vec3::Z, glam::Vec3::X),
                                        };

                                        // Project rotation axis to screen to see alignment
                                        // Rotation happens perpendicular to this screen vector.
                                        let p_axis_end = project(world_pos + rot_axis * world_scale);
                                        
                                        let rot_amount = if let Some(pe) = p_axis_end {
                                            let axis_screen = pe - p_origin;
                                            if axis_screen.length() < 5.0 {
                                                // Axis matches view direction -> Roll behavior
                                                delta.x * sensitivity
                                            } else {
                                                // Mouse movement perpendicular to axis vector on screen drives rotation
                                                let axis_dir = axis_screen.normalize();
                                                let perp_dir = glam::Vec2::new(-axis_dir.y, axis_dir.x); // CCW perpendicular
                                                
                                                // Dot product
                                                let val = mouse_delta.dot(perp_dir);
                                                val * sensitivity
                                            }
                                        } else {
                                            delta.x * sensitivity
                                        };
                                        
                                        // Apply Rotation
                                        let current_quat = glam::Quat::from_euler(
                                            glam::EulerRot::XYZ, 
                                            transform_mut.rotation[0].to_radians(), 
                                            transform_mut.rotation[1].to_radians(), 
                                            transform_mut.rotation[2].to_radians()
                                        );
                                        
                                        let delta_quat = glam::Quat::from_axis_angle(rot_axis, rot_amount.to_radians());
                                        let new_quat = delta_quat * current_quat; // Apply delta
                                        
                                        // Convert back to Euler
                                        let (rx, ry, rz) = new_quat.to_euler(glam::EulerRot::XYZ);
                                        transform_mut.rotation = [rx.to_degrees(), ry.to_degrees(), rz.to_degrees()];
                                    }
                                }
                                TransformTool::Scale => {
                                    if let Some(axis) = *drag_axis {
                                        let mouse_delta = glam::Vec2::new(delta.x, delta.y);
                                        let scale_sensitivity = 0.01;
                                        
                                        match axis {
                                            0 => { // X Axis
                                                if let Some(p_end) = project(world_pos + right * world_scale) {
                                                    let axis_vec = p_end - p_origin;
                                                    if axis_vec.length_squared() > 0.001 {
                                                        let t = mouse_delta.dot(axis_vec) / axis_vec.length_squared();
                                                        transform_mut.scale[0] = (transform_mut.scale[0] + t).max(0.01);
                                                    }
                                                }
                                            }
                                            1 => { // Y Axis
                                                if let Some(p_end) = project(world_pos + up * world_scale) {
                                                    let axis_vec = p_end - p_origin;
                                                    if axis_vec.length_squared() > 0.001 {
                                                        let t = mouse_delta.dot(axis_vec) / axis_vec.length_squared();
                                                        transform_mut.scale[1] = (transform_mut.scale[1] + t).max(0.01);
                                                    }
                                                }
                                            }
                                            2 => { // Z Axis
                                                if let Some(p_end) = project(world_pos + forward * world_scale) {
                                                    let axis_vec = p_end - p_origin;
                                                    if axis_vec.length_squared() > 0.001 {
                                                        let t = mouse_delta.dot(axis_vec) / axis_vec.length_squared();
                                                        // Safety check for Vec3 scale
                                                        if transform_mut.scale.len() > 2 {
                                                            transform_mut.scale[2] = (transform_mut.scale[2] + t).max(0.01); 
                                                        }
                                                    }
                                                }
                                            }
                                            3 => { // Uniform (Center)
                                                // Simple uniform scaling by dragging right/up
                                                let scale_delta = (delta.x - delta.y) * scale_sensitivity;
                                                transform_mut.scale[0] = (transform_mut.scale[0] + scale_delta).max(0.01);
                                                transform_mut.scale[1] = (transform_mut.scale[1] + scale_delta).max(0.01);
                                                if transform_mut.scale.len() > 2 {
                                                     transform_mut.scale[2] = (transform_mut.scale[2] + scale_delta).max(0.01);
                                                }
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

/// Check which gizmo axis is under the cursor
pub fn hit_test_gizmo(
    screen_x: f32,
    screen_y: f32,
    hover_pos: egui::Pos2,
    current_tool: &TransformTool,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    transform_space: &TransformSpace,
    transform: &ecs::Transform,
    viewport_rect: Option<egui::Rect>,
) -> Option<u8> {
    let gizmo_size = 80.0;
    let handle_size = 10.0;
    let center = egui::pos2(screen_x, screen_y);

    match scene_view_mode {
        SceneViewMode::Mode3D => {
            if let Some(rect) = viewport_rect {
                let viewport_size = glam::Vec2::new(rect.width(), rect.height());
                let world_pos = glam::Vec3::from(transform.position);

                let transform3d_temp = projection_3d::Transform3D::new(world_pos, 0.0, glam::Vec2::ONE);
                let cam_dist = transform3d_temp.depth_from_camera(scene_camera);
                let safe_dist = cam_dist.max(0.1);
                let scale = safe_dist * 0.15;

                let project = |pos: glam::Vec3| -> Option<egui::Pos2> {
                    projection_3d::world_to_screen(pos, scene_camera, viewport_size)
                        .map(|p| egui::pos2(rect.min.x + p.x, rect.min.y + p.y))
                };

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

                let p_origin = project(world_pos).unwrap_or(center);
                let p_right = project(world_pos + right * scale);
                let p_up = project(world_pos + up * scale);
                let p_fwd = project(world_pos + forward * scale);

                let dist_center = hover_pos.distance(p_origin);

                match current_tool {
                    TransformTool::Move => {
                        let dist_x = p_right.map_or(f32::MAX, |p| hover_pos.distance(p));
                        let dist_y = p_up.map_or(f32::MAX, |p| hover_pos.distance(p));
                        let dist_z = p_fwd.map_or(f32::MAX, |p| hover_pos.distance(p));

                        let hit_radius = handle_size * 2.5;

                        if dist_center < hit_radius {
                            Some(3) // All/Free
                        } else if dist_x < hit_radius {
                            Some(0) // X
                        } else if dist_y < hit_radius {
                            Some(1) // Y
                        } else if dist_z < hit_radius {
                            Some(2) // Z
                        } else {
                            None
                        }
                    }
                    TransformTool::Rotate => {
                        let radius_world = scale * 4.0;
                        let hit_threshold = 10.0;
                        
                        let get_ring_dist = |axis_u: glam::Vec3, axis_v: glam::Vec3| -> f32 {
                            let segments = 32;
                            let mut min_dist = f32::MAX;
                            for i in 0..segments {
                                let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                                let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                                let p1 = world_pos + (axis_u * a1.cos() + axis_v * a1.sin()) * radius_world;
                                let p2 = world_pos + (axis_u * a2.cos() + axis_v * a2.sin()) * radius_world;
                                
                                if let (Some(s1), Some(s2)) = (project(p1), project(p2)) {
                                    let pa = hover_pos - s1;
                                    let ba = s2 - s1;
                                    let h = (pa.x * ba.x + pa.y * ba.y) / (ba.x * ba.x + ba.y * ba.y);
                                    let h = h.clamp(0.0, 1.0);
                                    let closest = s1 + ba * h;
                                    let d = hover_pos.distance(closest);
                                    if d < min_dist { min_dist = d; }
                                }
                            }
                            min_dist
                        };
                        
                        let dist_x = get_ring_dist(up, forward);
                        let dist_y = get_ring_dist(right, forward);
                        let dist_z = get_ring_dist(right, up);
                        
                        let mut best_dist = hit_threshold;
                        let mut best_axis = None;
                        
                        if dist_x < best_dist { best_dist = dist_x; best_axis = Some(0u8); }
                        if dist_y < best_dist { best_dist = dist_y; best_axis = Some(1u8); }
                        if dist_z < best_dist { best_dist = dist_z; best_axis = Some(2u8); }
                        
                        best_axis
                    }
                    TransformTool::Scale => {
                        let dist_x = p_right.map_or(f32::MAX, |p| hover_pos.distance(p));
                        let dist_y = p_up.map_or(f32::MAX, |p| hover_pos.distance(p));
                        let dist_z = p_fwd.map_or(f32::MAX, |p| hover_pos.distance(p));
                        let hit_radius = handle_size * 2.5;

                        if dist_center < hit_radius {
                            Some(3u8) // Uniform
                        } else if dist_x < hit_radius {
                            Some(0u8) // X
                        } else if dist_y < hit_radius {
                            Some(1u8) // Y
                        } else if dist_z < hit_radius {
                            Some(2u8) // Z
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        SceneViewMode::Mode2D => {
            let rotation_rad = match transform_space {
                TransformSpace::Local => transform.rotation[2].to_radians(),
                TransformSpace::World => 0.0,
            };

            match current_tool {
                TransformTool::Move => {
                    let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
                    let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());
                    
                    let x_handle = egui::pos2(screen_x + x_dir.x * gizmo_size, screen_y + x_dir.y * gizmo_size);
                    let y_handle = egui::pos2(screen_x + y_dir.x * gizmo_size, screen_y + y_dir.y * gizmo_size);
                    
                    let dist_x = hover_pos.distance(x_handle);
                    let dist_y = hover_pos.distance(y_handle);
                    let dist_center = hover_pos.distance(center);
                    
                    if dist_center < handle_size * 1.8 {
                        Some(3)
                    } else if dist_x < handle_size * 1.8 {
                        Some(0)
                    } else if dist_y < handle_size * 1.8 {
                        Some(1)
                    } else {
                        None
                    }
                }
                TransformTool::Rotate => {
                    let radius = gizmo_size * 0.8;
                    let dist_from_center = hover_pos.distance(center);
                    let dist_from_circle = (dist_from_center - radius).abs();
                    
                    if dist_from_circle < 30.0 || dist_from_center < radius {
                        Some(0)
                    } else {
                        None
                    }
                }
                TransformTool::Scale => {
                    let axis_length = gizmo_size;
                    let x_dir = glam::Vec2::new(rotation_rad.cos(), -rotation_rad.sin());
                    let y_dir = glam::Vec2::new(-rotation_rad.sin(), -rotation_rad.cos());
                    
                    let x_handle = egui::pos2(screen_x + x_dir.x * axis_length, screen_y + x_dir.y * axis_length);
                    let y_handle = egui::pos2(screen_x + y_dir.x * axis_length, screen_y + y_dir.y * axis_length);
                    
                    let dist_x = hover_pos.distance(x_handle);
                    let dist_y = hover_pos.distance(y_handle);
                    let dist_center = hover_pos.distance(center);
                    
                    if dist_center < handle_size * 2.5 {
                        Some(2) // Uniform
                    } else if dist_x < handle_size * 2.5 {
                        Some(0)
                    } else if dist_y < handle_size * 2.5 {
                        Some(1)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
}
