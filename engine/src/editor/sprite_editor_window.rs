//! Sprite Editor Window
//!
//! Visual editor window for sprite sheets
//!
//! NOTE: This is a temporary stub. The full SpriteEditorWindow implementation
//! will be moved to the sprite_editor crate in a future update.

// Import from sprite_editor crate
use sprite_editor::{
    SpriteMetadata, SpriteDefinition, ExportFormat,
    SpriteStatistics, AutoSlicer,
};

// Local imports
use std::path::PathBuf;

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

/// Editor state
pub struct SpriteEditorState {
    pub texture_path: PathBuf,
    pub metadata_path: PathBuf,
    pub metadata: SpriteMetadata,
    pub selected_sprite: Option<usize>,
    pub drag_mode: DragMode,
    pub drag_start: Option<egui::Pos2>,
    pub zoom: f32,
    pub pan_offset: egui::Vec2,
    pub show_grid: bool,
    pub grid_size: u32,
}

impl SpriteEditorState {
    /// Create new editor state
    pub fn new(texture_path: PathBuf) -> Self {
        let metadata_path = texture_path.with_extension("sprite");
        let metadata = SpriteMetadata::load(&metadata_path).unwrap_or_else(|_| {
            SpriteMetadata {
                texture_path: texture_path.to_string_lossy().to_string(),
                texture_width: 256,
                texture_height: 256,
                sprites: Vec::new(),
            }
        });

        Self {
            texture_path,
            metadata_path,
            metadata,
            selected_sprite: None,
            drag_mode: DragMode::None,
            drag_start: None,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            show_grid: true,
            grid_size: 32,
        }
    }

    /// Check and reload metadata if file changed
    pub fn check_and_reload(&mut self, _dt: f32) -> bool {
        // TODO: Implement hot reload
        false
    }

    /// Save metadata to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.metadata.save(&self.metadata_path)?;
        Ok(())
    }
}

/// Sprite Editor Window
pub struct SpriteEditorWindow {
    pub is_open: bool,
    pub state: SpriteEditorState,
}

impl SpriteEditorWindow {
    /// Create a new sprite editor window
    pub fn new(texture_path: PathBuf) -> Self {
        Self {
            is_open: true,
            state: SpriteEditorState::new(texture_path),
        }
    }

