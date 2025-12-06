//! Widget Editor State
//! 
//! Manages the state of the widget editor

use crate::hud::HudAsset;
use std::path::PathBuf;

/// Editor tools
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorTool {
    Select,
    Move,
    // Future: Resize, Rotate
}

/// Widget editor state
pub struct WidgetEditorState {
    /// Currently loaded HUD
    pub current_hud: Option<HudAsset>,
    
    /// Current file path
    pub current_file: Option<PathBuf>,
    
    /// Selected element ID
    pub selected_element: Option<String>,
    
    /// Current tool
    pub current_tool: EditorTool,
    
    /// Is dragging
    pub is_dragging: bool,
    
    /// Drag start position
    pub drag_start: Option<[f32; 2]>,
    
    /// Element start position (before drag)
    pub element_start_pos: Option<[f32; 2]>,
    
    /// Has unsaved changes
    pub modified: bool,
    
    /// Canvas zoom
    pub zoom: f32,
    
    /// Canvas pan
    pub pan: [f32; 2],
}

impl WidgetEditorState {
    pub fn new() -> Self {
        Self {
            current_hud: None,
            current_file: None,
            selected_element: None,
            current_tool: EditorTool::Select,
            is_dragging: false,
            drag_start: None,
            element_start_pos: None,
            modified: false,
            zoom: 1.0,
            pan: [0.0, 0.0],
        }
    }
    
    /// Get selected element
    pub fn get_selected_element(&self) -> Option<&crate::hud::HudElement> {
        if let (Some(hud), Some(id)) = (&self.current_hud, &self.selected_element) {
            hud.elements.iter().find(|e| &e.id == id)
        } else {
            None
        }
    }
    
    /// Get mutable selected element
    pub fn get_selected_element_mut(&mut self) -> Option<&mut crate::hud::HudElement> {
        if let (Some(hud), Some(id)) = (&mut self.current_hud, &self.selected_element) {
            hud.elements.iter_mut().find(|e| &e.id == id)
        } else {
            None
        }
    }
    
    /// Select element by ID
    pub fn select_element(&mut self, id: String) {
        self.selected_element = Some(id);
    }
    
    /// Deselect current element
    pub fn deselect(&mut self) {
        self.selected_element = None;
    }
    
    /// Mark as modified
    pub fn mark_modified(&mut self) {
        self.modified = true;
    }
}

impl Default for WidgetEditorState {
    fn default() -> Self {
        Self::new()
    }
}
