//! UIScrollView component

use serde::{Deserialize, Serialize};
use glam::Vec2;

/// Scroll view component for scrollable content
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIScrollView {
    /// Content entity (the scrollable content container)
    pub content: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Viewport entity (the visible area)
    pub viewport: Option<u64>,
    
    /// Horizontal scrollbar
    pub horizontal_scrollbar: Option<u64>,
    
    /// Vertical scrollbar
    pub vertical_scrollbar: Option<u64>,
    
    /// Movement type
    pub movement_type: MovementType,
    
    /// Elasticity (for elastic movement)
    pub elasticity: f32,
    
    /// Inertia
    pub inertia: bool,
    
    /// Deceleration rate
    pub deceleration_rate: f32,
    
    /// Scroll sensitivity
    pub scroll_sensitivity: f32,
    
    /// Horizontal scroll enabled
    pub horizontal: bool,
    
    /// Vertical scroll enabled
    pub vertical: bool,
    
    /// Current scroll position (0-1)
    #[serde(skip)]
    pub normalized_position: Vec2,
    
    /// Velocity for inertia
    #[serde(skip)]
    pub velocity: Vec2,
}

impl Default for UIScrollView {
    fn default() -> Self {
        Self {
            content: None,
            viewport: None,
            horizontal_scrollbar: None,
            vertical_scrollbar: None,
            movement_type: MovementType::Elastic,
            elasticity: 0.1,
            inertia: true,
            deceleration_rate: 0.135,
            scroll_sensitivity: 1.0,
            horizontal: true,
            vertical: true,
            normalized_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }
}

/// Movement type for scroll view
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MovementType {
    Unrestricted,
    Elastic,
    Clamped,
}
