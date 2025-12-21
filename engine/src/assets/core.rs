use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use uuid::Uuid;

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
                "json" => Self::Scene, // Assuming scenes are JSON for now, could be specific ext
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_asset_id_generation() {
        let id1 = AssetId::new();
        let id2 = AssetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_asset_id_parsing() {
        let id = AssetId::new();
        let id_str = id.to_string();
        let parsed_id = AssetId::from_uuid_str(&id_str).unwrap();
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_asset_type_detection() {
        assert_eq!(AssetType::from_path(Path::new("assets/player.png")), AssetType::Texture);
        assert_eq!(AssetType::from_path(Path::new("assets/level.json")), AssetType::Scene);
        assert_eq!(AssetType::from_path(Path::new("assets/script.lua")), AssetType::Script);
        assert_eq!(AssetType::from_path(Path::new("assets/unknown.xyz")), AssetType::Unknown);
        // Note: is_dir check requires actual filesystem or mocking, 
        // relying on path extension logic for unit tests mostly.
    }
}
