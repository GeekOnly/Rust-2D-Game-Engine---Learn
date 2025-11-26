/// Unity-like toolbar with tool buttons
use egui::{Color32, Rect, Response, Sense, Stroke, Vec2};
use crate::editor::ui::TransformTool;

pub struct Toolbar;

impl Toolbar {
    /// Render Unity-like toolbar at the top of the scene view
    pub fn render(
        ui: &mut egui::Ui,
        current_tool: &mut TransformTool,
        is_playing: bool,
        play_request: &mut bool,
        stop_request: &mut bool,
    ) {
        let colors = super::theme::UnityTheme::colors();
        
        // Toolbar background
        let toolbar_height = 32.0;
        let toolbar_rect = Rect::from_min_size(
            ui.cursor().min,
            Vec2::new(ui.available_width(), toolbar_height),
        );
        
        ui.painter().rect_filled(
            toolbar_rect,
            0.0,
            colors.toolbar_bg,
        );
        
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            
            // Transform tools (left side)
            ui.label("🔧");
            ui.add_space(4.0);
            
            // View tool (Q) - Hand icon
            if Self::tool_button(ui, "👁", *current_tool == TransformTool::View, "View (Q)").clicked() {
                *current_tool = TransformTool::View;
            }
            
            // Move tool (W) - Arrows icon
            if Self::tool_button(ui, "✥", *current_tool == TransformTool::Move, "Move (W)").clicked() {
                *current_tool = TransformTool::Move;
            }
            
            // Rotate tool (E) - Rotate icon
            if Self::tool_button(ui, "↻", *current_tool == TransformTool::Rotate, "Rotate (E)").clicked() {
                *current_tool = TransformTool::Rotate;
            }
            
            // Scale tool (R) - Scale icon
            if Self::tool_button(ui, "⊡", *current_tool == TransformTool::Scale, "Scale (R)").clicked() {
                *current_tool = TransformTool::Scale;
            }
            
            ui.separator();
            
            // Pivot/Center toggle (placeholder)
            ui.label("Pivot:");
            if ui.small_button("Center").clicked() {
                // TODO: Toggle pivot mode
            }
            
            ui.separator();
            
            // Local/Global toggle (placeholder)
            ui.label("Space:");
            if ui.small_button("Local").clicked() {
                // TODO: Toggle space mode
            }
            
            // Spacer to push play controls to center
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                
                // Play controls (center)
                if !is_playing {
                    if Self::play_button(ui, "▶", "Play (Ctrl+P)").clicked() {
                        *play_request = true;
                    }
                } else {
                    if Self::play_button(ui, "⏸", "Pause").clicked() {
                        // TODO: Pause
                    }
                    if Self::stop_button(ui, "⏹", "Stop").clicked() {
                        *stop_request = true;
                    }
                }
            });
        });
        
        ui.add_space(4.0);
    }
    
    /// Tool button (square button with icon)
    fn tool_button(ui: &mut egui::Ui, icon: &str, selected: bool, tooltip: &str) -> Response {
        let colors = super::theme::UnityTheme::colors();
        let size = Vec2::splat(28.0);
        
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        
        if ui.is_rect_visible(rect) {
            let _visuals = ui.style().interact(&response);
            
            // Background
            let bg_color = if selected {
                colors.selected
            } else if response.hovered() {
                colors.bg_light
            } else {
                colors.bg_medium
            };
            
            ui.painter().rect_filled(rect, 2.0, bg_color);
            
            // Border
            if selected {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, colors.accent_hover));
            } else {
                ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, colors.border));
            }
            
            // Icon
            let text_color = if selected {
                Color32::WHITE
            } else {
                colors.text
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                text_color,
            );
        }
        
        response.on_hover_text(tooltip)
    }
    
    /// Play button (green)
    fn play_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> Response {
        let size = Vec2::new(40.0, 28.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        
        if ui.is_rect_visible(rect) {
            let bg_color = if response.hovered() {
                Color32::from_rgb(80, 180, 80)
            } else {
                Color32::from_rgb(60, 160, 60)
            };
            
            ui.painter().rect_filled(rect, 2.0, bg_color);
            ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(40, 140, 40)));
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                Color32::WHITE,
            );
        }
        
        response.on_hover_text(tooltip)
    }
    
    /// Stop button (red)
    fn stop_button(ui: &mut egui::Ui, icon: &str, tooltip: &str) -> Response {
        let size = Vec2::new(40.0, 28.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        
        if ui.is_rect_visible(rect) {
            let bg_color = if response.hovered() {
                Color32::from_rgb(200, 80, 80)
            } else {
                Color32::from_rgb(180, 60, 60)
            };
            
            ui.painter().rect_filled(rect, 2.0, bg_color);
            ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(160, 40, 40)));
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                Color32::WHITE,
            );
        }
        
        response.on_hover_text(tooltip)
    }
}
