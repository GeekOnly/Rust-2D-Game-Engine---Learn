use ecs::{World, Entity, EntityTag, Script, ScriptParameter};
use egui;
use std::collections::HashMap;

/// Renders the Inspector panel showing entity properties and components
pub fn render_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    entity_names: &mut HashMap<Entity, String>,
    edit_script_request: &mut Option<String>,
    project_path: &Option<std::path::PathBuf>,
) {
    // Unity-style header
    ui.horizontal(|ui| {
        ui.heading("Inspector");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Options menu
            ui.menu_button("⋮", |ui| {
                if ui.button("Reset").clicked() {
                    ui.close_menu();
                }
                if ui.button("Copy Component").clicked() {
                    ui.close_menu();
                }
                if ui.button("Paste Component Values").clicked() {
                    ui.close_menu();
                }
            });
        });
    });
    ui.separator();

        if let Some(entity) = *selected_entity {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ===== Unity-style Inspector Header (Gray bar) =====
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(60, 60, 60))
                    .inner_margin(egui::Margin::same(5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Active checkbox
                            let mut is_active = world.active.get(&entity).copied().unwrap_or(true);
                            if ui.checkbox(&mut is_active, "").changed() {
                                world.active.insert(entity, is_active);
                            }

                            // GameObject icon (cube)
                            ui.label("🎲");

                            // Entity name
                            if let Some(name) = entity_names.get_mut(&entity) {
                                ui.add(egui::TextEdit::singleline(name)
                                    .desired_width(120.0)
                                    .frame(false));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Static dropdown (Unity has this)
                                egui::ComboBox::from_id_source("static_dropdown")
                                    .selected_text("Static")
                                    .width(60.0)
                                    .show_ui(ui, |ui| {
                                        ui.label("Nothing");
                                        ui.label("Everything");
                                    });
                            });
                        });
                    });

                ui.add_space(5.0);

                // Tag and Layer in same row (Unity layout)
                ui.horizontal(|ui| {
                    ui.label("Tag");

                    let current_tag = world.tags.get(&entity).cloned();
                    let tag_text = match &current_tag {
                        Some(EntityTag::Player) => "Player",
                        Some(EntityTag::Item) => "Item",
                        None => "Untagged",
                    };

                    let mut new_tag = current_tag.clone();
                    let mut tag_changed = false;

                    egui::ComboBox::from_id_source("tag_dropdown")
                        .selected_text(tag_text)
                        .width(70.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(current_tag.is_none(), "Untagged").clicked() {
                                new_tag = None;
                                tag_changed = true;
                            }
                            if ui.selectable_label(matches!(current_tag, Some(EntityTag::Player)), "Player").clicked() {
                                new_tag = Some(EntityTag::Player);
                                tag_changed = true;
                            }
                            if ui.selectable_label(matches!(current_tag, Some(EntityTag::Item)), "Item").clicked() {
                                new_tag = Some(EntityTag::Item);
                                tag_changed = true;
                            }
                        });

                    if tag_changed {
                        if let Some(tag) = new_tag {
                            world.tags.insert(entity, tag);
                        } else {
                            world.tags.remove(&entity);
                        }
                    }

                    ui.add_space(10.0);
                    ui.label("Layer");

                    let current_layer = world.layers.get(&entity).copied().unwrap_or(0);
                    let layer_names = ["Default", "TransparentFX", "Ignore Raycast", "Water", "UI"];
                    let layer_text = if (current_layer as usize) < layer_names.len() {
                        layer_names[current_layer as usize]
                    } else {
                        "Custom"
                    };

                    egui::ComboBox::from_id_source("layer_dropdown")
                        .selected_text(layer_text)
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for (idx, &name) in layer_names.iter().enumerate() {
                                if ui.selectable_label(current_layer == idx as u8, name).clicked() {
                                    world.layers.insert(entity, idx as u8);
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                // Transform Component (Unity-style: always visible)
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    render_component_header(ui, "Transform", "⚙️", true);

                    // Transform fields - Unity style (X Y Z in same row)
                    egui::Grid::new("transform_grid")
                        .num_columns(7)
                        .spacing([5.0, 8.0])
                        .show(ui, |ui| {
                            // Position row
                            ui.label("Position");
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut transform.position[0])
                                    .speed(0.1)
                                    .max_decimals(2)
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut transform.position[1])
                                    .speed(0.1)
                                    .max_decimals(2)
                            );
                            ui.label("Z");
                            ui.add(
                                egui::DragValue::new(&mut transform.position[2])
                                    .speed(0.1)
                                    .max_decimals(2)
                            );
                            ui.end_row();

                            // Rotation row
                            ui.label("Rotation");
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut transform.rotation[0])
                                    .speed(0.5)
                                    .max_decimals(2)
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut transform.rotation[1])
                                    .speed(0.5)
                                    .max_decimals(2)
                            );
                            ui.label("Z");
                            ui.add(
                                egui::DragValue::new(&mut transform.rotation[2])
                                    .speed(0.5)
                                    .max_decimals(2)
                            );
                            ui.end_row();

                            // Scale row
                            ui.label("Scale");
                            ui.label("X");
                            ui.add(
                                egui::DragValue::new(&mut transform.scale[0])
                                    .speed(0.01)
                                    .max_decimals(2)
                            );
                            ui.label("Y");
                            ui.add(
                                egui::DragValue::new(&mut transform.scale[1])
                                    .speed(0.01)
                                    .max_decimals(2)
                            );
                            ui.label("Z");
                            ui.add(
                                egui::DragValue::new(&mut transform.scale[2])
                                    .speed(0.01)
                                    .max_decimals(2)
                            );
                            ui.end_row();
                        });

                    ui.add_space(10.0);
                }

                // Sprite Component (Unity-style collapsible)
                let has_sprite = world.sprites.contains_key(&entity);
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
                                        ui.label("Sprite");
                                        ui.text_edit_singleline(&mut sprite.texture_id);
                                        ui.end_row();
                                        
                                        ui.label("Color");
                                        ui.color_edit_button_rgba_unmultiplied(&mut sprite.color);
                                        ui.end_row();
                                        
                                        ui.label("Width");
                                        ui.add(egui::DragValue::new(&mut sprite.width).speed(1.0));
                                        ui.end_row();
                                        
                                        ui.label("Height");
                                        ui.add(egui::DragValue::new(&mut sprite.height).speed(1.0));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
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
                    world.sprites.remove(&entity);
                }

                // Collider Component (Unity-style)
                let has_collider = world.colliders.contains_key(&entity);
                let mut remove_collider = false;
                
                if has_collider {
                    let collider_id = ui.make_persistent_id("collider_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), collider_id, true
                    );
                    
                    render_component_header(ui, "Box Collider 2D", "📦", false);
                    
                    if is_open.is_open() {
                        if let Some(collider) = world.colliders.get_mut(&entity) {
                            ui.indent("collider_indent", |ui| {
                                egui::Grid::new("collider_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Width");
                                        ui.add(egui::DragValue::new(&mut collider.width).speed(1.0));
                                        ui.end_row();
                                        
                                        ui.label("Height");
                                        ui.add(egui::DragValue::new(&mut collider.height).speed(1.0));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_collider = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_collider {
                    world.colliders.remove(&entity);
                }

                // Velocity Component (Rigidbody) - Unity-style
                let has_velocity = world.velocities.contains_key(&entity);
                let mut remove_velocity = false;
                
                if has_velocity {
                    let velocity_id = ui.make_persistent_id("velocity_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), velocity_id, true
                    );
                    
                    render_component_header(ui, "Rigidbody 2D", "⚡", false);
                    
                    if is_open.is_open() {
                        if let Some(velocity) = world.velocities.get_mut(&entity) {
                            ui.indent("velocity_indent", |ui| {
                                egui::Grid::new("velocity_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Velocity X");
                                        ui.add(egui::DragValue::new(&mut velocity.0).speed(0.1));
                                        ui.end_row();
                                        
                                        ui.label("Velocity Y");
                                        ui.add(egui::DragValue::new(&mut velocity.1).speed(0.1));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_velocity = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_velocity {
                    world.velocities.remove(&entity);
                }

                // Mesh Component (3D) - Unity-style
                let has_mesh = world.meshes.contains_key(&entity);
                let mut remove_mesh = false;
                
                if has_mesh {
                    let mesh_id = ui.make_persistent_id("mesh_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), mesh_id, true
                    );
                    
                    render_component_header(ui, "Mesh Renderer", "🧊", false);
                    
                    if is_open.is_open() {
                        if let Some(mesh) = world.meshes.get_mut(&entity) {
                            ui.indent("mesh_indent", |ui| {
                                egui::Grid::new("mesh_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Mesh Type");
                                        egui::ComboBox::from_id_source("mesh_type_picker")
                                            .selected_text(format!("{:?}", mesh.mesh_type))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cube, "Cube");
                                                ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Sphere, "Sphere");
                                                ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Cylinder, "Cylinder");
                                                ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Plane, "Plane");
                                                ui.selectable_value(&mut mesh.mesh_type, ecs::MeshType::Capsule, "Capsule");
                                            });
                                        ui.end_row();
                                        
                                        ui.label("Color");
                                        let mut color = egui::Color32::from_rgba_unmultiplied(
                                            (mesh.color[0] * 255.0) as u8,
                                            (mesh.color[1] * 255.0) as u8,
                                            (mesh.color[2] * 255.0) as u8,
                                            (mesh.color[3] * 255.0) as u8,
                                        );
                                        if ui.color_edit_button_srgba(&mut color).changed() {
                                            mesh.color[0] = color.r() as f32 / 255.0;
                                            mesh.color[1] = color.g() as f32 / 255.0;
                                            mesh.color[2] = color.b() as f32 / 255.0;
                                            mesh.color[3] = color.a() as f32 / 255.0;
                                        }
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_mesh = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_mesh {
                    world.meshes.remove(&entity);
                }

                // Script Component (Unity-style)
                let has_script = world.scripts.contains_key(&entity);
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
                    world.scripts.remove(&entity);
                }

                ui.add_space(15.0);

                // ===== Add Component Button (Unity-style) =====
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 70.0);
                    ui.menu_button("➕ Add Component", |ui| {
                        ui.label("🎨 Rendering");
                        ui.separator();

                        // Add Sprite Renderer
                        if !world.sprites.contains_key(&entity) {
                            if ui.button("Sprite Renderer").clicked() {
                                world.sprites.insert(entity, ecs::Sprite {
                                    texture_id: "sprite".to_string(),
                                    width: 32.0,
                                    height: 32.0,
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    billboard: false, // Default sprite, not billboard
                                });
                                ui.close_menu();
                            }
                        }

                        ui.add_space(5.0);
                        ui.label("⚙️ Physics");
                        ui.separator();

                        // Add Collider
                        if !world.colliders.contains_key(&entity) {
                            if ui.button("Box Collider 2D").clicked() {
                                world.colliders.insert(entity, ecs::Collider {
                                    width: 32.0,
                                    height: 32.0,
                                });
                                ui.close_menu();
                            }
                        }

                        // Add Velocity
                        if !world.velocities.contains_key(&entity) {
                            if ui.button("Rigidbody 2D").clicked() {
                                world.velocities.insert(entity, (0.0, 0.0));
                                ui.close_menu();
                            }
                        }

                        ui.add_space(5.0);
                        ui.label("📜 Scripting");
                        ui.separator();

                        // Add Script
                        if !world.scripts.contains_key(&entity) {
                            if ui.button("Script").clicked() {
                                // Check for available scripts
                                let mut available_scripts = Vec::new();
                                if let Some(proj_path) = project_path {
                                    let scripts_path = proj_path.join("scripts");
                                    if let Ok(entries) = std::fs::read_dir(&scripts_path) {
                                        for entry in entries.flatten() {
                                            if let Some(name) = entry.file_name().to_str() {
                                                if name.ends_with(".lua") {
                                                    available_scripts.push(name.trim_end_matches(".lua").to_string());
                                                }
                                            }
                                        }
                                    }
                                }

                                // Use first available script or create new one
                                let script_name = if !available_scripts.is_empty() {
                                    available_scripts[0].clone()
                                } else {
                                    format!("Script_{}", entity)
                                };

                                world.scripts.insert(entity, Script {
                                    script_name: script_name.clone(),
                                    enabled: true,
                                    parameters: HashMap::new(),
                                });
                                ui.close_menu();
                            }
                        }
                    });
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                // Delete GameObject button (centered, Unity-style)
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 80.0);
                    if ui.button("🗑 Delete GameObject").clicked() {
                        world.despawn(entity);
                        entity_names.remove(&entity);
                        *selected_entity = None;
                    }
                });
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("Select an object to inspect").color(egui::Color32::GRAY));
            });
        }
}

