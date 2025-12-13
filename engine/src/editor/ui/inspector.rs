use ecs::{World, Entity, EntityTag, ScriptParameter, ComponentType, ComponentManager};
use egui;
use std::collections::HashMap;
use arboard::Clipboard;

/// Parse hex color string to egui Color32
fn parse_hex_color(hex: &str) -> Result<egui::Color32, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color format".to_string());
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;
    
    Ok(egui::Color32::from_rgb(r, g, b))
}

/// Renders the Inspector panel showing entity properties and components
pub fn render_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    selected_entity: &mut Option<Entity>,
    entity_names: &mut HashMap<Entity, String>,
    edit_script_request: &mut Option<String>,
    project_path: &Option<std::path::PathBuf>,
    open_sprite_editor_request: &mut Option<std::path::PathBuf>,
    sprite_picker_state: &mut super::sprite_picker::SpritePickerState,
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
                                        // Sprite picker (Unity-style)
                                        ui.label("Sprite");
                                        ui.horizontal(|ui| {
                                            // Show current sprite name with icon
                                            let sprite_display = if sprite.texture_id.is_empty() {
                                                "None (Sprite)".to_string()
                                            } else {
                                                sprite.texture_id.clone()
                                            };

                                            // Clickable sprite field (Unity-style)
                                            if ui.button(&sprite_display).clicked() {
                                                sprite_picker_state.open();
                                                log::info!("Opened sprite picker for Sprite Renderer");
                                            }

                                            // Manual text edit option
                                            ui.menu_button("⋮", |ui| {
                                                ui.label("Edit manually:");
                                                ui.text_edit_singleline(&mut sprite.texture_id);
                                            });
                                        });
                                        ui.end_row();
                                        
                                        // Color tint
                                        ui.label("Color");
                                        ui.color_edit_button_rgba_unmultiplied(&mut sprite.color);
                                        ui.end_row();
                                        
                                        // Flip options (Unity-style)
                                        ui.label("Flip");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.checkbox(&mut sprite.flip_x, "");
                                            ui.label("Y");
                                            ui.checkbox(&mut sprite.flip_y, "");
                                        });
                                        ui.end_row();

                                        // Draw Mode
                                        ui.label("Draw Mode");
                                        egui::ComboBox::from_id_source("draw_mode")
                                            .selected_text("Simple")
                                            .width(150.0)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_label(true, "Simple");
                                                ui.selectable_label(false, "Sliced");
                                                ui.selectable_label(false, "Tiled");
                                            });
                                        ui.end_row();

                                        // Billboard (3D mode)
                                        ui.label("Billboard");
                                        ui.checkbox(&mut sprite.billboard, "")
                                            .on_hover_text("Always face camera in 3D mode");
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // Info message about scale
                                ui.label(egui::RichText::new("💡 Use Transform Scale to resize sprite")
                                    .small()
                                    .color(egui::Color32::from_rgb(150, 150, 150)));
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("🎨 Open Sprite Editor").clicked() {
                                        // TODO: Open sprite editor
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

                // SpriteSheet Component (Unity-style collapsible)
                let has_sprite_sheet = world.has_component(entity, ComponentType::SpriteSheet);
                let mut remove_sprite_sheet = false;
                
                if has_sprite_sheet {
                    let sprite_sheet_id = ui.make_persistent_id("sprite_sheet_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), sprite_sheet_id, true
                    );
                    
                    render_component_header(ui, "Sprite Sheet", "🎞", false);
                    
                    if is_open.is_open() {
                        if let Some(sprite_sheet) = world.sprite_sheets.get_mut(&entity) {
                            ui.indent("sprite_sheet_indent", |ui| {
                                egui::Grid::new("sprite_sheet_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        // Sprite name display
                                        ui.label("Sprite Name");
                                        if !sprite_sheet.frames.is_empty() {
                                            // Show first frame name or "Multiple frames"
                                            let display_name = if sprite_sheet.frames.len() == 1 {
                                                sprite_sheet.frames[0].name.as_ref()
                                                    .map(|n| n.as_str())
                                                    .unwrap_or("Unnamed")
                                            } else {
                                                "Multiple frames"
                                            };
                                            ui.label(egui::RichText::new(display_name).strong());
                                        } else {
                                            ui.label(egui::RichText::new("No sprites").color(egui::Color32::GRAY));
                                        }
                                        ui.end_row();
                                        
                                        // Source texture path
                                        ui.label("Texture Path");
                                        ui.label(&sprite_sheet.texture_path);
                                        ui.end_row();
                                        
                                        // Texture dimensions
                                        ui.label("Texture Size");
                                        ui.label(format!("{}x{}", sprite_sheet.sheet_width, sprite_sheet.sheet_height));
                                        ui.end_row();
                                        
                                        // Frame count
                                        ui.label("Frame Count");
                                        ui.label(format!("{}", sprite_sheet.frames.len()));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // Info message
                                ui.label(egui::RichText::new("💡 Edit sprite definitions in the Sprite Editor")
                                    .small()
                                    .color(egui::Color32::from_rgb(150, 150, 150)));
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }

                                    // Select Sprite button - opens sprite picker
                                    if ui.button("🖼️ Select Sprite").on_hover_text("Choose a sprite from project").clicked() {
                                        sprite_picker_state.open();
                                        log::info!("Opened sprite picker");
                                    }

                                    // Edit Sprite Sheet button - opens sprite editor
                                    if ui.button("🎨 Edit Sprite Sheet").clicked() {
                                        // Request to open sprite editor for this texture
                                        let texture_path = std::path::PathBuf::from(&sprite_sheet.texture_path);
                                        *open_sprite_editor_request = Some(texture_path);
                                        log::info!("Requested to open sprite editor for: {}", sprite_sheet.texture_path);
                                    }

                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_sprite_sheet = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_sprite_sheet {
                    let _ = world.remove_component(entity, ComponentType::SpriteSheet);
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
                        // Migrate legacy colliders
                        if let Some(transform) = world.transforms.get(&entity) {
                            if let Some(collider) = world.colliders.get_mut(&entity) {
                                collider.migrate_from_legacy(transform.scale);
                            }
                        }
                        
                        if let Some(collider) = world.colliders.get_mut(&entity) {
                            ui.indent("collider_indent", |ui| {
                                egui::Grid::new("collider_grid")
                                    .num_columns(5)
                                    .spacing([5.0, 8.0])
                                    .show(ui, |ui| {
                                        // Edit Collider button
                                        ui.label("Edit Collider");
                                        if ui.button("🔧").on_hover_text("Edit collider shape").clicked() {
                                            // TODO: Open collider editor
                                        }
                                        ui.end_row();
                                        
                                        // Offset
                                        ui.label("Offset");
                                        ui.label("X");
                                        ui.add(egui::DragValue::new(&mut collider.offset[0]).speed(0.01).max_decimals(2));
                                        ui.label("Y");
                                        ui.add(egui::DragValue::new(&mut collider.offset[1]).speed(0.01).max_decimals(2));
                                        ui.end_row();
                                        
                                        // Size
                                        ui.label("Size");
                                        ui.label("X");
                                        ui.add(egui::DragValue::new(&mut collider.size[0]).speed(0.01).max_decimals(2).clamp_range(0.01..=100.0));
                                        ui.label("Y");
                                        ui.add(egui::DragValue::new(&mut collider.size[1]).speed(0.01).max_decimals(2).clamp_range(0.01..=100.0));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // Show actual world size
                                if let Some(transform) = world.transforms.get(&entity) {
                                    let world_width = collider.get_world_width(transform.scale[0]);
                                    let world_height = collider.get_world_height(transform.scale[1]);
                                    ui.label(egui::RichText::new(format!(
                                        "💡 World size: {:.2} x {:.2} (Size × Transform.scale)",
                                        world_width, world_height
                                    )).small().color(egui::Color32::from_rgb(150, 150, 150)));
                                }
                                
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

                                        ui.label("Enable CCD");
                                        ui.checkbox(&mut rigidbody.enable_ccd, "")
                                            .on_hover_text("Continuous Collision Detection - prevents fast objects from tunneling through colliders");
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
                                        
                                        ui.label("Pixels Per Unit");
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut camera.pixels_per_unit).speed(0.1).clamp_range(0.1..=1000.0))
                                                .on_hover_text("How many pixels = 1 world unit (1 meter)\n100.0 = Unity standard (recommended for 2.5D/3D)\n10.0 = pixel art games");
                                            if ui.small_button("Unity").on_hover_text("Unity standard (100 pixels = 1 world unit) - Recommended").clicked() {
                                                camera.pixels_per_unit = 100.0;
                                            }
                                            if ui.small_button("Pixel").on_hover_text("Pixel art (10 pixels = 1 world unit)").clicked() {
                                                camera.pixels_per_unit = 10.0;
                                            }
                                        });
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

                // Grid Component (Unity-style)
                let has_grid = world.grids.contains_key(&entity);
                let mut remove_grid = false;
                
                if has_grid {
                    let grid_id = ui.make_persistent_id("grid_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), grid_id, true
                    );
                    
                    render_component_header(ui, "Grid", "🗺️", false);
                    
                    if is_open.is_open() {
                        if let Some(grid) = world.grids.get_mut(&entity) {
                            ui.indent("grid_indent", |ui| {
                                egui::Grid::new("grid_component_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        // Cell Size
                                        ui.label("Cell Size");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.add(egui::DragValue::new(&mut grid.cell_size.0).speed(0.01).max_decimals(3));
                                            ui.label("Y");
                                            ui.add(egui::DragValue::new(&mut grid.cell_size.1).speed(0.01).max_decimals(3));
                                            ui.label("Z");
                                            ui.add(egui::DragValue::new(&mut grid.cell_size.2).speed(0.01).max_decimals(3));
                                        });
                                        ui.end_row();
                                        
                                        // Cell Gap (with Unity-style validation)
                                        ui.label("Cell Gap");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            let mut gap_x = grid.cell_gap.0;
                                            if ui.add(egui::DragValue::new(&mut gap_x).speed(0.01).max_decimals(3)).changed() {
                                                // Unity validation: clamp negative gap to -cell_size
                                                if gap_x < 0.0 && gap_x.abs() > grid.cell_size.0 {
                                                    gap_x = -grid.cell_size.0;
                                                }
                                                grid.cell_gap.0 = gap_x;
                                            }
                                            ui.label("Y");
                                            let mut gap_y = grid.cell_gap.1;
                                            if ui.add(egui::DragValue::new(&mut gap_y).speed(0.01).max_decimals(3)).changed() {
                                                // Unity validation: clamp negative gap to -cell_size
                                                if gap_y < 0.0 && gap_y.abs() > grid.cell_size.1 {
                                                    gap_y = -grid.cell_size.1;
                                                }
                                                grid.cell_gap.1 = gap_y;
                                            }
                                        });
                                        ui.end_row();
                                        
                                        // Cell Layout (Unity naming)
                                        ui.label("Cell Layout");
                                        let layout_text = match grid.layout {
                                            ecs::GridLayout::Rectangle => "Rectangle",
                                            ecs::GridLayout::Hexagon(ecs::HexagonOrientation::FlatTop) => "Hexagon (Flat Top)",
                                            ecs::GridLayout::Hexagon(ecs::HexagonOrientation::PointyTop) => "Hexagon (Pointy Top)",
                                            ecs::GridLayout::Isometric => "Isometric",
                                        };
                                        egui::ComboBox::from_id_source("grid_layout")
                                            .selected_text(layout_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(matches!(grid.layout, ecs::GridLayout::Rectangle), "Rectangle").clicked() {
                                                    grid.layout = ecs::GridLayout::Rectangle;
                                                }
                                                if ui.selectable_label(matches!(grid.layout, ecs::GridLayout::Hexagon(ecs::HexagonOrientation::FlatTop)), "Hexagon (Flat Top)").clicked() {
                                                    grid.layout = ecs::GridLayout::Hexagon(ecs::HexagonOrientation::FlatTop);
                                                }
                                                if ui.selectable_label(matches!(grid.layout, ecs::GridLayout::Hexagon(ecs::HexagonOrientation::PointyTop)), "Hexagon (Pointy Top)").clicked() {
                                                    grid.layout = ecs::GridLayout::Hexagon(ecs::HexagonOrientation::PointyTop);
                                                }
                                                if ui.selectable_label(matches!(grid.layout, ecs::GridLayout::Isometric), "Isometric").clicked() {
                                                    grid.layout = ecs::GridLayout::Isometric;
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Cell Swizzle (Unity naming)
                                        ui.label("Cell Swizzle");
                                        let swizzle_text = match grid.swizzle {
                                            ecs::CellSwizzle::XYZ => "XYZ",
                                            ecs::CellSwizzle::XZY => "XZY",
                                            ecs::CellSwizzle::YXZ => "YXZ",
                                            ecs::CellSwizzle::YZX => "YZX",
                                            ecs::CellSwizzle::ZXY => "ZXY",
                                            ecs::CellSwizzle::ZYX => "ZYX",
                                        };
                                        egui::ComboBox::from_id_source("grid_swizzle")
                                            .selected_text(swizzle_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::XYZ), "XYZ").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::XYZ;
                                                }
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::XZY), "XZY").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::XZY;
                                                }
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::YXZ), "YXZ").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::YXZ;
                                                }
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::YZX), "YZX").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::YZX;
                                                }
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::ZXY), "ZXY").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::ZXY;
                                                }
                                                if ui.selectable_label(matches!(grid.swizzle, ecs::CellSwizzle::ZYX), "ZYX").clicked() {
                                                    grid.swizzle = ecs::CellSwizzle::ZYX;
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Plane (Custom property - not in Unity)
                                        ui.label("Plane");
                                        let plane_text = match grid.plane {
                                            ecs::GridPlane::XY => "XY (Horizontal)",
                                            ecs::GridPlane::XZ => "XZ (Vertical)",
                                            ecs::GridPlane::YZ => "YZ (Side)",
                                        };
                                        egui::ComboBox::from_id_source("grid_plane")
                                            .selected_text(plane_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(matches!(grid.plane, ecs::GridPlane::XY), "XY (Horizontal)").clicked() {
                                                    grid.plane = ecs::GridPlane::XY;
                                                }
                                                if ui.selectable_label(matches!(grid.plane, ecs::GridPlane::XZ), "XZ (Vertical)").clicked() {
                                                    grid.plane = ecs::GridPlane::XZ;
                                                }
                                                if ui.selectable_label(matches!(grid.plane, ecs::GridPlane::YZ), "YZ (Side)").clicked() {
                                                    grid.plane = ecs::GridPlane::YZ;
                                                }
                                            });
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // 2D/3D Mode Toggle
                                ui.horizontal(|ui| {
                                    ui.label("Mode:");
                                    if ui.button(if grid.is_3d_mode() { "🎮 3D Mode" } else { "🎨 2D Mode" })
                                        .on_hover_text("Toggle between 2D and 3D tilemap mode")
                                        .clicked() 
                                    {
                                        if grid.is_3d_mode() {
                                            grid.to_2d_mode();
                                            log::info!("Switched Grid to 2D mode (XY plane)");
                                        } else {
                                            grid.to_3d_mode();
                                            log::info!("Switched Grid to 3D mode (XZ plane)");
                                        }
                                    }
                                });
                                
                                ui.add_space(5.0);
                                
                                // Info message
                                let mode_info = if grid.is_3d_mode() {
                                    "💡 3D Mode: Tilemaps render as vertical walls (XZ plane)"
                                } else {
                                    "💡 2D Mode: Tilemaps render horizontally (XY plane)"
                                };
                                ui.label(egui::RichText::new(mode_info)
                                    .small()
                                    .color(egui::Color32::from_rgb(150, 150, 150)));
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_grid = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_grid {
                    world.grids.remove(&entity);
                }

                // Tilemap Component (Unity-style)
                let has_tilemap = world.tilemaps.contains_key(&entity);
                let mut remove_tilemap = false;
                
                if has_tilemap {
                    let tilemap_id = ui.make_persistent_id("tilemap_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), tilemap_id, true
                    );
                    
                    render_component_header(ui, "Tilemap", "🗺️", false);
                    
                    if is_open.is_open() {
                        if let Some(tilemap) = world.tilemaps.get_mut(&entity) {
                            ui.indent("tilemap_indent", |ui| {
                                egui::Grid::new("tilemap_component_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        // Animation Frame Rate
                                        ui.label("Animation Frame Rate");
                                        ui.add(egui::DragValue::new(&mut tilemap.animation_frame_rate).speed(1).clamp_range(1..=60));
                                        ui.end_row();
                                        
                                        // Color
                                        ui.label("Color");
                                        ui.horizontal(|ui| {
                                            let mut color = egui::Color32::from_rgba_premultiplied(
                                                (tilemap.color[0] * 255.0) as u8,
                                                (tilemap.color[1] * 255.0) as u8,
                                                (tilemap.color[2] * 255.0) as u8,
                                                (tilemap.color[3] * 255.0) as u8,
                                            );
                                            if ui.color_edit_button_srgba(&mut color).changed() {
                                                tilemap.color[0] = color.r() as f32 / 255.0;
                                                tilemap.color[1] = color.g() as f32 / 255.0;
                                                tilemap.color[2] = color.b() as f32 / 255.0;
                                                tilemap.color[3] = color.a() as f32 / 255.0;
                                            }
                                        });
                                        ui.end_row();
                                        
                                        // Tile Anchor
                                        ui.label("Tile Anchor");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.add(egui::DragValue::new(&mut tilemap.tile_anchor[0]).speed(0.01).clamp_range(0.0..=1.0));
                                            ui.label("Y");
                                            ui.add(egui::DragValue::new(&mut tilemap.tile_anchor[1]).speed(0.01).clamp_range(0.0..=1.0));
                                        });
                                        ui.end_row();
                                        
                                        // Orientation
                                        ui.label("Orientation");
                                        egui::ComboBox::from_id_source("tilemap_orientation")
                                            .selected_text(&tilemap.orientation)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(tilemap.orientation == "XY", "XY").clicked() {
                                                    tilemap.orientation = "XY".to_string();
                                                }
                                                if ui.selectable_label(tilemap.orientation == "XZ", "XZ").clicked() {
                                                    tilemap.orientation = "XZ".to_string();
                                                }
                                                if ui.selectable_label(tilemap.orientation == "YZ", "YZ").clicked() {
                                                    tilemap.orientation = "YZ".to_string();
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Offset
                                        ui.label("Offset");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.add(egui::DragValue::new(&mut tilemap.offset[0]).speed(0.01));
                                            ui.label("Y");
                                            ui.add(egui::DragValue::new(&mut tilemap.offset[1]).speed(0.01));
                                            ui.label("Z");
                                            ui.add(egui::DragValue::new(&mut tilemap.offset[2]).speed(0.01));
                                        });
                                        ui.end_row();
                                        
                                        // Rotation
                                        ui.label("Rotation");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.add(egui::DragValue::new(&mut tilemap.rotation[0]).speed(0.1));
                                            ui.label("Y");
                                            ui.add(egui::DragValue::new(&mut tilemap.rotation[1]).speed(0.1));
                                            ui.label("Z");
                                            ui.add(egui::DragValue::new(&mut tilemap.rotation[2]).speed(0.1));
                                        });
                                        ui.end_row();
                                        
                                        // Scale
                                        ui.label("Scale");
                                        ui.horizontal(|ui| {
                                            ui.label("X");
                                            ui.add(egui::DragValue::new(&mut tilemap.scale[0]).speed(0.01));
                                            ui.label("Y");
                                            ui.add(egui::DragValue::new(&mut tilemap.scale[1]).speed(0.01));
                                            ui.label("Z");
                                            ui.add(egui::DragValue::new(&mut tilemap.scale[2]).speed(0.01));
                                        });
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // Info section
                                ui.collapsing("Info", |ui| {
                                    ui.label(format!("Tiles: {} ({}x{})", tilemap.tiles.len(), tilemap.width, tilemap.height));
                                    ui.label("Sprites: None");
                                });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_tilemap = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_tilemap {
                    world.tilemaps.remove(&entity);
                }

                // TilemapRenderer Component (Unity-style)
                let has_tilemap_renderer = world.tilemap_renderers.contains_key(&entity);
                let mut remove_tilemap_renderer = false;
                
                if has_tilemap_renderer {
                    let renderer_id = ui.make_persistent_id("tilemap_renderer_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), renderer_id, true
                    );
                    
                    render_component_header(ui, "Tilemap Renderer", "🎨", false);
                    
                    if is_open.is_open() {
                        if let Some(renderer) = world.tilemap_renderers.get_mut(&entity) {
                            ui.indent("tilemap_renderer_indent", |ui| {
                                egui::Grid::new("tilemap_renderer_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        // Sort Order
                                        ui.label("Sort Order");
                                        ui.label("Bottom Left");
                                        ui.end_row();
                                        
                                        // Mode
                                        ui.label("Mode");
                                        let mode_text = match renderer.mode {
                                            ecs::TilemapRenderMode::Individual => "Individual",
                                            ecs::TilemapRenderMode::Chunk => "Chunk",
                                        };
                                        egui::ComboBox::from_id_source("tilemap_render_mode")
                                            .selected_text(mode_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(matches!(renderer.mode, ecs::TilemapRenderMode::Individual), "Individual").clicked() {
                                                    renderer.mode = ecs::TilemapRenderMode::Individual;
                                                }
                                                if ui.selectable_label(matches!(renderer.mode, ecs::TilemapRenderMode::Chunk), "Chunk").clicked() {
                                                    renderer.mode = ecs::TilemapRenderMode::Chunk;
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Detect Chunk Culling Bounds
                                        ui.label("Detect Chunk Culling Bounds");
                                        let detect_text = if renderer.detect_chunk_culling { "Auto" } else { "Manual" };
                                        egui::ComboBox::from_id_source("chunk_culling")
                                            .selected_text(detect_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(renderer.detect_chunk_culling, "Auto").clicked() {
                                                    renderer.detect_chunk_culling = true;
                                                }
                                                if ui.selectable_label(!renderer.detect_chunk_culling, "Manual").clicked() {
                                                    renderer.detect_chunk_culling = false;
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Chunk Culling Bounds (if manual)
                                        if !renderer.detect_chunk_culling {
                                            ui.label("Chunk Culling Bounds");
                                            ui.horizontal(|ui| {
                                                ui.label("X: 0  Y: 0  Z: 0");
                                            });
                                            ui.end_row();
                                        }
                                        
                                        // Mask Interaction
                                        ui.label("Mask Interaction");
                                        let mask_text = match renderer.mask_interaction {
                                            ecs::MaskInteraction::None => "None",
                                            ecs::MaskInteraction::VisibleInsideMask => "Visible Inside Mask",
                                            ecs::MaskInteraction::VisibleOutsideMask => "Visible Outside Mask",
                                        };
                                        egui::ComboBox::from_id_source("mask_interaction")
                                            .selected_text(mask_text)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(matches!(renderer.mask_interaction, ecs::MaskInteraction::None), "None").clicked() {
                                                    renderer.mask_interaction = ecs::MaskInteraction::None;
                                                }
                                                if ui.selectable_label(matches!(renderer.mask_interaction, ecs::MaskInteraction::VisibleInsideMask), "Visible Inside Mask").clicked() {
                                                    renderer.mask_interaction = ecs::MaskInteraction::VisibleInsideMask;
                                                }
                                                if ui.selectable_label(matches!(renderer.mask_interaction, ecs::MaskInteraction::VisibleOutsideMask), "Visible Outside Mask").clicked() {
                                                    renderer.mask_interaction = ecs::MaskInteraction::VisibleOutsideMask;
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Material
                                        ui.label("Material");
                                        ui.horizontal(|ui| {
                                            ui.label("⚪ Sprite-Lit-Default");
                                            if ui.button("Edit...").clicked() {
                                                // Open material editor
                                            }
                                        });
                                        ui.end_row();
                                    });
                                
                                ui.add_space(5.0);
                                
                                // Additional Settings
                                ui.collapsing("Additional Settings", |ui| {
                                    egui::Grid::new("additional_settings_grid")
                                        .num_columns(2)
                                        .spacing([10.0, 8.0])
                                        .show(ui, |ui| {
                                            ui.label("Sorting Layer");
                                            ui.add(egui::TextEdit::singleline(&mut renderer.sorting_layer).desired_width(100.0));
                                            ui.end_row();
                                            
                                            ui.label("Order in Layer");
                                            ui.add(egui::DragValue::new(&mut renderer.order_in_layer).speed(1));
                                            ui.end_row();
                                        });
                                });
                                
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⚙️").on_hover_text("Component Settings").clicked() {
                                        // Component menu
                                    }
                                    if ui.button("❌ Remove Component").clicked() {
                                        remove_tilemap_renderer = true;
                                    }
                                });
                            });
                        }
                        ui.add_space(10.0);
                    }
                }
                
                if remove_tilemap_renderer {
                    world.tilemap_renderers.remove(&entity);
                }

                // Map Component
                if world.has_component(entity, ComponentType::Map) {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 50, 50))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            super::map_inspector::render_map_inspector(
                                ui,
                                world,
                                entity,
                                project_path,
                            );
                        });
                    ui.add_space(10.0);
                }

                // LdtkMap Component
                let has_ldtk_map = world.has_component(entity, ComponentType::LdtkMap);
                let mut remove_ldtk_map = false;
                
                if has_ldtk_map {
                    let ldtk_map_id = ui.make_persistent_id("ldtk_map_component");
                    let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(), ldtk_map_id, true
                    );
                    
                    render_component_header(ui, "LDTK Map", "🗺️", false);
                    
                    if is_open.is_open() {
                        // Store values to avoid borrow checker issues
                        let mut file_path = world.ldtk_maps.get(&entity).map(|m| m.file_path.clone()).unwrap_or_default();
                        let mut load_requested = false;
                        let mut clear_requested = false;
                        let mut reload_requested = false;
                        let mut regenerate_colliders_requested = false;
                        let mut toggle_visibility_requested = false;
                        let collider_count = world.get_children(entity).len();
                        
                        if let Some(ldtk_map) = world.ldtk_maps.get_mut(&entity) {
                            ui.indent("ldtk_map_indent", |ui| {
                                // === File Section ===
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("📁 File").strong());
                                    ui.separator();
                                    
                                    // File path with modern styling
                                    ui.horizontal(|ui| {
                                        let file_display = if file_path.is_empty() {
                                            egui::RichText::new("No file selected").color(egui::Color32::GRAY).italics()
                                        } else {
                                            let file_name = std::path::Path::new(&file_path)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or(&file_path);
                                            egui::RichText::new(file_name).strong()
                                        };
                                        
                                        ui.label(file_display);
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Recent files dropdown
                                            if ui.small_button("📋").on_hover_text("Recent files").clicked() {
                                                // TODO: Implement recent files
                                            }
                                            
                                            // Browse button
                                            if ui.small_button("📁").on_hover_text("Browse for LDTK file").clicked() {
                                                if let Some(project_path) = project_path {
                                                    if let Some(path) = rfd::FileDialog::new()
                                                        .add_filter("LDTK Files", &["ldtk"])
                                                        .set_directory(project_path)
                                                        .pick_file()
                                                    {
                                                        if let Some(relative_path) = path.strip_prefix(project_path).ok() {
                                                            file_path = relative_path.to_string_lossy().to_string();
                                                        } else {
                                                            file_path = path.to_string_lossy().to_string();
                                                        }
                                                        load_requested = true; // Auto-load when file selected
                                                    }
                                                }
                                            }
                                        });
                                    });
                                    
                                    // Full path display (collapsible)
                                    if !file_path.is_empty() {
                                        ui.collapsing("Full Path", |ui| {
                                            ui.horizontal(|ui| {
                                                ui.text_edit_singleline(&mut file_path);
                                                if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                                    if let Ok(mut clipboard) = Clipboard::new() {
                                                        let _ = clipboard.set_text(file_path.clone());
                                                    }
                                                }
                                            });
                                        });
                                    }
                                    
                                    // Drag & Drop area
                                    let drop_area = ui.allocate_response(
                                        egui::Vec2::new(ui.available_width(), 40.0),
                                        egui::Sense::hover()
                                    );
                                    
                                    ui.allocate_ui_at_rect(drop_area.rect, |ui| {
                                        egui::Frame::none()
                                            .fill(egui::Color32::from_rgb(40, 40, 40))
                                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)))
                                            .rounding(4.0)
                                            .inner_margin(egui::Margin::same(8.0))
                                            .show(ui, |ui| {
                                                ui.centered_and_justified(|ui| {
                                                    ui.label(egui::RichText::new("📂 Drag LDTK file here")
                                                        .color(egui::Color32::GRAY)
                                                        .italics());
                                                });
                                            });
                                    });
                                    
                                    // Handle drag & drop
                                    if let Some(dropped_files) = ui.input(|i| {
                                        if i.raw.dropped_files.is_empty() {
                                            None
                                        } else {
                                            Some(i.raw.dropped_files.clone())
                                        }
                                    }) {
                                        for dropped_file in dropped_files {
                                            if let Some(path) = &dropped_file.path {
                                                if path.extension().and_then(|s| s.to_str()) == Some("ldtk") {
                                                    if let Some(project_path) = project_path {
                                                        if let Ok(relative_path) = path.strip_prefix(project_path) {
                                                            file_path = relative_path.to_string_lossy().to_string();
                                                        } else {
                                                            file_path = path.to_string_lossy().to_string();
                                                        }
                                                        load_requested = true;
                                                        log::info!("Dropped LDTK file: {}", file_path);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                });
                                
                                // Update the file path
                                ldtk_map.file_path = file_path.clone();
                                
                                ui.add_space(5.0);
                                
                                // === Status Section ===
                                if !ldtk_map.identifier.is_empty() {
                                    ui.group(|ui| {
                                        ui.label(egui::RichText::new("📊 Status").strong());
                                        ui.separator();
                                        
                                        // Status indicator
                                        ui.horizontal(|ui| {
                                            ui.label("✅");
                                            ui.label(egui::RichText::new(format!("Loaded: {}", ldtk_map.identifier)).strong());
                                        });
                                        
                                        // Compact info display
                                        ui.horizontal(|ui| {
                                            ui.label("📏");
                                            ui.label(format!("{}x{}px", ldtk_map.world_width, ldtk_map.world_height));
                                            
                                            ui.separator();
                                            ui.label("🔲");
                                            ui.label(format!("Grid: {}px", ldtk_map.default_grid_size));
                                            
                                            ui.separator();
                                            ui.label("📋");
                                            ui.label(format!("{} levels", ldtk_map.levels.len()));
                                        });
                                        
                                        // Additional info
                                        ui.horizontal(|ui| {
                                            ui.label("🎨");
                                            ui.label(format!("{} tilesets", ldtk_map.tilesets.len()));
                                            
                                            ui.separator();
                                            ui.label("🔲");
                                            // let collider_count = world.get_children(entity).len(); // Captured from outside
                                            ui.label(format!("{} colliders", collider_count));
                                        });
                                    });
                                    
                                    ui.add_space(5.0);
                                }
                                
                                // === Actions Section ===
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("🔧 Actions").strong());
                                    ui.separator();
                                    
                                    // Primary actions
                                    ui.horizontal(|ui| {
                                        if ui.button("🔄 Reload")
                                            .on_hover_text("Reload map from file")
                                            .clicked() 
                                        {
                                            reload_requested = true;
                                        }
                                        
                                        if ui.button("🔨 Colliders")
                                            .on_hover_text("Regenerate colliders")
                                            .clicked() 
                                        {
                                            regenerate_colliders_requested = true;
                                        }
                                        
                                        if ui.button("👁 Toggle")
                                            .on_hover_text("Toggle visibility")
                                            .clicked() 
                                        {
                                            toggle_visibility_requested = true;
                                        }
                                    });
                                    
                                    // Secondary actions
                                    ui.horizontal(|ui| {
                                        if ui.small_button("⚙️ Settings")
                                            .on_hover_text("Advanced settings")
                                            .clicked() 
                                        {
                                            // TODO: Open settings dialog
                                        }
                                        
                                        if ui.small_button("📋 Layers")
                                            .on_hover_text("Layer management")
                                            .clicked() 
                                        {
                                            // TODO: Open layer panel
                                        }
                                        
                                        if ui.small_button("🎯 Focus")
                                            .on_hover_text("Focus camera on map")
                                            .clicked() 
                                        {
                                            // TODO: Focus camera
                                        }
                                        
                                        if ui.small_button("🗑️ Clear")
                                            .on_hover_text("Clear map data")
                                            .clicked() 
                                        {
                                            clear_requested = true;
                                        }
                                    });
                                });
                                
                                ui.add_space(5.0);
                                
                                // === Options Section ===
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("⚙️ Options").strong());
                                    ui.separator();
                                    
                                    // Level selection
                                    if !ldtk_map.levels.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.label("Level:");
                                            
                                            let current_level = ldtk_map.current_level.as_deref().unwrap_or("All Levels");
                                            egui::ComboBox::from_id_source(format!("level_select_{}", entity))
                                                .selected_text(current_level)
                                                .width(120.0)
                                                .show_ui(ui, |ui| {
                                                    if ui.selectable_value(&mut ldtk_map.current_level, None, "All Levels").clicked() {
                                                        log::info!("Selected all levels");
                                                    }
                                                    
                                                    for level in &ldtk_map.levels {
                                                        let mut current = ldtk_map.current_level.clone();
                                                        if ui.selectable_value(&mut current, Some(level.identifier.clone()), &level.identifier).clicked() {
                                                            ldtk_map.current_level = current;
                                                            log::info!("Selected level: {}", level.identifier);
                                                        }
                                                    }
                                                });
                                        });
                                    }
                                    
                                    // Auto options
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut ldtk_map.auto_reload, "Auto-reload")
                                            .on_hover_text("Automatically reload when file changes");
                                        
                                        // TODO: Add auto-generate colliders option
                                        let mut auto_colliders = true; // Placeholder
                                        ui.checkbox(&mut auto_colliders, "Auto-colliders")
                                            .on_hover_text("Automatically generate colliders");
                                    });
                                });
                                
                                // Load button for empty maps
                                if ldtk_map.identifier.is_empty() && !file_path.is_empty() {
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(ui.available_width() / 2.0 - 50.0);
                                        if ui.button("🔄 Load Map")
                                            .on_hover_text("Load the selected LDTK file")
                                            .clicked() 
                                        {
                                            load_requested = true;
                                        }
                                    });
                                }
                                if !ldtk_map.identifier.is_empty() {
                                    ui.add_space(5.0);
                                    ui.collapsing("📋 Details", |ui| {
                                        ui.group(|ui| {
                                            ui.label(egui::RichText::new("Map Information").strong());
                                            ui.separator();
                                            
                                            egui::Grid::new("ldtk_details_grid")
                                                .num_columns(2)
                                                .spacing([10.0, 4.0])
                                                .show(ui, |ui| {
                                                    ui.label("Identifier:");
                                                    ui.label(&ldtk_map.identifier);
                                                    ui.end_row();
                                                    
                                                    ui.label("File:");
                                                    ui.label(&ldtk_map.file_path);
                                                    ui.end_row();
                                                    
                                                    ui.label("Background:");
                                                    ui.horizontal(|ui| {
                                                        // Color preview
                                                        if let Ok(color) = parse_hex_color(&ldtk_map.bg_color) {
                                                            let color_rect = ui.allocate_response(
                                                                egui::Vec2::new(16.0, 16.0),
                                                                egui::Sense::hover()
                                                            );
                                                            ui.painter().rect_filled(
                                                                color_rect.rect,
                                                                2.0,
                                                                color
                                                            );
                                                        }
                                                        ui.label(&ldtk_map.bg_color);
                                                    });
                                                    ui.end_row();
                                                });
                                        });
                                        
                                        if !ldtk_map.levels.is_empty() {
                                            ui.add_space(5.0);
                                            ui.group(|ui| {
                                                ui.label(egui::RichText::new("Levels").strong());
                                                ui.separator();
                                                
                                                for (i, level) in ldtk_map.levels.iter().enumerate() {
                                                    ui.horizontal(|ui| {
                                                        ui.label(format!("{}.", i + 1));
                                                        ui.label(egui::RichText::new(&level.identifier).strong());
                                                        ui.label(egui::RichText::new(format!("({}x{}px)", level.px_width, level.px_height)).color(egui::Color32::GRAY));
                                                        
                                                        if level.world_x != 0 || level.world_y != 0 {
                                                            ui.label(egui::RichText::new(format!("at ({}, {})", level.world_x, level.world_y)).color(egui::Color32::GRAY).italics());
                                                        }
                                                    });
                                                }
                                            });
                                        }
                                    });
                                }
                            });
                        }
                        
                        // Handle all requests outside of the borrow
                        if load_requested || reload_requested {
                            if let Some(project_path) = project_path {
                                let full_path = project_path.join(&file_path);
                                
                                // Clear existing children if reloading
                                if reload_requested {
                                    let children: Vec<ecs::Entity> = world.get_children(entity).to_vec();
                                    for child in children {
                                        world.despawn(child);
                                    }
                                }
                                
                                // Add Grid component to current entity if it doesn't have one
                                if !world.grids.contains_key(&entity) {
                                    let grid = ecs::Grid {
                                        cell_size: (0.16, 0.16, 0.0), // Default 16px at 100 PPU
                                        cell_gap: (0.0, 0.0),
                                        layout: ecs::GridLayout::Rectangle,
                                        swizzle: ecs::CellSwizzle::XYZ,
                                        plane: ecs::GridPlane::XY,
                                    };
                                    world.grids.insert(entity, grid);
                                }
                                
                                // Load LDTK file and create tilemap layers as children of this entity
                                match ecs::loaders::LdtkLoader::load_project_with_grid_and_colliders(
                                    &full_path,
                                    world,
                                    true, // auto_generate_colliders
                                    1,    // collision_value
                                ) {
                                    Ok((grid_entity, tilemap_entities, collider_entities)) => {
                                        // Move all children from the created grid to our entity
                                        let children: Vec<ecs::Entity> = world.get_children(grid_entity).to_vec();
                                        for child in children {
                                            world.set_parent(child, Some(entity));
                                        }
                                        
                                        // Copy Grid component from created entity to our entity
                                        if let Some(created_grid) = world.grids.get(&grid_entity).cloned() {
                                            world.grids.insert(entity, created_grid);
                                        }
                                        
                                        // Remove the temporary grid entity
                                        world.despawn(grid_entity);
                                        
                                        // Update LdtkMap component with loaded data
                                        if let Some(ldtk_map) = world.ldtk_maps.get_mut(&entity) {
                                            ldtk_map.file_path = file_path.clone();
                                            if let Ok(content) = std::fs::read_to_string(&full_path) {
                                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                                    ldtk_map.identifier = json["identifier"].as_str().unwrap_or("").to_string();
                                                    ldtk_map.world_width = json["worldGridWidth"].as_i64().unwrap_or(0) as i32;
                                                    ldtk_map.world_height = json["worldGridHeight"].as_i64().unwrap_or(0) as i32;
                                                    ldtk_map.default_grid_size = json["defaultGridSize"].as_i64().unwrap_or(16) as i32;
                                                    ldtk_map.bg_color = json["bgColor"].as_str().unwrap_or("#40465B").to_string();
                                                    
                                                    // Load levels
                                                    if let Some(levels) = json["levels"].as_array() {
                                                        ldtk_map.levels.clear();
                                                        for level_json in levels {
                                                            let level = ecs::LdtkLevel {
                                                                identifier: level_json["identifier"].as_str().unwrap_or("").to_string(),
                                                                world_x: level_json["worldX"].as_i64().unwrap_or(0) as i32,
                                                                world_y: level_json["worldY"].as_i64().unwrap_or(0) as i32,
                                                                px_width: level_json["pxWid"].as_i64().unwrap_or(0) as i32,
                                                                px_height: level_json["pxHei"].as_i64().unwrap_or(0) as i32,
                                                                bg_color: level_json["bgColor"].as_str().map(|s| s.to_string()),
                                                                layers: Vec::new(), // Will be populated by loader
                                                                entities: Vec::new(),
                                                            };
                                                            ldtk_map.levels.push(level);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        log::info!("Successfully {} LDTK map: {} to entity {}", 
                                            if reload_requested { "reloaded" } else { "loaded" },
                                            file_path, entity);
                                        log::info!("Created {} tilemap layers and {} colliders", tilemap_entities.len(), collider_entities.len());
                                    }
                                    Err(e) => {
                                        log::error!("Failed to {} LDTK map: {}", 
                                            if reload_requested { "reload" } else { "load" }, e);
                                    }
                                }
                            }
                        }
                        
                        // Handle regenerate colliders request
                        if regenerate_colliders_requested {
                            let children: Vec<ecs::Entity> = world.get_children(entity).to_vec();
                            let mut collider_count = 0;
                            
                            // Remove existing colliders
                            for child in &children {
                                if world.colliders.contains_key(child) {
                                    world.despawn(*child);
                                    collider_count += 1;
                                }
                            }
                            
                            // Regenerate colliders if we have a loaded map
                            if let Some(ldtk_map) = world.ldtk_maps.get(&entity) {
                                if !ldtk_map.file_path.is_empty() {
                                    if let Some(project_path) = project_path {
                                        let full_path = project_path.join(&ldtk_map.file_path);
                                        
                                        // Generate new colliders
                                        match ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
                                            &full_path,
                                            world,
                                            1, // collision_value
                                        ) {
                                            Ok(new_colliders) => {
                                                // Set new colliders as children of this entity
                                                for &collider in &new_colliders {
                                                    world.set_parent(collider, Some(entity));
                                                }
                                                log::info!("Regenerated {} colliders (removed {})", new_colliders.len(), collider_count);
                                            }
                                            Err(e) => {
                                                log::error!("Failed to regenerate colliders: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Handle toggle visibility request
                        if toggle_visibility_requested {
                            let children: Vec<ecs::Entity> = world.get_children(entity).to_vec();
                            let mut toggled_count = 0;
                            
                            for child in children {
                                if let Some(active) = world.active.get_mut(&child) {
                                    *active = !*active;
                                    toggled_count += 1;
                                }
                            }
                            
                            log::info!("Toggled visibility for {} child entities", toggled_count);
                        }
                        
                        // Handle clear request
                        if clear_requested {
                            // Clear children
                            let children: Vec<ecs::Entity> = world.get_children(entity).to_vec();
                            for child in children {
                                world.despawn(child);
                            }
                            
                            // Clear LdtkMap data
                            if let Some(ldtk_map) = world.ldtk_maps.get_mut(&entity) {
                                *ldtk_map = ecs::LdtkMap::default();
                            }
                            
                            // Remove Grid component
                            world.grids.remove(&entity);
                            
                            log::info!("Cleared LDTK map data and children for entity {}", entity);
                        }
                        
                        ui.add_space(10.0);
                    }
                }
                
                if remove_ldtk_map {
                    let _ = world.remove_component(entity, ComponentType::LdtkMap);
                }

                // TilemapCollider Component
                let has_tilemap_collider = world.has_component(entity, ComponentType::TilemapCollider);
                let mut remove_tilemap_collider = false;
                
                if has_tilemap_collider {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 50, 50))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("🔲");
                                ui.label(egui::RichText::new("Tilemap Collider").strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("❌").clicked() {
                                        remove_tilemap_collider = true;
                                    }
                                });
                            });
                            ui.separator();
                            
                            if let Some(tilemap_collider) = world.tilemap_colliders.get_mut(&entity) {
                                ui.horizontal(|ui| {
                                    ui.label("Mode:");
                                    egui::ComboBox::from_id_source("tilemap_collider_mode")
                                        .selected_text(format!("{:?}", tilemap_collider.mode))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut tilemap_collider.mode, ecs::TilemapColliderMode::Individual, "Individual");
                                            ui.selectable_value(&mut tilemap_collider.mode, ecs::TilemapColliderMode::Composite, "Composite");
                                            ui.selectable_value(&mut tilemap_collider.mode, ecs::TilemapColliderMode::Polygon, "Polygon");
                                            ui.selectable_value(&mut tilemap_collider.mode, ecs::TilemapColliderMode::None, "None");
                                        });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Friction:");
                                    ui.add(egui::DragValue::new(&mut tilemap_collider.friction).speed(0.01).clamp_range(0.0..=1.0));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Restitution:");
                                    ui.add(egui::DragValue::new(&mut tilemap_collider.restitution).speed(0.01).clamp_range(0.0..=1.0));
                                });
                                
                                ui.checkbox(&mut tilemap_collider.use_composite, "Use Composite");
                                ui.checkbox(&mut tilemap_collider.is_trigger, "Is Trigger");
                                ui.checkbox(&mut tilemap_collider.auto_update, "Auto Update");
                            }
                        });
                    ui.add_space(10.0);
                }
                
                if remove_tilemap_collider {
                    let _ = world.remove_component(entity, ComponentType::TilemapCollider);
                }

                // LdtkIntGridCollider Component
                let has_ldtk_intgrid_collider = world.has_component(entity, ComponentType::LdtkIntGridCollider);
                let mut remove_ldtk_intgrid_collider = false;
                
                if has_ldtk_intgrid_collider {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(50, 50, 50))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("🔳");
                                ui.label(egui::RichText::new("LDTK IntGrid Collider").strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("❌").clicked() {
                                        remove_ldtk_intgrid_collider = true;
                                    }
                                });
                            });
                            ui.separator();
                            
                            if let Some(intgrid_collider) = world.ldtk_intgrid_colliders.get_mut(&entity) {
                                ui.horizontal(|ui| {
                                    ui.label("Collision Value:");
                                    ui.add(egui::DragValue::new(&mut intgrid_collider.collision_value).clamp_range(0..=255));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Mode:");
                                    egui::ComboBox::from_id_source("ldtk_intgrid_collider_mode")
                                        .selected_text(format!("{:?}", intgrid_collider.mode))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut intgrid_collider.mode, ecs::TilemapColliderMode::Individual, "Individual");
                                            ui.selectable_value(&mut intgrid_collider.mode, ecs::TilemapColliderMode::Composite, "Composite");
                                            ui.selectable_value(&mut intgrid_collider.mode, ecs::TilemapColliderMode::Polygon, "Polygon");
                                            ui.selectable_value(&mut intgrid_collider.mode, ecs::TilemapColliderMode::None, "None");
                                        });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Friction:");
                                    ui.add(egui::DragValue::new(&mut intgrid_collider.friction).speed(0.01).clamp_range(0.0..=1.0));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Restitution:");
                                    ui.add(egui::DragValue::new(&mut intgrid_collider.restitution).speed(0.01).clamp_range(0.0..=1.0));
                                });
                                
                                ui.checkbox(&mut intgrid_collider.is_trigger, "Is Trigger");
                                ui.checkbox(&mut intgrid_collider.auto_update, "Auto Update");
                            }
                        });
                    ui.add_space(10.0);
                }
                
                if remove_ldtk_intgrid_collider {
                    let _ = world.remove_component(entity, ComponentType::LdtkIntGridCollider);
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
                            let physics_components = vec![ComponentType::BoxCollider, ComponentType::Rigidbody, ComponentType::TilemapCollider, ComponentType::LdtkIntGridCollider];
                            let tilemap_components = vec![ComponentType::LdtkMap];
                            let other_components = vec![ComponentType::Camera, ComponentType::Script, ComponentType::Tag, ComponentType::Map];

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

                            // Tilemap Section
                            let has_tilemap = addable_components.iter().any(|c| tilemap_components.contains(c));
                            if has_tilemap {
                                ui.label("🗺️ Tilemap");
                                ui.separator();
                                for component_type in &tilemap_components {
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
        info.push_str(&format!("  Billboard: {}\n", sprite.billboard));
        if let Some(rect) = sprite.sprite_rect {
            info.push_str(&format!("  Sprite Rect: [{}, {}, {}, {}]\n", rect[0], rect[1], rect[2], rect[3]));
        }
        info.push_str("\n");
    }
    
    // Box Collider
    if let Some(collider) = world.colliders.get(&entity) {
        info.push_str("Box Collider 2D:\n");
        info.push_str(&format!("  Offset: [{:.2}, {:.2}]\n", collider.offset[0], collider.offset[1]));
        info.push_str(&format!("  Size: [{:.2}, {:.2}]\n", collider.size[0], collider.size[1]));
        if let Some(transform) = world.transforms.get(&entity) {
            let world_width = collider.get_world_width(transform.scale[0]);
            let world_height = collider.get_world_height(transform.scale[1]);
            info.push_str(&format!("  World Size: {:.2} x {:.2}\n\n", world_width, world_height));
        } else {
            info.push_str("\n");
        }
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
                } else if value_part.trim_end_matches(',') == "nil" {
                    // Entity reference (Unity-style GameObject)
                    // Pattern: local playerTarget = nil
                    Some(ScriptParameter::Entity(None))
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
