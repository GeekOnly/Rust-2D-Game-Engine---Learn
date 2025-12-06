//! # Sprite Editor
//!
//! A visual sprite sheet editor for defining and managing sprite rectangles
//! within texture atlases.
//!
//! ## Features
//!
//! - Visual sprite rectangle editing
//! - Auto-slicing for grid-based sprite sheets
//! - Multiple export formats (JSON, XML, TexturePacker)
//! - Sprite validation and statistics
//! - Hot-reloading of sprite metadata

// Module declarations
pub mod metadata;
pub mod statistics;
pub mod auto_slicer;
pub mod utils;

// Re-export main types
pub use metadata::{ExportFormat, SpriteDefinition, SpriteMetadata};
pub use statistics::SpriteStatistics;
pub use auto_slicer::AutoSlicer;
