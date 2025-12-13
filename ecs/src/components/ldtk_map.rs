use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LDtk Map component - represents a loaded LDtk map file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkMap {
    /// Path to the LDtk file
    pub file_path: String,
    
    /// Map identifier from LDtk
    pub identifier: String,
    
    /// World size in pixels
    pub world_width: i32,
    pub world_height: i32,
    
    /// Default grid size
    pub default_grid_size: i32,
    
    /// Background color
    pub bg_color: String,
    
    /// All levels in this map
    pub levels: Vec<LdtkLevel>,
    
    /// Tileset definitions
    pub tilesets: HashMap<String, LdtkTilesetDef>,
    
    /// Layer definitions
    pub layer_definitions: HashMap<String, LdtkLayerDef>,
    
    /// Entity definitions
    pub entity_definitions: HashMap<String, LdtkEntityDef>,
    
    /// Current active level (for single-level display)
    pub current_level: Option<String>,
    
    /// Auto-reload when file changes
    pub auto_reload: bool,
}

impl Default for LdtkMap {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            identifier: String::new(),
            world_width: 0,
            world_height: 0,
            default_grid_size: 16,
            bg_color: "#40465B".to_string(),
            levels: Vec::new(),
            tilesets: HashMap::new(),
            layer_definitions: HashMap::new(),
            entity_definitions: HashMap::new(),
            current_level: None,
            auto_reload: true,
        }
    }
}

/// LDtk Level data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkLevel {
    /// Level identifier
    pub identifier: String,
    
    /// Level position in world
    pub world_x: i32,
    pub world_y: i32,
    
    /// Level size
    pub px_width: i32,
    pub px_height: i32,
    
    /// Background color (optional)
    pub bg_color: Option<String>,
    
    /// All layer instances in this level
    pub layers: Vec<LdtkLayerInstance>,
    
    /// Entities in this level
    pub entities: Vec<LdtkEntityInstance>,
}

/// LDtk Layer Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkLayerInstance {
    /// Layer identifier
    pub identifier: String,
    
    /// Layer type
    pub layer_type: LdtkLayerType,
    
    /// Grid size for this layer
    pub grid_size: i32,
    
    /// Layer position offset
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    
    /// Layer opacity (0.0 - 1.0)
    pub opacity: f32,
    
    /// Is layer visible
    pub visible: bool,
    
    /// Tileset used (for Tiles layers)
    pub tileset_def_uid: Option<i32>,
    
    /// Layer data based on type
    pub data: LdtkLayerData,
}

/// LDtk Layer Data (union type)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LdtkLayerData {
    /// IntGrid layer data
    IntGrid {
        /// Grid values (0 = empty)
        values: Vec<i32>,
        /// Grid width in cells
        c_width: i32,
        /// Grid height in cells
        c_height: i32,
    },
    
    /// Tiles layer data
    Tiles {
        /// Tile instances
        tiles: Vec<LdtkTileInstance>,
        /// Grid width in cells
        c_width: i32,
        /// Grid height in cells
        c_height: i32,
    },
    
    /// AutoLayer data (generated from IntGrid)
    AutoLayer {
        /// Generated tile instances
        tiles: Vec<LdtkTileInstance>,
        /// Source IntGrid values
        int_grid: Vec<i32>,
        /// Grid width in cells
        c_width: i32,
        /// Grid height in cells
        c_height: i32,
    },
    
    /// Entities layer
    Entities {
        /// Entity instances
        entities: Vec<LdtkEntityInstance>,
    },
}

/// LDtk Layer Type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LdtkLayerType {
    IntGrid,
    Entities,
    Tiles,
    AutoLayer,
}

/// LDtk Tile Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkTileInstance {
    /// Tile ID in tileset
    pub tile_id: u32,
    
    /// Position in grid
    pub grid_x: i32,
    pub grid_y: i32,
    
    /// Pixel position
    pub px_x: i32,
    pub px_y: i32,
    
    /// Flip flags
    pub flip_x: bool,
    pub flip_y: bool,
}

/// LDtk Entity Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkEntityInstance {
    /// Entity identifier
    pub identifier: String,
    
    /// Entity definition UID
    pub def_uid: i32,
    
    /// Position in pixels
    pub px_x: i32,
    pub px_y: i32,
    
    /// Size in pixels
    pub width: i32,
    pub height: i32,
    
    /// Custom fields
    pub field_instances: HashMap<String, LdtkFieldValue>,
}

/// LDtk Field Value (simplified)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LdtkFieldValue {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Color(String),
    Point { x: i32, y: i32 },
    Array(Vec<LdtkFieldValue>),
}

/// LDtk Tileset Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkTilesetDef {
    /// Tileset identifier
    pub identifier: String,
    
    /// UID
    pub uid: i32,
    
    /// Texture path
    pub rel_path: String,
    
    /// Tile size
    pub tile_grid_size: i32,
    
    /// Tileset size in pixels
    pub px_width: i32,
    pub px_height: i32,
    
    /// Spacing between tiles
    pub spacing: i32,
    
    /// Padding around tileset
    pub padding: i32,
}

