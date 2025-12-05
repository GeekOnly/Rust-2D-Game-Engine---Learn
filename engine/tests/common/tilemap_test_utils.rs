// Test utilities for tilemap management tests
// Provides helpers for generating mock LDtk data and verifying tilemap operations

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Create a temporary test directory
pub fn create_test_dir() -> PathBuf {
    let test_dir = std::env::temp_dir().join(format!("tilemap_test_{}", std::process::id()));
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

/// Clean up a test directory
pub fn cleanup_test_dir(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

/// Generate a mock LDtk project JSON with specified parameters
pub fn create_mock_ldtk_project(
    width: u32,
    height: u32,
    grid_size: u32,
    intgrid_data: Option<Vec<i64>>,
) -> Value {
    let intgrid_csv = if let Some(data) = intgrid_data {
        data
    } else {
        // Default: all zeros
        vec![0; (width * height) as usize]
    };

    json!({
        "defaultGridSize": grid_size,
        "levels": [
            {
                "identifier": "Level_Test",
                "worldX": 0,
                "worldY": 0,
                "pxWid": width * grid_size,
                "pxHei": height * grid_size,
                "layerInstances": [
                    {
                        "__identifier": "IntGrid_layer",
                        "__type": "IntGrid",
                        "__cWid": width,
                        "__cHei": height,
                        "__gridSize": grid_size,
                        "__pxTotalOffsetX": 0,
                        "__pxTotalOffsetY": 0,
                        "intGridCsv": intgrid_csv,
                        "autoLayerTiles": []
                    }
                ]
            }
        ],
        "defs": {
            "tilesets": []
        }
    })
}

/// Create a temporary LDtk file with the given JSON data
pub fn create_temp_ldtk_file(test_dir: &Path, name: &str, data: &Value) -> PathBuf {
    let file_path = test_dir.join(format!("{}.ldtk", name));
    let json_str = serde_json::to_string_pretty(data).expect("Failed to serialize JSON");
    fs::write(&file_path, json_str).expect("Failed to write LDtk file");
    file_path
}

/// Generate random IntGrid data with specified density of collision tiles
pub fn generate_random_intgrid(width: u32, height: u32, density: f32, collision_value: i64) -> Vec<i64> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};
    
    let mut data = Vec::with_capacity((width * height) as usize);
    let hasher = RandomState::new();
    
    for y in 0..height {
        for x in 0..width {
            // Use position as seed for deterministic randomness
            let mut h = hasher.build_hasher();
            (x, y).hash(&mut h);
            let hash = h.finish();
            let random_value = (hash % 1000) as f32 / 1000.0;
            
            if random_value < density {
                data.push(collision_value);
            } else {
                data.push(0);
            }
        }
    }
    
    data
}

/// Generate IntGrid data with a specific pattern (e.g., checkerboard, horizontal lines)
pub fn generate_pattern_intgrid(width: u32, height: u32, pattern: IntGridPattern) -> Vec<i64> {
    let mut data = Vec::with_capacity((width * height) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let value = match pattern {
                IntGridPattern::Checkerboard => {
                    if (x + y) % 2 == 0 { 1 } else { 0 }
                }
                IntGridPattern::HorizontalLines => {
                    if y % 2 == 0 { 1 } else { 0 }
                }
                IntGridPattern::VerticalLines => {
                    if x % 2 == 0 { 1 } else { 0 }
                }
                IntGridPattern::SolidBlock { start_x, start_y, block_width, block_height } => {
                    if x >= start_x && x < start_x + block_width 
                        && y >= start_y && y < start_y + block_height {
                        1
                    } else {
                        0
                    }
                }
                IntGridPattern::Border => {
                    if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                        1
                    } else {
                        0
                    }
                }
                IntGridPattern::Empty => 0,
                IntGridPattern::Full => 1,
            };
            data.push(value);
        }
    }
    
    data
}

/// Patterns for generating IntGrid test data
#[derive(Clone, Copy, Debug)]
pub enum IntGridPattern {
    /// Checkerboard pattern (alternating 1s and 0s)
    Checkerboard,
    /// Horizontal lines (every other row is 1)
    HorizontalLines,
    /// Vertical lines (every other column is 1)
    VerticalLines,
    /// Solid rectangular block
    SolidBlock {
        start_x: u32,
        start_y: u32,
        block_width: u32,
        block_height: u32,
    },
    /// Border only (edges are 1, interior is 0)
    Border,
    /// All empty (all 0s)
    Empty,
    /// All full (all 1s)
    Full,
}

