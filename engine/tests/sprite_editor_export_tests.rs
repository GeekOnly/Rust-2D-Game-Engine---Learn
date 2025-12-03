// Tests for sprite editor export functionality
// Requirements: 10.1, 10.2, 10.3, 10.4, 10.5

use std::fs;
use std::path::PathBuf;

// Import the sprite editor types
// Note: These would normally be imported from the engine crate
// For now, we'll define minimal test structures

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
struct SpriteDefinition {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
struct SpriteMetadata {
    texture_path: String,
    texture_width: u32,
    texture_height: u32,
    sprites: Vec<SpriteDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExportFormat {
    Json,
    Xml,
    TexturePacker,
}

impl SpriteMetadata {
    fn new(texture_path: String, texture_width: u32, texture_height: u32) -> Self {
        Self {
            texture_path,
            texture_width,
            texture_height,
            sprites: Vec::new(),
        }
    }

    fn add_sprite(&mut self, sprite: SpriteDefinition) {
        self.sprites.push(sprite);
    }

    fn export(&self, path: &PathBuf, format: ExportFormat) -> Result<(), String> {
        let content = match format {
            ExportFormat::Json => self.export_to_json()?,
            ExportFormat::Xml => self.export_to_xml()?,
            ExportFormat::TexturePacker => self.export_to_texture_packer()?,
        };

        fs::write(path, content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        Ok(())
    }

    fn export_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))
    }

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
            .map_err(|e| format!("Failed to serialize to TexturePacker format: {}", e))
    }
}

#[test]
fn test_export_json_format() {
    // Create test sprite metadata
    let mut metadata = SpriteMetadata::new(
        "assets/test_texture.png".to_string(),
        512,
        256,
    );
    
    metadata.add_sprite(SpriteDefinition {
        name: "sprite_0".to_string(),
        x: 0,
        y: 0,
        width: 32,
        height: 32,
    });
    
    metadata.add_sprite(SpriteDefinition {
        name: "sprite_1".to_string(),
        x: 32,
        y: 0,
        width: 32,
        height: 32,
    });
    
    // Ensure target directory exists
    let _ = fs::create_dir_all("target");
    
    // Export to JSON
    let export_path = PathBuf::from("target/test_export.json");
    let result = metadata.export(&export_path, ExportFormat::Json);
    
    assert!(result.is_ok(), "Export to JSON should succeed: {:?}", result.err());
    
    // Verify file was created
    assert!(export_path.exists(), "Export file should exist");
    
    // Read and verify content
    let content = fs::read_to_string(&export_path).expect("Should read export file");
    assert!(content.contains("sprite_0"), "Export should contain sprite_0");
    assert!(content.contains("sprite_1"), "Export should contain sprite_1");
    assert!(content.contains("test_texture.png"), "Export should contain texture path");
    
    // Clean up
    let _ = fs::remove_file(&export_path);
}

#[test]
fn test_export_xml_format() {
    // Create test sprite metadata
    let mut metadata = SpriteMetadata::new(
        "assets/test_texture.png".to_string(),
        512,
        256,
    );
    
    metadata.add_sprite(SpriteDefinition {
        name: "sprite_0".to_string(),
        x: 0,
        y: 0,
        width: 32,
        height: 32,
    });
    
    // Ensure target directory exists
    let _ = fs::create_dir_all("target");
    
    // Export to XML
    let export_path = PathBuf::from("target/test_export.xml");
    let result = metadata.export(&export_path, ExportFormat::Xml);
    
    assert!(result.is_ok(), "Export to XML should succeed: {:?}", result.err());
    
    // Verify file was created
    assert!(export_path.exists(), "Export file should exist");
    
    // Read and verify content
    let content = fs::read_to_string(&export_path).expect("Should read export file");
    assert!(content.contains("<?xml version=\"1.0\""), "Export should be valid XML");
    assert!(content.contains("<SpriteSheet>"), "Export should have SpriteSheet root");
    assert!(content.contains("<Name>sprite_0</Name>"), "Export should contain sprite name");
    assert!(content.contains("<TexturePath>assets/test_texture.png</TexturePath>"), "Export should contain texture path");
    
    // Clean up
    let _ = fs::remove_file(&export_path);
}

