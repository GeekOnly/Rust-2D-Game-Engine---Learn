use egui::{self, Color32, RichText};
use ecs::{World, LdtkMap, TilemapCollider, LdtkIntGridCollider};
use crate::map_manager::MapManager;

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
    // Refresh button and help
    ui.horizontal(|ui| {
        if ui.button("🔄 Refresh")
            .on_hover_text("Scan project directory for .ldtk files")
            .clicked() 
        {
            map_manager.scan_ldtk_files();
        }
        
        // Help button
        if ui.button("❓")
            .on_hover_text("Show help")
            .clicked() 
        {
            // Help will be shown in a collapsing section below
        }
    });
    
    // Help section
    ui.collapsing("ℹ️ Help", |ui| {
        ui.label(RichText::new("Tilemap Management Workflow:").strong());
        ui.separator();
        
        ui.label("1. Click 'Add Map' to browse for .ldtk files");
        ui.label("2. Select a file to load it into the scene");
        ui.label("3. Use visibility toggles to show/hide layers");
        ui.label("4. Colliders are auto-generated from IntGrid layers");
        ui.label("5. Use 'Reload' to refresh after editing in LDtk");
        
        ui.separator();
        ui.label(RichText::new("Keyboard Shortcuts:").strong());
        ui.label("• Ctrl+R: Reload selected map");
        ui.label("• Ctrl+Shift+R: Regenerate colliders");
        ui.label("• Ctrl+H: Toggle layer visibility");
        
        ui.separator();
        ui.label(RichText::new("Tips:").strong().color(Color32::from_rgb(100, 200, 255)));
        ui.label("• Enable hot-reload in Collider Settings for automatic reloading");
        ui.label("• Use Layer Properties panel to adjust layer transforms");
        ui.label("• Use Layer Ordering panel to reorder layers");
        ui.label("• Check Performance panel for optimization tips");
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
    // Show LDtk Maps first
    ui.collapsing(RichText::new("🗺️ LDtk Maps").strong(), |ui| {
        if world.ldtk_maps.is_empty() {
            ui.label(RichText::new("No LDtk maps loaded").color(Color32::GRAY).italics());
        } else {
            for (entity, ldtk_map) in world.ldtk_maps.clone() {
                render_ldtk_map_details(ui, &ldtk_map, entity, world);
            }
        }
    });
    
    ui.separator();
    
    // Show Tilemap Colliders
    ui.collapsing(RichText::new("🔧 Tilemap Colliders").strong(), |ui| {
        if world.tilemap_colliders.is_empty() {
            ui.label(RichText::new("No tilemap colliders").color(Color32::GRAY).italics());
        } else {
            for (entity, mut collider) in world.tilemap_colliders.clone() {
                render_tilemap_collider_settings(ui, &mut collider, entity);
                // Update the collider in world
                world.tilemap_colliders.insert(entity, collider);
            }
        }
    });
    
    ui.separator();
    
    // Show IntGrid Colliders
    ui.collapsing(RichText::new("🔲 IntGrid Colliders").strong(), |ui| {
        if world.ldtk_intgrid_colliders.is_empty() {
            ui.label(RichText::new("No IntGrid colliders").color(Color32::GRAY).italics());
        } else {
            for (entity, mut collider) in world.ldtk_intgrid_colliders.clone() {
                render_ldtk_intgrid_collider_settings(ui, &mut collider, entity);
                // Update the collider in world
                world.ldtk_intgrid_colliders.insert(entity, collider);
            }
        }
    });
    
    ui.separator();
    
    // Legacy loaded maps section (for backward compatibility)
    ui.collapsing(RichText::new("📁 Legacy Maps").strong(), |ui| {
        if map_manager.loaded_maps.is_empty() {
            ui.label(RichText::new("No legacy maps loaded").color(Color32::GRAY).italics());
        } else {
            for (file_path, loaded_map) in map_manager.loaded_maps.clone() {
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                ui.collapsing(RichText::new(format!("🎯 {}", file_name)), |ui| {
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
    });
}

fn render_layer_item(
    ui: &mut egui::Ui,
    layer: &crate::map_manager::LayerInfo,
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
            ui.label("LDtk Maps:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.ldtk_maps.len())).strong().color(Color32::from_rgb(100, 200, 255)));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Tilemaps:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.tilemaps.len())).strong());
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("Tilemap Colliders:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.tilemap_colliders.len())).strong().color(Color32::from_rgb(255, 200, 100)));
            });
        });
        
        ui.horizontal(|ui| {
            ui.label("IntGrid Colliders:");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{}", world.ldtk_intgrid_colliders.len())).strong().color(Color32::from_rgb(255, 150, 100)));
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

/// Render LDtk Map details section
fn render_ldtk_map_details(
    ui: &mut egui::Ui,
    ldtk_map: &LdtkMap,
    entity: ecs::Entity,
    world: &mut World,
) {
    ui.collapsing(RichText::new(format!("🗺️ {}", ldtk_map.identifier)).strong(), |ui| {
        // Map info
        ui.horizontal(|ui| {
            ui.label("File:");
            ui.label(RichText::new(&ldtk_map.file_path).color(Color32::GRAY));
        });
        
        ui.horizontal(|ui| {
            ui.label("Size:");
            ui.label(format!("{}x{} px", ldtk_map.world_width, ldtk_map.world_height));
        });
        
        ui.horizontal(|ui| {
            ui.label("Grid Size:");
            ui.label(format!("{} px", ldtk_map.default_grid_size));
        });
        
        ui.horizontal(|ui| {
            ui.label("Levels:");
            ui.label(format!("{}", ldtk_map.levels.len()));
        });
        
        ui.separator();
        
        // Current level selection
        if !ldtk_map.levels.is_empty() {
            ui.horizontal(|ui| {
                ui.label("Current Level:");
                
                let current_level = ldtk_map.current_level.as_deref().unwrap_or("None");
                egui::ComboBox::from_id_source(format!("level_select_{}", entity))
                    .selected_text(current_level)
                    .show_ui(ui, |ui| {
                        // None option
                        if ui.selectable_value(&mut world.ldtk_maps.get_mut(&entity).unwrap().current_level, None, "None").clicked() {
                            log::info!("Deselected current level");
                        }
                        
                        // Level options
                        for level in &ldtk_map.levels {
                            let mut current = ldtk_map.current_level.clone();
                            if ui.selectable_value(&mut current, Some(level.identifier.clone()), &level.identifier).clicked() {
                                world.ldtk_maps.get_mut(&entity).unwrap().current_level = current;
                                log::info!("Selected level: {}", level.identifier);
                            }
                        }
                    });
            });
        }
        
        ui.separator();
        
        // Auto-reload toggle
        ui.horizontal(|ui| {
            let mut auto_reload = ldtk_map.auto_reload;
            if ui.checkbox(&mut auto_reload, "Auto-reload").changed() {
                world.ldtk_maps.get_mut(&entity).unwrap().auto_reload = auto_reload;
            }
            ui.label("Automatically reload when file changes");
        });
        
        ui.separator();
        
        // Levels details
        if !ldtk_map.levels.is_empty() {
            ui.collapsing("📋 Levels", |ui| {
                for level in &ldtk_map.levels {
                    render_ldtk_level_details(ui, level);
                }
            });
        }
        
        // Tilesets details
        if !ldtk_map.tilesets.is_empty() {
            ui.collapsing("🎨 Tilesets", |ui| {
                for (_, tileset) in &ldtk_map.tilesets {
                    render_ldtk_tileset_details(ui, tileset);
                }
            });
        }
    });
}

/// Render LDtk Level details
fn render_ldtk_level_details(
    ui: &mut egui::Ui,
    level: &ecs::LdtkLevel,
) {
    ui.collapsing(format!("📍 {}", level.identifier), |ui| {
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui.label(format!("({}, {})", level.world_x, level.world_y));
        });
        
        ui.horizontal(|ui| {
            ui.label("Size:");
            ui.label(format!("{}x{} px", level.px_width, level.px_height));
        });
        
        ui.horizontal(|ui| {
            ui.label("Layers:");
            ui.label(format!("{}", level.layers.len()));
        });
        
        ui.horizontal(|ui| {
            ui.label("Entities:");
            ui.label(format!("{}", level.entities.len()));
        });
        
        // Layer details
        if !level.layers.is_empty() {
            ui.separator();
            ui.label(RichText::new("Layers:").strong());
            for layer in &level.layers {
                render_ldtk_layer_summary(ui, layer);
            }
        }
    });
}

/// Render LDtk Layer summary
fn render_ldtk_layer_summary(
    ui: &mut egui::Ui,
    layer: &ecs::LdtkLayerInstance,
) {
    ui.horizontal(|ui| {
        let icon = match layer.layer_type {
            ecs::LdtkLayerType::IntGrid => "🔲",
            ecs::LdtkLayerType::Tiles => "🎨",
            ecs::LdtkLayerType::AutoLayer => "🤖",
            ecs::LdtkLayerType::Entities => "👤",
        };
        
        ui.label(icon);
        ui.label(&layer.identifier);
        
        let type_text = format!("{:?}", layer.layer_type);
        ui.label(RichText::new(type_text).color(Color32::GRAY).italics());
        
        if !layer.visible {
            ui.label(RichText::new("(Hidden)").color(Color32::RED));
        }
        
        if layer.opacity < 1.0 {
            ui.label(RichText::new(format!("({:.0}%)", layer.opacity * 100.0)).color(Color32::YELLOW));
        }
    });
}

/// Render LDtk Tileset details
fn render_ldtk_tileset_details(
    ui: &mut egui::Ui,
    tileset: &ecs::LdtkTilesetDef,
) {
    ui.horizontal(|ui| {
        ui.label("🎨");
        ui.label(&tileset.identifier);
        ui.label(RichText::new(format!("({}x{} px)", tileset.px_width, tileset.px_height)).color(Color32::GRAY));
        ui.label(RichText::new(format!("Grid: {}", tileset.tile_grid_size)).color(Color32::GRAY));
    });
}

/// Render Tilemap Collider settings
fn render_tilemap_collider_settings(
    ui: &mut egui::Ui,
    collider: &mut TilemapCollider,
    entity: ecs::Entity,
) {
    ui.collapsing(format!("🔧 Tilemap Collider ({})", entity), |ui| {
        // Collider mode
        ui.horizontal(|ui| {
            ui.label("Mode:");
            egui::ComboBox::from_id_source(format!("collider_mode_{}", entity))
                .selected_text(format!("{:?}", collider.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Individual, "Individual");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Composite, "Composite");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Polygon, "Polygon");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::None, "None");
                });
        });
        
        // Physics properties
        ui.horizontal(|ui| {
            ui.label("Friction:");
            ui.add(egui::Slider::new(&mut collider.friction, 0.0..=1.0).step_by(0.1));
        });
        
        ui.horizontal(|ui| {
            ui.label("Restitution:");
            ui.add(egui::Slider::new(&mut collider.restitution, 0.0..=1.0).step_by(0.1));
        });
        
        // Flags
        ui.checkbox(&mut collider.is_trigger, "Is Trigger");
        ui.checkbox(&mut collider.use_composite, "Use Composite");
        ui.checkbox(&mut collider.auto_update, "Auto Update");
        
        // Collision tiles
        ui.separator();
        ui.label("Collision Tiles:");
        if collider.collision_tiles.is_empty() {
            ui.label(RichText::new("All non-zero tiles").color(Color32::GRAY).italics());
        } else {
            ui.label(format!("Specific tiles: {:?}", collider.collision_tiles));
        }
    });
}