/// Count the number of collision tiles in IntGrid data
pub fn count_collision_tiles(intgrid_data: &[i64], collision_value: i64) -> usize {
    intgrid_data.iter().filter(|&&v| v == collision_value).count()
}

/// Find all rectangles of collision tiles (for testing composite collider optimization)
pub fn find_rectangles_in_intgrid(
    intgrid_data: &[i64],
    width: u32,
    height: u32,
    collision_value: i64,
) -> Vec<Rectangle> {
    let mut rectangles = Vec::new();
    let mut visited = vec![false; (width * height) as usize];
    
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            
            if visited[idx] || intgrid_data[idx] != collision_value {
                continue;
            }
            
            // Find the largest rectangle starting at (x, y)
            let rect = find_largest_rectangle(intgrid_data, width, height, x, y, collision_value, &visited);
            
            // Mark all cells in this rectangle as visited
            for ry in rect.y..rect.y + rect.height {
                for rx in rect.x..rect.x + rect.width {
                    let ridx = (ry * width + rx) as usize;
                    visited[ridx] = true;
                }
            }
            
            rectangles.push(rect);
        }
    }
    
    rectangles
}

/// Find the largest rectangle starting at (start_x, start_y)
fn find_largest_rectangle(
    intgrid_data: &[i64],
    width: u32,
    height: u32,
    start_x: u32,
    start_y: u32,
    collision_value: i64,
    visited: &[bool],
) -> Rectangle {
    // Find maximum width
    let mut max_width = 0;
    for x in start_x..width {
        let idx = (start_y * width + x) as usize;
        if visited[idx] || intgrid_data[idx] != collision_value {
            break;
        }
        max_width += 1;
    }
    
    // Find maximum height with this width
    let mut max_height = 1;
    for y in (start_y + 1)..height {
        let mut can_extend = true;
        for x in start_x..(start_x + max_width) {
            let idx = (y * width + x) as usize;
            if visited[idx] || intgrid_data[idx] != collision_value {
                can_extend = false;
                break;
            }
        }
        if !can_extend {
            break;
        }
        max_height += 1;
    }
    
    Rectangle {
        x: start_x,
        y: start_y,
        width: max_width,
        height: max_height,
    }
}

/// Rectangle structure for testing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mock_ldtk_project() {
        let project = create_mock_ldtk_project(10, 10, 8, None);
        
        assert_eq!(project["defaultGridSize"], 8);
        assert!(project["levels"].is_array());
        
        let levels = project["levels"].as_array().unwrap();
        assert_eq!(levels.len(), 1);
        
        let level = &levels[0];
        assert_eq!(level["identifier"], "Level_Test");
    }

    #[test]
    fn test_generate_pattern_intgrid() {
        let data = generate_pattern_intgrid(4, 4, IntGridPattern::Checkerboard);
        
        // Checkerboard pattern
        assert_eq!(data[0], 1); // (0,0)
        assert_eq!(data[1], 0); // (1,0)
        assert_eq!(data[4], 0); // (0,1)
        assert_eq!(data[5], 1); // (1,1)
    }

    #[test]
    fn test_count_collision_tiles() {
        let data = vec![0, 1, 1, 0, 1, 0, 0, 1];
        assert_eq!(count_collision_tiles(&data, 1), 4);
        assert_eq!(count_collision_tiles(&data, 0), 4);
    }

    #[test]
    fn test_find_rectangles_solid_block() {
        // 4x4 grid with a 2x2 solid block in the center
        let data = generate_pattern_intgrid(
            4,
            4,
            IntGridPattern::SolidBlock {
                start_x: 1,
                start_y: 1,
                block_width: 2,
                block_height: 2,
            },
        );
        
        let rectangles = find_rectangles_in_intgrid(&data, 4, 4, 1);
        
        // Should find exactly one 2x2 rectangle
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 2);
        assert_eq!(rectangles[0].height, 2);
        assert_eq!(rectangles[0].area(), 4);
    }

    #[test]
    fn test_find_rectangles_checkerboard() {
        // Checkerboard pattern should produce many 1x1 rectangles
        let data = generate_pattern_intgrid(4, 4, IntGridPattern::Checkerboard);
        let rectangles = find_rectangles_in_intgrid(&data, 4, 4, 1);
        
        // Each collision tile is isolated, so we get 1x1 rectangles
        assert_eq!(rectangles.len(), 8); // Half of 16 tiles
        
        for rect in &rectangles {
            assert_eq!(rect.width, 1);
            assert_eq!(rect.height, 1);
        }
    }
}