    /// Render the sprite editor window
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        texture_manager: &mut crate::texture_manager::TextureManager,
        dt: f32,
    ) {
        let title = format!("Sprite Editor - {}", self.state.texture_path.file_name().unwrap_or_default().to_string_lossy());
        
        let mut is_open = self.is_open;
        egui::Window::new(title)
            .open(&mut is_open)
            .default_size([800.0, 600.0])
            .show(ctx, |ui| {
                self.render_content(ui, texture_manager, dt);
            });
        self.is_open = is_open;
    }

    /// Render inline
    pub fn render_inline(
        &mut self,
        ui: &mut egui::Ui,
        texture_manager: &mut crate::texture_manager::TextureManager,
        dt: f32,
    ) {
        self.render_content(ui, texture_manager, dt);
    }

    /// Render the main content
    fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        texture_manager: &mut crate::texture_manager::TextureManager,
        _dt: f32,
    ) {
        // Top toolbar
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                if let Err(e) = self.state.save() {
                    log::error!("Failed to save sprite metadata: {}", e);
                } else {
                    log::info!("Saved sprite metadata to: {}", self.state.metadata_path.display());
                }
            }

            ui.separator();

            if ui.button("➕ Add Sprite").clicked() {
                self.state.metadata.sprites.push(SpriteDefinition {
                    name: format!("sprite_{}", self.state.metadata.sprites.len()),
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                });
            }

            if ui.button("🗑 Delete Selected").clicked() {
                if let Some(idx) = self.state.selected_sprite {
                    if idx < self.state.metadata.sprites.len() {
                        self.state.metadata.sprites.remove(idx);
                        self.state.selected_sprite = None;
                    }
                }
            }

            ui.separator();

            ui.checkbox(&mut self.state.show_grid, "Show Grid");
            if self.state.show_grid {
                ui.add(egui::DragValue::new(&mut self.state.grid_size).prefix("Grid: ").speed(1.0));
            }

            ui.separator();

            ui.label(format!("Zoom: {:.0}%", self.state.zoom * 100.0));
            if ui.button("🔍+").clicked() {
                self.state.zoom = (self.state.zoom * 1.2).min(5.0);
            }
            if ui.button("🔍-").clicked() {
                self.state.zoom = (self.state.zoom / 1.2).max(0.1);
            }
            if ui.button("Reset").clicked() {
                self.state.zoom = 1.0;
                self.state.pan_offset = egui::Vec2::ZERO;
            }
        });

        ui.separator();

        // Main content area - split into canvas and sprite list
        ui.horizontal(|ui| {
            // Left side - Canvas
            ui.vertical(|ui| {
                let available_size = ui.available_size();
                let canvas_size = egui::vec2(available_size.x * 0.7, available_size.y);
                
                self.render_canvas(ui, texture_manager, canvas_size);
            });

            ui.separator();

            // Right side - Sprite list
            ui.vertical(|ui| {
                ui.heading("Sprites");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, sprite) in self.state.metadata.sprites.iter_mut().enumerate() {
                        let is_selected = self.state.selected_sprite == Some(idx);
                        
                        ui.push_id(idx, |ui| {
                            let response = ui.selectable_label(is_selected, &sprite.name);
                            if response.clicked() {
                                self.state.selected_sprite = Some(idx);
                            }

                            if is_selected {
                                ui.indent(idx, |ui| {
                                    ui.text_edit_singleline(&mut sprite.name);
                                    ui.horizontal(|ui| {
                                        ui.label("X:");
                                        ui.add(egui::DragValue::new(&mut sprite.x).speed(1.0));
                                        ui.label("Y:");
                                        ui.add(egui::DragValue::new(&mut sprite.y).speed(1.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("W:");
                                        ui.add(egui::DragValue::new(&mut sprite.width).speed(1.0));
                                        ui.label("H:");
                                        ui.add(egui::DragValue::new(&mut sprite.height).speed(1.0));
                                    });
                                });
                            }
                        });
                    }
                });
            });
        });
    }

    /// Render the sprite canvas
    fn render_canvas(
        &mut self,
        ui: &mut egui::Ui,
        texture_manager: &mut crate::texture_manager::TextureManager,
        size: egui::Vec2,
    ) {
        let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

        // Draw background
        painter.rect_filled(response.rect, 0.0, egui::Color32::from_gray(40));

        // Try to load texture using absolute path to avoid double base_path joining
        let texture_path_str = self.state.texture_path.to_string_lossy().to_string();
        let texture_handle = texture_manager.load_texture_absolute(ui.ctx(), &texture_path_str, &self.state.texture_path);
        
        if let Some(texture_handle) = texture_handle {
            let texture_id = texture_handle.id();
            let texture_size = texture_handle.size();

            // Calculate scaled texture size
            let scaled_width = texture_size[0] as f32 * self.state.zoom;
            let scaled_height = texture_size[1] as f32 * self.state.zoom;

            // Center the texture with pan offset
            let texture_pos = response.rect.center() 
                + self.state.pan_offset 
                - egui::vec2(scaled_width / 2.0, scaled_height / 2.0);

            let texture_rect = egui::Rect::from_min_size(
                texture_pos,
                egui::vec2(scaled_width, scaled_height),
            );

            // Draw texture
            painter.image(
                texture_id,
                texture_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // Draw grid if enabled
            if self.state.show_grid {
                let grid_size = self.state.grid_size as f32 * self.state.zoom;
                let mut x = texture_rect.min.x;
                while x <= texture_rect.max.x {
                    painter.line_segment(
                        [egui::pos2(x, texture_rect.min.y), egui::pos2(x, texture_rect.max.y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50)),
                    );
                    x += grid_size;
                }
                let mut y = texture_rect.min.y;
                while y <= texture_rect.max.y {
                    painter.line_segment(
                        [egui::pos2(texture_rect.min.x, y), egui::pos2(texture_rect.max.x, y)],
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50)),
                    );
                    y += grid_size;
                }
            }

            // Draw sprite rectangles
            for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
                let is_selected = self.state.selected_sprite == Some(idx);
                
                let sprite_rect = egui::Rect::from_min_size(
                    texture_pos + egui::vec2(sprite.x as f32 * self.state.zoom, sprite.y as f32 * self.state.zoom),
                    egui::vec2(sprite.width as f32 * self.state.zoom, sprite.height as f32 * self.state.zoom),
                );

                let color = if is_selected {
                    egui::Color32::from_rgb(0, 255, 0)
                } else {
                    egui::Color32::from_rgb(255, 255, 0)
                };

                painter.rect_stroke(sprite_rect, 0.0, egui::Stroke::new(2.0, color));

                // Draw sprite name
                painter.text(
                    sprite_rect.min,
                    egui::Align2::LEFT_TOP,
                    &sprite.name,
                    egui::FontId::proportional(12.0),
                    color,
                );
            }

            // Handle mouse interactions - extract data first to avoid borrow issues
            let clicked_pos = if response.clicked() {
                response.interact_pointer_pos()
            } else {
                None
            };

            if let Some(pos) = clicked_pos {
                // Check if clicked on a sprite
                let mut clicked_sprite = None;
                for (idx, sprite) in self.state.metadata.sprites.iter().enumerate() {
                    let sprite_rect = egui::Rect::from_min_size(
                        texture_pos + egui::vec2(sprite.x as f32 * self.state.zoom, sprite.y as f32 * self.state.zoom),
                        egui::vec2(sprite.width as f32 * self.state.zoom, sprite.height as f32 * self.state.zoom),
                    );
                    if sprite_rect.contains(pos) {
                        clicked_sprite = Some(idx);
                        break;
                    }
                }
                self.state.selected_sprite = clicked_sprite;
            }

            // Handle panning
            let is_shift_pressed = ui.input(|i| i.modifiers.shift);
            if response.dragged() && is_shift_pressed {
                self.state.pan_offset += response.drag_delta();
            }
        } else {
            painter.text(
                response.rect.center(),
                egui::Align2::CENTER_CENTER,
                "Failed to load texture",
                egui::FontId::proportional(16.0),
                egui::Color32::RED,
            );
        }
    }
}
