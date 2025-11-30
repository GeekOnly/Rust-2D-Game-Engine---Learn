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
    _load_file_request: &mut Option<std::path::PathBuf>,
    _project_path: &Option<std::path::PathBuf>,
    current_scene_path: &Option<std::path::PathBuf>,
    _console: &mut Console,
    _get_scene_files_fn: impl Fn(&std::path::Path) -> Vec<String>,
    get_entity_icon_fn: &impl Fn(&World, Entity) -> &'static str,
) {
    // Unity-style header with title and icons
    ui.horizontal(|ui| {
        ui.heading("Hierarchy");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Lock icon (placeholder for future functionality)
            if ui.button("🔓").on_hover_text("Lock/Unlock Hierarchy").clicked() {
                // TODO: Implement lock functionality
            }
            // Options menu
            ui.menu_button("⋮", |ui| {
                if ui.button("Sort by Name").clicked() {
                    ui.close_menu();
                }
                if ui.button("Sort by Type").clicked() {
                    ui.close_menu();
                }
            });
        });
    });
    
    ui.separator();
    
    // Unity-style toolbar with Create button and Search
    ui.horizontal(|ui| {
        // Create dropdown button (Unity style)
        ui.menu_button("➕", |ui| {
            ui.label("Create");
            ui.separator();
            
            if ui.button("📦 Empty GameObject").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                entity_names.insert(entity, "GameObject".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            ui.separator();
            ui.label("2D Objects");
            
            if ui.button("🎮 Sprite").clicked() {
                let entity = world.spawn();
                // Create transform with scale matching sprite size (32x32)
                let mut transform = ecs::Transform::default();
                transform.scale = [32.0, 32.0, 1.0];
                world.transforms.insert(entity, transform);
                world.sprites.insert(entity, ecs::Sprite {
                    texture_id: "sprite".to_string(),
                    width: 1.0,  // Base size, actual size determined by transform.scale
                    height: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    billboard: false, // Default sprite, not billboard
                });
                entity_names.insert(entity, "Sprite".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            if ui.button("📷 Camera").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.cameras.insert(entity, ecs::Camera::default());
                entity_names.insert(entity, "Camera".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            ui.separator();
            ui.label("3D Objects");
            
            if ui.button("🧊 Cube").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.meshes.insert(entity, ecs::Mesh {
                    mesh_type: ecs::MeshType::Cube,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                entity_names.insert(entity, "Cube".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            if ui.button("⚪ Sphere").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.meshes.insert(entity, ecs::Mesh {
                    mesh_type: ecs::MeshType::Sphere,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                entity_names.insert(entity, "Sphere".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            if ui.button("🔷 Cylinder").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.meshes.insert(entity, ecs::Mesh {
                    mesh_type: ecs::MeshType::Cylinder,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                entity_names.insert(entity, "Cylinder".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            if ui.button("▭ Plane").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.meshes.insert(entity, ecs::Mesh {
                    mesh_type: ecs::MeshType::Plane,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                entity_names.insert(entity, "Plane".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            if ui.button("💊 Capsule").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                world.meshes.insert(entity, ecs::Mesh {
                    mesh_type: ecs::MeshType::Capsule,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
                entity_names.insert(entity, "Capsule".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
            
            ui.separator();
            ui.label("Light");
            
            if ui.button("💡 Directional Light").clicked() {
                let entity = world.spawn();
                world.transforms.insert(entity, ecs::Transform::default());
                entity_names.insert(entity, "Directional Light".to_string());
                *selected_entity = Some(entity);
                ui.close_menu();
            }
        });
        
        // Search box (Unity style)
        ui.add(
            egui::TextEdit::singleline(&mut String::new())
                .hint_text("🔍 Search")
                .desired_width(ui.available_width())
        );
    });
    
    ui.separator();

    // Main hierarchy scroll area
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Track entity to delete (for right-click menu)
        let mut entity_to_delete: Option<Entity> = None;
        let mut entity_to_create_child: Option<Entity> = None;

        // Scene root node (Unity style - always visible)
        let scene_name = if let Some(path) = current_scene_path {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            "Untitled Scene".to_string()
        };

        // Unity-style scene header (collapsible but default open)
        let scene_id = ui.make_persistent_id("scene_root");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), scene_id, true)
            .show_header(ui, |ui| {
                ui.label(format!("🎬 {}", scene_name));
            })
            .body(|ui| {
                // Collect roots (entities with no parent)
                let mut roots: Vec<Entity> = entity_names.keys()
                    .filter(|&e| world.parents.get(e).is_none())
                    .cloned()
                    .collect();

                // Sort by ID for stability
                roots.sort();

                // Draw all root entities
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
            });

        // Handle creation
        if let Some(parent) = entity_to_create_child {
            let child = world.spawn();
            world.transforms.insert(child, ecs::Transform::default());
            world.set_parent(child, Some(parent));
            entity_names.insert(child, "GameObject".to_string());
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

}

/// Recursively draw entity node in hierarchy with children (Unity style)
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
        // Unity-style parent node with arrow
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                let response = ui.selectable_label(is_selected, format!("{} {}", icon, name));
                
                if response.clicked() {
                    *selected_entity = Some(entity);
                }

                // Unity-style context menu
                response.context_menu(|ui| {
                    if ui.button("Create Empty Child").clicked() {
                        *entity_to_create_child = Some(entity);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Copy").clicked() {
                        // TODO: Implement copy
                        ui.close_menu();
                    }
                    
                    if ui.button("Paste").clicked() {
                        // TODO: Implement paste
                        ui.close_menu();
                    }
                    
                    if ui.button("Duplicate").clicked() {
                        // TODO: Implement duplicate
                        ui.close_menu();
                    }
                    
                    if ui.button("Rename").clicked() {
                        // TODO: Implement rename
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Delete").clicked() {
                        *entity_to_delete = Some(entity);
                        ui.close_menu();
                    }
                });
            })
            .body(|ui| {
                // Draw children with proper indentation
                for &child in children {
                    draw_entity_node(ui, child, world, entity_names, selected_entity, entity_to_delete, entity_to_create_child, get_entity_icon_fn);
                }
            });
    } else {
        // Unity-style leaf node (no arrow, just icon and name)
        ui.horizontal(|ui| {
            // Add spacing to align with parent nodes
            ui.add_space(18.0);
            
            let response = ui.selectable_label(is_selected, format!("{} {}", icon, name));
            
            if response.clicked() {
                *selected_entity = Some(entity);
            }

            // Unity-style context menu
            response.context_menu(|ui| {
                if ui.button("Create Empty Child").clicked() {
                    *entity_to_create_child = Some(entity);
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Copy").clicked() {
                    // TODO: Implement copy
                    ui.close_menu();
                }
                
                if ui.button("Paste").clicked() {
                    // TODO: Implement paste
                    ui.close_menu();
                }
                
                if ui.button("Duplicate").clicked() {
                    // TODO: Implement duplicate
                    ui.close_menu();
                }
                
                if ui.button("Rename").clicked() {
                    // TODO: Implement rename
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("Delete").clicked() {
                    *entity_to_delete = Some(entity);
                    ui.close_menu();
                }
            });
        });
    }
}
