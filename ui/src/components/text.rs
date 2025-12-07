//! UIText component

use serde::{Deserialize, Serialize};
use crate::Color;

/// Text component for displaying text
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIText {
    /// Text content
    pub text: String,
    
    /// Font asset ID
    pub font: String,
    
    /// Font size in points
    pub font_size: f32,
    
    /// Text color (overrides UIElement color)
    pub color: Color,
    
    /// Text alignment
    pub alignment: TextAlignment,
    
    /// Horizontal overflow mode
    pub horizontal_overflow: OverflowMode,
    
    /// Vertical overflow mode
    pub vertical_overflow: OverflowMode,
    
    /// Whether to enable rich text markup
    pub rich_text: bool,
    
    /// Line spacing multiplier
    pub line_spacing: f32,
    
    /// Whether to use best fit
    pub best_fit: bool,
    
    /// Min and max font size for best fit
    pub best_fit_min_size: f32,
    pub best_fit_max_size: f32,
}

impl Default for UIText {
    fn default() -> Self {
        Self {
            text: String::new(),
            font: String::from("default"),
            font_size: 14.0,
            color: [0.0, 0.0, 0.0, 1.0], // Black
            alignment: TextAlignment::MiddleCenter,
            horizontal_overflow: OverflowMode::Wrap,
            vertical_overflow: OverflowMode::Truncate,
            rich_text: false,
            line_spacing: 1.0,
            best_fit: false,
            best_fit_min_size: 10.0,
            best_fit_max_size: 40.0,
        }
    }
}

/// Text alignment options
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, MiddleCenter, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}

/// Text overflow handling mode
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum OverflowMode {
    /// Wrap text to next line
    Wrap,
    /// Allow text to overflow bounds
    Overflow,
    /// Truncate text at bounds
    Truncate,
}
