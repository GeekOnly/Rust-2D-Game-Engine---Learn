//! Texture Inspector UI (Unity-style)
//!
//! Shows texture import settings when a texture is selected in Asset Browser.

use crate::editor::texture_import_settings::*;
use egui;
use std::path::PathBuf;

/// Texture Inspector state
pub struct TextureInspector {
    /// Currently selected texture path
    pub selected_texture: Option<PathBuf>,
    /// Current import settings
    pub settings: TextureImportSettings,
    /// Has unsaved changes
    pub has_changes: bool,
}

impl Default for TextureInspector {
    fn default() -> Self {
        Self {
            selected_texture: None,
            settings: TextureImportSettings::default(),
            has_changes: false,
        }
    }
}

impl TextureInspector {
    /// Set the selected texture and load its settings
    pub fn set_texture(&mut self, texture_path: PathBuf) {
        self.selected_texture = Some(texture_path.clone());
        self.settings = TextureImportSettings::load(&texture_path)
            .unwrap_or_default();
        self.has_changes = false;
    }
    
    /// Clear selection
    pub fn clear(&mut self) {
        self.selected_texture = None;
        self.settings = TextureImportSettings::default();
        self.has_changes = false;
    }
    
    /// Render the inspector UI
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<TextureInspectorAction> {
        let mut action = None;
        
        if let Some(texture_path) = &self.selected_texture {
            // Header
            ui.heading("Texture Import Settings");
            ui.separator();
            
            // Texture info
            if let Some(filename) = texture_path.file_name() {
                ui.label(format!("📄 {}", filename.to_string_lossy()));
            }
            ui.add_space(10.0);
            
            // Texture Type
            egui::Grid::new("texture_settings_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Texture Type");
                    let mut changed = false;
                    egui::ComboBox::from_id_source("texture_type")
                        .selected_text(format!("{:?}", self.settings.texture_type))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.settings.texture_type, TextureType::Sprite2D, "Sprite (2D and UI)").changed();
                            changed |= ui.selectable_value(&mut self.settings.texture_type, TextureType::Default, "Default").changed();
                            changed |= ui.selectable_value(&mut self.settings.texture_type, TextureType::NormalMap, "Normal Map").changed();
                        });
                    if changed { self.has_changes = true; }
                    ui.end_row();
                    
                    // Sprite-specific settings
                    if self.settings.texture_type == TextureType::Sprite2D {
                        ui.label("Sprite Mode");
                        let mut changed = false;
                        egui::ComboBox::from_id_source("sprite_mode")
                            .selected_text(format!("{:?}", self.settings.sprite_mode))
                            .show_ui(ui, |ui| {
                                changed |= ui.selectable_value(&mut self.settings.sprite_mode, SpriteMode::Single, "Single").changed();
                                changed |= ui.selectable_value(&mut self.settings.sprite_mode, SpriteMode::Multiple, "Multiple").changed();
                            });
                        if changed { self.has_changes = true; }
                        ui.end_row();
                        
                        ui.label("Pixels Per Unit");
                        if ui.add(egui::DragValue::new(&mut self.settings.pixels_per_unit)
                            .speed(1.0)
                            .clamp_range(1.0..=1000.0)).changed() {
                            self.has_changes = true;
                        }
                        ui.end_row();
                    }
                    
                    // Filter Mode
                    ui.label("Filter Mode");
                    let mut changed = false;
                    egui::ComboBox::from_id_source("filter_mode")
                        .selected_text(format!("{:?}", self.settings.filter_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.settings.filter_mode, FilterMode::Point, "Point (no filter)").changed();
                            changed |= ui.selectable_value(&mut self.settings.filter_mode, FilterMode::Bilinear, "Bilinear").changed();
                            changed |= ui.selectable_value(&mut self.settings.filter_mode, FilterMode::Trilinear, "Trilinear").changed();
                        });
                    if changed { self.has_changes = true; }
                    ui.end_row();
                    
                    // Wrap Mode
                    ui.label("Wrap Mode");
                    let mut changed = false;
                    egui::ComboBox::from_id_source("wrap_mode")
                        .selected_text(format!("{:?}", self.settings.wrap_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.settings.wrap_mode, WrapMode::Clamp, "Clamp").changed();
                            changed |= ui.selectable_value(&mut self.settings.wrap_mode, WrapMode::Repeat, "Repeat").changed();
                            changed |= ui.selectable_value(&mut self.settings.wrap_mode, WrapMode::Mirror, "Mirror").changed();
                        });
                    if changed { self.has_changes = true; }
                    ui.end_row();
                    
                    // Max Size
                    ui.label("Max Size");
                    let mut changed = false;
                    egui::ComboBox::from_id_source("max_size")
                        .selected_text(format!("{}", self.settings.max_size))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.settings.max_size, 256, "256").changed();
                            changed |= ui.selectable_value(&mut self.settings.max_size, 512, "512").changed();
                            changed |= ui.selectable_value(&mut self.settings.max_size, 1024, "1024").changed();
                            changed |= ui.selectable_value(&mut self.settings.max_size, 2048, "2048").changed();
                            changed |= ui.selectable_value(&mut self.settings.max_size, 4096, "4096").changed();
                        });
                    if changed { self.has_changes = true; }
                    ui.end_row();
                    
                    // Compression
                    ui.label("Compression");
                    let mut changed = false;
                    egui::ComboBox::from_id_source("compression")
                        .selected_text(format!("{:?}", self.settings.compression))
                        .show_ui(ui, |ui| {
                            changed |= ui.selectable_value(&mut self.settings.compression, CompressionFormat::None, "None").changed();
                            changed |= ui.selectable_value(&mut self.settings.compression, CompressionFormat::LowQuality, "Low Quality").changed();
                            changed |= ui.selectable_value(&mut self.settings.compression, CompressionFormat::NormalQuality, "Normal Quality").changed();
                            changed |= ui.selectable_value(&mut self.settings.compression, CompressionFormat::HighQuality, "High Quality").changed();
                        });
                    if changed { self.has_changes = true; }
                    ui.end_row();
                });
            
            ui.add_space(10.0);
            
            // Advanced settings
            ui.collapsing("Advanced", |ui| {
                if ui.checkbox(&mut self.settings.srgb, "sRGB (Color Texture)").changed() {
                    self.has_changes = true;
                }
                if ui.checkbox(&mut self.settings.alpha_is_transparency, "Alpha Is Transparency").changed() {
                    self.has_changes = true;
                }
                if ui.checkbox(&mut self.settings.read_write_enabled, "Read/Write Enabled").changed() {
                    self.has_changes = true;
                }
                if ui.checkbox(&mut self.settings.generate_mipmaps, "Generate Mipmaps").changed() {
                    self.has_changes = true;
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            
            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("💾 Apply").clicked() && self.has_changes {
                    action = Some(TextureInspectorAction::Apply);
                }
                
                if ui.button("↺ Revert").clicked() && self.has_changes {
                    action = Some(TextureInspectorAction::Revert);
                }
                
                if self.has_changes {
                    ui.label(egui::RichText::new("● Unsaved changes").color(egui::Color32::YELLOW));
                }
            });
            
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a texture to view import settings");
            });
        }
        
        action
    }
}

/// Actions that can be triggered from the inspector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureInspectorAction {
    /// Apply settings and save to .meta file
    Apply,
    /// Revert to saved settings
    Revert,
}
