use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
            create_backup(path)?;
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
        let metadata: SpriteMetadata = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse sprite JSON: {}", e))?;

        Ok(metadata)
    }
}

/// Create a backup of an existing file
fn create_backup<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(());
    }

    // Generate backup filename with .bak extension
    let backup_path = path.with_extension("sprite.bak");

    // Copy the file to backup
    fs::copy(path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_path(filename: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        path
    }

    fn cleanup_test_file(path: &Path) {
        let _ = fs::remove_file(path);
        let backup_path = path.with_extension("sprite.bak");
        let _ = fs::remove_file(backup_path);
    }

    #[test]
    fn test_sprite_definition_creation() {
        let sprite = SpriteDefinition::new("test_sprite".to_string(), 10, 20, 32, 32);
        assert_eq!(sprite.name, "test_sprite");
        assert_eq!(sprite.x, 10);
        assert_eq!(sprite.y, 20);
        assert_eq!(sprite.width, 32);
        assert_eq!(sprite.height, 32);
    }

    #[test]
    fn test_sprite_metadata_creation() {
        let metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        assert_eq!(metadata.texture_path, "texture.png");
        assert_eq!(metadata.texture_width, 512);
        assert_eq!(metadata.texture_height, 256);
        assert_eq!(metadata.sprites.len(), 0);
    }

    #[test]
    fn test_add_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        assert_eq!(metadata.sprites.len(), 1);
        assert_eq!(metadata.sprites[0], sprite);
    }

    #[test]
    fn test_remove_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        let removed = metadata.remove_sprite(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), sprite);
        assert_eq!(metadata.sprites.len(), 0);
    }

    #[test]
    fn test_find_sprite() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite.clone());
        
        let found = metadata.find_sprite("sprite_0");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &sprite);
        
        let not_found = metadata.find_sprite("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_has_sprite_name() {
        let mut metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        let sprite = SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32);
        metadata.add_sprite(sprite);
        
        assert!(metadata.has_sprite_name("sprite_0"));
        assert!(!metadata.has_sprite_name("sprite_1"));
    }

    #[test]
    fn test_save_and_load() {
        let test_path = get_test_path("test_sprite.sprite");
        cleanup_test_file(&test_path);

        // Create metadata with sprites
        let mut metadata = SpriteMetadata::new("assets/texture.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("sprite_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("sprite_1".to_string(), 32, 0, 32, 32));

        // Save to file
        let save_result = metadata.save(&test_path);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());

        // Load from file
        let loaded_result = SpriteMetadata::load(&test_path);
        assert!(loaded_result.is_ok(), "Load failed: {:?}", loaded_result.err());
        
        let loaded = loaded_result.unwrap();
        assert_eq!(loaded, metadata);
        assert_eq!(loaded.sprites.len(), 2);
        assert_eq!(loaded.sprites[0].name, "sprite_0");
        assert_eq!(loaded.sprites[1].name, "sprite_1");

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_backup_creation() {
        let test_path = get_test_path("test_backup.sprite");
        cleanup_test_file(&test_path);

        // Create initial file
        let metadata = SpriteMetadata::new("texture.png".to_string(), 512, 256);
        metadata.save(&test_path).unwrap();

        // Save again to trigger backup
        let mut metadata2 = SpriteMetadata::new("texture2.png".to_string(), 1024, 512);
        metadata2.add_sprite(SpriteDefinition::new("new_sprite".to_string(), 0, 0, 64, 64));
        metadata2.save(&test_path).unwrap();

        // Check backup exists
        let backup_path = test_path.with_extension("sprite.bak");
        assert!(backup_path.exists(), "Backup file should exist");

        // Load backup and verify it's the original
        let backup_metadata = SpriteMetadata::load(&backup_path).unwrap();
        assert_eq!(backup_metadata.texture_path, "texture.png");
        assert_eq!(backup_metadata.sprites.len(), 0);

        cleanup_test_file(&test_path);
    }

    #[test]
    fn test_json_format() {
        let test_path = get_test_path("test_format.sprite");
        cleanup_test_file(&test_path);

        let mut metadata = SpriteMetadata::new("assets/knight.png".to_string(), 512, 256);
        metadata.add_sprite(SpriteDefinition::new("knight_idle_0".to_string(), 0, 0, 32, 32));
        metadata.add_sprite(SpriteDefinition::new("knight_run_0".to_string(), 32, 0, 32, 32));

        metadata.save(&test_path).unwrap();

        // Read the raw JSON to verify format
        let json_content = fs::read_to_string(&test_path).unwrap();
        
        // Verify it's valid JSON and contains expected fields
        assert!(json_content.contains("\"texture_path\""));
        assert!(json_content.contains("\"texture_width\""));
        assert!(json_content.contains("\"texture_height\""));
        assert!(json_content.contains("\"sprites\""));
        assert!(json_content.contains("\"knight_idle_0\""));
        assert!(json_content.contains("\"knight_run_0\""));

        cleanup_test_file(&test_path);
    }
}
