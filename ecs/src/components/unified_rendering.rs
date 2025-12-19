//! Unified 2D/3D Rendering Components
//!
//! This module contains components and types for the unified 2D/3D rendering system
//! that allows seamless integration of 2D sprites and tilemaps with 3D content.

use serde::{Deserialize, Serialize};

/// Pixel-perfect transform utilities for unified 2D/3D rendering
pub mod pixel_perfect_utils {
    use super::*;
    
    /// Calculate viewport-consistent pixel size
    pub fn calculate_viewport_pixel_size(
        viewport_size: (u32, u32),
        reference_resolution: (u32, u32),
        pixels_per_unit: f32,
        maintain_aspect_ratio: bool,
    ) -> f32 {
        if !maintain_aspect_ratio {
            return 1.0 / pixels_per_unit;
        }
        
        let scale_factor = calculate_viewport_scale_factor(viewport_size, reference_resolution);
        1.0 / (pixels_per_unit * scale_factor)
    }
    
    /// Calculate scale factor for viewport consistency
    pub fn calculate_viewport_scale_factor(
        current_resolution: (u32, u32),
        reference_resolution: (u32, u32),
    ) -> f32 {
        let ref_aspect = reference_resolution.0 as f32 / reference_resolution.1 as f32;
        let current_aspect = current_resolution.0 as f32 / current_resolution.1 as f32;
        
        if current_aspect > ref_aspect {
            // Current is wider - scale by height
            current_resolution.1 as f32 / reference_resolution.1 as f32
        } else {
            // Current is taller - scale by width
            current_resolution.0 as f32 / reference_resolution.0 as f32
        }
    }
    
    /// Snap a matrix to pixel boundaries
    pub fn snap_matrix_to_pixels(
        matrix: glam::Mat4,
        pixels_per_unit: f32,
        viewport_size: (u32, u32),
    ) -> glam::Mat4 {
        if pixels_per_unit <= 0.0 {
            return matrix;
        }
        
        // Extract translation from matrix
        let translation = matrix.w_axis.truncate();
        
        // Snap translation to pixel boundaries
        let pixel_size = 1.0 / pixels_per_unit;
        let snapped_translation = glam::Vec3::new(
            (translation.x / pixel_size).round() * pixel_size,
            (translation.y / pixel_size).round() * pixel_size,
            translation.z,
        );
        
        // Reconstruct matrix with snapped translation
        let mut snapped_matrix = matrix;
        snapped_matrix.w_axis = snapped_translation.extend(matrix.w_axis.w);
        
        snapped_matrix
    }
    
    /// Calculate orthographic projection matrix for perfect pixel rendering
    pub fn calculate_pixel_perfect_orthographic(
        viewport_size: (u32, u32),
        pixels_per_unit: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> glam::Mat4 {
        let half_width = (viewport_size.0 as f32 * 0.5) / pixels_per_unit;
        let half_height = (viewport_size.1 as f32 * 0.5) / pixels_per_unit;
        
        glam::Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            near_clip,
            far_clip,
        )
    }
    
    /// Check if two positions are pixel-aligned
    pub fn are_positions_pixel_aligned(
        pos1: glam::Vec3,
        pos2: glam::Vec3,
        pixels_per_unit: f32,
        threshold: f32,
    ) -> bool {
        if pixels_per_unit <= 0.0 {
            return false;
        }
        
        let pixel_size = 1.0 / pixels_per_unit;
        let diff = pos1 - pos2;
        
        let x_aligned = (diff.x / pixel_size).fract().abs() < threshold;
        let y_aligned = (diff.y / pixel_size).fract().abs() < threshold;
        
        x_aligned && y_aligned
    }
}

/// View mode for unified 2D/3D rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    /// 2D orthographic mode optimized for pixel-perfect rendering
    Mode2D,
    /// 3D perspective mode with full 3D capabilities
    Mode3D,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Mode2D
    }
}