/// Render LDtk IntGrid Collider settings
fn render_ldtk_intgrid_collider_settings(
    ui: &mut egui::Ui,
    collider: &mut LdtkIntGridCollider,
    entity: ecs::Entity,
) {
    ui.collapsing(format!("🔲 IntGrid Collider ({})", entity), |ui| {
        // Collision value
        ui.horizontal(|ui| {
            ui.label("Collision Value:");
            ui.add(egui::DragValue::new(&mut collider.collision_value).clamp_range(0..=255));
        });
        
        // Mode
        ui.horizontal(|ui| {
            ui.label("Mode:");
            egui::ComboBox::from_id_source(format!("intgrid_mode_{}", entity))
                .selected_text(format!("{:?}", collider.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Individual, "Individual");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Composite, "Composite");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::Polygon, "Polygon");
                    ui.selectable_value(&mut collider.mode, ecs::TilemapColliderMode::None, "None");
                });
        });
        
        // Physics properties
        ui.horizontal(|ui| {
            ui.label("Friction:");
            ui.add(egui::Slider::new(&mut collider.friction, 0.0..=1.0).step_by(0.1));
        });
        
        ui.horizontal(|ui| {
            ui.label("Restitution:");
            ui.add(egui::Slider::new(&mut collider.restitution, 0.0..=1.0).step_by(0.1));
        });
        
        // Flags
        ui.checkbox(&mut collider.is_trigger, "Is Trigger");
        ui.checkbox(&mut collider.auto_update, "Auto Update");
    });
}

