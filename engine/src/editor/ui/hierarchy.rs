use ecs::{World, Entity, Prefab};
use egui;
use std::collections::HashMap;
use crate::editor::Console;

/// Render the hierarchy panel (left panel) showing scene entities
pub fn render_hierarchy(
    ui: &mut egui::Ui,
    world: &mut World,
    entity_names: &mut HashMap<Entity, String>,
    selected_entity: &mut Option<Entity>,
    load_file_request: &mut Option<std::path::PathBuf>,
    project_path: &Option<std::path::PathBuf>,
    current_scene_path: &Option<std::path::PathBuf>,
    console: &mut Console,
    get_scene_files_fn: impl Fn(&std::path::Path) -> Vec<String>,
    get_entity_icon_fn: &impl Fn(&World, Entity) -> &'static str,
) {
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
                // Track entity to delete (for right-click menu)
                let mut entity_to_delete: Option<Entity> = None;
                let mut entity_to_create_child: Option<Entity> = None;

                // Collect roots (entities with no parent)
                let mut roots: Vec<Entity> = entity_names.keys()
                    .filter(|&e| world.parents.get(e).is_none())
                    .cloned()
                    .collect();

                // Sort by ID for stability
                roots.sort();

                for root in roots {
                    draw_entity_node(
                        ui,
                        root,
                        world,
                        entity_names,
                        selected_entity,
                        &mut entity_to_delete,
                        &mut entity_to_create_child,
                        get_entity_icon_fn,
                    );
                }

                // Handle creation
                if let Some(parent) = entity_to_create_child {
                    let child = Prefab::new("GameObject").spawn(world);
                    world.set_parent(child, Some(parent));
                    entity_names.insert(child, format!("GameObject {}", child));

                    // Select the new child
                    *selected_entity = Some(child);
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
            let scene_files = get_scene_files_fn(proj_path);

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
}

/// Recursively draw entity node in hierarchy with children
pub fn draw_entity_node(
    ui: &mut egui::Ui,
    entity: Entity,
    world: &World,
    entity_names: &HashMap<Entity, String>,
    selected_entity: &mut Option<Entity>,
    entity_to_delete: &mut Option<Entity>,
    entity_to_create_child: &mut Option<Entity>,
    get_entity_icon_fn: &impl Fn(&World, Entity) -> &'static str,
) {
    let name = entity_names.get(&entity).cloned().unwrap_or(format!("Entity {}", entity));
    let is_selected = *selected_entity == Some(entity);
    let icon = get_entity_icon_fn(world, entity);
    let children = world.get_children(entity);
    let has_children = !children.is_empty();

    let id = ui.make_persistent_id(entity);

    if has_children {
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    let response = ui.selectable_label(is_selected, format!("{} {}", icon, name));
                    if response.clicked() {
                        *selected_entity = Some(entity);
                    }

                    // Context Menu
                    response.context_menu(|ui| {
                        ui.label(format!("📝 {}", name));
                        ui.separator();
                        if ui.button("➕ Create Child Empty").clicked() {
                            *entity_to_create_child = Some(entity);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("❌ Delete").clicked() {
                            *entity_to_delete = Some(entity);
                            ui.close_menu();
                        }
                    });
                });
            })
            .body(|ui| {
                for &child in children {
                    draw_entity_node(ui, child, world, entity_names, selected_entity, entity_to_delete, entity_to_create_child, get_entity_icon_fn);
                }
            });
    } else {
        // Leaf node
        ui.horizontal(|ui| {
            // Indent to match collapsing header text (approx 15-20px)
            ui.add_space(20.0);
            let response = ui.selectable_label(is_selected, format!("{} {}", icon, name));
            if response.clicked() {
                *selected_entity = Some(entity);
            }

            // Context Menu
            response.context_menu(|ui| {
                ui.label(format!("📝 {}", name));
                ui.separator();
                if ui.button("➕ Create Child Empty").clicked() {
                    *entity_to_create_child = Some(entity);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("❌ Delete").clicked() {
                    *entity_to_delete = Some(entity);
                    ui.close_menu();
                }
            });
        });
    }
}
