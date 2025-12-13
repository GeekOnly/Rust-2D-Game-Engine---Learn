//! Game View Settings
//! 
//! Configuration for game view resolution and aspect ratio

use serde::{Serialize, Deserialize};

/// Game view resolution presets
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameViewResolution {
    // PC Resolutions (16:9)
    FullHD,      // 1920x1080
    HD,          // 1280x720
    WXGA,        // 1366x768
    QHD,         // 2560x1440
    UHD4K,       // 3840x2160
    
    // Mobile Resolutions (Portrait)
    IPhone14,    // 1170x2532 (19.5:9)
    IPhone14Pro, // 1179x2556 (19.5:9)
    IPhoneSE,    // 750x1334 (16:9)
    Pixel7,      // 1080x2400 (20:9)
    GalaxyS23,   // 1080x2340 (19.5:9)
    
    // Mobile Resolutions (Landscape)
    IPhone14Landscape,    // 2532x1170
    IPhone14ProLandscape, // 2556x1179
    Pixel7Landscape,      // 2400x1080
    
    // Tablet Resolutions
    IPadPro,     // 2048x2732 (4:3)
    IPadAir,     // 1640x2360 (3:4)
    
    // Custom
    Custom(u32, u32),
    
    // Free (use available space)
    Free,
}

impl GameViewResolution {
    pub fn get_size(&self) -> (u32, u32) {
        match self {
            // PC
            GameViewResolution::FullHD => (1920, 1080),
            GameViewResolution::HD => (1280, 720),
            GameViewResolution::WXGA => (1366, 768),
            GameViewResolution::QHD => (2560, 1440),
            GameViewResolution::UHD4K => (3840, 2160),
            
            // Mobile Portrait
            GameViewResolution::IPhone14 => (1170, 2532),
            GameViewResolution::IPhone14Pro => (1179, 2556),
            GameViewResolution::IPhoneSE => (750, 1334),
            GameViewResolution::Pixel7 => (1080, 2400),
            GameViewResolution::GalaxyS23 => (1080, 2340),
            
            // Mobile Landscape
            GameViewResolution::IPhone14Landscape => (2532, 1170),
            GameViewResolution::IPhone14ProLandscape => (2556, 1179),
            GameViewResolution::Pixel7Landscape => (2400, 1080),
            
            // Tablet
            GameViewResolution::IPadPro => (2048, 2732),
            GameViewResolution::IPadAir => (1640, 2360),
            
            // Custom/Free
            GameViewResolution::Custom(w, h) => (*w, *h),
            GameViewResolution::Free => (1920, 1080), // Default fallback
        }
    }
    
    pub fn get_aspect_ratio(&self) -> f32 {
        let (w, h) = self.get_size();
        w as f32 / h as f32
    }
    
    pub fn get_name(&self) -> &str {
        match self {
            GameViewResolution::FullHD => "Full HD (1920x1080)",
            GameViewResolution::HD => "HD (1280x720)",
            GameViewResolution::WXGA => "WXGA (1366x768)",
            GameViewResolution::QHD => "QHD (2560x1440)",
            GameViewResolution::UHD4K => "4K UHD (3840x2160)",
            GameViewResolution::IPhone14 => "iPhone 14 (1170x2532)",
            GameViewResolution::IPhone14Pro => "iPhone 14 Pro (1179x2556)",
            GameViewResolution::IPhoneSE => "iPhone SE (750x1334)",
            GameViewResolution::Pixel7 => "Pixel 7 (1080x2400)",
            GameViewResolution::GalaxyS23 => "Galaxy S23 (1080x2340)",
            GameViewResolution::IPhone14Landscape => "iPhone 14 Landscape",
            GameViewResolution::IPhone14ProLandscape => "iPhone 14 Pro Landscape",
            GameViewResolution::Pixel7Landscape => "Pixel 7 Landscape",
            GameViewResolution::IPadPro => "iPad Pro (2048x2732)",
            GameViewResolution::IPadAir => "iPad Air (1640x2360)",
            GameViewResolution::Custom(_, _) => "Custom",
            GameViewResolution::Free => "Free (Fit to Window)",
        }
    }
    
    pub fn get_category(&self) -> &str {
        match self {
            GameViewResolution::FullHD | GameViewResolution::HD | 
            GameViewResolution::WXGA | GameViewResolution::QHD | 
            GameViewResolution::UHD4K => "PC",
            
            GameViewResolution::IPhone14 | GameViewResolution::IPhone14Pro | 
            GameViewResolution::IPhoneSE | GameViewResolution::Pixel7 | 
            GameViewResolution::GalaxyS23 => "Mobile (Portrait)",
            
            GameViewResolution::IPhone14Landscape | GameViewResolution::IPhone14ProLandscape | 
            GameViewResolution::Pixel7Landscape => "Mobile (Landscape)",
            
            GameViewResolution::IPadPro | GameViewResolution::IPadAir => "Tablet",
            
            GameViewResolution::Custom(_, _) => "Custom",
            GameViewResolution::Free => "Free",
        }
    }
}

impl Default for GameViewResolution {
    fn default() -> Self {
        GameViewResolution::Free
    }
}

/// Game view settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameViewSettings {
    pub resolution: GameViewResolution,
    pub scale: f32,  // Scale factor (0.1 to 1.0)
    pub show_safe_area: bool,  // Show safe area guides
    pub show_resolution_info: bool,  // Show resolution info overlay
    pub background_color: [f32; 4],  // Background color outside game view
}

impl Default for GameViewSettings {
    fn default() -> Self {
        Self {
            resolution: GameViewResolution::Free,
            scale: 1.0,
            show_safe_area: false,
            show_resolution_info: true,
            background_color: [0.1, 0.1, 0.1, 1.0],
        }
    }
}

impl GameViewSettings {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get the actual render size based on resolution and scale
    pub fn get_render_size(&self) -> (u32, u32) {
        let (w, h) = self.resolution.get_size();
        ((w as f32 * self.scale) as u32, (h as f32 * self.scale) as u32)
    }
    
    /// Calculate the rect for game view within available space
    pub fn calculate_game_rect(&self, available_rect: egui::Rect) -> egui::Rect {
        if matches!(self.resolution, GameViewResolution::Free) {
            // Use full available space
            return available_rect;
        }
        
        let (target_w, target_h) = self.get_render_size();
        let target_aspect = target_w as f32 / target_h as f32;
        
        let available_w = available_rect.width();
        let available_h = available_rect.height();
        let available_aspect = available_w / available_h;
        
        let (final_w, final_h) = if available_aspect > target_aspect {
            // Available space is wider - fit to height
            let h = available_h.min(target_h as f32);
            let w = h * target_aspect;
            (w, h)
        } else {
            // Available space is taller - fit to width
            let w = available_w.min(target_w as f32);
            let h = w / target_aspect;
            (w, h)
        };
        
        // Center the game view
        let center = available_rect.center();
        egui::Rect::from_center_size(
            center,
            egui::vec2(final_w, final_h)
        )
    }
}
