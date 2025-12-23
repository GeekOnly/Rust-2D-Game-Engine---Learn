use egui_wgpu::wgpu;
use egui;
use egui_wgpu;

pub struct GameViewRenderer {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub texture_id: egui::TextureId,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl GameViewRenderer {
    pub fn new(
        device: &wgpu::Device,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) -> Self {
        let format = wgpu::TextureFormat::Bgra8UnormSrgb; // Use sRGB for game view

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game View Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Register with egui
        let texture_id = egui_renderer.register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Linear,
        );

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game View Depth Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            texture_id,
            depth_texture,
            depth_view,
            width,
            height,
            format,
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) {
        if (width == self.width && height == self.height) || width == 0 || height == 0 {
            return;
        }

        self.width = width;
        self.height = height;

        // Recreate texture
        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game View Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.view = self.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Update egui registration
        egui_renderer.update_egui_texture_from_wgpu_texture(
            device,
            &self.view,
            wgpu::FilterMode::Linear,
            self.texture_id,
        );

        // Recreate depth texture
        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game View Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }
}