/// Create a new LDtk Map entity from file path
pub fn create_ldtk_map_entity(
    world: &mut World,
    file_path: &std::path::Path,
) -> Result<ecs::Entity, String> {
    let entity = world.spawn();
    
    // Create LDtk Map component
    let ldtk_map = LdtkMap::new(file_path.to_string_lossy().to_string());
    world.ldtk_maps.insert(entity, ldtk_map);
    
    // Set entity name
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");
    world.names.insert(entity, format!("LDtk Map: {}", file_name));
    
    // Add transform (for positioning in world)
    world.transforms.insert(entity, ecs::Transform::default());
    
    log::info!("Created LDtk Map entity {} for file: {:?}", entity, file_path);
    Ok(entity)
}

/// Create a Tilemap Collider entity for a tilemap
pub fn create_tilemap_collider_entity(
    world: &mut World,
    tilemap_entity: ecs::Entity,
    collision_tiles: Vec<u32>,
) -> Result<ecs::Entity, String> {
    let entity = world.spawn();
    
    // Create Tilemap Collider component
    let collider = TilemapCollider::with_collision_tiles(collision_tiles);
    world.tilemap_colliders.insert(entity, collider);
    
    // Set entity name
    let tilemap_name = world.names.get(&tilemap_entity)
        .map(|s| s.as_str())
        .unwrap_or("Unknown Tilemap");
    world.names.insert(entity, format!("Collider for {}", tilemap_name));
    
    // Add transform
    world.transforms.insert(entity, ecs::Transform::default());
    
    // Set parent to tilemap entity
    world.set_parent(entity, Some(tilemap_entity));
    
    log::info!("Created Tilemap Collider entity {} for tilemap {}", entity, tilemap_entity);
    Ok(entity)
}

