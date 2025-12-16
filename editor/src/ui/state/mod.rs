use crate::states::EditorAction;
use std::collections::HashMap;
use std::path::PathBuf;

/// Editor UI State (separated from Data State)
pub struct EditorUIState {
    // ---- Window & Dialog States ----
    pub show_save_required_dialog: bool,
    pub show_unsaved_changes_dialog: bool,
    pub show_project_settings: bool,
    pub show_create_menu: bool,
    pub show_rename_dialog: bool,
    pub show_camera_settings: bool,
    pub show_export_dialog: bool,
    pub show_exit_dialog: bool,
    pub show_save_layout_dialog: bool,
    
    // ---- Tab & Layout State ----
    pub scene_view_tab: usize,
    pub bottom_panel_tab: usize,
    pub current_layout_name: String,
    pub current_layout_type: String,
    pub layout_request: Option<String>,
    pub save_layout_name: String,
    
    // ---- View Options ----
    pub show_colliders: bool,
    pub show_velocities: bool,
    pub show_debug_lines: bool,
    
    // ---- Selection & Interaction ----
    pub current_tool: crate::ui::TransformTool,
    pub selection: crate::SelectionManager,
    pub snap_settings: crate::tools::snapping::SnapSettings,
    
    // ---- Temporary Inputs ----
    pub rename_buffer: String,
    pub hierarchy_search: String,
    pub pending_action: Option<EditorAction>,
}

impl EditorUIState {
    pub fn new() -> Self {
        Self {
            show_save_required_dialog: false,
            show_unsaved_changes_dialog: false,
            show_project_settings: false,
            show_create_menu: false,
            show_rename_dialog: false,
            show_camera_settings: false,
            show_export_dialog: false,
            show_exit_dialog: false,
            show_save_layout_dialog: false,
            
            scene_view_tab: 0,
            bottom_panel_tab: 1, // Console by default
            current_layout_name: "default".to_string(),
            current_layout_type: "default".to_string(),
            layout_request: None,
            save_layout_name: String::new(),
            
            show_colliders: true,
            show_velocities: false,
            show_debug_lines: true,
            
            current_tool: crate::ui::TransformTool::View,
            selection: crate::SelectionManager::new(),
            snap_settings: crate::tools::snapping::SnapSettings::load().unwrap_or_default(),
            
            rename_buffer: String::new(),
            hierarchy_search: String::new(),
            pending_action: None,
        }
    }
}
