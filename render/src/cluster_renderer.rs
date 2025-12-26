use crate::CameraBinding;

// Constants (Must match Shader)
pub const TILE_SIZE: u32 = 16;
pub const MAX_LIGHTS_PER_CLUSTER: u32 = 64;
pub const CLUSTER_Z_SLICES: u32 = 24;

// GPU Structs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Cluster {
    pub offset: u32,
    pub count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPULight {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub radius: f32,
    pub padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterUniform {
    pub inverse_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub near_plane: f32,
    pub far_plane: f32,
}

pub struct ClusterRenderer {
    pub compute_pipeline: wgpu::ComputePipeline,
    
    // Compute-only (read-write for bindings 2,3)
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    
    // Fragment-only (read-only for all storage bindings)
    pub fragment_bind_group_layout: wgpu::BindGroupLayout,
    pub fragment_bind_group: wgpu::BindGroup,
    
    // Buffers
    pub light_buffer: wgpu::Buffer,
    pub cluster_buffer: wgpu::Buffer,
    pub global_light_index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    
    // Grid Dimensions
    pub grid_dims: [u32; 3],
}

impl ClusterRenderer {
    pub fn new(
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration, 
        camera_binding: &CameraBinding
    ) -> Self {
        // 1. Calculate Grid Dimensions
        let width = config.width;
        let height = config.height;
        let grid_dims = [
            (width as f32 / TILE_SIZE as f32).ceil() as u32,
            (height as f32 / TILE_SIZE as f32).ceil() as u32,
            CLUSTER_Z_SLICES,
        ];
        
        let total_clusters = grid_dims[0] * grid_dims[1] * grid_dims[2];
        let total_indices = total_clusters * MAX_LIGHTS_PER_CLUSTER;
        
        // 2. Create Buffers
        let max_global_lights = 1024;
        let light_buffer_size = (max_global_lights * std::mem::size_of::<GPULight>()) as wgpu::BufferAddress;
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Light Buffer"),
            size: light_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cluster_buffer_size = (total_clusters as usize * std::mem::size_of::<Cluster>()) as wgpu::BufferAddress;
        let cluster_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Buffer"),
            size: cluster_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer_size = (total_indices as usize * std::mem::size_of::<u32>()) as wgpu::BufferAddress;
        let global_light_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Global Light Index Buffer"),
            size: index_buffer_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let uniform_size = std::mem::size_of::<ClusterUniform>() as wgpu::BufferAddress;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Uniform Buffer"),
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 3. Compute Bind Group Layout (read-write for clusters/indices)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cluster Compute Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, // Compute writes
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, // Compute writes
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // 4. Fragment Bind Group Layout (read-only for all storage)
        let fragment_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cluster Fragment Layout"),
            entries: &[
                // Binding 1: Lights (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 2: Clusters (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 3: Global Indices (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 4: Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // 5. Compute Bind Group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_binding.buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: light_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: cluster_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: global_light_index_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: uniform_buffer.as_entire_binding() },
            ],
        });
        
        // 6. Fragment Bind Group (same buffers, different layout)
        let fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Fragment Bind Group"),
            layout: &fragment_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 1, resource: light_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: cluster_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: global_light_index_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: uniform_buffer.as_entire_binding() },
            ],
        });
        
        // 7. Compute Pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cluster Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("cluster_culling.wgsl").into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cluster Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cluster Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            compute_pipeline,
            bind_group_layout,
            bind_group,
            fragment_bind_group_layout,
            fragment_bind_group,
            light_buffer,
            cluster_buffer,
            global_light_index_buffer,
            uniform_buffer,
            grid_dims,
        }
    }
    
    pub fn compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Light Culling Pass"),
            timestamp_writes: None,
        });
        
        pass.set_pipeline(&self.compute_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        
        let x_groups = (self.grid_dims[0] as f32 / 16.0).ceil() as u32;
        let y_groups = (self.grid_dims[1] as f32 / 16.0).ceil() as u32;
        let z_groups = self.grid_dims[2];
        
        pass.dispatch_workgroups(x_groups, y_groups, z_groups);
    }
    
    pub fn update_lights(&self, queue: &wgpu::Queue, lights: &[GPULight]) {
        let count = lights.len().min(1024);
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&lights[0..count]));
    }

    pub fn update_view(&self, queue: &wgpu::Queue, view: glam::Mat4, proj: glam::Mat4, screen_size: (f32, f32), near: f32, far: f32) {
        let inverse_proj = proj.inverse();
        let uniform = ClusterUniform {
            inverse_proj: inverse_proj.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            screen_size: [screen_size.0, screen_size.1],
            near_plane: near,
            far_plane: far,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