/// Filter mode for texture sampling in perfect pixel rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    /// Nearest neighbor filtering for pixel-perfect rendering
    Nearest,
    /// Linear filtering for smooth scaling
    Linear,
}

impl Default for FilterMode {
    fn default() -> Self {
        FilterMode::Nearest
    }
}

/// Pixel snapping mode for different rendering scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelSnapMode {
    /// Always snap to pixel boundaries
    Always,
    /// Only snap when scale is near integer values
    IntegerScaleOnly,
    /// Snap based on distance threshold
    Threshold,
    /// No pixel snapping
    Never,
}

impl Default for PixelSnapMode {
    fn default() -> Self {
        PixelSnapMode::Always
    }
}

/// Perfect pixel rendering settings component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfectPixelSettings {
    /// Whether perfect pixel rendering is enabled
    pub enabled: bool,
    /// Whether to snap positions to pixel boundaries
    pub snap_to_pixel: bool,
    /// Filter mode for texture sampling
    pub filter_mode: FilterMode,
    /// How many pixels equal 1 world unit (Unity standard: 100)
    pub pixels_per_unit: f32,
    /// Reference resolution for consistent scaling
    pub reference_resolution: (u32, u32),
    /// Snap threshold - positions within this distance are snapped to pixels
    pub snap_threshold: f32,
    /// Whether to maintain aspect ratio when scaling
    pub maintain_aspect_ratio: bool,
    /// Pixel snap mode for different scenarios
    pub snap_mode: PixelSnapMode,
}

impl Default for PerfectPixelSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_pixel: true,
            filter_mode: FilterMode::Nearest,
            pixels_per_unit: 100.0, // Unity standard
            reference_resolution: (1920, 1080), // Common reference resolution
            snap_threshold: 0.01, // 1% threshold for snapping
            maintain_aspect_ratio: true,
            snap_mode: PixelSnapMode::Always,
        }
    }
}

impl PerfectPixelSettings {
    /// Create new perfect pixel settings with custom pixels per unit
    pub fn new(pixels_per_unit: f32) -> Self {
        Self {
            pixels_per_unit,
            ..Default::default()
        }
    }
    
    /// Create settings optimized for 2D pixel art
    pub fn pixel_art() -> Self {
        Self {
            enabled: true,
            snap_to_pixel: true,
            filter_mode: FilterMode::Nearest,
            pixels_per_unit: 100.0,
            reference_resolution: (1920, 1080),
            snap_threshold: 0.001, // Very precise snapping for pixel art
            maintain_aspect_ratio: true,
            snap_mode: PixelSnapMode::Always,
        }
    }
    
    /// Create settings for smooth 2D graphics
    pub fn smooth_2d() -> Self {
        Self {
            enabled: true,
            snap_to_pixel: false,
            filter_mode: FilterMode::Linear,
            pixels_per_unit: 100.0,
            reference_resolution: (1920, 1080),
            snap_threshold: 0.1,
            maintain_aspect_ratio: true,
            snap_mode: PixelSnapMode::IntegerScaleOnly,
        }
    }
    
    /// Disable perfect pixel rendering
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            snap_to_pixel: false,
            filter_mode: FilterMode::Linear,
            pixels_per_unit: 100.0,
            reference_resolution: (1920, 1080),
            snap_threshold: 0.0,
            maintain_aspect_ratio: false,
            snap_mode: PixelSnapMode::Never,
        }
    }
    
    /// Calculate the pixel size in world units
    pub fn pixel_size(&self) -> f32 {
        if self.pixels_per_unit <= 0.0 {
            0.01 // Fallback to prevent division by zero
        } else {
            1.0 / self.pixels_per_unit
        }
    }
    
    /// Calculate scale factor for viewport consistency
    pub fn calculate_viewport_scale(&self, current_resolution: (u32, u32)) -> f32 {
        if !self.maintain_aspect_ratio {
            return 1.0;
        }
        
        let ref_aspect = self.reference_resolution.0 as f32 / self.reference_resolution.1 as f32;
        let current_aspect = current_resolution.0 as f32 / current_resolution.1 as f32;
        
        if current_aspect > ref_aspect {
            // Current is wider - scale by height
            current_resolution.1 as f32 / self.reference_resolution.1 as f32
        } else {
            // Current is taller - scale by width
            current_resolution.0 as f32 / self.reference_resolution.0 as f32
        }
    }
}

