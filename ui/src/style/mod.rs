//! UI styling and theme system

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use glam::Vec4;
use crate::Color;

/// UI Style definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIStyle {
    /// Style name
    pub name: String,
    
    /// Colors
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub text_color: Color,
    pub disabled_color: Color,
    
    /// Fonts
    pub default_font: String,
    pub default_font_size: f32,
    
    /// Sprites
    pub button_sprite: Option<String>,
    pub panel_sprite: Option<String>,
    pub input_field_sprite: Option<String>,
    pub slider_background_sprite: Option<String>,
    pub slider_fill_sprite: Option<String>,
    pub slider_handle_sprite: Option<String>,
    pub toggle_background_sprite: Option<String>,
    pub toggle_checkmark_sprite: Option<String>,
    pub dropdown_sprite: Option<String>,
    pub scrollbar_sprite: Option<String>,
    
    /// Spacing
    pub default_spacing: f32,
    pub default_padding: Vec4,
}

impl Default for UIStyle {
    fn default() -> Self {
        Self {
            name: String::from("default"),
            primary_color: [0.2, 0.4, 0.8, 1.0],
            secondary_color: [0.6, 0.6, 0.6, 1.0],
            background_color: [0.9, 0.9, 0.9, 1.0],
            text_color: [0.0, 0.0, 0.0, 1.0],
            disabled_color: [0.5, 0.5, 0.5, 0.5],
            default_font: String::from("default"),
            default_font_size: 14.0,
            button_sprite: None,
            panel_sprite: None,
            input_field_sprite: None,
            slider_background_sprite: None,
            slider_fill_sprite: None,
            slider_handle_sprite: None,
            toggle_background_sprite: None,
            toggle_checkmark_sprite: None,
            dropdown_sprite: None,
            scrollbar_sprite: None,
            default_spacing: 5.0,
            default_padding: Vec4::new(10.0, 10.0, 10.0, 10.0),
        }
    }
}

/// UI Theme (collection of styles)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UITheme {
    pub name: String,
    pub styles: HashMap<String, UIStyle>,
    pub active_style: String,
}

impl Default for UITheme {
    fn default() -> Self {
        let mut styles = HashMap::new();
        let default_style = UIStyle::default();
        styles.insert(default_style.name.clone(), default_style);
        
        Self {
            name: String::from("default"),
            styles,
            active_style: String::from("default"),
        }
    }
}
