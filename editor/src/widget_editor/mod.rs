//! Prefab Editor Module
//! 
//! Visual editor for creating and editing UI prefabs (Unity-style UI editor)

pub mod canvas;
pub mod properties;
pub mod state;

pub use canvas::PrefabCanvas;
pub use properties::PropertiesPanel;
pub use state::{PrefabEditorState, EditorTool, DragMode};

use egui;
use ui::prefab::UIPrefab;
use std::path::PathBuf;

/// Main prefab editor
pub struct PrefabEditor {
    pub state: PrefabEditorState,
    pub canvas: PrefabCanvas,
    pub properties: PropertiesPanel,
}

impl PrefabEditor {
    pub fn new() -> Self {
        Self {
            state: PrefabEditorState::new(),
            canvas: PrefabCanvas::new(),
            properties: PropertiesPanel::new(),
        }
    }
    
    /// Load prefab file into editor
    pub fn load_prefab(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Validate file extension
        if let Some(ext) = path.extension() {
            if ext != "uiprefab" {
                return Err(format!("Invalid file extension. Expected .uiprefab, got .{}", ext.to_string_lossy()).into());
            }
        } else {
            return Err("File has no extension. Expected .uiprefab".into());
        }
        
        // Read and parse file
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let prefab: UIPrefab = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse prefab JSON: {}", e))?;
        
        // Validate prefab structure
        self.validate_prefab(&prefab)?;
        
        self.state.current_prefab = Some(prefab);
        self.state.current_file = Some(path.clone());
        self.state.modified = false;
        Ok(())
    }
    
    /// Validate prefab structure
    fn validate_prefab(&self, prefab: &UIPrefab) -> Result<(), Box<dyn std::error::Error>> {
        // Check that prefab has a name
        if prefab.name.is_empty() {
            return Err("Prefab name cannot be empty".into());
        }
        
        // Validate root element
        self.validate_element(&prefab.root)?;
        
        Ok(())
    }
    
    /// Validate element structure recursively
    fn validate_element(&self, element: &ui::prefab::UIPrefabElement) -> Result<(), Box<dyn std::error::Error>> {
        // Check that element has a name
        if element.name.is_empty() {
            return Err("Element name cannot be empty".into());
        }
        
        // Validate anchor values
        let rt = &element.rect_transform;
        if rt.anchor_min.x < 0.0 || rt.anchor_min.x > 1.0 ||
           rt.anchor_min.y < 0.0 || rt.anchor_min.y > 1.0 {
            return Err(format!("Element '{}': anchor_min values must be between 0 and 1", element.name).into());
        }
        
        if rt.anchor_max.x < 0.0 || rt.anchor_max.x > 1.0 ||
           rt.anchor_max.y < 0.0 || rt.anchor_max.y > 1.0 {
            return Err(format!("Element '{}': anchor_max values must be between 0 and 1", element.name).into());
        }
        
        // Validate pivot values
        if rt.pivot.x < 0.0 || rt.pivot.x > 1.0 ||
           rt.pivot.y < 0.0 || rt.pivot.y > 1.0 {
            return Err(format!("Element '{}': pivot values must be between 0 and 1", element.name).into());
        }
        
        // Validate children recursively
        for child in &element.children {
            self.validate_element(child)?;
        }
        
        Ok(())
    }
    
    /// Save current prefab to file
    pub fn save_prefab(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(prefab), Some(path)) = (&self.state.current_prefab, &self.state.current_file) {
            // Validate before saving
            self.validate_prefab(prefab)?;
            
            // Serialize to JSON
            let json = serde_json::to_string_pretty(prefab)
                .map_err(|e| format!("Failed to serialize prefab: {}", e))?;
            
            // Write to file
            std::fs::write(path, json)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            
            self.state.modified = false;
            log::info!("Saved prefab to: {}", path.display());
            Ok(())
        } else {
            Err("No prefab loaded or no file path".into())
        }
    }
    
    /// Save prefab to a new file
    pub fn save_prefab_as(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Validate file extension
        if let Some(ext) = path.extension() {
            if ext != "uiprefab" {
                return Err(format!("Invalid file extension. Expected .uiprefab, got .{}", ext.to_string_lossy()).into());
            }
        } else {
            return Err("File has no extension. Expected .uiprefab".into());
        }
        
        if let Some(prefab) = &self.state.current_prefab {
            // Validate before saving
            self.validate_prefab(prefab)?;
            
            // Serialize to JSON
            let json = serde_json::to_string_pretty(prefab)
                .map_err(|e| format!("Failed to serialize prefab: {}", e))?;
            
            // Write to file
            std::fs::write(path, json)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            
            self.state.current_file = Some(path.clone());
            self.state.modified = false;
            log::info!("Saved prefab to: {}", path.display());
            Ok(())
        } else {
            Err("No prefab loaded".into())
        }
    }
    
    /// Create a new empty prefab
    pub fn new_prefab(&mut self, name: String) {
        let prefab = UIPrefab {
            name: name.clone(),
            root: ui::prefab::UIPrefabElement {
                name: "Root".to_string(),
                rect_transform: ui::RectTransform::default(),
                ui_element: ui::UIElement::default(),
                image: None,
                text: None,
                button: None,
                panel: None,
                slider: None,
                toggle: None,
                dropdown: None,
                input_field: None,
                scroll_view: None,
                mask: None,
                horizontal_layout: None,
                vertical_layout: None,
                grid_layout: None,
                children: vec![],
            },
        };
        
        self.state.current_prefab = Some(prefab);
        self.state.current_file = None;
        self.state.modified = true;
        log::info!("Created new prefab: {}", name);
    }
    
    /// Render prefab editor UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Top toolbar
        self.render_toolbar(ui);
        
        ui.separator();
        
        // Main content area
        egui::SidePanel::left("prefab_properties")
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
                if let Err(e) = self.save_prefab() {
                    log::error!("Failed to save prefab: {}", e);
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
            
            if ui.selectable_label(self.state.current_tool == EditorTool::Resize, "↔️ Resize").clicked() {
                self.state.current_tool = EditorTool::Resize;
            }
            
            ui.separator();
            
            // View options
            ui.checkbox(&mut self.canvas.show_grid, "Grid");
            ui.checkbox(&mut self.canvas.show_safe_area, "Safe Area");
            ui.checkbox(&mut self.canvas.show_anchors, "Anchors");
            ui.checkbox(&mut self.canvas.show_pivot, "Pivot");
            
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

impl Default for PrefabEditor {
    fn default() -> Self {
        Self::new()
    }
}
