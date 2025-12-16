use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single tile in a tileset
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileData {
    /// Tile ID within the tileset
    pub id: u32,
    /// X coordinate in the tileset texture (pixels)
    pub x: u32,
    /// Y coordinate in the tileset texture (pixels)
    pub y: u32,
    /// Width of the tile (pixels)
    pub width: u32,
    /// Height of the tile (pixels)
    pub height: u32,
    /// Custom properties for this tile
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// Helper for deserializing TileSet with path normalization
#[derive(Deserialize)]
struct TileSetRaw {
    name: String,
    texture_path: String,
    texture_id: String,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
    tile_count: u32,
    #[serde(default)]
    spacing: u32,
    #[serde(default)]
    margin: u32,
    #[serde(default)]
    tiles: HashMap<u32, TileData>,
}

/// Tileset component containing tile data and texture information
#[derive(Clone, Debug, Serialize)]
pub struct TileSet {
    /// Name of the tileset
    pub name: String,
    /// Path to the tileset texture
    pub texture_path: String,
    /// Texture ID for rendering
    pub texture_id: String,
    /// Width of a single tile (pixels)
    pub tile_width: u32,
    /// Height of a single tile (pixels)
    pub tile_height: u32,
    /// Number of columns in the tileset
    pub columns: u32,
    /// Total number of tiles
    pub tile_count: u32,
    /// Spacing between tiles (pixels)
    pub spacing: u32,
    /// Margin around the tileset (pixels)
    pub margin: u32,
    /// Individual tile data (for tiles with custom properties)
    #[serde(default)]
    pub tiles: HashMap<u32, TileData>,
}

impl From<TileSetRaw> for TileSet {
    fn from(raw: TileSetRaw) -> Self {
        Self {
            name: raw.name,
            texture_path: normalize_texture_path(&raw.texture_path),
            texture_id: raw.texture_id,
            tile_width: raw.tile_width,
            tile_height: raw.tile_height,
            columns: raw.columns,
            tile_count: raw.tile_count,
            spacing: raw.spacing,
            margin: raw.margin,
            tiles: raw.tiles,
        }
    }
}

impl<'de> serde::Deserialize<'de> for TileSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = TileSetRaw::deserialize(deserializer)?;
        Ok(TileSet::from(raw))
    }
}

/// Normalize texture path - extract filename from absolute paths
fn normalize_texture_path(path: &str) -> String {
    use std::path::Path;

    // Check if it's an absolute path (Windows or Unix style)
    if path.contains(":\\") || path.starts_with('/') {
        // Extract just the filename
        if let Some(filename) = Path::new(path).file_name() {
            if let Some(name) = filename.to_str() {
                // Return as assets/filename
                log::info!("TileSet: Normalized absolute path '{}' to 'assets/{}'", path, name);
                return format!("assets/{}", name);
            }
        }
    }

    // Already relative, return as-is
    path.to_string()
}

impl TileSet {
    /// Create a new tileset
    pub fn new(
        name: impl Into<String>,
        texture_path: impl Into<String>,
        texture_id: impl Into<String>,
        tile_width: u32,
        tile_height: u32,
        columns: u32,
        tile_count: u32,
    ) -> Self {
        Self {
            name: name.into(),
            texture_path: texture_path.into(),
            texture_id: texture_id.into(),
            tile_width,
            tile_height,
            columns,
            tile_count,
            spacing: 0,
            margin: 0,
            tiles: HashMap::new(),
        }
    }

    /// Get tile coordinates in the tileset texture
    pub fn get_tile_coords(&self, tile_id: u32) -> Option<(u32, u32)> {
        if tile_id >= self.tile_count {
            return None;
        }

        let col = tile_id % self.columns;
        let row = tile_id / self.columns;

        let x = self.margin + col * (self.tile_width + self.spacing);
        let y = self.margin + row * (self.tile_height + self.spacing);

        Some((x, y))
    }

    /// Get tile data for a specific tile ID
    pub fn get_tile_data(&self, tile_id: u32) -> Option<&TileData> {
        self.tiles.get(&tile_id)
    }
}

/// Represents a single tile instance in a tilemap
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    /// Tile ID from the tileset (0 = empty)
    pub tile_id: u32,
    /// Horizontal flip
    pub flip_h: bool,
    /// Vertical flip
    pub flip_v: bool,
    /// Diagonal flip (for 90-degree rotation)
    pub flip_d: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_id: 0,
            flip_h: false,
            flip_v: false,
            flip_d: false,
        }
    }
}

