pub mod app;
pub mod theme;
pub mod shortcuts;
pub mod console;
pub mod tools;
pub mod systems;
// pub mod undo; // Moved to systems
// pub mod clipboard; // Moved to systems
// pub mod camera; // Moved to systems
pub mod grid;
// pub mod snapping; // Moved to tools
// pub mod selection; // Moved to tools
pub mod rendering_3d;
pub mod game_view_renderer;
pub mod scene_view_renderer;

pub mod states;
pub mod toolbar;
pub mod autosave;
pub mod asset_manager;
pub mod drag_drop;
pub mod shortcuts_handler;
pub mod texture_import_settings;
pub mod debug_draw;
pub mod map_manager;
pub mod hot_reload;
pub mod tilemap_error;
pub mod tilemap_settings;
pub mod widget_editor;
pub mod prefab;
pub mod sprite_editor_window;
pub mod ui;
// Re-exports for convenience (matching old engine::editor interface)
pub use console::Console;
pub use ui::{EditorUI, TransformTool};
pub use states::{AppState, LauncherState, EditorState, EditorAction};
pub use shortcuts::EditorShortcut; // Wait, shortcuts is in root.
pub use systems::camera::{SceneCamera, SceneProjectionMode};
pub use ui::camera_settings::CameraStateDisplay;
pub use grid::{SceneGrid, InfiniteGrid, CameraState};
pub use theme::UnityTheme;
pub use asset_manager::AssetManager;
pub use drag_drop::{DragDropState, DraggedAsset};
pub use systems::undo::{UndoStack, CreateEntityCommand, DeleteEntityCommand, BatchCommand};
pub use tools::selection::{SelectionManager, SelectionMode};
pub use systems::clipboard::{Clipboard, copy_selected, paste_from_clipboard, duplicate_selected};
pub use debug_draw::DebugDrawManager;
pub use map_manager::MapManager;
pub use tilemap_error::TilemapError;
pub use tilemap_settings::TilemapSettings;
pub use widget_editor::PrefabEditor;
pub use prefab::{Prefab, PrefabManager, PrefabEntity, PrefabMetadata};
pub use sprite_editor_window::SpriteEditorWindow;
pub mod editor_logic;