/// Render Unity-style component header
fn render_component_header(ui: &mut egui::Ui, name: &str, icon: &str, always_open: bool) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(56, 56, 56))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if !always_open {
                    ui.label("▼");
                }
                ui.label(icon);
                ui.strong(name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("⋮").on_hover_text("Component Options").clicked() {
                        // Component menu
                    }
                });
            });
        });
    ui.add_space(8.0);
}

/// Parse Lua script file to extract variable declarations (Unity-like parameters)
/// Looks for patterns like: `local speed = 10`, `jumpForce = 5.0`, `name = "Player"`
pub fn parse_lua_script_parameters(script_path: &std::path::Path) -> HashMap<String, ScriptParameter> {
    let mut parameters = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(script_path) {
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("--") {
                continue;
            }

            // Match patterns: "local name = value" or "name = value"
            if let Some(equals_pos) = trimmed.find('=') {
                let var_part = &trimmed[..equals_pos].trim();
                let value_part = trimmed[equals_pos + 1..].trim();

                // Remove "local" keyword if present
                let var_name = var_part
                    .strip_prefix("local")
                    .unwrap_or(var_part)
                    .trim()
                    .to_string();

                // Skip if variable name is empty or contains spaces (not a simple variable)
                if var_name.is_empty() || var_name.contains(' ') {
                    continue;
                }

                // Parse value type
                let param = if value_part.starts_with('"') || value_part.starts_with('\'') {
                    // String value
                    let str_value = value_part
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_end_matches(',')
                        .to_string();
                    Some(ScriptParameter::String(str_value))
                } else if value_part == "true" || value_part == "false" {
                    // Boolean value
                    let bool_value = value_part == "true";
                    Some(ScriptParameter::Bool(bool_value))
                } else if let Ok(float_value) = value_part.trim_end_matches(',').parse::<f32>() {
                    // Try parsing as float first
                    if value_part.contains('.') {
                        Some(ScriptParameter::Float(float_value))
                    } else {
                        // Integer (no decimal point)
                        Some(ScriptParameter::Int(float_value as i32))
                    }
                } else {
                    None
                };

                if let Some(p) = param {
                    parameters.insert(var_name, p);
                }
            }
        }
    }

    parameters
}