impl Tile {
    /// Create a new tile
    pub fn new(tile_id: u32) -> Self {
        Self {
            tile_id,
            ..Default::default()
        }
    }

    /// Check if the tile is empty
    pub fn is_empty(&self) -> bool {
        self.tile_id == 0
    }
}

/// Tilemap component representing a layer of tiles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tilemap {
    /// Name of the tilemap layer
    pub name: String,
    /// Reference to the tileset entity or ID
    pub tileset_id: String,
    /// Width of the tilemap (in tiles)
    pub width: u32,
    /// Height of the tilemap (in tiles)
    pub height: u32,
    /// Animation frame rate
    #[serde(default = "default_animation_frame_rate")]
    pub animation_frame_rate: u32,
    /// Tint color (RGBA, 0.0-1.0)
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    /// Tile anchor point (0.0-1.0, where 0.5,0.5 is center)
    #[serde(default = "default_tile_anchor")]
    pub tile_anchor: [f32; 2],
    /// Orientation (XY, XZ, YZ)
    #[serde(default = "default_orientation")]
    pub orientation: String,
    /// Position offset
    #[serde(default = "default_offset")]
    pub offset: [f32; 3],
    /// Rotation (euler angles)
    #[serde(default = "default_rotation")]
    pub rotation: [f32; 3],
    /// Scale
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
    /// Tile data (row-major order: tiles[y * width + x])
    pub tiles: Vec<Tile>,
    /// Z-order for rendering (higher = rendered on top)
    #[serde(default)]
    pub z_order: i32,
    /// Is this layer visible?
    #[serde(default = "default_visible")]
    pub visible: bool,
    /// Layer opacity (0.0 - 1.0)
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// Parallax scroll factor (for background layers)
    #[serde(default = "default_parallax_factor")]
    pub parallax_factor: (f32, f32),
}

fn default_animation_frame_rate() -> u32 { 1 }
fn default_color() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }
fn default_tile_anchor() -> [f32; 2] { [0.5, 0.5] }
fn default_orientation() -> String { "XY".to_string() }
fn default_offset() -> [f32; 3] { [0.0, 0.0, 0.0] }
fn default_rotation() -> [f32; 3] { [0.0, 0.0, 0.0] }
fn default_scale() -> [f32; 3] { [1.0, 1.0, 1.0] }
fn default_visible() -> bool { true }
fn default_opacity() -> f32 { 1.0 }
fn default_parallax_factor() -> (f32, f32) { (1.0, 1.0) }

impl Tilemap {
    /// Create a new empty tilemap
    pub fn new(
        name: impl Into<String>,
        tileset_id: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Self {
        let tile_count = (width * height) as usize;
        Self {
            name: name.into(),
            tileset_id: tileset_id.into(),
            width,
            height,
            animation_frame_rate: 1,
            color: [1.0, 1.0, 1.0, 1.0],
            tile_anchor: [0.5, 0.5],
            orientation: "XY".to_string(),
            offset: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            tiles: vec![Tile::default(); tile_count],
            z_order: 0,
            visible: true,
            opacity: 1.0,
            parallax_factor: (1.0, 1.0),
        }
    }

    /// Get a tile at the specified position
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.tiles.get(index)
    }

    /// Get a mutable tile at the specified position
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.tiles.get_mut(index)
    }

    /// Set a tile at the specified position
    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = (y * self.width + x) as usize;
        if let Some(t) = self.tiles.get_mut(index) {
            *t = tile;
            true
        } else {
            false
        }
    }

    /// Set a tile ID at the specified position
    pub fn set_tile_id(&mut self, x: u32, y: u32, tile_id: u32) -> bool {
        self.set_tile(x, y, Tile::new(tile_id))
    }

    /// Clear all tiles
    pub fn clear(&mut self) {
        for tile in &mut self.tiles {
            *tile = Tile::default();
        }
    }

    /// Get the world position of a tile (in pixels, assuming tile size)
    pub fn tile_to_world(&self, x: u32, y: u32, tile_width: u32, tile_height: u32) -> (f32, f32) {
        (
            (x * tile_width) as f32,
            (y * tile_height) as f32,
        )
    }

    /// Get the tile position from world coordinates
    pub fn world_to_tile(&self, world_x: f32, world_y: f32, tile_width: u32, tile_height: u32) -> (u32, u32) {
        (
            (world_x / tile_width as f32) as u32,
            (world_y / tile_height as f32) as u32,
        )
    }
}

