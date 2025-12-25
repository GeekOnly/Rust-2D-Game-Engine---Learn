use wgpu::util::DeviceExt;

pub struct ShadowTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl ShadowTexture {
    pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let size = wgpu::Extent3d {
            width: 2048, 
            height: 2048,
            depth_or_array_layers: 2, // 2 Cascades
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            array_layer_count: Some(2),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub view_proj: [[[f32; 4]; 4]; 4], // 4 Cascades x 64 bytes
    pub splits: [f32; 4], // Split distances
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32, view_projs: [[[f32; 4]; 4]; 4], splits: [f32; 4]) -> Self {
        Self {
            position: [position[0], position[1], position[2], 1.0],
            color: [color[0], color[1], color[2], intensity],
            view_proj: view_projs,
            splits,
        }
    }
}

pub struct LightBinding {
    pub buffer: wgpu::Buffer,
    pub shadow_texture: ShadowTexture,
    pub scene_depth_texture: wgpu::TextureView, // Copy of Scene Depth
    pub scene_depth_sampler: wgpu::Sampler, // Regular sampler for scene depth
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl LightBinding {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        // Initial Dummy Data
        let uniform = LightUniform::new(
            [0.0; 3], 
            [1.0; 3], 
            1.0, 
            [[[0.0; 4]; 4]; 4],
            [0.0; 4]
        );

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let shadow_texture = ShadowTexture::new(device, config);

        // Scene Depth Texture (Copy Destination)
        let scene_depth_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Depth Copy"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let scene_depth_view = scene_depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Regular sampler for scene depth (AAA Mobile: bilinear filtering for contact shadows)
        let scene_depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Scene Depth Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None, // Regular sampler (not comparison)
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Light Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX, // Vertex needs matrix? Actually usually Frag does for pixel calc, but vertex might need it strictly? Let's check logic. Usually Frag does shadow compare.
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Shadow Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array, // Updated to Array
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                // Shadow Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                // Scene Depth Texture (Binding 3)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                // Scene Depth Sampler (Binding 4)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("light_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&scene_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&scene_depth_sampler),
                },
            ],
            label: Some("light_bind_group"),
        });

        Self {
            buffer,
            shadow_texture,
            scene_depth_texture: scene_depth_view,
            scene_depth_sampler,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, position: [f32; 3], color: [f32; 3], intensity: f32, view_projs: [[[f32; 4]; 4]; 4], splits: [f32; 4]) {
        let uniform = LightUniform::new(position, color, intensity, view_projs, splits);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_resources(&mut self, device: &wgpu::Device, new_scene_depth_view: &wgpu::TextureView) {
        // Recreate Bind Group with new Scene Depth
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(new_scene_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.scene_depth_sampler),
                },
            ],
            label: Some("light_bind_group"),
        });
    }
}
