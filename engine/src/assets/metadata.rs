use crate::assets::core::{AssetId, AssetType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
        // Using YAML for meta files is common (like Unity), or JSON. 
        // Let's stick to JSON for simplicity now, or match existing if any.
        // Assuming JSON based on previous context.
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
    use std::io::Write;
    use tempfile::tempdir;

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
        assert_eq!(meta.asset_type, deserialized.asset_type);
    }
    
    #[test]
    fn test_meta_path_generation() {
        let asset_path = Path::new("assets/player.png");
        let meta_path = AssetMetadata::get_meta_path(asset_path);
        assert_eq!(meta_path, PathBuf::from("assets/player.png.meta"));
    }
}
