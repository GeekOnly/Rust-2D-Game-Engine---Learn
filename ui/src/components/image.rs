//! UIImage component

use serde::{Deserialize, Serialize};
use glam::Vec4;

/// Image component for displaying sprites and textures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIImage {
    /// Sprite or texture to display
    pub sprite: Option<String>, // Texture ID
    
    /// Image type (simple, sliced, tiled, filled)
    pub image_type: ImageType,
    
    /// 9-slice borders (for sliced type) - left, bottom, right, top
    pub slice_borders: Vec4,
    
    /// Fill method (for filled type)
    pub fill_method: FillMethod,
    
    /// Fill amount (0.0 to 1.0 for filled type)
    pub fill_amount: f32,
    
    /// Fill origin (for filled type)
    pub fill_origin: i32,
    
    /// Whether to preserve aspect ratio
    pub preserve_aspect: bool,
}

impl Default for UIImage {
    fn default() -> Self {
        Self {
            sprite: None,
            image_type: ImageType::Simple,
            slice_borders: Vec4::ZERO,
            fill_method: FillMethod::Horizontal,
            fill_amount: 1.0,
            fill_origin: 0,
            preserve_aspect: false,
        }
    }
}

/// Image rendering type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImageType {
    /// Simple stretched rendering
    Simple,
    /// 9-slice rendering (preserves corners and edges)
    Sliced,
    /// Tiled rendering
    Tiled,
    /// Filled rendering (progress bars, radial fills)
    Filled,
}

/// Fill method for filled images
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FillMethod {
    /// Fill horizontally (left to right)
    Horizontal,
    /// Fill vertically (bottom to top)
    Vertical,
    /// Fill radially 90 degrees
    Radial90,
    /// Fill radially 180 degrees
    Radial180,
    /// Fill radially 360 degrees
    Radial360,
}
