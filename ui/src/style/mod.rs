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

impl UITheme {
    /// Get the active style
    pub fn get_active_style(&self) -> Option<&UIStyle> {
        self.styles.get(&self.active_style)
    }
    
    /// Get a style by name
    pub fn get_style(&self, name: &str) -> Option<&UIStyle> {
        self.styles.get(name)
    }
    
    /// Set the active style
    pub fn set_active_style(&mut self, name: String) {
        if self.styles.contains_key(&name) {
            self.active_style = name;
        }
    }
    
    /// Add or update a style
    pub fn add_style(&mut self, style: UIStyle) {
        self.styles.insert(style.name.clone(), style);
    }
}

/// Component that marks a UI element as styled
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyledElement {
    /// Style name to apply (None means inherit from parent)
    pub style_name: Option<String>,
    
    /// Whether to inherit style from parent
    pub inherit_from_parent: bool,
    
    /// Cached resolved style name (for performance)
    #[serde(skip)]
    pub resolved_style_name: Option<String>,
    
    /// Whether this element needs style update
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for StyledElement {
    fn default() -> Self {
        Self {
            style_name: None,
            inherit_from_parent: true,
            resolved_style_name: None,
            dirty: true,
        }
    }
}

impl StyledElement {
    /// Create a styled element with a specific style
    pub fn with_style(style_name: String) -> Self {
        Self {
            style_name: Some(style_name.clone()),
            inherit_from_parent: false,
            resolved_style_name: Some(style_name),
            dirty: true,
        }
    }
    
    /// Create a styled element that inherits from parent
    pub fn inheriting() -> Self {
        Self {
            style_name: None,
            inherit_from_parent: true,
            resolved_style_name: None,
            dirty: true,
        }
    }
    
    /// Mark this element as needing style update
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

/// Component for animating style transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleTransition {
    /// Duration of the transition in seconds
    pub duration: f32,
    
    /// Elapsed time
    #[serde(skip)]
    pub elapsed: f32,
    
    /// Whether transition is active
    #[serde(skip)]
    pub active: bool,
    
    /// Source colors (before transition)
    #[serde(skip)]
    pub from_primary: Color,
    #[serde(skip)]
    pub from_secondary: Color,
    #[serde(skip)]
    pub from_background: Color,
    #[serde(skip)]
    pub from_text: Color,
    
    /// Target colors (after transition)
    #[serde(skip)]
    pub to_primary: Color,
    #[serde(skip)]
    pub to_secondary: Color,
    #[serde(skip)]
    pub to_background: Color,
    #[serde(skip)]
    pub to_text: Color,
}

impl Default for StyleTransition {
    fn default() -> Self {
        Self {
            duration: 0.3,
            elapsed: 0.0,
            active: false,
            from_primary: [0.0, 0.0, 0.0, 1.0],
            from_secondary: [0.0, 0.0, 0.0, 1.0],
            from_background: [0.0, 0.0, 0.0, 1.0],
            from_text: [0.0, 0.0, 0.0, 1.0],
            to_primary: [0.0, 0.0, 0.0, 1.0],
            to_secondary: [0.0, 0.0, 0.0, 1.0],
            to_background: [0.0, 0.0, 0.0, 1.0],
            to_text: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl StyleTransition {
    /// Start a transition from current colors to new colors
    pub fn start(&mut self, from_style: &UIStyle, to_style: &UIStyle) {
        self.from_primary = from_style.primary_color;
        self.from_secondary = from_style.secondary_color;
        self.from_background = from_style.background_color;
        self.from_text = from_style.text_color;
        
        self.to_primary = to_style.primary_color;
        self.to_secondary = to_style.secondary_color;
        self.to_background = to_style.background_color;
        self.to_text = to_style.text_color;
        
        self.elapsed = 0.0;
        self.active = true;
    }
    
    /// Update the transition
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.active {
            return false;
        }
        
        self.elapsed += delta_time;
        
        if self.elapsed >= self.duration {
            self.active = false;
            return false;
        }
        
        true
    }
    
    /// Get the interpolation factor (0.0 to 1.0)
    pub fn get_t(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).min(1.0)
    }
    
    /// Interpolate a color
    pub fn lerp_color(from: Color, to: Color, t: f32) -> Color {
        [
            from[0] + (to[0] - from[0]) * t,
            from[1] + (to[1] - from[1]) * t,
            from[2] + (to[2] - from[2]) * t,
            from[3] + (to[3] - from[3]) * t,
        ]
    }
    
    /// Get the current interpolated colors
    pub fn get_current_colors(&self) -> (Color, Color, Color, Color) {
        let t = self.get_t();
        (
            Self::lerp_color(self.from_primary, self.to_primary, t),
            Self::lerp_color(self.from_secondary, self.to_secondary, t),
            Self::lerp_color(self.from_background, self.to_background, t),
            Self::lerp_color(self.from_text, self.to_text, t),
        )
    }
}
