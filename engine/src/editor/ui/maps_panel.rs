use egui::{self, Color32, RichText};
use ecs::World;
use crate::editor::map_manager::MapManager;

/// Render the Maps panel as a standalone window
pub fn render_maps_panel(
    ctx: &egui::Context,
    map_manager: &mut MapManager,
    world: &mut World,
    open: &mut bool,
) {
    egui::Window::new("🗺️ Maps")
        .open(open)
        .default_width(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            render_maps_panel_content(ui, map_manager, world);
        });
}

/// Render the Maps panel content (for use in docking system)
pub fn render_maps_panel_content(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    // Refresh button
    ui.horizontal(|ui| {
        if ui.button("🔄 Refresh").clicked() {
            map_manager.scan_ldtk_files();
        }
    });
    
    ui.separator();
    
    // LDtk Files section
    render_ldtk_files_section(ui, map_manager, world);
    
    ui.separator();
    
    // Loaded Maps section
    render_loaded_maps_section(ui, map_manager, world);
    
    ui.separator();
    
    // Actions section
    render_actions_section(ui, map_manager, world);
    
    ui.separator();
    
    // Statistics section
    render_statistics_section(ui, world);
}

fn render_ldtk_files_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.collapsing(RichText::new("📁 LDtk Files").strong(), |ui| {
        // Add Map button with file dialog
        if ui.button("➕ Add Map").on_hover_text("Browse for .ldtk file").clicked() {
            // Open file dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("LDtk Files", &["ldtk"])
                .pick_file()
            {
                // Add to available files if not already there
                if !map_manager.available_files.contains(&path) {
                    map_manager.available_files.push(path.clone());
                }
                
                // Load the map
                if let Err(e) = map_manager.load_map(&path, world) {
                    log::error!("Failed to load map: {}", e);
                }
            }
        }
        
        ui.separator();
        
        if map_manager.available_files.is_empty() {
            ui.label(RichText::new("No LDtk files found").color(Color32::GRAY));
            ui.label(RichText::new("Click 'Add Map' to browse for files").color(Color32::GRAY).italics());
            return;
        }
        
        // Display available files
        for file_path in map_manager.available_files.clone() {
            ui.horizontal(|ui| {
                // File icon
                ui.label("🗺️");
                
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                // Check if loaded
                let is_loaded = map_manager.loaded_maps.contains_key(&file_path);
                let is_selected = map_manager.selected_map.as_ref() == Some(&file_path);
                
                // Selectable file with load status indicator
                let text = if is_loaded {
                    RichText::new(format!("{} ✓", file_name)).color(Color32::from_rgb(100, 200, 100))
                } else {
                    RichText::new(file_name)
                };
                
                if ui.selectable_label(is_selected, text).clicked() {
                    map_manager.selected_map = Some(file_path.clone());
                    
                    // Load if not loaded
                    if !is_loaded {
                        if let Err(e) = map_manager.load_map(&file_path, world) {
                            log::error!("Failed to load map: {}", e);
                        }
                    }
                }
                
                // Action buttons
                if is_loaded {
                    // Reload button
                    if ui.small_button("↻").on_hover_text("Reload").clicked() {
                        if let Err(e) = map_manager.reload_map(&file_path, world) {
                            log::error!("Failed to reload map: {}", e);
                        }
                    }
                    
                    // Unload button
                    if ui.small_button("✖").on_hover_text("Unload").clicked() {
                        map_manager.unload_map(&file_path, world);
                    }
                } else {
                    // Load button for unloaded files
                    if ui.small_button("📂").on_hover_text("Load").clicked() {
                        if let Err(e) = map_manager.load_map(&file_path, world) {
                            log::error!("Failed to load map: {}", e);
                        }
                    }
                }
            });
        }
    });
}

fn render_loaded_maps_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    for (file_path, loaded_map) in map_manager.loaded_maps.clone() {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        
        ui.collapsing(RichText::new(format!("🎯 {}", file_name)).strong(), |ui| {
            // Info
            ui.label(format!("Loaded Tilemaps: {}", loaded_map.layer_entities.len()));
            
            ui.separator();
            
            // Grid entity
            if let Some(grid_name) = world.names.get(&loaded_map.grid_entity) {
                ui.collapsing(format!("📐 {}", grid_name), |ui| {
                    // Grid info
                    if let Some(grid) = world.grids.get(&loaded_map.grid_entity) {
                        ui.label(format!("Cell Size: {:.2} x {:.2}", 
                            grid.cell_size.0, grid.cell_size.1));
                        ui.label(format!("Layout: {:?}", grid.layout));
                    }
                    
                    ui.separator();
                    
                    // Layers
                    for layer in &loaded_map.layer_entities {
                        render_layer_item(ui, layer, map_manager, world);
                    }
                });
            }
        });
    }
}

