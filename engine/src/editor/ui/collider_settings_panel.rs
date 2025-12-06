use egui::{self, Color32, RichText};
use crate::editor::map_manager::MapManager;

/// Collider type selection
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ColliderType {
    /// Merge adjacent tiles into larger composite shapes
    Composite,
    /// Create one collider per tile
    Individual,
    /// Create polygon colliders (not yet implemented)
    Polygon,
}

impl Default for ColliderType {
    fn default() -> Self {
        ColliderType::Composite
    }
}

/// Collider configuration settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColliderConfiguration {
    /// Type of colliders to generate
    pub collider_type: ColliderType,
    
    /// Collision value to use from IntGrid (default: 1)
    pub collision_value: i64,
    
    /// Auto-regenerate colliders on reload
    pub auto_regenerate: bool,
}

impl Default for ColliderConfiguration {
    fn default() -> Self {
        Self {
            collider_type: ColliderType::Composite,
            collision_value: 1,
            auto_regenerate: true,
        }
    }
}

/// Collider Settings Panel for configuring collider generation
pub struct ColliderSettingsPanel {
    /// Current collider configuration
    pub configuration: ColliderConfiguration,
    
    /// Whether settings have been modified
    pub has_changes: bool,
}

impl ColliderSettingsPanel {
    /// Create a new Collider Settings Panel
    pub fn new() -> Self {
        Self {
            configuration: ColliderConfiguration::default(),
            has_changes: false,
        }
    }
    
    /// Create with specific configuration
    pub fn with_configuration(configuration: ColliderConfiguration) -> Self {
        Self {
            configuration,
            has_changes: false,
        }
    }
    