/// LDtk Layer Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkLayerDef {
    /// Layer identifier
    pub identifier: String,
    
    /// Layer type
    pub layer_type: LdtkLayerType,
    
    /// UID
    pub uid: i32,
    
    /// Grid size
    pub grid_size: i32,
    
    /// Display opacity
    pub display_opacity: f32,
    
    /// IntGrid values (for IntGrid layers)
    pub int_grid_values: Vec<LdtkIntGridValue>,
    
    /// Tileset UID (for Tiles/AutoLayer)
    pub tileset_def_uid: Option<i32>,
}

/// LDtk IntGrid Value Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkIntGridValue {
    /// Value (1, 2, 3, etc.)
    pub value: i32,
    
    /// Identifier/name
    pub identifier: String,
    
    /// Color
    pub color: String,
}

/// LDtk Entity Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkEntityDef {
    /// Entity identifier
    pub identifier: String,
    
    /// UID
    pub uid: i32,
    
    /// Size in pixels
    pub width: i32,
    pub height: i32,
    
    /// Color
    pub color: String,
    
    /// Field definitions
    pub field_defs: Vec<LdtkFieldDef>,
}

/// LDtk Field Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LdtkFieldDef {
    /// Field identifier
    pub identifier: String,
    
    /// Field type
    pub field_type: String,
    
    /// Default value
    pub default_value: Option<LdtkFieldValue>,
}

impl LdtkMap {
    /// Create a new empty LDtk map
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            ..Default::default()
        }
    }
    
    /// Get a level by identifier
    pub fn get_level(&self, identifier: &str) -> Option<&LdtkLevel> {
        self.levels.iter().find(|l| l.identifier == identifier)
    }
    
    /// Get a layer definition by identifier
    pub fn get_layer_def(&self, identifier: &str) -> Option<&LdtkLayerDef> {
        self.layer_definitions.get(identifier)
    }
    
    /// Get a tileset definition by UID
    pub fn get_tileset_def(&self, uid: i32) -> Option<&LdtkTilesetDef> {
        self.tilesets.values().find(|t| t.uid == uid)
    }
    
    /// Set the current active level
    pub fn set_current_level(&mut self, identifier: Option<String>) {
        self.current_level = identifier;
    }
}

impl LdtkLayerInstance {
    /// Get IntGrid value at position
    pub fn get_int_grid_value(&self, grid_x: i32, grid_y: i32) -> Option<i32> {
        match &self.data {
            LdtkLayerData::IntGrid { values, c_width, c_height } |
            LdtkLayerData::AutoLayer { int_grid: values, c_width, c_height, .. } => {
                if grid_x < 0 || grid_y < 0 || grid_x >= *c_width || grid_y >= *c_height {
                    return None;
                }
                let index = (grid_y * c_width + grid_x) as usize;
                values.get(index).copied()
            }
            _ => None,
        }
    }
    
    /// Get all tiles in this layer
    pub fn get_tiles(&self) -> Vec<&LdtkTileInstance> {
        match &self.data {
            LdtkLayerData::Tiles { tiles, .. } |
            LdtkLayerData::AutoLayer { tiles, .. } => tiles.iter().collect(),
            _ => Vec::new(),
        }
    }
    
    /// Get all entities in this layer
    pub fn get_entities(&self) -> Vec<&LdtkEntityInstance> {
        match &self.data {
            LdtkLayerData::Entities { entities } => entities.iter().collect(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldtk_map_creation() {
        let map = LdtkMap::new("test.ldtk");
        assert_eq!(map.file_path, "test.ldtk");
        assert_eq!(map.default_grid_size, 16);
        assert!(map.auto_reload);
    }

    #[test]
    fn test_int_grid_value_access() {
        let layer = LdtkLayerInstance {
            identifier: "test".to_string(),
            layer_type: LdtkLayerType::IntGrid,
            grid_size: 16,
            px_offset_x: 0,
            px_offset_y: 0,
            opacity: 1.0,
            visible: true,
            tileset_def_uid: None,
            data: LdtkLayerData::IntGrid {
                values: vec![0, 1, 2, 0, 1, 0],
                c_width: 3,
                c_height: 2,
            },
        };

        assert_eq!(layer.get_int_grid_value(0, 0), Some(0));
        assert_eq!(layer.get_int_grid_value(1, 0), Some(1));
        assert_eq!(layer.get_int_grid_value(2, 0), Some(2));
        assert_eq!(layer.get_int_grid_value(0, 1), Some(0));
        assert_eq!(layer.get_int_grid_value(1, 1), Some(1));
        assert_eq!(layer.get_int_grid_value(2, 1), Some(0));
        
        // Out of bounds
        assert_eq!(layer.get_int_grid_value(3, 0), None);
        assert_eq!(layer.get_int_grid_value(0, 2), None);
        assert_eq!(layer.get_int_grid_value(-1, 0), None);
    }
}