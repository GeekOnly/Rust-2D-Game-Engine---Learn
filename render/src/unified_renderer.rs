//! Unified 2D/3D Rendering Pipeline
//!
//! This module provides the core WGPU pipeline modifications for unified 2D/3D rendering,
//! enabling seamless integration of 2D sprites and tilemaps with 3D content.

use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};

/// Unified camera uniform for WGPU shaders
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UnifiedCameraUniform {
    /// Combined view-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Camera world position (vec3 padded to vec4)
    pub view_pos: [f32; 4],
    /// View mode (0.0 = 2D, 1.0 = 3D)
    pub view_mode: f32,
    /// Perfect pixel settings (pixels_per_unit, snap_threshold, enabled, padding)
    pub perfect_pixel: [f32; 4],
    /// Viewport size (width, height, scale_factor, padding)
    pub viewport: [f32; 4],
}

impl UnifiedCameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            view_pos: [0.0; 4],
            view_mode: 0.0, // Default to 2D
            perfect_pixel: [100.0, 0.01, 1.0, 0.0], // pixels_per_unit, threshold, enabled, padding
            viewport: [1920.0, 1080.0, 1.0, 0.0], // width, height, scale_factor, padding
        }
    }

    pub fn update_2d(
        &mut self,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        camera_pos: Vec3,
        pixels_per_unit: f32,
        snap_threshold: f32,
        perfect_pixel_enabled: bool,
        viewport_size: (u32, u32),
        scale_factor: f32,
    ) {
        self.view_proj = (projection_matrix * view_matrix).to_cols_array_2d();
        self.view_pos = [camera_pos.x, camera_pos.y, camera_pos.z, 1.0];
        self.view_mode = 0.0; // 2D mode
        self.perfect_pixel = [
            pixels_per_unit,
            snap_threshold,
            if perfect_pixel_enabled { 1.0 } else { 0.0 },
            0.0,
        ];
        self.viewport = [
            viewport_size.0 as f32,
            viewport_size.1 as f32,
            scale_factor,
            0.0,
        ];
    }

    pub fn update_3d(
        &mut self,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        camera_pos: Vec3,
        viewport_size: (u32, u32),
        scale_factor: f32,
    ) {
        self.view_proj = (projection_matrix * view_matrix).to_cols_array_2d();
        self.view_pos = [camera_pos.x, camera_pos.y, camera_pos.z, 1.0];
        self.view_mode = 1.0; // 3D mode
        self.perfect_pixel = [0.0, 0.0, 0.0, 0.0]; // Disabled in 3D
        self.viewport = [
            viewport_size.0 as f32,
            viewport_size.1 as f32,
            scale_factor,
            0.0,
        ];
    }
}

/// Unified camera binding for WGPU
pub struct UnifiedCameraBinding {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl std::fmt::Debug for UnifiedCameraBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedCameraBinding")
            .field("buffer", &"wgpu::Buffer")
            .field("bind_group", &"wgpu::BindGroup")
            .field("bind_group_layout", &"wgpu::BindGroupLayout")
            .finish()
    }
}

impl UnifiedCameraBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = UnifiedCameraUniform::new();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unified Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("unified_camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("unified_camera_bind_group"),
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update_2d(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        camera_pos: Vec3,
        pixels_per_unit: f32,
        snap_threshold: f32,
        perfect_pixel_enabled: bool,
        viewport_size: (u32, u32),
        scale_factor: f32,
    ) {
        let mut uniform = UnifiedCameraUniform::new();
        uniform.update_2d(
            view_matrix,
            projection_matrix,
            camera_pos,
            pixels_per_unit,
            snap_threshold,
            perfect_pixel_enabled,
            viewport_size,
            scale_factor,
        );
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_3d(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        camera_pos: Vec3,
        viewport_size: (u32, u32),
        scale_factor: f32,
    ) {
        let mut uniform = UnifiedCameraUniform::new();
        uniform.update_3d(view_matrix, projection_matrix, camera_pos, viewport_size, scale_factor);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}

/// Render context for unified 2D/3D rendering
pub struct UnifiedRenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub camera_binding: UnifiedCameraBinding,
    pub view_mode: ecs::components::ViewMode,
    pub perfect_pixel_settings: ecs::components::PerfectPixelSettings,
    pub viewport: ecs::components::Viewport,
}

impl std::fmt::Debug for UnifiedRenderContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedRenderContext")
            .field("device", &"wgpu::Device")
            .field("queue", &"wgpu::Queue")
            .field("camera_binding", &self.camera_binding)
            .field("view_mode", &self.view_mode)
            .field("perfect_pixel_settings", &self.perfect_pixel_settings)
            .field("viewport", &self.viewport)
            .finish()
    }
}

