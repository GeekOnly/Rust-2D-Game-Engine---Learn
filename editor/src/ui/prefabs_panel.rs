use egui;
use ecs::World;
use std::collections::HashMap;
use crate::prefab::PrefabManager;

/// Render the Prefabs panel
pub fn render_prefabs_panel(
    ui: &mut egui::Ui,
    prefab_manager: &mut PrefabManager,
    world: &mut World,
    entity_names: &mut HashMap<ecs::Entity, String>,
    selected_entity: &mut Option<ecs::Entity>,
) {
    // Header
    ui.horizontal(|ui| {
        ui.heading("📦 Prefabs");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🔄").on_hover_text("Refresh").clicked() {
                prefab_manager.scan_prefabs();
            }
            
            if ui.button("❓").on_hover_text("Help").clicked() {
                // Show help
            }
        });
    });
    
    ui.separator();
    
    // Help section (collapsible)
    egui::CollapsingHeader::new("ℹ️ Help")
        .default_open(false)
        .show(ui, |ui| {
            ui.label("Prefabs are reusable templates for entities.");
            ui.label("");
            ui.label("To create a prefab:");
            ui.label("1. Right-click an entity in Hierarchy");
            ui.label("2. Select 'Create Prefab'");
            ui.label("3. Enter a name and save");
            ui.label("");
            ui.label("To use a prefab:");
            ui.label("1. Click on a prefab below");
            ui.label("2. Click 'Instantiate' to add to scene");
        });
    
    ui.separator();
    
    // Statistics
    ui.horizontal(|ui| {
        ui.label(format!("📊 Total: {}", prefab_manager.available_files.len()));
        ui.label(format!("💾 Loaded: {}", prefab_manager.prefabs.len()));
    });
    
    ui.separator();
    
    // Prefabs list
    egui::ScrollArea::vertical().show(ui, |ui| {
        if prefab_manager.available_files.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("No prefabs found");
                ui.label("Create one by right-clicking an entity");
            });
        } else {
            for prefab_path in prefab_manager.available_files.clone() {
                let file_name = prefab_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");
                
                let is_selected = prefab_manager.selected_prefab.as_ref() == Some(&prefab_path);
                let is_loaded = prefab_manager.prefabs.contains_key(&prefab_path);
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Icon
                        ui.label("📦");
                        
                        // Name (selectable)
                        if ui.selectable_label(is_selected, file_name).clicked() {
                            prefab_manager.selected_prefab = Some(prefab_path.clone());
                            
                            // Auto-load if not loaded
                            if !is_loaded {
                                if let Err(e) = prefab_manager.load_prefab(&prefab_path) {
                                    log::error!("Failed to load prefab: {}", e);
                                }
                            }
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete button
                            if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                if let Err(e) = prefab_manager.delete_prefab(&prefab_path) {
                                    log::error!("Failed to delete prefab: {}", e);
                                }
                            }
                            
                            // Instantiate button
                            if ui.small_button("➕").on_hover_text("Instantiate").clicked() {
                                // Load if not loaded
                                if !is_loaded {
                                    if let Err(e) = prefab_manager.load_prefab(&prefab_path) {
                                        log::error!("Failed to load prefab: {}", e);
                                        return;
                                    }
                                }
                                
                                // Instantiate
                                match prefab_manager.instantiate_prefab(
                                    &prefab_path,
                                    world,
                                    entity_names,
                                    None, // No parent
                                ) {
                                    Ok(entity) => {
                                        *selected_entity = Some(entity);
                                        log::info!("Instantiated prefab: {}", file_name);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to instantiate prefab: {}", e);
                                    }
                                }
                            }
                        });
                    });
                    
                    // Show details if selected and loaded
                    if is_selected && is_loaded {
                        if let Some(prefab) = prefab_manager.prefabs.get(&prefab_path) {
                            ui.separator();
                            ui.label(format!("Name: {}", prefab.name));
                            ui.label(format!("Root: {}", prefab.root.name));
                            ui.label(format!("Children: {}", count_children(&prefab.root)));
                            ui.label(format!("Version: {}", prefab.metadata.version));
                            
                            if !prefab.metadata.tags.is_empty() {
                                ui.label(format!("Tags: {}", prefab.metadata.tags.join(", ")));
                            }
                        }
                    }
                });
                
                ui.add_space(4.0);
            }
        }
    });
}

/// Count total children recursively
fn count_children(entity: &crate::prefab::PrefabEntity) -> usize {
    let mut count = entity.children.len();
    for child in &entity.children {
        count += count_children(child);
    }
    count
}
