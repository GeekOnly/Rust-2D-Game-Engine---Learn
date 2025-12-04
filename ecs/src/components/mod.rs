//! Component definitions for the ECS
//!
//! All components are defined here and can be used with any backend.

pub mod sprite_sheet;
pub mod tilemap;
pub mod map;

// Re-export all components
pub use sprite_sheet::{SpriteSheet, SpriteFrame, AnimatedSprite, AnimationMode};
pub use tilemap::{TileSet, Tilemap, Tile, TileData, TilemapChunk};
pub use map::{Map, MapType};
