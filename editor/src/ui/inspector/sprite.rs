use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;

pub fn render_sprite_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    sprite_picker_state: &mut crate::ui::sprite_picker::SpritePickerState,
    open_sprite_editor_request: &mut Option<std::path::PathBuf>,
) {
    // Sprite Component (Unity-style collapsible)
    let has_sprite = world.has_component(entity, ComponentType::Sprite);
    let mut remove_sprite = false;
    
    if has_sprite {
        let sprite_id = ui.make_persistent_id("sprite_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), sprite_id, true
        );
        
        render_component_header(ui, "Sprite Renderer", "🎨", false);
        
        if is_open.is_open() {
            if let Some(sprite) = world.sprites.get_mut(&entity) {
                ui.indent("sprite_indent", |ui| {
                    egui::Grid::new("sprite_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            // Sprite picker (Unity-style)
                            ui.label("Sprite");
                            ui.horizontal(|ui| {
                                // Show current sprite name with icon
                                let sprite_display = if sprite.texture_id.is_empty() {
                                    "None (Sprite)".to_string()
                                } else {
                                    sprite.texture_id.clone()
                                };

                                // Clickable sprite field (Unity-style)
                                if ui.button(&sprite_display).clicked() {
                                    sprite_picker_state.open();
                                    log::info!("Opened sprite picker for Sprite Renderer");
                                }

                                // Manual text edit option
                                ui.menu_button("⋮", |ui| {
                                    ui.label("Edit manually:");
                                    ui.text_edit_singleline(&mut sprite.texture_id);
                                });
                            });
                            ui.end_row();
                            
                            // Color tint
                            ui.label("Color");
                            ui.color_edit_button_rgba_unmultiplied(&mut sprite.color);
                            ui.end_row();
                            
                            // Flip options (Unity-style)
                            ui.label("Flip");
                            ui.horizontal(|ui| {
                                ui.label("X");
                                ui.checkbox(&mut sprite.flip_x, "");
                                ui.label("Y");
                                ui.checkbox(&mut sprite.flip_y, "");
                            });
                            ui.end_row();

                            // Draw Mode
                            ui.label("Draw Mode");
                            egui::ComboBox::from_id_source("draw_mode")
                                .selected_text("Simple")
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(true, "Simple");
                                    let _ = ui.selectable_label(false, "Sliced");
                                    let _ = ui.selectable_label(false, "Tiled");
                                });
                            ui.end_row();

                            // Billboard (3D mode)
                            ui.label("Billboard");
                            ui.checkbox(&mut sprite.billboard, "")
                                .on_hover_text("Always face camera in 3D mode");
                            ui.end_row();
                        });
                    
                    ui.add_space(5.0);
                    
                    // Info message about scale
                    ui.label(egui::RichText::new("💡 Use Transform Scale to resize sprite")
                        .small()
                        .color(egui::Color32::from_rgb(150, 150, 150)));
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }
                        if ui.button("🎨 Open Sprite Editor").clicked() {
                            // Request to open sprite editor
                             if !sprite.texture_id.is_empty() {
                                // Assuming texture_id can be mapped to a path or use asset manager lookup
                                // keeping logic simple as in legacy:
                                // *open_sprite_editor_request = Some(path);
                             }
                        }
                        if ui.button("❌ Remove Component").clicked() {
                            remove_sprite = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_sprite {
        let _ = world.remove_component(entity, ComponentType::Sprite);
    }

    // SpriteSheet Component (Unity-style collapsible)
    let has_sprite_sheet = world.has_component(entity, ComponentType::SpriteSheet);
    let mut remove_sprite_sheet = false;
    
    if has_sprite_sheet {
        let sprite_sheet_id = ui.make_persistent_id("sprite_sheet_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), sprite_sheet_id, true
        );
        
        render_component_header(ui, "Sprite Sheet", "🎞", false);
        
        if is_open.is_open() {
            if let Some(sprite_sheet) = world.sprite_sheets.get_mut(&entity) {
                ui.indent("sprite_sheet_indent", |ui| {
                    egui::Grid::new("sprite_sheet_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            // Sprite name display
                            ui.label("Sprite Name");
                            if !sprite_sheet.frames.is_empty() {
                                // Show first frame name or "Multiple frames"
                                let display_name = if sprite_sheet.frames.len() == 1 {
                                    sprite_sheet.frames[0].name.as_ref()
                                        .map(|n| n.as_str())
                                        .unwrap_or("Unnamed")
                                } else {
                                    "Multiple frames"
                                };
                                ui.label(egui::RichText::new(display_name).strong());
                            } else {
                                ui.label(egui::RichText::new("No sprites").color(egui::Color32::GRAY));
                            }
                            ui.end_row();
                            
                            // Source texture path
                            ui.label("Texture Path");
                            ui.label(&sprite_sheet.texture_path);
                            ui.end_row();
                            
                            // Texture dimensions
                            ui.label("Texture Size");
                            ui.label(format!("{}x{}", sprite_sheet.sheet_width, sprite_sheet.sheet_height));
                            ui.end_row();
                            
                            // Frame count
                            ui.label("Frame Count");
                            ui.label(format!("{}", sprite_sheet.frames.len()));
                            ui.end_row();
                        });
                    
                    ui.add_space(5.0);
                    
                    // Info message
                    ui.label(egui::RichText::new("💡 Edit sprite definitions in the Sprite Editor")
                        .small()
                        .color(egui::Color32::from_rgb(150, 150, 150)));
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                            // Component menu
                        }

                        // Select Sprite button - opens sprite picker
                        if ui.button("🖼️ Select Sprite").on_hover_text("Choose a sprite from project").clicked() {
                            sprite_picker_state.open();
                            log::info!("Opened sprite picker");
                        }

                        // Edit Sprite Sheet button - opens sprite editor
                        if ui.button("🎨 Edit Sprite Sheet").clicked() {
                            // Request to open sprite editor for this texture
                            let texture_path = std::path::PathBuf::from(&sprite_sheet.texture_path);
                            *open_sprite_editor_request = Some(texture_path);
                            log::info!("Requested to open sprite editor for: {}", sprite_sheet.texture_path);
                        }

                        if ui.button("❌ Remove Component").clicked() {
                            remove_sprite_sheet = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_sprite_sheet {
        let _ = world.remove_component(entity, ComponentType::SpriteSheet);
    }
}
