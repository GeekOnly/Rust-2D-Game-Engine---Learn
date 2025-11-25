use ecs::{World, Entity, Sprite, Collider, EntityTag, Script, Prefab};
use egui;
use std::collections::HashMap;
use crate::console::Console;

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
        scene_view_tab: &mut usize,
        is_playing: bool,
        show_colliders: &mut bool,
        show_velocities: &mut bool,
        console: &mut Console,
        bottom_panel_tab: &mut usize,
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
                let entities: Vec<Entity> = entity_names.keys().cloned().collect();

                for entity in entities {
                    let name = entity_names.get(&entity).cloned().unwrap_or(format!("Entity {}", entity));
                    let is_selected = *selected_entity == Some(entity);

                    if ui.selectable_label(is_selected, &name).clicked() {
                        *selected_entity = Some(entity);
                    }
                }
            });

            ui.separator();
            if ui.button("➕ Create Empty GameObject").clicked() {
                let entity = Prefab::new("GameObject").spawn(world);
                entity_names.insert(entity, format!("GameObject {}", entity));
            }
        });

        // Inspector Panel (Right)
        egui::SidePanel::right("inspector").min_width(300.0).show(ctx, |ui| {
            ui.heading("🔧 Inspector");
            ui.separator();

            if let Some(entity) = *selected_entity {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Entity name
                    if let Some(name) = entity_names.get_mut(&entity) {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(name);
                        });
                    }

                    ui.add_space(10.0);

                    // Transform Component
                    if let Some(transform) = world.transforms.get_mut(&entity) {
                        ui.collapsing("Transform", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Position X:");
                                ui.add(egui::DragValue::new(&mut transform.position[0]).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Position Y:");
                                ui.add(egui::DragValue::new(&mut transform.position[1]).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Position Z:");
                                ui.add(egui::DragValue::new(&mut transform.position[2]).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Rotation X:");
                                ui.add(egui::DragValue::new(&mut transform.rotation[0]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Rotation Y:");
                                ui.add(egui::DragValue::new(&mut transform.rotation[1]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Rotation Z:");
                                ui.add(egui::DragValue::new(&mut transform.rotation[2]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Scale X:");
                                ui.add(egui::DragValue::new(&mut transform.scale[0]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Scale Y:");
                                ui.add(egui::DragValue::new(&mut transform.scale[1]).speed(0.1));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Scale Z:");
                                ui.add(egui::DragValue::new(&mut transform.scale[2]).speed(0.1));
                            });
                        });
                    }

                    // Sprite Component
                    let has_sprite = world.sprites.contains_key(&entity);
                    ui.collapsing("Sprite Renderer", |ui| {
                        if has_sprite {
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
                        } else {
                            if ui.button("Add Sprite Renderer").clicked() {
                                world.sprites.insert(entity, Sprite {
                                    texture_id: "sprite".to_string(),
                                    width: 32.0,
                                    height: 32.0,
                                    color: [1.0, 1.0, 1.0, 1.0],
                                });
                            }
                        }
                    });

                    // Collider Component
                    let has_collider = world.colliders.contains_key(&entity);
                    ui.collapsing("Box Collider", |ui| {
                        if has_collider {
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
                        } else {
                            if ui.button("Add Box Collider").clicked() {
                                world.colliders.insert(entity, Collider {
                                    width: 32.0,
                                    height: 32.0,
                                });
                            }
                        }
                    });

                    // Tag Component
                    let has_tag = world.tags.contains_key(&entity);
                    ui.collapsing("Tag", |ui| {
                        if has_tag {
                            if let Some(tag) = world.tags.get_mut(&entity) {
                                let mut tag_index = match tag {
                                    EntityTag::Player => 0,
                                    EntityTag::Item => 1,
                                };

                                egui::ComboBox::from_label("Tag Type")
                                    .selected_text(format!("{:?}", tag))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut tag_index, 0, "Player");
                                        ui.selectable_value(&mut tag_index, 1, "Item");
                                    });

                                *tag = match tag_index {
                                    0 => EntityTag::Player,
                                    1 => EntityTag::Item,
                                    _ => EntityTag::Player,
                                };

                                if ui.button("Remove Component").clicked() {
                                    world.tags.remove(&entity);
                                }
                            }
                        } else {
                            if ui.button("Add Tag").clicked() {
                                world.tags.insert(entity, EntityTag::Player);
                            }
                        }
                    });

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

                        // TRANSFORM GIZMO: Draw move handles for selected entity
                        if *selected_entity == Some(entity) {
                            let gizmo_size = 50.0;
                            let handle_size = 8.0;

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
                            let gizmo_size = 50.0;
                            let handle_size = 8.0;

                            // Check which handle is being hovered
                            let x_handle_pos = egui::pos2(screen_x + gizmo_size, screen_y);
                            let y_handle_pos = egui::pos2(screen_x, screen_y + gizmo_size);
                            let center_handle_pos = egui::pos2(screen_x, screen_y);

                            // Determine drag axis on hover
                            let mut drag_axis = None;
                            if let Some(hover_pos) = response.hover_pos() {
                                let dist_to_x = hover_pos.distance(x_handle_pos);
                                let dist_to_y = hover_pos.distance(y_handle_pos);
                                let dist_to_center = hover_pos.distance(center_handle_pos);

                                if dist_to_center < handle_size * 1.5 {
                                    drag_axis = Some(2); // Both axes
                                } else if dist_to_x < handle_size * 1.5 {
                                    drag_axis = Some(0); // X axis
                                } else if dist_to_y < handle_size * 1.5 {
                                    drag_axis = Some(1); // Y axis
                                }
                            }

                            // Handle dragging - use absolute mouse position for smooth tracking
                            if response.dragged() && drag_axis.is_some() {
                                if let Some(interact_pos) = response.interact_pointer_pos() {
                                    // Convert screen to world coordinates
                                    let world_x = interact_pos.x - center_x;
                                    let world_y = interact_pos.y - center_y;

                                    if let Some(transform) = world.transforms.get_mut(&sel_entity) {
                                        match drag_axis.unwrap() {
                                            0 => {
                                                // X axis only - lock Y
                                                transform.set_x(world_x);
                                            }
                                            1 => {
                                                // Y axis only - lock X
                                                transform.set_y(world_y);
                                            }
                                            2 => {
                                                // Both axes - free movement
                                                transform.set_x(world_x);
                                                transform.set_y(world_y);
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
    }
}
