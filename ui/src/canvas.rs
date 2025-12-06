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

impl Canvas {
    /// Create a new Canvas with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Canvas with specified render mode
    pub fn with_render_mode(render_mode: CanvasRenderMode) -> Self {
        Self {
            render_mode,
            ..Default::default()
        }
    }

    /// Create a new Canvas with specified sort order
    pub fn with_sort_order(sort_order: i32) -> Self {
        Self {
            sort_order,
            ..Default::default()
        }
    }

    /// Mark the canvas as dirty, requiring a rebuild
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the canvas is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Update cached screen size and mark dirty if changed
    pub fn update_screen_size(&mut self, width: u32, height: u32) -> bool {
        if self.cached_screen_size != (width, height) {
            self.cached_screen_size = (width, height);
            self.dirty = true;
            true
        } else {
            false
        }
    }
}

impl CanvasScaler {
    /// Create a new CanvasScaler with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a CanvasScaler with ConstantPixelSize mode
    pub fn constant_pixel_size() -> Self {
        Self {
            mode: ScaleMode::ConstantPixelSize,
            ..Default::default()
        }
    }

    /// Create a CanvasScaler with ScaleWithScreenSize mode
    pub fn scale_with_screen_size(reference_width: f32, reference_height: f32) -> Self {
        Self {
            mode: ScaleMode::ScaleWithScreenSize,
            reference_resolution: (reference_width, reference_height),
            ..Default::default()
        }
    }

    /// Create a CanvasScaler with ConstantPhysicalSize mode
    pub fn constant_physical_size(reference_dpi: f32) -> Self {
        Self {
            mode: ScaleMode::ConstantPhysicalSize,
            reference_dpi,
            ..Default::default()
        }
    }

    /// Calculate the scale factor based on current screen size and DPI
    /// 
    /// # Arguments
    /// * `screen_width` - Current screen width in pixels
    /// * `screen_height` - Current screen height in pixels
    /// * `screen_dpi` - Current screen DPI (dots per inch)
    /// 
    /// # Returns
    /// The calculated scale factor, clamped between min_scale and max_scale
    pub fn calculate_scale_factor(
        &mut self,
        screen_width: f32,
        screen_height: f32,
        screen_dpi: f32,
    ) -> f32 {
        let scale = match self.mode {
            ScaleMode::ConstantPixelSize => {
                // Maintain constant pixel size - no scaling
                1.0
            }
            ScaleMode::ScaleWithScreenSize => {
                // Scale proportionally to reference resolution
                let (ref_width, ref_height) = self.reference_resolution;
                
                // Calculate scale factors for width and height
                let width_scale = screen_width / ref_width;
                let height_scale = screen_height / ref_height;
                
                // Blend between width and height scale based on match_width_or_height
                // 0.0 = match width, 1.0 = match height, 0.5 = average
                let match_factor = self.match_width_or_height.clamp(0.0, 1.0);
                
                // Use logarithmic blending for better results
                let log_width = width_scale.ln();
                let log_height = height_scale.ln();
                let log_scale = log_width * (1.0 - match_factor) + log_height * match_factor;
                
                log_scale.exp()
            }
            ScaleMode::ConstantPhysicalSize => {
                // Scale based on DPI to maintain constant physical size
                screen_dpi / self.reference_dpi
            }
        };

        // Clamp the scale factor to min/max bounds
        let clamped_scale = scale.clamp(self.min_scale, self.max_scale);
        
        // Cache the calculated scale factor
        self.scale_factor = clamped_scale;
        
        clamped_scale
    }

    /// Get the current cached scale factor
    pub fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Set the reference resolution for ScaleWithScreenSize mode
    pub fn set_reference_resolution(&mut self, width: f32, height: f32) {
        self.reference_resolution = (width, height);
    }

    /// Set the match width or height factor (0.0 = width, 1.0 = height)
    pub fn set_match_width_or_height(&mut self, value: f32) {
        self.match_width_or_height = value.clamp(0.0, 1.0);
    }

    /// Set the scale mode
    pub fn set_mode(&mut self, mode: ScaleMode) {
        self.mode = mode;
    }

    /// Set the minimum scale factor
    pub fn set_min_scale(&mut self, min: f32) {
        self.min_scale = min.max(0.0);
    }

    /// Set the maximum scale factor
    pub fn set_max_scale(&mut self, max: f32) {
        self.max_scale = max.max(0.0);
    }

    /// Set the reference DPI for ConstantPhysicalSize mode
    pub fn set_reference_dpi(&mut self, dpi: f32) {
        self.reference_dpi = dpi.max(1.0);
    }
}
