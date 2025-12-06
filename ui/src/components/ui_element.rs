//! Base UI element component

use serde::{Deserialize, Serialize};
use crate::Color;

/// Base component for all UI elements, providing common properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIElement {
    /// Whether this element can receive raycast events
    pub raycast_target: bool,
    
    /// Whether this element blocks raycasts to elements behind it
    pub blocks_raycasts: bool,
    
    /// Z-order within siblings (higher renders on top)
    pub z_order: i32,
    
    /// Color tint applied to this element
    pub color: Color,
    
    /// Alpha transparency (0.0 = fully transparent, 1.0 = fully opaque)
    pub alpha: f32,
    
    /// Whether this element is interactable
    pub interactable: bool,
    
    /// Whether to ignore parent groups (for layout)
    pub ignore_layout: bool,
    
    /// Cached canvas entity (updated by hierarchy system)
    #[serde(skip)]
    pub canvas_entity: Option<u64>, // Using u64 as placeholder for Entity
}

impl Default for UIElement {
    fn default() -> Self {
        Self {
            raycast_target: true,
            blocks_raycasts: true,
            z_order: 0,
            color: [1.0, 1.0, 1.0, 1.0], // White
            alpha: 1.0,
            interactable: true,
            ignore_layout: false,
            canvas_entity: None,
        }
    }
}
