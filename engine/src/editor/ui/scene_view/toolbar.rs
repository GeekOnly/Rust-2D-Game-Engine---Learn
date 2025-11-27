//! Toolbar
//!
//! Scene view toolbar UI (tools, mode switches, play/stop buttons).

use egui;
use crate::editor::ui::TransformTool;
use super::types::*;

/// Render scene toolbar
pub fn render_scene_toolbar(
    ui: &mut egui::Ui,
    current_tool: &mut TransformTool,
    is_playing: bool,
    play_request: &mut bool,
    stop_request: &mut bool,
    scene_view_mode: &mut SceneViewMode,
    transform_space: &mut TransformSpace,
) {
    ui.horizontal(|ui| {
        // Transform tools
        ui.selectable_value(current_tool, TransformTool::View, "🖐 View (Q)");
        ui.selectable_value(current_tool, TransformTool::Move, "➕ Move (W)");
        ui.selectable_value(current_tool, TransformTool::Rotate, "🔄 Rotate (E)");
        ui.selectable_value(current_tool, TransformTool::Scale, "📏 Scale (R)");
        
        ui.separator();
        
        // 2D/3D toggle
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode2D, "2D");
        ui.selectable_value(scene_view_mode, SceneViewMode::Mode3D, "3D");
        
        ui.separator();
        
        // Pivot/Center toggle
        ui.label("Pivot: Center");
        
        ui.separator();
        
        // Space: Local/World toggle
        ui.label("Space:");
        ui.selectable_value(transform_space, TransformSpace::Local, "Local");
        ui.selectable_value(transform_space, TransformSpace::World, "World");
        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Play/Stop buttons
            if !is_playing {
                if ui.button("▶ Play").clicked() {
                    *play_request = true;
                }
            } else {
                if ui.button("⏹ Stop").clicked() {
                    *stop_request = true;
                }
            }
        });
    });
    ui.separator();
}
