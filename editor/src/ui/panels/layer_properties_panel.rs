use egui::{self, Color32, RichText};
use ecs::{World, Entity};
use crate::map_manager::MapManager;

/// Layer Properties Panel for editing individual layer properties
pub struct LayerPropertiesPanel {
    /// Currently selected layer entity
    pub selected_layer: Option<Entity>,
}

impl LayerPropertiesPanel {
    /// Create a new Layer Properties Panel
    pub fn new() -> Self {
        Self {
            selected_layer: None,
        }
    }
    
    /// Render the Layer Properties Panel as a standalone window
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &mut World,
        map_manager: &mut MapManager,
        open: &mut bool,
    ) {
        egui::Window::new("🎨 Layer Properties")
            .open(open)
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui, world, map_manager);
            });
    }
    
    /// Render the Layer Properties Panel content (for use in docking system)
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        // Check if we have a selected layer
        if let Some(layer_entity) = self.selected_layer {
            // Verify the entity still exists
            if !world.transforms.contains_key(&layer_entity) {
                self.selected_layer = None;
                ui.label(RichText::new("No layer selected").color(Color32::GRAY).italics());
                return;
            }
            
            // Render layer properties
            self.render_layer_properties(ui, layer_entity, world, map_manager);
        } else {
            // No selection state with help
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No layer selected").color(Color32::GRAY).italics());
                ui.add_space(10.0);
                ui.label(RichText::new("Select a layer from the Maps Panel").color(Color32::GRAY).small());
            });
            
            ui.separator();
            
            // Help section
            ui.collapsing("ℹ️ Help", |ui| {
                ui.label(RichText::new("Layer Properties:").strong());
                ui.separator();
                
                ui.label("Transform:");
                ui.label("• Adjust position, rotation, and scale");
                ui.label("• Use reset buttons to restore defaults");
                
                ui.separator();
                ui.label("Rendering:");
                ui.label("• Toggle visibility with the eye icon");
                ui.label("• Adjust Z-Order to control render order");
                ui.label("• Higher Z-Order renders on top");
                ui.label("• Modify opacity and color tint");
                
                ui.separator();
                ui.label(RichText::new("Tips:").strong().color(Color32::from_rgb(100, 200, 255)));
                ui.label("• Changes apply immediately");
                ui.label("• Use Layer Ordering panel for drag & drop reordering");
                ui.label("• Press Ctrl+H to toggle visibility");
            });
        }
    }
    
    /// Render properties for a specific layer
    fn render_layer_properties(
        &mut self,
        ui: &mut egui::Ui,
        layer_entity: Entity,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        // Get layer name
        let layer_name = world.names.get(&layer_entity)
            .cloned()
            .unwrap_or_else(|| "Unknown Layer".to_string());
        
        ui.heading(&layer_name);
        ui.separator();
        
        // Transform section
        self.render_transform_section(ui, layer_entity, world);
        
        ui.separator();
        
        // Rendering section
        self.render_rendering_section(ui, layer_entity, world, map_manager);
        
        ui.separator();
        
        // Tilemap info section
        self.render_tilemap_info_section(ui, layer_entity, world);
        
        ui.separator();
        
        // Advanced section
        self.render_advanced_section(ui, layer_entity, world);
    }
    
    /// Render transform editing section
    fn render_transform_section(
        &mut self,
        ui: &mut egui::Ui,
        layer_entity: Entity,
        world: &mut World,
    ) {
        ui.collapsing(RichText::new("🔧 Transform").strong(), |ui| {
            if let Some(transform) = world.transforms.get_mut(&layer_entity) {
                // Position
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add_space(5.0);
                    
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut transform.position[0]).speed(0.1)).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut transform.position[1]).speed(0.1)).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Z:");
                    if ui.add(egui::DragValue::new(&mut transform.position[2]).speed(0.1)).changed() {
                        // Transform updated
                    }
                    
                    if ui.small_button("↺").on_hover_text("Reset Position").clicked() {
                        transform.position = [0.0, 0.0, 0.0];
                    }
                });
                
                // Rotation
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add_space(5.0);
                    
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut transform.rotation[0]).speed(1.0).suffix("°")).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut transform.rotation[1]).speed(1.0).suffix("°")).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Z:");
                    if ui.add(egui::DragValue::new(&mut transform.rotation[2]).speed(1.0).suffix("°")).changed() {
                        // Transform updated
                    }
                    
                    if ui.small_button("↺").on_hover_text("Reset Rotation").clicked() {
                        transform.rotation = [0.0, 0.0, 0.0];
                    }
                });
                
                // Scale
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.add_space(5.0);
                    
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(&mut transform.scale[0]).speed(0.01).clamp_range(0.01..=100.0)).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(&mut transform.scale[1]).speed(0.01).clamp_range(0.01..=100.0)).changed() {
                        // Transform updated
                    }
                    
                    ui.label("Z:");
                    if ui.add(egui::DragValue::new(&mut transform.scale[2]).speed(0.01).clamp_range(0.01..=100.0)).changed() {
                        // Transform updated
                    }
                    
                    if ui.small_button("↺").on_hover_text("Reset Scale").clicked() {
                        transform.scale = [1.0, 1.0, 1.0];
                    }
                });
            } else {
                ui.label(RichText::new("No transform component").color(Color32::GRAY).italics());
            }
        });
    }
    
    /// Render rendering section
    fn render_rendering_section(
        &mut self,
        ui: &mut egui::Ui,
        layer_entity: Entity,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        ui.collapsing(RichText::new("🎨 Rendering").strong(), |ui| {
            // Visibility checkbox
            if let Some(active) = world.active.get_mut(&layer_entity) {
                let mut visible = *active;
                if ui.checkbox(&mut visible, "Visible").changed() {
                    *active = visible;
                    
                    // Update MapManager's LayerInfo
                    map_manager.toggle_layer_visibility(layer_entity, world);
                }
            }
            
            // Z-Order
            if let Some(tilemap) = world.tilemaps.get_mut(&layer_entity) {
                ui.horizontal(|ui| {
                    ui.label("Z-Order:");
                    ui.add(egui::DragValue::new(&mut tilemap.z_order).speed(1.0));
                });
                
                ui.label(RichText::new("Higher values render on top").color(Color32::GRAY).small());
            }
            
            // Opacity slider (placeholder - would need to add opacity to Tilemap component)
            ui.horizontal(|ui| {
                ui.label("Opacity:");
                let mut opacity = 1.0f32;
                ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0));
            });
            ui.label(RichText::new("(Not yet implemented)").color(Color32::GRAY).small().italics());
            
            // Color tint picker (placeholder - would need to add color to Tilemap component)
            ui.horizontal(|ui| {
                ui.label("Tint:");
                let mut color = [1.0, 1.0, 1.0];
                ui.color_edit_button_rgb(&mut color);
            });
            ui.label(RichText::new("(Not yet implemented)").color(Color32::GRAY).small().italics());
        });
    }
    
    /// Render tilemap info section
    fn render_tilemap_info_section(
        &mut self,
        ui: &mut egui::Ui,
        layer_entity: Entity,
        world: &World,
    ) {
        ui.collapsing(RichText::new("📊 Tilemap Info").strong(), |ui| {
            if let Some(tilemap) = world.tilemaps.get(&layer_entity) {
                // Tilemap size
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} x {}", tilemap.width, tilemap.height)).strong());
                    });
                });
                
                // Tileset
                if let Some(tileset) = world.tilesets.get(&layer_entity) {
                    ui.horizontal(|ui| {
                        ui.label("Tileset:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&tileset.name).strong());
                        });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Tile Size:");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} x {}", tileset.tile_width, tileset.tile_height)).strong());
                        });
                    });
                }
                
                // Tile count
                let tile_count = tilemap.tiles.len();
                ui.horizontal(|ui| {
                    ui.label("Tile Count:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{}", tile_count)).strong());
                    });
                });
                
                // Memory usage estimate (rough calculation)
                let memory_bytes = tile_count * std::mem::size_of::<ecs::Tile>();
                let memory_kb = memory_bytes as f32 / 1024.0;
                ui.horizontal(|ui| {
                    ui.label("Memory:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.2} KB", memory_kb)).strong());
                    });
                });
            } else {
                ui.label(RichText::new("No tilemap component").color(Color32::GRAY).italics());
            }
        });
    }
    
    /// Render advanced section
    fn render_advanced_section(
        &mut self,
        ui: &mut egui::Ui,
        layer_entity: Entity,
        world: &World,
    ) {
        ui.collapsing(RichText::new("⚙️ Advanced").strong(), |ui| {
            // Entity ID
            ui.horizontal(|ui| {
                ui.label("Entity ID:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", layer_entity)).strong().monospace());
                });
            });
            
            // Parent info
            if let Some(parent) = world.get_parent(layer_entity) {
                ui.horizontal(|ui| {
                    ui.label("Parent:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let parent_name = world.names.get(&parent)
                            .cloned()
                            .unwrap_or_else(|| format!("Entity {}", parent));
                        ui.label(RichText::new(parent_name).strong());
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Parent:");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("None").color(Color32::GRAY));
                    });
                });
            }
            
            // Children count
            let children_count = world.get_children(layer_entity).len();
            ui.horizontal(|ui| {
                ui.label("Children:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", children_count)).strong());
                });
            });
            
            // Components list
            ui.label(RichText::new("Components:").strong());
            ui.indent("components_list", |ui| {
                if world.transforms.contains_key(&layer_entity) {
                    ui.label("• Transform");
                }
                if world.tilemaps.contains_key(&layer_entity) {
                    ui.label("• Tilemap");
                }
                if world.tilesets.contains_key(&layer_entity) {
                    ui.label("• TileSet");
                }
                if world.active.contains_key(&layer_entity) {
                    ui.label("• Active");
                }
                if world.sprites.contains_key(&layer_entity) {
                    ui.label("• Sprite");
                }
                if world.colliders.contains_key(&layer_entity) {
                    ui.label("• Collider");
                }
            });
        });
    }
}

impl Default for LayerPropertiesPanel {
    fn default() -> Self {
        Self::new()
    }
}
