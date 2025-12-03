/// Example demonstrating how to load sprite sheets from .sprite files
/// 
/// This example shows how to use the SpriteSheet::from_sprite_file method
/// to load sprite metadata created by the Sprite Editor.

use ecs::components::sprite_sheet::SpriteSheet;
use std::path::Path;

fn main() {
    println!("=== Sprite Sheet Loading Example ===\n");

    // Example 1: Load a sprite sheet from a .sprite file
    println!("Loading sprite sheet from test_sprite.sprite...");
    match SpriteSheet::from_sprite_file(Path::new("test_sprite.sprite")) {
        Ok(sprite_sheet) => {
            println!("✓ Successfully loaded sprite sheet!");
            println!("  Texture: {}", sprite_sheet.texture_path);
            println!("  Dimensions: {}x{}", sprite_sheet.sheet_width, sprite_sheet.sheet_height);
            println!("  Frame count: {}\n", sprite_sheet.frames.len());

            // Example 2: Access sprites by index
            println!("Accessing sprites by index:");
            for (i, frame) in sprite_sheet.frames.iter().enumerate() {
                if let Some(name) = &frame.name {
                    println!("  [{}] {} - Position: ({}, {}), Size: {}x{}", 
                        i, name, frame.x, frame.y, frame.width, frame.height);
                }
            }
            println!();

            // Example 3: Access sprites by name
            println!("Accessing sprites by name:");
            if let Some(frame) = sprite_sheet.get_frame_by_name("knight_idle") {
                println!("  Found 'knight_idle' at ({}, {})", frame.x, frame.y);
            } else {
                println!("  'knight_idle' not found");
            }

            if let Some(frame) = sprite_sheet.get_frame_by_name("sprite_0") {
                println!("  Found 'sprite_0' at ({}, {})", frame.x, frame.y);
            }
            println!();

            // Example 4: Use in a game loop (pseudo-code)
            println!("Usage in game loop:");
            println!("  // Get the current animation frame");
            println!("  let frame_index = animated_sprite.get_frame_index();");
            println!("  if let Some(frame) = sprite_sheet.get_frame(frame_index) {{");
            println!("      // Render the sprite using frame.x, frame.y, frame.width, frame.height");
            println!("      // to calculate UV coordinates for the texture");
            println!("  }}");
        }
        Err(e) => {
            println!("✗ Failed to load sprite sheet: {}", e);
            println!("\nNote: Make sure test_sprite.sprite exists in the current directory.");
            println!("You can create one using the Sprite Editor or by running tests.");
        }
    }

    println!("\n=== Example Complete ===");
}
