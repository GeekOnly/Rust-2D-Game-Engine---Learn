use ecs::{World, Entity, Map, MapType};
use egui;
use std::path::PathBuf;
use std::process::Command;

/// Render Map component inspector
pub fn render_map_inspector(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    project_path: &Option<PathBuf>,
) -> bool {
    let mut changed = false;
    let mut should_remove = false;
    let mut should_load = false;
    let mut should_reload = false;
    let mut should_generate_colliders = false;
    
    if let Some(map) = world.maps.get_mut(&entity) {
        ui.horizontal(|ui| {
            ui.label("🗺️ Map");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Remove component button
                if ui.small_button("✖").on_hover_text("Remove Map component").clicked() {
                    should_remove = true;
                    changed = true;
                }
            });
        });
        
        ui.separator();
        
        // Display name
        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut map.display_name).changed() {
                changed = true;
            }
        });
        
        // Map type
        ui.horizontal(|ui| {
            ui.label("Type:");
            let mut map_type_index = match map.map_type {
                MapType::LDtk => 0,
                MapType::Tiled => 1,
            };
            
            if egui::ComboBox::from_id_source("map_type")
                .selected_text(map.map_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut map_type_index, 0, "LDtk");
                    ui.selectable_value(&mut map_type_index, 1, "Tiled");
                })
                .response
                .changed()
            {
                map.map_type = match map_type_index {
                    0 => MapType::LDtk,
                    1 => MapType::Tiled,
                    _ => MapType::LDtk,
                };
                changed = true;
            }
        });
        
        // File path
        ui.horizontal(|ui| {
            ui.label("File:");
            if ui.text_edit_singleline(&mut map.file_path).changed() {
                changed = true;
            }
            
            // Browse button
            if ui.button("📁").on_hover_text("Browse for map file").clicked() {
                if let Some(path) = browse_for_map_file(&map.map_type, project_path) {
                    map.file_path = path;
                    changed = true;
                }
            }
        });
        
        // File status
        if !map.file_path.is_empty() {
            let file_exists = if let Some(proj_path) = project_path {
                map.absolute_path(proj_path).exists()
            } else {
                false
            };
            
            let status_text = if file_exists {
                "✓ File exists"
            } else {
                "✗ File not found"
            };
            
            let status_color = if file_exists {
                egui::Color32::from_rgb(100, 200, 100)
            } else {
                egui::Color32::from_rgb(200, 100, 100)
            };
            
            ui.colored_label(status_color, status_text);
        }
        
        ui.add_space(5.0);
        
        // Hot-reload toggle
        ui.horizontal(|ui| {
            if ui.checkbox(&mut map.hot_reload_enabled, "Hot-Reload")
                .on_hover_text("Automatically reload map when file changes")
                .changed()
            {
                changed = true;
            }
            
            if cfg!(debug_assertions) {
                ui.label("(Debug mode)");
            }
        });
        
        ui.add_space(10.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            // Open in editor button
            let editor_name = map.map_type.display_name();
            if ui.button(format!("🎨 Open in {}", editor_name))
                .on_hover_text(format!("Open this map in {} editor", editor_name))
                .clicked()
            {
                if let Some(proj_path) = project_path {
                    open_in_external_editor(map, proj_path);
                }
            }
            
            // Load button
            if ui.button("📥 Load Map")
                .on_hover_text("Load this map into the scene")
                .clicked()
            {
                should_load = true;
            }
            
            // Reload button (if loaded)
            if map.is_loaded {
                if ui.button("🔄 Reload")
                    .on_hover_text("Reload the map")
                    .clicked()
                {
                    should_reload = true;
                }
            }
        });
        
        // Generate Colliders button (for LDtk maps)
        if map.map_type == MapType::LDtk && map.is_loaded {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("🔲 Generate Colliders (Composite)")
                    .on_hover_text("Generate optimized collision boxes by merging adjacent tiles")
                    .clicked()
                {
                    should_generate_colliders = true;
                }
                
                ui.label("(IntGrid value = 1)");
            });
        }
        
        ui.add_space(5.0);
        
        // Info section
        if map.is_loaded {
            ui.separator();
            ui.label(format!("Loaded: {} entities", map.spawned_entities.len()));
        }
        
        // Help section
        ui.add_space(10.0);
        ui.separator();
        ui.collapsing("ℹ️ Help", |ui| {
            ui.label(format!("Map Type: {}", map.map_type.display_name()));
            ui.label(format!("Extensions: {}", map.map_type.extensions().join(", ")));
            
            ui.add_space(5.0);
            
            if ui.button(format!("📖 Download {}", map.map_type.display_name())).clicked() {
                let url = map.map_type.editor_url();
                if let Err(e) = open::that(url) {
                    log::error!("Failed to open URL: {}", e);
                }
            }
            
            ui.add_space(5.0);
            ui.label("Hot-reload workflow:");
            ui.label("1. Edit map in external editor");
            ui.label("2. Save the file");
            ui.label("3. See changes instantly in game");
        });
    }
    
    // Handle actions after borrowing map
    if should_remove {
        world.maps.remove(&entity);
    }
    
    if should_load || should_reload {
        if let Some(proj_path) = project_path {
            // Get map data before borrowing world mutably
            let map_data = world.maps.get(&entity).map(|m| (m.file_path.clone(), m.map_type.clone(), m.spawned_entities.clone()));
            
            if let Some((file_path, map_type, old_entities)) = map_data {
                // Despawn old entities
                for &old_entity in &old_entities {
                    world.despawn(old_entity);
                }
                
                // Load map
                let full_path = proj_path.join(&file_path);
                if full_path.exists() {
                    log::info!("Loading map: {:?}", full_path);
                    
                    let result = match map_type {
                        ecs::MapType::LDtk => {
                            ecs::loaders::LdtkLoader::load_project(&full_path, world)
                        }
                        ecs::MapType::Tiled => {
                            ecs::loaders::TiledLoader::load_map(&full_path, world)
                        }
                    };
                    
                    match result {
                        Ok(entities) => {
                            log::info!("✓ Loaded {} entities from map", entities.len());
                            if let Some(map) = world.maps.get_mut(&entity) {
                                map.spawned_entities = entities;
                                map.is_loaded = true;
                            }
                        }
                        Err(e) => {
                            log::error!("✗ Failed to load map: {}", e);
                            if let Some(map) = world.maps.get_mut(&entity) {
                                map.is_loaded = false;
                                map.spawned_entities.clear();
                            }
                        }
                    }
                } else {
                    log::error!("Map file not found: {:?}", full_path);
                }
                
                changed = true;
            }
        }
    }
    
    // Handle collider generation
    if should_generate_colliders {
        if let Some(proj_path) = project_path {
            // Get map file path
            let map_data = world.maps.get(&entity).map(|m| m.file_path.clone());
            
            if let Some(file_path) = map_data {
                let full_path = proj_path.join(&file_path);
                
                // Use composite collider generation (optimized)
                match ecs::loaders::LdtkLoader::generate_composite_colliders_from_intgrid(
                    &full_path,
                    world,
                    1,  // IntGrid value 1 = solid
                    None
                ) {
                    Ok(collider_entities) => {
                        log::info!("✓ Generated {} composite colliders for map", collider_entities.len());
                        // Store collider entities in map for cleanup
                        if let Some(map) = world.maps.get_mut(&entity) {
                            map.spawned_entities.extend(collider_entities);
                        }
                        changed = true;
                    }
                    Err(e) => {
                        log::error!("✗ Failed to generate colliders: {}", e);
                    }
                }
            }
        }
    }
    
    changed
}