fn render_layer_item(
    ui: &mut egui::Ui,
    layer: &crate::editor::map_manager::LayerInfo,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.horizontal(|ui| {
        // Layer icon
        ui.label("🎨");
        
        // Layer name and size
        let layer_text = format!("{} ({}x{})", 
            layer.name.replace("LDTK Layer: ", ""),
            layer.size.0, 
            layer.size.1
        );
        
        let text = if layer.visible {
            RichText::new(layer_text)
        } else {
            RichText::new(layer_text).color(Color32::GRAY)
        };
        
        // Make layer selectable (for Layer Properties Panel)
        if ui.selectable_label(false, text).clicked() {
            // Layer selection would be handled by the Layer Properties Panel
            // For now, just log it
            log::info!("Selected layer: {}", layer.name);
        }
        
        // Visibility toggle
        let icon = if layer.visible { "👁" } else { "👁‍🗨" };
        if ui.small_button(icon)
            .on_hover_text(if layer.visible { "Hide" } else { "Show" })
            .clicked() 
        {
            map_manager.toggle_layer_visibility(layer.entity, world);
        }
    });
}

fn render_actions_section(
    ui: &mut egui::Ui,
    map_manager: &mut MapManager,
    world: &mut World,
) {
    ui.collapsing(RichText::new("⚙️ Actions").strong(), |ui| {
        let has_selection = map_manager.selected_map.is_some();
        
        // Reload Map (with auto-generated colliders)
        ui.add_enabled_ui(has_selection, |ui| {
            if ui.button("🔄 Reload Map")
                .on_hover_text("Reload map with auto-generated colliders")
                .clicked() 
            {
                if let Some(path) = &map_manager.selected_map.clone() {
                    if let Err(e) = map_manager.reload_map(path, world) {
                        log::error!("Failed to reload map: {}", e);
                    }
                }
            }
        });
        
        // Regenerate Colliders (replaces Generate + Clean Up)
        ui.add_enabled_ui(has_selection, |ui| {
            if ui.button("🔨 Regenerate Colliders")
                .on_hover_text("Remove old colliders and generate new ones")
                .clicked() 
            {
                if let Some(path) = &map_manager.selected_map.clone() {
                    match map_manager.regenerate_colliders(path, world) {
                        Ok(count) => {
                            log::info!("Regenerated {} colliders", count);
                        }
                        Err(e) => {
                            log::error!("Failed to regenerate colliders: {}", e);
                        }
                    }
                }
            }
        });
        
        ui.separator();
        
        // Clean Up Colliders (for selected map)
        ui.add_enabled_ui(has_selection, |ui| {
            let selected_path = map_manager.selected_map.clone();
            if let Some(path) = selected_path {
                let collider_count = map_manager.loaded_maps.get(&path)
                    .map(|m| m.collider_entities.len())
                    .unwrap_or(0);
                
                if ui.button(format!("🧹 Clean Up Colliders ({})", collider_count))
                    .on_hover_text("Remove all colliders for this map")
                    .clicked() 
                {
                    let removed = map_manager.clean_up_colliders(&path, world);
                    log::info!("Removed {} colliders", removed);
                }
            }
        });
        
        // Clean Up All Colliders
        let total_colliders = map_manager.count_colliders(world);
        if ui.button(format!("🧹 Clean Up All ({})", total_colliders))
            .on_hover_text("Remove all colliders from all maps")
            .clicked() 
        {
            let removed = map_manager.clean_up_all_colliders(world);
            log::info!("Removed {} colliders from all maps", removed);
        }
    });
}

fn render_statistics_section(
    ui: &mut egui::Ui,
    world: &World,
) {
    ui.collapsing(RichText::new("📊 Statistics").strong(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Entities:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.transforms.len())).strong());
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Tilemaps:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.tilemaps.len())).strong());
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Colliders:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.colliders.len())).strong());
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Sprites:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.sprites.len())).strong());
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Grids:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.grids.len())).strong());
            });
        });
    });
}
