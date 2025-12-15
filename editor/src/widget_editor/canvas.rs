//! Prefab Canvas
//! 
//! Visual canvas for editing UI prefabs

use egui;
use super::state::{PrefabEditorState, EditorTool, DragMode};
use ui::prefab::UIPrefabElement;
use engine::runtime::GameViewResolution;

pub struct PrefabCanvas {
    pub resolution: GameViewResolution,
    pub show_grid: bool,
    pub show_safe_area: bool,
    pub show_anchors: bool,
    pub show_pivot: bool,
    pub background_color: egui::Color32,
}

impl PrefabCanvas {
    pub fn new() -> Self {
        Self {
            resolution: GameViewResolution::FullHD,
            show_grid: true,
            show_safe_area: true,
            show_anchors: true,
            show_pivot: true,
            background_color: egui::Color32::from_rgb(40, 40, 45),
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut PrefabEditorState) {
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
        
        // Render prefab elements
        if let Some(prefab) = &state.current_prefab {
            self.render_element(ui, &prefab.root, canvas_rect, state, None);
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
    
    fn render_element(&self, ui: &mut egui::Ui, element: &UIPrefabElement, canvas_rect: egui::Rect, state: &PrefabEditorState, parent_rect: Option<egui::Rect>) {
        let painter = ui.painter();
        
        // Calculate element rect based on RectTransform
        let element_rect = self.calculate_element_rect(element, canvas_rect, parent_rect);
        
        // Render element preview based on components
        if let Some(_image) = &element.image {
            // Render as image (gray box for now)
            let color = element.ui_element.color;
            let egui_color = egui::Color32::from_rgba_unmultiplied(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                (color[3] * element.ui_element.alpha * 255.0) as u8,
            );
            painter.rect_filled(element_rect, 2.0, egui_color);
        } else if let Some(text) = &element.text {
            // Render text
            let text_color = egui::Color32::from_rgba_unmultiplied(
                (text.color[0] * 255.0) as u8,
                (text.color[1] * 255.0) as u8,
                (text.color[2] * 255.0) as u8,
                (text.color[3] * element.ui_element.alpha * 255.0) as u8,
            );
            painter.text(
                element_rect.min,
                egui::Align2::LEFT_TOP,
                &text.text,
                egui::FontId::proportional(text.font_size),
                text_color,
            );
        } else if element.button.is_some() {
            // Render button (gray box with border)
            painter.rect_filled(element_rect, 4.0, egui::Color32::from_gray(80));
            painter.rect_stroke(element_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_gray(120)));
            painter.text(
                element_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Button",
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        } else if element.panel.is_some() {
            // Render panel (light gray box)
            painter.rect_filled(element_rect, 4.0, egui::Color32::from_gray(60));
        } else {
            // Default: gray box
            painter.rect_filled(element_rect, 2.0, egui::Color32::from_gray(80));
        }
        
        // Show anchors if enabled
        if self.show_anchors {
            let anchor_painter = ui.painter();
            self.render_anchors(anchor_painter, element, canvas_rect, parent_rect);
        }
        
        // Show pivot if enabled
        if self.show_pivot {
            let pivot_painter = ui.painter();
            self.render_pivot(pivot_painter, element_rect, &element.rect_transform);
        }
        
        // Selection outline
        if state.selected_element.as_ref() == Some(&element.name) {
            painter.rect_stroke(
                element_rect,
                2.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 150, 255)),
            );
            
            // Selection handles (corners and edges)
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
            
            // Edge handles (for resizing)
            let edge_handles = [
                egui::pos2((element_rect.min.x + element_rect.max.x) / 2.0, element_rect.min.y), // Top
                egui::pos2((element_rect.min.x + element_rect.max.x) / 2.0, element_rect.max.y), // Bottom
                egui::pos2(element_rect.min.x, (element_rect.min.y + element_rect.max.y) / 2.0), // Left
                egui::pos2(element_rect.max.x, (element_rect.min.y + element_rect.max.y) / 2.0), // Right
            ];
            
            for handle in edge_handles {
                painter.circle_filled(handle, handle_size * 0.7, egui::Color32::WHITE);
                painter.circle_stroke(handle, handle_size * 0.7, egui::Stroke::new(1.0, egui::Color32::BLACK));
            }
        }
        
        // Element label
        painter.text(
            element_rect.left_top() - egui::vec2(0.0, 15.0),
            egui::Align2::LEFT_BOTTOM,
            &element.name,
            egui::FontId::proportional(10.0),
            egui::Color32::from_gray(200),
        );
        
        // Render children recursively
        for child in &element.children {
            self.render_element(ui, child, canvas_rect, state, Some(element_rect));
        }
    }
    
    fn calculate_element_rect(&self, element: &UIPrefabElement, canvas_rect: egui::Rect, parent_rect: Option<egui::Rect>) -> egui::Rect {
        let parent = parent_rect.unwrap_or(canvas_rect);
        let rt = &element.rect_transform;
        
        // IMPORTANT: Flip Y-axis to match Unity coordinate system
        // Unity: Y=0 is bottom, Y=1 is top
        // egui: Y=0 is top, Y=1 is bottom
        let flipped_anchor_min_y = 1.0 - rt.anchor_max.y;
        let flipped_anchor_max_y = 1.0 - rt.anchor_min.y;
        
        // Calculate anchor points in parent space (with flipped Y)
        let anchor_min = egui::pos2(
            parent.min.x + parent.width() * rt.anchor_min.x,
            parent.min.y + parent.height() * flipped_anchor_min_y,
        );
        let anchor_max = egui::pos2(
            parent.min.x + parent.width() * rt.anchor_max.x,
            parent.min.y + parent.height() * flipped_anchor_max_y,
        );
        
        // Calculate anchor center
        let anchor_center = egui::pos2(
            (anchor_min.x + anchor_max.x) / 2.0,
            (anchor_min.y + anchor_max.y) / 2.0,
        );
        
        // Flip pivot Y
        let flipped_pivot_y = 1.0 - rt.pivot.y;
        
        // Calculate size
        let size = if rt.anchor_min == rt.anchor_max {
            // Fixed size
            egui::vec2(rt.get_size().x, rt.get_size().y)
        } else {
            // Stretched
            egui::vec2(
                (anchor_max.x - anchor_min.x) + rt.size_delta.x,
                (anchor_max.y - anchor_min.y) + rt.size_delta.y,
            )
        };
        
        // Calculate final position (flip anchored_position Y by subtracting instead of adding)
        let min = egui::pos2(
            anchor_center.x + rt.anchored_position.x - rt.pivot.x * size.x,
            anchor_center.y - rt.anchored_position.y - flipped_pivot_y * size.y,
        );
        
        egui::Rect::from_min_size(min, size)
    }
    
    fn render_anchors(&self, painter: &egui::Painter, element: &UIPrefabElement, canvas_rect: egui::Rect, parent_rect: Option<egui::Rect>) {
        let parent = parent_rect.unwrap_or(canvas_rect);
        let rt = &element.rect_transform;
        
        // Flip Y-axis for anchor visualization
        let flipped_anchor_min_y = 1.0 - rt.anchor_max.y;
        let flipped_anchor_max_y = 1.0 - rt.anchor_min.y;
        
        // Draw anchor points (with flipped Y)
        let anchor_min_pos = egui::pos2(
            parent.min.x + parent.width() * rt.anchor_min.x,
            parent.min.y + parent.height() * flipped_anchor_min_y,
        );
        let anchor_max_pos = egui::pos2(
            parent.min.x + parent.width() * rt.anchor_max.x,
            parent.min.y + parent.height() * flipped_anchor_max_y,
        );
        
        // Draw anchor visualization
        if rt.anchor_min == rt.anchor_max {
            // Single anchor point
            painter.circle_stroke(anchor_min_pos, 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 0)));
        } else {
            // Stretched anchors - draw rectangle
            let anchor_rect = egui::Rect::from_two_pos(anchor_min_pos, anchor_max_pos);
            painter.rect_stroke(anchor_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 0)));
        }
    }
    
    fn render_pivot(&self, painter: &egui::Painter, element_rect: egui::Rect, rt: &ui::RectTransform) {
        // Flip pivot Y for visualization
        let flipped_pivot_y = 1.0 - rt.pivot.y;
        
        // Calculate pivot position in element space (with flipped Y)
        let pivot_pos = egui::pos2(
            element_rect.min.x + element_rect.width() * rt.pivot.x,
            element_rect.min.y + element_rect.height() * flipped_pivot_y,
        );
        
        // Draw pivot point (crosshair)
        let size = 8.0;
        painter.line_segment(
            [egui::pos2(pivot_pos.x - size, pivot_pos.y), egui::pos2(pivot_pos.x + size, pivot_pos.y)],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
        );
        painter.line_segment(
            [egui::pos2(pivot_pos.x, pivot_pos.y - size), egui::pos2(pivot_pos.x, pivot_pos.y + size)],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
        );
        painter.circle_filled(pivot_pos, 3.0, egui::Color32::from_rgb(0, 255, 0));
    }
    
    fn handle_interactions(&mut self, ui: &mut egui::Ui, canvas_rect: egui::Rect, state: &mut PrefabEditorState) {
        let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
        
        // Handle click to select
        if response.clicked() && !state.is_dragging {
            if let Some(prefab) = &state.current_prefab {
                let click_pos = response.interact_pointer_pos().unwrap();
                
                // Check if clicking on a handle first
                if let Some(element) = state.get_selected_element() {
                    let element_rect = self.calculate_element_rect(element, canvas_rect, None);
                    
                    // Check anchor handles
                    if self.show_anchors {
                        if self.is_near_anchor_min(click_pos, element, canvas_rect, None) {
                            // Don't change selection, just prepare for anchor drag
                            return;
                        }
                        if self.is_near_anchor_max(click_pos, element, canvas_rect, None) {
                            return;
                        }
                    }
                    
                    // Check pivot handle
                    if self.show_pivot {
                        if self.is_near_pivot(click_pos, element_rect, &element.rect_transform) {
                            return;
                        }
                    }
                }
                
                // Find clicked element (recursive search)
                let found_element = self.find_element_at_position(&prefab.root, click_pos, canvas_rect, None);
                
                if let Some(name) = found_element {
                    state.select_element(name);
                } else {
                    state.deselect();
                }
            }
        }
        
        // Handle drag start - determine what we're dragging
        if response.drag_started() && state.selected_element.is_some() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                // Determine drag mode and store initial values
                let (drag_mode, start_anchor_min, start_anchor_max, start_pivot, start_size, start_pos) = {
                    if let Some(element) = state.get_selected_element() {
                        let element_rect = self.calculate_element_rect(element, canvas_rect, None);
                        
                        // Check what we're dragging
                        if self.show_anchors && self.is_near_anchor_min(click_pos, element, canvas_rect, None) {
                            (DragMode::AnchorMin, Some(element.rect_transform.anchor_min), None, None, None, None)
                        } else if self.show_anchors && self.is_near_anchor_max(click_pos, element, canvas_rect, None) {
                            (DragMode::AnchorMax, None, Some(element.rect_transform.anchor_max), None, None, None)
                        } else if self.show_pivot && self.is_near_pivot(click_pos, element_rect, &element.rect_transform) {
                            (DragMode::Pivot, None, None, Some(element.rect_transform.pivot), None, None)
                        } else if state.current_tool == EditorTool::Resize {
                            (DragMode::Resize, None, None, None, Some(element.rect_transform.get_size()), None)
                        } else if state.current_tool == EditorTool::Move {
                            (DragMode::Position, None, None, None, None, Some(element.rect_transform.anchored_position))
                        } else {
                            (DragMode::None, None, None, None, None, None)
                        }
                    } else {
                        (DragMode::None, None, None, None, None, None)
                    }
                };
                
                state.drag_mode = drag_mode;
                state.element_start_anchor_min = start_anchor_min;
                state.element_start_anchor_max = start_anchor_max;
                state.element_start_pivot = start_pivot;
                state.element_start_size = start_size;
                state.element_start_pos = start_pos;
                state.is_dragging = true;
                state.drag_start = Some([click_pos.x, click_pos.y]);
            }
        }
        
        // Handle dragging
        if response.dragged() && state.is_dragging {
            if let (Some(drag_start), Some(current_pos)) = 
                (state.drag_start, response.interact_pointer_pos()) {
                
                let delta = ui::Vec2::new(
                    current_pos.x - drag_start[0],
                    current_pos.y - drag_start[1],
                );
                
                match state.drag_mode {
                    DragMode::Position => {
                        if let (Some(element_start), Some(element)) = 
                            (state.element_start_pos, state.get_selected_element_mut()) {
                            element.rect_transform.anchored_position = ui::Vec2::new(
                                element_start.x + delta.x,
                                element_start.y + delta.y,
                            );
                            state.mark_modified();
                        }
                    }
                    DragMode::Resize => {
                        if let (Some(element_start_size), Some(element)) = 
                            (state.element_start_size, state.get_selected_element_mut()) {
                            let new_size = ui::Vec2::new(
                                (element_start_size.x + delta.x).max(10.0),
                                (element_start_size.y + delta.y).max(10.0),
                            );
                            element.rect_transform.set_size(new_size);
                            state.mark_modified();
                        }
                    }
                    DragMode::AnchorMin => {
                        if let (Some(element_start_anchor), Some(element)) = 
                            (state.element_start_anchor_min, state.get_selected_element_mut()) {
                            // Convert delta to normalized parent space
                            let parent_size = canvas_rect.size();
                            let normalized_delta = ui::Vec2::new(
                                delta.x / parent_size.x,
                                delta.y / parent_size.y,
                            );
                            
                            element.rect_transform.anchor_min = ui::Vec2::new(
                                (element_start_anchor.x + normalized_delta.x).clamp(0.0, 1.0),
                                (element_start_anchor.y + normalized_delta.y).clamp(0.0, 1.0),
                            );
                            state.mark_modified();
                        }
                    }
                    DragMode::AnchorMax => {
                        if let (Some(element_start_anchor), Some(element)) = 
                            (state.element_start_anchor_max, state.get_selected_element_mut()) {
                            // Convert delta to normalized parent space
                            let parent_size = canvas_rect.size();
                            let normalized_delta = ui::Vec2::new(
                                delta.x / parent_size.x,
                                delta.y / parent_size.y,
                            );
                            
                            element.rect_transform.anchor_max = ui::Vec2::new(
                                (element_start_anchor.x + normalized_delta.x).clamp(0.0, 1.0),
                                (element_start_anchor.y + normalized_delta.y).clamp(0.0, 1.0),
                            );
                            state.mark_modified();
                        }
                    }
                    DragMode::Pivot => {
                        if let (Some(element_start_pivot), Some(element)) = 
                            (state.element_start_pivot, state.get_selected_element_mut()) {
                            // Convert delta to normalized element space
                            let element_size = element.rect_transform.get_size();
                            let normalized_delta = ui::Vec2::new(
                                delta.x / element_size.x,
                                delta.y / element_size.y,
                            );
                            
                            element.rect_transform.pivot = ui::Vec2::new(
                                (element_start_pivot.x + normalized_delta.x).clamp(0.0, 1.0),
                                (element_start_pivot.y + normalized_delta.y).clamp(0.0, 1.0),
                            );
                            state.mark_modified();
                        }
                    }
                    DragMode::None => {}
                }
            }
        }
        
        // Handle drag end
        if response.drag_stopped() {
            state.is_dragging = false;
            state.drag_mode = DragMode::None;
            state.drag_start = None;
            state.element_start_pos = None;
            state.element_start_size = None;
            state.element_start_anchor_min = None;
            state.element_start_anchor_max = None;
            state.element_start_pivot = None;
        }
    }
    
    fn is_near_anchor_min(&self, pos: egui::Pos2, element: &UIPrefabElement, canvas_rect: egui::Rect, parent_rect: Option<egui::Rect>) -> bool {
        let parent = parent_rect.unwrap_or(canvas_rect);
        let flipped_anchor_min_y = 1.0 - element.rect_transform.anchor_max.y;
        let anchor_pos = egui::pos2(
            parent.min.x + parent.width() * element.rect_transform.anchor_min.x,
            parent.min.y + parent.height() * flipped_anchor_min_y,
        );
        pos.distance(anchor_pos) < 8.0
    }
    
    fn is_near_anchor_max(&self, pos: egui::Pos2, element: &UIPrefabElement, canvas_rect: egui::Rect, parent_rect: Option<egui::Rect>) -> bool {
        let parent = parent_rect.unwrap_or(canvas_rect);
        let flipped_anchor_max_y = 1.0 - element.rect_transform.anchor_min.y;
        let anchor_pos = egui::pos2(
            parent.min.x + parent.width() * element.rect_transform.anchor_max.x,
            parent.min.y + parent.height() * flipped_anchor_max_y,
        );
        pos.distance(anchor_pos) < 8.0
    }
    
    fn is_near_pivot(&self, pos: egui::Pos2, element_rect: egui::Rect, rt: &ui::RectTransform) -> bool {
        let flipped_pivot_y = 1.0 - rt.pivot.y;
        let pivot_pos = egui::pos2(
            element_rect.min.x + element_rect.width() * rt.pivot.x,
            element_rect.min.y + element_rect.height() * flipped_pivot_y,
        );
        pos.distance(pivot_pos) < 8.0
    }
    
    fn find_element_at_position(&self, element: &UIPrefabElement, pos: egui::Pos2, canvas_rect: egui::Rect, parent_rect: Option<egui::Rect>) -> Option<String> {
        // Check children first (render order - last child is on top)
        for child in element.children.iter().rev() {
            let element_rect = self.calculate_element_rect(element, canvas_rect, parent_rect);
            if let Some(found) = self.find_element_at_position(child, pos, canvas_rect, Some(element_rect)) {
                return Some(found);
            }
        }
        
        // Check this element
        let element_rect = self.calculate_element_rect(element, canvas_rect, parent_rect);
        if element_rect.contains(pos) {
            return Some(element.name.clone());
        }
        
        None
    }
}

impl Default for PrefabCanvas {
    fn default() -> Self {
        Self::new()
    }
}
