use ecs::{World, Entity, ComponentType, ComponentManager, ScriptParameter};
use egui;
use super::utils::{render_component_header, parse_lua_script_parameters};

pub fn render_script_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    project_path: &Option<std::path::PathBuf>,
    edit_script_request: &mut Option<String>,
) {
    // Script Component (Unity-style)
    let has_script = world.has_component(entity, ComponentType::Script);
    let mut remove_script = false;
    
    if has_script {
        let script_id = ui.make_persistent_id("script_component");
        let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), script_id, true
        );
        
        render_component_header(ui, "Script", "📜", false);
        
        if is_open.is_open() {
            ui.indent("script_indent", |ui| {
            if let Some(script) = world.scripts.get_mut(&entity) {
                // Get available scripts from project
                let mut available_scripts = Vec::new();
                if let Some(proj_path) = project_path {
                    let scripts_path = proj_path.join("scripts");
                    if let Ok(entries) = std::fs::read_dir(&scripts_path) {
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.ends_with(".lua") {
                                    // Remove .lua extension for display
                                    available_scripts.push(name.trim_end_matches(".lua").to_string());
                                }
                            }
                        }
                    }
                }

                egui::Grid::new("script_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Script");
                        if !available_scripts.is_empty() {
                            egui::ComboBox::from_id_source("script_picker")
                                .selected_text(&script.script_name)
                                .show_ui(ui, |ui| {
                                    for script_name in &available_scripts {
                                        ui.selectable_value(&mut script.script_name, script_name.clone(), script_name);
                                    }
                                });
                        } else {
                            ui.text_edit_singleline(&mut script.script_name);
                        }
                        ui.end_row();
                        
                        ui.label("Enabled");
                        ui.checkbox(&mut script.enabled, "");
                        ui.end_row();
                    });

                ui.add_space(10.0);

                // Parse script parameters from Lua file (Unity-like)
                if let Some(proj_path) = project_path {
                    let script_file = proj_path.join("scripts").join(format!("{}.lua", script.script_name));
                    if script_file.exists() {
                        let parsed_params = parse_lua_script_parameters(&script_file);

                        // Merge parsed parameters with existing ones (keep user-modified values)
                        for (key, default_value) in parsed_params {
                            script.parameters.entry(key).or_insert(default_value);
                        }

                        // Display all parameters (Unity-style)
                        if !script.parameters.is_empty() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.label(egui::RichText::new("Parameters").strong());
                            ui.add_space(5.0);

                            egui::Grid::new("script_params_grid")
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    let param_keys: Vec<String> = script.parameters.keys().cloned().collect();
                                    for key in param_keys {
                                        if let Some(value) = script.parameters.get_mut(&key) {
                                            ui.label(&key);

                                            match value {
                                                ScriptParameter::Float(f) => {
                                                    ui.add(egui::DragValue::new(f).speed(0.1));
                                                }
                                                ScriptParameter::Int(i) => {
                                                    ui.add(egui::DragValue::new(i).speed(1));
                                                }
                                                ScriptParameter::String(s) => {
                                                    ui.text_edit_singleline(s);
                                                }
                                                ScriptParameter::Bool(b) => {
                                                    ui.checkbox(b, "");
                                                }
                                                ScriptParameter::Entity(entity_opt) => {
                                                    // Entity dropdown (Unity-style GameObject reference)
                                                    let current_text = if let Some(e) = entity_opt {
                                                        if let Some(name) = world.names.get(e) {
                                                            format!("{} ({})", name, e)
                                                        } else {
                                                            format!("Entity {}", e)
                                                        }
                                                    } else {
                                                        "None".to_string()
                                                    };

                                                    egui::ComboBox::from_id_source(format!("entity_param_{}", key))
                                                        .selected_text(current_text)
                                                        .show_ui(ui, |ui| {
                                                            // None option
                                                            if ui.selectable_label(entity_opt.is_none(), "None").clicked() {
                                                                *entity_opt = None;
                                                            }

                                                            // List all entities
                                                            // Note: iter() on world.transforms might be suboptimal if too many entities,
                                                            // but it matches legacy behavior
                                                            for (e, _) in world.transforms.iter() {
                                                                let label = if let Some(name) = world.names.get(e) {
                                                                    format!("{} ({})", name, e)
                                                                } else {
                                                                    format!("Entity {}", e)
                                                                };
                                                                
                                                                let is_selected = entity_opt.map_or(false, |selected| selected == *e);
                                                                if ui.selectable_label(is_selected, label).clicked() {
                                                                    *entity_opt = Some(*e);
                                                                }
                                                            }
                                                        });
                                                }
                                            }
                                            ui.end_row();
                                        }
                                    }
                                });
                        }
                    }
                }

                ui.add_space(10.0);

                let script_name = script.script_name.clone();
                ui.horizontal(|ui| {
                    if ui.button("📝 Edit Script").clicked() {
                        *edit_script_request = Some(script_name);
                    }
                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                        // Component menu
                    }
                    if ui.button("❌ Remove Component").clicked() {
                        remove_script = true;
                    }
                });
            }
            });
            ui.add_space(10.0);
        }
    }
    
    if remove_script {
        let _ = world.remove_component(entity, ComponentType::Script);
    }
}
