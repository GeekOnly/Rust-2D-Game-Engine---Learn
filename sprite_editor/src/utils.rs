//! Utility Functions
//!
//! Helper functions for file operations and sprite editing.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Create a backup of a file before overwriting
pub fn create_backup<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(());
    }

    // Generate backup filename with timestamp
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs();

    let backup_path = path.with_extension(format!(
        "{}.backup_{}",
        path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or(""),
        timestamp
    ));

    // Copy file to backup
    fs::copy(path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    log::info!("Created backup at: {}", backup_path.display());

    Ok(())
}

/// Generate a unique sprite name based on existing names
pub fn generate_unique_name(base_name: &str, existing_names: &[String]) -> String {
    let mut name = base_name.to_string();
    let mut counter = 1;

    while existing_names.contains(&name) {
        name = format!("{}_{}", base_name, counter);
        counter += 1;
    }

    name
}

/// Validate sprite bounds
pub fn validate_sprite_bounds(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    texture_width: u32,
    texture_height: u32,
) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("Sprite width and height must be greater than 0".to_string());
    }

    if x + width > texture_width {
        return Err(format!(
            "Sprite extends beyond texture width (x: {}, width: {}, texture_width: {})",
            x, width, texture_width
        ));
    }

    if y + height > texture_height {
        return Err(format!(
            "Sprite extends beyond texture height (y: {}, height: {}, texture_height: {})",
            y, height, texture_height
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_name() {
        let existing = vec!["sprite".to_string(), "sprite_1".to_string()];
        let unique = generate_unique_name("sprite", &existing);
        assert_eq!(unique, "sprite_2");
    }

    #[test]
    fn test_validate_sprite_bounds() {
        // Valid bounds
        assert!(validate_sprite_bounds(0, 0, 100, 100, 256, 256).is_ok());

        // Invalid: zero width
        assert!(validate_sprite_bounds(0, 0, 0, 100, 256, 256).is_err());

        // Invalid: extends beyond texture
        assert!(validate_sprite_bounds(200, 0, 100, 100, 256, 256).is_err());
    }
}
