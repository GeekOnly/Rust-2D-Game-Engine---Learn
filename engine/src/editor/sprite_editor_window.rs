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

/// Implement TextureManager for our texture manager
impl TextureManager for crate::texture_manager::TextureManager {
    fn get_base_path(&self) -> Option<&Path> {
        crate::texture_manager::TextureManager::get_base_path(self).map(|p| p.as_ref())
    }
    
    fn load_texture(&mut self, ctx: &egui::Context, id: &str, path: &Path) -> Option<TextureHandle> {
        crate::texture_manager::TextureManager::load_texture(self, ctx, id, path).cloned()
    }
    
    fn load_texture_absolute(&mut self, ctx: &egui::Context, id: &str, path: &Path) -> Option<TextureHandle> {
        crate::texture_manager::TextureManager::load_texture_absolute(self, ctx, id, path).cloned()
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
        texture_manager: &mut crate::texture_manager::TextureManager,
        dt: f32,
    ) {
        // Sync state
        self.inner.state = self.state.clone();
        self.inner.is_open = self.is_open;
        
        self.inner.render(ctx, texture_manager, dt);
        
        // Sync back
        self.state = self.inner.state.clone();
        self.is_open = self.inner.is_open;
    }

    /// Render inline
    pub fn render_inline(
        &mut self,
        ui: &mut egui::Ui,
        texture_manager: &mut crate::texture_manager::TextureManager,
        dt: f32,
    ) {
        // Sync state
        self.inner.state = self.state.clone();
        self.inner.is_open = self.is_open;
        
        self.inner.render_inline(ui, texture_manager, dt);
        
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
