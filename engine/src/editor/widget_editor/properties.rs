//! Properties Panel
//! 
//! Panel for editing selected UI prefab element properties

use egui;
use super::state::PrefabEditorState;

pub struct PropertiesPanel {
    show_component_palette: bool,
}

impl PropertiesPanel {
    pub fn new() -> Self {
        Self {
            show_component_palette: true,
        }
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut PrefabEditorState) {
        // Component Palette
        ui.collapsing("Component Palette", |ui| {
            ui.label("Available Components:");
            ui.separator();
            
            if ui.button("📄 Panel").clicked() {
                // TODO: Add panel component to selected element
            }
            if ui.button("🖼️ Image").clicked() {
                // TODO: Add image component to selected element
            }
            if ui.button("📝 Text").clicked() {
                // TODO: Add text component to selected element
            }
            if ui.button("🔘 Button").clicked() {
                // TODO: Add button component to selected element
            }
            if ui.button("🎚️ Slider").clicked() {
                // TODO: Add slider component to selected element
            }
            if ui.button("☑️ Toggle").clicked() {
                // TODO: Add toggle component to selected element
            }
            if ui.button("📋 Dropdown").clicked() {
                // TODO: Add dropdown component to selected element
            }
            if ui.button("⌨️ Input Field").clicked() {
                // TODO: Add input field component to selected element
            }
            if ui.button("📜 Scroll View").clicked() {
                // TODO: Add scroll view component to selected element
            }
            
            ui.separator();
            ui.label("Layout Components:");
            
            if ui.button("↔️ Horizontal Layout").clicked() {
                // TODO: Add horizontal layout to selected element
            }
            if ui.button("↕️ Vertical Layout").clicked() {
                // TODO: Add vertical layout to selected element
            }
            if ui.button("⊞ Grid Layout").clicked() {
                // TODO: Add grid layout to selected element
            }
        });
        
        ui.separator();
        
        // Hierarchy Panel
        let has_prefab = state.current_prefab.is_some();
        if has_prefab {
            ui.collapsing("Hierarchy", |ui| {
                if let Some(prefab) = &state.current_prefab {
                    ui.label(format!("Prefab: {}", prefab.name));
                    ui.separator();
                    // Clone the root to avoid borrowing issues
                    let root_clone = prefab.root.clone();
                    self.render_hierarchy_tree(ui, &root_clone, state, 0);
                }
            });
        } else {
            ui.collapsing("Hierarchy", |ui| {
                ui.label("No prefab loaded");
            });
        }
        
        ui.separator();
        ui.heading("Properties");
        ui.separator();
        
        if let Some(element) = state.get_selected_element() {
            ui.label(format!("Name: {}", element.name));
            ui.separator();
            
            // RectTransform properties
            ui.heading("RectTransform");
            
            // Anchored Position
            ui.label("Anchored Position:");
            ui.horizontal(|ui| {
                ui.label(format!("X: {:.1}", element.rect_transform.anchored_position.x));
                ui.label(format!("Y: {:.1}", element.rect_transform.anchored_position.y));
            });
            
            // Size
            let size = element.rect_transform.get_size();
            ui.label("Size:");
            ui.horizontal(|ui| {
                ui.label(format!("W: {:.1}", size.x));
                ui.label(format!("H: {:.1}", size.y));
            });
            
            // Anchor Min/Max
            ui.label("Anchor Min:");
            ui.horizontal(|ui| {
                ui.label(format!("X: {:.2}", element.rect_transform.anchor_min.x));
                ui.label(format!("Y: {:.2}", element.rect_transform.anchor_min.y));
            });
            
            ui.label("Anchor Max:");
            ui.horizontal(|ui| {
                ui.label(format!("X: {:.2}", element.rect_transform.anchor_max.x));
                ui.label(format!("Y: {:.2}", element.rect_transform.anchor_max.y));
            });
            
            // Pivot
            ui.label("Pivot:");
            ui.horizontal(|ui| {
                ui.label(format!("X: {:.2}", element.rect_transform.pivot.x));
                ui.label(format!("Y: {:.2}", element.rect_transform.pivot.y));
            });
            
            ui.separator();
            
            // UIElement properties
            ui.heading("UIElement");
            ui.label(format!("Raycast Target: {}", element.ui_element.raycast_target));
            ui.label(format!("Interactable: {}", element.ui_element.interactable));
            ui.label(format!("Alpha: {:.2}", element.ui_element.alpha));
            
            ui.separator();
            
            // Component-specific properties
            if let Some(text) = &element.text {
                ui.heading("Text");
                ui.label(format!("Text: {}", text.text));
                ui.label(format!("Font Size: {}", text.font_size));
            }
            
            if element.button.is_some() {
                ui.heading("Button");
                ui.label("Button component attached");
            }
            
            if element.panel.is_some() {
                ui.heading("Panel");
                ui.label("Panel component attached");
            }
            
            if element.image.is_some() {
                ui.heading("Image");
                ui.label("Image component attached");
            }
            
            if element.slider.is_some() {
                ui.heading("Slider");
                ui.label("Slider component attached");
            }
            
            if element.toggle.is_some() {
                ui.heading("Toggle");
                ui.label("Toggle component attached");
            }
            
            if element.dropdown.is_some() {
                ui.heading("Dropdown");
                ui.label("Dropdown component attached");
            }
            
            if element.input_field.is_some() {
                ui.heading("Input Field");
                ui.label("Input Field component attached");
            }
            
            if element.scroll_view.is_some() {
                ui.heading("Scroll View");
                ui.label("Scroll View component attached");
            }
            
            // Layout components
            if element.horizontal_layout.is_some() {
                ui.heading("Horizontal Layout");
                ui.label("Horizontal Layout Group attached");
            }
            
            if element.vertical_layout.is_some() {
                ui.heading("Vertical Layout");
                ui.label("Vertical Layout Group attached");
            }
            
            if element.grid_layout.is_some() {
                ui.heading("Grid Layout");
                ui.label("Grid Layout Group attached");
            }
        } else {
            ui.label("No element selected");
            ui.separator();
            ui.label("Click on an element in the canvas to edit its properties.");
        }
    }
    
    fn render_hierarchy_tree(&self, ui: &mut egui::Ui, element: &ui::prefab::UIPrefabElement, state: &mut PrefabEditorState, depth: usize) {
        let indent = depth as f32 * 20.0;
        
        ui.horizontal(|ui| {
            ui.add_space(indent);
            
            let is_selected = state.selected_element.as_ref() == Some(&element.name);
            let label_text = if element.children.is_empty() {
                format!("  {}", element.name)
            } else {
                format!("▼ {}", element.name)
            };
            
            if ui.selectable_label(is_selected, label_text).clicked() {
                state.select_element(element.name.clone());
            }
        });
        
        // Render children
        for child in &element.children {
            self.render_hierarchy_tree(ui, child, state, depth + 1);
        }
    }
}

impl Default for PropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}