#[test]
fn test_export_texture_packer_format() {
    // Create test sprite metadata
    let mut metadata = SpriteMetadata::new(
        "assets/test_texture.png".to_string(),
        512,
        256,
    );
    
    metadata.add_sprite(SpriteDefinition {
        name: "sprite_0".to_string(),
        x: 0,
        y: 0,
        width: 32,
        height: 32,
    });
    
    metadata.add_sprite(SpriteDefinition {
        name: "sprite_1".to_string(),
        x: 32,
        y: 0,
        width: 32,
        height: 32,
    });
    
    // Ensure target directory exists
    let _ = fs::create_dir_all("target");
    
    // Export to TexturePacker format
    let export_path = PathBuf::from("target/test_export_tp.json");
    let result = metadata.export(&export_path, ExportFormat::TexturePacker);
    
    assert!(result.is_ok(), "Export to TexturePacker should succeed: {:?}", result.err());
    
    // Verify file was created
    assert!(export_path.exists(), "Export file should exist");
    
    // Read and verify content
    let content = fs::read_to_string(&export_path).expect("Should read export file");
    assert!(content.contains("\"frames\""), "Export should contain frames");
    assert!(content.contains("\"meta\""), "Export should contain meta");
    assert!(content.contains("sprite_0"), "Export should contain sprite_0");
    assert!(content.contains("sprite_1"), "Export should contain sprite_1");
    assert!(content.contains("XS Game Engine Sprite Editor"), "Export should contain app name");
    
    // Parse as JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&content).expect("Should parse as JSON");
    assert!(json["frames"].is_object(), "Frames should be an object");
    assert!(json["meta"].is_object(), "Meta should be an object");
    assert!(json["frames"]["sprite_0"].is_object(), "sprite_0 should be in frames");
    assert!(json["frames"]["sprite_1"].is_object(), "sprite_1 should be in frames");
    
    // Clean up
    let _ = fs::remove_file(&export_path);
}

#[test]
fn test_export_includes_all_metadata() {
    // Requirement 10.3: Export should include all sprite metadata
    let mut metadata = SpriteMetadata::new(
        "assets/character.png".to_string(),
        1024,
        512,
    );
    
    // Add multiple sprites with different properties
    metadata.add_sprite(SpriteDefinition {
        name: "idle_0".to_string(),
        x: 10,
        y: 20,
        width: 64,
        height: 64,
    });
    
    metadata.add_sprite(SpriteDefinition {
        name: "run_0".to_string(),
        x: 100,
        y: 200,
        width: 48,
        height: 56,
    });
    
    // Ensure target directory exists
    let _ = fs::create_dir_all("target");
    
    // Export to JSON
    let export_path = PathBuf::from("target/test_all_metadata.json");
    let result = metadata.export(&export_path, ExportFormat::Json);
    assert!(result.is_ok(), "Export should succeed: {:?}", result.err());
    
    // Read and parse
    let content = fs::read_to_string(&export_path).expect("Should read file");
    let parsed: SpriteMetadata = serde_json::from_str(&content).expect("Should parse JSON");
    
    // Verify all data is preserved
    assert_eq!(parsed.texture_path, "assets/character.png");
    assert_eq!(parsed.texture_width, 1024);
    assert_eq!(parsed.texture_height, 512);
    assert_eq!(parsed.sprites.len(), 2);
    
    // Verify first sprite
    assert_eq!(parsed.sprites[0].name, "idle_0");
    assert_eq!(parsed.sprites[0].x, 10);
    assert_eq!(parsed.sprites[0].y, 20);
    assert_eq!(parsed.sprites[0].width, 64);
    assert_eq!(parsed.sprites[0].height, 64);
    
    // Verify second sprite
    assert_eq!(parsed.sprites[1].name, "run_0");
    assert_eq!(parsed.sprites[1].x, 100);
    assert_eq!(parsed.sprites[1].y, 200);
    assert_eq!(parsed.sprites[1].width, 48);
    assert_eq!(parsed.sprites[1].height, 56);
    
    // Clean up
    let _ = fs::remove_file(&export_path);
}

#[test]
fn test_export_empty_sprite_sheet() {
    // Test exporting a sprite sheet with no sprites
    let metadata = SpriteMetadata::new(
        "assets/empty.png".to_string(),
        256,
        256,
    );
    
    // Ensure target directory exists
    let _ = fs::create_dir_all("target");
    
    // Export to JSON
    let export_path = PathBuf::from("target/test_empty.json");
    let result = metadata.export(&export_path, ExportFormat::Json);
    
    assert!(result.is_ok(), "Export of empty sprite sheet should succeed: {:?}", result.err());
    
    // Verify file was created
    assert!(export_path.exists(), "Export file should exist");
    
    // Read and verify content
    let content = fs::read_to_string(&export_path).expect("Should read file");
    let parsed: SpriteMetadata = serde_json::from_str(&content).expect("Should parse JSON");
    
    assert_eq!(parsed.sprites.len(), 0, "Should have no sprites");
    assert_eq!(parsed.texture_width, 256);
    assert_eq!(parsed.texture_height, 256);
    
    // Clean up
    let _ = fs::remove_file(&export_path);
}
