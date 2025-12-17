use egui;
use std::path::PathBuf;
use crate::states::EditorState;
use std::sync::mpsc;
use std::thread;
use std::process::Command;
use std::fs;

pub struct ExportGameDialog;

impl ExportGameDialog {
    pub fn render(
        ctx: &egui::Context,
        editor_state: &mut EditorState,
    ) {
        if !editor_state.show_export_dialog {
            return;
        }

        // Create a local bool for the window state.
        // We initialize it to true since we check show_export_dialog above.
        // When the window is closed via X, egui sets this to false.
        let mut open = true;
        let mut should_start_build = false;
        let mut should_close = false;
        
        // We use a scope here to borrow editor_state inside, but not lock it for the 'open' check logic
        {
            let window = egui::Window::new("Export Game")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open);

            window.show(ctx, |ui| {
                ui.heading("Build Settings");
                ui.separator();
                
                // Borrow ONLY what is needed
                let params = &mut editor_state.build_params;
                
                // Game Name
                ui.horizontal(|ui| {
                    ui.label("Game Name:");
                    ui.text_edit_singleline(&mut params.game_name);
                });
                
                // Output Path
                ui.horizontal(|ui| {
                    ui.label("Output Folder:");
                    // Truncate path if too long for display
                    let path_str = params.output_path.to_string_lossy();
                    let display_path = if path_str.len() > 30 {
                        format!("...{}", &path_str[path_str.len()-27..])
                    } else {
                        path_str.to_string()
                    };
                    ui.label(display_path);
                    
                    if !params.is_building {
                         if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                params.output_path = path;
                            }
                        }
                    }
                });

                // Target Platform (Display only for now)
                ui.horizontal(|ui| {
                    ui.label("Target Platform:");
                    ui.label(&params.target_platform);
                });
                
                ui.separator();
                
                if params.is_building {
                    ui.horizontal(|ui| {
                        ui.label("Building...");
                        ui.spinner();
                    });
                    
                    ui.collapsing("Build Log", |ui| {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.label(&params.build_output);
                        });
                    });
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("Build").clicked() {
                             should_start_build = true;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                }
                
                if let Some(error) = &params.build_error {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                }
            });
        }
        
        // Now updates to editor_state based on what happened
        if !open || should_close {
            editor_state.show_export_dialog = false;
        }
        
        if should_start_build {
             // We need to clone the path because start_build_process takes other mutable borrows
             // Wait, start_build_process takes &mut editor_state, so we can't have borrowed it above.
             // But the borrow of 'params' ended with the scope block.
             let project_path = editor_state.current_project_path.clone().unwrap_or(std::env::current_dir().unwrap_or_default());
             start_build_process(editor_state, project_path);
        }
    }
}

fn start_build_process(editor_state: &mut EditorState, project_path: PathBuf) {
    editor_state.build_params.is_building = true;
    editor_state.build_params.build_output.clear();
    editor_state.build_params.build_error = None;
    
    let (tx, rx) = mpsc::channel();
    editor_state.build_receiver = Some(rx);
    
    let game_name = editor_state.build_params.game_name.clone();
    let output_path = editor_state.build_params.output_path.clone();
    
    // We assume the editor is running from the engine root, so we use current_dir for building
    let engine_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    thread::spawn(move || {
        let _ = tx.send(format!("Starting build for {}...", game_name));
        let _ = tx.send(format!("Engine path: {:?}", engine_path));
        let _ = tx.send(format!("Project path: {:?}", project_path));
        let _ = tx.send(format!("Output directory: {:?}", output_path));

        // 1. Compile player binary
        let _ = tx.send("Compiling player binary (this may take a while)...".to_string());
        
        // Ensure Cargo.toml exists
        if !engine_path.join("Cargo.toml").exists() {
             let _ = tx.send("WARNING: Cargo.toml not found in engine path. Build may fail.".to_string());
        }
        
        let output = Command::new("cargo")
            .current_dir(&engine_path)
            .args(&["build", "--release", "--bin", "player"])
            .output();
            
        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let _ = tx.send(format!("ERROR: Compilation failed:\n{}", stderr));
                    return;
                } else {
                    let _ = tx.send("Compilation successful!".to_string());
                }
            }
            Err(e) => {
                let _ = tx.send(format!("ERROR: Failed to run cargo: {}", e));
                return;
            }
        }
        
        // 2. Create output directory
        if !output_path.exists() {
            if let Err(e) = fs::create_dir_all(&output_path) {
                 let _ = tx.send(format!("ERROR: Failed to create output directory: {}", e));
                 return;
            }
        }
        
        // 3. Copy executable
        let exe_name = if cfg!(target_os = "windows") { "player.exe" } else { "player" };

        // Find workspace root - try engine_path/../target/release first
        // If engine is in workspace/engine, workspace root is parent of engine_path
        let workspace_target = if engine_path.file_name().and_then(|s| s.to_str()) == Some("engine") {
            // We're in the engine subdirectory, go up one level
            engine_path.parent().unwrap_or(&engine_path).join("target/release")
        } else {
            // We're already at workspace root
            engine_path.join("target/release")
        };

        let target_exe = workspace_target.join(exe_name);
        
        let dest_exe = output_path.join(format!("{}{}", game_name, if cfg!(target_os = "windows") { ".exe" } else { "" }));
        
        let _ = tx.send(format!("Copying executable from {:?} to {:?}...", target_exe, dest_exe));
        if let Err(e) = fs::copy(&target_exe, &dest_exe) {
             let _ = tx.send(format!("ERROR: Failed to copy executable: {}", e));
             return;
        }

        // 4. Copy assets folder (from project_path)
        let assets_src = project_path.join("assets");
        let assets_dest = output_path.join("assets");
        if assets_src.exists() {
            let _ = tx.send("Copying assets...".to_string());
             if let Err(e) = copy_dir_recursive(&assets_src, &assets_dest) {
                 let _ = tx.send(format!("ERROR: Failed to copy assets: {}", e));
                 return;
             }
        } else {
            let _ = tx.send("WARNING: No assets folder found in project path.".to_string());
        }

       // 5. Copy scenes folder (from project_path)
        let scenes_src = project_path.join("scenes");
        let scenes_dest = output_path.join("scenes");
        if scenes_src.exists() {
            let _ = tx.send("Copying scenes...".to_string());
             if let Err(e) = copy_dir_recursive(&scenes_src, &scenes_dest) {
                 let _ = tx.send(format!("ERROR: Failed to copy scenes: {}", e));
                 return;
             }
        } else {
            let _ = tx.send("WARNING: No scenes folder found in project path.".to_string());
        }
        
       // 6. Copy scripts folder (from project_path)
        let scripts_src = project_path.join("scripts");
        let scripts_dest = output_path.join("scripts");
        if scripts_src.exists() {
            let _ = tx.send("Copying scripts...".to_string());
             if let Err(e) = copy_dir_recursive(&scripts_src, &scripts_dest) {
                 let _ = tx.send(format!("ERROR: Failed to copy scripts: {}", e));
                 return;
             }
        }
        
        let _ = tx.send("Build completed successfully!".to_string());
        let _ = tx.send("SUCCESS".to_string());
    });
}
// Helper function to copy directories
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
