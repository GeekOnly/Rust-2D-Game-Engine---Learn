use serde::{Deserialize, Serialize};

/// Root object of the LDtk JSON file
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkJson {
    /// Project background color
    pub bg_color: String,
    
    /// Default grid size for new layers
    pub default_grid_size: i32,
    
    /// Definitions of entities, layers, tilesets...
    pub defs: LdtkDefs,
    
    /// All levels
    pub levels: Vec<LdtkLevel>,
    
    /// World layout type (Free, GridVania, LinearHorizontal, LinearVertical)
    pub world_layout: Option<String>,
    
    /// World grid width (if GridVania)
    pub world_grid_width: Option<i32>,
    
    /// World grid height (if GridVania)
    pub world_grid_height: Option<i32>,

    /// True if external levels are used
    pub external_levels: bool,
}

/// Definitions of sub-elements
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkDefs {
    pub layers: Vec<LayerDef>,
    pub entities: Vec<EntityDef>,
    pub tilesets: Vec<TilesetDef>,
}

/// Level data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLevel {
    pub identifier: String,
    pub iid: String,
    pub uid: i32,
    pub world_x: i32,
    pub world_y: i32,
    pub px_wid: i32,
    pub px_hei: i32,
    
    #[serde(default)]
    pub layer_instances: Option<Vec<LayerInstance>>,
}

/// Layer Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerDef {
    #[serde(rename = "__type")]
    pub layer_type: String, // "IntGrid", "Entities", "Tiles", "AutoLayer"
    pub identifier: String,
    pub uid: i32,
    pub grid_size: i32,
    pub display_opacity: f32,
    pub px_offset_x: i32,
    pub px_offset_y: i32,
    pub tileset_def_uid: Option<i32>,
}

/// Entity Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityDef {
    pub identifier: String,
    pub uid: i32,
    pub width: i32,
    pub height: i32,
    pub color: String,
    pub field_defs: Vec<FieldDef>,
}

/// Field Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDef {
    pub identifier: String,
    #[serde(rename = "__type")]
    pub field_type: String,
}

/// Tileset Definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesetDef {
    pub identifier: String,
    pub uid: i32,
    pub rel_path: Option<String>,
    pub px_wid: i32,
    pub px_hei: i32,
    pub tile_grid_size: i32,
    pub spacing: i32,
    pub padding: i32,
}

/// Layer Instance (The actual layer data in a level)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerInstance {
    pub __identifier: String,
    pub __type: String, // "IntGrid", "Entities", "Tiles", "AutoLayer"
    pub __c_wid: i32,
    pub __c_hei: i32,
    pub __grid_size: i32,
    pub __opacity: f32,
    pub __px_total_offset_x: i32,
    pub __px_total_offset_y: i32,
    pub __tileset_def_uid: Option<i32>,
    pub layer_def_uid: i32,
    pub level_id: i32,
    pub visible: bool,
    
    /// Grid-based IntGrid values
    #[serde(default)]
    pub int_grid_csv: Vec<i32>,
    
    /// Auto-layer tiles
    #[serde(default)]
    pub auto_layer_tiles: Vec<TileInstance>,
    
    /// Grid tiles
    #[serde(default)]
    pub grid_tiles: Vec<TileInstance>,
    
    /// Entity instances
    #[serde(default)]
    pub entity_instances: Vec<EntityInstance>,
}

/// Tile Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileInstance {
    #[serde(rename = "px")]
    pub px: [i32; 2],
    #[serde(rename = "src")]
    pub src: [i32; 2],
    #[serde(rename = "f")]
    pub f: i32, // flip bits
    #[serde(rename = "t")]
    pub t: i32, // tile id
    #[serde(rename = "d")]
    pub d: [i32; 1], // internal data
}

/// Entity Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityInstance {
    pub __identifier: String,
    pub __grid: [i32; 2],
    pub __pivot: [f32; 2],
    pub __tags: Vec<String>,
    pub width: i32,
    pub height: i32,
    pub def_uid: i32,
    #[serde(rename = "px")]
    pub px: [i32; 2],
    pub iid: String,
    pub field_instances: Vec<FieldInstance>,
}

/// Field Instance
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldInstance {
    pub __identifier: String,
    #[serde(rename = "__type")]
    pub __type: String,
    pub __value: serde_json::Value, // Dynamic value
    pub def_uid: i32,
}