/// Tilemap chunk for efficient rendering of large tilemaps
#[derive(Clone, Debug)]
pub struct TilemapChunk {
    /// Chunk position (in chunk coordinates)
    pub chunk_x: u32,
    pub chunk_y: u32,
    /// Chunk size (in tiles)
    pub chunk_size: u32,
    /// Tiles in this chunk
    pub tiles: Vec<Tile>,
}

impl TilemapChunk {
    /// Create a new chunk
    pub fn new(chunk_x: u32, chunk_y: u32, chunk_size: u32) -> Self {
        let tile_count = (chunk_size * chunk_size) as usize;
        Self {
            chunk_x,
            chunk_y,
            chunk_size,
            tiles: vec![Tile::default(); tile_count],
        }
    }

    /// Check if the chunk has any non-empty tiles
    pub fn is_empty(&self) -> bool {
        self.tiles.iter().all(|t| t.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilemap_creation() {
        let tilemap = Tilemap::new("test_layer", "tileset_1", 10, 10);
        
        assert_eq!(tilemap.name, "test_layer");
        assert_eq!(tilemap.tileset_id, "tileset_1");
        assert_eq!(tilemap.width, 10);
        assert_eq!(tilemap.height, 10);
        assert_eq!(tilemap.tiles.len(), 100);
        assert!(tilemap.visible);
        assert_eq!(tilemap.opacity, 1.0);
    }

    #[test]
    fn test_tilemap_get_set_tile() {
        let mut tilemap = Tilemap::new("test_layer", "tileset_1", 5, 5);
        
        // Initially all tiles should be empty
        let tile = tilemap.get_tile(0, 0).unwrap();
        assert_eq!(tile.tile_id, 0);
        assert!(tile.is_empty());
        
        // Set a tile
        let new_tile = Tile::new(42);
        assert!(tilemap.set_tile(2, 3, new_tile.clone()));
        
        // Verify the tile was set
        let retrieved = tilemap.get_tile(2, 3).unwrap();
        assert_eq!(retrieved.tile_id, 42);
        assert!(!retrieved.is_empty());
        
        // Verify other tiles are still empty
        let other = tilemap.get_tile(0, 0).unwrap();
        assert!(other.is_empty());
    }

    #[test]
    fn test_tilemap_bounds_checking() {
        let mut tilemap = Tilemap::new("test_layer", "tileset_1", 5, 5);
        
        // Valid positions
        assert!(tilemap.get_tile(0, 0).is_some());
        assert!(tilemap.get_tile(4, 4).is_some());
        
        // Out of bounds positions
        assert!(tilemap.get_tile(5, 0).is_none());
        assert!(tilemap.get_tile(0, 5).is_none());
        assert!(tilemap.get_tile(10, 10).is_none());
        
        // Set tile out of bounds should return false
        assert!(!tilemap.set_tile(5, 0, Tile::new(1)));
        assert!(!tilemap.set_tile(0, 5, Tile::new(1)));
    }

    #[test]
    fn test_tilemap_set_tile_id() {
        let mut tilemap = Tilemap::new("test_layer", "tileset_1", 3, 3);
        
        // Set tile by ID
        assert!(tilemap.set_tile_id(1, 1, 99));
        
        // Verify it was set
        let tile = tilemap.get_tile(1, 1).unwrap();
        assert_eq!(tile.tile_id, 99);
    }

    #[test]
    fn test_tilemap_clear() {
        let mut tilemap = Tilemap::new("test_layer", "tileset_1", 3, 3);
        
        // Set some tiles
        tilemap.set_tile_id(0, 0, 1);
        tilemap.set_tile_id(1, 1, 2);
        tilemap.set_tile_id(2, 2, 3);
        
        // Clear all tiles
        tilemap.clear();
        
        // Verify all tiles are empty
        for y in 0..3 {
            for x in 0..3 {
                let tile = tilemap.get_tile(x, y).unwrap();
                assert!(tile.is_empty());
            }
        }
    }

    #[test]
    fn test_tile_flips() {
        let mut tile = Tile::new(10);
        assert!(!tile.flip_h);
        assert!(!tile.flip_v);
        assert!(!tile.flip_d);
        
        tile.flip_h = true;
        tile.flip_v = true;
        assert!(tile.flip_h);
        assert!(tile.flip_v);
    }

    #[test]
    fn test_tileset_tile_coords() {
        let tileset = TileSet::new(
            "test_tileset",
            "tileset.png",
            "tileset_1",
            16,  // tile_width
            16,  // tile_height
            8,   // columns
            64,  // tile_count
        );
        
        // First tile (0,0)
        assert_eq!(tileset.get_tile_coords(0), Some((0, 0)));
        
        // Second tile in first row
        assert_eq!(tileset.get_tile_coords(1), Some((16, 0)));
        
        // First tile in second row
        assert_eq!(tileset.get_tile_coords(8), Some((0, 16)));
        
        // Out of bounds
        assert_eq!(tileset.get_tile_coords(64), None);
        assert_eq!(tileset.get_tile_coords(100), None);
    }

    #[test]
    fn test_tileset_with_spacing_and_margin() {
        let mut tileset = TileSet::new(
            "test_tileset",
            "tileset.png",
            "tileset_1",
            16,  // tile_width
            16,  // tile_height
            4,   // columns
            16,  // tile_count
        );
        tileset.spacing = 2;
        tileset.margin = 1;
        
        // First tile should account for margin
        assert_eq!(tileset.get_tile_coords(0), Some((1, 1)));
        
        // Second tile should account for margin + spacing
        assert_eq!(tileset.get_tile_coords(1), Some((19, 1))); // 1 + 16 + 2
        
        // First tile in second row
        assert_eq!(tileset.get_tile_coords(4), Some((1, 19))); // 1 + 16 + 2
    }

    #[test]
    fn test_tilemap_renderer_default() {
        let renderer = TilemapRenderer::default();
        assert_eq!(renderer.mode, TilemapRenderMode::Chunk);
        assert_eq!(renderer.sorting_layer, "Default");
        assert_eq!(renderer.order_in_layer, 0);
        assert_eq!(renderer.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(renderer.chunk_size, 16);
        assert!(renderer.detect_chunk_culling);
    }

    #[test]
    fn test_tilemap_renderer_with_sorting() {
        let renderer = TilemapRenderer::with_sorting("Background", -10);
        assert_eq!(renderer.sorting_layer, "Background");
        assert_eq!(renderer.order_in_layer, -10);
    }

    #[test]
    fn test_tilemap_renderer_with_color() {
        let renderer = TilemapRenderer::new()
            .with_color(1.0, 0.5, 0.0, 0.8);
        assert_eq!(renderer.color, [1.0, 0.5, 0.0, 0.8]);
    }
}

/// Tilemap Renderer component (Unity-style)
/// 
/// Controls how the tilemap is rendered, similar to Unity's Tilemap Renderer.
/// This component should be attached to the same entity as the Tilemap component.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilemapRenderer {
    /// Rendering mode
    pub mode: TilemapRenderMode,
    
    /// Sorting layer (for 2D rendering order)
    pub sorting_layer: String,
    
    /// Order in layer (higher = rendered on top)
    pub order_in_layer: i32,
    
    /// Material/shader to use (optional)
    pub material: Option<String>,
    
    /// Tint color (RGBA, 0.0-1.0)
    pub color: [f32; 4],
    
    /// Chunk size for rendering optimization (0 = no chunking)
    pub chunk_size: u32,
    
    /// Detect chunk culling (don't render off-screen chunks)
    pub detect_chunk_culling: bool,
    
    /// Mask interaction (for sprite masking)
    pub mask_interaction: MaskInteraction,
}

impl Default for TilemapRenderer {
    fn default() -> Self {
        Self {
            mode: TilemapRenderMode::Chunk,
            sorting_layer: "Default".to_string(),
            order_in_layer: 0,
            material: None,
            color: [1.0, 1.0, 1.0, 1.0],  // White (no tint)
            chunk_size: 16,  // 16x16 tiles per chunk
            detect_chunk_culling: true,
            mask_interaction: MaskInteraction::None,
        }
    }
}

impl TilemapRenderer {
    /// Create a new tilemap renderer with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a tilemap renderer with custom sorting
    pub fn with_sorting(sorting_layer: impl Into<String>, order_in_layer: i32) -> Self {
        Self {
            sorting_layer: sorting_layer.into(),
            order_in_layer,
            ..Default::default()
        }
    }
    
    /// Set the tint color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
    
    /// Enable/disable chunk culling
    pub fn with_chunk_culling(mut self, enabled: bool) -> Self {
        self.detect_chunk_culling = enabled;
        self
    }
}

/// Tilemap rendering mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TilemapRenderMode {
    /// Render each tile individually (simple, but slower for large tilemaps)
    Individual,
    
    /// Render in chunks for better performance (Unity default)
    Chunk,
}

/// Mask interaction mode (Unity-style)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskInteraction {
    /// No mask interaction
    None,
    
    /// Visible inside mask
    VisibleInsideMask,
    
    /// Visible outside mask
    VisibleOutsideMask,
}
