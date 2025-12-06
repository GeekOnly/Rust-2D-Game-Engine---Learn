//! Widget Editor Module
//! 
//! Visual editor for creating and editing HUD/UI widgets (Unreal UMG style)

pub mod canvas;
pub mod properties;
pub mod state;

pub use canvas::WidgetCanvas;
pub use properties::PropertiesPanel;
pub use state::{WidgetEditorState, EditorTool};

use egui;
use crate::hud::{HudAsset, HudElement};
use std::path::PathBuf;

/// Main widget editor
pub struct WidgetEditor {
    pub state: WidgetEditorState,
    pub canvas: WidgetCanvas,
    pub properties: PropertiesPanel,
}

impl WidgetEditor {
    pub fn new() -> Self {
        Self {
            state: WidgetEditorState::new(),
            canvas: WidgetCanvas::new(),
            properties: PropertiesPanel::new(),
        }
    }
    
    /// Load HUD file into editor
    pub fn load_hud(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let hud = HudAsset::load(path.to_str().unwrap())?;
        self.state.current_hud = Some(hud);
        self.state.current_file = Some(path.clone());
        self.state.modified = false;
        Ok(())
    }
    
    /// Save current HUD to file
    pub fn save_hud(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(hud), Some(path)) = (&self.state.current_hud, &self.state.current_file) {
            hud.save(path.to_str().unwrap())?;
            self.state.modified = false;
            Ok(())
        } else {
            Err("No HUD loaded or no file path".into())
        }
    }
    
    /// Render widget editor UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Top toolbar
        self.render_toolbar(ui);
        
        ui.separator();
        
        // Main content area
        egui::SidePanel::left("widget_properties")
            .default_width(250.0)
            .show_inside(ui, |ui| {
                self.properties.render(ui, &mut self.state);
            });
        
        // Canvas (center)
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.canvas.render(ui, &mut self.state);
        });
    }
    
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // File operations
            if ui.button("📁 Open").clicked() {
                // TODO: File dialog
            }
            
            if ui.button("💾 Save").clicked() {
                if let Err(e) = self.save_hud() {
                    log::error!("Failed to save HUD: {}", e);
                }
            }
            
            ui.separator();
            
            // Tools
            if ui.selectable_label(self.state.current_tool == EditorTool::Select, "🖱️ Select").clicked() {
                self.state.current_tool = EditorTool::Select;
            }
            
            if ui.selectable_label(self.state.current_tool == EditorTool::Move, "✋ Move").clicked() {
                self.state.current_tool = EditorTool::Move;
            }
            
            ui.separator();
            
            // View options
            ui.checkbox(&mut self.canvas.show_grid, "Grid");
            ui.checkbox(&mut self.canvas.show_safe_area, "Safe Area");
            
            ui.separator();
            
            // Status
            if self.state.modified {
                ui.label(egui::RichText::new("● Modified").color(egui::Color32::YELLOW));
            }
            
            if let Some(selected) = &self.state.selected_element {
                ui.label(format!("Selected: {}", selected));
            }
        });
    }
}

impl Default for WidgetEditor {
    fn default() -> Self {
        Self::new()
    }
}
