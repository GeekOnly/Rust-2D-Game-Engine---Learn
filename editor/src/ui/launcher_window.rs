use egui;
use crate::states::{AppState, LauncherState, EditorState};
use engine_core::project::ProjectManager;
// Sample game removed - use projects/ folder for game content
// use crate::EditorMod; // If needed

pub struct LauncherWindow;

impl LauncherWindow {
    pub fn render(
        ctx: &egui::Context,
        app_state: &mut AppState,
        launcher_state: &mut LauncherState,
        editor_state: &mut EditorState,
        asset_loader: &dyn engine_core::assets::AssetLoader,
        // editor_mod: &mut EditorMod, // If needed
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎮 Rust 2D Game Engine");
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("➕ New Project").clicked() {
                    launcher_state.show_new_project_dialog = true;
                    launcher_state.new_project_name.clear();
                    launcher_state.new_project_desc.clear();
                }

                if ui.button("📁 Open Project").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        match launcher_state.project_manager.open_project(&folder) {
                            Ok(_) => {
                                // Initialize editor state with project
                                *app_state = AppState::Editor;
                                *editor_state = EditorState::new();
                                editor_state.set_project_path(folder.clone());

                                // Log to console
                                log::info!("Project opened: {}", folder.display());
                                editor_state.console.info(format!("📁 Project opened: {}", folder.display()));
                                editor_state.console.info("👋 Welcome to Rust 2D Game Engine!");
                                editor_state.console.debug("Debug logging enabled".to_string());

                                // Load editor layout
                                editor_state.load_editor_layout();

                                // Try to load last opened scene first, then startup scene
                                Self::load_initial_scene(editor_state, launcher_state, &folder, asset_loader);
                            }
                            Err(e) => {
                                launcher_state.error_message = Some(format!("Error: {}", e));
                            }
                        }
                    }
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Example projects section
            ui.heading("📦 Example Projects");
            ui.add_space(5.0);

            for (name, desc) in ProjectManager::get_example_projects() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(name);
                            ui.label(desc);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Open").clicked() {
                                Self::open_example_project(name, desc, app_state, launcher_state, editor_state, asset_loader);
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Recent projects
            ui.heading("📂 Recent Projects");
            ui.add_space(5.0);

