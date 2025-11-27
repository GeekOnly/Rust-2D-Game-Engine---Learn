//! Keyboard Shortcuts
//!
//! Keyboard shortcut handlers for scene view.

use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
use super::types::*;

/// Handle keyboard shortcuts for tools and camera views
pub fn handle_keyboard_shortcuts(
    ui: &mut egui::Ui,
    current_tool: &mut TransformTool,
    scene_camera: &mut SceneCamera,
    scene_view_mode: &SceneViewMode,
) {
    // Tool shortcuts (Unity-like: Q, W, E, R)
    if ui.input(|i| i.key_pressed(egui::Key::Q)) {
        *current_tool = TransformTool::View;
    }
    if ui.input(|i| i.key_pressed(egui::Key::W)) {
        *current_tool = TransformTool::Move;
    }
    if ui.input(|i| i.key_pressed(egui::Key::E)) {
        *current_tool = TransformTool::Rotate;
    }
    if ui.input(|i| i.key_pressed(egui::Key::R)) {
        *current_tool = TransformTool::Scale;
    }
    
    // Camera view shortcuts (only in 3D mode)
    if *scene_view_mode == SceneViewMode::Mode3D {
        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);
        
        // Numpad 1 - Front/Back view
        if ui.input(|i| i.key_pressed(egui::Key::Num1)) {
            if ctrl_pressed {
                scene_camera.set_view_back();
            } else {
                scene_camera.set_view_front();
            }
        }
        
        // Numpad 3 - Right/Left view
        if ui.input(|i| i.key_pressed(egui::Key::Num3)) {
            if ctrl_pressed {
                scene_camera.set_view_left();
            } else {
                scene_camera.set_view_right();
            }
        }
        
        // Numpad 7 - Top/Bottom view
        if ui.input(|i| i.key_pressed(egui::Key::Num7)) {
            if ctrl_pressed {
                scene_camera.set_view_bottom();
            } else {
                scene_camera.set_view_top();
            }
        }
        
        // Numpad 0 - Perspective view
        if ui.input(|i| i.key_pressed(egui::Key::Num0)) {
            scene_camera.set_view_perspective();
        }
    }
}
