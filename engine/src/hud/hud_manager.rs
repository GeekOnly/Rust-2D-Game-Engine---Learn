//! HUD Manager
//! 
//! Manages HUD state, data bindings, and rendering

use super::hud_asset::{HudAsset, HudElement, HudElementType};
use ecs::World;
use std::collections::HashMap;

pub type BindingFunction = Box<dyn Fn(&World) -> f32>;

/// HUD Manager - manages HUD state and data bindings
pub struct HudManager {
    current_hud: Option<HudAsset>,
    bindings: HashMap<String, BindingFunction>,
    cached_values: HashMap<String, f32>,
}

impl HudManager {
    pub fn new() -> Self {
        Self {
            current_hud: None,
            bindings: HashMap::new(),
            cached_values: HashMap::new(),
        }
    }
    
    /// Load HUD from file
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let hud = HudAsset::load(path)?;
        self.current_hud = Some(hud);
        Ok(())
    }
    
    /// Set HUD asset directly
    pub fn set_hud(&mut self, hud: HudAsset) {
        self.current_hud = Some(hud);
    }
    
    /// Get current HUD
    pub fn get_hud(&self) -> Option<&HudAsset> {
        self.current_hud.as_ref()
    }
    
    /// Get mutable HUD
    pub fn get_hud_mut(&mut self) -> Option<&mut HudAsset> {
        self.current_hud.as_mut()
    }
    
    /// Clear current HUD
    pub fn clear(&mut self) {
        self.current_hud = None;
        self.cached_values.clear();
    }
    
    /// Bind a data source to a key
    pub fn bind(&mut self, key: impl Into<String>, getter: impl Fn(&World) -> f32 + 'static) {
        self.bindings.insert(key.into(), Box::new(getter));
    }
    
    /// Update cached values from world
    pub fn update(&mut self, world: &World) {
        for (key, getter) in &self.bindings {
            let value = getter(world);
            self.cached_values.insert(key.clone(), value);
        }
    }
    
    /// Get cached value
    pub fn get_value(&self, key: &str) -> Option<f32> {
        self.cached_values.get(key).copied()
    }
    
    /// Set element visibility
    pub fn set_element_visible(&mut self, element_id: &str, visible: bool) {
        if let Some(hud) = &mut self.current_hud {
            for element in &mut hud.elements {
                if element.id == element_id {
                    element.visible = visible;
                    break;
                }
            }
        }
    }
    
    /// Render HUD using egui
    pub fn render_egui(&self, ctx: &egui::Context, world: &World, screen_width: f32, screen_height: f32) {
        if let Some(hud) = &self.current_hud {
            for element in &hud.elements {
                if !element.visible {
                    continue;
                }
                self.render_element_egui(ctx, element, world, screen_width, screen_height);
            }
        }
    }
    
    /// Render HUD clipped to a specific rect (for Game View)
    pub fn render_egui_clipped(&self, ui: &mut egui::Ui, world: &World, screen_width: f32, screen_height: f32, clip_rect: egui::Rect) {
        if let Some(hud) = &self.current_hud {
            // Set clip rect to limit rendering to game view
            ui.set_clip_rect(clip_rect);
            
            for element in &hud.elements {
                if !element.visible {
                    continue;
                }
                
                // Calculate position relative to clip rect
                let pos = element.get_screen_position(screen_width, screen_height);
                let absolute_pos = [
                    clip_rect.min.x + pos[0],
                    clip_rect.min.y + pos[1],
                ];
                
                self.render_element_egui_at(ui.ctx(), element, world, absolute_pos, element.size);
            }
        }
    }
    
    fn render_element_egui_at(
        &self,
        ctx: &egui::Context,
        element: &HudElement,
        world: &World,
        pos: [f32; 2],
        size: [f32; 2],
    ) {
        match &element.element_type {
            HudElementType::HealthBar { binding, color, background_color } => {
                self.render_health_bar_egui(ctx, &element.id, pos, size, binding, *color, *background_color);
            }
            HudElementType::ProgressBar { binding, color, background_color } => {
                self.render_progress_bar_egui(ctx, &element.id, pos, size, binding, *color, *background_color);
            }
            HudElementType::Text { text, font_size, color } => {
                self.render_text_egui(ctx, &element.id, pos, text, *font_size, *color);
            }
            HudElementType::DynamicText { format, font_size, color } => {
                let text = self.format_text(format);
                self.render_text_egui(ctx, &element.id, pos, &text, *font_size, *color);
            }
            HudElementType::Minimap { zoom, background_color } => {
                self.render_minimap_egui(ctx, &element.id, pos, size, *zoom, *background_color, world);
            }
            HudElementType::Image { .. } => {
                // TODO: Implement image rendering
            }
            HudElementType::Container { children } => {
                for child in children {
                    let child_pos = child.get_screen_position(size[0], size[1]);
                    let absolute_child_pos = [pos[0] + child_pos[0], pos[1] + child_pos[1]];
                    self.render_element_egui_at(ctx, child, world, absolute_child_pos, child.size);
                }
            }
        }
    }
    
    fn render_element_egui(
        &self,
        ctx: &egui::Context,
        element: &HudElement,
        world: &World,
        screen_width: f32,
        screen_height: f32,
    ) {
        let pos = element.get_screen_position(screen_width, screen_height);
        
        match &element.element_type {
            HudElementType::HealthBar { binding, color, background_color } => {
                self.render_health_bar_egui(ctx, &element.id, pos, element.size, binding, *color, *background_color);
            }
            HudElementType::ProgressBar { binding, color, background_color } => {
                self.render_progress_bar_egui(ctx, &element.id, pos, element.size, binding, *color, *background_color);
            }
            HudElementType::Text { text, font_size, color } => {
                self.render_text_egui(ctx, &element.id, pos, text, *font_size, *color);
            }
            HudElementType::DynamicText { format, font_size, color } => {
                let text = self.format_text(format);
                self.render_text_egui(ctx, &element.id, pos, &text, *font_size, *color);
            }
            HudElementType::Minimap { zoom, background_color } => {
                self.render_minimap_egui(ctx, &element.id, pos, element.size, *zoom, *background_color, world);
            }
            HudElementType::Image { texture, tint } => {
                // TODO: Implement image rendering
            }
            HudElementType::Container { children } => {
                for child in children {
                    self.render_element_egui(ctx, child, world, screen_width, screen_height);
                }
            }
        }
    }
    
    fn render_health_bar_egui(
        &self,
        ctx: &egui::Context,
        id: &str,
        pos: [f32; 2],
        size: [f32; 2],
        binding: &str,
        color: [f32; 4],
        background_color: [f32; 4],
    ) {
        let value = self.get_value(binding).unwrap_or(0.0);
        let progress = value.clamp(0.0, 1.0);
        
        egui::Area::new(egui::Id::new(id))
            .fixed_pos(egui::pos2(pos[0], pos[1]))
            .show(ctx, |ui| {
                ui.set_width(size[0]);
                ui.set_height(size[1]);
                
                // Background
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgba_premultiplied(
                        (background_color[0] * 255.0) as u8,
                        (background_color[1] * 255.0) as u8,
                        (background_color[2] * 255.0) as u8,
                        (background_color[3] * 255.0) as u8,
                    ),
                );
                
                // Foreground (health)
                let filled_width = size[0] * progress;
                let filled_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(filled_width, size[1]),
                );
                ui.painter().rect_filled(
                    filled_rect,
                    2.0,
                    egui::Color32::from_rgba_premultiplied(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                        (color[3] * 255.0) as u8,
                    ),
                );
            });
    }
    
    fn render_progress_bar_egui(
        &self,
        ctx: &egui::Context,
        id: &str,
        pos: [f32; 2],
        size: [f32; 2],
        binding: &str,
        color: [f32; 4],
        background_color: [f32; 4],
    ) {
        self.render_health_bar_egui(ctx, id, pos, size, binding, color, background_color);
    }
    
    fn render_text_egui(
        &self,
        ctx: &egui::Context,
        id: &str,
        pos: [f32; 2],
        text: &str,
        font_size: f32,
        color: [f32; 4],
    ) {
        egui::Area::new(egui::Id::new(id))
            .fixed_pos(egui::pos2(pos[0], pos[1]))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(text)
                        .size(font_size)
                        .color(egui::Color32::from_rgba_premultiplied(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                            (color[3] * 255.0) as u8,
                        ))
                );
            });
    }
    
    fn render_minimap_egui(
        &self,
        ctx: &egui::Context,
        id: &str,
        pos: [f32; 2],
        size: [f32; 2],
        _zoom: f32,
        background_color: [f32; 4],
        _world: &World,
    ) {
        egui::Area::new(egui::Id::new(id))
            .fixed_pos(egui::pos2(pos[0], pos[1]))
            .show(ctx, |ui| {
                ui.set_width(size[0]);
                ui.set_height(size[1]);
                
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    rect,
                    4.0,
                    egui::Color32::from_rgba_premultiplied(
                        (background_color[0] * 255.0) as u8,
                        (background_color[1] * 255.0) as u8,
                        (background_color[2] * 255.0) as u8,
                        (background_color[3] * 255.0) as u8,
                    ),
                );
                
                // TODO: Render minimap content (entities, terrain, etc.)
                ui.centered_and_justified(|ui| {
                    ui.label("Minimap");
                });
            });
    }
    
    fn format_text(&self, format: &str) -> String {
        let mut result = format.to_string();
        
        // Replace {key} with cached values
        for (key, value) in &self.cached_values {
            let placeholder = format!("{{{}}}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &format!("{:.0}", value));
            }
        }
        
        result
    }
}

impl Default for HudManager {
    fn default() -> Self {
        Self::new()
    }
}
