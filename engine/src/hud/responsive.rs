//! Responsive UI System
//! 
//! Advanced positioning with percentage-based anchors and constraints

use serde::{Serialize, Deserialize};

/// Advanced anchor with min/max constraints
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponsiveAnchor {
    /// Base anchor point
    pub anchor: super::hud_asset::Anchor,
    
    /// Offset in pixels or percentage
    pub offset: OffsetMode,
    
    /// Minimum distance from edges (safe area)
    pub min_margin: Option<[f32; 4]>, // [left, top, right, bottom]
    
    /// Maximum size constraints
    pub max_size: Option<[f32; 2]>,
    
    /// Scale with screen size
    pub scale_with_screen: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OffsetMode {
    /// Fixed pixel offset
    Pixels([f32; 2]),
    
    /// Percentage of screen size (0.0 to 1.0)
    Percentage([f32; 2]),
    
    /// Mixed (x in pixels, y in percentage)
    Mixed { x_pixels: f32, y_percent: f32 },
}

impl ResponsiveAnchor {
    /// Calculate final position based on screen size
    pub fn calculate_position(&self, screen_width: f32, screen_height: f32) -> [f32; 2] {
        // Get base anchor position
        let anchor_pos = self.anchor.get_position(screen_width, screen_height);
        
        // Calculate offset
        let offset = match &self.offset {
            OffsetMode::Pixels(px) => *px,
            OffsetMode::Percentage(pct) => [
                pct[0] * screen_width,
                pct[1] * screen_height,
            ],
            OffsetMode::Mixed { x_pixels, y_percent } => [
                *x_pixels,
                y_percent * screen_height,
            ],
        };
        
        // Apply offset
        let mut final_pos = [
            anchor_pos[0] + offset[0],
            anchor_pos[1] + offset[1],
        ];
        
        // Apply min margin constraints
        if let Some(margin) = &self.min_margin {
            final_pos[0] = final_pos[0].max(margin[0]); // left
            final_pos[1] = final_pos[1].max(margin[1]); // top
            final_pos[0] = final_pos[0].min(screen_width - margin[2]); // right
            final_pos[1] = final_pos[1].min(screen_height - margin[3]); // bottom
        }
        
        final_pos
    }
    
    /// Calculate responsive size
    pub fn calculate_size(&self, base_size: [f32; 2], screen_width: f32, screen_height: f32) -> [f32; 2] {
        let mut size = base_size;
        
        // Scale with screen if enabled
        if self.scale_with_screen {
            let scale_factor = (screen_width / 1920.0).min(screen_height / 1080.0);
            size[0] *= scale_factor;
            size[1] *= scale_factor;
        }
        
        // Apply max size constraint
        if let Some(max_size) = &self.max_size {
            size[0] = size[0].min(max_size[0]);
            size[1] = size[1].min(max_size[1]);
        }
        
        size
    }
}

/// Responsive layout presets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResponsivePreset {
    /// Mobile-first (small screens)
    Mobile,
    
    /// Tablet (medium screens)
    Tablet,
    
    /// Desktop (large screens)
    Desktop,
    
    /// Custom breakpoints
    Custom {
        mobile_max: f32,
        tablet_max: f32,
    },
}

impl ResponsivePreset {
    /// Get current preset based on screen width
    pub fn from_screen_width(width: f32) -> Self {
        if width < 768.0 {
            ResponsivePreset::Mobile
        } else if width < 1366.0 {
            ResponsivePreset::Tablet
        } else {
            ResponsivePreset::Desktop
        }
    }
    
    /// Get scale factor for this preset
    pub fn get_scale_factor(&self) -> f32 {
        match self {
            ResponsivePreset::Mobile => 0.8,
            ResponsivePreset::Tablet => 0.9,
            ResponsivePreset::Desktop => 1.0,
            ResponsivePreset::Custom { .. } => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentage_offset() {
        let anchor = ResponsiveAnchor {
            anchor: super::super::hud_asset::Anchor::TopLeft,
            offset: OffsetMode::Percentage([0.1, 0.05]), // 10% right, 5% down
            min_margin: None,
            max_size: None,
            scale_with_screen: false,
        };
        
        let pos = anchor.calculate_position(1920.0, 1080.0);
        assert_eq!(pos, [192.0, 54.0]); // 10% of 1920, 5% of 1080
    }
    
    #[test]
    fn test_min_margin() {
        let anchor = ResponsiveAnchor {
            anchor: super::super::hud_asset::Anchor::TopLeft,
            offset: OffsetMode::Pixels([5.0, 5.0]),
            min_margin: Some([20.0, 20.0, 20.0, 20.0]), // 20px safe area
            max_size: None,
            scale_with_screen: false,
        };
        
        let pos = anchor.calculate_position(1920.0, 1080.0);
        assert_eq!(pos, [20.0, 20.0]); // Clamped to min margin
    }
}