/// Trait for managing view modes in the unified rendering system
pub trait ViewModeManager {
    fn set_view_mode(&mut self, mode: ecs::components::ViewMode);
    fn get_view_mode(&self) -> ecs::components::ViewMode;
    fn toggle_view_mode(&mut self);
    fn is_transitioning(&self) -> bool;
}

/// Trait for perfect pixel rendering calculations
pub trait PerfectPixelRenderer {
    fn snap_to_pixel(&self, position: Vec3, pixels_per_unit: f32) -> Vec3;
    fn calculate_pixel_scale(&self, world_scale: Vec3, pixels_per_unit: f32) -> Vec3;
    fn should_use_nearest_filter(&self, scale: Vec3) -> bool;
}

/// Trait for the unified render pipeline
pub trait UnifiedRenderPipeline {
    fn render_2d_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport);
    fn render_3d_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport);
    fn render_mixed_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport);
    fn apply_perfect_pixel(&mut self, settings: &ecs::components::PerfectPixelSettings);
}

/// Implementation of perfect pixel rendering utilities
pub struct PerfectPixelUtils;

impl PerfectPixelRenderer for PerfectPixelUtils {
    fn snap_to_pixel(&self, position: Vec3, pixels_per_unit: f32) -> Vec3 {
        ecs::components::PixelPerfectTransform::snap_to_pixel(position, pixels_per_unit)
    }

    fn calculate_pixel_scale(&self, world_scale: Vec3, pixels_per_unit: f32) -> Vec3 {
        ecs::components::PixelPerfectTransform::calculate_pixel_scale(world_scale, pixels_per_unit)
    }

    fn should_use_nearest_filter(&self, scale: Vec3) -> bool {
        ecs::components::PixelPerfectTransform::should_use_nearest_filter(scale)
    }
}

/// Unified rendering pipeline implementation
pub struct UnifiedRenderer {
    pub camera_binding: UnifiedCameraBinding,
    pub perfect_pixel_utils: PerfectPixelUtils,
    pub current_view_mode: ecs::components::ViewMode,
    pub transitioning: bool,
}

impl UnifiedRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera_binding: UnifiedCameraBinding::new(device),
            perfect_pixel_utils: PerfectPixelUtils,
            current_view_mode: ecs::components::ViewMode::Mode2D,
            transitioning: false,
        }
    }
}

impl ViewModeManager for UnifiedRenderer {
    fn set_view_mode(&mut self, mode: ecs::components::ViewMode) {
        if self.current_view_mode != mode {
            self.transitioning = true;
            self.current_view_mode = mode;
            // Transition logic would be implemented here
            self.transitioning = false;
        }
    }

    fn get_view_mode(&self) -> ecs::components::ViewMode {
        self.current_view_mode
    }

    fn toggle_view_mode(&mut self) {
        let new_mode = match self.current_view_mode {
            ecs::components::ViewMode::Mode2D => ecs::components::ViewMode::Mode3D,
            ecs::components::ViewMode::Mode3D => ecs::components::ViewMode::Mode2D,
        };
        self.set_view_mode(new_mode);
    }

    fn is_transitioning(&self) -> bool {
        self.transitioning
    }
}

impl UnifiedRenderPipeline for UnifiedRenderer {
    fn render_2d_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport) {
        // Implementation would render 2D sprites and tilemaps with perfect pixel settings
        // This is a placeholder for the actual rendering logic
    }

    fn render_3d_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport) {
        // Implementation would render 3D meshes with full lighting and materials
        // This is a placeholder for the actual rendering logic
    }

    fn render_mixed_content(&mut self, camera: &ecs::Camera, viewport: &ecs::components::Viewport) {
        // Implementation would render both 2D and 3D content with proper depth sorting
        // This is a placeholder for the actual rendering logic
    }

    fn apply_perfect_pixel(&mut self, settings: &ecs::components::PerfectPixelSettings) {
        // Implementation would apply perfect pixel settings to the rendering pipeline
        // This is a placeholder for the actual perfect pixel logic
    }
}