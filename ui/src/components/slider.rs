//! UISlider component

use serde::{Deserialize, Serialize};

/// Slider component for value selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UISlider {
    /// Fill rect entity
    pub fill_rect: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Handle rect entity
    pub handle_rect: Option<u64>,
    
    /// Direction
    pub direction: SliderDirection,
    
    /// Min value
    pub min_value: f32,
    
    /// Max value
    pub max_value: f32,
    
    /// Current value
    pub value: f32,
    
    /// Whether to use whole numbers
    pub whole_numbers: bool,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

impl Default for UISlider {
    fn default() -> Self {
        Self {
            fill_rect: None,
            handle_rect: None,
            direction: SliderDirection::LeftToRight,
            min_value: 0.0,
            max_value: 1.0,
            value: 0.0,
            whole_numbers: false,
            on_value_changed: None,
        }
    }
}

/// Slider direction
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SliderDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}
