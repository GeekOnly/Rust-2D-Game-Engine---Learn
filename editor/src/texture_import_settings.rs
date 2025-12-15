//! Texture Import Settings (Unity-style)
//!
//! Manages texture import settings similar to Unity's texture importer.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Texture type determines how the texture is used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureType {
    /// Default texture (no special processing)
    Default,
    /// Sprite texture for 2D games
    Sprite2D,
    /// Normal map
    NormalMap,
    /// UI texture
    UI,
    /// Cursor texture
    Cursor,
    /// Lightmap
    Lightmap,
}

impl Default for TextureType {
    fn default() -> Self {
        Self::Sprite2D
    }
}

/// Sprite mode for sprite textures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteMode {
    /// Single sprite
    Single,
    /// Multiple sprites (sprite sheet)
    Multiple,
}

impl Default for SpriteMode {
    fn default() -> Self {
        Self::Single
    }
}

/// Filter mode for texture sampling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    /// Point (no filter) - best for pixel art
    Point,
    /// Bilinear filtering
    Bilinear,
    /// Trilinear filtering
    Trilinear,
}

impl Default for FilterMode {
    fn default() -> Self {
        Self::Point // Default to Point for pixel art
    }
}

/// Wrap mode for texture coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapMode {
    /// Repeat the texture
    Repeat,
    /// Clamp to edge
    Clamp,
    /// Mirror the texture
    Mirror,
}

impl Default for WrapMode {
    fn default() -> Self {
        Self::Clamp
    }
}

/// Compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    /// No compression (RGBA)
    None,
    /// Low quality compression
    LowQuality,
    /// Normal quality compression
    NormalQuality,
    /// High quality compression
    HighQuality,
}

impl Default for CompressionFormat {
    fn default() -> Self {
        Self::NormalQuality
    }
}

/// Platform-specific override settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverride {
    /// Maximum texture size
    pub max_size: u32,
    /// Compression format
    pub compression: CompressionFormat,
    /// Filter mode
    pub filter_mode: FilterMode,
}

impl Default for PlatformOverride {
    fn default() -> Self {
        Self {
            max_size: 2048,
            compression: CompressionFormat::NormalQuality,
            filter_mode: FilterMode::Point,
        }
    }
}

/// Texture import settings (Unity-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureImportSettings {
    /// Texture type
    pub texture_type: TextureType,
    
    /// Sprite mode (for Sprite2D type)
    pub sprite_mode: SpriteMode,
    
    /// Pixels per unit (for Sprite2D type)
    pub pixels_per_unit: f32,
    
    /// Generate mipmaps
    pub generate_mipmaps: bool,
    
    /// sRGB color space
    pub srgb: bool,
    
    /// Alpha is transparency
    pub alpha_is_transparency: bool,
    
    /// Read/Write enabled (allows CPU access)
    pub read_write_enabled: bool,
    
    /// Filter mode
    pub filter_mode: FilterMode,
    
    /// Wrap mode
    pub wrap_mode: WrapMode,
    
    /// Maximum texture size
    pub max_size: u32,
    
    /// Compression format
    pub compression: CompressionFormat,
    
    /// Platform-specific overrides
    #[serde(default)]
    pub platform_overrides: std::collections::HashMap<String, PlatformOverride>,
}

impl Default for TextureImportSettings {
    fn default() -> Self {
        Self {
            texture_type: TextureType::Sprite2D,
            sprite_mode: SpriteMode::Single,
            pixels_per_unit: 100.0,
            generate_mipmaps: false,
            srgb: true,
            alpha_is_transparency: true,
            read_write_enabled: false,
            filter_mode: FilterMode::Point,
            wrap_mode: WrapMode::Clamp,
            max_size: 2048,
            compression: CompressionFormat::NormalQuality,
            platform_overrides: std::collections::HashMap::new(),
        }
    }
}

impl TextureImportSettings {
    /// Load settings from .meta file
    pub fn load(texture_path: &Path) -> Result<Self, String> {
        let meta_path = Self::get_meta_path(texture_path);
        
        if meta_path.exists() {
            let contents = std::fs::read_to_string(&meta_path)
                .map_err(|e| format!("Failed to read meta file: {}", e))?;
            
            serde_json::from_str(&contents)
                .map_err(|e| format!("Failed to parse meta file: {}", e))
        } else {
            // Return default settings if no meta file exists
            Ok(Self::default())
        }
    }
    
    /// Save settings to .meta file
    pub fn save(&self, texture_path: &Path) -> Result<(), String> {
        let meta_path = Self::get_meta_path(texture_path);
        
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        std::fs::write(&meta_path, contents)
            .map_err(|e| format!("Failed to write meta file: {}", e))?;
        
        Ok(())
    }
    
    /// Get .meta file path for a texture
    fn get_meta_path(texture_path: &Path) -> PathBuf {
        let mut meta_path = texture_path.to_path_buf();
        let current_ext = meta_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        meta_path.set_extension(format!("{}.meta", current_ext));
        meta_path
    }
    
    /// Check if settings need to be applied (meta file is newer than texture)
    pub fn needs_reimport(texture_path: &Path) -> bool {
        let meta_path = Self::get_meta_path(texture_path);
        
        if !meta_path.exists() {
            return false; // No meta file, use defaults
        }
        
        // Check if meta file is newer than texture
        if let (Ok(texture_meta), Ok(meta_meta)) = (
            std::fs::metadata(texture_path),
            std::fs::metadata(&meta_path)
        ) {
            if let (Ok(texture_time), Ok(meta_time)) = (
                texture_meta.modified(),
                meta_meta.modified()
            ) {
                return meta_time > texture_time;
            }
        }
        
        false
    }
}
