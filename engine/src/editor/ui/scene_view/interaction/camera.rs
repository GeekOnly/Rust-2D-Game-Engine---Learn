//! Camera Controls
//!
//! Camera interaction handlers (pan, orbit, zoom, preset views).

use ecs::{World, Entity};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;

/// Handle camera controls (pan, orbit, zoom)
pub fn handle_camera_controls(
    response: &egui::Response,
    scene_camera: &mut SceneCamera,
    rect: egui::Rect,
    scene_view_mode: &SceneViewMode,
    selected_entity: &Option<Entity>,
    world: &World,
) {
    let is_alt_pressed = response.ctx.input(|i| i.modifiers.alt);

    // === UNITY-LIKE CAMERA CONTROLS ===

    // Alt + Left Mouse Button - Orbit around pivot point (3D mode only)
    if *scene_view_mode == SceneViewMode::Mode3D && is_alt_pressed {
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if response.drag_started_by(egui::PointerButton::Primary) {
                    // Get pivot point from selected entity or use camera position
                    let pivot = if let Some(entity) = selected_entity {
                        if let Some(transform) = world.transforms.get(entity) {
                            glam::Vec2::new(transform.x(), transform.y())
                        } else {
                            scene_camera.position
                        }
                    } else {
                        scene_camera.position
                    };
                    scene_camera.start_orbit(glam::Vec2::new(mouse_pos.x, mouse_pos.y), pivot);
                } else {
                    scene_camera.update_orbit(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            }
        } else {
            scene_camera.stop_orbit();
        }
    }

    // Right Mouse Button - Free look / Fly camera (3D mode)
    // In 2D mode - also enables panning
    if response.dragged_by(egui::PointerButton::Secondary) && !is_alt_pressed {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            if *scene_view_mode == SceneViewMode::Mode3D {
                // 3D mode: Right mouse = rotate camera (free look)
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    scene_camera.start_rotate(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                } else {
                    scene_camera.update_rotate(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            } else {
                // 2D mode: Right mouse = pan (Unity 2D behavior)
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    scene_camera.start_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                } else {
                    scene_camera.update_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
                }
            }
        }
    } else if *scene_view_mode == SceneViewMode::Mode3D {
        scene_camera.stop_rotate();
    }

    // Middle Mouse Button - Pan camera (works in both 2D and 3D modes)
    if response.dragged_by(egui::PointerButton::Middle) {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            if response.drag_started_by(egui::PointerButton::Middle) {
                scene_camera.start_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
            } else {
                scene_camera.update_pan(glam::Vec2::new(mouse_pos.x, mouse_pos.y));
            }
        }
    } else if !response.dragged_by(egui::PointerButton::Secondary) || *scene_view_mode == SceneViewMode::Mode3D {
        scene_camera.stop_pan();
    }

    // Scroll Wheel - Zoom (Unity-like smooth zoom)
    // Only zoom if mouse is hovering over the scene view
    if response.hovered() {
        let scroll_delta = response.ctx.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.1 {
                i.smooth_scroll_delta.y
            } else {
                i.raw_scroll_delta.y * 0.1
            }
        });

        if scroll_delta.abs() > 0.1 {
            let mouse_pos = response.hover_pos().unwrap_or(rect.center());
            let zoom_direction = if scroll_delta > 0.0 { 1.0 } else { -1.0 };
            
            // Convert absolute screen position to relative position from rect center
            let center = rect.center();
            let relative_pos = glam::Vec2::new(
                mouse_pos.x - center.x,
                mouse_pos.y - center.y
            );
            
            scene_camera.zoom(zoom_direction, relative_pos);
        }
    }
}

/// Handle clicks on gizmo axes to snap camera to preset views
pub fn handle_gizmo_axis_clicks(
    ui: &mut egui::Ui,
    center_x: f32,
    center_y: f32,
    gizmo_size: f32,
    scene_camera: &mut SceneCamera,
) {
    let gizmo_center = egui::pos2(center_x, center_y);
    let axis_len = gizmo_size * 0.35;
    let click_radius = 12.0; // Clickable area around axis endpoint

    // Get camera rotation for current axis positions
    let yaw_rad = scene_camera.get_rotation_radians();
    let pitch_rad = scene_camera.get_pitch_radians();

    // Calculate axis endpoint positions (same as in render function)
    let x_dir = (yaw_rad.cos(), yaw_rad.sin());
    let x_end = egui::pos2(
        gizmo_center.x + x_dir.0 * axis_len,
        gizmo_center.y + x_dir.1 * axis_len,
    );

    let y_offset = pitch_rad.cos() * axis_len;
    let y_end = egui::pos2(gizmo_center.x, gizmo_center.y - y_offset);

    let z_dir = (-yaw_rad.sin(), yaw_rad.cos());
    let z_end = egui::pos2(
        gizmo_center.x + z_dir.0 * axis_len,
        gizmo_center.y + z_dir.1 * axis_len,
    );

    // Create invisible clickable areas for each axis
    let x_rect = egui::Rect::from_center_size(x_end, egui::vec2(click_radius * 2.0, click_radius * 2.0));
    let y_rect = egui::Rect::from_center_size(y_end, egui::vec2(click_radius * 2.0, click_radius * 2.0));
    let z_rect = egui::Rect::from_center_size(z_end, egui::vec2(click_radius * 2.0, click_radius * 2.0));

    // X axis click (Right view)
    let x_response = ui.allocate_rect(x_rect, egui::Sense::click());
    if x_response.clicked() {
        scene_camera.rotation = 90.0;  // Look from +X axis
        scene_camera.pitch = 0.0;
    }

    // Y axis click (Top view)
    let y_response = ui.allocate_rect(y_rect, egui::Sense::click());
    if y_response.clicked() {
        scene_camera.rotation = 0.0;
        scene_camera.pitch = 90.0;  // Look from +Y axis (top)
    }

    // Z axis click (Front view)
    let z_response = ui.allocate_rect(z_rect, egui::Sense::click());
    if z_response.clicked() {
        scene_camera.rotation = 0.0;   // Look from +Z axis
        scene_camera.pitch = 0.0;
    }

    // Also handle clicks on opposite directions (click axis label with Shift for opposite view)
    if ui.input(|i| i.modifiers.shift) {
        if x_response.clicked() {
            scene_camera.rotation = -90.0; // Look from -X axis (left)
            scene_camera.pitch = 0.0;
        }
        if y_response.clicked() {
            scene_camera.rotation = 0.0;
            scene_camera.pitch = -90.0; // Look from -Y axis (bottom)
        }
        if z_response.clicked() {
            scene_camera.rotation = 180.0; // Look from -Z axis (back)
            scene_camera.pitch = 0.0;
        }
    }
}
