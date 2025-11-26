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
