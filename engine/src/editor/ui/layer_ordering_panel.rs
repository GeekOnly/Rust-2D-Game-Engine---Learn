use egui::{self, Color32, RichText, Pos2, Rect};
use ecs::{World, Entity};
use crate::editor::map_manager::MapManager;
use std::path::PathBuf;

/// Layer Ordering Panel for reordering layers using drag & drop
pub struct LayerOrderingPanel {
    /// Currently selected map for ordering
    pub selected_map: Option<PathBuf>,
    
    /// Drag state
    drag_state: Option<DragState>,
}

/// State for drag and drop operation
struct DragState {
    /// Index of the layer being dragged
    dragging_index: usize,
    
    /// Current mouse position during drag
    current_pos: Pos2,
    
    /// Start position of drag
    start_pos: Pos2,
    
    /// Entity being dragged
    entity: Entity,
}

impl LayerOrderingPanel {
    /// Create a new Layer Ordering Panel
    pub fn new() -> Self {
        Self {
            selected_map: None,
            drag_state: None,
        }
    }
    
    /// Render the Layer Ordering Panel as a standalone window
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &mut World,
        map_manager: &mut MapManager,
        open: &mut bool,
    ) {
        egui::Window::new("📑 Layer Ordering")
            .open(open)
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui, world, map_manager);
            });
    }
    
    /// Render the Layer Ordering Panel content (for use in docking system)
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        // Map selection dropdown
        self.render_map_selection(ui, map_manager);
        
        ui.separator();
        
        // Layer list with drag & drop
        if let Some(selected_map) = &self.selected_map.clone() {
            if let Some(loaded_map) = map_manager.loaded_maps.get(selected_map) {
                self.render_layer_list(ui, world, map_manager, loaded_map.clone());
            } else {
                ui.label(RichText::new("Map not loaded").color(Color32::GRAY).italics());
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No map selected").color(Color32::GRAY).italics());
                ui.add_space(10.0);
                ui.label(RichText::new("Select a map from the dropdown above").color(Color32::GRAY).small());
            });
        }
    }
    
    /// Render map selection dropdown
    fn render_map_selection(
        &mut self,
        ui: &mut egui::Ui,
        map_manager: &MapManager,
    ) {
        ui.horizontal(|ui| {
            ui.label("Map:");
            
            // Get list of loaded maps
            let loaded_maps: Vec<PathBuf> = map_manager.loaded_maps.keys().cloned().collect();
            
            if loaded_maps.is_empty() {
                ui.label(RichText::new("No maps loaded").color(Color32::GRAY).italics());
                return;
            }
            
            // Current selection text
            let current_text = if let Some(selected) = &self.selected_map {
                selected.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
            } else {
                "Select a map..."
            };
            
            // Dropdown
            egui::ComboBox::from_id_source("map_selection")
                .selected_text(current_text)
                .show_ui(ui, |ui| {
                    for map_path in loaded_maps {
                        let map_name = map_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");
                        
                        let is_selected = self.selected_map.as_ref() == Some(&map_path);
                        
                        if ui.selectable_label(is_selected, map_name).clicked() {
                            self.selected_map = Some(map_path);
                        }
                    }
                });
        });
    }
    
    /// Render layer list with drag & drop functionality
    fn render_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
        loaded_map: crate::editor::map_manager::LoadedMap,
    ) {
        ui.heading("Layers");
        ui.label(RichText::new("Drag to reorder • Higher = Renders on top").color(Color32::GRAY).small());
        ui.add_space(5.0);
        
        // Get layers sorted by Z-Order (highest first)
        let mut layers = loaded_map.layer_entities.clone();
        layers.sort_by(|a, b| b.z_order.cmp(&a.z_order));
        
        // Render each layer
        for (index, layer) in layers.iter().enumerate() {
            self.render_layer_item(ui, world, map_manager, layer, index, &layers);
        }
    }
    
    /// Render a single layer item with drag & drop support
    fn render_layer_item(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
        layer: &crate::editor::map_manager::LayerInfo,
        index: usize,
        all_layers: &[crate::editor::map_manager::LayerInfo],
    ) {
        let layer_name = layer.name.replace("LDTK Layer: ", "");
        
        // Check if this layer is being dragged
        let is_dragging = self.drag_state.as_ref()
            .map(|ds| ds.entity == layer.entity)
            .unwrap_or(false);
        
        // Create a frame for the layer item
        let frame = egui::Frame::none()
            .fill(if is_dragging {
                Color32::from_rgb(60, 100, 140) // Highlight when dragging
            } else if index % 2 == 0 {
                Color32::from_gray(30)
            } else {
                Color32::from_gray(35)
            })
            .inner_margin(egui::Margin::same(8.0))
            .rounding(4.0);
        
        let response = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Drag handle
                let drag_handle = ui.label("☰");
                
                // Layer icon
                ui.label("🎨");
                
                // Layer name and Z-Order
                let text = if layer.visible {
                    RichText::new(format!("{} (Z: {})", layer_name, layer.z_order))
                } else {
                    RichText::new(format!("{} (Z: {})", layer_name, layer.z_order))
                        .color(Color32::GRAY)
                };
                
                ui.label(text);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Move down button (decrease Z-Order)
                    if ui.small_button("▼")
                        .on_hover_text("Move Down (Decrease Z-Order)")
                        .clicked() 
                    {
                        self.move_layer_down(layer.entity, world, map_manager);
                    }
                    
                    // Move up button (increase Z-Order)
                    if ui.small_button("▲")
                        .on_hover_text("Move Up (Increase Z-Order)")
                        .clicked() 
                    {
                        self.move_layer_up(layer.entity, world, map_manager);
                    }
                    
                    // Visibility toggle
                    let icon = if layer.visible { "👁" } else { "👁‍🗨" };
                    if ui.small_button(icon)
                        .on_hover_text(if layer.visible { "Hide" } else { "Show" })
                        .clicked() 
                    {
                        map_manager.toggle_layer_visibility(layer.entity, world);
                    }
                    
                    // Lock toggle (placeholder)
                    let lock_icon = "🔓";
                    if ui.small_button(lock_icon)
                        .on_hover_text("Lock/Unlock (Not yet implemented)")
                        .clicked() 
                    {
                        // TODO: Implement lock functionality
                        log::info!("Lock toggle clicked for layer: {}", layer_name);
                    }
                });
                
                drag_handle
            }).inner
        });
        
        // Handle drag and drop
        let item_rect = response.response.rect;
        
        // Start drag on drag handle
        if response.inner.hovered() && ui.input(|i| i.pointer.primary_down()) {
            if self.drag_state.is_none() {
                // Start dragging
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    self.drag_state = Some(DragState {
                        dragging_index: index,
                        current_pos: pointer_pos,
                        start_pos: pointer_pos,
                        entity: layer.entity,
                    });
                    log::info!("Started dragging layer: {}", layer_name);
                }
            }
        }
        
        // Update drag position
        let should_end_drag = if let Some(drag_state) = &mut self.drag_state {
            if drag_state.entity == layer.entity {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    drag_state.current_pos = pointer_pos;
                }
                
                // Check if drag should end
                ui.input(|i| i.pointer.primary_released())
            } else {
                false
            }
        } else {
            false
        };
        
        // End drag on release (outside of borrow)
        if should_end_drag {
            if let Some(drag_state) = &self.drag_state {
                // Calculate drop index based on current position
                let drop_index = self.calculate_drop_index(
                    drag_state.current_pos,
                    all_layers,
                    ui,
                );
                
                log::info!("Dropped layer at index: {}", drop_index);
                
                // Reorder layers
                if drop_index != index {
                    self.reorder_layers(
                        index,
                        drop_index,
                        world,
                        map_manager,
                    );
                }
            }
            
            self.drag_state = None;
        }
        
        // Draw visual feedback during drag
        if let Some(drag_state) = &self.drag_state {
            if drag_state.entity == layer.entity {
                // Visual indicator at drop position
                ui.painter().line_segment(
                    [
                        Pos2::new(item_rect.min.x, item_rect.min.y),
                        Pos2::new(item_rect.max.x, item_rect.min.y),
                    ],
                    egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                );
            }
        }
        
        ui.add_space(2.0);
    }
    
    /// Calculate the drop index based on mouse position
    fn calculate_drop_index(
        &self,
        mouse_pos: Pos2,
        all_layers: &[crate::editor::map_manager::LayerInfo],
        _ui: &egui::Ui,
    ) -> usize {
        // Simple calculation: find which layer the mouse is over
        // For now, just return the index based on vertical position
        // This is a simplified version - a full implementation would track
        // the actual positions of each layer item
        
        if let Some(drag_state) = &self.drag_state {
            let delta_y = mouse_pos.y - drag_state.start_pos.y;
            let item_height = 40.0; // Approximate height of each item
            let index_change = (delta_y / item_height).round() as i32;
            
            let new_index = (drag_state.dragging_index as i32 + index_change)
                .max(0)
                .min(all_layers.len() as i32 - 1) as usize;
            
            new_index
        } else {
            0
        }
    }
    
    /// Move layer up (increment Z-Order)
    fn move_layer_up(
        &mut self,
        entity: Entity,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        if let Some(tilemap) = world.tilemaps.get_mut(&entity) {
            tilemap.z_order += 1;
            
            // Update MapManager's LayerInfo
            for loaded_map in map_manager.loaded_maps.values_mut() {
                if let Some(layer) = loaded_map.layer_entities.iter_mut().find(|l| l.entity == entity) {
                    layer.z_order = tilemap.z_order;
                }
            }
            
            log::info!("Moved layer up: Entity {} Z-Order now {}", entity, tilemap.z_order);
        }
    }
    
    /// Move layer down (decrement Z-Order, minimum -100)
    fn move_layer_down(
        &mut self,
        entity: Entity,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        if let Some(tilemap) = world.tilemaps.get_mut(&entity) {
            // Minimum Z-Order is -100
            if tilemap.z_order > -100 {
                tilemap.z_order -= 1;
                
                // Update MapManager's LayerInfo
                for loaded_map in map_manager.loaded_maps.values_mut() {
                    if let Some(layer) = loaded_map.layer_entities.iter_mut().find(|l| l.entity == entity) {
                        layer.z_order = tilemap.z_order;
                    }
                }
                
                log::info!("Moved layer down: Entity {} Z-Order now {}", entity, tilemap.z_order);
            }
        }
    }
    
    /// Reorder layers by moving a layer from one position to another
    /// Updates all Z-Order values to maintain monotonic increasing order
    fn reorder_layers(
        &mut self,
        from_index: usize,
        to_index: usize,
        world: &mut World,
        map_manager: &mut MapManager,
    ) {
        if from_index == to_index {
            return;
        }
        
        // Get the selected map
        let selected_map = match &self.selected_map {
            Some(path) => path.clone(),
            None => return,
        };
        
        // Get the loaded map
        let loaded_map = match map_manager.loaded_maps.get_mut(&selected_map) {
            Some(map) => map,
            None => return,
        };
        
        // Get layers sorted by Z-Order (highest first)
        let mut layers = loaded_map.layer_entities.clone();
        layers.sort_by(|a, b| b.z_order.cmp(&a.z_order));
        
        // Move the layer in the list
        if from_index >= layers.len() || to_index >= layers.len() {
            return;
        }
        
        let moved_layer = layers.remove(from_index);
        layers.insert(to_index, moved_layer);
        
        // Update Z-Order values based on new positions
        // Higher index = higher Z-Order (renders on top)
        let base_z_order = 0;
        for (i, layer) in layers.iter().enumerate() {
            let new_z_order = base_z_order + (layers.len() - 1 - i) as i32;
            
            // Update tilemap component
            if let Some(tilemap) = world.tilemaps.get_mut(&layer.entity) {
                tilemap.z_order = new_z_order;
            }
            
            // Update MapManager's LayerInfo
            if let Some(loaded_map) = map_manager.loaded_maps.get_mut(&selected_map) {
                if let Some(layer_info) = loaded_map.layer_entities.iter_mut().find(|l| l.entity == layer.entity) {
                    layer_info.z_order = new_z_order;
                }
            }
            
            log::info!("Updated layer {} Z-Order to {}", layer.name, new_z_order);
        }
        
        log::info!("Reordered layers: moved from index {} to {}", from_index, to_index);
    }
}

impl Default for LayerOrderingPanel {
    fn default() -> Self {
        Self::new()
    }
}
