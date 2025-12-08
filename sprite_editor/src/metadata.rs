//! Sprite Metadata
//!
//! Data structures for sprite definitions and sprite sheet metadata.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Export format for sprite metadata
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Xml,
    TexturePacker,
}

/// Represents a single sprite definition within a sprite sheet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteDefinition {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SpriteDefinition {
    /// Create a new sprite definition
    pub fn new(name: String, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }
}

/// Metadata for a sprite sheet containing multiple sprites
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteMetadata {
    pub texture_path: String,
    pub texture_width: u32,
    pub texture_height: u32,
    pub sprites: Vec<SpriteDefinition>,
}

impl SpriteMetadata {
    /// Create a new sprite metadata
    pub fn new(texture_path: String, texture_width: u32, texture_height: u32) -> Self {
        Self {
            texture_path,
            texture_width,
            texture_height,
            sprites: Vec::new(),
        }
    }

    /// Add a sprite to the metadata
    pub fn add_sprite(&mut self, sprite: SpriteDefinition) {
        self.sprites.push(sprite);
    }

    /// Remove a sprite by index
    pub fn remove_sprite(&mut self, index: usize) -> Option<SpriteDefinition> {
        if index < self.sprites.len() {
            Some(self.sprites.remove(index))
        } else {
            None
        }
    }

    /// Find a sprite by name
    pub fn find_sprite(&self, name: &str) -> Option<&SpriteDefinition> {
        self.sprites.iter().find(|s| s.name == name)
    }

    /// Check if a sprite name already exists
    pub fn has_sprite_name(&self, name: &str) -> bool {
        self.sprites.iter().any(|s| s.name == name)
    }

    /// Save sprite metadata to a .sprite JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();

        // Create backup if file exists
        if path.exists() {
            crate::utils::create_backup(path)?;
        }

        // Serialize to JSON with pretty formatting
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize sprite metadata: {}", e))?;

        // Write to file
        fs::write(path, json)
            .map_err(|e| format!("Failed to write sprite file: {}", e))?;