/// Unified camera component that extends the base Camera with 2D/3D mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCamera {
    /// Current view mode (2D or 3D)
    pub view_mode: ViewMode,
    
    /// Perfect pixel rendering enabled for this camera
    pub perfect_pixel_enabled: bool,
    
    /// Threshold for pixel snapping (smaller values = more precise snapping)
    pub pixel_snap_threshold: f32,
    
    // 2D Mode Settings
    /// Orthographic size (half-height in world units) for 2D mode
    pub orthographic_size: f32,
    /// Pixels per unit override for this camera (None = use global setting)
    pub pixels_per_unit: Option<f32>,
    
    // 3D Mode Settings
    /// Field of view in degrees for 3D perspective mode
    pub fov: f32,
    /// Near clipping plane distance
    pub near_clip: f32,
    /// Far clipping plane distance
    pub far_clip: f32,
    
    // Transition Settings
    /// Enable smooth transitions between 2D and 3D modes
    pub smooth_transition: bool,
    /// Speed of mode transitions (higher = faster)
    pub transition_speed: f32,
}

impl Default for UnifiedCamera {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Mode2D,
            perfect_pixel_enabled: true,
            pixel_snap_threshold: 0.01,
            
            // 2D settings
            orthographic_size: 5.0,
            pixels_per_unit: None, // Use global setting
            
            // 3D settings
            fov: 60.0,
            near_clip: 0.3,
            far_clip: 1000.0,
            
            // Transition settings
            smooth_transition: true,
            transition_speed: 5.0,
        }
    }
}

/// Enhanced sprite component for unified 2D/3D rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSprite {
    /// Texture identifier
    pub texture_id: String,
    /// Sprite width in world units
    pub width: f32,
    /// Sprite height in world units
    pub height: f32,
    /// Sprite tint color (RGBA)
    pub color: [f32; 4],
    
    // 2D/3D Rendering Options
    /// Whether sprite should face the camera in 3D mode (billboard)
    pub billboard: bool,
    /// Render as world space UI element
    pub world_space_ui: bool,
    /// Enable perfect pixel rendering for this sprite
    pub pixel_perfect: bool,
    /// Manual depth sorting order (higher values render on top)
    pub sort_order: i32,
    
    // Perfect Pixel Settings
    /// Override pixels per unit for this sprite (None = use camera/global setting)
    pub pixels_per_unit: Option<f32>,
}

impl Default for UnifiedSprite {
    fn default() -> Self {
        Self {
            texture_id: String::new(),
            width: 1.0,
            height: 1.0,
            color: [1.0, 1.0, 1.0, 1.0], // White
            billboard: false,
            world_space_ui: false,
            pixel_perfect: true,
            sort_order: 0,
            pixels_per_unit: None,
        }
    }
}

/// Enhanced tilemap component for unified 2D/3D rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTilemap {
    /// Tileset identifier
    pub tileset_id: String,
    /// Tile data storage (grid coordinates to tile IDs)
    pub tiles: std::collections::HashMap<(i32, i32), u32>,
    /// Chunk size for efficient rendering
    pub chunk_size: (u32, u32),
    
    // 2D/3D Rendering
    /// Depth layer for this tilemap (affects rendering order)
    pub layer_depth: f32,
    /// Enable perfect pixel rendering for this tilemap
    pub pixel_perfect: bool,
    /// Scale factor when rendering in world space
    pub world_space_scale: f32,
    
    // Perfect Pixel Settings
    /// Override pixels per unit for this tilemap (None = use camera/global setting)
    pub pixels_per_unit: Option<f32>,
    /// Size of each tile in pixels
    pub tile_size: (u32, u32),
}

