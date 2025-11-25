use ecs::{World, Entity, Sprite, Collider, EntityTag, Script, Prefab, ScriptParameter};
use egui;
use std::collections::HashMap;
use crate::console::Console;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformTool {
    View,   // Q - No gizmo, just view
    Move,   // W - Move gizmo
    Rotate, // E - Rotation gizmo
    Scale,  // R - Scale gizmo
}

pub struct EditorUI;

impl EditorUI {
    pub fn render_editor(
        ctx: &egui::Context,
        world: &mut World,
        selected_entity: &mut Option<Entity>,
        entity_names: &mut HashMap<Entity, String>,
        save_request: &mut bool,
        save_as_request: &mut bool,
        load_request: &mut bool,
        load_file_request: &mut Option<std::path::PathBuf>,
        new_scene_request: &mut bool,
        play_request: &mut bool,
        stop_request: &mut bool,
        edit_script_request: &mut Option<String>,
        project_path: &Option<std::path::PathBuf>,
        current_scene_path: &Option<std::path::PathBuf>,
        scene_view_tab: &mut usize,
        is_playing: bool,
        show_colliders: &mut bool,
        show_velocities: &mut bool,
        console: &mut Console,
        bottom_panel_tab: &mut usize,
        current_tool: &TransformTool,
        show_project_settings: &mut bool,
    ) {
        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Scene").clicked() {
                        *new_scene_request = true;
                    }
                    ui.separator();
                    if ui.button("Save Scene").clicked() {
                        *save_request = true;
                    }
                    if ui.button("Save Scene As...").clicked() {
                        *save_as_request = true;
                    }
                    ui.separator();
                    if ui.button("Load Scene...").clicked() {
                        *load_request = true;
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
                    let scene_files = Self::get_scene_files(proj_path);

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

                // Play/Stop buttons in menu bar
                if !is_playing {
                    if ui.button("▶ Play").clicked() {
                        *play_request = true;
                    }
                } else {
                    if ui.button("⏹ Stop").clicked() {
                        *stop_request = true;
                    }
                }
            });
        });

        // Hierarchy Panel (Left)
        egui::SidePanel::left("hierarchy").min_width(200.0).show(ctx, |ui| {
            ui.heading("📋 Hierarchy");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Scene root node
                let scene_name = if let Some(path) = current_scene_path {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_string()
                } else {
                    "Untitled Scene".to_string()
                };

                // Scene root is always expanded (collapsing API with default_open)
                egui::CollapsingHeader::new(format!("📁 {}", scene_name))
                    .default_open(true)
                    .show(ui, |ui| {
                        let entities: Vec<Entity> = entity_names.keys().cloned().collect();

                        // Track entity to delete (for right-click menu)
                        let mut entity_to_delete: Option<Entity> = None;

                        for entity in entities {
                            let name = entity_names.get(&entity).cloned().unwrap_or(format!("Entity {}", entity));
                            let is_selected = *selected_entity == Some(entity);

                            // Get icon based on entity type
                            let icon = Self::get_entity_icon(world, entity);

                            ui.horizontal(|ui| {
                                // Selectable label with icon
                                let response = ui.selectable_label(is_selected, format!("{} {}", icon, name));

                                if response.clicked() {
                                    *selected_entity = Some(entity);
                                }

                                // Right-click context menu
                                response.context_menu(|ui| {
                                    ui.label(format!("📝 {}", name));
                                    ui.separator();

                                    if ui.button("🔄 Duplicate").clicked() {
                                        // TODO: Implement duplicate
                                        ui.close_menu();
                                    }

                                    if ui.button("📋 Rename").clicked() {
                                        // Already editable in Inspector
                                        ui.close_menu();
                                    }

                                    ui.separator();

                                    if ui.button("❌ Delete").clicked() {
                                        entity_to_delete = Some(entity);
                                        ui.close_menu();
                                    }
                                });
                            });
                        }

                        // Delete entity if requested
                        if let Some(entity) = entity_to_delete {
                            world.despawn(entity);
                            entity_names.remove(&entity);
                            if *selected_entity == Some(entity) {
                                *selected_entity = None;
                            }
                        }
                    });
            });

            ui.separator();

            // Scenes section (if project is open)
            if let Some(proj_path) = project_path {
                ui.heading("📁 Scenes");
                ui.separator();

                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    let scene_files = Self::get_scene_files(proj_path);

                    if scene_files.is_empty() {
                        ui.label("No scenes found");
                        ui.label("Create a scene with File → Save Scene");
                    } else {
                        for scene_file in scene_files {
                            // Check if this is the current scene
                            let is_current = if let Some(current) = current_scene_path {
                                if let Ok(relative) = current.strip_prefix(proj_path) {
                                    relative.to_string_lossy() == scene_file
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            let scene_name = std::path::Path::new(&scene_file)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&scene_file);

                            ui.horizontal(|ui| {
                                let label = if is_current {
                                    format!("▶ {}", scene_name)
                                } else {
                                    format!("  {}", scene_name)
                                };

                                let response = ui.selectable_label(is_current, label);

                                if response.clicked() && !is_current {
                                    // Load this scene
                                    let scene_path = proj_path.join(&scene_file);
                                    *load_file_request = Some(scene_path);
                                }

                                // Right-click context menu
                                response.context_menu(|ui| {
                                    ui.label(format!("📝 {}", scene_name));
                                    ui.separator();

                                    if ui.button("📂 Open").clicked() {
                                        let scene_path = proj_path.join(&scene_file);
                                        *load_file_request = Some(scene_path);
                                        ui.close_menu();
                                    }

                                    if ui.button("🗑 Delete Scene").clicked() {
                                        let scene_path = proj_path.join(&scene_file);
                                        if let Err(e) = std::fs::remove_file(&scene_path) {
                                            console.error(format!("Failed to delete scene: {}", e));
                                        } else {
                                            console.info(format!("Deleted scene: {}", scene_name));
                                        }
                                        ui.close_menu();
                                    }
                                });
                            });
                        }
                    }
                });

                ui.separator();
            }

            // Create menu button with dropdown
            ui.menu_button("➕ Create", |ui| {
                ui.label("🎮 2D Objects");
                ui.separator();

                if ui.button("📦 Empty GameObject").clicked() {
                    // Create GameObject with only Transform (Unity behavior)
                    let entity = world.spawn();
                    world.transforms.insert(entity, ecs::Transform::default());
                    entity_names.insert(entity, "GameObject".to_string());
                    *selected_entity = Some(entity);
                    ui.close_menu();
                }

                if ui.button("🎮 Sprite").clicked() {
                    let entity = world.spawn();
                    world.transforms.insert(entity, ecs::Transform::default());
                    world.sprites.insert(entity, ecs::Sprite {
                        texture_id: "sprite".to_string(),
                        width: 32.0,
                        height: 32.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                    });
                    entity_names.insert(entity, "Sprite".to_string());
                    *selected_entity = Some(entity);
                    ui.close_menu();
                }

                if ui.button("📷 Camera").clicked() {
                    let entity = Prefab::new("Camera").spawn(world);
                    entity_names.insert(entity, "Camera".to_string());
                    *selected_entity = Some(entity);
                    ui.close_menu();
                }
            });
        });

        // Inspector Panel (Right)
        egui::SidePanel::right("inspector").min_width(300.0).show(ctx, |ui| {
            ui.heading("🔧 Inspector");
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

                    // Transform Component (Unity-like: always visible, no collapsing)
                    if let Some(transform) = world.transforms.get_mut(&entity) {
                        // Transform header (gray bar like Unity)
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(56, 56, 56))
                            .inner_margin(egui::Margin::same(5.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("⚙️");
                                    ui.strong("Transform");

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Unity has gear icon and three-dot menu here
                                        ui.label("⚙️");
                                    });
                                });
                            });

                        // Transform fields (always visible)
                        ui.add_space(5.0);

                        // Position (grouped in one row)
                        ui.horizontal(|ui| {
                            ui.label("Position");
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut transform.position[0]).speed(1.0).max_decimals(2));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut transform.position[1]).speed(1.0).max_decimals(2));
                            ui.label("Z");
                            ui.add(egui::DragValue::new(&mut transform.position[2]).speed(1.0).max_decimals(2));
                        });

                        // Rotation (grouped in one row)
                        ui.horizontal(|ui| {
                            ui.label("Rotation");
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut transform.rotation[0]).speed(0.1).max_decimals(2));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut transform.rotation[1]).speed(0.1).max_decimals(2));
                            ui.label("Z");
                            ui.add(egui::DragValue::new(&mut transform.rotation[2]).speed(0.1).max_decimals(2));
                        });

                        // Scale (grouped in one row)
                        ui.horizontal(|ui| {
                            ui.label("Scale    ");
                            ui.label("X");
                            ui.add(egui::DragValue::new(&mut transform.scale[0]).speed(0.1).max_decimals(2));
                            ui.label("Y");
                            ui.add(egui::DragValue::new(&mut transform.scale[1]).speed(0.1).max_decimals(2));
                            ui.label("Z");
                            ui.add(egui::DragValue::new(&mut transform.scale[2]).speed(0.1).max_decimals(2));
                        });

                        ui.add_space(10.0);
                    }

                    // Sprite Component (only show if has sprite)
                    let has_sprite = world.sprites.contains_key(&entity);
                    if has_sprite {
                        ui.collapsing("Sprite Renderer", |ui| {
                            if let Some(sprite) = world.sprites.get_mut(&entity) {
                                ui.text_edit_singleline(&mut sprite.texture_id);
                                ui.horizontal(|ui| {
                                    ui.label("Width:");
                                    ui.add(egui::DragValue::new(&mut sprite.width).speed(1.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Height:");
                                    ui.add(egui::DragValue::new(&mut sprite.height).speed(1.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    ui.color_edit_button_rgba_unmultiplied(&mut sprite.color);
                                });

                                if ui.button("Remove Component").clicked() {
                                    world.sprites.remove(&entity);
                                }
                            }
                        });
                    }

                    // Collider Component (only show if has collider)
                    let has_collider = world.colliders.contains_key(&entity);
                    if has_collider {
                        ui.collapsing("Box Collider", |ui| {
                            if let Some(collider) = world.colliders.get_mut(&entity) {
                                ui.horizontal(|ui| {
                                    ui.label("Width:");
                                    ui.add(egui::DragValue::new(&mut collider.width).speed(1.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Height:");
                                    ui.add(egui::DragValue::new(&mut collider.height).speed(1.0));
                                });

                                if ui.button("Remove Component").clicked() {
                                    world.colliders.remove(&entity);
                                }
                            }
                        });
                    }

                    // Velocity Component (Rigidbody) - only show if has velocity
                    let has_velocity = world.velocities.contains_key(&entity);
                    if has_velocity {
                        ui.collapsing("Rigidbody 2D", |ui| {
                            if let Some(velocity) = world.velocities.get_mut(&entity) {
                                ui.horizontal(|ui| {
                                    ui.label("Velocity X:");
                                    ui.add(egui::DragValue::new(&mut velocity.0).speed(1.0));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Velocity Y:");
                                    ui.add(egui::DragValue::new(&mut velocity.1).speed(1.0));
                                });

                                if ui.button("Remove Component").clicked() {
                                    world.velocities.remove(&entity);
                                }
                            }
                        });
                    }


                    // Script Component
                    let has_script = world.scripts.contains_key(&entity);
                    let mut remove_script = false;
                    ui.collapsing("Script", |ui| {
                        if has_script {
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

                                ui.horizontal(|ui| {
                                    ui.label("Script:");
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
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Enabled:");
                                    ui.checkbox(&mut script.enabled, "");
                                });

                                ui.add_space(10.0);

                                // Parse script parameters from Lua file (Unity-like)
                                if let Some(proj_path) = project_path {
                                    let script_file = proj_path.join("scripts").join(format!("{}.lua", script.script_name));
                                    if script_file.exists() {
                                        let parsed_params = Self::parse_lua_script_parameters(&script_file);

                                        // Merge parsed parameters with existing ones (keep user-modified values)
                                        for (key, default_value) in parsed_params {
                                            script.parameters.entry(key).or_insert(default_value);
                                        }

                                        // Display all parameters
                                        if !script.parameters.is_empty() {
                                            ui.separator();
                                            ui.label(egui::RichText::new("Script Parameters:").strong());

                                            let param_keys: Vec<String> = script.parameters.keys().cloned().collect();
                                            for key in param_keys {
                                                if let Some(value) = script.parameters.get_mut(&key) {
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("{}:", key));

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
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                ui.add_space(10.0);

                                let script_name = script.script_name.clone();
                                ui.horizontal(|ui| {
                                    if ui.button("📝 Edit Script").clicked() {
                                        *edit_script_request = Some(script_name);
                                    }
                                    if ui.button("Remove Component").clicked() {
                                        remove_script = true;
                                    }
                                });
                            }
                        } else {
                            if ui.button("Add Script").clicked() {
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

                                // Create file if it doesn't exist
                                if available_scripts.is_empty() {
                                    *edit_script_request = Some(script_name);
                                }
                            }
                        }
                    });

                    if remove_script {
                        world.scripts.remove(&entity);
                    }

                    ui.add_space(20.0);

                    // ===== Add Component Button (Unity-like) =====
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

                    ui.add_space(10.0);

                    if ui.button("🗑 Delete GameObject").clicked() {
                        world.despawn(entity);
                        entity_names.remove(&entity);
                        *selected_entity = None;
                    }
                });
            } else {
                ui.label("Select an entity to edit");
            }
        });

        // Center Panel - Scene/Game View
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(scene_view_tab, 0, "🎬 Scene");
                ui.selectable_value(scene_view_tab, 1, "🎮 Game");
            });
            ui.separator();

            match *scene_view_tab {
                0 => {
                    // Scene View - Visual editor
                    let (response, painter) = ui.allocate_painter(
                        ui.available_size(),
                        egui::Sense::click_and_drag(),
                    );
                    let rect = response.rect;

                    // Draw grid background
                    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(40, 40, 50));

                    // Draw grid lines
                    let grid_size = 50.0;
                    for i in 0..((rect.width() / grid_size) as usize) {
                        let x = rect.min.x + i as f32 * grid_size;
                        painter.line_segment(
                            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
                        );
                    }
                    for i in 0..((rect.height() / grid_size) as usize) {
                        let y = rect.min.y + i as f32 * grid_size;
                        painter.line_segment(
                            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
                        );
                    }

                    // Draw entities
                    let center_x = rect.center().x;
                    let center_y = rect.center().y;

                    for (&entity, transform) in &world.transforms {
                        let screen_x = center_x + transform.x();
                        let screen_y = center_y + transform.y();

                        if let Some(sprite) = world.sprites.get(&entity) {
                            let size = egui::vec2(sprite.width, sprite.height);
                            let color = egui::Color32::from_rgba_unmultiplied(
                                (sprite.color[0] * 255.0) as u8,
                                (sprite.color[1] * 255.0) as u8,
                                (sprite.color[2] * 255.0) as u8,
                                (sprite.color[3] * 255.0) as u8,
                            );

                            painter.rect_filled(
                                egui::Rect::from_center_size(
                                    egui::pos2(screen_x, screen_y),
                                    size,
                                ),
                                2.0,
                                color,
                            );

                            // Draw selection outline
                            if *selected_entity == Some(entity) {
                                painter.rect_stroke(
                                    egui::Rect::from_center_size(
                                        egui::pos2(screen_x, screen_y),
                                        size + egui::vec2(4.0, 4.0),
                                    ),
                                    2.0,
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                                );
                            }
                        } else {
                            // Draw as simple circle
                            painter.circle_filled(
                                egui::pos2(screen_x, screen_y),
                                5.0,
                                egui::Color32::from_rgb(150, 150, 150),
                            );

                            if *selected_entity == Some(entity) {
                                painter.circle_stroke(
                                    egui::pos2(screen_x, screen_y),
                                    8.0,
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
                                );
                            }
                        }

                        // GIZMOS: Draw collider boundaries (green wireframe)
                        if *show_colliders {
                            if let Some(collider) = world.colliders.get(&entity) {
                            let collider_size = egui::vec2(collider.width, collider.height);
                            painter.rect_stroke(
                                egui::Rect::from_center_size(
                                    egui::pos2(screen_x, screen_y),
                                    collider_size,
                                ),
                                0.0,
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100)),
                            );

                            // Draw corner handles for selected entity
                            if *selected_entity == Some(entity) {
                                let half_w = collider.width / 2.0;
                                let half_h = collider.height / 2.0;
                                let handle_radius = 3.0;

                                // Four corners
                                let corners = [
                                    (screen_x - half_w, screen_y - half_h),
                                    (screen_x + half_w, screen_y - half_h),
                                    (screen_x + half_w, screen_y + half_h),
                                    (screen_x - half_w, screen_y + half_h),
                                ];

                                for (cx, cy) in corners {
                                    painter.circle_filled(
                                        egui::pos2(cx, cy),
                                        handle_radius,
                                        egui::Color32::from_rgb(0, 255, 100),
                                    );
                                }
                            }
                            }
                        }

                        // TRANSFORM GIZMO: Draw gizmo for selected entity based on current tool
                        if *selected_entity == Some(entity) {
                            let gizmo_size = 50.0;
                            let handle_size = 8.0;

                            match current_tool {
                                TransformTool::View => {
                                    // No gizmo in View mode (Q)
                                }
                                TransformTool::Move => {
                                    // Move Tool (W) - Arrows for X/Y axes
                                    // X axis arrow (Red)
                                    let x_end = egui::pos2(screen_x + gizmo_size, screen_y);
                                    painter.line_segment(
                                        [egui::pos2(screen_x, screen_y), x_end],
                                        egui::Stroke::new(3.0, egui::Color32::from_rgb(255, 0, 0)),
                                    );
                                    painter.circle_filled(x_end, handle_size, egui::Color32::from_rgb(255, 0, 0));

                                    // Y axis arrow (Green)
                                    let y_end = egui::pos2(screen_x, screen_y + gizmo_size);
                                    painter.line_segment(
                                        [egui::pos2(screen_x, screen_y), y_end],
                                        egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 255, 0)),
                                    );
                                    painter.circle_filled(y_end, handle_size, egui::Color32::from_rgb(0, 255, 0));

                                    // Center handle (Both axes - Yellow)
                                    painter.circle_filled(
                                        egui::pos2(screen_x, screen_y),
                                        handle_size,
                                        egui::Color32::from_rgb(255, 255, 0),
                                    );
                                    painter.circle_stroke(
                                        egui::pos2(screen_x, screen_y),
                                        handle_size,
                                        egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 200, 0)),
                                    );
                                }
                                TransformTool::Rotate => {
                                    // Rotate Tool (E) - Circular ring
                                    let radius = gizmo_size * 0.8;
                                    painter.circle_stroke(
                                        egui::pos2(screen_x, screen_y),
                                        radius,
                                        egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 150, 255)),
                                    );

                                    // Rotation handles (4 points on circle)
                                    let handle_positions = [
                                        (radius, 0.0),           // Right
                                        (0.0, radius),           // Bottom
                                        (-radius, 0.0),          // Left
                                        (0.0, -radius),          // Top
                                    ];

                                    for (dx, dy) in handle_positions.iter() {
                                        painter.circle_filled(
                                            egui::pos2(screen_x + dx, screen_y + dy),
                                            handle_size,
                                            egui::Color32::from_rgb(0, 150, 255),
                                        );
                                    }

                                    // Center dot
                                    painter.circle_filled(
                                        egui::pos2(screen_x, screen_y),
                                        handle_size * 0.5,
                                        egui::Color32::from_rgb(0, 150, 255),
                                    );
                                }
                                TransformTool::Scale => {
                                    // Scale Tool (R) - Box with corner handles
                                    let box_size = gizmo_size * 0.7;

                                    // Box outline
                                    let top_left = egui::pos2(screen_x - box_size, screen_y - box_size);
                                    let top_right = egui::pos2(screen_x + box_size, screen_y - box_size);
                                    let bottom_left = egui::pos2(screen_x - box_size, screen_y + box_size);
                                    let bottom_right = egui::pos2(screen_x + box_size, screen_y + box_size);

                                    painter.line_segment([top_left, top_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                                    painter.line_segment([top_right, bottom_right], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                                    painter.line_segment([bottom_right, bottom_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));
                                    painter.line_segment([bottom_left, top_left], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 150, 0)));

                                    // Corner handles
                                    painter.circle_filled(top_left, handle_size, egui::Color32::from_rgb(255, 150, 0));
                                    painter.circle_filled(top_right, handle_size, egui::Color32::from_rgb(255, 150, 0));
                                    painter.circle_filled(bottom_left, handle_size, egui::Color32::from_rgb(255, 150, 0));
                                    painter.circle_filled(bottom_right, handle_size, egui::Color32::from_rgb(255, 150, 0));

                                    // Center handle (uniform scale)
                                    painter.circle_filled(
                                        egui::pos2(screen_x, screen_y),
                                        handle_size,
                                        egui::Color32::from_rgb(255, 200, 100),
                                    );
                                }
                            }
                        }
                    }

                    // Draw velocity vectors as arrows (for debugging)
                    if *show_velocities {
                        for (&entity, velocity) in &world.velocities {
                        if let Some(transform) = world.transforms.get(&entity) {
                            let screen_x = center_x + transform.x();
                            let screen_y = center_y + transform.y();
                            let (vx, vy) = velocity;

                            // Only draw if velocity is non-zero
                            if vx.abs() > 0.1 || vy.abs() > 0.1 {
                                let arrow_scale = 0.5;
                                let end_x = screen_x + vx * arrow_scale;
                                let end_y = screen_y + vy * arrow_scale;

                                // Arrow line
                                painter.line_segment(
                                    [egui::pos2(screen_x, screen_y), egui::pos2(end_x, end_y)],
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 0)),
                                );

                                // Arrow head
                                let arrow_size = 5.0;
                                painter.circle_filled(
                                    egui::pos2(end_x, end_y),
                                    arrow_size,
                                    egui::Color32::from_rgb(255, 255, 0),
                                );
                            }
                        }
                        }
                    }

                    // INTERACTION: Handle transform gizmo dragging
                    if let Some(sel_entity) = *selected_entity {
                        if let Some(transform) = world.transforms.get(&sel_entity) {
                            let screen_x = center_x + transform.x();
                            let screen_y = center_y + transform.y();

                            if let Some(hover_pos) = response.hover_pos() {
                                let gizmo_size = 50.0;
                                let handle_size = 8.0;

                                // Check which handle is being hovered
                                let x_handle_pos = egui::pos2(screen_x + gizmo_size, screen_y);
                                let y_handle_pos = egui::pos2(screen_x, screen_y + gizmo_size);
                                let center_handle_pos = egui::pos2(screen_x, screen_y);

                                let dist_to_x = hover_pos.distance(x_handle_pos);
                                let dist_to_y = hover_pos.distance(y_handle_pos);
                                let dist_to_center = hover_pos.distance(center_handle_pos);

                                // Determine which axis to drag
                                let mut drag_axis = None;
                                if dist_to_center < handle_size * 1.5 {
                                    drag_axis = Some(2); // Both axes
                                } else if dist_to_x < handle_size * 1.5 {
                                    drag_axis = Some(0); // X axis
                                } else if dist_to_y < handle_size * 1.5 {
                                    drag_axis = Some(1); // Y axis
                                }

                                // Handle dragging
                                if response.dragged() && drag_axis.is_some() {
                                    let delta = response.drag_delta();

                                    if let Some(transform) = world.transforms.get_mut(&sel_entity) {
                                        match drag_axis.unwrap() {
                                            0 => transform.position[0] += delta.x, // X only
                                            1 => transform.position[1] += delta.y, // Y only
                                            2 => {
                                                // Both axes
                                                transform.position[0] += delta.x;
                                                transform.position[1] += delta.y;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                1 => {
                    // Game View - show actual game when playing
                    if is_playing {
                        let (response, painter) = ui.allocate_painter(
                            ui.available_size(),
                            egui::Sense::hover(),
                        );
                        let rect = response.rect;

                        // Draw game background
                        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 25, 35));

                        // Draw entities (same as Scene but without grid)
                        let center_x = rect.center().x;
                        let center_y = rect.center().y;

                        for (&entity, transform) in &world.transforms {
                            let screen_x = center_x + transform.x();
                            let screen_y = center_y + transform.y();

                            if let Some(sprite) = world.sprites.get(&entity) {
                                let size = egui::vec2(sprite.width, sprite.height);
                                let color = egui::Color32::from_rgba_unmultiplied(
                                    (sprite.color[0] * 255.0) as u8,
                                    (sprite.color[1] * 255.0) as u8,
                                    (sprite.color[2] * 255.0) as u8,
                                    (sprite.color[3] * 255.0) as u8,
                                );

                                painter.rect_filled(
                                    egui::Rect::from_center_size(
                                        egui::pos2(screen_x, screen_y),
                                        size,
                                    ),
                                    2.0,
                                    color,
                                );
                            }
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Game View\n(Press Play to start)");
                        });
                    }
                }
                _ => {}
            }
        });

        // Bottom Panel - Assets & Console (Tabbed Unity-style)
        egui::TopBottomPanel::bottom("bottom_panel").min_height(250.0).show(ctx, |ui| {
            // Tab buttons (Unity-like)
            ui.horizontal(|ui| {
                ui.selectable_value(bottom_panel_tab, 0, "📁 Project");
                ui.selectable_value(bottom_panel_tab, 1, "📝 Console");
            });

            ui.separator();

            match *bottom_panel_tab {
                0 => {
                    // PROJECT TAB - Unity-style Asset Browser with Grid View
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(proj_path) = project_path {
                            // Grid layout for folders (Unity-like)
                            ui.horizontal_wrapped(|ui| {
                                ui.set_min_width(ui.available_width());

                                // Scripts folder
                                let scripts_path = proj_path.join("scripts");
                                if scripts_path.exists() {
                                    let folder_btn = egui::Button::new(
                                        egui::RichText::new("📁\nscripts")
                                            .size(14.0)
                                    ).min_size(egui::vec2(80.0, 60.0));

                                    if ui.add(folder_btn).clicked() {
                                        // Open folder (future: show files in bottom)
                                    }
                                }

                                // Scenes folder
                                let scenes_path = proj_path.join("scenes");
                                if scenes_path.exists() {
                                    let folder_btn = egui::Button::new(
                                        egui::RichText::new("📁\nscenes")
                                            .size(14.0)
                                    ).min_size(egui::vec2(80.0, 60.0));

                                    if ui.add(folder_btn).clicked() {
                                        // Open folder
                                    }
                                }
                            });

                            ui.add_space(10.0);
                            ui.separator();
                            ui.label(egui::RichText::new("📄 Files").strong());

                            // List files (will be improved later)
                            ui.horizontal_wrapped(|ui| {
                                ui.set_min_width(ui.available_width());

                                // Show scripts
                                let scripts_path = proj_path.join("scripts");
                                if scripts_path.exists() {
                                    if let Ok(entries) = std::fs::read_dir(&scripts_path) {
                                        for entry in entries.flatten() {
                                            if let Some(name) = entry.file_name().to_str() {
                                                if name.ends_with(".lua") {
                                                    let file_btn = egui::Button::new(
                                                        egui::RichText::new(format!("📄\n{}", name))
                                                            .size(12.0)
                                                    ).min_size(egui::vec2(70.0, 50.0));

                                                    ui.add(file_btn);
                                                }
                                            }
                                        }
                                    }
                                }

                                // Show scenes
                                let scenes_path = proj_path.join("scenes");
                                if scenes_path.exists() {
                                    if let Ok(entries) = std::fs::read_dir(&scenes_path) {
                                        for entry in entries.flatten() {
                                            if let Some(name) = entry.file_name().to_str() {
                                                if name.ends_with(".json") {
                                                    let file_btn = egui::Button::new(
                                                        egui::RichText::new(format!("🎬\n{}", name))
                                                            .size(12.0)
                                                    ).min_size(egui::vec2(70.0, 50.0));

                                                    ui.add(file_btn);
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    egui::RichText::new("No project open")
                                        .size(16.0)
                                        .color(egui::Color32::GRAY)
                                );
                            });
                        }
                    });
                }
                1 => {
                    // CONSOLE TAB
                    console.render(ui);
                }
                _ => {}
            }
        });

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
                            let scene_files = Self::get_scene_files(path);

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
                            let scene_files = Self::get_scene_files(path);

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

    /// Get all .scene files in project scenes folder
    fn get_scene_files(project_path: &std::path::Path) -> Vec<String> {
        let scenes_folder = project_path.join("scenes");
        let mut scene_files = Vec::new();

        if scenes_folder.exists() {
            if let Ok(entries) = std::fs::read_dir(&scenes_folder) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(path) = entry.path().to_str() {
                                if path.ends_with(".scene") {
                                    // Get relative path from project root
                                    if let Ok(relative) = entry.path().strip_prefix(project_path) {
                                        scene_files.push(relative.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        scene_files.sort();
        scene_files
    }

    /// Get icon for entity based on its components
    fn get_entity_icon(world: &World, entity: Entity) -> &'static str {
        // Check for specific entity types
        if let Some(tag) = world.tags.get(&entity) {
            return match tag {
                EntityTag::Player => "🎮",
                EntityTag::Item => "💎",
            };
        }

        // Check for components
        let has_sprite = world.sprites.contains_key(&entity);
        let has_collider = world.colliders.contains_key(&entity);
        let has_velocity = world.velocities.contains_key(&entity);
        let has_script = world.scripts.contains_key(&entity);

        // Determine icon based on component combination
        if has_script {
            "📜" // Script
        } else if has_velocity && has_collider {
            "🏃" // Physics object (moving + collision)
        } else if has_sprite && has_collider {
            "📦" // Sprite with collision
        } else if has_sprite {
            "🖼️" // Sprite only
        } else if has_collider {
            "⬜" // Collider only (invisible)
        } else {
            "📍" // Empty GameObject
        }
    }

    /// Parse Lua script file to extract variable declarations (Unity-like parameters)
    /// Looks for patterns like: `local speed = 10`, `jumpForce = 5.0`, `name = "Player"`
    fn parse_lua_script_parameters(script_path: &std::path::Path) -> HashMap<String, ScriptParameter> {
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
}
