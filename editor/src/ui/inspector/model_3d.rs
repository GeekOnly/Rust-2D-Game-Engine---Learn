use ecs::{World, Entity, ComponentType, ComponentManager};
use egui;
use super::utils::render_component_header;
use std::path::Path;

pub fn render_model_3d_inspector(ui: &mut egui::Ui, world: &mut World, entity: Entity, project_path: Option<&Path>) {
    // Model 3D Component (XSG)
    let has_model = world.has_component(entity, ComponentType::Model3D);
    let mut remove_model = false;
    
    if has_model {
        let model_id = ui.make_persistent_id("model_3d_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), model_id, true
        );
        
        render_component_header(ui, "Model 3D (XSG)", "🏛️", false);
        
        if is_open.is_open() {
            if let Some(model) = world.model_3ds.get_mut(&entity) {
                ui.indent("model_3d_indent", |ui| {
                    egui::Grid::new("model_3d_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Asset Path");
                            ui.horizontal(|ui| {
                                let mut path = model.asset_id.clone();
                                let response = ui.text_edit_singleline(&mut path);
                                if response.changed() {
                                    model.asset_id = path.clone();
                                }
                                
                                if ui.button("📁").on_hover_text("Browse...").clicked() {
                                    // Set default directory to project's assets folder if available
                                    let mut dialog = rfd::FileDialog::new()
                                        .add_filter("XSG Model", &["xsg"]);

                                    if let Some(proj_path) = project_path {
                                        let assets_path = proj_path.join("assets");
                                        if assets_path.exists() {
                                            dialog = dialog.set_directory(&assets_path);
                                        }
                                    }

                                    if let Some(file_path) = dialog.pick_file() {
                                        let mut new_path = file_path.to_string_lossy().to_string();
                                        
                                        // Try to make path relative to project folder
                                        if let Some(proj_path) = project_path {
                                            // Canonicalize paths to handle separators consistently
                                            if let (Ok(abs_file), Ok(abs_proj)) = (std::fs::canonicalize(&file_path), std::fs::canonicalize(proj_path)) {
                                                if let Ok(relative) = abs_file.strip_prefix(&abs_proj) {
                                                    // Ensure forward slashes for cross-platform consistency
                                                    new_path = relative.to_string_lossy().replace("\\", "/");
                                                }
                                            } else {
                                                // Fallback: simple string manipulation if canonicalize fails (e.g. non-existent file)
                                                let proj_str = proj_path.to_string_lossy().to_string();
                                                if new_path.starts_with(&proj_str) {
                                                     new_path = new_path.replace(&proj_str, "").trim_start_matches(|c| c == '\\' || c == '/').replace("\\", "/");
                                                }
                                            }
                                        }
                                        
                                        model.asset_id = new_path;
                                    }
                                }
                            });
                            ui.end_row();

                            ui.label("");
                            ui.colored_label(
                                egui::Color32::from_rgb(150, 150, 150),
                                "💡 Path relative to project folder (.xsg)"
                            );
                            ui.end_row();
                            
                            // Reload info
                            ui.label("Status");
                            ui.label(if model.asset_id.ends_with(".xsg") { 
                                egui::RichText::new("Valid Extension").color(egui::Color32::GREEN) 
                            } else { 
                                egui::RichText::new("Invalid Extension (must be .xsg)").color(egui::Color32::RED) 
                            });
                            ui.end_row();
                        });
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("❌ Remove Component").clicked() {
                            remove_model = true;
                        }
                    });
                });
            }
            ui.add_space(10.0);
        }
    }
    
    if remove_model {
        let _ = world.remove_component(entity, ComponentType::Model3D);
    }
}
