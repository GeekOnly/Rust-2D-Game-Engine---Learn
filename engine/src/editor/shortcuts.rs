/// Keyboard shortcuts system for Unity-like editor
use winit::keyboard::KeyCode;
use egui::Modifiers;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorShortcut {
    // Transform tools
    ViewTool,      // Q
    MoveTool,      // W
    RotateTool,    // E
    ScaleTool,     // R
    RectTool,      // T
    
    // Frame & Focus
    FrameSelected, // F
    
    // File operations
    NewScene,      // Ctrl+N
    OpenScene,     // Ctrl+O
    SaveScene,     // Ctrl+S
    SaveSceneAs,   // Ctrl+Shift+S
    Exit,          // Ctrl+Q
    
    // Edit operations
    Undo,          // Ctrl+Z
    Redo,          // Ctrl+Y or Ctrl+Shift+Z
    Duplicate,     // Ctrl+D
    Delete,        // Delete
    SelectAll,     // Ctrl+A
    DeselectAll,   // Ctrl+Shift+A
    
    // Play controls
    Play,          // Ctrl+P
    Pause,         // Ctrl+Shift+P
    Step,          // Ctrl+Alt+P
    
    // View
    ToggleGrid,    // G
    ToggleGizmos,  // Ctrl+G
}

pub struct ShortcutManager {
    modifiers: Modifiers,
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            modifiers: Modifiers::default(),
        }
    }
    
    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }
    
    pub fn check_shortcut(&self, key: KeyCode) -> Option<EditorShortcut> {
        let ctrl = self.modifiers.ctrl;
        let shift = self.modifiers.shift;
        let alt = self.modifiers.alt;
        
        match (key, ctrl, shift, alt) {
            // Transform tools (no modifiers)
            (KeyCode::KeyQ, false, false, false) => Some(EditorShortcut::ViewTool),
            (KeyCode::KeyW, false, false, false) => Some(EditorShortcut::MoveTool),
            (KeyCode::KeyE, false, false, false) => Some(EditorShortcut::RotateTool),
            (KeyCode::KeyR, false, false, false) => Some(EditorShortcut::ScaleTool),
            (KeyCode::KeyT, false, false, false) => Some(EditorShortcut::RectTool),
            
            // Frame selected
            (KeyCode::KeyF, false, false, false) => Some(EditorShortcut::FrameSelected),
            
            // File operations
            (KeyCode::KeyN, true, false, false) => Some(EditorShortcut::NewScene),
            (KeyCode::KeyO, true, false, false) => Some(EditorShortcut::OpenScene),
            (KeyCode::KeyS, true, false, false) => Some(EditorShortcut::SaveScene),
            (KeyCode::KeyS, true, true, false) => Some(EditorShortcut::SaveSceneAs),
            (KeyCode::KeyQ, true, false, false) => Some(EditorShortcut::Exit),
            
            // Edit operations
            (KeyCode::KeyZ, true, false, false) => Some(EditorShortcut::Undo),
            (KeyCode::KeyY, true, false, false) => Some(EditorShortcut::Redo),
            (KeyCode::KeyZ, true, true, false) => Some(EditorShortcut::Redo),
            (KeyCode::KeyD, true, false, false) => Some(EditorShortcut::Duplicate),
            (KeyCode::Delete, false, false, false) => Some(EditorShortcut::Delete),
            (KeyCode::KeyA, true, false, false) => Some(EditorShortcut::SelectAll),
            (KeyCode::KeyA, true, true, false) => Some(EditorShortcut::DeselectAll),
            
            // Play controls
            (KeyCode::KeyP, true, false, false) => Some(EditorShortcut::Play),
            (KeyCode::KeyP, true, true, false) => Some(EditorShortcut::Pause),
            (KeyCode::KeyP, true, false, true) => Some(EditorShortcut::Step),
            
            // View
            (KeyCode::KeyG, false, false, false) => Some(EditorShortcut::ToggleGrid),
            (KeyCode::KeyG, true, false, false) => Some(EditorShortcut::ToggleGizmos),
            
            _ => None,
        }
    }
}
