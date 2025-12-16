use crate::states::EditorState;

pub struct ExitDialog;

impl ExitDialog {
    pub fn render(
        egui_ctx: &egui::Context,
        editor_state: &mut EditorState,
    ) {
        if !editor_state.show_exit_dialog {
            return;
        }

        egui::Window::new("Exit Editor")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(egui_ctx, |ui| {
                if editor_state.scene_modified {
                    ui.label("You have unsaved changes. Do you want to save before exiting?");
                } else {
                    ui.label("Are you sure you want to exit?");
                }
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if editor_state.scene_modified {
                        if ui.button("Save and Exit").clicked() {
                            // Save scene
                            let mut saved = false;
                            if let Some(ref path) = editor_state.current_scene_path.clone() {
                                if let Err(e) = editor_state.save_scene(path) {
                                    editor_state.console.error(format!("Failed to save: {}", e));
                                } else {
                                    saved = true;
                                }
                            }
                            
                            if saved {
                                editor_state.should_exit = true;
                                editor_state.show_exit_dialog = false;
                            }
                        }
                        
                        if ui.button("Exit Without Saving").clicked() {
                            editor_state.should_exit = true;
                            editor_state.show_exit_dialog = false;
                        }
                    } else {
                        if ui.button("Exit").clicked() {
                            editor_state.should_exit = true;
                            editor_state.show_exit_dialog = false;
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        editor_state.show_exit_dialog = false;
                    }
                });
            });
    }
}