impl Default for UnifiedTilemap {
    fn default() -> Self {
        Self {
            tileset_id: String::new(),
            tiles: std::collections::HashMap::new(),
            chunk_size: (16, 16), // 16x16 tile chunks
            layer_depth: 0.0,
            pixel_perfect: true,
            world_space_scale: 1.0,
            pixels_per_unit: None,
            tile_size: (32, 32), // 32x32 pixel tiles
        }
    }
}

/// Viewport configuration for unified rendering
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Screen rectangle (x, y, width, height)
    pub rect: [f32; 4],
    /// Pixel dimensions (width, height)
    pub size: (u32, u32),
    /// DPI scaling factor
    pub scale_factor: f32,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Optional render target texture
    pub target_texture: Option<String>, // Texture ID for render-to-texture
    /// Previous size for change detection
    pub previous_size: (u32, u32),
    /// Viewport consistency settings
    pub consistency_settings: ViewportConsistencySettings,
}

/// Settings for maintaining viewport consistency across size changes
#[derive(Debug, Clone)]
pub struct ViewportConsistencySettings {
    /// Whether to maintain pixel ratio consistency
    pub maintain_pixel_ratio: bool,
    /// Reference resolution for scaling calculations
    pub reference_resolution: (u32, u32),
    /// Scaling mode for viewport changes
    pub scaling_mode: ViewportScalingMode,
    /// Minimum scale factor to prevent over-scaling
    pub min_scale_factor: f32,
    /// Maximum scale factor to prevent under-scaling
    pub max_scale_factor: f32,
}

/// Viewport scaling modes for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportScalingMode {
    /// Scale to fit width (may letterbox vertically)
    FitWidth,
    /// Scale to fit height (may pillarbox horizontally)
    FitHeight,
    /// Scale to fit both dimensions (may distort aspect ratio)
    Stretch,
    /// Scale to fill viewport (may crop content)
    Fill,
    /// Use integer scaling only (crisp pixels)
    IntegerScale,
}

impl Default for ViewportConsistencySettings {
    fn default() -> Self {
        Self {
            maintain_pixel_ratio: true,
            reference_resolution: (1920, 1080),
            scaling_mode: ViewportScalingMode::FitHeight,
            min_scale_factor: 0.1,
            max_scale_factor: 10.0,
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            rect: [0.0, 0.0, 1.0, 1.0], // Full screen normalized
            size: (1920, 1080),
            scale_factor: 1.0,
            view_mode: ViewMode::Mode2D,
            target_texture: None,
            previous_size: (1920, 1080),
            consistency_settings: ViewportConsistencySettings::default(),
        }
    }
}

impl Viewport {
    /// Create a new viewport with specific size
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            size: (width, height),
            previous_size: (width, height),
            ..Default::default()
        }
    }
    
    /// Update viewport size and handle consistency
    pub fn update_size(&mut self, new_width: u32, new_height: u32) -> bool {
        let size_changed = self.size.0 != new_width || self.size.1 != new_height;
        
        if size_changed {
            self.previous_size = self.size;
            self.size = (new_width, new_height);
            
            if self.consistency_settings.maintain_pixel_ratio {
                self.update_scale_factor();
            }
        }
        
        size_changed
    }
    
    /// Update scale factor based on consistency settings
    fn update_scale_factor(&mut self) {
        let new_scale = self.calculate_consistency_scale_factor();
        self.scale_factor = new_scale.clamp(
            self.consistency_settings.min_scale_factor,
            self.consistency_settings.max_scale_factor,
        );
    }
    
    /// Calculate scale factor for viewport consistency
    pub fn calculate_consistency_scale_factor(&self) -> f32 {
        let ref_res = self.consistency_settings.reference_resolution;
        let current_width = self.size.0 as f32;
        let current_height = self.size.1 as f32;
        let ref_width = ref_res.0 as f32;
        let ref_height = ref_res.1 as f32;

        match self.consistency_settings.scaling_mode {
            ViewportScalingMode::FitWidth => current_width / ref_width,
            ViewportScalingMode::FitHeight => current_height / ref_height,
            ViewportScalingMode::Fill => {
                // Use the larger scale to fill viewport
                (current_width / ref_width).max(current_height / ref_height)
            },
            ViewportScalingMode::Stretch => {
                // Use average of both dimensions
                ((current_width / ref_width) + (current_height / ref_height)) / 2.0
            },
            ViewportScalingMode::IntegerScale => {
                // Use smaller dimension and round to integer
                let scale = (current_width / ref_width).min(current_height / ref_height);
                scale.floor().max(1.0)
            },
        }
    }
}

