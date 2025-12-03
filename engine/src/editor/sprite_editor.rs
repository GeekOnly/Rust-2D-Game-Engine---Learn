use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use egui::TextureHandle;

/// Export format for sprite metadata
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Xml,
    TexturePacker,
}

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

    /// Export sprite metadata to a file in the specified format
    pub fn export<P: AsRef<Path>>(&self, path: P, format: ExportFormat) -> Result<(), String> {
        let path = path.as_ref();
        
        let content = match format {
            ExportFormat::Json => self.export_to_json()?,
            ExportFormat::Xml => self.export_to_xml()?,
            ExportFormat::TexturePacker => self.export_to_texture_packer()?,
        };

        // Write to file
        fs::write(path, content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        Ok(())
    }

    /// Export to standard JSON format
    fn export_to_json(&self) -> Result<String, String> {
        // Use the same format as our internal .sprite files
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

    /// Export to XML format
    fn export_to_xml(&self) -> Result<String, String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<SpriteSheet>\n");
        xml.push_str(&format!("  <TexturePath>{}</TexturePath>\n", self.texture_path));
        xml.push_str(&format!("  <TextureWidth>{}</TextureWidth>\n", self.texture_width));
        xml.push_str(&format!("  <TextureHeight>{}</TextureHeight>\n", self.texture_height));
        xml.push_str("  <Sprites>\n");
        
        for sprite in &self.sprites {
            xml.push_str("    <Sprite>\n");
            xml.push_str(&format!("      <Name>{}</Name>\n", sprite.name));
            xml.push_str(&format!("      <X>{}</X>\n", sprite.x));
            xml.push_str(&format!("      <Y>{}</Y>\n", sprite.y));
            xml.push_str(&format!("      <Width>{}</Width>\n", sprite.width));
            xml.push_str(&format!("      <Height>{}</Height>\n", sprite.height));
            xml.push_str("    </Sprite>\n");
        }
        
        xml.push_str("  </Sprites>\n");
        xml.push_str("</SpriteSheet>\n");
        
        Ok(xml)
    }

    /// Export to TexturePacker format (JSON)
    fn export_to_texture_packer(&self) -> Result<String, String> {
        // TexturePacker format structure
        let mut tp_data = serde_json::json!({
            "frames": {},
            "meta": {
                "app": "XS Game Engine Sprite Editor",
                "version": "1.0",
                "image": self.texture_path,
                "format": "RGBA8888",
                "size": {
                    "w": self.texture_width,
                    "h": self.texture_height
                },
                "scale": "1"
            }
        });

        // Add each sprite as a frame
        if let Some(frames) = tp_data.get_mut("frames") {
            if let Some(frames_obj) = frames.as_object_mut() {
                for sprite in &self.sprites {
                    let frame_data = serde_json::json!({
                        "frame": {
                            "x": sprite.x,
                            "y": sprite.y,
                            "w": sprite.width,
                            "h": sprite.height
                        },
                        "rotated": false,
                        "trimmed": false,
                        "spriteSourceSize": {
                            "x": 0,
                            "y": 0,
                            "w": sprite.width,
                            "h": sprite.height
                        },
                        "sourceSize": {
                            "w": sprite.width,
                            "h": sprite.height
                        }
                    });
                    
                    frames_obj.insert(sprite.name.clone(), frame_data);
                }
            }
        }

        serde_json::to_string_pretty(&tp_data)
            .map_err(|e| format!("Failed to serialize to TexturePacker format: {}", e))
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

/// Statistics and validation results for sprite sheet
#[derive(Debug, Clone, Default)]
pub struct SpriteStatistics {
    pub sprite_count: usize,
    pub texture_coverage_percent: f32,
    pub overlapping_sprites: Vec<(usize, usize)>,
    pub out_of_bounds_sprites: Vec<usize>,
}

impl SpriteStatistics {
    /// Calculate statistics for a sprite metadata
    pub fn calculate(metadata: &SpriteMetadata) -> Self {
        let sprite_count = metadata.sprites.len();
        
        // Calculate texture coverage
        let texture_area = (metadata.texture_width * metadata.texture_height) as f32;
        let total_sprite_area: u32 = metadata.sprites.iter()
            .map(|s| s.width * s.height)
            .sum();
        let texture_coverage_percent = if texture_area > 0.0 {
            (total_sprite_area as f32 / texture_area) * 100.0
        } else {
            0.0
        };
        
        // Detect overlapping sprites
        let mut overlapping_sprites = Vec::new();
        for i in 0..metadata.sprites.len() {
            for j in (i + 1)..metadata.sprites.len() {
                if Self::sprites_overlap(&metadata.sprites[i], &metadata.sprites[j]) {
                    overlapping_sprites.push((i, j));
                }
            }
        }
        
        // Detect out-of-bounds sprites
        let mut out_of_bounds_sprites = Vec::new();
        for (idx, sprite) in metadata.sprites.iter().enumerate() {
            if sprite.x + sprite.width > metadata.texture_width
                || sprite.y + sprite.height > metadata.texture_height
            {
                out_of_bounds_sprites.push(idx);
            }
        }
        
        Self {
            sprite_count,
            texture_coverage_percent,
            overlapping_sprites,
            out_of_bounds_sprites,
        }
    }
    
    /// Check if two sprites overlap
    fn sprites_overlap(sprite1: &SpriteDefinition, sprite2: &SpriteDefinition) -> bool {
        let s1_left = sprite1.x;
        let s1_right = sprite1.x + sprite1.width;
        let s1_top = sprite1.y;
        let s1_bottom = sprite1.y + sprite1.height;
        
        let s2_left = sprite2.x;
        let s2_right = sprite2.x + sprite2.width;
        let s2_top = sprite2.y;
        let s2_bottom = sprite2.y + sprite2.height;
        
        // Check if rectangles overlap
        !(s1_right <= s2_left || s2_right <= s1_left || s1_bottom <= s2_top || s2_bottom <= s1_top)
    }
    
    /// Check if there are any warnings or errors
    pub fn has_issues(&self) -> bool {
        !self.overlapping_sprites.is_empty() || !self.out_of_bounds_sprites.is_empty()
    }
}

/// Auto-slicer for grid-based sprite slicing
pub struct AutoSlicer;

impl AutoSlicer {
    /// Slice a texture into a grid of sprites
    /// 
    /// # Arguments
    /// * `texture_width` - Width of the texture in pixels
    /// * `texture_height` - Height of the texture in pixels
    /// * `columns` - Number of columns in the grid
    /// * `rows` - Number of rows in the grid
    /// * `padding` - Padding from the edges of the texture in pixels
    /// * `spacing` - Spacing between sprites in pixels
    /// 
    /// # Returns
    /// A vector of sprite definitions arranged in a grid
    pub fn slice_by_grid(
        texture_width: u32,
        texture_height: u32,
        columns: u32,
        rows: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition> {
        if columns == 0 || rows == 0 {
            return Vec::new();
        }
        
        // Calculate available space after accounting for padding
        let available_width = texture_width.saturating_sub(padding * 2);
        let available_height = texture_height.saturating_sub(padding * 2);
        
        // Calculate total spacing
        let total_horizontal_spacing = spacing * (columns - 1);
        let total_vertical_spacing = spacing * (rows - 1);
        
        // Calculate sprite dimensions
        let sprite_width = available_width.saturating_sub(total_horizontal_spacing) / columns;
        let sprite_height = available_height.saturating_sub(total_vertical_spacing) / rows;
        
        // Validate sprite dimensions
        if sprite_width == 0 || sprite_height == 0 {
            return Vec::new();
        }
        
        let mut sprites = Vec::new();
        let mut sprite_index = 0;
        
        for row in 0..rows {
            for col in 0..columns {
                // Calculate sprite position
                let x = padding + (col * (sprite_width + spacing));
                let y = padding + (row * (sprite_height + spacing));
                
                // Create sprite with sequential name
                let sprite = SpriteDefinition::new(
                    format!("sprite_{}", sprite_index),
                    x,
                    y,
                    sprite_width,
                    sprite_height,
                );
                
                sprites.push(sprite);
                sprite_index += 1;
            }
        }
        
        sprites
    }
    
    /// Slice a texture by cell size
    /// 
    /// # Arguments
    /// * `texture_width` - Width of the texture in pixels
    /// * `texture_height` - Height of the texture in pixels
    /// * `cell_width` - Width of each sprite cell in pixels
    /// * `cell_height` - Height of each sprite cell in pixels
    /// * `padding` - Padding from the edges of the texture in pixels
    /// * `spacing` - Spacing between sprites in pixels
    /// 
    /// # Returns
    /// A vector of sprite definitions based on cell size
    pub fn slice_by_cell_size(
        texture_width: u32,
        texture_height: u32,
        cell_width: u32,
        cell_height: u32,
        padding: u32,
        spacing: u32,
    ) -> Vec<SpriteDefinition> {
        if cell_width == 0 || cell_height == 0 {
            return Vec::new();
        }
        
        // Calculate available space after accounting for padding
        let available_width = texture_width.saturating_sub(padding * 2);
        let available_height = texture_height.saturating_sub(padding * 2);
        
        // Calculate how many sprites fit
        let columns = if spacing > 0 {
            (available_width + spacing) / (cell_width + spacing)
        } else {
            available_width / cell_width
        };
        
        let rows = if spacing > 0 {
            (available_height + spacing) / (cell_height + spacing)
        } else {
            available_height / cell_height
        };
        
        if columns == 0 || rows == 0 {
            return Vec::new();
        }
        
        let mut sprites = Vec::new();
        let mut sprite_index = 0;
        
        for row in 0..rows {
            for col in 0..columns {
                // Calculate sprite position
                let x = padding + (col * (cell_width + spacing));
                let y = padding + (row * (cell_height + spacing));
                
                // Ensure sprite doesn't exceed texture bounds
                if x + cell_width <= texture_width && y + cell_height <= texture_height {
                    let sprite = SpriteDefinition::new(
                        format!("sprite_{}", sprite_index),
                        x,
                        y,
                        cell_width,
                        cell_height,
                    );
                    
                    sprites.push(sprite);
                    sprite_index += 1;
                }
            }
        }
        
        sprites
    }
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
    
    // Hot-reloading
    pub last_modified: Option<SystemTime>,
    pub check_interval: f32,
    pub time_since_check: f32,
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
        
        // Get initial modification time
        let last_modified = if metadata_path.exists() {
            fs::metadata(&metadata_path)
                .and_then(|m| m.modified())
                .ok()
        } else {
            None
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
            last_modified,
            check_interval: 1.0, // Check every 1 second
            time_since_check: 0.0,
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
    
    /// Check if the sprite file has been modified and reload if necessary
    /// Returns true if the file was reloaded
    pub fn check_and_reload(&mut self, dt: f32) -> bool {
        // Update timer
        self.time_since_check += dt;
        
        // Only check at intervals to avoid excessive file system calls
        if self.time_since_check < self.check_interval {
            return false;
        }
        
        self.time_since_check = 0.0;
        
        // Check if file exists
        if !self.metadata_path.exists() {
            return false;
        }
        
        // Get current modification time
        let current_modified = match fs::metadata(&self.metadata_path)
            .and_then(|m| m.modified())
        {
            Ok(time) => time,
            Err(_) => return false,
        };
        
        // Check if file has been modified since last check
        if let Some(last_modified) = self.last_modified {
            if current_modified > last_modified {
                // File has been modified, reload it
                match SpriteMetadata::load(&self.metadata_path) {
                    Ok(new_metadata) => {
                        log::info!("Hot-reloaded sprite metadata from {:?}", self.metadata_path);
                        self.metadata = new_metadata;
                        self.last_modified = Some(current_modified);
                        
                        // Clear selection if it's now out of bounds
                        if let Some(selected_idx) = self.selected_sprite {
                            if selected_idx >= self.metadata.sprites.len() {
                                self.selected_sprite = None;
                            }
                        }
                        
                        return true;
                    }
                    Err(e) => {
                        log::warn!("Failed to reload sprite metadata: {}", e);
                        return false;
                    }
                }
            }
        } else {
            // First time checking, just store the modification time
            self.last_modified = Some(current_modified);
        }
        
        false
    }
}

/// Sprite Editor Window
pub struct SpriteEditorWindow {
    pub state: SpriteEditorState,
    pub is_open: bool,
    /// Temporary buffer for editing sprite name
    name_edit_buffer: String,
    /// Whether there's a duplicate name error
    duplicate_name_error: bool,
    /// Auto-slice dialog state
    show_auto_slice_dialog: bool,
    auto_slice_columns: u32,
    auto_slice_rows: u32,
    auto_slice_padding: u32,
    auto_slice_spacing: u32,
    auto_slice_mode: AutoSliceMode,
    auto_slice_cell_width: u32,
    auto_slice_cell_height: u32,
    /// Export dialog state
    show_export_dialog: bool,
    export_format: ExportFormat,
    export_message: Option<String>,
    export_error: Option<String>,
    /// Sprite statistics and validation
    statistics: SpriteStatistics,
}

/// Auto-slice mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum AutoSliceMode {
    Grid,
    CellSize,
}

impl SpriteEditorWindow {
    /// Create a new sprite editor window
    pub fn new(texture_path: PathBuf) -> Self {
        let state = SpriteEditorState::new(texture_path);
        let statistics = SpriteStatistics::calculate(&state.metadata);
        
        Self {
            state,
            is_open: true,
            name_edit_buffer: String::new(),
            duplicate_name_error: false,
            show_auto_slice_dialog: false,
            auto_slice_columns: 4,
            auto_slice_rows: 4,
            auto_slice_padding: 0,
            auto_slice_spacing: 0,
            auto_slice_mode: AutoSliceMode::Grid,
            auto_slice_cell_width: 32,
            auto_slice_cell_height: 32,
            show_export_dialog: false,
            export_format: ExportFormat::Json,
            export_message: None,
            export_error: None,
            statistics,
        }
    }
    
    /// Update statistics based on current metadata
    fn update_statistics(&mut self) {
        self.statistics = SpriteStatistics::calculate(&self.state.metadata);
    }
    
    /// Render the sprite editor window
    pub fn render(&mut self, ctx: &egui::Context, texture_manager: &mut crate::texture_manager::TextureManager, dt: f32) {
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
        
        // Check for file changes and reload if necessary
        if self.state.check_and_reload(dt) {
            // File was reloaded, update statistics
            self.update_statistics();
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
        
        // Render auto-slice dialog if open
        if self.show_auto_slice_dialog {
            self.render_auto_slice_dialog(ctx);
        }
        
        // Render export dialog if open
        if self.show_export_dialog {
            self.render_export_dialog(ctx);
        }
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
                self.update_statistics();
            }
            
            // Handle Ctrl+Y to redo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                self.state.redo();
                self.update_statistics();
            }
            
            // Handle Escape to deselect
            if i.key_pressed(egui::Key::Escape) {
                self.state.selected_sprite = None;
            }
            
            // Handle Tab to cycle through overlapping sprites
            if i.key_pressed(egui::Key::Tab) {
                self.cycle_overlapping_sprites();
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
            
            // Update statistics
            self.update_statistics();
            
            log::info!("Deleted sprite at index {}", selected_idx);
        }
    }
    
    /// Cycle through overlapping sprites at the current selection
    fn cycle_overlapping_sprites(&mut self) {
        // Only cycle if we have a selected sprite
        if let Some(current_idx) = self.state.selected_sprite {
            if let Some(current_sprite) = self.state.metadata.sprites.get(current_idx) {
                // Find all sprites that overlap with the current selection
                let overlapping: Vec<usize> = self.state.metadata.sprites
                    .iter()
                    .enumerate()
                    .filter(|(idx, sprite)| {
                        // Check if sprite overlaps with current sprite
                        *idx != current_idx && self.sprites_overlap(current_sprite, sprite)
                    })
                    .map(|(idx, _)| idx)
                    .collect();
                
                if !overlapping.is_empty() {
                    // Include current sprite in the cycle
                    let mut cycle_list = vec![current_idx];
                    cycle_list.extend(overlapping);
                    cycle_list.sort();
                    
                    // Find current position in cycle and move to next
                    if let Some(pos) = cycle_list.iter().position(|&idx| idx == current_idx) {
                        let next_pos = (pos + 1) % cycle_list.len();
                        self.state.selected_sprite = Some(cycle_list[next_pos]);
                        log::info!("Cycled to sprite at index {}", cycle_list[next_pos]);
                    }
                }
            }
        }
    }
    
    /// Check if two sprites overlap
    fn sprites_overlap(&self, sprite1: &SpriteDefinition, sprite2: &SpriteDefinition) -> bool {
        let s1_left = sprite1.x;
        let s1_right = sprite1.x + sprite1.width;
        let s1_top = sprite1.y;
        let s1_bottom = sprite1.y + sprite1.height;
        
        let s2_left = sprite2.x;
        let s2_right = sprite2.x + sprite2.width;
        let s2_top = sprite2.y;
        let s2_bottom = sprite2.y + sprite2.height;
        
        // Check if rectangles overlap
        !(s1_right <= s2_left || s2_right <= s1_left || s1_bottom <= s2_top || s2_bottom <= s1_top)
    }
    
    /// Render the window content
    fn render_content(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("💾 Save (Ctrl+S)").clicked() {
                match self.state.save() {
                    Ok(_) => {
                        log::info!("Sprite metadata saved successfully");
                        self.export_message = Some("Saved successfully!".to_string());
                        self.export_error = None;
                    }
                    Err(e) => {
                        log::error!("Failed to save sprite metadata: {}", e);
                        self.export_error = Some(format!("Save failed: {}", e));
                        self.export_message = None;
                    }
                }
            }
            
            ui.separator();
            
            if ui.button("✂ Auto Slice").clicked() {
                self.show_auto_slice_dialog = true;
            }
            
            ui.separator();
            
            if ui.button("📤 Export").clicked() {
                self.show_export_dialog = true;
                self.export_message = None;
                self.export_error = None;
            }
            
            ui.separator();
            
            if ui.button("↶ Undo (Ctrl+Z)").clicked() {
                self.state.undo();
                self.update_statistics();
            }
            
            if ui.button("↷ Redo (Ctrl+Y)").clicked() {
                self.state.redo();
                self.update_statistics();
            }
            
            ui.separator();
            
            ui.label(format!("Zoom: {:.0}%", self.state.zoom * 100.0));
        });
        
        ui.separator();
        
        // Keyboard shortcuts hint panel
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("⌨ Shortcuts:")
                    .small()
                    .color(egui::Color32::GRAY)
            );
            ui.label(
                egui::RichText::new("Delete: Remove sprite")
                    .small()
                    .color(egui::Color32::LIGHT_GRAY)
            );
            ui.separator();
            ui.label(
                egui::RichText::new("Esc: Deselect")
                    .small()
                    .color(egui::Color32::LIGHT_GRAY)
            );
            ui.separator();
            ui.label(
                egui::RichText::new("Tab: Cycle overlapping")
                    .small()
                    .color(egui::Color32::LIGHT_GRAY)
            );
        });
        
        ui.separator();
        
        // Main content area
        ui.horizontal(|ui| {
            // Left panel - Sprite list
            ui.vertical(|ui| {
                ui.set_width(200.0);
                
                // Header with sprite count
                ui.horizontal(|ui| {
                    ui.heading("Sprites");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", self.state.metadata.sprites.len()))
                                .strong()
                                .color(egui::Color32::from_rgb(100, 150, 255))
                        );
                    });
                });
                
                ui.separator();
                
                // Scrollable sprite list with thumbnails
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_sprite_list(ui);
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
            
            // Right panel - Properties and Statistics
            ui.vertical(|ui| {
                ui.set_width(300.0);
                ui.heading("Properties");
                ui.separator();
                
                self.render_properties_panel(ui);
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.heading("Statistics");
                ui.separator();
                
                self.render_statistics_panel(ui);
            });
        });
        
        // Status bar with statistics
        ui.separator();
        ui.horizontal(|ui| {
            // Sprite count
            ui.label(format!("Sprites: {}", self.statistics.sprite_count));
            ui.separator();
            
            // Texture dimensions
            ui.label(format!(
                "Texture: {}x{}",
                self.state.metadata.texture_width,
                self.state.metadata.texture_height
            ));
            ui.separator();
            
            // Coverage percentage
            ui.label(format!(
                "Coverage: {:.1}%",
                self.statistics.texture_coverage_percent
            ));
            
            // Warnings and errors
            if !self.statistics.overlapping_sprites.is_empty() {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 100),
                    format!("⚠ {} overlapping", self.statistics.overlapping_sprites.len())
                );
            }
            
            if !self.statistics.out_of_bounds_sprites.is_empty() {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("❌ {} out of bounds", self.statistics.out_of_bounds_sprites.len())
                );
            }
        });
    }
    
    /// Render the auto-slice dialog
    fn render_auto_slice_dialog(&mut self, ctx: &egui::Context) {
        let mut dialog_open = self.show_auto_slice_dialog;
        
        egui::Window::new("✂ Auto Slice")
            .open(&mut dialog_open)
            .resizable(false)
            .collapsible(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Grid Slicing Options");
                ui.add_space(10.0);
                
                // Mode selection
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    ui.radio_value(&mut self.auto_slice_mode, AutoSliceMode::Grid, "Grid (Columns × Rows)");
                    ui.radio_value(&mut self.auto_slice_mode, AutoSliceMode::CellSize, "Cell Size");
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                match self.auto_slice_mode {
                    AutoSliceMode::Grid => {
                        // Grid mode: specify columns and rows
                        ui.horizontal(|ui| {
                            ui.label("Columns:");
                            ui.add(egui::DragValue::new(&mut self.auto_slice_columns).clamp_range(1..=100).speed(0.1));
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Rows:");
                            ui.add(egui::DragValue::new(&mut self.auto_slice_rows).clamp_range(1..=100).speed(0.1));
                        });
                        
                        // Calculate and display sprite dimensions
                        let texture_width = self.state.metadata.texture_width;
                        let texture_height = self.state.metadata.texture_height;
                        
                        if texture_width > 0 && texture_height > 0 {
                            let available_width = texture_width.saturating_sub(self.auto_slice_padding * 2);
                            let available_height = texture_height.saturating_sub(self.auto_slice_padding * 2);
                            
                            let total_h_spacing = self.auto_slice_spacing * (self.auto_slice_columns.saturating_sub(1));
                            let total_v_spacing = self.auto_slice_spacing * (self.auto_slice_rows.saturating_sub(1));
                            
                            let sprite_width = available_width.saturating_sub(total_h_spacing) / self.auto_slice_columns.max(1);
                            let sprite_height = available_height.saturating_sub(total_v_spacing) / self.auto_slice_rows.max(1);
                            
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new(format!("→ Sprite size: {}×{} px", sprite_width, sprite_height))
                                    .color(egui::Color32::from_rgb(100, 200, 255))
                            );
                        }
                    }
                    AutoSliceMode::CellSize => {
                        // Cell size mode: specify width and height
                        ui.horizontal(|ui| {
                            ui.label("Cell Width:");
                            ui.add(egui::DragValue::new(&mut self.auto_slice_cell_width).clamp_range(1..=1024).speed(1.0));
                            ui.label("px");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Cell Height:");
                            ui.add(egui::DragValue::new(&mut self.auto_slice_cell_height).clamp_range(1..=1024).speed(1.0));
                            ui.label("px");
                        });
                        
                        // Calculate and display how many sprites will be created
                        let texture_width = self.state.metadata.texture_width;
                        let texture_height = self.state.metadata.texture_height;
                        
                        if texture_width > 0 && texture_height > 0 {
                            let available_width = texture_width.saturating_sub(self.auto_slice_padding * 2);
                            let available_height = texture_height.saturating_sub(self.auto_slice_padding * 2);
                            
                            let columns = if self.auto_slice_spacing > 0 {
                                (available_width + self.auto_slice_spacing) / (self.auto_slice_cell_width + self.auto_slice_spacing)
                            } else {
                                available_width / self.auto_slice_cell_width
                            };
                            
                            let rows = if self.auto_slice_spacing > 0 {
                                (available_height + self.auto_slice_spacing) / (self.auto_slice_cell_height + self.auto_slice_spacing)
                            } else {
                                available_height / self.auto_slice_cell_height
                            };
                            
                            let total_sprites = columns * rows;
                            
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new(format!("→ Will create {} sprites ({}×{})", total_sprites, columns, rows))
                                    .color(egui::Color32::from_rgb(100, 200, 255))
                            );
                        }
                    }
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Common options
                ui.horizontal(|ui| {
                    ui.label("Padding:");
                    ui.add(egui::DragValue::new(&mut self.auto_slice_padding).clamp_range(0..=100).speed(0.1));
                    ui.label("px");
                });
                ui.label(
                    egui::RichText::new("Padding from texture edges")
                        .small()
                        .color(egui::Color32::GRAY)
                );
                
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label("Spacing:");
                    ui.add(egui::DragValue::new(&mut self.auto_slice_spacing).clamp_range(0..=100).speed(0.1));
                    ui.label("px");
                });
                ui.label(
                    egui::RichText::new("Space between sprites")
                        .small()
                        .color(egui::Color32::GRAY)
                );
                
                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("✂ Slice").clicked() {
                        self.apply_auto_slice();
                        self.show_auto_slice_dialog = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_auto_slice_dialog = false;
                    }
                });
                
                ui.add_space(5.0);
                
                // Warning if sprites already exist
                if !self.state.metadata.sprites.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        format!("⚠ This will replace {} existing sprite(s)", self.state.metadata.sprites.len())
                    );
                }
            });
        
        self.show_auto_slice_dialog = dialog_open;
    }
    
    /// Render the export dialog
    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        let mut dialog_open = self.show_export_dialog;
        
        egui::Window::new("📤 Export Sprite Sheet")
            .open(&mut dialog_open)
            .resizable(false)
            .collapsible(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Export Options");
                ui.add_space(10.0);
                
                // Format selection
                ui.label("Export Format:");
                ui.add_space(5.0);
                
                ui.radio_value(&mut self.export_format, ExportFormat::Json, "JSON (Standard)");
                ui.label(
                    egui::RichText::new("Standard JSON format compatible with most tools")
                        .small()
                        .color(egui::Color32::GRAY)
                );
                ui.add_space(5.0);
                
                ui.radio_value(&mut self.export_format, ExportFormat::Xml, "XML");
                ui.label(
                    egui::RichText::new("XML format for legacy tools and engines")
                        .small()
                        .color(egui::Color32::GRAY)
                );
                ui.add_space(5.0);
                
                ui.radio_value(&mut self.export_format, ExportFormat::TexturePacker, "TexturePacker");
                ui.label(
                    egui::RichText::new("TexturePacker JSON format for compatibility")
                        .small()
                        .color(egui::Color32::GRAY)
                );
                
                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Show sprite count
                ui.horizontal(|ui| {
                    ui.label("Sprites to export:");
                    ui.label(
                        egui::RichText::new(format!("{}", self.state.metadata.sprites.len()))
                            .strong()
                            .color(egui::Color32::from_rgb(100, 200, 255))
                    );
                });
                
                ui.add_space(10.0);
                
                // Show success/error messages
                if let Some(msg) = &self.export_message {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 255, 100),
                        format!("✓ {}", msg)
                    );
                    ui.add_space(5.0);
                }
                
                if let Some(err) = &self.export_error {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        format!("❌ {}", err)
                    );
                    ui.add_space(5.0);
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("📤 Export").clicked() {
                        self.perform_export();
                    }
                    
                    if ui.button("Close").clicked() {
                        self.show_export_dialog = false;
                    }
                });
                
                ui.add_space(5.0);
                
                // Warning if no sprites
                if self.state.metadata.sprites.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        "⚠ No sprites to export"
                    );
                }
            });
        
        self.show_export_dialog = dialog_open;
    }
    
    /// Perform the export operation
    fn perform_export(&mut self) {
        // Determine export file path based on format
        let extension = match self.export_format {
            ExportFormat::Json => "json",
            ExportFormat::Xml => "xml",
            ExportFormat::TexturePacker => "json",
        };
        
        // Create export filename based on texture name
        let export_path = self.state.metadata_path
            .with_file_name(format!(
                "{}_export.{}",
                self.state.metadata_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sprite_sheet"),
                extension
            ));
        
        // Perform export
        match self.state.metadata.export(&export_path, self.export_format) {
            Ok(_) => {
                let format_name = match self.export_format {
                    ExportFormat::Json => "JSON",
                    ExportFormat::Xml => "XML",
                    ExportFormat::TexturePacker => "TexturePacker",
                };
                
                let message = format!(
                    "Exported {} sprites to {} format: {}",
                    self.state.metadata.sprites.len(),
                    format_name,
                    export_path.display()
                );
                
                log::info!("{}", message);
                self.export_message = Some(message);
                self.export_error = None;
            }
            Err(e) => {
                let error = format!("Export failed: {}", e);
                log::error!("{}", error);
                self.export_error = Some(error);
                self.export_message = None;
            }
        }
    }
    
    /// Apply auto-slice based on current settings
    fn apply_auto_slice(&mut self) {
        // Push current state to undo stack before slicing
        self.state.push_undo();
        
        let texture_width = self.state.metadata.texture_width;
        let texture_height = self.state.metadata.texture_height;
        
        // Generate sprites based on mode
        let sprites = match self.auto_slice_mode {
            AutoSliceMode::Grid => {
                AutoSlicer::slice_by_grid(
                    texture_width,
                    texture_height,
                    self.auto_slice_columns,
                    self.auto_slice_rows,
                    self.auto_slice_padding,
                    self.auto_slice_spacing,
                )
            }
            AutoSliceMode::CellSize => {
                AutoSlicer::slice_by_cell_size(
                    texture_width,
                    texture_height,
                    self.auto_slice_cell_width,
                    self.auto_slice_cell_height,
                    self.auto_slice_padding,
                    self.auto_slice_spacing,
                )
            }
        };
        
        // Replace existing sprites with new ones
        self.state.metadata.sprites = sprites;
        
        // Clear selection
        self.state.selected_sprite = None;
        
        // Update statistics
        self.update_statistics();
        
        log::info!("Auto-slice created {} sprites", self.state.metadata.sprites.len());
    }
    
    /// Render the sprite list panel with thumbnails
    fn render_sprite_list(&mut self, ui: &mut egui::Ui) {
        if self.state.metadata.sprites.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("No sprites yet")
                        .color(egui::Color32::GRAY)
                        .italics()
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Click and drag on the canvas\nto create sprite regions")
                        .small()
                        .color(egui::Color32::DARK_GRAY)
                );
            });
            return;
        }
        
        // Render each sprite as a list item with thumbnail
        for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
            let is_selected = self.state.selected_sprite == Some(idx);
            
            // Create a frame for each sprite item
            let frame = if is_selected {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(60, 90, 150))
                    .inner_margin(egui::Margin::same(4.0))
                    .rounding(4.0)
            } else {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(40, 40, 45))
                    .inner_margin(egui::Margin::same(4.0))
                    .rounding(4.0)
            };
            
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Render thumbnail
                    if let Some(texture_handle) = &self.state.texture_handle {
                        let texture_size = texture_handle.size();
                        
                        // Calculate UV coordinates for the sprite region
                        let uv_min = egui::pos2(
                            sprite.x as f32 / texture_size[0] as f32,
                            sprite.y as f32 / texture_size[1] as f32,
                        );
                        let uv_max = egui::pos2(
                            (sprite.x + sprite.width) as f32 / texture_size[0] as f32,
                            (sprite.y + sprite.height) as f32 / texture_size[1] as f32,
                        );
                        
                        // Calculate thumbnail size (48x48 max, maintain aspect ratio)
                        let thumbnail_size = 48.0;
                        let aspect_ratio = sprite.width as f32 / sprite.height as f32;
                        let (thumb_width, thumb_height) = if aspect_ratio > 1.0 {
                            (thumbnail_size, thumbnail_size / aspect_ratio)
                        } else {
                            (thumbnail_size * aspect_ratio, thumbnail_size)
                        };
                        
                        // Allocate space for thumbnail
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(thumbnail_size, thumbnail_size),
                            egui::Sense::hover()
                        );
                        
                        // Center the thumbnail in the allocated space
                        let thumb_rect = egui::Rect::from_center_size(
                            rect.center(),
                            egui::vec2(thumb_width, thumb_height)
                        );
                        
                        // Draw thumbnail
                        ui.painter().image(
                            texture_handle.id(),
                            thumb_rect,
                            egui::Rect::from_min_max(uv_min, uv_max),
                            egui::Color32::WHITE,
                        );
                        
                        // Draw border around thumbnail
                        ui.painter().rect_stroke(
                            thumb_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 85)),
                        );
                    } else {
                        // Placeholder if texture not loaded
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(48.0, 48.0),
                            egui::Sense::hover()
                        );
                        ui.painter().rect_filled(
                            rect,
                            2.0,
                            egui::Color32::from_rgb(60, 60, 65)
                        );
                    }
                    
                    ui.add_space(8.0);
                    
                    // Sprite info
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        
                        // Sprite name
                        let name_text = if is_selected {
                            egui::RichText::new(&sprite.name)
                                .strong()
                                .color(egui::Color32::WHITE)
                        } else {
                            egui::RichText::new(&sprite.name)
                                .color(egui::Color32::LIGHT_GRAY)
                        };
                        ui.label(name_text);
                        
                        // Sprite dimensions
                        ui.label(
                            egui::RichText::new(format!("{}×{}", sprite.width, sprite.height))
                                .small()
                                .color(egui::Color32::GRAY)
                        );
                        
                        // Sprite position
                        ui.label(
                            egui::RichText::new(format!("({}, {})", sprite.x, sprite.y))
                                .small()
                                .color(egui::Color32::DARK_GRAY)
                        );
                    });
                });
            });
            
            // Handle click to select sprite
            let item_response = ui.interact(
                ui.min_rect(),
                ui.id().with(idx),
                egui::Sense::click()
            );
            
            if item_response.clicked() {
                self.state.selected_sprite = Some(idx);
            }
            
            // Add hover effect
            if item_response.hovered() {
                ui.painter().rect_stroke(
                    ui.min_rect(),
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255))
                );
            }
            
            ui.add_space(4.0);
        }
    }
    
    /// Render the properties panel for the selected sprite
    fn render_properties_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(idx) = self.state.selected_sprite {
            if let Some(sprite) = self.state.metadata.sprites.get(idx).cloned() {
                // Initialize name buffer if empty or if selection changed
                if self.name_edit_buffer.is_empty() || self.name_edit_buffer != sprite.name {
                    self.name_edit_buffer = sprite.name.clone();
                    self.duplicate_name_error = false;
                }
                
                // Sprite name editing
                ui.label("Name:");
                let name_response = ui.text_edit_singleline(&mut self.name_edit_buffer);
                
                // Check for duplicate names when editing
                if name_response.changed() {
                    // Check if the new name is a duplicate (excluding the current sprite)
                    let is_duplicate = self.state.metadata.sprites.iter().enumerate()
                        .any(|(i, s)| i != idx && s.name == self.name_edit_buffer);
                    
                    self.duplicate_name_error = is_duplicate;
                    
                    // Update sprite name if not duplicate and not empty
                    if !is_duplicate && !self.name_edit_buffer.trim().is_empty() {
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.name = self.name_edit_buffer.clone();
                            // Note: Name changes don't affect statistics (coverage, overlaps, bounds)
                            // so we don't need to update statistics here
                        }
                    }
                }
                
                // Show warning for duplicate names
                if self.duplicate_name_error {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        "⚠ Duplicate name! Please choose a unique name."
                    );
                }
                
                // Show warning for empty names
                if self.name_edit_buffer.trim().is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        "⚠ Name cannot be empty"
                    );
                }
                
                ui.add_space(10.0);
                
                // Display sprite properties (read-only)
                ui.label("Position & Size:");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.label(format!("{} px", sprite.x));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.label(format!("{} px", sprite.y));
                });
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.label(format!("{} px", sprite.width));
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.label(format!("{} px", sprite.height));
                });
                
                ui.add_space(10.0);
                
                // Display sprite dimensions
                ui.label("Dimensions:");
                ui.label(format!("{}×{} pixels", sprite.width, sprite.height));
                
                ui.add_space(10.0);
                
                // Display sprite preview
                ui.label("Preview:");
                if let Some(texture_handle) = &self.state.texture_handle {
                    let texture_size = texture_handle.size();
                    
                    // Calculate UV coordinates for the sprite region
                    let uv_min = egui::pos2(
                        sprite.x as f32 / texture_size[0] as f32,
                        sprite.y as f32 / texture_size[1] as f32,
                    );
                    let uv_max = egui::pos2(
                        (sprite.x + sprite.width) as f32 / texture_size[0] as f32,
                        (sprite.y + sprite.height) as f32 / texture_size[1] as f32,
                    );
                    
                    // Calculate preview size (max 256x256, maintain aspect ratio)
                    let max_preview_size = 256.0;
                    let aspect_ratio = sprite.width as f32 / sprite.height as f32;
                    let (preview_width, preview_height) = if aspect_ratio > 1.0 {
                        (max_preview_size, max_preview_size / aspect_ratio)
                    } else {
                        (max_preview_size * aspect_ratio, max_preview_size)
                    };
                    
                    // Clamp to actual sprite size if smaller
                    let preview_width = preview_width.min(sprite.width as f32);
                    let preview_height = preview_height.min(sprite.height as f32);
                    
                    // Draw the preview with a border
                    let preview_rect = ui.allocate_space(egui::vec2(preview_width, preview_height));
                    
                    ui.painter().image(
                        texture_handle.id(),
                        preview_rect.1,
                        egui::Rect::from_min_max(uv_min, uv_max),
                        egui::Color32::WHITE,
                    );
                    
                    // Draw border around preview
                    ui.painter().rect_stroke(
                        preview_rect.1,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                    );
                } else {
                    ui.label("No texture loaded");
                }
            }
        } else {
            ui.label("No sprite selected");
            ui.add_space(10.0);
            ui.label("Select a sprite from the canvas or sprite list to view and edit its properties.");
        }
    }
    
    /// Render the statistics panel showing validation and metrics
    fn render_statistics_panel(&self, ui: &mut egui::Ui) {
        // Texture dimensions
        ui.label("Texture Dimensions:");
        ui.label(format!(
            "{}×{} pixels",
            self.state.metadata.texture_width,
            self.state.metadata.texture_height
        ));
        
        ui.add_space(10.0);
        
        // Sprite count
        ui.label("Sprite Count:");
        ui.label(format!("{} sprites", self.statistics.sprite_count));
        
        ui.add_space(10.0);
        
        // Coverage percentage
        ui.label("Texture Coverage:");
        ui.label(format!("{:.2}%", self.statistics.texture_coverage_percent));
        
        // Visual coverage bar
        let coverage_fraction = (self.statistics.texture_coverage_percent / 100.0).min(1.0);
        let bar_width = ui.available_width();
        let bar_height = 20.0;
        
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(bar_width, bar_height),
            egui::Sense::hover()
        );
        
        // Draw background
        ui.painter().rect_filled(
            rect,
            2.0,
            egui::Color32::from_rgb(40, 40, 45)
        );
        
        // Draw filled portion
        let filled_width = bar_width * coverage_fraction;
        let filled_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(filled_width, bar_height)
        );
        
        let coverage_color = if coverage_fraction > 0.9 {
            egui::Color32::from_rgb(255, 200, 100) // Warning: high coverage
        } else {
            egui::Color32::from_rgb(100, 200, 255) // Normal
        };
        
        ui.painter().rect_filled(
            filled_rect,
            2.0,
            coverage_color
        );
        
        // Draw border
        ui.painter().rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 85))
        );
        
        ui.add_space(10.0);
        
        // Validation warnings and errors
        if self.statistics.has_issues() {
            ui.separator();
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new("Validation Issues:")
                    .strong()
                    .color(egui::Color32::from_rgb(255, 200, 100))
            );
            ui.add_space(5.0);
        }
        
        // Overlapping sprites warning
        if !self.statistics.overlapping_sprites.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 100),
                format!("⚠ {} overlapping sprite pairs", self.statistics.overlapping_sprites.len())
            );
            
            // Show details in a collapsing section
            egui::CollapsingHeader::new("Show overlapping pairs")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for (idx1, idx2) in &self.statistics.overlapping_sprites {
                                if let (Some(sprite1), Some(sprite2)) = (
                                    self.state.metadata.sprites.get(*idx1),
                                    self.state.metadata.sprites.get(*idx2)
                                ) {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "• {} ↔ {}",
                                            sprite1.name,
                                            sprite2.name
                                        ))
                                        .small()
                                        .color(egui::Color32::LIGHT_GRAY)
                                    );
                                }
                            }
                        });
                });
            
            ui.add_space(5.0);
        }
        
        // Out-of-bounds sprites error
        if !self.statistics.out_of_bounds_sprites.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(255, 100, 100),
                format!("❌ {} sprites out of bounds", self.statistics.out_of_bounds_sprites.len())
            );
            
            // Show details in a collapsing section
            egui::CollapsingHeader::new("Show out-of-bounds sprites")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for idx in &self.statistics.out_of_bounds_sprites {
                                if let Some(sprite) = self.state.metadata.sprites.get(*idx) {
                                    let max_x = sprite.x + sprite.width;
                                    let max_y = sprite.y + sprite.height;
                                    let texture_w = self.state.metadata.texture_width;
                                    let texture_h = self.state.metadata.texture_height;
                                    
                                    let issue = if max_x > texture_w && max_y > texture_h {
                                        format!("extends beyond right ({}) and bottom ({})", max_x, max_y)
                                    } else if max_x > texture_w {
                                        format!("extends beyond right edge ({})", max_x)
                                    } else {
                                        format!("extends beyond bottom edge ({})", max_y)
                                    };
                                    
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "• {}: {}",
                                            sprite.name,
                                            issue
                                        ))
                                        .small()
                                        .color(egui::Color32::LIGHT_GRAY)
                                    );
                                }
                            }
                        });
                });
            
            ui.add_space(5.0);
        }
        
        // Show success message if no issues
        if !self.statistics.has_issues() && self.statistics.sprite_count > 0 {
            ui.add_space(5.0);
            ui.colored_label(
                egui::Color32::from_rgb(100, 255, 100),
                "✓ No validation issues"
            );
        }
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
                
                // Show tooltip for hovered sprite
                self.render_sprite_tooltip(&response, ui);
                
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
                                // Update statistics after sprite modification
                                self.update_statistics();
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
            
            // Update statistics
            self.update_statistics();
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
    
    /// Render tooltip for hovered sprite
    fn render_sprite_tooltip(&self, response: &egui::Response, ui: &mut egui::Ui) {
        // Only show tooltip if hovering over a sprite
        if let Some(hovered_idx) = self.state.hovered_sprite {
            if let Some(sprite) = self.state.metadata.sprites.get(hovered_idx) {
                // Show tooltip with sprite name when hovering
                if response.hovered() {
                    egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("sprite_tooltip"), |ui| {
                        ui.label(
                            egui::RichText::new(&sprite.name)
                                .color(egui::Color32::WHITE)
                                .size(14.0)
                        );
                    });
                }
            }
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

    // Export functionality tests
    #[test]
    fn test_export_to_json() {
        let test_path = get_test_path("test_export.json");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/knight.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("knight_idle_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("knight_run_0".to_string(), 32, 0, 32, 32));

        // Export to JSON
        let result = metadata.export(&test_path, ExportFormat::Json);
        assert!(result.is_ok(), "Export to JSON failed: {:?}", result.err());

        // Verify file exists
        assert!(test_path.exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(&test_path).unwrap();
        assert!(content.contains("\"texture_path\""));
        assert!(content.contains("\"knight_idle_0\""));
        assert!(content.contains("\"knight_run_0\""));

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_export_to_xml() {
        let test_path = get_test_path("test_export.xml");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/knight.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("knight_idle_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("knight_run_0".to_string(), 32, 0, 32, 32));

        // Export to XML
        let result = metadata.export(&test_path, ExportFormat::Xml);
        assert!(result.is_ok(), "Export to XML failed: {:?}", result.err());

        // Verify file exists
        assert!(test_path.exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(&test_path).unwrap();
        assert!(content.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(content.contains("<SpriteSheet>"));
        assert!(content.contains("<TexturePath>assets/knight.png</TexturePath>"));
        assert!(content.contains("<TextureWidth>512</TextureWidth>"));
        assert!(content.contains("<TextureHeight>256</TextureHeight>"));
        assert!(content.contains("<Sprites>"));
        assert!(content.contains("<Name>knight_idle_0</Name>"));
        assert!(content.contains("<Name>knight_run_0</Name>"));
        assert!(content.contains("<X>0</X>"));
        assert!(content.contains("<Y>0</Y>"));
        assert!(content.contains("<Width>32</Width>"));
        assert!(content.contains("<Height>32</Height>"));

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_export_to_texture_packer() {
        let test_path = get_test_path("test_export_tp.json");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/knight.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("knight_idle_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("knight_run_0".to_string(), 32, 0, 32, 32));

        // Export to TexturePacker format
        let result = metadata.export(&test_path, ExportFormat::TexturePacker);
        assert!(result.is_ok(), "Export to TexturePacker failed: {:?}", result.err());

        // Verify file exists
        assert!(test_path.exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(&test_path).unwrap();
        
        // Parse as JSON to verify structure
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        // Verify meta section
        assert!(json.get("meta").is_some());
        let meta = json.get("meta").unwrap();
        assert_eq!(meta.get("image").unwrap().as_str().unwrap(), "assets/knight.png");
        assert_eq!(meta.get("size").unwrap().get("w").unwrap().as_u64().unwrap(), 512);
        assert_eq!(meta.get("size").unwrap().get("h").unwrap().as_u64().unwrap(), 256);
        
        // Verify frames section
        assert!(json.get("frames").is_some());
        let frames = json.get("frames").unwrap();
        assert!(frames.get("knight_idle_0").is_some());
        assert!(frames.get("knight_run_0").is_some());
        
        // Verify frame data
        let frame1 = frames.get("knight_idle_0").unwrap();
        assert_eq!(frame1.get("frame").unwrap().get("x").unwrap().as_u64().unwrap(), 0);
        assert_eq!(frame1.get("frame").unwrap().get("y").unwrap().as_u64().unwrap(), 0);
        assert_eq!(frame1.get("frame").unwrap().get("w").unwrap().as_u64().unwrap(), 32);
        assert_eq!(frame1.get("frame").unwrap().get("h").unwrap().as_u64().unwrap(), 32);

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_export_empty_sprite_sheet() {
        let test_path = get_test_path("test_export_empty.json");
        cleanup_test_file(&test_path);

        let metadata = SpriteMetadata::new("assets/empty.png".to_string(), 512, 256);

        // Export empty sprite sheet
        let result = metadata.export(&test_path, ExportFormat::Json);
        assert!(result.is_ok(), "Export of empty sprite sheet failed: {:?}", result.err());

        // Verify file exists
        assert!(test_path.exists(), "Export file should exist");

        // Read and verify content
        let content = fs::read_to_string(&test_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(json.get("sprites").unwrap().as_array().unwrap().len(), 0);

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_export_all_formats() {
        let mut metadata = SpriteMetadata::new("assets/test.png".to_string(), 256, 128);
        metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));

        // Test all formats
        for (format, extension) in [
            (ExportFormat::Json, "json"),
            (ExportFormat::Xml, "xml"),
            (ExportFormat::TexturePacker, "json"),
        ] {
            let test_path = get_test_path(&format!("test_export_all.{}", extension));
            cleanup_test_file(&test_path);

            let result = metadata.export(&test_path, format);
            assert!(result.is_ok(), "Export to {:?} failed: {:?}", format, result.err());
            assert!(test_path.exists(), "Export file for {:?} should exist", format);

            cleanup_test_file(&test_path);
        }
    }

    #[test]
    fn test_export_includes_all_metadata() {
        let test_path = get_test_path("test_export_metadata.json");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/complete.png".to_string(), 1024, 512);
        metadata.add_sprite(SpriteDefinition::new("sprite_a".to_string(), 10, 20, 64, 64));
        metadata.add_sprite(SpriteDefinition::new("sprite_b".to_string(), 100, 200, 128, 128));

        // Export to JSON
        metadata.export(&test_path, ExportFormat::Json).unwrap();

        // Load back and verify all data is preserved
        let loaded = SpriteMetadata::load(&test_path).unwrap();
        
        assert_eq!(loaded.texture_path, "assets/complete.png");
        assert_eq!(loaded.texture_width, 1024);
        assert_eq!(loaded.texture_height, 512);
        assert_eq!(loaded.sprites.len(), 2);
        
        assert_eq!(loaded.sprites[0].name, "sprite_a");
        assert_eq!(loaded.sprites[0].x, 10);
        assert_eq!(loaded.sprites[0].y, 20);
        assert_eq!(loaded.sprites[0].width, 64);
        assert_eq!(loaded.sprites[0].height, 64);
        
        assert_eq!(loaded.sprites[1].name, "sprite_b");
        assert_eq!(loaded.sprites[1].x, 100);
        assert_eq!(loaded.sprites[1].y, 200);
        assert_eq!(loaded.sprites[1].width, 128);
        assert_eq!(loaded.sprites[1].height, 128);

        cleanup_test_file(&test_path);
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
        
        let found_1 = window.find_sprite_at_position(egui::pos2(66.0, 66.0), texture_pos);
        assert_eq!(found_1, Some(1));
        
        let found_2 = window.find_sprite_at_position(egui::pos2(116.0, 116.0), texture_pos);
        assert_eq!(found_2, Some(2));
        
        // Test position outside any sprite
        let found_none = window.find_sprite_at_position(egui::pos2(200.0, 200.0), texture_pos);
        assert_eq!(found_none, None);
    }

    #[test]
    fn test_sprites_overlap() {
        let texture_path = PathBuf::from("test_texture.png");
        let window = SpriteEditorWindow::new(texture_path);
        
        // Create two overlapping sprites
        let sprite1 = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 50, 50);
        let sprite2 = SpriteDefinition::new("sprite_1".to_string(), 25, 25, 50, 50);
        
        // They should overlap
        assert!(window.sprites_overlap(&sprite1, &sprite2));
        assert!(window.sprites_overlap(&sprite2, &sprite1));
        
        // Create two non-overlapping sprites
        let sprite3 = SpriteDefinition::new("sprite_2".to_string(), 100, 100, 50, 50);
        
        // They should not overlap
        assert!(!window.sprites_overlap(&sprite1, &sprite3));
        assert!(!window.sprites_overlap(&sprite3, &sprite1));
        
        // Edge case: sprites touching but not overlapping
        let sprite4 = SpriteDefinition::new("sprite_3".to_string(), 50, 0, 50, 50);
        assert!(!window.sprites_overlap(&sprite1, &sprite4));
    }

    #[test]
    fn test_cycle_overlapping_sprites() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add three overlapping sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 50, 50));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 25, 25, 50, 50));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 10, 10, 50, 50));
        
        // Select first sprite
        window.state.selected_sprite = Some(0);
        
        // Cycle should move to next overlapping sprite
        window.cycle_overlapping_sprites();
        
        // Should cycle to one of the overlapping sprites (1 or 2)
        assert!(window.state.selected_sprite == Some(1) || window.state.selected_sprite == Some(2));
        
        let first_cycle = window.state.selected_sprite;
        
        // Cycle again
        window.cycle_overlapping_sprites();
        
        // Should cycle to a different sprite
        assert_ne!(window.state.selected_sprite, first_cycle);
    }

    #[test]
    fn test_cycle_overlapping_sprites_no_overlap() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add non-overlapping sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 100, 100, 32, 32));
        
        // Select first sprite
        window.state.selected_sprite = Some(0);
        
        // Cycle should not change selection when no overlapping sprites
        window.cycle_overlapping_sprites();
        
        // Selection should remain the same
        assert_eq!(window.state.selected_sprite, Some(0));
    }

    #[test]
    fn test_delete_selected_sprite() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_2".to_string(), 64, 0, 32, 32));
        
        assert_eq!(window.state.metadata.sprites.len(), 3);
        
        // Select and delete middle sprite
        window.state.selected_sprite = Some(1);
        window.delete_selected_sprite();
        
        // Should have 2 sprites left
        assert_eq!(window.state.metadata.sprites.len(), 2);
        
        // Selection should be cleared
        assert_eq!(window.state.selected_sprite, None);
        
        // Undo stack should have one entry
        assert_eq!(window.state.undo_stack.len(), 1);
        
        // Remaining sprites should be sprite_0 and sprite_2
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
        assert_eq!(window.state.metadata.sprites[1].name, "sprite_2");
    }

    #[test]
    fn test_delete_with_no_selection() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        // Try to delete with no selection
        window.state.selected_sprite = None;
        window.delete_selected_sprite();
        
        // Sprite should still exist
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        // Undo stack should be empty
        assert_eq!(window.state.undo_stack.len(), 0);
    }



    // AutoSlicer tests
    #[test]
    fn test_auto_slicer_grid_basic() {
        let sprites = AutoSlicer::slice_by_grid(
            512,  // texture_width
            256,  // texture_height
            4,    // columns
            2,    // rows
            0,    // padding
            0,    // spacing
        );
        
        // Should create 4 * 2 = 8 sprites
        assert_eq!(sprites.len(), 8);
        
        // Each sprite should be 128x128 (512/4 x 256/2)
        for sprite in &sprites {
            assert_eq!(sprite.width, 128);
            assert_eq!(sprite.height, 128);
        }
        
        // Check first sprite position
        assert_eq!(sprites[0].x, 0);
        assert_eq!(sprites[0].y, 0);
        assert_eq!(sprites[0].name, "sprite_0");
        
        // Check second sprite position (next column)
        assert_eq!(sprites[1].x, 128);
        assert_eq!(sprites[1].y, 0);
        assert_eq!(sprites[1].name, "sprite_1");
        
        // Check fifth sprite position (second row, first column)
        assert_eq!(sprites[4].x, 0);
        assert_eq!(sprites[4].y, 128);
        assert_eq!(sprites[4].name, "sprite_4");
    }

    #[test]
    fn test_auto_slicer_grid_with_padding() {
        let sprites = AutoSlicer::slice_by_grid(
            512,  // texture_width
            256,  // texture_height
            4,    // columns
            2,    // rows
            10,   // padding
            0,    // spacing
        );
        
        assert_eq!(sprites.len(), 8);
        
        // Available space: 512 - 20 = 492 width, 256 - 20 = 236 height
        // Sprite size: 492/4 = 123 width, 236/2 = 118 height
        for sprite in &sprites {
            assert_eq!(sprite.width, 123);
            assert_eq!(sprite.height, 118);
        }
        
        // First sprite should start at padding offset
        assert_eq!(sprites[0].x, 10);
        assert_eq!(sprites[0].y, 10);
        
        // Second sprite
        assert_eq!(sprites[1].x, 10 + 123);
        assert_eq!(sprites[1].y, 10);
    }

    #[test]
    fn test_auto_slicer_grid_with_spacing() {
        let sprites = AutoSlicer::slice_by_grid(
            512,  // texture_width
            256,  // texture_height
            4,    // columns
            2,    // rows
            0,    // padding
            5,    // spacing
        );
        
        assert_eq!(sprites.len(), 8);
        
        // Total spacing: 5 * (4-1) = 15 horizontal, 5 * (2-1) = 5 vertical
        // Available for sprites: 512 - 15 = 497 width, 256 - 5 = 251 height
        // Sprite size: 497/4 = 124 width, 251/2 = 125 height
        for sprite in &sprites {
            assert_eq!(sprite.width, 124);
            assert_eq!(sprite.height, 125);
        }
        
        // First sprite
        assert_eq!(sprites[0].x, 0);
        assert_eq!(sprites[0].y, 0);
        
        // Second sprite (with spacing)
        assert_eq!(sprites[1].x, 124 + 5);
        assert_eq!(sprites[1].y, 0);
        
        // Fifth sprite (second row with spacing)
        assert_eq!(sprites[4].x, 0);
        assert_eq!(sprites[4].y, 125 + 5);
    }

    #[test]
    fn test_auto_slicer_grid_with_padding_and_spacing() {
        let sprites = AutoSlicer::slice_by_grid(
            512,  // texture_width
            256,  // texture_height
            4,    // columns
            2,    // rows
            10,   // padding
            5,    // spacing
        );
        
        assert_eq!(sprites.len(), 8);
        
        // Available space: 512 - 20 = 492 width, 256 - 20 = 236 height
        // Total spacing: 5 * 3 = 15 horizontal, 5 * 1 = 5 vertical
        // Available for sprites: 492 - 15 = 477 width, 236 - 5 = 231 height
        // Sprite size: 477/4 = 119 width, 231/2 = 115 height
        for sprite in &sprites {
            assert_eq!(sprite.width, 119);
            assert_eq!(sprite.height, 115);
        }
        
        // First sprite starts at padding
        assert_eq!(sprites[0].x, 10);
        assert_eq!(sprites[0].y, 10);
        
        // Second sprite (padding + sprite_width + spacing)
        assert_eq!(sprites[1].x, 10 + 119 + 5);
        assert_eq!(sprites[1].y, 10);
    }

    #[test]
    fn test_auto_slicer_grid_sequential_naming() {
        let sprites = AutoSlicer::slice_by_grid(512, 256, 3, 2, 0, 0);
        
        assert_eq!(sprites.len(), 6);
        
        // Verify sequential naming
        for (i, sprite) in sprites.iter().enumerate() {
            assert_eq!(sprite.name, format!("sprite_{}", i));
        }
    }

    #[test]
    fn test_auto_slicer_grid_zero_columns() {
        let sprites = AutoSlicer::slice_by_grid(512, 256, 0, 2, 0, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_grid_zero_rows() {
        let sprites = AutoSlicer::slice_by_grid(512, 256, 4, 0, 0, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_grid_excessive_padding() {
        // Padding larger than texture should result in no sprites
        let sprites = AutoSlicer::slice_by_grid(512, 256, 4, 2, 300, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_grid_excessive_spacing() {
        // Spacing that leaves no room for sprites
        let sprites = AutoSlicer::slice_by_grid(512, 256, 4, 2, 0, 200);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_cell_size_basic() {
        let sprites = AutoSlicer::slice_by_cell_size(
            512,  // texture_width
            256,  // texture_height
            32,   // cell_width
            32,   // cell_height
            0,    // padding
            0,    // spacing
        );
        
        // Should fit 16 columns (512/32) and 8 rows (256/32) = 128 sprites
        assert_eq!(sprites.len(), 128);
        
        // All sprites should be 32x32
        for sprite in &sprites {
            assert_eq!(sprite.width, 32);
            assert_eq!(sprite.height, 32);
        }
        
        // Check first sprite
        assert_eq!(sprites[0].x, 0);
        assert_eq!(sprites[0].y, 0);
        assert_eq!(sprites[0].name, "sprite_0");
        
        // Check second sprite
        assert_eq!(sprites[1].x, 32);
        assert_eq!(sprites[1].y, 0);
    }

    #[test]
    fn test_auto_slicer_cell_size_with_padding() {
        let sprites = AutoSlicer::slice_by_cell_size(
            512,  // texture_width
            256,  // texture_height
            32,   // cell_width
            32,   // cell_height
            10,   // padding
            0,    // spacing
        );
        
        // Available space: 512 - 20 = 492 width, 256 - 20 = 236 height
        // Columns: 492/32 = 15, Rows: 236/32 = 7
        assert_eq!(sprites.len(), 15 * 7);
        
        // First sprite starts at padding
        assert_eq!(sprites[0].x, 10);
        assert_eq!(sprites[0].y, 10);
        
        // Second sprite
        assert_eq!(sprites[1].x, 10 + 32);
        assert_eq!(sprites[1].y, 10);
    }

    #[test]
    fn test_auto_slicer_cell_size_with_spacing() {
        let sprites = AutoSlicer::slice_by_cell_size(
            512,  // texture_width
            256,  // texture_height
            32,   // cell_width
            32,   // cell_height
            0,    // padding
            2,    // spacing
        );
        
        // With spacing: (512 + 2) / (32 + 2) = 15 columns
        // (256 + 2) / (32 + 2) = 7 rows
        assert_eq!(sprites.len(), 15 * 7);
        
        // First sprite
        assert_eq!(sprites[0].x, 0);
        assert_eq!(sprites[0].y, 0);
        
        // Second sprite (with spacing)
        assert_eq!(sprites[1].x, 32 + 2);
        assert_eq!(sprites[1].y, 0);
    }

    #[test]
    fn test_auto_slicer_cell_size_bounds_checking() {
        let sprites = AutoSlicer::slice_by_cell_size(
            100,  // texture_width
            100,  // texture_height
            32,   // cell_width
            32,   // cell_height
            0,    // padding
            0,    // spacing
        );
        
        // Should fit 3x3 = 9 sprites (100/32 = 3 with remainder)
        assert_eq!(sprites.len(), 9);
        
        // Verify all sprites are within bounds
        for sprite in &sprites {
            assert!(sprite.x + sprite.width <= 100);
            assert!(sprite.y + sprite.height <= 100);
        }
    }

    #[test]
    fn test_auto_slicer_cell_size_sequential_naming() {
        let sprites = AutoSlicer::slice_by_cell_size(128, 64, 32, 32, 0, 0);
        
        // Should create 4 columns * 2 rows = 8 sprites
        assert_eq!(sprites.len(), 8);
        
        // Verify sequential naming
        for (i, sprite) in sprites.iter().enumerate() {
            assert_eq!(sprite.name, format!("sprite_{}", i));
        }
    }

    #[test]
    fn test_auto_slicer_cell_size_zero_cell_width() {
        let sprites = AutoSlicer::slice_by_cell_size(512, 256, 0, 32, 0, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_cell_size_zero_cell_height() {
        let sprites = AutoSlicer::slice_by_cell_size(512, 256, 32, 0, 0, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slicer_cell_size_larger_than_texture() {
        // Cell size larger than texture should result in no sprites
        let sprites = AutoSlicer::slice_by_cell_size(512, 256, 1024, 512, 0, 0);
        assert_eq!(sprites.len(), 0);
    }

    #[test]
    fn test_auto_slice_integration() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        // Set texture dimensions
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add some existing sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("old_sprite".to_string(), 0, 0, 16, 16));
        assert_eq!(window.state.metadata.sprites.len(), 1);
        
        // Configure auto-slice settings
        window.auto_slice_mode = AutoSliceMode::Grid;
        window.auto_slice_columns = 4;
        window.auto_slice_rows = 2;
        window.auto_slice_padding = 0;
        window.auto_slice_spacing = 0;
        
        // Apply auto-slice
        window.apply_auto_slice();
        
        // Should replace existing sprites with new grid
        assert_eq!(window.state.metadata.sprites.len(), 8);
        
        // Verify undo stack was updated
        assert_eq!(window.state.undo_stack.len(), 1);
        
        // Verify old sprite is in undo stack
        assert_eq!(window.state.undo_stack[0].sprites.len(), 1);
        assert_eq!(window.state.undo_stack[0].sprites[0].name, "old_sprite");
        
        // Verify new sprites have correct dimensions
        for sprite in &window.state.metadata.sprites {
            assert_eq!(sprite.width, 128);
            assert_eq!(sprite.height, 128);
        }
        
        // Verify selection was cleared
        assert_eq!(window.state.selected_sprite, None);
    }

    #[test]
    fn test_auto_slice_cell_size_mode() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Configure auto-slice with cell size mode
        window.auto_slice_mode = AutoSliceMode::CellSize;
        window.auto_slice_cell_width = 32;
        window.auto_slice_cell_height = 32;
        window.auto_slice_padding = 0;
        window.auto_slice_spacing = 0;
        
        // Apply auto-slice
        window.apply_auto_slice();
        
        // Should create 16 * 8 = 128 sprites
        assert_eq!(window.state.metadata.sprites.len(), 128);
        
        // All sprites should be 32x32
        for sprite in &window.state.metadata.sprites {
            assert_eq!(sprite.width, 32);
            assert_eq!(sprite.height, 32);
        }
    }

    #[test]
    fn test_auto_slice_undo_redo() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add initial sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("original".to_string(), 0, 0, 64, 64));
        let original_count = window.state.metadata.sprites.len();
        
        // Apply auto-slice
        window.auto_slice_mode = AutoSliceMode::Grid;
        window.auto_slice_columns = 2;
        window.auto_slice_rows = 2;
        window.auto_slice_padding = 0;
        window.auto_slice_spacing = 0;
        window.apply_auto_slice();
        
        assert_eq!(window.state.metadata.sprites.len(), 4);
        
        // Undo should restore original sprite
        window.state.undo();
        assert_eq!(window.state.metadata.sprites.len(), original_count);
        assert_eq!(window.state.metadata.sprites[0].name, "original");
        
        // Redo should restore auto-sliced sprites
        window.state.redo();
        assert_eq!(window.state.metadata.sprites.len(), 4);
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
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

    #[test]
    fn test_properties_panel_name_editing() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        
        // Select first sprite
        window.state.selected_sprite = Some(0);
        
        // Simulate name editing
        window.name_edit_buffer = "new_name".to_string();
        
        // Check for duplicates (should be false)
        let is_duplicate = window.state.metadata.sprites.iter().enumerate()
            .any(|(i, s)| i != 0 && s.name == window.name_edit_buffer);
        assert!(!is_duplicate);
        
        // Update the sprite name
        if let Some(sprite) = window.state.metadata.sprites.get_mut(0) {
            sprite.name = window.name_edit_buffer.clone();
        }
        
        // Verify name was updated
        assert_eq!(window.state.metadata.sprites[0].name, "new_name");
    }
    
    #[test]
    fn test_properties_panel_duplicate_name_detection() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprites
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));
        
        // Select first sprite
        window.state.selected_sprite = Some(0);
        
        // Try to set name to duplicate
        window.name_edit_buffer = "sprite_1".to_string();
        
        // Check for duplicates (should be true)
        let is_duplicate = window.state.metadata.sprites.iter().enumerate()
            .any(|(i, s)| i != 0 && s.name == window.name_edit_buffer);
        assert!(is_duplicate);
        
        // Name should not be updated when duplicate
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
    }
    
    #[test]
    fn test_properties_panel_empty_name_validation() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Select sprite
        window.state.selected_sprite = Some(0);
        
        // Try to set empty name
        window.name_edit_buffer = "".to_string();
        
        // Empty names should not be allowed
        let is_empty = window.name_edit_buffer.trim().is_empty();
        assert!(is_empty);
        
        // Original name should remain
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
    }
    
    #[test]
    fn test_properties_panel_whitespace_name_validation() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Select sprite
        window.state.selected_sprite = Some(0);
        
        // Try to set whitespace-only name
        window.name_edit_buffer = "   ".to_string();
        
        // Whitespace-only names should not be allowed
        let is_empty = window.name_edit_buffer.trim().is_empty();
        assert!(is_empty);
        
        // Original name should remain
        assert_eq!(window.state.metadata.sprites[0].name, "sprite_0");
    }
    
    #[test]
    fn test_properties_panel_displays_sprite_info() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite with specific properties
        window.state.metadata.add_sprite(SpriteDefinition::new(
            "test_sprite".to_string(),
            100,
            200,
            64,
            128
        ));
        
        // Select sprite
        window.state.selected_sprite = Some(0);
        
        // Verify sprite properties can be accessed
        if let Some(sprite) = window.state.metadata.sprites.get(0) {
            assert_eq!(sprite.name, "test_sprite");
            assert_eq!(sprite.x, 100);
            assert_eq!(sprite.y, 200);
            assert_eq!(sprite.width, 64);
            assert_eq!(sprite.height, 128);
        }
    }
    
    #[test]
    fn test_properties_panel_no_selection() {
        let texture_path = PathBuf::from("test_texture.png");
        let window = SpriteEditorWindow::new(texture_path);
        
        // No sprite selected
        assert_eq!(window.state.selected_sprite, None);
        
        // Properties panel should handle no selection gracefully
        // (This is tested by the render method, but we verify the state)
    }
    
    #[test]
    fn test_properties_panel_name_buffer_initialization() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut window = SpriteEditorWindow::new(texture_path);
        
        window.state.metadata.texture_width = 512;
        window.state.metadata.texture_height = 256;
        
        // Add sprite
        window.state.metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        
        // Initially buffer should be empty
        assert_eq!(window.name_edit_buffer, "");
        
        // Select sprite
        window.state.selected_sprite = Some(0);
        
        // Simulate buffer initialization (would happen in render)
        if let Some(sprite) = window.state.metadata.sprites.get(0) {
            if window.name_edit_buffer.is_empty() || window.name_edit_buffer != sprite.name {
                window.name_edit_buffer = sprite.name.clone();
            }
        }
        
        // Buffer should now contain sprite name
        assert_eq!(window.name_edit_buffer, "sprite_0");
    }

    #[test]
    fn test_hot_reload_check_interval() {
        let texture_path = PathBuf::from("test_texture.png");
        let mut state = SpriteEditorState::new(texture_path);
        
        // Initially time_since_check should be 0
        assert_eq!(state.time_since_check, 0.0);
        
        // Check with dt less than interval - should not check file
        let reloaded = state.check_and_reload(0.5);
        assert!(!reloaded); // File doesn't exist, so won't reload
        assert_eq!(state.time_since_check, 0.5);
        
        // Check again with more time - should accumulate
        let reloaded = state.check_and_reload(0.3);
        assert!(!reloaded);
        assert_eq!(state.time_since_check, 0.8);
        
        // Check with enough time to exceed interval
        let reloaded = state.check_and_reload(0.3);
        assert!(!reloaded); // File doesn't exist
        assert_eq!(state.time_since_check, 0.0); // Should reset after check
    }
    
    #[test]
    fn test_hot_reload_state_initialization() {
        let texture_path = PathBuf::from("test_texture.png");
        let state = SpriteEditorState::new(texture_path);
        
        // Check hot-reload fields are initialized
        assert_eq!(state.check_interval, 1.0);
        assert_eq!(state.time_since_check, 0.0);
        assert!(state.last_modified.is_none()); // File doesn't exist
    }
}
