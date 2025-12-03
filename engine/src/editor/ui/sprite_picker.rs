/// Sprite Picker - Unity-style sprite selection popup
///
/// Displays all sprites from sprite sheets in the project
/// Allows user to select a sprite to assign to a SpriteSheet component

use std::path::PathBuf;

/// Result from sprite picker
#[derive(Debug, Clone)]
pub struct SpritePickerResult {
    pub sprite_file_path: PathBuf,  // Path to the .sprite file
    pub texture_path: PathBuf,       // Path to the texture image
    pub sprite_name: String,
    pub frame_index: usize,
}

/// State for sprite picker popup
pub struct SpritePickerState {
    /// Whether the picker is open
    pub is_open: bool,
    /// Search filter text
    pub search_filter: String,
    /// Selected sprite (if any)
    pub selected: Option<SpritePickerResult>,
    /// Scroll position
    pub scroll_offset: f32,
}

impl Default for SpritePickerState {
    fn default() -> Self {
        Self {
            is_open: false,
            search_filter: String::new(),
            selected: None,
            scroll_offset: 0.0,
        }
    }
}

impl SpritePickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.search_filter.clear();
        self.selected = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Render sprite picker popup window
pub fn render_sprite_picker(
    ctx: &egui::Context,
    state: &mut SpritePickerState,
    project_path: Option<&PathBuf>,
    texture_manager: &mut crate::texture_manager::TextureManager,
) -> Option<SpritePickerResult> {
    if !state.is_open {
        return None;
    }

    let mut result = None;
    let mut should_close = false;

    egui::Window::new("🖼️ Sprite Picker")
        .open(&mut state.is_open)
        .default_size([600.0, 500.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Search bar
            ui.horizontal(|ui| {
                ui.label("🔍 Search:");
                ui.text_edit_singleline(&mut state.search_filter);
                if ui.button("Clear").clicked() {
                    state.search_filter.clear();
                }
            });

            ui.separator();

            // Get all sprite files in project
            if let Some(project_path) = project_path {
                // Search for .sprite files recursively from assets directory
                let assets_dir = project_path.join("assets");

                if !assets_dir.exists() {
                    ui.label("No assets directory found in project");
                    ui.label(format!("Expected: {}", assets_dir.display()));
                    return;
                }

                // Scan for .sprite files recursively
                let sprite_files = find_sprite_files(&assets_dir);

                if sprite_files.is_empty() {
                    ui.label("No sprite files found in assets directory");
                    ui.label("Create sprites using the Sprite Editor");
                    return;
                }

                // Display sprites in a grid
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for sprite_file in &sprite_files {
                            if let Ok(metadata) = load_sprite_metadata(sprite_file) {
                                // Filter by search
                                let file_name = sprite_file.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("");

                                if !state.search_filter.is_empty()
                                    && !file_name.to_lowercase().contains(&state.search_filter.to_lowercase()) {
                                    continue;
                                }

                                // Show sprite file as a group
                                egui::CollapsingHeader::new(format!("📄 {}", file_name))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        // Grid of sprites
                                        let available_width = ui.available_width();
                                        let sprite_size = 80.0;
                                        let spacing = 10.0;
                                        let sprites_per_row = ((available_width - spacing) / (sprite_size + spacing)).floor().max(1.0) as usize;

                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                                            for (idx, sprite) in metadata.sprites.iter().enumerate() {
                                                // Try to load texture and show preview
                                                let texture_path_full = project_path.join(&metadata.texture_path);
                                                let texture_id = format!("sprite_picker_{}_{}", file_name, idx);
                                                
                                                // Create a frame for the sprite
                                                let (rect, response) = ui.allocate_exact_size(
                                                    egui::vec2(sprite_size, sprite_size),
                                                    egui::Sense::click()
                                                );
                                                
                                                // Draw background
                                                ui.painter().rect_filled(
                                                    rect,
                                                    2.0,
                                                    if response.hovered() {
                                                        egui::Color32::from_rgb(60, 60, 70)
                                                    } else {
                                                        egui::Color32::from_rgb(45, 45, 50)
                                                    }
                                                );
                                                
                                                // Try to load and draw texture
                                                if let Some(texture) = texture_manager.load_texture(
                                                    ctx,
                                                    &texture_id,
                                                    std::path::Path::new(&metadata.texture_path)
                                                ) {
                                                    // Calculate UV coordinates for this sprite
                                                    let tex_size = texture.size();
                                                    let u_min = sprite.x as f32 / tex_size[0] as f32;
                                                    let v_min = sprite.y as f32 / tex_size[1] as f32;
                                                    let u_max = (sprite.x + sprite.width) as f32 / tex_size[0] as f32;
                                                    let v_max = (sprite.y + sprite.height) as f32 / tex_size[1] as f32;
                                                    
                                                    // Draw sprite with proper UV coordinates
                                                    let sprite_rect = rect.shrink(4.0); // Add padding
                                                    ui.painter().image(
                                                        texture.id(),
                                                        sprite_rect,
                                                        egui::Rect::from_min_max(
                                                            egui::pos2(u_min, v_min),
                                                            egui::pos2(u_max, v_max)
                                                        ),
                                                        egui::Color32::WHITE
                                                    );
                                                } else {
                                                    // Fallback: show sprite name
                                                    ui.painter().text(
                                                        rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        &sprite.name,
                                                        egui::FontId::proportional(10.0),
                                                        egui::Color32::WHITE
                                                    );
                                                }

                                                if response.clicked() {
                                                    // Use the relative texture path from metadata
                                                    // metadata.texture_path is already relative to project root
                                                    let texture_path = PathBuf::from(&metadata.texture_path);

                                                    result = Some(SpritePickerResult {
                                                        sprite_file_path: sprite_file.clone(),
                                                        texture_path,
                                                        sprite_name: sprite.name.clone(),
                                                        frame_index: idx,
                                                    });
                                                    should_close = true;
                                                }

                                                // Show tooltip with sprite info
                                                response.on_hover_ui(|ui| {
                                                    ui.label(&sprite.name);
                                                    ui.label(format!("{}×{} px", sprite.width, sprite.height));
                                                    ui.label(format!("Position: ({}, {})", sprite.x, sprite.y));
                                                });

                                                // Add spacing for grid layout
                                                if (idx + 1) % sprites_per_row == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                    });

                                ui.add_space(5.0);
                            }
                        }
                    });
            } else {
                ui.label("No project open");
            }
        });

    if should_close {
        state.close();
    }

    result
}

/// Find all .sprite files recursively
fn find_sprite_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                files.extend(find_sprite_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("sprite") {
                files.push(path);
            }
        }
    }

    files
}

/// Load sprite metadata from .sprite file
fn load_sprite_metadata(path: &PathBuf) -> Result<crate::editor::SpriteMetadata, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let metadata: crate::editor::SpriteMetadata = serde_json::from_str(&contents)?;
    Ok(metadata)
}
