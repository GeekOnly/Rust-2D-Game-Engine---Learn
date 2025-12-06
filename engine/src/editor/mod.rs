// Editor-only modules
pub mod console;
pub mod ui;
pub mod states;
pub mod shortcuts;
pub mod camera;
pub mod grid;
pub mod theme;
pub mod toolbar;
pub mod autosave;
pub mod asset_manager;
pub mod drag_drop;
pub mod rendering_3d;
pub mod undo;
pub mod selection;
pub mod clipboard;
pub mod snapping;
pub mod shortcuts_handler;
pub mod texture_import_settings;
pub mod debug_draw;
pub mod map_manager;
pub mod hot_reload;
pub mod tilemap_error;
pub mod tilemap_settings;
pub mod widget_editor;

// Re-exports for convenience
pub use console::Console;
pub use ui::{EditorUI, TransformTool};
pub use states::{AppState, LauncherState, EditorState, EditorAction};
pub use shortcuts::EditorShortcut;
pub use camera::SceneCamera;
pub use grid::SceneGrid;
pub use theme::UnityTheme;
pub use asset_manager::AssetManager;
pub use drag_drop::{DragDropState, DraggedAsset};
pub use undo::{UndoStack, CreateEntityCommand, DeleteEntityCommand, BatchCommand};
pub use selection::{SelectionManager, SelectionMode};
pub use clipboard::{Clipboard, copy_selected, paste_from_clipboard, duplicate_selected};
pub use debug_draw::DebugDrawManager;
pub use map_manager::MapManager;
pub use tilemap_error::TilemapError;
pub use tilemap_settings::TilemapSettings;
pub use widget_editor::WidgetEditor;

// Re-export from sprite_editor crate
pub use sprite_editor::{SpriteMetadata, SpriteDefinition, ExportFormat};

// Sprite editor window (still in engine for now - TODO: move to sprite_editor crate)
pub mod sprite_editor_window;
pub use sprite_editor_window::SpriteEditorWindow;
