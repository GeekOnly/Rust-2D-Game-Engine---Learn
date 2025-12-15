use ecs::{World, Prefab};
use egui;
use std::collections::HashMap;
use ecs::Entity;

/// Render the top menu bar with File, Edit, View, GameObject menus
pub fn render_menu_bar(
    ui: &mut egui::Ui,
    world: &mut World,
    entity_names: &mut HashMap<Entity, String>,
    new_scene_request: &mut bool,
    save_request: &mut bool,
    save_as_request: &mut bool,
    load_request: &mut bool,
    load_file_request: &mut Option<std::path::PathBuf>,
    play_request: &mut bool,
    stop_request: &mut bool,
    show_project_settings: &mut bool,
    show_colliders: &mut bool,
    show_velocities: &mut bool,
    show_debug_lines: &mut bool,
    project_path: &Option<std::path::PathBuf>,
    current_scene_path: &Option<std::path::PathBuf>,
    is_playing: bool,
    show_exit_dialog: &mut bool,
    show_export_dialog: &mut bool,
    layout_request: &mut Option<String>,
    current_layout_name: &str,
    get_scene_files_fn: impl Fn(&std::path::Path) -> Vec<String>,
) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New Scene").clicked() {
                *new_scene_request = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("💾 Save Scene (Ctrl+S)").clicked() {
                *save_request = true;
                ui.close_menu();
            }
            if ui.button("Save Scene As...").clicked() {
                *save_as_request = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Load Scene...").clicked() {
                *load_request = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Export Game...").clicked() {
                *show_export_dialog = true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("⬅ Back to Launcher (Ctrl+Q)").clicked() {
                *show_exit_dialog = true;
                ui.close_menu();
            }
        });
        ui.menu_button("Edit", |ui| {
            if ui.button("⚙ Project Settings").clicked() {
                *show_project_settings = true;
                ui.close_menu();
            }
        });
        ui.menu_button("View", |ui| {
            ui.label("🔧 Gizmos");
            ui.separator();
            ui.checkbox(show_colliders, "Show Colliders");
            ui.checkbox(show_velocities, "Show Velocities");
            ui.checkbox(show_debug_lines, "Show Debug Lines");
        });
        ui.menu_button("GameObject", |ui| {
            if ui.button("Create Empty").clicked() {
                let entity = Prefab::new("GameObject").spawn(world);
                entity_names.insert(entity, format!("GameObject {}", entity));
            }
            if ui.button("Create Player").clicked() {
                let entity = Prefab::player().spawn(world);
                entity_names.insert(entity, "Player".to_string());
            }
            if ui.button("Create Item").clicked() {
                let entity = Prefab::item().spawn(world);
                entity_names.insert(entity, format!("Item {}", entity));
            }
        });

        ui.separator();

        // Scene dropdown in toolbar (if project is open)
        if let Some(proj_path) = project_path {
            let scene_files = get_scene_files_fn(proj_path);

            if !scene_files.is_empty() {
                let current_scene_name = if let Some(current) = current_scene_path {
                    current.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_string()
                } else {
                    "No Scene".to_string()
                };

                egui::ComboBox::from_label("Scene:")
                    .selected_text(&current_scene_name)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for scene_file in scene_files {
                            let scene_name = std::path::Path::new(&scene_file)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&scene_file);

                            let scene_path = proj_path.join(&scene_file);

                            let is_current = if let Some(current) = current_scene_path {
                                current == &scene_path
                            } else {
                                false
                            };

                            if ui.selectable_label(is_current, scene_name).clicked() && !is_current {
                                *load_file_request = Some(scene_path);
                            }
                        }
                    });

                ui.separator();
            }
        }

        // Play/Stop buttons in menu bar (center)
        if !is_playing {
            if ui.button("▶ Play").clicked() {
                *play_request = true;
            }
        } else {
            if ui.button("⏹ Stop").clicked() {
                *stop_request = true;
            }
        }

        // Push layout dropdown to the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Layout dropdown (right side)
            egui::ComboBox::from_id_source("layout_selector")
                .selected_text(format!("📐 {}", current_layout_name))
                .show_ui(ui, |ui| {
                    ui.label("Built-in Layouts");
                    ui.separator();
                    
                    if ui.selectable_label(current_layout_name == "default", "Default").clicked() {
                        *layout_request = Some("load:default".to_string());
                    }
                    if ui.selectable_label(current_layout_name == "2column", "2 Columns").clicked() {
                        *layout_request = Some("load:2column".to_string());
                    }
                    if ui.selectable_label(current_layout_name == "tall", "Tall").clicked() {
                        *layout_request = Some("load:tall".to_string());
                    }
                    if ui.selectable_label(current_layout_name == "wide", "Wide").clicked() {
                        *layout_request = Some("load:wide".to_string());
                    }
                    
                    // Load and display custom layouts
                    if let Some(proj_path) = project_path {
                        let custom_layouts = super::load_custom_layouts(proj_path);
                        if !custom_layouts.is_empty() {
                            ui.separator();
                            ui.label("Custom Layouts");
                            ui.separator();
                            
                            for (name, _) in custom_layouts {
                                if ui.selectable_label(current_layout_name == name, &name).clicked() {
                                    *layout_request = Some(format!("custom:{}", name));
                                }
                            }
                        }
                    }
                    
                    ui.separator();
                    if ui.button("💾 Save Layout As...").clicked() {
                        *layout_request = Some("save_as".to_string());
                    }
                    if ui.button("✓ Set as Default").clicked() {
                        *layout_request = Some("save_default".to_string());
                    }
                });
        });
    });
}
