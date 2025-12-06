//! Widget Canvas
//! 
//! Visual canvas for editing widgets

use egui;
use super::state::{WidgetEditorState, EditorTool};
use crate::hud::{HudElement, HudElementType};
use crate::runtime::GameViewResolution;

pub struct WidgetCanvas {
    pub resolution: GameViewResolution,
    pub show_grid: bool,
    pub show_safe_area: bool,
    pub background_color: egui::Color32,
}

impl WidgetCanvas {
    pub fn new() -> Self {
        Self {
            resolution: GameViewResolution::FullHD,
            show_grid: true,
            show_safe_area: true,
            background_color: egui::Color32::from_rgb(40, 40, 45),
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut WidgetEditorState) {
        let available_rect = ui.available_rect_before_wrap();
        
        // Calculate canvas rect (centered, maintaining aspect ratio)
        let (target_w, target_h) = self.resolution.get_size();
        let canvas_rect = self.calculate_canvas_rect(available_rect, target_w as f32, target_h as f32);
        
        // Background
        ui.painter().rect_filled(available_rect, 0.0, self.background_color);
        
        // Canvas background (game view area)
        ui.painter().rect_filled(canvas_rect, 0.0, egui::Color32::from_rgb(30, 30, 35));
        
        // Grid
        if self.show_grid {
            self.render_grid(ui, canvas_rect);
        }
        
        // Safe area
        if self.show_safe_area {
            self.render_safe_area(ui, canvas_rect);
        }
        
        // Render HUD elements
        if let Some(hud) = &state.current_hud {
            for element in &hud.elements {
                self.render_element(ui, element, canvas_rect, state);
            }
        }
        
        // Handle interactions
        self.handle_interactions(ui, canvas_rect, state);
        
        // Canvas border
        ui.painter().rect_stroke(
            canvas_rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
        );
        
        // Resolution info
        let info_text = format!("{}\n{}x{}", self.resolution.get_name(), target_w, target_h);
        ui.painter().text(
            canvas_rect.left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            info_text,
            egui::FontId::proportional(12.0),
            egui::Color32::from_gray(150),
        );
    }
    
    fn calculate_canvas_rect(&self, available: egui::Rect, target_w: f32, target_h: f32) -> egui::Rect {
        let target_aspect = target_w / target_h;
        let available_aspect = available.width() / available.height();
        
        let (w, h) = if available_aspect > target_aspect {
            let h = available.height() * 0.9;
            (h * target_aspect, h)
        } else {
            let w = available.width() * 0.9;
            (w, w / target_aspect)
        };
        
        egui::Rect::from_center_size(available.center(), egui::vec2(w, h))
    }
    
    fn render_grid(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let grid_size = 50.0;
        let painter = ui.painter();
        
        // Vertical lines
        let mut x = rect.min.x;
        while x <= rect.max.x {
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
            );
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = rect.min.y;
        while y <= rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
            );
            y += grid_size;
        }
    }
    
    fn render_safe_area(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let margin = 0.05;
        let safe_rect = rect.shrink2(egui::vec2(
            rect.width() * margin,
            rect.height() * margin,
        ));
        
        ui.painter().rect_stroke(
            safe_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 200, 0)),
        );
    }
    
    fn render_element(&self, ui: &mut egui::Ui, element: &HudElement, canvas_rect: egui::Rect, state: &WidgetEditorState) {
        let painter = ui.painter();
        
        // Calculate element position on canvas
        let pos = element.get_screen_position(canvas_rect.width(), canvas_rect.height());
        let element_rect = egui::Rect::from_min_size(
            canvas_rect.min + egui::vec2(pos[0], pos[1]),
            egui::vec2(element.size[0], element.size[1]),
        );
        
        // Render element preview
        match &element.element_type {
            HudElementType::Text { text, font_size, color } | 
            HudElementType::DynamicText { format: text, font_size, color } => {
                let text_color = egui::Color32::from_rgba_unmultiplied(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (color[3] * 255.0) as u8,
                );
                painter.text(
                    element_rect.min,
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(*font_size),
                    text_color,
                );
            }
            HudElementType::HealthBar { color, background_color, .. } |
            HudElementType::ProgressBar { color, background_color, .. } => {
                // Background
                let bg_color = egui::Color32::from_rgba_unmultiplied(
                    (background_color[0] * 255.0) as u8,
                    (background_color[1] * 255.0) as u8,
                    (background_color[2] * 255.0) as u8,
                    (background_color[3] * 255.0) as u8,
                );
                painter.rect_filled(element_rect, 2.0, bg_color);
                
                // Foreground (75% filled for preview)
                let filled_rect = egui::Rect::from_min_size(
                    element_rect.min,
                    egui::vec2(element_rect.width() * 0.75, element_rect.height()),
                );
                let fg_color = egui::Color32::from_rgba_unmultiplied(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (color[3] * 255.0) as u8,
                );
                painter.rect_filled(filled_rect, 2.0, fg_color);
            }
            HudElementType::Minimap { background_color, .. } => {
                let bg_color = egui::Color32::from_rgba_unmultiplied(
                    (background_color[0] * 255.0) as u8,
                    (background_color[1] * 255.0) as u8,
                    (background_color[2] * 255.0) as u8,
                    (background_color[3] * 255.0) as u8,
                );
                painter.rect_filled(element_rect, 4.0, bg_color);
                painter.text(
                    element_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Minimap",
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );
            }
            _ => {
                // Default: gray box
                painter.rect_filled(element_rect, 2.0, egui::Color32::from_gray(80));
            }
        }
        
        // Selection outline
        if state.selected_element.as_ref() == Some(&element.id) {
            painter.rect_stroke(
                element_rect,
                2.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 150, 255)),
            );
            
            // Selection handles (corners)
            let handle_size = 6.0;
            let corners = [
                element_rect.left_top(),
                element_rect.right_top(),
                element_rect.left_bottom(),
                element_rect.right_bottom(),
            ];
            
            for corner in corners {
                painter.circle_filled(corner, handle_size, egui::Color32::WHITE);
                painter.circle_stroke(corner, handle_size, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
        
        // Element label
        painter.text(
            element_rect.left_top() - egui::vec2(0.0, 15.0),
            egui::Align2::LEFT_BOTTOM,
            &element.id,
            egui::FontId::proportional(10.0),
            egui::Color32::from_gray(200),
        );
    }
    
    fn handle_interactions(&mut self, ui: &mut egui::Ui, canvas_rect: egui::Rect, state: &mut WidgetEditorState) {
        let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
        
        // Handle click to select
        if response.clicked() {
            if let Some(hud) = &state.current_hud {
                let click_pos = response.interact_pointer_pos().unwrap();
                
                // Find clicked element
                let mut found_element: Option<String> = None;
                for element in hud.elements.iter().rev() {
                    let pos = element.get_screen_position(canvas_rect.width(), canvas_rect.height());
                    let element_rect = egui::Rect::from_min_size(
                        canvas_rect.min + egui::vec2(pos[0], pos[1]),
                        egui::vec2(element.size[0], element.size[1]),
                    );
                    
                    if element_rect.contains(click_pos) {
                        found_element = Some(element.id.clone());
                        break;
                    }
                }
                
                if let Some(id) = found_element {
                    state.select_element(id);
                } else {
                    state.deselect();
                }
            }
        }
        
        if let Some(hud) = &mut state.current_hud {
            
            // Handle drag to move
            if state.current_tool == EditorTool::Move && state.selected_element.is_some() {
                if response.drag_started() {
                    state.is_dragging = true;
                    state.drag_start = response.interact_pointer_pos().map(|p| [p.x, p.y]);
                    
                    // Store element start position
                    if let Some(element) = state.get_selected_element() {
                        state.element_start_pos = Some(element.offset);
                    }
                }
                
                if response.dragged() && state.is_dragging {
                    if let (Some(drag_start), Some(element_start), Some(current_pos)) = 
                        (state.drag_start, state.element_start_pos, response.interact_pointer_pos()) {
                        
                        let delta = [
                            current_pos.x - drag_start[0],
                            current_pos.y - drag_start[1],
                        ];
                        
                        // Update element position
                        if let Some(element) = state.get_selected_element_mut() {
                            element.offset = [
                                element_start[0] + delta[0],
                                element_start[1] + delta[1],
                            ];
                            state.mark_modified();
                        }
                    }
                }
                
                if response.drag_released() {
                    state.is_dragging = false;
                    state.drag_start = None;
                    state.element_start_pos = None;
                }
            }
        }
    }
}

impl Default for WidgetCanvas {
    fn default() -> Self {
        Self::new()
    }
}
