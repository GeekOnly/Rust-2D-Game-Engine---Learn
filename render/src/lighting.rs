use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 4], // xyz is position, w is generic/padding (or intensity if we pack)
    pub color: [f32; 4],    // rgb is color, a is intensity
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            position: [position[0], position[1], position[2], 1.0],
            color: [color[0], color[1], color[2], intensity],
        }
    }
}

pub struct LightBinding {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl LightBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = LightUniform::new([0.0; 3], [1.0; 3], 1.0);

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("light_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("light_bind_group"),
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, position: [f32; 3], color: [f32; 3], intensity: f32) {
        let uniform = LightUniform::new(position, color, intensity);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
