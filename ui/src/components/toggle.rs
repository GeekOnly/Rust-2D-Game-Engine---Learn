//! UIToggle component

use serde::{Deserialize, Serialize};

/// Toggle component for checkboxes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIToggle {
    /// Checkmark graphic entity
    pub graphic: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Whether toggle is on
    pub is_on: bool,
    
    /// Toggle transition
    pub toggle_transition: ToggleTransition,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

impl Default for UIToggle {
    fn default() -> Self {
        Self {
            graphic: None,
            is_on: false,
            toggle_transition: ToggleTransition::Fade,
            on_value_changed: None,
        }
    }
}

/// Toggle transition type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToggleTransition {
    None,
    Fade,
}
