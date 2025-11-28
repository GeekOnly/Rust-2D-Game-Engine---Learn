//! Component definitions for the ECS
//!
//! All components are defined here and can be used with any backend.

mod transform;
mod rendering;
mod physics;
mod camera;
mod scripting;
mod gameplay;
pub mod sprite_sheet;
pub mod tilemap;

// Re-export all components
pub use transform::Transform;
pub use rendering::{Sprite, Mesh, MeshType};
pub use physics::Collider;
pub use camera::{Camera, CameraProjection, CameraClearFlags};
pub use scripting::{Script, ScriptParameter};
pub use gameplay::EntityTag;
pub use sprite_sheet::{SpriteSheet, SpriteFrame, AnimatedSprite, AnimationMode};
pub use tilemap::{TileSet, Tilemap, Tile, TileData, TilemapChunk};
