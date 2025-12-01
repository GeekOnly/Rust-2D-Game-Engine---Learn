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

/// Drag mode for sprite editing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragMode {
    None,
    Creating,
    MovingSprite(usize),
    ResizingSprite(usize, ResizeHandle),
}

/// Resize handle position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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
    
    // Drag state for editing
    pub drag_mode: DragMode,
    pub drag_start_pos: Option<(f32, f32)>,
    pub drag_original_sprite: Option<SpriteDefinition>,
    
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
            drag_mode: DragMode::None,
            drag_start_pos: None,
            drag_original_sprite: None,
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
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);
        
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
    
    /// Handle keyboard shortcuts for the sprite editor
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Handle Delete key to remove selected sprite
            if i.key_pressed(egui::Key::Delete) {
                self.delete_selected_sprite();
            }
            
            // Handle Ctrl+S to save
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if let Err(e) = self.state.save() {
                    log::error!("Failed to save sprite metadata: {}", e);
                } else {
                    log::info!("Sprite metadata saved successfully");
                }
            }
            
            // Handle Ctrl+Z to undo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                self.state.undo();
            }
            
            // Handle Ctrl+Y to redo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                self.state.redo();
            }
            
            // Handle Escape to deselect
            if i.key_pressed(egui::Key::Escape) {
                self.state.selected_sprite = None;
            }
        });
    }
    
    /// Delete the currently selected sprite
    fn delete_selected_sprite(&mut self) {
        if let Some(selected_idx) = self.state.selected_sprite {
            // Push current state to undo stack before deletion
            self.state.push_undo();
            
            // Remove the sprite from metadata
            self.state.metadata.remove_sprite(selected_idx);
            
            // Clear selection
            self.state.selected_sprite = None;
            
            log::info!("Deleted sprite at index {}", selected_idx);
        }
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
            // Left panel - Sprite list
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
            
            // Center panel - Canvas
            ui.vertical(|ui| {
                ui.heading("Canvas");
                
                if let Some(texture_handle) = self.state.texture_handle.clone() {
                    self.render_canvas(ui, &texture_handle);
                } else {
                    ui.label("Loading texture...");
                }
            });
            
            ui.separator();
            
            // Right panel - Properties
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
    
    /// Render the sprite canvas with texture, zoom, pan, and sprite rectangles
    fn render_canvas(&mut self, ui: &mut egui::Ui, texture_handle: &TextureHandle) {
        let texture_size = texture_handle.size();
        
        // Calculate scaled size based on zoom
        let scaled_width = texture_size[0] as f32 * self.state.zoom;
        let scaled_height = texture_size[1] as f32 * self.state.zoom;
        
        // Create a scrollable area for the canvas
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Allocate space for the canvas with pan offset
                let canvas_size = egui::vec2(
                    scaled_width + self.state.pan_offset.0.abs() * 2.0,
                    scaled_height + self.state.pan_offset.1.abs() * 2.0
                );
                
                let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
                
                // Handle zoom with mouse wheel
                if response.hovered() {
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta != 0.0 {
                        // Zoom in/out based on scroll direction
                        let zoom_factor = 1.0 + (scroll_delta * 0.001);
                        self.state.zoom = (self.state.zoom * zoom_factor).clamp(0.1, 10.0);
                    }
                }
                
                // Handle pan with middle mouse button drag
                if response.dragged_by(egui::PointerButton::Middle) {
                    let drag_delta = response.drag_delta();
                    self.state.pan_offset.0 += drag_delta.x;
                    self.state.pan_offset.1 += drag_delta.y;
                }
                
                // Calculate texture position with pan offset
                let texture_pos = response.rect.min + egui::vec2(
                    self.state.pan_offset.0,
                    self.state.pan_offset.1
                );
                
                // Draw the texture
                let texture_rect = egui::Rect::from_min_size(
                    texture_pos,
                    egui::vec2(scaled_width, scaled_height)
                );
                
                painter.image(
                    texture_handle.id(),
                    texture_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE
                );
                
                // Handle sprite selection and hover detection
                self.handle_sprite_interaction(&response, texture_pos);
                
                // Handle sprite editing (resize and move)
                self.handle_sprite_editing(&response, texture_pos, texture_size);
                
                // Handle sprite rectangle creation with left mouse button
                self.handle_sprite_creation(&response, texture_pos, texture_size);
                
                // Draw sprite rectangles and labels
                self.render_sprite_rectangles(&painter, texture_pos, texture_size);
                
                // Draw resize handles for selected sprite
                self.render_resize_handles(&painter, texture_pos);
                
                // Draw the rectangle being created
                if self.state.is_drawing {
                    if let (Some(start), Some(current)) = (self.state.draw_start, self.state.draw_current) {
                        let rect = self.calculate_draw_rect(start, current);
                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0))
                        );
                    }
                }
                
                // Display texture info
                ui.label(format!(
                    "Texture: {}x{} pixels",
                    texture_size[0], texture_size[1]
                ));
            });
    }
    
    /// Handle sprite selection and hover detection
    fn handle_sprite_interaction(&mut self, response: &egui::Response, texture_pos: egui::Pos2) {
        // Get pointer position if hovering
        if let Some(pointer_pos) = response.hover_pos() {
            // Check which sprite is being hovered
            self.state.hovered_sprite = self.find_sprite_at_position(pointer_pos, texture_pos);
        } else {
            self.state.hovered_sprite = None;
        }
        
        // Handle click to select sprite
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                // Find sprite at click position
                let clicked_sprite = self.find_sprite_at_position(pointer_pos, texture_pos);
                
                // Only update selection if we clicked on a sprite
                // If we clicked on empty space, sprite creation will handle it
                if clicked_sprite.is_some() {
                    self.state.selected_sprite = clicked_sprite;
                }
            }
        }
    }
    
    /// Find which sprite (if any) is at the given screen position
    fn find_sprite_at_position(&self, screen_pos: egui::Pos2, texture_pos: egui::Pos2) -> Option<usize> {
        let zoom = self.state.zoom;
        
        // Convert screen position to texture coordinates
        let texture_x = (screen_pos.x - texture_pos.x) / zoom;
        let texture_y = (screen_pos.y - texture_pos.y) / zoom;
        
        // Check sprites in reverse order (top to bottom in rendering)
        // This ensures we select the topmost sprite if they overlap
        for (idx, sprite) in self.state.metadata.sprites.iter().enumerate().rev() {
            let sprite_min_x = sprite.x as f32;
            let sprite_min_y = sprite.y as f32;
            let sprite_max_x = (sprite.x + sprite.width) as f32;
            let sprite_max_y = (sprite.y + sprite.height) as f32;
            
            // Check if point is inside sprite rectangle
            if texture_x >= sprite_min_x && texture_x <= sprite_max_x
                && texture_y >= sprite_min_y && texture_y <= sprite_max_y
            {
                return Some(idx);
            }
        }
        
        None
    }
    
    /// Get resize handle at screen position for a sprite
    fn get_resize_handle_at_position(&self, screen_pos: egui::Pos2, sprite_idx: usize, texture_pos: egui::Pos2) -> Option<ResizeHandle> {
        if let Some(sprite) = self.state.metadata.sprites.get(sprite_idx) {
            let zoom = self.state.zoom;
            let handle_size = 8.0; // 8x8px handles
            
            // Calculate sprite corners in screen space
            let sprite_screen_x = texture_pos.x + (sprite.x as f32 * zoom);
            let sprite_screen_y = texture_pos.y + (sprite.y as f32 * zoom);
            let sprite_screen_width = sprite.width as f32 * zoom;
            let sprite_screen_height = sprite.height as f32 * zoom;
            
            // Define handle rectangles
            let top_left = egui::Rect::from_min_size(
                egui::pos2(sprite_screen_x - handle_size / 2.0, sprite_screen_y - handle_size / 2.0),
                egui::vec2(handle_size, handle_size)
            );
            let top_right = egui::Rect::from_min_size(
                egui::pos2(sprite_screen_x + sprite_screen_width - handle_size / 2.0, sprite_screen_y - handle_size / 2.0),
                egui::vec2(handle_size, handle_size)
            );
            let bottom_left = egui::Rect::from_min_size(
                egui::pos2(sprite_screen_x - handle_size / 2.0, sprite_screen_y + sprite_screen_height - handle_size / 2.0),
                egui::vec2(handle_size, handle_size)
            );
            let bottom_right = egui::Rect::from_min_size(
                egui::pos2(sprite_screen_x + sprite_screen_width - handle_size / 2.0, sprite_screen_y + sprite_screen_height - handle_size / 2.0),
                egui::vec2(handle_size, handle_size)
            );
            
            // Check which handle is hit
            if top_left.contains(screen_pos) {
                return Some(ResizeHandle::TopLeft);
            } else if top_right.contains(screen_pos) {
                return Some(ResizeHandle::TopRight);
            } else if bottom_left.contains(screen_pos) {
                return Some(ResizeHandle::BottomLeft);
            } else if bottom_right.contains(screen_pos) {
                return Some(ResizeHandle::BottomRight);
            }
        }
        None
    }
    
    /// Check if position is inside sprite center (for moving)
    fn is_inside_sprite_center(&self, screen_pos: egui::Pos2, sprite_idx: usize, texture_pos: egui::Pos2) -> bool {
        if let Some(sprite) = self.state.metadata.sprites.get(sprite_idx) {
            let zoom = self.state.zoom;
            let handle_size = 8.0;
            
            // Calculate sprite rectangle in screen space
            let sprite_screen_x = texture_pos.x + (sprite.x as f32 * zoom);
            let sprite_screen_y = texture_pos.y + (sprite.y as f32 * zoom);
            let sprite_screen_width = sprite.width as f32 * zoom;
            let sprite_screen_height = sprite.height as f32 * zoom;
            
            // Create a slightly smaller rectangle for the center (excluding handle areas)
            let center_rect = egui::Rect::from_min_size(
                egui::pos2(sprite_screen_x + handle_size, sprite_screen_y + handle_size),
                egui::vec2(sprite_screen_width - handle_size * 2.0, sprite_screen_height - handle_size * 2.0)
            );
            
            center_rect.contains(screen_pos)
        } else {
            false
        }
    }
    
    /// Handle sprite editing (resize and move)
    fn handle_sprite_editing(&mut self, response: &egui::Response, texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        // Start drag operation
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                // Check if we're starting to edit a selected sprite
                if let Some(selected_idx) = self.state.selected_sprite {
                    // Check for resize handle
                    if let Some(handle) = self.get_resize_handle_at_position(pointer_pos, selected_idx, texture_pos) {
                        self.state.drag_mode = DragMode::ResizingSprite(selected_idx, handle);
                        self.state.drag_start_pos = Some((pointer_pos.x, pointer_pos.y));
                        self.state.drag_original_sprite = self.state.metadata.sprites.get(selected_idx).cloned();
                    }
                    // Check for center drag (move)
                    else if self.is_inside_sprite_center(pointer_pos, selected_idx, texture_pos) {
                        self.state.drag_mode = DragMode::MovingSprite(selected_idx);
                        self.state.drag_start_pos = Some((pointer_pos.x, pointer_pos.y));
                        self.state.drag_original_sprite = self.state.metadata.sprites.get(selected_idx).cloned();
                    }
                }
            }
        }
        
        // Continue drag operation
        if response.dragged_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                match self.state.drag_mode {
                    DragMode::ResizingSprite(sprite_idx, handle) => {
                        self.resize_sprite(sprite_idx, handle, pointer_pos, texture_pos, texture_size);
                    }
                    DragMode::MovingSprite(sprite_idx) => {
                        self.move_sprite(sprite_idx, pointer_pos, texture_pos, texture_size);
                    }
                    _ => {}
                }
            }
        }
        
        // End drag operation
        if response.drag_released_by(egui::PointerButton::Primary) {
            match self.state.drag_mode {
                DragMode::ResizingSprite(_, _) | DragMode::MovingSprite(_) => {
                    // Push to undo stack after edit
                    if let Some(original) = &self.state.drag_original_sprite {
                        // Only push if sprite actually changed
                        if let Some(current) = self.state.selected_sprite.and_then(|idx| self.state.metadata.sprites.get(idx)) {
                            if original != current {
                                self.state.push_undo();
                            }
                        }
                    }
                }
                _ => {}
            }
            
            // Reset drag state
            self.state.drag_mode = DragMode::None;
            self.state.drag_start_pos = None;
            self.state.drag_original_sprite = None;
        }
    }
    
    /// Resize sprite by dragging a corner handle
    fn resize_sprite(&mut self, sprite_idx: usize, handle: ResizeHandle, pointer_pos: egui::Pos2, texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        if let (Some(sprite), Some(original)) = (
            self.state.metadata.sprites.get_mut(sprite_idx),
            &self.state.drag_original_sprite
        ) {
            let zoom = self.state.zoom;
            
            // Convert pointer position to texture coordinates
            let texture_x = ((pointer_pos.x - texture_pos.x) / zoom).max(0.0).min(texture_size[0] as f32);
            let texture_y = ((pointer_pos.y - texture_pos.y) / zoom).max(0.0).min(texture_size[1] as f32);
            
            // Calculate new bounds based on which handle is being dragged
            let (new_x, new_y, new_width, new_height) = match handle {
                ResizeHandle::TopLeft => {
                    // Dragging top-left: adjust x, y, width, height
                    let new_x = texture_x.min((original.x + original.width - 1) as f32);
                    let new_y = texture_y.min((original.y + original.height - 1) as f32);
                    let new_width = (original.x + original.width) as f32 - new_x;
                    let new_height = (original.y + original.height) as f32 - new_y;
                    (new_x, new_y, new_width, new_height)
                }
                ResizeHandle::TopRight => {
                    // Dragging top-right: adjust y, width, height
                    let new_y = texture_y.min((original.y + original.height - 1) as f32);
                    let new_width = texture_x - original.x as f32;
                    let new_height = (original.y + original.height) as f32 - new_y;
                    (original.x as f32, new_y, new_width, new_height)
                }
                ResizeHandle::BottomLeft => {
                    // Dragging bottom-left: adjust x, width, height
                    let new_x = texture_x.min((original.x + original.width - 1) as f32);
                    let new_width = (original.x + original.width) as f32 - new_x;
                    let new_height = texture_y - original.y as f32;
                    (new_x, original.y as f32, new_width, new_height)
                }
                ResizeHandle::BottomRight => {
                    // Dragging bottom-right: adjust width, height
                    let new_width = texture_x - original.x as f32;
                    let new_height = texture_y - original.y as f32;
                    (original.x as f32, original.y as f32, new_width, new_height)
                }
            };
            
            // Validate positive dimensions and clamp to texture bounds
            let final_width = new_width.max(1.0).min((texture_size[0] as f32 - new_x).max(1.0));
            let final_height = new_height.max(1.0).min((texture_size[1] as f32 - new_y).max(1.0));
            let final_x = new_x.max(0.0).min((texture_size[0] - 1) as f32);
            let final_y = new_y.max(0.0).min((texture_size[1] - 1) as f32);
            
            // Update sprite
            sprite.x = final_x.round() as u32;
            sprite.y = final_y.round() as u32;
            sprite.width = final_width.round() as u32;
            sprite.height = final_height.round() as u32;
        }
    }
    
    /// Move sprite by dragging its center
    fn move_sprite(&mut self, sprite_idx: usize, pointer_pos: egui::Pos2, texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        if let (Some(sprite), Some(drag_start), Some(original)) = (
            self.state.metadata.sprites.get_mut(sprite_idx),
            self.state.drag_start_pos,
            &self.state.drag_original_sprite
        ) {
            let zoom = self.state.zoom;
            
            // Calculate drag delta in texture space
            let delta_x = (pointer_pos.x - drag_start.0) / zoom;
            let delta_y = (pointer_pos.y - drag_start.1) / zoom;
            
            // Calculate new position
            let new_x = (original.x as f32 + delta_x).max(0.0);
            let new_y = (original.y as f32 + delta_y).max(0.0);
            
            // Clamp to texture bounds (sprite must stay fully inside texture)
            let max_x = (texture_size[0] as f32 - sprite.width as f32).max(0.0);
            let max_y = (texture_size[1] as f32 - sprite.height as f32).max(0.0);
            
            let clamped_x = new_x.min(max_x);
            let clamped_y = new_y.min(max_y);
            
            // Update sprite position (dimensions remain unchanged)
            sprite.x = clamped_x.round() as u32;
            sprite.y = clamped_y.round() as u32;
        }
    }
    
    /// Handle sprite rectangle creation via click-and-drag
    fn handle_sprite_creation(&mut self, response: &egui::Response, texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        // Only handle creation if we're not in an editing drag mode
        if matches!(self.state.drag_mode, DragMode::ResizingSprite(_, _) | DragMode::MovingSprite(_)) {
            return;
        }
        
        // Only handle left mouse button for drawing
        if response.clicked_by(egui::PointerButton::Primary) {
            // Start drawing only if we didn't click on an existing sprite
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let clicked_sprite = self.find_sprite_at_position(pointer_pos, texture_pos);
                
                // Check if we clicked on a handle or center of selected sprite
                let clicked_on_edit_area = if let Some(selected_idx) = self.state.selected_sprite {
                    self.get_resize_handle_at_position(pointer_pos, selected_idx, texture_pos).is_some()
                        || self.is_inside_sprite_center(pointer_pos, selected_idx, texture_pos)
                } else {
                    false
                };
                
                // Only start drawing if we clicked on empty space (not on sprite or edit area)
                if clicked_sprite.is_none() && !clicked_on_edit_area {
                    self.state.is_drawing = true;
                    self.state.drag_mode = DragMode::Creating;
                    self.state.draw_start = Some((pointer_pos.x, pointer_pos.y));
                    self.state.draw_current = Some((pointer_pos.x, pointer_pos.y));
                }
            }
        }
        
        if response.dragged_by(egui::PointerButton::Primary) && self.state.is_drawing {
            // Update current position while dragging
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                self.state.draw_current = Some((pointer_pos.x, pointer_pos.y));
            }
        }
        
        if response.drag_released_by(egui::PointerButton::Primary) && self.state.is_drawing {
            // Finish drawing and create sprite
            if let (Some(start), Some(end)) = (self.state.draw_start, self.state.draw_current) {
                self.create_sprite_from_drag(start, end, texture_pos, texture_size);
            }
            
            // Reset drawing state
            self.state.is_drawing = false;
            self.state.drag_mode = DragMode::None;
            self.state.draw_start = None;
            self.state.draw_current = None;
        }
    }
    
    /// Calculate the rectangle being drawn
    fn calculate_draw_rect(&self, start: (f32, f32), current: (f32, f32)) -> egui::Rect {
        let min_x = start.0.min(current.0);
        let min_y = start.1.min(current.1);
        let max_x = start.0.max(current.0);
        let max_y = start.1.max(current.1);
        
        egui::Rect::from_min_max(
            egui::pos2(min_x, min_y),
            egui::pos2(max_x, max_y)
        )
    }
    
    /// Create a sprite from drag coordinates
    fn create_sprite_from_drag(&mut self, start: (f32, f32), end: (f32, f32), texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        // Convert screen coordinates to texture coordinates
        let zoom = self.state.zoom;
        
        // Calculate relative positions from texture origin
        let start_x = (start.0 - texture_pos.x) / zoom;
        let start_y = (start.1 - texture_pos.y) / zoom;
        let end_x = (end.0 - texture_pos.x) / zoom;
        let end_y = (end.1 - texture_pos.y) / zoom;
        
        // Calculate sprite bounds (min/max to handle any drag direction)
        let min_x = start_x.min(end_x).max(0.0);
        let min_y = start_y.min(end_y).max(0.0);
        let max_x = start_x.max(end_x).min(texture_size[0] as f32);
        let max_y = start_y.max(end_y).min(texture_size[1] as f32);
        
        // Calculate width and height
        let width = (max_x - min_x).round() as u32;
        let height = (max_y - min_y).round() as u32;
        
        // Validate rectangle has positive dimensions
        if width > 0 && height > 0 {
            // Push current state to undo stack before making changes
            self.state.push_undo();
            
            // Generate sequential name
            let sprite_name = self.generate_sequential_name();
            
            // Create new sprite
            let sprite = SpriteDefinition::new(
                sprite_name,
                min_x.round() as u32,
                min_y.round() as u32,
                width,
                height
            );
            
            // Add sprite to metadata
            self.state.metadata.add_sprite(sprite);
            
            // Select the newly created sprite
            self.state.selected_sprite = Some(self.state.metadata.sprites.len() - 1);
        }
    }
    
    /// Generate a sequential sprite name (sprite_0, sprite_1, etc.)
    fn generate_sequential_name(&self) -> String {
        let mut index = 0;
        loop {
            let name = format!("sprite_{}", index);
            if !self.state.metadata.has_sprite_name(&name) {
                return name;
            }
            index += 1;
        }
    }
    
    /// Render resize handles for the selected sprite
    fn render_resize_handles(&self, painter: &egui::Painter, texture_pos: egui::Pos2) {
        if let Some(selected_idx) = self.state.selected_sprite {
            if let Some(sprite) = self.state.metadata.sprites.get(selected_idx) {
                let zoom = self.state.zoom;
                let handle_size = 8.0;
                
                // Calculate sprite corners in screen space
                let sprite_screen_x = texture_pos.x + (sprite.x as f32 * zoom);
                let sprite_screen_y = texture_pos.y + (sprite.y as f32 * zoom);
                let sprite_screen_width = sprite.width as f32 * zoom;
                let sprite_screen_height = sprite.height as f32 * zoom;
                
                // Define handle positions
                let handles = [
                    (sprite_screen_x, sprite_screen_y), // Top-left
                    (sprite_screen_x + sprite_screen_width, sprite_screen_y), // Top-right
                    (sprite_screen_x, sprite_screen_y + sprite_screen_height), // Bottom-left
                    (sprite_screen_x + sprite_screen_width, sprite_screen_y + sprite_screen_height), // Bottom-right
                ];
                
                // Draw handles as filled squares
                for (x, y) in handles.iter() {
                    let handle_rect = egui::Rect::from_min_size(
                        egui::pos2(x - handle_size / 2.0, y - handle_size / 2.0),
                        egui::vec2(handle_size, handle_size)
                    );
                    
                    // Fill with white
                    painter.rect_filled(handle_rect, 0.0, egui::Color32::WHITE);
                    
                    // Border with black
                    painter.rect_stroke(
                        handle_rect,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::BLACK)
                    );
                }
            }
        }
    }
    
    /// Render sprite rectangles with borders and name labels
    fn render_sprite_rectangles(&self, painter: &egui::Painter, texture_pos: egui::Pos2, texture_size: [usize; 2]) {
        let zoom = self.state.zoom;
        
        for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
            // Calculate sprite rectangle position in screen space
            let sprite_screen_pos = egui::pos2(
                texture_pos.x + (sprite.x as f32 * zoom),
                texture_pos.y + (sprite.y as f32 * zoom)
            );
            
            let sprite_screen_size = egui::vec2(
                sprite.width as f32 * zoom,
                sprite.height as f32 * zoom
            );
            
            let sprite_rect = egui::Rect::from_min_size(sprite_screen_pos, sprite_screen_size);
            
            // Determine border color based on selection state
            let (border_color, border_width) = if Some(idx) == self.state.selected_sprite {
                // Selected sprite: yellow border, 2px
                (egui::Color32::from_rgb(255, 255, 0), 2.0)
            } else if Some(idx) == self.state.hovered_sprite {
                // Hovered sprite: white border, 1px
                (egui::Color32::WHITE, 1.0)
            } else {
                // Unselected sprite: semi-transparent blue border, 1px
                (egui::Color32::from_rgba_unmultiplied(100, 150, 255, 180), 1.0)
            };
            
            // Draw sprite rectangle border
            painter.rect_stroke(
                sprite_rect,
                0.0,
                egui::Stroke::new(border_width, border_color)
            );
            
            // Draw sprite name label
            let label_pos = egui::pos2(
                sprite_screen_pos.x + 2.0,
                sprite_screen_pos.y + 2.0
            );
            
            // Draw label background for readability
            let label_text = &sprite.name;
            let font_id = egui::FontId::proportional(12.0);
            let galley = painter.layout_no_wrap(
                label_text.clone(),
                font_id.clone(),
                egui::Color32::WHITE
            );
            
            let label_bg_rect = egui::Rect::from_min_size(
                label_pos,
                galley.size() + egui::vec2(4.0, 2.0)
            );
            
            // Draw semi-transparent black background
            painter.rect_filled(
                label_bg_rect,
                2.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)
            );
            
            // Draw label text
            painter.text(
                label_pos + egui::vec2(2.0, 1.0),
                egui::Align2::LEFT_TOP,
                label_text,
                font_id,
                egui::Color32::WHITE
            );
        }
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

    #[test]
    fn test_sprite_creation_with_positive_dimensions() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        // Set texture dimensions
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Simulate drag from (10, 10) to (42, 42) in texture space
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        let start = (10.0, 10.0);
        let end = (42.0, 42.0);
        
        window.create_sprite_from_drag(start, end, texture_pos, texture_size);
        
        // Verify sprite was created
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.name, "sprite_0");
        assert_eq!(sprite.x, 10);
        assert_eq!(sprite.y, 10);
        assert_eq!(sprite.width, 32);
        assert_eq!(sprite.height, 32);
        
        // Verify sprite is selected
        assert_eq!(window.state.selected_sprite, Some(0));
        
        // Verify undo stack was updated
        assert_eq!(window.state.undo_stack.len(), 1);
    }

    #[test]
    fn test_sprite_creation_with_reverse_drag() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Simulate drag from bottom-right to top-left
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        let start = (100.0, 100.0);
        let end = (50.0, 50.0);
        
        window.create_sprite_from_drag(start, end, texture_pos, texture_size);
        
        // Verify sprite was created with correct bounds
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 50);
        assert_eq!(sprite.y, 50);
        assert_eq!(sprite.width, 50);
        assert_eq!(sprite.height, 50);
    }

    #[test]
    fn test_sprite_creation_clamped_to_texture_bounds() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Simulate drag that goes beyond texture bounds
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        let start = (400.0, 200.0);
        let end = (600.0, 300.0); // Beyond texture bounds
        
        window.create_sprite_from_drag(start, end, texture_pos, texture_size);
        
        // Verify sprite was clamped to texture bounds
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 400);
        assert_eq!(sprite.y, 200);
        assert_eq!(sprite.width, 112); // Clamped to 512 - 400
        assert_eq!(sprite.height, 56); // Clamped to 256 - 200
    }

    #[test]
    fn test_sprite_creation_with_zero_dimensions_rejected() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Simulate drag with same start and end (zero dimensions)
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        let start = (100.0, 100.0);
        let end = (100.0, 100.0);
        
        window.create_sprite_from_drag(start, end, texture_pos, texture_size);
        
        // Verify no sprite was created
        assert_eq!(window.state.metadata.sprites.len(), 0);
        assert_eq!(window.state.undo_stack.len(), 0);
    }

    #[test]
    fn test_sequential_sprite_naming() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Create first sprite
        window.create_sprite_from_drag((0.0, 0.0), (32.0, 32.0), texture_pos, texture_size);
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
        
        // Create second sprite
        window.create_sprite_from_drag((32.0, 0.0), (64.0, 32.0), texture_pos, texture_size);
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_1");
        
        // Create third sprite
        window.create_sprite_from_drag((64.0, 0.0), (96.0, 32.0), texture_pos, texture_size);
        assert_eq!(window.state.metadata.sprites[2].name, "sprite_2");
        
        assert_eq!(window.state.metadata.sprites.len(), 3);
    }

    #[test]
    fn test_sequential_naming_with_gaps() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Manually add sprites with gaps in numbering
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        // Generate next name should fill the gap
        let name = window.generate_sequential_name();
        assert_eq!(name, "sprite_1");
    }

    #[test]
    fn test_sprite_creation_with_zoom() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        window.state.zoom = 2.0; // 2x zoom
        
        // Simulate drag in screen space (which will be scaled by zoom)
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        let start = (20.0, 20.0); // Screen space
        let end = (84.0, 84.0);   // Screen space
        
        window.create_sprite_from_drag(start, end, texture_pos, texture_size);
        
        // Verify sprite coordinates are in texture space (divided by zoom)
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 10); // 20 / 2.0
        assert_eq!(sprite.y, 10); // 20 / 2.0
        assert_eq!(sprite.width, 32); // (84 - 20) / 2.0
        assert_eq!(sprite.height, 32); // (84 - 20) / 2.0
    }

    #[test]
    fn test_find_sprite_at_position() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 50, 50, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 100, 100, 32, 32));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // Test finding sprite at various positions
        let found_0 = window.find_sprite_at_position(egui::pos2(16.0, 16.0), texture_pos);
        assert_eq!(found_0, Some(0));
        
        let found_1 = window.find_sprite_at_position(egui::pos2(60.0, 60.0), texture_pos);
        assert_eq!(found_1, Some(1));
        
        let found_2 = window.find_sprite_at_position(egui::pos2(110.0, 110.0), texture_pos);
        assert_eq!(found_2, Some(2));
        
        // Test position outside any sprite
        let found_none = window.find_sprite_at_position(egui::pos2(200.0, 200.0), texture_pos);
        assert_eq!(found_none, None);
    }

    #[test]
    fn test_delete_selected_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        assert_eq!(window.state.metadata.sprites.len(), 3);
        
        // Select the middle sprite
        window.state.selected_sprite = Some(1);
        
        // Delete the selected sprite
        window.delete_selected_sprite();
        
        // Verify sprite was removed
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Verify selection was cleared
        assert_eq!(window.state.selected_sprite, None);
        
        // Verify the correct sprite was removed (sprite_1 should be gone)
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_2");
        
        // Verify undo stack was updated
        assert_eq!(window.state.undo_stack.len(), 1);
    }
    
    #[test]
    fn test_delete_with_no_selection() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // No sprite selected
        window.state.selected_sprite = None;
        
        // Try to delete (should do nothing)
        window.delete_selected_sprite();
        
        // Verify nothing was removed
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Verify undo stack was not updated
        assert_eq!(window.state.undo_stack.len(), 0);
    }
    
    #[test]
    fn test_delete_first_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        // Select the first sprite
        window.state.selected_sprite = Some(0);
        
        // Delete the selected sprite
        window.delete_selected_sprite();
        
        // Verify sprite was removed
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Verify the correct sprites remain
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_1");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_2");
        
        // Verify selection was cleared
        assert_eq!(window.state.selected_sprite, None);
    }
    
    #[test]
    fn test_delete_last_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        // Select the last sprite
        window.state.selected_sprite = Some(2);
        
        // Delete the selected sprite
        window.delete_selected_sprite();
        
        // Verify sprite was removed
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Verify the correct sprites remain
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_1");
        
        // Verify selection was cleared
        assert_eq!(window.state.selected_sprite, None);
    }
    
    #[test]
    fn test_delete_only_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add only one sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Select the sprite
        window.state.selected_sprite = Some(0);
        
        // Delete the selected sprite
        window.delete_selected_sprite();
        
        // Verify sprite was removed
        assert_eq!(window.state.metadata.sprites.len(), 0);
        
        // Verify selection was cleared
        assert_eq!(window.state.selected_sprite, None);
    }
    
    #[test]
    fn test_delete_and_undo() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        // Select and delete a sprite
        window.state.selected_sprite = Some(1);
        window.delete_selected_sprite();
        
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Undo the deletion
        window.state.undo();
        
        // Verify sprite was restored
        assert_eq!(window.state.metadata.sprites.len(), 3);
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_1");
        assert_eq!(window.state.metadata.sprites[2].name, "sprite_2");
    }
    
    #[test]
    fn test_delete_multiple_sprites_sequentially() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add multiple sprites
        for i in 0..5 {
            window.state.metadata.add_sprite(SpriteDefinition::new(
                format!("sprite_{}", i),
                i * 32,
                0,
                32,
                32
            ));
        }
        
        assert_eq!(window.state.metadata.sprites.len(), 5);
        
        // Delete sprites one by one
        window.state.selected_sprite = Some(2);
        window.delete_selected_sprite();
        assert_eq!(window.state.metadata.sprites.len(), 4);
        assert_eq!(window.state.selected_sprite, None);
        
        window.state.selected_sprite = Some(0);
        window.delete_selected_sprite();
        assert_eq!(window.state.metadata.sprites.len(), 3);
        assert_eq!(window.state.selected_sprite, None);
        
        // Verify correct sprites remain
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_1");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_3");
        assert_eq!(window.state.metadata.sprites[2].name, "sprite_4");
        
        // Verify undo stack has both deletions
        assert_eq!(window.state.undo_stack.len(), 2);
    }

    #[test]
    fn test_find_sprite_at_position_with_zoom() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        window.state.zoom = 2.0; // 2x zoom
        
        // Add sprite at texture coordinates (0, 0, 32, 32)
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // In screen space, the sprite should be at (0, 0) to (64, 64) due to 2x zoom
        let found = window.find_sprite_at_position(egui::pos2(32.0, 32.0), texture_pos);
        assert_eq!(found, Some(0));
        
        // Position outside the zoomed sprite
        let found_none = window.find_sprite_at_position(egui::pos2(70.0, 70.0), texture_pos);
        assert_eq!(found_none, None);
    }

    #[test]
    fn test_find_sprite_at_position_with_overlapping_sprites() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add overlapping sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 64, 64));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 32, 64, 64));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // Position in overlap area should return the last sprite (topmost)
        let found = window.find_sprite_at_position(egui::pos2(40.0, 40.0), texture_pos);
        assert_eq!(found, Some(1), "Should select topmost sprite in overlap");
    }

    #[test]
    fn test_resize_handle_detection_top_left() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // Test top-left handle (should be at 100, 100)
        let handle = window.get_resize_handle_at_position(egui::pos2(100.0, 100.0), 0, texture_pos);
        assert_eq!(handle, Some(ResizeHandle::TopLeft));
    }

    #[test]
    fn test_resize_handle_detection_all_corners() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // Test all four corners
        assert_eq!(window.get_resize_handle_at_position(egui::pos2(100.0, 100.0), 0, texture_pos), Some(ResizeHandle::TopLeft));
        assert_eq!(window.get_resize_handle_at_position(egui::pos2(164.0, 100.0), 0, texture_pos), Some(ResizeHandle::TopRight));
        assert_eq!(window.get_resize_handle_at_position(egui::pos2(100.0, 164.0), 0, texture_pos), Some(ResizeHandle::BottomLeft));
        assert_eq!(window.get_resize_handle_at_position(egui::pos2(164.0, 164.0), 0, texture_pos), Some(ResizeHandle::BottomRight));
    }

    #[test]
    fn test_is_inside_sprite_center() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        let texture_pos = egui::pos2(0.0, 0.0);
        
        // Test center position (should be inside)
        assert!(window.is_inside_sprite_center(egui::pos2(132.0, 132.0), 0, texture_pos));
        
        // Test corner positions (should be outside center, in handle area)
        assert!(!window.is_inside_sprite_center(egui::pos2(100.0, 100.0), 0, texture_pos));
        assert!(!window.is_inside_sprite_center(egui::pos2(164.0, 164.0), 0, texture_pos));
    }

    #[test]
    fn test_move_sprite_basic() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        // Set up drag state
        window.state.drag_start_pos = Some((100.0, 100.0));
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Move sprite by dragging to (150, 150)
        window.move_sprite(0, egui::pos2(150.0, 150.0), texture_pos, texture_size);
        
        // Verify sprite moved by 50 pixels in both directions
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 150);
        assert_eq!(sprite.y, 150);
        assert_eq!(sprite.width, 64); // Width should remain unchanged
        assert_eq!(sprite.height, 64); // Height should remain unchanged
    }

    #[test]
    fn test_move_sprite_clamped_to_bounds() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (400, 200) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 400, 200, 64, 64));
        
        // Set up drag state
        window.state.drag_start_pos = Some((400.0, 200.0));
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Try to move sprite beyond texture bounds
        window.move_sprite(0, egui::pos2(500.0, 250.0), texture_pos, texture_size);
        
        // Verify sprite is clamped to texture bounds
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 448); // 512 - 64 = 448 (max x position)
        assert_eq!(sprite.y, 192); // 256 - 64 = 192 (max y position)
        assert_eq!(sprite.width, 64);
        assert_eq!(sprite.height, 64);
    }

    #[test]
    fn test_resize_sprite_bottom_right() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        // Set up drag state
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Resize by dragging bottom-right handle to (200, 200)
        window.resize_sprite(0, ResizeHandle::BottomRight, egui::pos2(200.0, 200.0), texture_pos, texture_size);
        
        // Verify sprite was resized
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 100);
        assert_eq!(sprite.y, 100);
        assert_eq!(sprite.width, 100); // 200 - 100
        assert_eq!(sprite.height, 100); // 200 - 100
    }

    #[test]
    fn test_resize_sprite_top_left() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        // Set up drag state
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Resize by dragging top-left handle to (80, 80)
        window.resize_sprite(0, ResizeHandle::TopLeft, egui::pos2(80.0, 80.0), texture_pos, texture_size);
        
        // Verify sprite was resized (position changes, bottom-right stays fixed)
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 80);
        assert_eq!(sprite.y, 80);
        assert_eq!(sprite.width, 84); // (100 + 64) - 80
        assert_eq!(sprite.height, 84); // (100 + 64) - 80
    }

    #[test]
    fn test_resize_sprite_maintains_positive_dimensions() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (100, 100) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        // Set up drag state
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Try to resize to negative dimensions by dragging bottom-right to top-left
        window.resize_sprite(0, ResizeHandle::BottomRight, egui::pos2(50.0, 50.0), texture_pos, texture_size);
        
        // Verify sprite maintains minimum dimensions (at least 1x1)
        let sprite = &window.state.metadata.sprites[0];
        assert!(sprite.width >= 1, "Width should be at least 1");
        assert!(sprite.height >= 1, "Height should be at least 1");
    }

    #[test]
    fn test_resize_sprite_clamped_to_texture_bounds() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite at (400, 200) with size 64x64
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 400, 200, 64, 64));
        
        // Set up drag state
        window.state.drag_original_sprite = window.state.metadata.sprites.get(0).cloned();
        
        let texture_pos = egui::pos2(0.0, 0.0);
        let texture_size = [512, 256];
        
        // Try to resize beyond texture bounds
        window.resize_sprite(0, ResizeHandle::BottomRight, egui::pos2(600.0, 300.0), texture_pos, texture_size);
        
        // Verify sprite is clamped to texture bounds
        let sprite = &window.state.metadata.sprites[0];
        assert_eq!(sprite.x, 400);
        assert_eq!(sprite.y, 200);
        assert_eq!(sprite.width, 112); // 512 - 400 = 112 (max width)
        assert_eq!(sprite.height, 56); // 256 - 200 = 56 (max height)
    }

    #[test]
    fn test_sprite_editing_pushes_to_undo_stack() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 100, 100, 64, 64));
        
        // Initial undo stack should be empty
        assert_eq!(window.state.undo_stack.len(), 0);
        
        // Simulate editing by pushing to undo stack
        window.state.push_undo();
        
        // Verify undo stack has one entry
        assert_eq!(window.state.undo_stack.len(), 1);
    }

    #[test]
    fn test_drag_mode_states() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        // Initial state should be None
        assert_eq!(window.state.drag_mode, DragMode::None);
        
        // Test setting different drag modes
        window.state.drag_mode = DragMode::Creating;
        assert_eq!(window.state.drag_mode, DragMode::Creating);
        
        window.state.drag_mode = DragMode::MovingSprite(0);
        assert!(matches!(window.state.drag_mode, DragMode::MovingSprite(0)));
        
        window.state.drag_mode = DragMode::ResizingSprite(0, ResizeHandle::TopLeft);
        assert!(matches!(window.state.drag_mode, DragMode::ResizingSprite(0, ResizeHandle::TopLeft)));
    }

    #[test]
    fn test_sprite_selection_state() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Initially no sprite selected
        assert_eq!(window.state.selected_sprite, None);
        
        // Add sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 50, 50, 32, 32));
        
        // Select first sprite
        window.state.selected_sprite = Some(0);
        assert_eq!(window.state.selected_sprite, Some(0));
        
        // Select second sprite
        window.state.selected_sprite = Some(1);
        assert_eq!(window.state.selected_sprite, Some(1));
        
        // Deselect
        window.state.selected_sprite = None;
        assert_eq!(window.state.selected_sprite, None);
    }

    #[test]
    fn test_hover_state() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Initially no sprite hovered
        assert_eq!(window.state.hovered_sprite, None);
        
        // Add sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Set hover state
        window.state.hovered_sprite = Some(0);
        assert_eq!(window.state.hovered_sprite, Some(0));
        
        // Clear hover state
        window.state.hovered_sprite = None;
        assert_eq!(window.state.hovered_sprite, None);
    }

    #[test]
    fn test_sprite_creation_does_not_start_when_clicking_on_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add existing sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Verify we have 1 sprite
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        // The sprite creation logic should not start drawing when clicking on an existing sprite
        // This is tested implicitly by the find_sprite_at_position logic
        let texture_pos = egui::pos2(0.0, 0.0);
        let clicked_sprite = window.find_sprite_at_position(egui::pos2(16.0, 16.0), texture_pos);
        assert_eq!(clicked_sprite, Some(0), "Should find existing sprite at click position");
    }