            match launcher_state.project_manager.list_projects() {
                Ok(projects) => {
                    if projects.is_empty() {
                        ui.label("No projects yet. Create a new one to get started!");
                    } else {
                        for project in projects.iter() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.strong(&project.name);
                                        ui.label(&project.description);
                                        ui.label(format!("Last modified: {}", project.last_modified));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("🗑 Delete").clicked() {
                                            if let Err(e) = launcher_state.project_manager.delete_project(&project.path) {
                                                launcher_state.error_message = Some(format!("Error: {}", e));
                                            }
                                        }
                                        if ui.button("▶ Open").clicked() {
                                            if let Err(e) = launcher_state.project_manager.open_project(&project.path) {
                                                launcher_state.error_message = Some(format!("Error: {}", e));
                                                return;
                                            }
                                            
                                            // Open existing projects in editor mode
                                            *app_state = AppState::Editor;
                                            *editor_state = EditorState::new();
                                            editor_state.set_project_path(project.path.clone());
                                            
                                            Self::load_initial_scene(editor_state, launcher_state, &project.path, asset_loader);
                                        }
                                    });
                                });
                            });
                            ui.add_space(5.0);
                        }
                    }
                }
                Err(e) => {
                    ui.label(format!("Error loading projects: {}", e));
                }
            }

            // Error message
            if let Some(ref error) = launcher_state.error_message {
                ui.add_space(10.0);
                ui.colored_label(egui::Color32::RED, error);
            }
        });

        // New project dialog
        if launcher_state.show_new_project_dialog {
            egui::Window::new("Create New Project")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Project Name:");
                    ui.text_edit_singleline(&mut launcher_state.new_project_name);
                    
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut launcher_state.new_project_desc);

                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            match launcher_state.project_manager.create_project(
                                &launcher_state.new_project_name, 
                                &launcher_state.new_project_desc
                            ) {
                                Ok(metadata) => {
                                    *app_state = AppState::Editor;
                                    *editor_state = EditorState::new();
                                    editor_state.set_project_path(metadata.path);
                                    launcher_state.show_new_project_dialog = false;
                                }
                                Err(e) => {
                                    launcher_state.error_message = Some(format!("Error: {}", e));
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            launcher_state.show_new_project_dialog = false;
                        }
                    });
                });
        }
    }

    fn load_initial_scene(editor_state: &mut EditorState, launcher_state: &LauncherState, folder: &std::path::Path, asset_loader: &dyn engine_core::assets::AssetLoader) {
        let mut scene_loaded = false;
                                                        
        // 1. Try last opened scene
        if let Ok(Some(last_scene)) = launcher_state.project_manager.get_last_opened_scene(folder) {
            let scene_path = folder.join(&last_scene);
            if scene_path.exists() {
                if let Err(e) = editor_state.load_scene(&scene_path, asset_loader) {
                    editor_state.console.error(format!("Failed to load last scene: {}", e));
                } else {
                    editor_state.current_scene_path = Some(scene_path.clone());
                    editor_state.console.info(format!("Loaded last scene: {}", last_scene.display()));
                    scene_loaded = true;
                }
            }
        }
        
        // 2. If no last scene, try startup scene
        if !scene_loaded {
            if let Ok(Some(startup_scene)) = launcher_state.project_manager.get_startup_scene(folder) {
                let scene_path = folder.join(&startup_scene);
                if scene_path.exists() {
                    if let Err(e) = editor_state.load_scene(&scene_path, asset_loader) {
                        editor_state.console.error(format!("Failed to load startup scene: {}", e));
                    } else {
                        editor_state.current_scene_path = Some(scene_path.clone());
                        editor_state.console.info(format!("Loaded startup scene: {}", startup_scene.display()));
                    }
                }
            }
        }
    }

    fn open_example_project(
        name: &str, 
        _desc: &str, 
        app_state: &mut AppState, 
        launcher_state: &mut LauncherState, 
        editor_state: &mut EditorState,
        asset_loader: &dyn engine_core::assets::AssetLoader,
    ) {
         // Check if this is an existing example project (Celeste Demo or FPS 3D Example)
         if name == "Celeste Demo" || name == "FPS 3D Example" {
            // Try multiple possible paths
            let possible_paths = vec![
                std::path::PathBuf::from(format!("projects/{}", name)),
                std::path::PathBuf::from(format!("../projects/{}", name)),
            ];
            
            let project_path = possible_paths.iter()
                .find(|p| p.exists())
                .cloned();
            
            if let Some(project_path) = project_path {
                match launcher_state.project_manager.open_project(&project_path) {
                    Ok(_) => {
                        *app_state = AppState::Editor;
                        *editor_state = EditorState::new();
                        editor_state.set_project_path(project_path.clone());
                        
                        // Load the main scene
                        let scene_path = project_path.join("scenes/main.json");
                        log::info!("Attempting to load scene: {:?}", scene_path);
                        if scene_path.exists() {
                            match editor_state.load_scene(&scene_path, asset_loader) {
                                Ok(_) => {
                                    log::info!("Scene loaded successfully!");
                                    
                                    // Special case for Celeste Demo HUD
                                    if name == "Celeste Demo" {
                                        let hud_path = project_path.join("assets/ui/celeste_hud.uiprefab");
                                        if hud_path.exists() {
                                            let hud_path_str = hud_path.to_string_lossy().to_string();
                                            match editor_state.ui_manager.load_prefab(&hud_path_str) {
                                                Ok(_) => {
                                                    let _ = editor_state.ui_manager.activate_prefab(&hud_path_str, "celeste_hud");
                                                    editor_state.console.info("🎮 Celeste HUD loaded and active".to_string());
                                                }
                                                Err(e) => log::error!("✗ Failed to load HUD prefab: {}", e),
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    launcher_state.error_message = Some(format!("Error loading scene: {}", e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        launcher_state.error_message = Some(format!("Error opening {}: {}", name, e));
                    }
                }
            } else {
                launcher_state.error_message = Some(format!("{} project not found. Tried: {:?}", name, possible_paths));
            }
        } else {
            // Create example project for other examples
            match launcher_state.project_manager.create_project(name, _desc) {
                Ok(metadata) => {
                    // Open editor for all examples
                    *app_state = AppState::Editor;
                    *editor_state = EditorState::new();
                    editor_state.set_project_path(metadata.path);
                }
                Err(e) => {
                    launcher_state.error_message = Some(format!("Error: {}", e));
                }
            }
        }
    }
}
