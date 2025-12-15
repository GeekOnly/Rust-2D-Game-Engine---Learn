use egui;
use std::path::PathBuf;

/// Renders the Project Settings window with General and Play Mode sections.
///
/// # Parameters
/// - `ctx`: The egui context for rendering
/// - `show_project_settings`: Boolean flag to control window visibility
/// - `project_path`: Optional path to the currently open project
/// - `get_scene_files_fn`: Closure function to retrieve scene files from a path
pub fn render_project_settings(
    ctx: &egui::Context,
    show_project_settings: &mut bool,
    project_path: &Option<PathBuf>,
    get_scene_files_fn: impl Fn(&std::path::Path) -> Vec<String>,
) {
    // Project Settings Dialog
    if *show_project_settings {
        egui::Window::new("⚙ Project Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .show(ctx, |ui| {
                if let Some(path) = project_path {
                    use engine_core::project::ProjectManager;

                    ui.heading("Project Configuration");
                    ui.separator();

                    // General Section
                    ui.collapsing("📁 General", |ui| {
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Project Name:");
                            ui.label(egui::RichText::new(
                                path.file_name().unwrap_or_default().to_string_lossy().to_string()
                            ).strong());
                        });
                        ui.horizontal(|ui| {
                            ui.label("Project Path:");
                            ui.label(path.display().to_string());
                        });
                        ui.add_space(5.0);
                    });

                    ui.add_space(10.0);

                    // Play Mode Section
                    ui.collapsing("🎮 Play Mode", |ui| {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("Configure startup scenes:").strong());
                        ui.add_space(10.0);

                        // Editor Startup Scene
                        ui.label(egui::RichText::new("Editor Startup Scene").strong());
                        ui.label("Scene to load when opening project in Editor");
                        ui.add_space(5.0);

                        let mut current_editor_scene = String::new();
                        if let Ok(pm) = ProjectManager::new() {
                            if let Ok(Some(scene)) = pm.get_editor_startup_scene(path) {
                                current_editor_scene = scene.to_string_lossy().to_string();
                            }
                        }

                        let mut new_editor_scene = current_editor_scene.clone();

                        // Get all .scene files in project
                        let scene_files = get_scene_files_fn(path);

                        // Dropdown to select scene
                        let selected_text = if new_editor_scene.is_empty() {
                            "(None)".to_string()
                        } else {
                            new_editor_scene.clone()
                        };

                        egui::ComboBox::from_label("")
                            .selected_text(&selected_text)
                            .width(400.0)
                            .show_ui(ui, |ui| {
                                // None option
                                if ui.selectable_value(&mut new_editor_scene, String::new(), "(None)").clicked() {
                                    new_editor_scene.clear();
                                }

                                ui.separator();

                                // All .scene files
                                for scene_file in scene_files {
                                    ui.selectable_value(&mut new_editor_scene, scene_file.clone(), &scene_file);
                                }
                            });

                        if new_editor_scene != current_editor_scene {
                            if let Ok(pm) = ProjectManager::new() {
                                let scene_path = if new_editor_scene.is_empty() {
                                    None
                                } else {
                                    Some(std::path::PathBuf::from(&new_editor_scene))
                                };
                                let _ = pm.set_editor_startup_scene(path, scene_path);
                            }
                        }

                        ui.add_space(15.0);

                        // Game Startup Scene
                        ui.label(egui::RichText::new("Game Startup Scene").strong());
                        ui.label("Scene to load when running exported game");
                        ui.add_space(5.0);

                        let mut current_game_scene = String::new();
                        if let Ok(pm) = ProjectManager::new() {
                            if let Ok(Some(scene)) = pm.get_game_startup_scene(path) {
                                current_game_scene = scene.to_string_lossy().to_string();
                            }
                        }

                        let mut new_game_scene = current_game_scene.clone();

                        // Get all .scene files in project
                        let scene_files = get_scene_files_fn(path);

                        // Dropdown to select scene
                        let selected_text = if new_game_scene.is_empty() {
                            "(None)".to_string()
                        } else {
                            new_game_scene.clone()
                        };

                        egui::ComboBox::from_label("")
                            .selected_text(&selected_text)
                            .width(400.0)
                            .show_ui(ui, |ui| {
                                // None option
                                if ui.selectable_value(&mut new_game_scene, String::new(), "(None)").clicked() {
                                    new_game_scene.clear();
                                }

                                ui.separator();

                                // All .scene files
                                for scene_file in scene_files {
                                    ui.selectable_value(&mut new_game_scene, scene_file.clone(), &scene_file);
                                }
                            });

                        if new_game_scene != current_game_scene {
                            if let Ok(pm) = ProjectManager::new() {
                                let scene_path = if new_game_scene.is_empty() {
                                    None
                                } else {
                                    Some(std::path::PathBuf::from(&new_game_scene))
                                };
                                let _ = pm.set_game_startup_scene(path, scene_path);
                            }
                        }

                        ui.add_space(10.0);
                    });

                } else {
                    ui.label("No project open.");
                }

                ui.add_space(20.0);
                ui.separator();
                if ui.button("Close").clicked() {
                    *show_project_settings = false;
                }
            });
    }
}
