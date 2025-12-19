//! Helper functions for unified 2D/3D rendering
//!
//! This module provides convenient functions for working with the unified rendering system.

use crate::{Camera, CameraProjection};
use crate::components::{ViewMode, PerfectPixelSettings};
use crate::components::unified_rendering::PixelSnapMode;

/// Helper functions for unified rendering
pub struct UnifiedRenderingHelpers;

impl UnifiedRenderingHelpers {
    /// Create a camera with unified 2D/3D rendering enabled
    pub fn create_unified_camera() -> Camera {
        let mut camera = Camera::default();
        camera.enable_unified_rendering();
        camera
    }
    
    /// Create a camera optimized for 2D pixel-perfect rendering
    pub fn create_2d_camera(pixels_per_unit: f32, orthographic_size: f32) -> Camera {
        let mut camera = Camera::default();
        camera.projection = CameraProjection::Orthographic;
        camera.orthographic_size = orthographic_size;
        camera.pixels_per_unit = pixels_per_unit;
        
        // Enable unified rendering with 2D mode
        camera.enable_unified_rendering();
        if let Some(ref mut unified) = camera.unified_rendering {
            unified.view_mode = ViewMode::Mode2D;
            unified.perfect_pixel_enabled = true;
            unified.pixels_per_unit = Some(pixels_per_unit);
            unified.orthographic_size = orthographic_size;
        }
        
        camera
    }
    
    /// Create a camera optimized for 3D perspective rendering
    pub fn create_3d_camera(fov: f32, near_clip: f32, far_clip: f32) -> Camera {
        let mut camera = Camera::default();
        camera.projection = CameraProjection::Perspective;
        camera.fov = fov;
        camera.near_clip = near_clip;
        camera.far_clip = far_clip;
        
        // Enable unified rendering with 3D mode
        camera.enable_unified_rendering();
        if let Some(ref mut unified) = camera.unified_rendering {
            unified.view_mode = ViewMode::Mode3D;
            unified.perfect_pixel_enabled = false; // Disabled in 3D
            unified.fov = fov;
            unified.near_clip = near_clip;
            unified.far_clip = far_clip;
        }
        
        camera
    }
    
    /// Switch a camera to 2D mode with perfect pixel rendering
    pub fn switch_to_2d_mode(camera: &mut Camera, pixels_per_unit: Option<f32>) {
        camera.enable_unified_rendering();
        camera.set_view_mode(ViewMode::Mode2D);
        
        if let Some(ref mut unified) = camera.unified_rendering {
            unified.perfect_pixel_enabled = true;
            if let Some(ppu) = pixels_per_unit {
                unified.pixels_per_unit = Some(ppu);
            }
        }
    }
    
    /// Switch a camera to 3D mode with perspective projection
    pub fn switch_to_3d_mode(camera: &mut Camera, fov: Option<f32>) {
        camera.enable_unified_rendering();
        camera.set_view_mode(ViewMode::Mode3D);
        
        if let Some(ref mut unified) = camera.unified_rendering {
            unified.perfect_pixel_enabled = false;
            if let Some(field_of_view) = fov {
                unified.fov = field_of_view;
            }
        }
    }
    
    /// Create default perfect pixel settings
    pub fn create_perfect_pixel_settings() -> PerfectPixelSettings {
        PerfectPixelSettings::default()
    }
    
    /// Create perfect pixel settings for pixel art
    pub fn create_pixel_art_settings(pixels_per_unit: f32) -> PerfectPixelSettings {
        PerfectPixelSettings {
            enabled: true,
            snap_to_pixel: true,
            filter_mode: crate::components::FilterMode::Nearest,
            pixels_per_unit,
            reference_resolution: (1920, 1080),
            snap_threshold: 0.01,
            maintain_aspect_ratio: true,
            snap_mode: PixelSnapMode::Always,
        }
    }
    
    /// Create perfect pixel settings for high-resolution sprites
    pub fn create_hd_sprite_settings(pixels_per_unit: f32) -> PerfectPixelSettings {
        PerfectPixelSettings {
            enabled: true,
            snap_to_pixel: false, // Allow sub-pixel positioning for HD
            filter_mode: crate::components::FilterMode::Linear,
            pixels_per_unit,
            reference_resolution: (1920, 1080),
            snap_threshold: 0.01,
            maintain_aspect_ratio: true,
            snap_mode: PixelSnapMode::Never,
        }
    }
    
    /// Check if a camera is in 2D mode
    pub fn is_2d_mode(camera: &Camera) -> bool {
        matches!(camera.get_view_mode(), Some(ViewMode::Mode2D))
    }
    
    /// Check if a camera is in 3D mode
    pub fn is_3d_mode(camera: &Camera) -> bool {
        matches!(camera.get_view_mode(), Some(ViewMode::Mode3D))
    }
    
    /// Get the effective pixels per unit for a camera
    pub fn get_pixels_per_unit(camera: &Camera) -> f32 {
        camera.get_effective_pixels_per_unit()
    }
}