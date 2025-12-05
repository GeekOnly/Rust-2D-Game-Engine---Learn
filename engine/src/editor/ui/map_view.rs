/// Map View Panel - Dedicated panel for LDtk map management
/// Shows list of maps, layers, and provides quick actions

use egui;
use ecs::World;
use std::path::PathBuf;

pub struct MapViewState {
    pub selected_map: Option<String>,
    pub show_layers: bool,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            selected_map: None,
            show_layers: true,
        }
    }
}

pub fn render_map_view(
    ui: &mut egui::Ui,
    world: &mut World,
    project_path: &Option<PathBuf>,
    state: &mut MapViewState,
    console: &mut crate::editor::Console,
) {
    // Header with icon
    ui.horizontal(|ui| {
        ui.heading("🗺️ Maps");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("🔄").on_hover_text("Refresh").clicked() {
                log::info!("Refresh maps");
            }
        });
    });
    
    ui.separator();

    if project_path.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("No project loaded").color(egui::Color32::GRAY));
        });
        return;
    }

    // List all LDtk files in project
    if let Some(proj_path) = project_path {
        let levels_path = proj_path.join("levels");
        
        if !levels_path.exists() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("No 'levels' folder found").color(egui::Color32::GRAY));
                ui.add_space(5.0);
                ui.label(egui::RichText::new("Create a 'levels' folder and add .ldtk files").small().color(egui::Color32::DARK_GRAY));
            });
            return;
        }

        // LDtk Files Section
        egui::CollapsingHeader::new("📁 LDtk Files")
            .default_open(true)
            .show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(&levels_path) {
                    let mut found_any = false;
                    
                    for entry in entries.flatten() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "ldtk" {
                                found_any = true;
                                let file_name = entry.file_name().to_string_lossy().to_string();
                                let is_selected = state.selected_map.as_ref() == Some(&file_name);
                                
                                let response = ui.selectable_label(is_selected, format!("📄 {}", file_name));
                                
                                if response.clicked() {
                                    state.selected_map = Some(file_name.clone());
                                }
                                
                                // Context menu
                                response.context_menu(|ui| {
                                    if ui.button("🔄 Reload").clicked() {
                                        log::info!("Reload: {}", file_name);
                                        ui.close_menu();
                                    }
                                    if ui.button("📂 Open in LDtk").clicked() {
                                        let path = levels_path.join(&file_name);
                                        let _ = open::that(path);
                                        ui.close_menu();
                                    }
                                });
                            }
                        }
                    }
                    
                    if !found_any {
                        ui.label(egui::RichText::new("No .ldtk files found").color(egui::Color32::GRAY).italics());
                    }
                } else {
                    ui.label(egui::RichText::new("Cannot read levels folder").color(egui::Color32::RED));
                }
            });

        ui.add_space(5.0);

        // Map Details Section
        if let Some(map_name) = &state.selected_map {
            ui.separator();
            
            egui::CollapsingHeader::new(format!("📋 {}", map_name))
                .default_open(true)
                .show(ui, |ui| {
                    // Find tilemap entities for this map
                    let tilemap_entities: Vec<_> = world.tilemaps.keys().copied().collect();
                    
                    if tilemap_entities.is_empty() {
                        ui.label(egui::RichText::new("No tilemaps loaded in scene").color(egui::Color32::GRAY).italics());
                    } else {
                        ui.label(egui::RichText::new(format!("Loaded Tilemaps: {}", tilemap_entities.len())).strong());
                        ui.add_space(5.0);
                        
                        for entity in &tilemap_entities {
                            if let Some(tilemap) = world.tilemaps.get(entity) {
                                ui.horizontal(|ui| {
                                    ui.label("📐");
                                    ui.label(format!("{}x{}", tilemap.width, tilemap.height));
                                    ui.label("-");
                                    ui.label(&tilemap.name);
                                });
                            }
                        }
                    }
                });

            ui.add_space(5.0);
            ui.separator();

            // Actions Section
            egui::CollapsingHeader::new("⚙️ Actions")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add_space(5.0);
                    
                    // Reload button
                    if ui.button("🔄 Reload Map").clicked() {
                        if let Some(proj_path) = project_path {
                            let map_path = proj_path.join("levels").join(map_name);
                            
                            if map_path.exists() {
                                match ecs::loaders::LdtkLoader::load_project(&map_path, world) {
                                    Ok(entities) => {
                                        console.info(format!("✅ Reloaded map: {} ({} entities)", map_name, entities.len()));
                                        log::info!("Reloaded map: {} ({} entities)", map_name, entities.len());
                                    }
                                    Err(e) => {
                                        console.error(format!("❌ Failed to reload map: {}", e));
                                        log::error!("Failed to reload map: {}", e);
                                    }
                                }
                            } else {
                                console.error(format!("❌ Map file not found: {}", map_name));
                            }
                        }
                    }
                    ui.add_space(3.0);

                    // Generate Colliders button
                    if ui.button("🔧 Generate Colliders").clicked() {
                        if let Some(proj_path) = project_path {
                            let map_path = proj_path.join("levels").join(map_name);
                            
                            if map_path.exists() {
                                match ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(&map_path, world, 1) {
                                    Ok(collider_entities) => {
                                        console.info(format!("✅ Generated {} composite colliders", collider_entities.len()));
                                        log::info!("Generated {} composite colliders", collider_entities.len());
                                    }
                                    Err(e) => {
                                        console.error(format!("❌ Failed to generate colliders: {}", e));
                                        log::error!("Failed to generate colliders: {}", e);
                                    }
                                }
                            } else {
                                console.error(format!("❌ Map file not found: {}", map_name));
                            }
                        }
                    }
                    ui.add_space(3.0);

                    // Clean Up Colliders button
                    let collider_count = world.colliders.keys()
                        .filter(|e| {
                            world.names.get(e)
                                .map(|name| name.contains("Collider"))
                                .unwrap_or(false)
                        })
                        .count();
                    
                    let button_text = if collider_count > 0 {
                        format!("🧹 Clean Up Colliders ({})", collider_count)
                    } else {
                        "🧹 Clean Up Colliders".to_string()
                    };
                    
                    if ui.button(button_text).clicked() {
                        // Remove all collider entities
                        let collider_entities: Vec<_> = world.colliders.keys()
                            .filter(|e| {
                                world.names.get(e)
                                    .map(|name| name.contains("Collider"))
                                    .unwrap_or(false)
                            })
                            .copied()
                            .collect();
                        
                        let count = collider_entities.len();
                        for entity in collider_entities {
                            world.despawn(entity);
                        }
                        
                        log::info!("Cleaned up {} collider entities", count);
                    }
                    
                    ui.add_space(5.0);
                });

            ui.add_space(5.0);
            ui.separator();

            // Statistics Section
            egui::CollapsingHeader::new("📊 Statistics")
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(5.0);
                    
                    let entity_count = world.transforms.len();
                    let tilemap_count = world.tilemaps.len();
                    let collider_count = world.colliders.len();
                    let sprite_count = world.sprites.len();
                    
                    ui.horizontal(|ui| {
                        ui.label("Entities:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(entity_count.to_string());
                        });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Tilemaps:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(tilemap_count.to_string());
                        });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Colliders:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(collider_count.to_string());
                        });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Sprites:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(sprite_count.to_string());
                        });
                    });
                    
                    ui.add_space(5.0);
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Select a map to view details").color(egui::Color32::GRAY).italics());
            });
        }
    }
}
