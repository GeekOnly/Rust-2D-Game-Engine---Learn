//! Sprite Editor Window
//!
//! Visual editor window for sprite sheets with egui

use crate::{SpriteMetadata, SpriteDefinition, ExportFormat, SpriteStatistics};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use egui::TextureHandle;

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

/// Auto-slice mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum AutoSliceMode {
    Grid,
    CellSize,
}

/// Texture manager trait for loading textures
/// This allows the sprite editor to work with different texture management systems
pub trait TextureManager {
    fn get_base_path(&self) -> Option<&std::path::Path>;
    fn load_texture(&mut self, ctx: &egui::Context, id: &str, path: &std::path::Path) -> Option<TextureHandle>;
    fn load_texture_absolute(&mut self, ctx: &egui::Context, id: &str, path: &std::path::Path) -> Option<TextureHandle>;
}

/// Editor state for the sprite editor
#[derive(Clone)]
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
    /// Convert full path to relative path (remove "projects/ProjectName/" prefix)
    fn make_relative_path(path: &PathBuf) -> String {
        let path_str = path.to_string_lossy();
        
        // Check if path starts with "projects/"
        if let Some(idx) = path_str.find("projects/") {
            // Find the next "/" after "projects/ProjectName/"
            let after_projects = &path_str[idx + "projects/".len()..];
            if let Some(next_slash) = after_projects.find('/') {
                // Return everything after "projects/ProjectName/"
                return after_projects[next_slash + 1..].replace('\\', "/");
            }
        }
        
        // If no "projects/" prefix, return as-is with forward slashes
        path_str.replace('\\', "/")
    }
    
    /// Create a new sprite editor state
    pub fn new(texture_path: PathBuf) -> Self {
        // Determine metadata path (.sprite file)
        let metadata_path = texture_path.with_extension("sprite");
        
        // Convert to relative path (remove "projects/ProjectName/" prefix if exists)
        let relative_texture_path = Self::make_relative_path(&texture_path);
        
        // Try to load existing metadata or create new
        let metadata = if metadata_path.exists() {
            match SpriteMetadata::load(&metadata_path) {
                Ok(mut loaded_metadata) => {
                    // Ensure texture_path in metadata is correct (not .sprite file)
                    if loaded_metadata.texture_path.ends_with(".sprite") {
                        log::warn!("Sprite metadata has .sprite extension in texture_path, fixing to: {}", relative_texture_path);
                        loaded_metadata.texture_path = relative_texture_path.clone();
                    }
                    loaded_metadata
                }
                Err(e) => {
                    log::warn!("Failed to load sprite metadata: {}", e);
                    SpriteMetadata::new(
                        relative_texture_path.clone(),
                        0,
                        0,
                    )
                }
            }
        } else {
            SpriteMetadata::new(
                relative_texture_path,
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
            check_interval: 1.0,
            time_since_check: 0.0,
        }
    }
    
    /// Push current state to undo stack
    pub fn push_undo(&mut self) {
        if self.undo_stack.len() >= 50 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(self.metadata.clone());
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
    pub fn load_texture<T: TextureManager>(&mut self, ctx: &egui::Context, texture_manager: &mut T) -> Result<(), String> {
        let texture_id = format!("sprite_editor_{}", self.texture_path.to_string_lossy());
        
        if let Some(base) = texture_manager.get_base_path() {
            let full_path = base.join(&self.texture_path);
            
            if full_path.exists() {
                log::info!("Sprite editor loading texture (relative): {}", self.texture_path.display());
                if let Some(handle) = texture_manager.load_texture(ctx, &texture_id, &self.texture_path) {
                    self.texture_handle = Some(handle.clone());
                    let size = handle.size();
                    self.metadata.texture_width = size[0] as u32;
                    self.metadata.texture_height = size[1] as u32;
                    return Ok(());
                }
            } else {
                log::info!("Sprite editor loading texture (absolute): {}", self.texture_path.display());
                if let Some(handle) = texture_manager.load_texture_absolute(ctx, &texture_id, &self.texture_path) {
                    self.texture_handle = Some(handle.clone());
                    let size = handle.size();
                    self.metadata.texture_width = size[0] as u32;
                    self.metadata.texture_height = size[1] as u32;
                    return Ok(());
                }
            }
            
            Err(format!("Failed to load texture: {}", self.texture_path.display()))
        } else {
            log::info!("Sprite editor loading texture (no base): {}", self.texture_path.display());
            if let Some(handle) = texture_manager.load_texture_absolute(ctx, &texture_id, &self.texture_path) {
                self.texture_handle = Some(handle.clone());
                let size = handle.size();
                self.metadata.texture_width = size[0] as u32;
                self.metadata.texture_height = size[1] as u32;
                Ok(())
            } else {
                Err(format!("Failed to load texture: {}", self.texture_path.display()))
            }
        }
    }
    
    /// Check if the sprite file has been modified and reload if necessary
    pub fn check_and_reload(&mut self, dt: f32) -> bool {
        self.time_since_check += dt;
        
        if self.time_since_check < self.check_interval {
            return false;
        }
        
        self.time_since_check = 0.0;
        
        if !self.metadata_path.exists() {
            return false;
        }
        
        let current_modified = match fs::metadata(&self.metadata_path)
            .and_then(|m| m.modified())
        {
            Ok(time) => time,
            Err(_) => return false,
        };
        
        if let Some(last_modified) = self.last_modified {
            if current_modified > last_modified {
                match SpriteMetadata::load(&self.metadata_path) {
                    Ok(new_metadata) => {
                        log::info!("Hot-reloaded sprite metadata from {:?}", self.metadata_path);
                        self.metadata = new_metadata;
                        self.last_modified = Some(current_modified);
                        
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
            self.last_modified = Some(current_modified);
        }
        
        false
    }
}

/// Sprite Editor Window
pub struct SpriteEditorWindow {
    pub state: SpriteEditorState,
    pub is_open: bool,
    name_edit_buffer: String,
    duplicate_name_error: bool,
    show_auto_slice_dialog: bool,
    auto_slice_columns: u32,
    auto_slice_rows: u32,
    auto_slice_padding: u32,
    auto_slice_spacing: u32,
    auto_slice_mode: AutoSliceMode,
    auto_slice_cell_width: u32,
    auto_slice_cell_height: u32,
    show_export_dialog: bool,
    export_format: ExportFormat,
    export_message: Option<String>,
    export_error: Option<String>,
    statistics: SpriteStatistics,
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
    
    /// Render the sprite editor as a standalone window
    pub fn render<T: TextureManager>(&mut self, ctx: &egui::Context, texture_manager: &mut T, dt: f32) {
        if !self.is_open {
            return;
        }

        // Load texture if not already loaded
        if self.state.texture_handle.is_none() {
            if let Err(e) = self.state.load_texture(ctx, texture_manager) {
                log::error!("Failed to load texture: {}", e);
                self.is_open = false;
                return;
            }
        }

        // Check for file changes and reload if necessary
        if self.state.check_and_reload(dt) {
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

        // Render dialogs
        if self.show_auto_slice_dialog {
            self.render_auto_slice_dialog(ctx);
        }

        if self.show_export_dialog {
            self.render_export_dialog(ctx);
        }
    }

    /// Render the sprite editor inline (for dockable tab view)
    pub fn render_inline<T: TextureManager>(&mut self, ui: &mut egui::Ui, texture_manager: &mut T, dt: f32) {
        // Load texture if not already loaded
        if self.state.texture_handle.is_none() {
            if let Err(e) = self.state.load_texture(ui.ctx(), texture_manager) {
                ui.colored_label(egui::Color32::RED, format!("Failed to load texture: {}", e));
                return;
            }
        }

        // Check for file changes and reload if necessary
        if self.state.check_and_reload(dt) {
            self.update_statistics();
        }

        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ui.ctx());

        // Render content directly
        self.render_content(ui);

        // Render dialogs
        if self.show_auto_slice_dialog {
            self.render_auto_slice_dialog(ui.ctx());
        }

        if self.show_export_dialog {
            self.render_export_dialog(ui.ctx());
        }
    }
    
    /// Handle keyboard shortcuts
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Delete) {
                self.delete_selected_sprite();
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if let Err(e) = self.state.save() {
                    log::error!("Failed to save sprite metadata: {}", e);
                } else {
                    log::info!("Sprite metadata saved successfully");
                }
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                self.state.undo();
                self.update_statistics();
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                self.state.redo();
                self.update_statistics();
            }
            
            if i.key_pressed(egui::Key::Escape) {
                self.state.selected_sprite = None;
            }

            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                self.duplicate_selected_sprite();
            }
        });
    }

    /// Delete the currently selected sprite
    fn delete_selected_sprite(&mut self) {
        if let Some(selected_idx) = self.state.selected_sprite {
            self.state.push_undo();
            self.state.metadata.sprites.remove(selected_idx);
            self.state.selected_sprite = None;
            self.update_statistics();
            log::info!("Deleted sprite at index {}", selected_idx);
        }
    }

    /// Duplicate the currently selected sprite
    fn duplicate_selected_sprite(&mut self) {
        if let Some(selected_idx) = self.state.selected_sprite {
            if let Some(sprite) = self.state.metadata.sprites.get(selected_idx).cloned() {
                self.state.push_undo();

                let mut duplicated = sprite.clone();
                let texture_width = self.state.metadata.texture_width;
                let texture_height = self.state.metadata.texture_height;

                if duplicated.x + duplicated.width + 10 <= texture_width {
                    duplicated.x += 10;
                } else if duplicated.x >= 10 {
                    duplicated.x -= 10;
                }

                if duplicated.y + duplicated.height + 10 <= texture_height {
                    duplicated.y += 10;
                } else if duplicated.y >= 10 {
                    duplicated.y -= 10;
                }

                duplicated.name = self.generate_duplicate_name(&sprite.name);
                self.state.metadata.sprites.push(duplicated);
                self.state.selected_sprite = Some(self.state.metadata.sprites.len() - 1);
                self.update_statistics();
                log::info!("Duplicated sprite: {}", sprite.name);
            }
        }
    }

    /// Generate a unique name for duplicated sprite
    fn generate_duplicate_name(&self, original_name: &str) -> String {
        let base_name = if original_name.ends_with("_copy") {
            original_name.to_string()
        } else {
            format!("{}_copy", original_name)
        };

        let mut candidate = base_name.clone();
        let mut counter = 2;

        while self.state.metadata.sprites.iter().any(|s| s.name == candidate) {
            candidate = format!("{}_{}", base_name, counter);
            counter += 1;
        }

        candidate
    }
    
    // UI rendering methods
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
            
            let has_sprites = !self.state.metadata.sprites.is_empty();
            ui.add_enabled_ui(has_sprites, |ui| {
                if ui.button("📤 Export").clicked() {
                    self.show_export_dialog = true;
                    self.export_message = None;
                    self.export_error = None;
                }
            });
            
            ui.separator();
            
            let can_undo = !self.state.undo_stack.is_empty();
            ui.add_enabled_ui(can_undo, |ui| {
                if ui.button("↶ Undo (Ctrl+Z)").clicked() {
                    self.state.undo();
                    self.update_statistics();
                }
            });
            
            let can_redo = !self.state.redo_stack.is_empty();
            ui.add_enabled_ui(can_redo, |ui| {
                if ui.button("↷ Redo (Ctrl+Y)").clicked() {
                    self.state.redo();
                    self.update_statistics();
                }
            });
            
            ui.separator();
            
            ui.label(format!("Zoom: {:.0}%", self.state.zoom * 100.0));
        });
        
        ui.separator();
        
        // Main content area
        egui::SidePanel::left("sprite_list_panel")
            .exact_width(200.0)
            .resizable(false)
            .show_inside(ui, |ui| {
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

                egui::ScrollArea::vertical()
                    .id_source("sprite_list_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_sprite_list(ui);
                    });
            });

        egui::SidePanel::right("properties_panel")
            .exact_width(300.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.heading("Properties");
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_source("properties_scroll")
                    .show(ui, |ui| {
                        self.render_properties_panel(ui);

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.heading("Statistics");
                        ui.separator();

                        self.render_statistics_panel(ui);
                    });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Canvas");
            ui.separator();

            if let Some(texture_handle) = self.state.texture_handle.clone() {
                self.render_canvas(ui, &texture_handle);
            } else {
                ui.label("Loading texture...");
            }
        });
        
        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Sprites: {}", self.statistics.sprite_count));
            ui.separator();
            
            ui.label(format!(
                "Texture: {}x{}",
                self.state.metadata.texture_width,
                self.state.metadata.texture_height
            ));
            ui.separator();
            
            ui.label(format!(
                "Coverage: {:.1}%",
                self.statistics.texture_coverage_percent
            ));
            
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
            
            let frame_response = frame.show(ui, |ui| {
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
                frame_response.response.rect,
                ui.id().with(format!("sprite_item_{}", idx)),
                egui::Sense::click()
            );

            if item_response.clicked() {
                self.state.selected_sprite = Some(idx);
            }

            // Add hover effect
            if item_response.hovered() {
                ui.painter().rect_stroke(
                    frame_response.response.rect,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255))
                );
            }
            
            ui.add_space(4.0);
        }
    }
    
    fn render_properties_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(idx) = self.state.selected_sprite {
            if let Some(sprite) = self.state.metadata.sprites.get(idx).cloned() {
                ui.label("Name:");
                if self.name_edit_buffer.is_empty() || self.name_edit_buffer != sprite.name {
                    self.name_edit_buffer = sprite.name.clone();
                }
                
                let name_response = ui.text_edit_singleline(&mut self.name_edit_buffer);
                
                if name_response.changed() {
                    let is_duplicate = self.state.metadata.sprites.iter().enumerate()
                        .any(|(i, s)| i != idx && s.name == self.name_edit_buffer);
                    
                    self.duplicate_name_error = is_duplicate;
                    
                    if !is_duplicate && !self.name_edit_buffer.trim().is_empty() {
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.name = self.name_edit_buffer.clone();
                        }
                    }
                }
                
                if self.duplicate_name_error {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        "⚠ Duplicate name!"
                    );
                }
                
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("📋 Duplicate (Ctrl+D)").clicked() {
                        self.duplicate_selected_sprite();
                    }

                    if ui.button("🗑 Delete").clicked() {
                        self.delete_selected_sprite();
                    }
                });

                ui.add_space(10.0);

                ui.label("Position & Size:");

                let mut x_value = sprite.x as i32;
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut x_value).speed(1.0).suffix(" px")).changed() {
                        self.state.push_undo();
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.x = x_value.max(0) as u32;
                        }
                        self.update_statistics();
                    }
                });

                let mut y_value = sprite.y as i32;
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut y_value).speed(1.0).suffix(" px")).changed() {
                        self.state.push_undo();
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.y = y_value.max(0) as u32;
                        }
                        self.update_statistics();
                    }
                });

                let mut width_value = sprite.width as i32;
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    if ui.add(egui::DragValue::new(&mut width_value).speed(1.0).clamp_range(1..=self.state.metadata.texture_width).suffix(" px")).changed() {
                        self.state.push_undo();
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.width = width_value.max(1) as u32;
                        }
                        self.update_statistics();
                    }
                });

                let mut height_value = sprite.height as i32;
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    if ui.add(egui::DragValue::new(&mut height_value).speed(1.0).clamp_range(1..=self.state.metadata.texture_height).suffix(" px")).changed() {
                        self.state.push_undo();
                        if let Some(sprite_mut) = self.state.metadata.sprites.get_mut(idx) {
                            sprite_mut.height = height_value.max(1) as u32;
                        }
                        self.update_statistics();
                    }
                });
            }
        } else {
            ui.label("No sprite selected");
        }
    }
    
    fn render_statistics_panel(&self, ui: &mut egui::Ui) {
        ui.label("Texture Dimensions:");
        ui.label(format!(
            "{}×{} pixels",
            self.state.metadata.texture_width,
            self.state.metadata.texture_height
        ));
        
        ui.add_space(10.0);
        
        ui.label("Sprite Count:");
        ui.label(format!("{} sprites", self.statistics.sprite_count));
        
        ui.add_space(10.0);
        
        ui.label("Texture Coverage:");
        ui.label(format!("{:.2}%", self.statistics.texture_coverage_percent));
    }
    
    fn render_canvas(&mut self, ui: &mut egui::Ui, texture_handle: &TextureHandle) {
        let texture_size = texture_handle.size();
        let scaled_width = texture_size[0] as f32 * self.state.zoom;
        let scaled_height = texture_size[1] as f32 * self.state.zoom;
        
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(scaled_width, scaled_height),
                    egui::Sense::click_and_drag()
                );
                
                let texture_rect = egui::Rect::from_min_size(
                    response.rect.min,
                    egui::vec2(scaled_width, scaled_height)
                );
                
                painter.image(
                    texture_handle.id(),
                    texture_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE
                );
                
                // Draw sprite rectangles
                for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
                    let is_selected = self.state.selected_sprite == Some(idx);
                    
                    let sprite_rect = egui::Rect::from_min_size(
                        texture_rect.min + egui::vec2(sprite.x as f32 * self.state.zoom, sprite.y as f32 * self.state.zoom),
                        egui::vec2(sprite.width as f32 * self.state.zoom, sprite.height as f32 * self.state.zoom),
                    );

                    let color = if is_selected {
                        egui::Color32::from_rgb(0, 255, 0)
                    } else {
                        egui::Color32::from_rgb(255, 255, 0)
                    };

                    painter.rect_stroke(sprite_rect, 0.0, egui::Stroke::new(2.0, color));

                    if is_selected {
                        painter.rect_filled(sprite_rect, 0.0, egui::Color32::from_rgba_unmultiplied(0, 255, 0, 30));
                    }

                    let text_pos = sprite_rect.min + egui::vec2(2.0, 2.0);
                    let text_galley = painter.layout_no_wrap(
                        sprite.name.clone(),
                        egui::FontId::proportional(12.0),
                        color,
                    );
                    let text_rect = egui::Rect::from_min_size(
                        text_pos,
                        text_galley.size(),
                    );
                    painter.rect_filled(text_rect.expand(2.0), 2.0, egui::Color32::from_black_alpha(180));
                    painter.galley(text_pos, text_galley, color);
                }
                
                // Handle click to select sprite
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let mut clicked_sprite = None;
                        for (idx, sprite) in self.state.metadata.sprites.iter().enumerate().rev() {
                            let sprite_rect = egui::Rect::from_min_size(
                                texture_rect.min + egui::vec2(sprite.x as f32 * self.state.zoom, sprite.y as f32 * self.state.zoom),
                                egui::vec2(sprite.width as f32 * self.state.zoom, sprite.height as f32 * self.state.zoom),
                            );
                            if sprite_rect.contains(pos) {
                                clicked_sprite = Some(idx);
                                break;
                            }
                        }
                        self.state.selected_sprite = clicked_sprite;
                    }
                }
                
                // Handle zoom with scroll
                if response.hovered() {
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta != 0.0 {
                        let zoom_factor = 1.0 + (scroll_delta * 0.001);
                        self.state.zoom = (self.state.zoom * zoom_factor).clamp(0.1, 10.0);
                    }
                }
            });
    }
    
    fn render_auto_slice_dialog(&mut self, _ctx: &egui::Context) {
        // TODO: Implement auto-slice dialog
    }
    
    fn render_export_dialog(&mut self, _ctx: &egui::Context) {
        // TODO: Implement export dialog
    }
}
