use ecs::{World, Entity, EntityTag, Script, ScriptParameter, ComponentType, ComponentManager};
use egui;
use std::collections::HashMap;
use arboard::Clipboard;

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
            // Copy button for debugging
            if let Some(entity) = *selected_entity {
                if ui.button("📋 Copy All").on_hover_text("Copy all component values to clipboard").clicked() {
                    let debug_info = format_entity_debug_info(world, entity, entity_names);
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(debug_info);
                    }
                }
            }
            
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

                    ui.add_space(5.0);
                    
                    // Copy buttons for Transform
                    ui.horizontal(|ui| {
                        if ui.small_button("📋 Pos").on_hover_text("Copy position").clicked() {
                            let text = format!("{:.2}, {:.2}, {:.2}", 
                                transform.position[0], transform.position[1], transform.position[2]);
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        }
                        if ui.small_button("📋 Rot").on_hover_text("Copy rotation").clicked() {
                            let text = format!("{:.2}, {:.2}, {:.2}", 
                                transform.rotation[0], transform.rotation[1], transform.rotation[2]);
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        }
                        if ui.small_button("📋 Scale").on_hover_text("Copy scale").clicked() {
                            let text = format!("{:.2}, {:.2}, {:.2}", 
                                transform.scale[0], transform.scale[1], transform.scale[2]);
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        }
                    });

                    ui.add_space(10.0);
                }

                // Sprite Component (Unity-style collapsible)
                let has_sprite = world.has_component(entity, ComponentType::Sprite);
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

                                        ui.label("Billboard");
                                        ui.checkbox(&mut sprite.billboard, "")
                                            .on_hover_text("Always face camera in 3D mode");
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
                    let _ = world.remove_component(entity, ComponentType::Sprite);
                }

                // Collider Component (Unity-style)
                let has_collider = world.has_component(entity, ComponentType::BoxCollider);
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
                    let _ = world.remove_component(entity, ComponentType::BoxCollider);
                }

                // Rigidbody 2D Component - Unity-style with full properties
                let has_rigidbody = world.has_component(entity, ComponentType::Rigidbody);
                let mut remove_rigidbody = false;
                
                if has_rigidbody {
                    let rigidbody_id = ui.make_persistent_id("rigidbody_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), rigidbody_id, true
                    );
                    
                    render_component_header(ui, "Rigidbody 2D", "⚡", false);
                    
                    if is_open.is_open() {
                        // Ensure rigidbody exists (create if only legacy velocity exists)
                        if !world.rigidbodies.contains_key(&entity) {
                            let vel = world.velocities.get(&entity).copied().unwrap_or((0.0, 0.0));
                            let mut rb = ecs::Rigidbody2D::default();
                            rb.velocity = vel;
                            world.rigidbodies.insert(entity, rb);
                        }

                        if let Some(rigidbody) = world.rigidbodies.get_mut(&entity) {
                            ui.indent("rigidbody_indent", |ui| {
                                egui::Grid::new("rigidbody_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Velocity X");
                                        ui.add(egui::DragValue::new(&mut rigidbody.velocity.0).speed(0.1));
                                        ui.end_row();
                                        
                                        ui.label("Velocity Y");
                                        ui.add(egui::DragValue::new(&mut rigidbody.velocity.1).speed(0.1));
                                        ui.end_row();

                                        ui.label("Gravity Scale");
                                        ui.add(egui::DragValue::new(&mut rigidbody.gravity_scale).speed(0.1).clamp_range(0.0..=10.0))
                                            .on_hover_text("0 = no gravity, 1 = normal gravity");
                                        ui.end_row();

                                        ui.label("Mass");
                                        ui.add(egui::DragValue::new(&mut rigidbody.mass).speed(0.1).clamp_range(0.1..=100.0))
                                            .on_hover_text("Affects collision response");
                                        ui.end_row();

                                        ui.label("Is Kinematic");
                                        ui.checkbox(&mut rigidbody.is_kinematic, "")
                                            .on_hover_text("If checked, not affected by physics forces");
                                        ui.end_row();

                                        ui.label("Freeze Rotation");
                                        ui.checkbox(&mut rigidbody.freeze_rotation, "")
                                            .on_hover_text("Prevent rotation (for 2D games)");
                                        ui.end_row();
                                    });

                                // Sync with legacy velocity
                                world.velocities.insert(entity, rigidbody.velocity);
                                
                                ui.add_space(10.0);
                                
                                // Debug info
                                ui.label(egui::RichText::new("Debug Info:").color(egui::Color32::GRAY).small());
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!(
                                        "Speed: {:.2} px/s", 
                                        (rigidbody.velocity.0.powi(2) + rigidbody.velocity.1.powi(2)).sqrt()
                                    )).small().color(egui::Color32::GRAY));
                                    
                                    if ui.small_button("📋").on_hover_text("Copy velocity").clicked() {
                                        let text = format!("{:.2}, {:.2}", rigidbody.velocity.0, rigidbody.velocity.1);
                                        if let Ok(mut clipboard) = Clipboard::new() {
                                            let _ = clipboard.set_text(text);
                                        }
                                    }
                                });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_rigidbody = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_rigidbody {
                    let _ = world.remove_component(entity, ComponentType::Rigidbody);
                }

                // Mesh Component (3D) - Unity-style
                let has_mesh = world.has_component(entity, ComponentType::Mesh);
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
                    let _ = world.remove_component(entity, ComponentType::Mesh);
                }

                // Camera Component (Unity-style)
                let has_camera = world.has_component(entity, ComponentType::Camera);
                let mut remove_camera = false;

                if has_camera {
                    let camera_id = ui.make_persistent_id("camera_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), camera_id, true
                    );

                    render_component_header(ui, "Camera", "📷", false);

                    if is_open.is_open() {
                        if let Some(camera) = world.cameras.get_mut(&entity) {
                            ui.indent("camera_indent", |ui| {
                                egui::Grid::new("camera_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Projection");
                                        egui::ComboBox::from_id_source("projection_picker")
                                            .selected_text(match camera.projection {
                                                ecs::CameraProjection::Orthographic => "Orthographic",
                                                ecs::CameraProjection::Perspective => "Perspective",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut camera.projection, ecs::CameraProjection::Orthographic, "Orthographic");
                                                ui.selectable_value(&mut camera.projection, ecs::CameraProjection::Perspective, "Perspective");
                                            });
                                        ui.end_row();

                                        // Show different fields based on projection mode
                                        match camera.projection {
                                            ecs::CameraProjection::Orthographic => {
                                                ui.label("Size");
                                                ui.add(egui::DragValue::new(&mut camera.orthographic_size).speed(0.1).clamp_range(0.1..=1000.0));
                                                ui.end_row();
                                            }
                                            ecs::CameraProjection::Perspective => {
                                                ui.label("FOV");
                                                ui.add(egui::Slider::new(&mut camera.fov, 1.0..=179.0).suffix("°"));
                                                ui.end_row();
                                            }
                                        }

                                        ui.label("Depth");
                                        ui.add(egui::DragValue::new(&mut camera.depth).speed(1.0))
                                            .on_hover_text("Camera rendering order (lower renders first)");
                                        ui.end_row();

                                        ui.label("Background");
                                        ui.color_edit_button_rgba_unmultiplied(&mut camera.background_color);
                                        ui.end_row();

                                        ui.label("Near Clip");
                                        ui.add(egui::DragValue::new(&mut camera.near_clip).speed(0.1).clamp_range(0.01..=1000.0));
                                        ui.end_row();

                                        ui.label("Far Clip");
                                        ui.add(egui::DragValue::new(&mut camera.far_clip).speed(1.0).clamp_range(1.0..=10000.0));
                                        ui.end_row();
                                        
                                        // Aspect Ratio Presets
                                        ui.label("Aspect Ratio");
                                        ui.horizontal(|ui| {
                                            // Calculate current aspect ratio
                                            let current_aspect = camera.viewport_rect[2] / camera.viewport_rect[3];
                                            
                                            // Preset buttons
                                            if ui.small_button("16:9").on_hover_text("Set to 16:9 (1.778)").clicked() {
                                                camera.viewport_rect[2] = 1.0;
                                                camera.viewport_rect[3] = 9.0 / 16.0;
                                            }
                                            if ui.small_button("16:10").on_hover_text("Set to 16:10 (1.6)").clicked() {
                                                camera.viewport_rect[2] = 1.0;
                                                camera.viewport_rect[3] = 10.0 / 16.0;
                                            }
                                            if ui.small_button("4:3").on_hover_text("Set to 4:3 (1.333)").clicked() {
                                                camera.viewport_rect[2] = 1.0;
                                                camera.viewport_rect[3] = 3.0 / 4.0;
                                            }
                                            if ui.small_button("1:1").on_hover_text("Set to 1:1 (Square)").clicked() {
                                                camera.viewport_rect[2] = 1.0;
                                                camera.viewport_rect[3] = 1.0;
                                            }
                                            
                                            ui.label(egui::RichText::new(format!("{:.2}:1", current_aspect)).small().color(egui::Color32::GRAY));
                                        });
                                        ui.end_row();
                                    });

                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_camera = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }

                if remove_camera {
                    let _ = world.remove_component(entity, ComponentType::Camera);
                }

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

                ui.add_space(15.0);

                // ===== Add Component Button (Unity-style) =====
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 70.0);
                    ui.menu_button("➕ Add Component", |ui| {
                        // ใช้ Component Manager เพื่อดึงรายการ Component ที่สามารถเพิ่มได้
                        let addable_components = world.get_addable_components(entity);

                        if addable_components.is_empty() {
                            ui.label("All components added");
                        } else {
                            // จัดกลุ่ม Component ตามหมวดหมู่
                            let rendering_components = vec![ComponentType::Sprite, ComponentType::Mesh];
                            let physics_components = vec![ComponentType::BoxCollider, ComponentType::Rigidbody];
                            let other_components = vec![ComponentType::Camera, ComponentType::Script, ComponentType::Tag];

                            // Rendering Section
                            let has_rendering = addable_components.iter().any(|c| rendering_components.contains(c));
                            if has_rendering {
                                ui.label("🎨 Rendering");
                                ui.separator();
                                for component_type in &rendering_components {
                                    if addable_components.contains(component_type) {
                                        if ui.button(component_type.display_name()).clicked() {
                                            let _ = world.add_component(entity, *component_type);
                                            ui.close_menu();
                                        }
                                    }
                                }
                                ui.add_space(5.0);
                            }

                            // Physics Section
                            let has_physics = addable_components.iter().any(|c| physics_components.contains(c));
                            if has_physics {
                                ui.label("⚙️ Physics");
                                ui.separator();
                                for component_type in &physics_components {
                                    if addable_components.contains(component_type) {
                                        if ui.button(component_type.display_name()).clicked() {
                                            let _ = world.add_component(entity, *component_type);
                                            ui.close_menu();
                                        }
                                    }
                                }
                                ui.add_space(5.0);
                            }

                            // Other Components Section
                            let has_other = addable_components.iter().any(|c| other_components.contains(c));
                            if has_other {
                                ui.label("📜 Other");
                                ui.separator();
                                for component_type in &other_components {
                                    if addable_components.contains(component_type) {
                                        if ui.button(component_type.display_name()).clicked() {
                                            let _ = world.add_component(entity, *component_type);
                                            ui.close_menu();
                                        }
                                    }
                                }
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

/// Format entity debug information for copying to clipboard
fn format_entity_debug_info(
    world: &World,
    entity: ecs::Entity,
    entity_names: &HashMap<ecs::Entity, String>,
) -> String {
    let mut info = String::new();
    
    // Entity header
    let name = entity_names.get(&entity).map(|s| s.as_str()).unwrap_or("Unnamed");
    info.push_str(&format!("=== Entity {} ({}) ===\n\n", entity, name));
    
    // Active state
    let is_active = world.active.get(&entity).copied().unwrap_or(true);
    info.push_str(&format!("Active: {}\n", is_active));
    
    // Layer
    let layer = world.layers.get(&entity).copied().unwrap_or(0);
    info.push_str(&format!("Layer: {}\n\n", layer));
    
    // Transform
    if let Some(transform) = world.transforms.get(&entity) {
        info.push_str("Transform:\n");
        info.push_str(&format!("  Position: [{:.2}, {:.2}, {:.2}]\n", 
            transform.position[0], transform.position[1], transform.position[2]));
        info.push_str(&format!("  Rotation: [{:.2}, {:.2}, {:.2}]\n", 
            transform.rotation[0], transform.rotation[1], transform.rotation[2]));
        info.push_str(&format!("  Scale: [{:.2}, {:.2}, {:.2}]\n\n", 
            transform.scale[0], transform.scale[1], transform.scale[2]));
    }
    
    // Sprite
    if let Some(sprite) = world.sprites.get(&entity) {
        info.push_str("Sprite Renderer:\n");
        info.push_str(&format!("  Texture: {}\n", sprite.texture_id));
        info.push_str(&format!("  Size: {:.1} x {:.1}\n", sprite.width, sprite.height));
        info.push_str(&format!("  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n", 
            sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]));
        info.push_str(&format!("  Billboard: {}\n\n", sprite.billboard));
    }
    
    // Box Collider
    if let Some(collider) = world.colliders.get(&entity) {
        info.push_str("Box Collider 2D:\n");
        info.push_str(&format!("  Width: {:.1}\n", collider.width));
        info.push_str(&format!("  Height: {:.1}\n\n", collider.height));
    }
    
    // Rigidbody
    if let Some(rigidbody) = world.rigidbodies.get(&entity) {
        info.push_str("Rigidbody 2D:\n");
        info.push_str(&format!("  Velocity: [{:.2}, {:.2}]\n", 
            rigidbody.velocity.0, rigidbody.velocity.1));
        info.push_str(&format!("  Gravity Scale: {:.2}\n", rigidbody.gravity_scale));
        info.push_str(&format!("  Mass: {:.2}\n", rigidbody.mass));
        info.push_str(&format!("  Is Kinematic: {}\n", rigidbody.is_kinematic));
        info.push_str(&format!("  Freeze Rotation: {}\n\n", rigidbody.freeze_rotation));
    } else if let Some(velocity) = world.velocities.get(&entity) {
        info.push_str("Velocity (Legacy):\n");
        info.push_str(&format!("  [{:.2}, {:.2}]\n\n", velocity.0, velocity.1));
    }
    
    // Mesh
    if let Some(mesh) = world.meshes.get(&entity) {
        info.push_str("Mesh Renderer:\n");
        info.push_str(&format!("  Type: {:?}\n", mesh.mesh_type));
        info.push_str(&format!("  Color: [{:.2}, {:.2}, {:.2}, {:.2}]\n\n", 
            mesh.color[0], mesh.color[1], mesh.color[2], mesh.color[3]));
    }
    
    // Camera
    if let Some(camera) = world.cameras.get(&entity) {
        info.push_str("Camera:\n");
        info.push_str(&format!("  Projection: {:?}\n", camera.projection));
        info.push_str(&format!("  FOV: {:.1}°\n", camera.fov));
        info.push_str(&format!("  Orthographic Size: {:.1}\n", camera.orthographic_size));
        info.push_str(&format!("  Near Clip: {:.2}\n", camera.near_clip));
        info.push_str(&format!("  Far Clip: {:.1}\n", camera.far_clip));
        info.push_str(&format!("  Depth: {}\n\n", camera.depth));
    }
    
    // Script
    if let Some(script) = world.scripts.get(&entity) {
        info.push_str("Script:\n");
        info.push_str(&format!("  Name: {}\n", script.script_name));
        info.push_str(&format!("  Enabled: {}\n", script.enabled));
        if !script.parameters.is_empty() {
            info.push_str("  Parameters:\n");
            for (key, value) in &script.parameters {
                info.push_str(&format!("    {}: {:?}\n", key, value));
            }
        }
        info.push_str("\n");
    }
    
    // Tag
    if let Some(tag) = world.tags.get(&entity) {
        info.push_str(&format!("Tag: {:?}\n\n", tag));
    }
    
    // Hierarchy
    if let Some(parent) = world.parents.get(&entity) {
        let parent_name = entity_names.get(parent).map(|s| s.as_str()).unwrap_or("Unnamed");
        info.push_str(&format!("Parent: {} ({})\n", parent, parent_name));
    }
    
    let children = world.children.get(&entity);
    if let Some(children) = children {
        if !children.is_empty() {
            info.push_str(&format!("Children: {}\n", children.len()));
            for child in children {
                let child_name = entity_names.get(child).map(|s| s.as_str()).unwrap_or("Unnamed");
                info.push_str(&format!("  - {} ({})\n", child, child_name));
            }
        }
    }
    
    info
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
