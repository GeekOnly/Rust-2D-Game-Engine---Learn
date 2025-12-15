//! Sprite Editor Window
//!
//! Visual editor window for sprite sheets
//!
//! NOTE: This is a thin wrapper around the sprite_editor crate's UI components.
//! The actual implementation is in the sprite_editor crate.

// Re-export from sprite_editor crate
pub use sprite_editor::ui::{
    SpriteEditorWindow as SpriteEditorWindowImpl,
    SpriteEditorState,
    DragMode,
    ResizeHandle,
    TextureManager,
};

use std::path::{Path, PathBuf};
use egui::TextureHandle;

/// Wrapper for engine texture manager to implement sprite editor trait
pub struct EditorTextureManager<'a>(pub &'a mut engine::texture_manager::TextureManager);

impl<'a> TextureManager for EditorTextureManager<'a> {
    fn get_base_path(&self) -> Option<&Path> {
        engine::texture_manager::TextureManager::get_base_path(self.0).map(|p| p.as_ref())
    }
    
    fn load_texture(&mut self, ctx: &egui::Context, id: &str, path: &Path) -> Option<TextureHandle> {
        engine::texture_manager::TextureManager::load_texture(self.0, ctx, id, path).cloned()
    }
    
    fn load_texture_absolute(&mut self, ctx: &egui::Context, id: &str, path: &Path) -> Option<TextureHandle> {
        engine::texture_manager::TextureManager::load_texture_absolute(self.0, ctx, id, path).cloned()
    }
}

/// Sprite Editor Window wrapper
pub struct SpriteEditorWindow {
    inner: SpriteEditorWindowImpl,
    pub is_open: bool,
    pub state: SpriteEditorState,
}

impl SpriteEditorWindow {
    /// Create a new sprite editor window
    pub fn new(texture_path: PathBuf) -> Self {
        let inner = SpriteEditorWindowImpl::new(texture_path.clone());
        let state = SpriteEditorState::new(texture_path);
        
        Self {
            inner,
            is_open: true,
            state,
        }
    }

    /// Render the sprite editor window
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        texture_manager: &mut engine::texture_manager::TextureManager,
        dt: f32,
    ) {
        // Sync state
        self.inner.state = self.state.clone();
        self.inner.is_open = self.is_open;
        
        let mut wrapper = EditorTextureManager(texture_manager);
        self.inner.render(ctx, &mut wrapper, dt);
        
        // Sync back
        self.state = self.inner.state.clone();
        self.is_open = self.inner.is_open;
    }

    /// Render inline
    pub fn render_inline(
        &mut self,
        ui: &mut egui::Ui,
        texture_manager: &mut engine::texture_manager::TextureManager,
        dt: f32,
    ) {
        // Sync state
        self.inner.state = self.state.clone();
        self.inner.is_open = self.is_open;
        
        let mut wrapper = EditorTextureManager(texture_manager);
        self.inner.render_inline(ui, &mut wrapper, dt);
        
        // Sync back
        self.state = self.inner.state.clone();
        self.is_open = self.inner.is_open;
    }
    
    /// Get reference to state
    pub fn state(&self) -> &SpriteEditorState {
        &self.state
    }
    
    /// Get mutable reference to state
    pub fn state_mut(&mut self) -> &mut SpriteEditorState {
        &mut self.state
    }
}