    /// Render the Collider Settings Panel as a standalone window
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        map_manager: &mut MapManager,
        open: &mut bool,
    ) {
        egui::Window::new("⚙️ Collider Settings")
            .open(open)
            .default_width(350.0)
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui, map_manager);
            });
    }
    
    /// Render the Collider Settings Panel content (for use in docking system)
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        map_manager: &mut MapManager,
    ) {
        ui.heading("Collider Configuration");
        ui.separator();
        
        // Current configuration display
        self.render_current_configuration(ui);
        
        ui.separator();
        
        // Collider type selection
        self.render_collider_type_selection(ui);
        
        ui.separator();
        
        // Collision value input
        self.render_collision_value_input(ui);
        
        ui.separator();
        
        // Auto-regenerate toggle
        self.render_auto_regenerate_toggle(ui);
        
        ui.separator();
        
        // Action buttons
        if self.render_action_buttons_with_apply(ui) {
            // Apply settings to MapManager
            self.apply_to_map_manager(map_manager);
        }
        
        // Show unsaved changes indicator
        if self.has_changes {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚠").color(Color32::from_rgb(255, 200, 0)));
                ui.label(RichText::new("Unsaved changes").color(Color32::from_rgb(255, 200, 0)).italics());
            });
        }
    }
    
    /// Render action buttons with apply functionality (returns true if settings were applied)
    fn render_action_buttons_with_apply(&mut self, ui: &mut egui::Ui) -> bool {
        let mut applied = false;
        
        ui.horizontal(|ui| {
            if ui.button("💾 Apply Settings").clicked() {
                applied = true;
                self.has_changes = false;
            }
            
            if ui.button("↺ Reset to Defaults").clicked() {
                self.configuration = ColliderConfiguration::default();
                self.has_changes = true;
            }
        });
        
        applied
    }
    
    /// Render current configuration display
    fn render_current_configuration(&self, ui: &mut egui::Ui) {
        ui.collapsing(RichText::new("📋 Current Configuration").strong(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Collider Type:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let type_str = match self.configuration.collider_type {
                        ColliderType::Composite => "Composite",
                        ColliderType::Individual => "Individual",
                        ColliderType::Polygon => "Polygon",
                    };
                    ui.label(RichText::new(type_str).strong());
                });
            });
            
            ui.horizontal(|ui| {
                ui.label("Collision Value:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{}", self.configuration.collision_value)).strong());
                });
            });
            
            ui.horizontal(|ui| {
                ui.label("Auto-Regenerate:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let status = if self.configuration.auto_regenerate { "Enabled" } else { "Disabled" };
                    ui.label(RichText::new(status).strong());
                });
            });
        });
    }
    
    /// Render collider type selection (subtask 17.3)
    fn render_collider_type_selection(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Collider Type").strong());
        ui.add_space(5.0);
        
        let mut changed = false;
        
        // Composite radio button
        if ui.radio_value(
            &mut self.configuration.collider_type,
            ColliderType::Composite,
            "Composite"
        ).changed() {
            changed = true;
        }
        ui.label(RichText::new("Merges adjacent tiles into larger shapes for better performance").color(Color32::GRAY).small());
        ui.add_space(5.0);
        
        // Individual radio button
        if ui.radio_value(
            &mut self.configuration.collider_type,
            ColliderType::Individual,
            "Individual"
        ).changed() {
            changed = true;
        }
        ui.label(RichText::new("Creates one collider per tile (higher collider count)").color(Color32::GRAY).small());
        ui.add_space(5.0);
        
        // Polygon radio button (disabled for now)
        ui.add_enabled_ui(false, |ui| {
            ui.radio_value(
                &mut self.configuration.collider_type,
                ColliderType::Polygon,
                "Polygon (Not Implemented)"
            );
        });
        ui.label(RichText::new("Creates polygon colliders following tile edges").color(Color32::GRAY).small().italics());
        
        if changed {
            self.has_changes = true;
        }
    }
    
    /// Render collision value input
    fn render_collision_value_input(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Collision Value").strong());
        ui.add_space(5.0);
        
        ui.horizontal(|ui| {
            ui.label("IntGrid Value:");
            if ui.add(
                egui::DragValue::new(&mut self.configuration.collision_value)
                    .speed(1)
                    .clamp_range(0..=255)
            ).changed() {
                self.has_changes = true;
            }
        });
        
        ui.label(RichText::new("Only tiles with this IntGrid value will generate colliders").color(Color32::GRAY).small());
        ui.label(RichText::new("Default: 1 (solid tiles)").color(Color32::GRAY).small().italics());
    }
    
    /// Render auto-regenerate toggle (subtask 17.6)
    fn render_auto_regenerate_toggle(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Auto-Regenerate").strong());
        ui.add_space(5.0);
        
        if ui.checkbox(
            &mut self.configuration.auto_regenerate,
            "Auto-regenerate colliders on reload"
        ).changed() {
            self.has_changes = true;
        }
        
        ui.label(RichText::new("When enabled, colliders are automatically regenerated when maps are reloaded").color(Color32::GRAY).small());
        ui.label(RichText::new("Useful for hot-reload workflow with LDtk").color(Color32::GRAY).small().italics());
    }
    
    /// Render action buttons
    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("💾 Save Settings").clicked() {
                // Settings will be saved by the caller
                self.has_changes = false;
            }
            
            if ui.button("↺ Reset to Defaults").clicked() {
                self.configuration = ColliderConfiguration::default();
                self.has_changes = true;
            }
        });
    }
    
    /// Apply configuration to MapManager
    pub fn apply_to_map_manager(&self, map_manager: &mut MapManager) {
        map_manager.set_collision_value(self.configuration.collision_value);
        map_manager.set_auto_regenerate_colliders(self.configuration.auto_regenerate);
    }
    
    /// Load configuration from MapManager
    pub fn load_from_map_manager(&mut self, map_manager: &MapManager) {
        self.configuration.collision_value = map_manager.get_collision_value();
        self.configuration.auto_regenerate = map_manager.get_auto_regenerate_colliders();
        self.has_changes = false;
    }
    
    /// Load configuration from project settings
    pub fn load_configuration(&mut self, project_path: &std::path::Path) -> Result<(), String> {
        let settings_path = project_path.join(".kiro/settings/tilemap.json");
        
        if settings_path.exists() {
            let json_str = std::fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings file: {}", e))?;
            
            let config: ColliderConfiguration = serde_json::from_str(&json_str)
                .map_err(|e| format!("Failed to parse settings: {}", e))?;
            
            self.configuration = config;
            self.has_changes = false;
            
            log::info!("Loaded collider configuration from {:?}", settings_path);
            Ok(())
        } else {
            // No settings file exists, use defaults
            self.configuration = ColliderConfiguration::default();
            self.has_changes = false;
            Ok(())
        }
    }
    
    /// Save configuration to project settings
    pub fn save_configuration(&mut self, project_path: &std::path::Path) -> Result<(), String> {
        // Create .kiro/settings directory if it doesn't exist
        let settings_dir = project_path.join(".kiro/settings");
        std::fs::create_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        
        let settings_path = settings_dir.join("tilemap.json");
        
        // Serialize configuration to JSON
        let json_str = serde_json::to_string_pretty(&self.configuration)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        // Write to file
        std::fs::write(&settings_path, json_str)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        self.has_changes = false;
        
        log::info!("Saved collider configuration to {:?}", settings_path);
        Ok(())
    }
    
    /// Get the current configuration
    pub fn get_configuration(&self) -> &ColliderConfiguration {
        &self.configuration
    }
    
    /// Set the configuration
    pub fn set_configuration(&mut self, configuration: ColliderConfiguration) {
        self.configuration = configuration;
        self.has_changes = true;
    }
}

impl Default for ColliderSettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}
