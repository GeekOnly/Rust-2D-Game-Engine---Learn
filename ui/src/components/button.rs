//! UIButton component

use serde::{Deserialize, Serialize};
use crate::Color;

/// Button component with state management and transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIButton {
    /// Button state
    #[serde(skip)]
    pub state: ButtonState,
    
    /// Transition type
    pub transition: ButtonTransition,
    
    /// Color tint for each state (for ColorTint transition)
    pub normal_color: Color,
    pub highlighted_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    
    /// Color fade duration
    pub fade_duration: f32,
    
    /// Sprite swap (for SpriteSwap transition)
    pub highlighted_sprite: Option<String>,
    pub pressed_sprite: Option<String>,
    pub disabled_sprite: Option<String>,
    
    /// Animation trigger (for Animation transition)
    pub normal_trigger: String,
    pub highlighted_trigger: String,
    pub pressed_trigger: String,
    pub disabled_trigger: String,
    
    /// Lua callback function name
    pub on_click: Option<String>,
}

impl Default for UIButton {
    fn default() -> Self {
        Self {
            state: ButtonState::Normal,
            transition: ButtonTransition::ColorTint,
            normal_color: [1.0, 1.0, 1.0, 1.0],
            highlighted_color: [0.9, 0.9, 0.9, 1.0],
            pressed_color: [0.7, 0.7, 0.7, 1.0],
            disabled_color: [0.5, 0.5, 0.5, 0.5],
            fade_duration: 0.1,
            highlighted_sprite: None,
            pressed_sprite: None,
            disabled_sprite: None,
            normal_trigger: String::new(),
            highlighted_trigger: String::new(),
            pressed_trigger: String::new(),
            disabled_trigger: String::new(),
            on_click: None,
        }
    }
}

/// Button state
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum ButtonState {
    #[default]
    Normal,
    Highlighted,
    Pressed,
    Disabled,
}

/// Button transition type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ButtonTransition {
    None,
    ColorTint,
    SpriteSwap,
    Animation,
}
