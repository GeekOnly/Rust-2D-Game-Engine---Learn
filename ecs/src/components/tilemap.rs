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

/// Tileset component containing tile data and texture information
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// Tile data (row-major order: tiles[y * width + x])
    pub tiles: Vec<Tile>,
    /// Z-order for rendering (higher = rendered on top)
    pub z_order: i32,
    /// Is this layer visible?
    pub visible: bool,
    /// Layer opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Parallax scroll factor (for background layers)
    pub parallax_factor: (f32, f32),
}

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
