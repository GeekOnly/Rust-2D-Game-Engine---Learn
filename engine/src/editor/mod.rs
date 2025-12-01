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
pub mod sprite_editor;

// Re-exports for convenience
pub use console::Console;
pub use ui::{EditorUI, TransformTool};
pub use states::{AppState, LauncherState, EditorState, EditorAction};
pub use shortcuts::{ShortcutManager, EditorShortcut};
pub use camera::SceneCamera;
pub use grid::SceneGrid;
pub use theme::UnityTheme;
pub use toolbar::Toolbar;
pub use autosave::AutoSave;
pub use asset_manager::AssetManager;
pub use drag_drop::{DragDropState, DraggedAsset};
pub use rendering_3d::{Point3D, Face3D, depth_sort_faces};
pub use undo::{UndoStack, Command, CreateEntityCommand, DeleteEntityCommand, MoveEntityCommand, RotateEntityCommand, ScaleEntityCommand, RenameEntityCommand, BatchCommand};
pub use selection::{SelectionManager, SelectionMode, BoxSelection, handle_scene_selection, handle_hierarchy_selection};
pub use clipboard::{Clipboard, ClipboardAction, copy_selected, paste_from_clipboard, duplicate_selected, handle_clipboard_shortcuts};
pub use snapping::{SnapSettings, SnapMode, snap_position, snap_rotation, snap_scale, render_snap_grid, render_snap_indicator, handle_snap_shortcuts};
pub use shortcuts_handler::{handle_editor_shortcuts, get_shortcut_hints, render_shortcuts_help};
pub use sprite_editor::{SpriteDefinition, SpriteMetadata};
