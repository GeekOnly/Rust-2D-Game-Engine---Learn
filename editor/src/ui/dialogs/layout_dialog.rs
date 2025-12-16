use crate::states::EditorState;

pub struct LayoutDialog;

impl LayoutDialog {
    pub fn render(
        egui_ctx: &egui::Context,
        editor_state: &mut EditorState,
    ) {
        if !editor_state.show_save_layout_dialog {
            return;
        }

        egui::Window::new("Save Layout As")
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.label("Layout Name:");
                ui.text_edit_singleline(&mut editor_state.save_layout_name);
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && !editor_state.save_layout_name.is_empty() {
                        if let Some(ref project_path) = editor_state.current_project_path {
                            if let Err(e) = crate::ui::save_custom_layout_state(
                                &editor_state.save_layout_name,
                                &editor_state.dock_state,
                                project_path
                            ) {
                                editor_state.console.error(format!("Failed to save layout: {}", e));
                            } else {
                                let saved_name = editor_state.save_layout_name.clone();
                                editor_state.current_layout_name = saved_name.clone();
                                editor_state.current_layout_type = "custom".to_string();
                                editor_state.console.info(format!("Saved layout as '{}'", saved_name));
                                editor_state.show_save_layout_dialog = false;
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.show_save_layout_dialog = false;
                    }
                });
            });
    }
}
