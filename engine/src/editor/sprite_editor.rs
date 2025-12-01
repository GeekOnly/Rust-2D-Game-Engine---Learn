use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use egui::TextureHandle;

/// Represents a single sprite definition within a sprite sheet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteDefinition {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SpriteDefinition {
    /// Create a new sprite definition
    pub fn new(name: String, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }
}

/// Metadata for a sprite sheet containing multiple sprites
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteMetadata {
    pub texture_path: String,
    pub texture_width: u32,
    pub texture_height: u32,
    pub sprites: Vec<SpriteDefinition>,
}

impl SpriteMetadata {
    /// Create a new sprite metadata
    pub fn new(texture_path: String, texture_width: u32, texture_height: u32) -> Self {
        Self {
            texture_path,
            texture_width,
            texture_height,
            sprites: Vec::new(),
        }
    }

    /// Add a sprite to the metadata
    pub fn add_sprite(&mut self, sprite: SpriteDefinition) {
        self.sprites.push(sprite);
    }

    /// Remove a sprite by index
    pub fn remove_sprite(&mut self, index: usize) -> Option<SpriteDefinition> {
        if index < self.sprites.len() {
            Some(self.sprites.remove(index))
        } else {
            None
        }
    }

    /// Find a sprite by name
    pub fn find_sprite(&self, name: &str) -> Option<&SpriteDefinition> {
        self.sprites.iter().find(|s| s.name == name)
    }

    /// Check if a sprite name already exists
    pub fn has_sprite_name(&self, name: &str) -> bool {
        self.sprites.iter().any(|s| s.name == name)
    }

    /// Save sprite metadata to a .sprite JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        
        // Create backup if file exists
        if path.exists() {
            create_backup(path)?;
        }

        // Serialize to JSON with pretty formatting
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize sprite metadata: {}", e))?;

        // Write to file
        fs::write(path, json)
            .map_err(|e| format!("Failed to write sprite file: {}", e))?;

        Ok(())
    }

    /// Load sprite metadata from a .sprite JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();

        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read sprite file: {}", e))?;

        // Deserialize from JSON
        let metadata: SpriteMetadata = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse sprite JSON: {}", e))?;

        Ok(metadata)
    }
}

/// Create a backup of an existing file
fn create_backup<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(());
    }

    // Generate backup filename with .bak extension
    let backup_path = path.with_extension("sprite.bak");

    // Copy the file to backup
    fs::copy(path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(())
}

/// Editor state for the sprite editor
pub struct SpriteEditorState {
    // File management
    pub texture_path: PathBuf,
    pub metadata_path: PathBuf,
    pub metadata: SpriteMetadata,
    
    // Editor state
    pub selected_sprite: Option<usize>,
    pub hovered_sprite: Option<usize>,
    pub is_drawing: bool,
    pub draw_start: Option<(f32, f32)>,
    pub draw_current: Option<(f32, f32)>,
    
    // View state
    pub zoom: f32,
    pub pan_offset: (f32, f32),
    
    // Undo/Redo
    pub undo_stack: Vec<SpriteMetadata>,
    pub redo_stack: Vec<SpriteMetadata>,
    
    // Texture
    pub texture_handle: Option<TextureHandle>,
}