/// Perfect pixel transform data for positioning calculations
#[derive(Debug, Clone)]
pub struct PixelPerfectTransform {
    /// Original world position
    pub world_position: glam::Vec3,
    /// Position snapped to pixel boundaries
    pub snapped_position: glam::Vec3,
    /// Scale adjusted for pixel-perfect rendering
    pub pixel_scale: glam::Vec3,
    /// Pixels per unit used for this transform
    pub pixels_per_unit: f32,
}

impl PixelPerfectTransform {
    /// Create a new pixel perfect transform
    pub fn new(world_position: glam::Vec3, pixels_per_unit: f32) -> Self {
        let snapped_position = Self::snap_to_pixel(world_position, pixels_per_unit);
        let pixel_scale = glam::Vec3::ONE; // Default scale
        
        Self {
            world_position,
            snapped_position,
            pixel_scale,
            pixels_per_unit,
        }
    }
    
    /// Create a pixel perfect transform with settings
    pub fn with_settings(
        world_position: glam::Vec3, 
        world_scale: glam::Vec3,
        settings: &PerfectPixelSettings
    ) -> Self {
        let snapped_position = if settings.enabled && settings.snap_to_pixel {
            Self::snap_to_pixel_with_mode(world_position, settings.pixels_per_unit, settings.snap_mode, settings.snap_threshold)
        } else {
            world_position
        };
        
        let pixel_scale = if settings.enabled {
            Self::calculate_pixel_scale(world_scale, settings.pixels_per_unit)
        } else {
            world_scale
        };
        
        Self {
            world_position,
            snapped_position,
            pixel_scale,
            pixels_per_unit: settings.pixels_per_unit,
        }
    }
    
    /// Snap a world position to pixel boundaries
    pub fn snap_to_pixel(position: glam::Vec3, pixels_per_unit: f32) -> glam::Vec3 {
        Self::snap_to_pixel_with_mode(position, pixels_per_unit, PixelSnapMode::Always, 0.01)
    }
    
    /// Snap position with specific mode and threshold
    pub fn snap_to_pixel_with_mode(
        position: glam::Vec3, 
        pixels_per_unit: f32, 
        snap_mode: PixelSnapMode,
        threshold: f32
    ) -> glam::Vec3 {
        if pixels_per_unit <= 0.0 {
            return position;
        }
        
        match snap_mode {
            PixelSnapMode::Never => position,
            PixelSnapMode::Always => {
                let pixel_size = 1.0 / pixels_per_unit;
                glam::Vec3::new(
                    (position.x / pixel_size).round() * pixel_size,
                    (position.y / pixel_size).round() * pixel_size,
                    position.z, // Don't snap Z in 3D
                )
            },
            PixelSnapMode::Threshold => {
                let pixel_size = 1.0 / pixels_per_unit;
                let snapped_x = (position.x / pixel_size).round() * pixel_size;
                let snapped_y = (position.y / pixel_size).round() * pixel_size;
                
                // Only snap if within threshold
                let x = if (position.x - snapped_x).abs() <= threshold { snapped_x } else { position.x };
                let y = if (position.y - snapped_y).abs() <= threshold { snapped_y } else { position.y };
                
                glam::Vec3::new(x, y, position.z)
            },
            PixelSnapMode::IntegerScaleOnly => {
                // Only snap if we're rendering at near-integer scale
                let pixel_size = 1.0 / pixels_per_unit;
                let scale_x = 1.0 / pixel_size; // Assuming unit scale for now
                let scale_y = 1.0 / pixel_size;
                
                if Self::is_near_integer_scale(scale_x, scale_y, threshold) {
                    glam::Vec3::new(
                        (position.x / pixel_size).round() * pixel_size,
                        (position.y / pixel_size).round() * pixel_size,
                        position.z,
                    )
                } else {
                    position
                }
            }
        }
    }
    