/// Browse for a map file
fn browse_for_map_file(map_type: &MapType, project_path: &Option<PathBuf>) -> Option<String> {
    let extensions: Vec<&str> = map_type.extensions().to_vec();
    
    let mut dialog = rfd::FileDialog::new()
        .set_title(format!("Select {} Map File", map_type.display_name()));
    
    // Set starting directory to project path if available
    if let Some(proj_path) = project_path {
        dialog = dialog.set_directory(proj_path);
    }
    
    // Add file filters
    for ext in &extensions {
        dialog = dialog.add_filter(map_type.display_name(), &[ext]);
    }
    
    if let Some(path) = dialog.pick_file() {
        // Make path relative to project if possible
        if let Some(proj_path) = project_path {
            if let Ok(relative) = path.strip_prefix(proj_path) {
                return Some(relative.to_string_lossy().to_string());
            }
        }
        
        // Return absolute path if can't make relative
        return Some(path.to_string_lossy().to_string());
    }
    
    None
}

/// Open map file in external editor
fn open_in_external_editor(map: &Map, project_path: &PathBuf) {
    let file_path = map.absolute_path(project_path);
    
    if !file_path.exists() {
        log::error!("Map file not found: {:?}", file_path);
        return;
    }
    
    // Try to open with default application first
    match open::that(&file_path) {
        Ok(_) => {
            log::info!("Opened map in default editor: {:?}", file_path);
        }
        Err(e) => {
            log::warn!("Failed to open with default app: {}, trying specific editor...", e);
            
            // Try to open with specific editor
            let editor_exe = map.map_type.editor_executable();
            
            #[cfg(target_os = "windows")]
            let result = Command::new("cmd")
                .args(&["/C", "start", "", editor_exe, file_path.to_str().unwrap_or("")])
                .spawn();
            
            #[cfg(target_os = "macos")]
            let result = Command::new("open")
                .args(&["-a", editor_exe, file_path.to_str().unwrap_or("")])
                .spawn();
            
            #[cfg(target_os = "linux")]
            let result = Command::new(editor_exe)
                .arg(file_path.to_str().unwrap_or(""))
                .spawn();
            
            match result {
                Ok(_) => log::info!("Opened map in {}: {:?}", editor_exe, file_path),
                Err(e) => {
                    log::error!("Failed to open map editor: {}", e);
                    log::info!("Please install {} from: {}", 
                        map.map_type.display_name(), 
                        map.map_type.editor_url()
                    );
                }
            }
        }
    }
}

/// Render "Add Map Component" button in inspector
pub fn render_add_map_button(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
) -> bool {
    if ui.button("+ Map").clicked() {
        let map = Map::default();
        world.maps.insert(entity, map);
        return true;
    }
    false
}