        Ok(())
    }

    /// Load sprite metadata from a .sprite JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();

        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read sprite file: {}", e))?;

        // Deserialize from JSON
        let mut metadata: SpriteMetadata = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse sprite JSON: {}", e))?;

        // Normalize texture path (convert absolute paths to relative)
        metadata.texture_path = Self::normalize_texture_path(&metadata.texture_path);

        Ok(metadata)
    }

    /// Normalize texture path - extract just the filename if it's an absolute path
    fn normalize_texture_path(path: &str) -> String {
        use std::path::Path as StdPath;

        // Check if it's an absolute path (Windows or Unix style)
        if path.contains(":\\") || path.starts_with('/') {
            // Extract just the filename
            if let Some(filename) = StdPath::new(path).file_name() {
                if let Some(name) = filename.to_str() {
                    // Return as assets/filename
                    log::info!("Normalized absolute path '{}' to 'assets/{}'", path, name);
                    return format!("assets/{}", name);
                }
            }
        }

        // Already relative, return as-is
        path.to_string()
    }

    /// Export sprite metadata to a file in the specified format
    pub fn export<P: AsRef<Path>>(&self, path: P, format: ExportFormat) -> Result<(), String> {
        let path = path.as_ref();

        let content = match format {
            ExportFormat::Json => self.export_to_json()?,
            ExportFormat::Xml => self.export_to_xml()?,
            ExportFormat::TexturePacker => self.export_to_texture_packer()?,
        };

        // Write to file
        fs::write(path, content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        Ok(())
    }

    /// Export to standard JSON format
    fn export_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

    /// Export to XML format
    fn export_to_xml(&self) -> Result<String, String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<SpriteSheet>\n");
        xml.push_str(&format!("  <TexturePath>{}</TexturePath>\n", self.texture_path));
        xml.push_str(&format!("  <TextureWidth>{}</TextureWidth>\n", self.texture_width));
        xml.push_str(&format!("  <TextureHeight>{}</TextureHeight>\n", self.texture_height));
        xml.push_str("  <Sprites>\n");

        for sprite in &self.sprites {
            xml.push_str("    <Sprite>\n");
            xml.push_str(&format!("      <Name>{}</Name>\n", sprite.name));
            xml.push_str(&format!("      <X>{}</X>\n", sprite.x));
            xml.push_str(&format!("      <Y>{}</Y>\n", sprite.y));
            xml.push_str(&format!("      <Width>{}</Width>\n", sprite.width));
            xml.push_str(&format!("      <Height>{}</Height>\n", sprite.height));
            xml.push_str("    </Sprite>\n");
        }

        xml.push_str("  </Sprites>\n");
        xml.push_str("</SpriteSheet>\n");

        Ok(xml)
    }

    /// Export to TexturePacker format (JSON)
    fn export_to_texture_packer(&self) -> Result<String, String> {
        let mut tp_data = serde_json::json!({
            "frames": {},
            "meta": {
                "app": "XS Game Engine Sprite Editor",
                "version": "1.0",
                "image": self.texture_path,
                "format": "RGBA8888",
                "size": {
                    "w": self.texture_width,
                    "h": self.texture_height
                },
                "scale": "1"
            }
        });

        // Add each sprite as a frame
        if let Some(frames) = tp_data.get_mut("frames") {
            if let Some(frames_obj) = frames.as_object_mut() {
                for sprite in &self.sprites {
                    let frame_data = serde_json::json!({
                        "frame": {
                            "x": sprite.x,
                            "y": sprite.y,
                            "w": sprite.width,
                            "h": sprite.height
                        },
                        "rotated": false,
                        "trimmed": false,
                        "spriteSourceSize": {
                            "x": 0,
                            "y": 0,
                            "w": sprite.width,
                            "h": sprite.height
                        },
                        "sourceSize": {
                            "w": sprite.width,
                            "h": sprite.height
                        }
                    });

                    frames_obj.insert(sprite.name.clone(), frame_data);
                }
            }
        }

        serde_json::to_string_pretty(&tp_data)
            .map_err(|e| format!("Failed to serialize TexturePacker format: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_definition_new() {
        let sprite = SpriteDefinition::new("test".to_string(), 10, 20, 30, 40);
        assert_eq!(sprite.name, "test");
        assert_eq!(sprite.x, 10);
        assert_eq!(sprite.y, 20);
        assert_eq!(sprite.width, 30);
        assert_eq!(sprite.height, 40);
    }

    #[test]
    fn test_sprite_metadata_operations() {
        let mut metadata = SpriteMetadata::new("test.png".to_string(), 256, 256);

        // Add sprite
        metadata.add_sprite(SpriteDefinition::new("sprite1".to_string(), 0, 0, 32, 32));
        assert_eq!(metadata.sprites.len(), 1);

        // Find sprite
        assert!(metadata.find_sprite("sprite1").is_some());
        assert!(metadata.find_sprite("sprite2").is_none());

        // Check duplicate name
        assert!(metadata.has_sprite_name("sprite1"));
        assert!(!metadata.has_sprite_name("sprite2"));

        // Remove sprite
        let removed = metadata.remove_sprite(0);
        assert!(removed.is_some());
        assert_eq!(metadata.sprites.len(), 0);
    }

    #[test]
    fn test_normalize_texture_path() {
        // Absolute Windows path
        let path = "C:\\Users\\Test\\file.png";
        let normalized = SpriteMetadata::normalize_texture_path(path);
        assert_eq!(normalized, "assets/file.png");

        // Absolute Unix path
        let path = "/home/user/file.png";
        let normalized = SpriteMetadata::normalize_texture_path(path);
        assert_eq!(normalized, "assets/file.png");

        // Already relative
        let path = "assets/file.png";
        let normalized = SpriteMetadata::normalize_texture_path(path);
        assert_eq!(normalized, "assets/file.png");
    }
}