    /// Calculate pixel-perfect scale for a given world scale
    pub fn calculate_pixel_scale(world_scale: glam::Vec3, pixels_per_unit: f32) -> glam::Vec3 {
        if pixels_per_unit <= 0.0 {
            return world_scale;
        }
        
        // For pixel-perfect rendering, prefer integer scales when possible
        let pixel_size = 1.0 / pixels_per_unit;
        glam::Vec3::new(
            Self::quantize_scale(world_scale.x, pixel_size),
            Self::quantize_scale(world_scale.y, pixel_size),
            world_scale.z, // Don't quantize Z scale
        )
    }
    
    /// Quantize a scale value to pixel boundaries
    fn quantize_scale(scale: f32, pixel_size: f32) -> f32 {
        let pixel_scale = scale / pixel_size;
        let rounded_scale = pixel_scale.round().max(1.0);
        rounded_scale * pixel_size
    }
    
    /// Check if a scale should use nearest neighbor filtering
    pub fn should_use_nearest_filter(scale: glam::Vec3) -> bool {
        // Use nearest filtering for integer or near-integer scales
        let x_near_int = (scale.x.round() - scale.x).abs() < 0.01;
        let y_near_int = (scale.y.round() - scale.y).abs() < 0.01;
        x_near_int && y_near_int
    }
    
    /// Check if scale values are near integers
    fn is_near_integer_scale(scale_x: f32, scale_y: f32, threshold: f32) -> bool {
        let x_near_int = (scale_x.round() - scale_x).abs() <= threshold;
        let y_near_int = (scale_y.round() - scale_y).abs() <= threshold;
        x_near_int && y_near_int
    }
    
    /// Calculate pixel offset for sub-pixel positioning
    pub fn calculate_pixel_offset(&self) -> glam::Vec2 {
        let pixel_size = 1.0 / self.pixels_per_unit;
        glam::Vec2::new(
            (self.world_position.x - self.snapped_position.x) / pixel_size,
            (self.world_position.y - self.snapped_position.y) / pixel_size,
        )
    }
    
    /// Update the transform with new world position
    pub fn update_position(&mut self, new_position: glam::Vec3, settings: &PerfectPixelSettings) {
        self.world_position = new_position;
        
        if settings.enabled && settings.snap_to_pixel {
            self.snapped_position = Self::snap_to_pixel_with_mode(
                new_position, 
                settings.pixels_per_unit, 
                settings.snap_mode, 
                settings.snap_threshold
            );
        } else {
            self.snapped_position = new_position;
        }
    }
    
    /// Update the transform with new world scale
    pub fn update_scale(&mut self, new_scale: glam::Vec3, settings: &PerfectPixelSettings) {
        if settings.enabled {
            self.pixel_scale = Self::calculate_pixel_scale(new_scale, settings.pixels_per_unit);
        } else {
            self.pixel_scale = new_scale;
        }
    }
    
    /// Get the effective position for rendering (snapped or world)
    pub fn get_render_position(&self, use_snapping: bool) -> glam::Vec3 {
        if use_snapping {
            self.snapped_position
        } else {
            self.world_position
        }
    }
    
    /// Get the effective scale for rendering
    pub fn get_render_scale(&self) -> glam::Vec3 {
        self.pixel_scale
    }
}