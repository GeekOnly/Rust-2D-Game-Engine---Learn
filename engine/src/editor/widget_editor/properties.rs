//! Properties Panel
//! 
//! Panel for editing selected widget properties

use egui;
use super::state::WidgetEditorState;
use crate::hud::{HudElementType, Anchor};

pub struct PropertiesPanel {
    // Future: Add property editing state
}

impl PropertiesPanel {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut WidgetEditorState) {
        ui.heading("Properties");
        ui.separator();
        
        if let Some(element) = state.get_selected_element() {
            ui.label(format!("ID: {}", element.id));
            ui.separator();
            
            // Position
            ui.label("Position:");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut 0.0).speed(1.0));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut 0.0).speed(1.0));
            });
            
            // Size
            ui.label("Size:");
            ui.horizontal(|ui| {
                ui.label("W:");
                ui.add(egui::DragValue::new(&mut 0.0).speed(1.0));
                ui.label("H:");
                ui.add(egui::DragValue::new(&mut 0.0).speed(1.0));
            });
            
            ui.separator();
            
            // Anchor
            ui.label("Anchor:");
            egui::ComboBox::from_id_source("anchor_combo")
                .selected_text(format!("{:?}", element.anchor))
                .show_ui(ui, |ui| {
                    // TODO: Make anchor editable
                    ui.label("TopLeft");
                    ui.label("TopCenter");
                    ui.label("TopRight");
                    ui.label("CenterLeft");
                    ui.label("Center");
                    ui.label("CenterRight");
                    ui.label("BottomLeft");
                    ui.label("BottomCenter");
                    ui.label("BottomRight");
                });
            
            ui.separator();
            
            // Element-specific properties
            match &element.element_type {
                HudElementType::Text { text, font_size, color } => {
                    ui.label("Text Properties:");
                    ui.text_edit_singleline(&mut text.clone());
                    ui.add(egui::Slider::new(&mut font_size.clone(), 8.0..=72.0).text("Font Size"));
                }
                HudElementType::HealthBar { .. } => {
                    ui.label("Health Bar Properties:");
                    ui.label("(Edit in JSON for now)");
                }
                HudElementType::ProgressBar { .. } => {
                    ui.label("Progress Bar Properties:");
                    ui.label("(Edit in JSON for now)");
                }
                _ => {
                    ui.label("Element Type:");
                    ui.label(format!("{:?}", element.element_type));
                }
            }
        } else {
            ui.label("No element selected");
            ui.separator();
            ui.label("Click on an element in the canvas to edit its properties.");
        }
    }
}

impl Default for PropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}
