//! UI components for the sprite editor
//!
//! This module provides egui-based UI components for visual sprite editing.

#[cfg(feature = "editor_ui")]
mod sprite_editor_window;

#[cfg(feature = "editor_ui")]
pub use sprite_editor_window::{
    SpriteEditorWindow, SpriteEditorState,
    DragMode, ResizeHandle, TextureManager,
};
