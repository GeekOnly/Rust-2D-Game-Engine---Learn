//! UI rendering system

use serde::{Deserialize, Serialize};

pub mod clipping;
pub mod mask_system;
pub mod nine_slice;
pub mod batch_builder;

#[cfg(feature = "rendering")]
pub mod ui_renderer;

pub use clipping::{ClipRegion, ViewportClippingSystem};
pub use mask_system::{MaskingSystem, MaskState};
pub use nine_slice::{UIVertex, UIMesh, generate_nine_slice_mesh, generate_simple_mesh};
pub use batch_builder::{
    UIBatch, BatchableElement, UIBatchBuilder, BatchStats, UIRenderSystem,
};

#[cfg(feature = "rendering")]
pub use ui_renderer::UIRenderPass;

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
