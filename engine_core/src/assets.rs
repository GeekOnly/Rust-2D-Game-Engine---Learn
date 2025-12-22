use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ==================================================================================
// Core Types (Moved from engine/src/assets/core.rs)
// ==================================================================================

/// Unique identifier for an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    /// Generates a new random AssetId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an AssetId from a specific UUID string.
    pub fn from_uuid_str(uuid_str: &str) -> Option<Self> {
        Uuid::parse_str(uuid_str).ok().map(Self)
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of asset, determined by file extension or metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    SpriteSheet,
    Scene,
    Script,
    Audio,
    Font,
    Model,
    Prefab,
    Folder,
    Unknown,
}

impl AssetType {
    /// Determines AssetType from a file path extension.
    pub fn from_path(path: &Path) -> Self {
        if path.is_dir() {
            return Self::Folder;
        }

        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            Some(ext) => match ext.as_str() {
                "png" | "jpg" | "jpeg" | "bmp" => Self::Texture,
                "sprite" => Self::SpriteSheet,
                "json" => Self::Scene, 
                "lua" => Self::Script,
                "wav" | "mp3" | "ogg" => Self::Audio,
                "ttf" | "otf" => Self::Font,
                "obj" | "gltf" | "glb" => Self::Model,
                "prefab" => Self::Prefab,
                _ => Self::Unknown,
            },
            None => Self::Unknown,
        }
    }
}

// ==================================================================================
// Metadata Logic (Moved from engine/src/assets/metadata.rs)
// ==================================================================================

/// Metadata stored in the sidecar `.meta` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: AssetId,
    pub asset_type: AssetType,
    #[serde(default)]
    pub import_settings: ImportSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportSettings {
    pub read_write: bool, // Example setting
    // Add specific settings variants or fields here as needed
}

impl AssetMetadata {
    /// Generates default metadata for a new asset.
    pub fn generate(path: &Path) -> Self {
        Self {
            id: AssetId::new(),
            asset_type: AssetType::from_path(path),
            import_settings: ImportSettings::default(),
        }
    }

    /// Loads metadata from a `.meta` file.
    pub fn load_from_file(meta_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(meta_path)?;
        let metadata: Self = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    /// Saves metadata to a `.meta` file.
    pub fn save_to_file(&self, meta_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(meta_path, content)?;
        Ok(())
    }
    
    /// Get the expected meta file path for a given asset path.
    pub fn get_meta_path(asset_path: &Path) -> PathBuf {
        let mut meta_path = asset_path.to_path_buf();
        if let Some(file_name) = asset_path.file_name() {
             let mut new_name = file_name.to_os_string();
             new_name.push(".meta");
             meta_path.set_file_name(new_name);
        }
        meta_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id_generation() {
        let id1 = AssetId::new();
        let id2 = AssetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_asset_type_detection() {
        assert_eq!(AssetType::from_path(Path::new("test.png")), AssetType::Texture);
    }
    
    #[test]
    fn test_meta_serialization() {
        let meta = AssetMetadata {
            id: AssetId::new(),
            asset_type: AssetType::Texture,
            import_settings: ImportSettings::default(),
        };
        
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: AssetMetadata = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(meta.id, deserialized.id);
    }
}
