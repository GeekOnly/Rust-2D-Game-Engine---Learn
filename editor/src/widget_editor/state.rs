//! Prefab Editor State
//! 
//! Manages the state of the UI prefab editor

use ui::prefab::{UIPrefab, UIPrefabElement};
use std::path::PathBuf;

/// Editor tools
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorTool {
    Select,
    Move,
    Resize,
    // Future: Rotate
}

/// Drag mode for different types of manipulation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragMode {
    None,
    Position,
    Resize,
    AnchorMin,
    AnchorMax,
    Pivot,
}

/// Prefab editor state
pub struct PrefabEditorState {
    /// Currently loaded prefab
    pub current_prefab: Option<UIPrefab>,
    
    /// Current file path
    pub current_file: Option<PathBuf>,
    
    /// Selected element name
    pub selected_element: Option<String>,
    
    /// Current tool
    pub current_tool: EditorTool,
    
    /// Current drag mode
    pub drag_mode: DragMode,
    
    /// Is dragging
    pub is_dragging: bool,
    
    /// Drag start position
    pub drag_start: Option<[f32; 2]>,
    
    /// Element start position (before drag)
    pub element_start_pos: Option<ui::Vec2>,
    
    /// Element start size (before resize)
    pub element_start_size: Option<ui::Vec2>,
    
    /// Element start anchor min (before drag)
    pub element_start_anchor_min: Option<ui::Vec2>,
    
    /// Element start anchor max (before drag)
    pub element_start_anchor_max: Option<ui::Vec2>,
    
    /// Element start pivot (before drag)
    pub element_start_pivot: Option<ui::Vec2>,
    
    /// Has unsaved changes
    pub modified: bool,
    
    /// Canvas zoom
    pub zoom: f32,
    
    /// Canvas pan
    pub pan: [f32; 2],
}

impl PrefabEditorState {
    pub fn new() -> Self {
        Self {
            current_prefab: None,
            current_file: None,
            selected_element: None,
            current_tool: EditorTool::Select,
            drag_mode: DragMode::None,
            is_dragging: false,
            drag_start: None,
            element_start_pos: None,
            element_start_size: None,
            element_start_anchor_min: None,
            element_start_anchor_max: None,
            element_start_pivot: None,
            modified: false,
            zoom: 1.0,
            pan: [0.0, 0.0],
        }
    }
    
    /// Get selected element
    pub fn get_selected_element(&self) -> Option<&UIPrefabElement> {
        if let (Some(prefab), Some(name)) = (&self.current_prefab, &self.selected_element) {
            self.find_element_by_name(&prefab.root, name)
        } else {
            None
        }
    }
    
    /// Get mutable selected element
    pub fn get_selected_element_mut(&mut self) -> Option<&mut UIPrefabElement> {
        if let Some(name) = self.selected_element.clone() {
            if let Some(prefab) = &mut self.current_prefab {
                return Self::find_element_by_name_mut(&mut prefab.root, &name);
            }
        }
        None
    }
    
    /// Find element by name (recursive)
    fn find_element_by_name<'a>(&self, element: &'a UIPrefabElement, name: &str) -> Option<&'a UIPrefabElement> {
        if element.name == name {
            return Some(element);
        }
        
        for child in &element.children {
            if let Some(found) = self.find_element_by_name(child, name) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Find mutable element by name (recursive) - static method
    fn find_element_by_name_mut<'a>(element: &'a mut UIPrefabElement, name: &str) -> Option<&'a mut UIPrefabElement> {
        if element.name == name {
            return Some(element);
        }
        
        for child in &mut element.children {
            if let Some(found) = Self::find_element_by_name_mut(child, name) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Select element by name
    pub fn select_element(&mut self, name: String) {
        self.selected_element = Some(name);
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

impl Default for PrefabEditorState {
    fn default() -> Self {
        Self::new()
    }
}
