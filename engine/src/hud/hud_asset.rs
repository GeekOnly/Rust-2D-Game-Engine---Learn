//! HUD Asset Definition
//! 
//! Defines the structure of HUD assets that can be loaded from JSON files

use serde::{Serialize, Deserialize};

/// HUD Asset - defines a complete HUD layout
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudAsset {
    pub name: String,
    pub elements: Vec<HudElement>,
}

impl HudAsset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            elements: Vec::new(),
        }
    }
    
    pub fn add_element(&mut self, element: HudElement) {
        self.elements.push(element);
    }
    
    /// Load HUD from JSON file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let hud: HudAsset = serde_json::from_str(&json)?;
        Ok(hud)
    }
    
    /// Save HUD to JSON file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Individual HUD element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudElement {
    pub id: String,
    pub element_type: HudElementType,
    pub anchor: Anchor,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

impl HudElement {
    pub fn new(
        id: impl Into<String>,
        element_type: HudElementType,
        anchor: Anchor,
        offset: [f32; 2],
        size: [f32; 2],
    ) -> Self {
        Self {
            id: id.into(),
            element_type,
            anchor,
            offset,
            size,
            visible: true,
        }
    }
    
    /// Calculate screen position based on anchor and offset
    pub fn get_screen_position(&self, screen_width: f32, screen_height: f32) -> [f32; 2] {
        let anchor_pos = self.anchor.get_position(screen_width, screen_height);
        [
            anchor_pos[0] + self.offset[0],
            anchor_pos[1] + self.offset[1],
        ]
    }
}

/// Types of HUD elements
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HudElementType {
    /// Health bar with data binding
    HealthBar {
        binding: String,
        #[serde(default = "default_bar_color")]
        color: [f32; 4],
        #[serde(default = "default_background_color")]
        background_color: [f32; 4],
    },
    /// Progress bar (generic)
    ProgressBar {
        binding: String,
        #[serde(default = "default_bar_color")]
        color: [f32; 4],
        #[serde(default = "default_background_color")]
        background_color: [f32; 4],
    },
    /// Minimap
    Minimap {
        #[serde(default = "default_zoom")]
        zoom: f32,
        #[serde(default = "default_background_color")]
        background_color: [f32; 4],
    },
    /// Text label
    Text {
        text: String,
        #[serde(default = "default_font_size")]
        font_size: f32,
        #[serde(default = "default_text_color")]
        color: [f32; 4],
    },
    /// Dynamic text with binding
    DynamicText {
        format: String, // e.g., "Score: {score}"
        #[serde(default = "default_font_size")]
        font_size: f32,
        #[serde(default = "default_text_color")]
        color: [f32; 4],
    },
    /// Image/Icon
    Image {
        texture: String,
        #[serde(default = "default_white_color")]
        tint: [f32; 4],
    },
    /// Container for grouping elements
    Container {
        children: Vec<HudElement>,
    },
}

fn default_bar_color() -> [f32; 4] {
    [1.0, 0.2, 0.2, 1.0] // Red
}

fn default_background_color() -> [f32; 4] {
    [0.2, 0.2, 0.2, 0.8] // Dark gray
}

fn default_zoom() -> f32 {
    2.0
}

fn default_font_size() -> f32 {
    18.0
}

fn default_text_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0] // White
}

fn default_white_color() -> [f32; 4] {
    [1.0, 1.0, 1.0, 1.0]
}

/// Anchor points for HUD elements
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    /// Get anchor position in screen coordinates
    pub fn get_position(&self, screen_width: f32, screen_height: f32) -> [f32; 2] {
        match self {
            Anchor::TopLeft => [0.0, 0.0],
            Anchor::TopCenter => [screen_width / 2.0, 0.0],
            Anchor::TopRight => [screen_width, 0.0],
            Anchor::CenterLeft => [0.0, screen_height / 2.0],
            Anchor::Center => [screen_width / 2.0, screen_height / 2.0],
            Anchor::CenterRight => [screen_width, screen_height / 2.0],
            Anchor::BottomLeft => [0.0, screen_height],
            Anchor::BottomCenter => [screen_width / 2.0, screen_height],
            Anchor::BottomRight => [screen_width, screen_height],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_positions() {
        let width = 1920.0;
        let height = 1080.0;
        
        assert_eq!(Anchor::TopLeft.get_position(width, height), [0.0, 0.0]);
        assert_eq!(Anchor::Center.get_position(width, height), [960.0, 540.0]);
        assert_eq!(Anchor::BottomRight.get_position(width, height), [1920.0, 1080.0]);
    }
    
    #[test]
    fn test_hud_element_screen_position() {
        let element = HudElement::new(
            "test",
            HudElementType::Text {
                text: "Hello".to_string(),
                font_size: 18.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Anchor::TopLeft,
            [20.0, 20.0],
            [100.0, 30.0],
        );
        
        let pos = element.get_screen_position(1920.0, 1080.0);
        assert_eq!(pos, [20.0, 20.0]);
    }
}
