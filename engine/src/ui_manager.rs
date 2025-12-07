//! UI System Manager
//!
//! Manages the new UI system integration with the engine.
//! This replaces the old HUD system with the comprehensive UI crate.
//!
//! Note: The UI system uses its own entity management separate from the engine's ECS.
//! Full integration will be completed in future updates.

use ecs::World;
use ui::{UIPrefab, UIPrefabElement};
use std::collections::HashMap;

/// UI System Manager - coordinates all UI systems
pub struct UIManager {
    /// Loaded UI prefabs (path -> prefab)
    loaded_prefabs: HashMap<String, UIPrefab>,
    
    /// Active UI instances (name -> prefab)
    active_uis: HashMap<String, UIPrefab>,
    
    /// UI data for dynamic updates (element_path -> value)
    /// element_path format: "prefab_name/element_name"
    ui_data: HashMap<String, String>,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            loaded_prefabs: HashMap::new(),
            active_uis: HashMap::new(),
            ui_data: HashMap::new(),
        }
    }

    /// Load a UI prefab from file
    pub fn load_prefab(&mut self, path: &str) -> Result<(), String> {
        let file_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read prefab file {}: {}", path, e))?;
        
        let prefab: UIPrefab = serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse prefab {}: {}", path, e))?;
        
        log::info!("Loaded UI prefab: {} from {}", prefab.name, path);
        self.loaded_prefabs.insert(path.to_string(), prefab);
        Ok(())
    }

    /// Activate a loaded prefab (make it visible)
    pub fn activate_prefab(&mut self, path: &str, instance_name: &str) -> Result<(), String> {
        let prefab = self.loaded_prefabs.get(path)
            .ok_or_else(|| format!("Prefab not loaded: {}", path))?
            .clone();
        
        log::info!("Activated UI prefab: {} as {}", prefab.name, instance_name);
        self.active_uis.insert(instance_name.to_string(), prefab);
        Ok(())
    }

    /// Deactivate a UI instance
    pub fn deactivate_prefab(&mut self, instance_name: &str) {
        self.active_uis.remove(instance_name);
        log::info!("Deactivated UI: {}", instance_name);
    }

    /// Update UI element data (for dynamic text, values, etc.)
    pub fn set_ui_data(&mut self, element_path: &str, value: String) {
        self.ui_data.insert(element_path.to_string(), value);
    }

    /// Get UI element data
    pub fn get_ui_data(&self, element_path: &str) -> Option<&String> {
        self.ui_data.get(element_path)
    }

    /// Update all UI systems
    pub fn update(&mut self, _world: &mut World, _dt: f32, _screen_size: (u32, u32)) {
        // Future: Update animations, layouts, etc.
    }

    /// Render UI (to be called during game view rendering)
    pub fn render(&mut self, ui: &mut egui::Ui, _world: &World, rect: egui::Rect) {
        // Debug: Log when render is called
        if !self.active_uis.is_empty() {
            log::debug!("UIManager::render called with {} active UIs", self.active_uis.len());
        }
        
        // Render all active UI instances
        for (instance_name, prefab) in &self.active_uis {
            log::debug!("Rendering UI instance: {}", instance_name);
            self.render_prefab(ui, rect, instance_name, prefab);
        }
    }

    /// Render a single prefab
    fn render_prefab(&self, ui: &mut egui::Ui, screen_rect: egui::Rect, instance_name: &str, prefab: &UIPrefab) {
        let painter = ui.painter_at(screen_rect);
        
        // Render root and all children recursively
        self.render_element(
            &painter,
            screen_rect,
            instance_name,
            &prefab.root,
            screen_rect.size(),
        );
    }

    /// Render a UI element recursively
    fn render_element(
        &self,
        painter: &egui::Painter,
        parent_rect: egui::Rect,
        instance_name: &str,
        element: &UIPrefabElement,
        canvas_size: egui::Vec2,
    ) {
        // Calculate element rect based on RectTransform
        let element_rect = self.calculate_rect(parent_rect, &element.rect_transform, canvas_size);
        
        // Debug: Log element position (use RUST_LOG=debug to see)
        log::debug!(
            "Element '{}': anchor=[{:.1},{:.1}]->[{:.1},{:.1}], pos=[{:.1},{:.1}], rect={:?}",
            element.name,
            element.rect_transform.anchor_min.x,
            element.rect_transform.anchor_min.y,
            element.rect_transform.anchor_max.x,
            element.rect_transform.anchor_max.y,
            element.rect_transform.anchored_position.x,
            element.rect_transform.anchored_position.y,
            element_rect
        );
        
        // Render background image if present
        if let Some(image) = &element.image {
            let color = egui::Color32::from_rgba_unmultiplied(
                (element.ui_element.color[0] * 255.0) as u8,
                (element.ui_element.color[1] * 255.0) as u8,
                (element.ui_element.color[2] * 255.0) as u8,
                (element.ui_element.color[3] * element.ui_element.alpha * 255.0) as u8,
            );
            
            // For filled images, adjust width based on fill_amount
            let render_rect = if matches!(image.image_type, ui::ImageType::Filled) {
                let fill_width = element_rect.width() * image.fill_amount;
                egui::Rect::from_min_size(
                    element_rect.min,
                    egui::vec2(fill_width, element_rect.height()),
                )
            } else {
                element_rect
            };
            
            painter.rect_filled(render_rect, 2.0, color);
        }
        
        // Render text if present
        if let Some(text) = &element.text {
            let element_path = format!("{}/{}", instance_name, element.name);
            let display_text = self.ui_data.get(&element_path)
                .map(|s| s.as_str())
                .unwrap_or(&text.text);
            
            let color = egui::Color32::from_rgba_unmultiplied(
                (text.color[0] * 255.0) as u8,
                (text.color[1] * 255.0) as u8,
                (text.color[2] * 255.0) as u8,
                (text.color[3] * element.ui_element.alpha * 255.0) as u8,
            );
            
            let align = match text.alignment {
                ui::TextAlignment::TopLeft | ui::TextAlignment::MiddleLeft | ui::TextAlignment::BottomLeft => egui::Align2::LEFT_CENTER,
                ui::TextAlignment::TopCenter | ui::TextAlignment::MiddleCenter | ui::TextAlignment::BottomCenter => egui::Align2::CENTER_CENTER,
                ui::TextAlignment::TopRight | ui::TextAlignment::MiddleRight | ui::TextAlignment::BottomRight => egui::Align2::RIGHT_CENTER,
            };
            
            painter.text(
                element_rect.center(),
                align,
                display_text,
                egui::FontId::proportional(text.font_size),
                color,
            );
        }
        
        // Render children
        for child in &element.children {
            self.render_element(painter, element_rect, instance_name, child, canvas_size);
        }
    }

    /// Calculate screen rect from RectTransform (Unity-style)
    fn calculate_rect(
        &self,
        parent_rect: egui::Rect,
        transform: &ui::RectTransform,
        _canvas_size: egui::Vec2,
    ) -> egui::Rect {
        let parent_size = parent_rect.size();
        
        // Unity uses bottom-up Y (0=bottom, 1=top)
        // egui uses top-down Y (0=top, 1=bottom)
        // So we need to flip the Y anchors
        let flipped_anchor_min_y = 1.0 - transform.anchor_max.y;
        let flipped_anchor_max_y = 1.0 - transform.anchor_min.y;
        
        // Calculate anchor points in parent space (Unity-style with Y-flip)
        let anchor_min_pos = egui::pos2(
            parent_rect.min.x + parent_size.x * transform.anchor_min.x,
            parent_rect.min.y + parent_size.y * flipped_anchor_min_y,
        );
        
        let anchor_max_pos = egui::pos2(
            parent_rect.min.x + parent_size.x * transform.anchor_max.x,
            parent_rect.min.y + parent_size.y * flipped_anchor_max_y,
        );
        
        // Calculate the rect between anchors
        let anchored_rect = egui::Rect::from_min_max(anchor_min_pos, anchor_max_pos);
        
        // Apply size_delta (offset from anchored size)
        // If anchors are the same point, size_delta defines the full size
        // If anchors are different, size_delta is added to the anchored size
        let final_size = egui::vec2(
            anchored_rect.width() + transform.size_delta.x,
            anchored_rect.height() + transform.size_delta.y,
        );
        
        // Calculate the center of the anchored rect
        let anchor_center = anchored_rect.center();
        
        // Apply anchored_position (offset from anchor center)
        // Flip Y offset because Unity uses bottom-up Y
        let offset_center = egui::pos2(
            anchor_center.x + transform.anchored_position.x,
            anchor_center.y - transform.anchored_position.y,  // Flip Y
        );
        
        // Apply pivot to determine the actual position
        // Unity pivot: (0,0)=bottom-left, (0.5,0.5)=center, (1,1)=top-right
        // egui origin: top-left
        // So we need to flip pivot.y: Unity (0,0)=bottom-left → egui (0,1)=bottom-left
        let flipped_pivot_y = 1.0 - transform.pivot.y;
        
        let pivot_offset = egui::vec2(
            -final_size.x * transform.pivot.x,
            -final_size.y * flipped_pivot_y,
        );
        
        let final_min = egui::pos2(
            offset_center.x + pivot_offset.x,
            offset_center.y + pivot_offset.y,
        );
        
        egui::Rect::from_min_size(final_min, final_size)
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}
