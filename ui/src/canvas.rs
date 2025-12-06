//! Canvas component and management system

use serde::{Deserialize, Serialize};

/// The Canvas is the root component for all UI rendering, defining the coordinate space and render mode
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Canvas {
    /// Render mode determines how UI is positioned and rendered
    pub render_mode: CanvasRenderMode,
    
    /// Sort order for multiple canvases (higher renders on top)
    pub sort_order: i32,
    
    /// Reference camera for Screen Space Camera and World Space modes
    pub camera_entity: Option<u64>, // Using u64 as placeholder for Entity
    
    /// Plane distance for Screen Space Camera mode
    pub plane_distance: f32,
    
    /// Canvas scaler for resolution independence
    pub scaler: CanvasScaler,
    
    /// Whether this canvas blocks raycasts to canvases behind it
    pub blocks_raycasts: bool,
    
    /// Cached screen size for dirty checking
    #[serde(skip)]
    pub cached_screen_size: (u32, u32),
    
    /// Dirty flag for rebuild
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            render_mode: CanvasRenderMode::ScreenSpaceOverlay,
            sort_order: 0,
            camera_entity: None,
            plane_distance: 100.0,
            scaler: CanvasScaler::default(),
            blocks_raycasts: true,
            cached_screen_size: (0, 0),
            dirty: true,
        }
    }
}

/// Canvas render mode determines how UI is positioned and rendered
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasRenderMode {
    /// UI rendered in screen space on top of everything
    ScreenSpaceOverlay,
    
    /// UI rendered in screen space at a distance from camera
    ScreenSpaceCamera,
    
    /// UI rendered as part of the 3D world
    WorldSpace,
}

/// Canvas scaler handles UI scaling across different screen resolutions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasScaler {
    /// Scaling mode
    pub mode: ScaleMode,
    
    /// Reference resolution for Scale With Screen Size mode
    pub reference_resolution: (f32, f32),
    
    /// Match width (0.0) or height (1.0) or blend
    pub match_width_or_height: f32,
    
    /// Reference DPI for Constant Physical Size mode
    pub reference_dpi: f32,
    
    /// Minimum and maximum scale factors
    pub min_scale: f32,
    pub max_scale: f32,
    
    /// Cached scale factor
    #[serde(skip)]
    pub scale_factor: f32,
}

impl Default for CanvasScaler {
    fn default() -> Self {
        Self {
            mode: ScaleMode::ConstantPixelSize,
            reference_resolution: (1920.0, 1080.0),
            match_width_or_height: 0.0,
            reference_dpi: 96.0,
            min_scale: 0.1,
            max_scale: 10.0,
            scale_factor: 1.0,
        }
    }
}

/// Scaling mode for canvas scaler
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScaleMode {
    /// UI elements maintain their specified pixel dimensions
    ConstantPixelSize,
    
    /// UI elements scale proportionally to the reference resolution
    ScaleWithScreenSize,
    
    /// UI elements maintain consistent physical dimensions based on DPI
    ConstantPhysicalSize,
}
