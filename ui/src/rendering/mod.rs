//! UI rendering system

use serde::{Deserialize, Serialize};

/// UI Mask component for clipping
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIMask {
    /// Whether to show the mask graphic
    pub show_mask_graphic: bool,
    
    /// Whether to use sprite alpha for masking
    pub use_sprite_alpha: bool,
}

impl Default for UIMask {
    fn default() -> Self {
        Self {
            show_mask_graphic: true,
            use_sprite_alpha: false,
        }
    }
}