/// Create an IntGrid Collider entity for LDtk IntGrid layer
pub fn create_intgrid_collider_entity(
    world: &mut World,
    layer_entity: ecs::Entity,
    collision_value: i32,
) -> Result<ecs::Entity, String> {
    let entity = world.spawn();
    
    // Create IntGrid Collider component
    let collider = LdtkIntGridCollider::with_value(collision_value);
    world.ldtk_intgrid_colliders.insert(entity, collider);
    
    // Set entity name
    let layer_name = world.names.get(&layer_entity)
        .map(|s| s.as_str())
        .unwrap_or("Unknown Layer");
    world.names.insert(entity, format!("IntGrid Collider for {}", layer_name));
    
    // Add transform
    world.transforms.insert(entity, ecs::Transform::default());
    
    // Set parent to layer entity
    world.set_parent(entity, Some(layer_entity));
    
    log::info!("Created IntGrid Collider entity {} for layer {} with value {}", 
        entity, layer_entity, collision_value);
    Ok(entity)
}

/// Helper function to find LDtk Map entity by file path
pub fn find_ldtk_map_by_path(
    world: &World,
    file_path: &std::path::Path,
) -> Option<ecs::Entity> {
    let path_str = file_path.to_string_lossy();
    
    for (entity, ldtk_map) in &world.ldtk_maps {
        if ldtk_map.file_path == path_str {
            return Some(*entity);
        }
    }
    
    None
}

/// Helper function to get all tilemap colliders for a specific tilemap
pub fn get_tilemap_colliders_for_tilemap(
    world: &World,
    tilemap_entity: ecs::Entity,
) -> Vec<ecs::Entity> {
    let mut colliders = Vec::new();
    
    // Check children of tilemap entity
    if let Some(children) = world.children.get(&tilemap_entity) {
        for &child in children {
            if world.tilemap_colliders.contains_key(&child) {
                colliders.push(child);
            }
        }
    }
    
    colliders
}

/// Helper function to get all IntGrid colliders for a specific layer
pub fn get_intgrid_colliders_for_layer(
    world: &World,
    layer_entity: ecs::Entity,
) -> Vec<ecs::Entity> {
    let mut colliders = Vec::new();
    
    // Check children of layer entity
    if let Some(children) = world.children.get(&layer_entity) {
        for &child in children {
            if world.ldtk_intgrid_colliders.contains_key(&child) {
                colliders.push(child);
            }
        }
    }
    
    colliders
}