impl SpriteEditorState {
    /// Create a new sprite editor state
    pub fn new(texture_path: PathBuf) -> Self {
        // Determine metadata path (.sprite file)
        let metadata_path = texture_path.with_extension("sprite");
        
        // Try to load existing metadata or create new
        let metadata = if metadata_path.exists() {
            SpriteMetadata::load(&metadata_path).unwrap_or_else(|e| {
                log::warn!("Failed to load sprite metadata: {}", e);
                SpriteMetadata::new(
                    texture_path.to_string_lossy().to_string(),
                    0,
                    0,
                )
            })
        } else {
            SpriteMetadata::new(
                texture_path.to_string_lossy().to_string(),
                0,
                0,
            )
        };
        
        Self {
            texture_path,
            metadata_path,
            metadata,
            selected_sprite: None,
            hovered_sprite: None,
            is_drawing: false,
            draw_start: None,
            draw_current: None,
            zoom: 1.0,
            pan_offset: (0.0, 0.0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            texture_handle: None,
        }
    }
    
    /// Push current state to undo stack
    pub fn push_undo(&mut self) {
        // Limit undo stack to 50 actions
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.metadata.clone());
        // Clear redo stack on new action
        self.redo_stack.clear();
    }
    
    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.redo_stack.push(self.metadata.clone());
            self.metadata = previous_state;
        }
    }
    
    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.metadata.clone());
            self.metadata = next_state;
        }
    }
    
    /// Save sprite metadata to file
    pub fn save(&self) -> Result<(), String> {
        self.metadata.save(&self.metadata_path)
    }
    
    /// Load texture using texture manager
    pub fn load_texture(&mut self, ctx: &egui::Context, texture_manager: &mut crate::texture_manager::TextureManager) -> Result<(), String> {
        // Generate unique texture ID for sprite editor
        let texture_id = format!("sprite_editor_{}", self.texture_path.to_string_lossy());
        
        // Load texture through texture manager
        if let Some(handle) = texture_manager.load_texture(ctx, &texture_id, &self.texture_path) {
            self.texture_handle = Some(handle.clone());
            
            // Update metadata with texture dimensions
            let size = handle.size();
            self.metadata.texture_width = size[0] as u32;
            self.metadata.texture_height = size[1] as u32;
            
            Ok(())
        } else {
            Err(format!("Failed to load texture: {}", self.texture_path.display()))
        }
    }
}

/// Sprite Editor Window
pub struct SpriteEditorWindow {
    pub state: SpriteEditorState,
    pub is_open: bool,
}

impl SpriteEditorWindow {
    /// Create a new sprite editor window
    pub fn new(texture_path: PathBuf) -> Self {
        Self {
            state: SpriteEditorState::new(texture_path),
            is_open: true,
        }
    }
    
