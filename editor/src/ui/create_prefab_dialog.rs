use egui;
use ecs::{World, Entity};
use std::collections::HashMap;

/// State for create prefab dialog
pub struct CreatePrefabDialog {
    pub show: bool,
    pub entity: Option<Entity>,
    pub name: String,
    pub error: Option<String>,
}

impl Default for CreatePrefabDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl CreatePrefabDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            entity: None,
            name: String::new(),
            error: None,
        }
    }
    
    /// Open dialog for creating prefab from entity
    pub fn open(&mut self, entity: Entity, entity_names: &HashMap<Entity, String>) {
        self.show = true;
        self.entity = Some(entity);
        
        // Suggest name based on entity name
        if let Some(entity_name) = entity_names.get(&entity) {
            self.name = entity_name.clone();
        } else {
            self.name = format!("Entity_{}", entity);
        }
        
        self.error = None;
    }
    
    /// Render the dialog
    /// Returns Some(name) if user clicked Create, None otherwise
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        world: &World,
        entity_names: &HashMap<Entity, String>,
        prefab_manager: &crate::PrefabManager,
    ) -> Option<String> {
        if !self.show {
            return None;
        }
        
        let mut result = None;
        let mut should_close = false;
        
        egui::Window::new("📦 Create Prefab")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                
                // Entity info
                if let Some(entity) = self.entity {
                    let entity_name = entity_names.get(&entity)
                        .cloned()
                        .unwrap_or_else(|| format!("Entity {}", entity));
                    
                    ui.horizontal(|ui| {
                        ui.label("Entity:");
                        ui.label(egui::RichText::new(&entity_name).strong());
                    });
                    
                    // Show component count
                    let component_count = count_components(world, entity);
                    ui.label(format!("Components: {}", component_count));
                    
                    // Show children count
                    let children_count = world.get_children(entity).len();
                    if children_count > 0 {
                        ui.label(format!("Children: {}", children_count));
                    }
                    
                    ui.separator();
                }
                
                // Name input
                ui.horizontal(|ui| {
                    ui.label("Prefab Name:");
                    let response = ui.text_edit_singleline(&mut self.name);
                    
                    // Auto-focus on first show
                    if response.gained_focus() {
                        response.request_focus();
                    }
                    
                    // Handle Enter key
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if self.validate_name(prefab_manager) {
                            result = Some(self.name.clone());
                            should_close = true;
                        }
                    }
                });
                
                // Error message
                if let Some(ref error) = self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
                
                ui.separator();
                
                // Help text
                ui.label(egui::RichText::new("ℹ️ Tip:").small());
                ui.label(egui::RichText::new("Prefab will be saved to prefabs/ folder").small());
                ui.label(egui::RichText::new("Use descriptive names like 'Player', 'Enemy', 'Platform'").small());
                
                ui.separator();
                
                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("✅ Create").clicked() {
                        if self.validate_name(prefab_manager) {
                            result = Some(self.name.clone());
                            should_close = true;
                        }
                    }
                    
                    if ui.button("❌ Cancel").clicked() {
                        should_close = true;
                    }
                });
            });
        
        if should_close {
            self.show = false;
            self.entity = None;
            self.name.clear();
            self.error = None;
        }
        
        result
    }
    
    /// Validate prefab name
    fn validate_name(&mut self, prefab_manager: &crate::PrefabManager) -> bool {
        // Check if name is empty
        if self.name.trim().is_empty() {
            self.error = Some("Name cannot be empty".to_string());
            return false;
        }
        
        // Check for invalid characters
        if self.name.contains('/') || self.name.contains('\\') || self.name.contains(':') {
            self.error = Some("Name contains invalid characters".to_string());
            return false;
        }
        
        // Check if prefab already exists
        let prefab_file = format!("{}.prefab", self.name.replace(" ", "_"));
        if let Some(project_path) = &prefab_manager.project_path {
            let prefab_path = project_path.join("prefabs").join(&prefab_file);
            if prefab_path.exists() {
                self.error = Some(format!("Prefab '{}' already exists", self.name));
                return false;
            }
        }
        
        self.error = None;
        true
    }
}

/// Count components on an entity
fn count_components(world: &World, entity: Entity) -> usize {
    let mut count = 0;
    
    if world.transforms.contains_key(&entity) { count += 1; }
    if world.sprites.contains_key(&entity) { count += 1; }
    if world.cameras.contains_key(&entity) { count += 1; }
    if world.meshes.contains_key(&entity) { count += 1; }
    if world.colliders.contains_key(&entity) { count += 1; }
    if world.rigidbodies.contains_key(&entity) { count += 1; }
    if world.tilemaps.contains_key(&entity) { count += 1; }
    if world.tilemap_renderers.contains_key(&entity) { count += 1; }
    if world.tilesets.contains_key(&entity) { count += 1; }
    if world.grids.contains_key(&entity) { count += 1; }
    if world.scripts.contains_key(&entity) { count += 1; }
    
    count
}
