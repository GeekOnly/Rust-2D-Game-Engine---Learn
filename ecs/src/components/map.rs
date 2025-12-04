use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Map component for LDtk/Tiled integration
/// 
/// This component represents a map/level file that can be loaded
/// and hot-reloaded in the game engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    /// Path to the map file (.ldtk or .tmx)
    pub file_path: String,
    
    /// Map type (LDtk or Tiled)
    pub map_type: MapType,
    
    /// Whether hot-reload is enabled for this map
    #[serde(default = "default_hot_reload")]
    pub hot_reload_enabled: bool,
    
    /// Map display name (optional)
    #[serde(default)]
    pub display_name: String,
    
    /// Whether the map is currently loaded
    #[serde(skip)]
    pub is_loaded: bool,
    
    /// Entities spawned from this map (runtime only)
    #[serde(skip)]
    pub spawned_entities: Vec<u32>,
}

fn default_hot_reload() -> bool {
    cfg!(debug_assertions) // Enable hot-reload in debug builds by default
}

impl Default for Map {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            map_type: MapType::LDtk,
            hot_reload_enabled: default_hot_reload(),
            display_name: String::new(),
            is_loaded: false,
            spawned_entities: Vec::new(),
        }
    }
}

impl Map {
    /// Create a new Map component
    pub fn new(file_path: impl Into<String>, map_type: MapType) -> Self {
        let file_path = file_path.into();
        let display_name = PathBuf::from(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Map")
            .to_string();
        
        Self {
            file_path,
            map_type,
            hot_reload_enabled: default_hot_reload(),
            display_name,
            is_loaded: false,
            spawned_entities: Vec::new(),
        }
    }
    
    /// Create a new LDtk map
    pub fn ldtk(file_path: impl Into<String>) -> Self {
        Self::new(file_path, MapType::LDtk)
    }
    
    /// Create a new Tiled map
    pub fn tiled(file_path: impl Into<String>) -> Self {
        Self::new(file_path, MapType::Tiled)
    }
    
    /// Get the file extension
    pub fn extension(&self) -> Option<String> {
        PathBuf::from(&self.file_path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
    
    /// Check if the file path is valid
    pub fn is_valid_path(&self) -> bool {
        !self.file_path.is_empty() && self.extension().is_some()
    }
    
    /// Get the absolute path (if project path is provided)
    pub fn absolute_path(&self, project_path: &std::path::Path) -> PathBuf {
        project_path.join(&self.file_path)
    }
}

/// Map file type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapType {
    /// LDtk map file (.ldtk)
    LDtk,
    /// Tiled map file (.tmx, .json)
    Tiled,
}

impl MapType {
    /// Get the display name
    pub fn display_name(&self) -> &str {
        match self {
            MapType::LDtk => "LDtk",
            MapType::Tiled => "Tiled",
        }
    }
    
    /// Get the file extensions for this map type
    pub fn extensions(&self) -> &[&str] {
        match self {
            MapType::LDtk => &["ldtk"],
            MapType::Tiled => &["tmx", "json"],
        }
    }
    
    /// Get the editor executable name
    pub fn editor_executable(&self) -> &str {
        match self {
            MapType::LDtk => "ldtk",
            MapType::Tiled => "tiled",
        }
    }
    
    /// Get the editor download URL
    pub fn editor_url(&self) -> &str {
        match self {
            MapType::LDtk => "https://ldtk.io/",
            MapType::Tiled => "https://www.mapeditor.org/",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_creation() {
        let map = Map::ldtk("levels/world.ldtk");
        assert_eq!(map.file_path, "levels/world.ldtk");
        assert_eq!(map.map_type, MapType::LDtk);
        assert_eq!(map.display_name, "world");
    }

    #[test]
    fn test_map_extension() {
        let map = Map::ldtk("levels/world.ldtk");
        assert_eq!(map.extension(), Some("ldtk".to_string()));
    }

    #[test]
    fn test_map_type_extensions() {
        assert_eq!(MapType::LDtk.extensions(), &["ldtk"]);
        assert_eq!(MapType::Tiled.extensions(), &["tmx", "json"]);
    }
}