    /// Render the sprite editor window
    pub fn render(&mut self, ctx: &egui::Context, texture_manager: &mut crate::texture_manager::TextureManager) {
        if !self.is_open {
            return;
        }
        
        // Load texture if not already loaded
        if self.state.texture_handle.is_none() {
            if let Err(e) = self.state.load_texture(ctx, texture_manager) {
                log::error!("Failed to load texture: {}", e);
                // Show error and close window
                self.is_open = false;
                return;
            }
        }
        
        let mut is_open = self.is_open;
        egui::Window::new("🎨 Sprite Editor")
            .open(&mut is_open)
            .default_size([1200.0, 800.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }
    
    /// Render the window content
    fn render_content(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("💾 Save (Ctrl+S)").clicked() {
                if let Err(e) = self.state.save() {
                    log::error!("Failed to save sprite metadata: {}", e);
                } else {
                    log::info!("Sprite metadata saved successfully");
                }
            }
            
            ui.separator();
            
            if ui.button("↶ Undo (Ctrl+Z)").clicked() {
                self.state.undo();
            }
            
            if ui.button("↷ Redo (Ctrl+Y)").clicked() {
                self.state.redo();
            }
            
            ui.separator();
            
            ui.label(format!("Zoom: {:.0}%", self.state.zoom * 100.0));
        });
        
        ui.separator();
        
        // Main content area
        ui.horizontal(|ui| {
            // Left panel - Sprite list (placeholder for now)
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.heading("Sprites");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(format!("Total: {}", self.state.metadata.sprites.len()));
                    
                    for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
                        let is_selected = self.state.selected_sprite == Some(idx);
                        if ui.selectable_label(is_selected, &sprite.name).clicked() {
                            self.state.selected_sprite = Some(idx);
                        }
                    }
                });
            });
            
            ui.separator();
            
            // Center panel - Canvas (placeholder for now)
            ui.vertical(|ui| {
                ui.heading("Canvas");
                
                if let Some(texture_handle) = &self.state.texture_handle {
                    let texture_size = texture_handle.size();
                    
                    // Calculate scaled size based on zoom
                    let scaled_width = texture_size[0] as f32 * self.state.zoom;
                    let scaled_height = texture_size[1] as f32 * self.state.zoom;
                    
                    // Display texture with size
                    ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                        texture_handle.id(),
                        [scaled_width, scaled_height]
                    )));
                    
                    ui.label(format!(
                        "Texture: {}x{} pixels",
                        texture_size[0], texture_size[1]
                    ));
                } else {
                    ui.label("Loading texture...");
                }
            });
            
            ui.separator();
            
            // Right panel - Properties (placeholder for now)
            ui.vertical(|ui| {
                ui.set_width(300.0);
                ui.heading("Properties");
                ui.separator();
                
                if let Some(idx) = self.state.selected_sprite {
                    if let Some(sprite) = self.state.metadata.sprites.get(idx) {
                        ui.label(format!("Name: {}", sprite.name));
                        ui.label(format!("X: {}", sprite.x));
                        ui.label(format!("Y: {}", sprite.y));
                        ui.label(format!("Width: {}", sprite.width));
                        ui.label(format!("Height: {}", sprite.height));
                    }
                } else {
                    ui.label("No sprite selected");
                }
            });
        });
        
        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Sprites: {}", self.state.metadata.sprites.len()));
            ui.separator();
            ui.label(format!(
                "Texture: {}x{}",
                self.state.metadata.texture_width,
                self.state.metadata.texture_height
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_path(filename: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        path
    }

    fn cleanup_test_file(path: &Path) {
        let _ = fs::remove_file(path);
        let backup_path = path.with_extension("sprite.bak");
        let _ = fs::remove_file(backup_path);
    }

    #[test]
    fn test_sprite_definition_creation() {
        let sprite = SpriteDefinition::new("test_sprite".to_string(), 10, 20, 32, 32);
        assert_eq!(sprite.name, "test_sprite");
        assert_eq!(sprite.x, 10);
        assert_eq!(sprite.y, 20);
        assert_eq!(sprite.width, 32);
        assert_eq!(sprite.height, 32);
    }

    #[test]
    fn test_sprite_metadata_creation() {
        let metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        assert_eq!(metadata.texture_path, "texture.png");
        assert_eq!(metadata.texture_width, 512);
        assert_eq!(metadata.texture_height, 256);
        assert_eq!(metadata.sprites.len(), 0);
    }

    #[test]
    fn test_add_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        assert_eq!(metadata.sprites.len(), 1);
        assert_eq!(metadata.sprites[0], sprite);
    }

    #[test]
    fn test_remove_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        let removed = metadata.remove_sprite(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), sprite);
        assert_eq!(metadata.sprites.len(), 0);
    }

    #[test]
    fn test_find_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        let found = metadata.find_sprite("sprite_0");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &sprite);
        
        let not_found = metadata.find_sprite("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_has_sprite_name() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite);
        
        assert!(metadata.has_sprite_name("sprite_0"));
        assert!(!metadata.has_sprite_name("sprite_1"));
    }

    #[test]
    fn test_save_and_load() {
        let test_path = get_test_path("test_sprite.sprite");
        cleanup_test_file(&test_path);

        // Create metadata with sprites
        let mut metadata = SpriteMetadata::new("assets/texture.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));

        // Save to file
        let save_result = metadata.save(&test_path);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());

        // Load from file
        let loaded_result = SpriteMetadata::load(&test_path);
        assert!(loaded_result.is_ok(), "Load failed: {:?}", loaded_result.err());
        
        let loaded = loaded_result.unwrap();
        assert_eq!(loaded, metadata);
        assert_eq!(loaded.sprites.len(), 2);
        assert_eq!(loaded.sprites[0].name, "sprite_0");
        assert_eq!(loaded.sprites[1].name, "sprite_1");

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_backup_creation() {
        let test_path = get_test_path("test_backup.sprite");
        cleanup_test_file(&test_path);

        // Create initial file
        let metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        metadata.save(&test_path).unwrap();

        // Save again to trigger backup
        let mut metadata2 = SpriteMetadata::new("texture2.png".to_string(), 1024, 512);
        metadata2.add_sprite(SpriteDefinition::new("new_sprite".to_string(), 0, 0, 64, 64));
        metadata2.save(&test_path).unwrap();

        // Check backup exists
        let backup_path = test_path.with_extension("sprite.bak");
        assert!(backup_path.exists(), "Backup file should exist");

        // Load backup and verify it's the original
        let backup_metadata = SpriteMetadata::load(&backup_path).unwrap();
        assert_eq!(backup_metadata.texture_path, "texture.png");
        assert_eq!(backup_metadata.sprites.len(), 0);

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_json_format() {
        let test_path = get_test_path("test_format.sprite");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/knight.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("knight_idle_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("knight_run_0".to_string(), 32, 0, 32, 32));

        metadata.save(&test_path).unwrap();

        // Read the raw JSON to verify format
        let json_content = fs::read_to_string(&test_path).unwrap();
        
        // Verify it's valid JSON and contains expected fields
        assert!(json_content.contains("\"texture_path\""));
        assert!(json_content.contains("\"texture_width\""));
        assert!(json_content.contains("\"texture_height\""));
        assert!(json_content.contains("\"sprites\""));
        assert!(json_content.contains("\"knight_idle_0\""));
        assert!(json_content.contains("\"knight_run_0\""));

        cleanup_test_file(&test_path);
    }
}

    #[test]
    fn test_sprite_editor_state_creation() {
        let texture_path = PathBuf::from("test_texture.png");
        let state = SpriteEditorState::new(texture_path.clone());
        
        assert_eq!(state.texture_path, texture_path);
        assert_eq!(state.metadata_path, PathBuf::from("test_texture.sprite"));
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan_offset, (0.0, 0.0));
        assert!(state.selected_sprite.is_none());
        assert!(state.hovered_sprite.is_none());
        assert!(!state.is_drawing);
        assert!(state.undo_stack.is_empty());
        assert!(state.redo_stack.is_empty());
    }

    #[test]
    fn test_sprite_editor_window_creation() {
        let texture_path = PathBuf::from("test_texture.png");
        let window = SpriteEditorWindow::new(texture_path.clone());
        
        assert!(window.is_open);
        assert_eq!(window.state.texture_path, texture_path);
    }

    #[test]
    fn test_undo_redo_stack_management() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut state = SpriteEditorState::new(texture_path);
        
        // Initial state
        assert_eq!(state.metadata.sprites.len(), 0);
        
        // Add a sprite and push to undo
        state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        state.push_undo();
        
        // Add another sprite
        state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        assert_eq!(state.metadata.sprites.len(), 2);
        
        // Undo should restore to 1 sprite
        state.undo();
        assert_eq!(state.metadata.sprites.len(), 1);
        assert_eq!(state.redo_stack.len(), 1);
        
        // Redo should restore to 2 sprites
        state.redo();
        assert_eq!(state.metadata.sprites.len(), 2);
        assert_eq!(state.redo_stack.len(), 0);
    }

    #[test]
    fn test_undo_stack_limit() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut state = SpriteEditorState::new(texture_path);
        
        // Push 60 states (more than the 50 limit)
        for i in 0..60 {
            state.metadata.add_sprite(SpriteDefinition::new(
                format!("sprite_{}", i),
                i as u32 * 32,
                0,
                32,
                32
            ));
            state.push_undo();
        }
        
        // Stack should be limited to 50
        assert_eq!(state.undo_stack.len(), 50);
    }

    #[test]
    fn test_new_action_clears_redo_stack() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut state = SpriteEditorState::new(texture_path);
        
        // Add sprite and push to undo
        state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        state.push_undo();
        
        // Add another sprite
        state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        state.push_undo();
        
        // Undo once
        state.undo();
        assert_eq!(state.redo_stack.len(), 1);
        
        // New action should clear redo stack
        state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        state.push_undo();
        assert_eq!(state.redo_stack.len(), 0);
    }
