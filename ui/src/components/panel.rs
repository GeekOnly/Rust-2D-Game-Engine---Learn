//! UIPanel component

use serde::{Deserialize, Serialize};
use glam::Vec4;

/// Panel component for background containers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPanel {
    /// Background sprite
    pub background: Option<String>,
    
    /// Whether to use 9-slice
    pub use_nine_slice: bool,
    
    /// 9-slice borders - left, bottom, right, top
    pub slice_borders: Vec4,
    
    /// Padding inside the panel - left, bottom, right, top
    pub padding: Vec4,
}

impl Default for UIPanel {
    fn default() -> Self {
        Self {
            background: None,
            use_nine_slice: false,
            slice_borders: Vec4::ZERO,
            padding: Vec4::ZERO,
        }
    }
}
