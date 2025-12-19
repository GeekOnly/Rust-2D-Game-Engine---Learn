//! Unified 2D/3D Rendering Components
//!
//! This module contains components and types for the unified 2D/3D rendering system
//! that allows seamless integration of 2D sprites and tilemaps with 3D content.

use serde::{Deserialize, Serialize};

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
}

impl Default for PerfectPixelSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_pixel: true,
            filter_mode: FilterMode::Nearest,
            pixels_per_unit: 100.0, // Unity standard
            reference_resolution: (1920, 1080), // Common reference resolution
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
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            rect: [0.0, 0.0, 1.0, 1.0], // Full screen normalized
            size: (1920, 1080),
            scale_factor: 1.0,
            view_mode: ViewMode::Mode2D,
            target_texture: None,
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
    
    /// Snap a world position to pixel boundaries
    pub fn snap_to_pixel(position: glam::Vec3, pixels_per_unit: f32) -> glam::Vec3 {
        if pixels_per_unit <= 0.0 {
            return position;
        }
        
        let pixel_size = 1.0 / pixels_per_unit;
        glam::Vec3::new(
            (position.x / pixel_size).round() * pixel_size,
            (position.y / pixel_size).round() * pixel_size,
            position.z, // Don't snap Z in 3D
        )
    }
    
    /// Calculate pixel-perfect scale for a given world scale
    pub fn calculate_pixel_scale(world_scale: glam::Vec3, pixels_per_unit: f32) -> glam::Vec3 {
        if pixels_per_unit <= 0.0 {
            return world_scale;
        }
        
        // For pixel-perfect rendering, prefer integer scales when possible
        let pixel_size = 1.0 / pixels_per_unit;
        glam::Vec3::new(
            (world_scale.x / pixel_size).round().max(1.0) * pixel_size,
            (world_scale.y / pixel_size).round().max(1.0) * pixel_size,
            world_scale.z, // Don't quantize Z scale
        )
    }
    
    /// Check if a scale should use nearest neighbor filtering
    pub fn should_use_nearest_filter(scale: glam::Vec3) -> bool {
        // Use nearest filtering for integer or near-integer scales
        let x_near_int = (scale.x.round() - scale.x).abs() < 0.01;
        let y_near_int = (scale.y.round() - scale.y).abs() < 0.01;
        x_near_int && y_near_int
    }
}