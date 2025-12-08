

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Tilemap configuration settings stored in project settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilemapSettings {
    /// Automatically generate colliders when loading maps
    #[serde(default = "default_auto_generate")]
    pub auto_generate_colliders: bool,
    
    /// Collision value to use for collider generation (default: 1)
    #[serde(default = "default_collision_value")]
    pub collision_value: i64,
    
    /// Collider type: "Composite", "Individual", or "Polygon"
    #[serde(default = "default_collider_type")]
    pub collider_type: String,
    
    /// Enable hot-reload for LDtk files
    #[serde(default = "default_hot_reload")]
    pub hot_reload_enabled: bool,
    
    /// Pixels per unit conversion (default: 100.0 - Unity standard)
    /// This should match Sprite.pixels_per_unit for consistent scale
    #[serde(default = "default_pixels_per_unit")]
    pub pixels_per_unit: f32,
}

fn default_auto_generate() -> bool {
    true
}

fn default_collision_value() -> i64 {
    1
}

fn default_collider_type() -> String {
    "Composite".to_string()
}

fn default_hot_reload() -> bool {
    true
}

/// Unity standard: 100 pixels = 1 world unit (1 meter)
/// This ensures consistent scale between sprites and tilemaps
fn default_pixels_per_unit() -> f32 {
    100.0
}

impl Default for TilemapSettings {
    fn default() -> Self {
        Self {
            auto_generate_colliders: default_auto_generate(),
            collision_value: default_collision_value(),
            collider_type: default_collider_type(),
            hot_reload_enabled: default_hot_reload(),
            pixels_per_unit: default_pixels_per_unit(),
        }
    }
}

impl TilemapSettings {
    /// Load tilemap settings from project directory
    /// Returns default settings if file doesn't exist
    pub fn load(project_path: &Path) -> Self {
        let settings_path = Self::get_settings_path(project_path);
        
        if settings_path.exists() {
            match fs::read_to_string(&settings_path) {
                Ok(json) => {
                    match serde_json::from_str(&json) {
                        Ok(settings) => {
                            log::info!("Loaded tilemap settings from {:?}", settings_path);
                            settings
                        }
                        Err(e) => {
                            log::warn!("Failed to parse tilemap settings: {}. Using defaults.", e);
                            Self::default()
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read tilemap settings: {}. Using defaults.", e);
                    Self::default()
                }
            }
        } else {
            log::info!("No tilemap settings found. Using defaults.");
            Self::default()
        }
    }
    
    /// Save tilemap settings to project directory
    pub fn save(&self, project_path: &Path) -> Result<(), String> {
        let settings_path = Self::get_settings_path(project_path);
        
        // Create .kiro/settings directory if it doesn't exist
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize tilemap settings: {}", e))?;
        
        fs::write(&settings_path, json)
            .map_err(|e| format!("Failed to write tilemap settings: {}", e))?;
        
        log::info!("Saved tilemap settings to {:?}", settings_path);
        Ok(())
    }
    
    /// Get the path to the tilemap settings file
    fn get_settings_path(project_path: &Path) -> PathBuf {
        project_path.join(".kiro").join("settings").join("tilemap.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_settings() {
        let settings = TilemapSettings::default();
        assert_eq!(settings.auto_generate_colliders, true);
        assert_eq!(settings.collision_value, 1);
        assert_eq!(settings.collider_type, "Composite");
        assert_eq!(settings.hot_reload_enabled, true);
        assert_eq!(settings.pixels_per_unit, 100.0); // Unity standard
    }
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create custom settings
        let mut settings = TilemapSettings::default();
        settings.auto_generate_colliders = false;
        settings.collision_value = 2;
        settings.collider_type = "Individual".to_string();
        settings.hot_reload_enabled = false;
        settings.pixels_per_unit = 16.0;
        
        // Save settings
        settings.save(project_path).unwrap();
        
        // Load settings
        let loaded = TilemapSettings::load(project_path);
        
        assert_eq!(loaded.auto_generate_colliders, false);
        assert_eq!(loaded.collision_value, 2);
        assert_eq!(loaded.collider_type, "Individual");
        assert_eq!(loaded.hot_reload_enabled, false);
        assert_eq!(loaded.pixels_per_unit, 16.0);
    }
    
    #[test]
    fn test_load_nonexistent_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        let settings = TilemapSettings::load(project_path);
        
        // Should return default settings
        assert_eq!(settings.auto_generate_colliders, true);
        assert_eq!(settings.collision_value, 1);
    }
}
