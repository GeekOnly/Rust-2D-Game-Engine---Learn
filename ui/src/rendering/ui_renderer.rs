//! UI Renderer - Integration with the render crate
//!
//! This module provides the bridge between the UI system and the WGPU-based
//! rendering pipeline.

use super::{UIBatch, UIVertex};

/// UI Render Pass
///
/// Manages the rendering of UI batches to the screen using WGPU.
/// This integrates with the existing sprite renderer in the render crate.
pub struct UIRenderPass {
    /// Vertex buffer for UI rendering
    vertex_buffer: Option<wgpu::Buffer>,
    
    /// Index buffer for UI rendering
    index_buffer: Option<wgpu::Buffer>,
    
    /// Render pipeline for UI
    render_pipeline: Option<wgpu::RenderPipeline>,
    
    /// Bind group layout for textures
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    
    /// Current capacity of vertex buffer
    vertex_capacity: usize,
    
    /// Current capacity of index buffer
    index_capacity: usize,
}

impl UIRenderPass {
    /// Create a new UI render pass
    pub fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            render_pipeline: None,
            bind_group_layout: None,
            vertex_capacity: 0,
            index_capacity: 0,
        }
    }

    /// Initialize the render pass with WGPU device and configuration
    ///
    /// This should be called once during setup with the WGPU device and surface configuration.
    pub fn initialize(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) {
        // Create bind group layout for textures
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Texture Bind Group Layout"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // Position
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // UV
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // Color
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // UI doesn't need culling
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // UI renders on top, no depth testing
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.render_pipeline = Some(render_pipeline);
        self.bind_group_layout = Some(bind_group_layout);
    }

    /// Ensure buffers have sufficient capacity
    fn ensure_buffer_capacity(
        &mut self,
        device: &wgpu::Device,
        vertex_count: usize,
        index_count: usize,
    ) {
        // Ensure vertex buffer capacity
        if vertex_count > self.vertex_capacity {
            let new_capacity = (vertex_count * 2).max(1024);
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Vertex Buffer"),
                size: (new_capacity * std::mem::size_of::<UIVertex>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = new_capacity;
        }

        // Ensure index buffer capacity
        if index_count > self.index_capacity {
            let new_capacity = (index_count * 2).max(2048);
            self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Index Buffer"),
                size: (new_capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.index_capacity = new_capacity;
        }
    }

    /// Render UI batches
    ///
    /// This should be called during the render pass to draw all UI elements.
    ///
    /// # Arguments
    ///
    /// * `device` - WGPU device
    /// * `queue` - WGPU queue for uploading data
    /// * `encoder` - Command encoder for recording render commands
    /// * `view` - Target texture view to render to
    /// * `batches` - UI batches to render
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        batches: &[UIBatch],
    ) {
        if batches.is_empty() {
            return;
        }

        // Initialize if not already done
        if self.render_pipeline.is_none() {
            // Can't render without initialization
            return;
        }

        // Calculate total vertex and index counts
        let total_vertices: usize = batches.iter().map(|b| b.vertices.len()).sum();
        let total_indices: usize = batches.iter().map(|b| b.indices.len()).sum();

        if total_vertices == 0 || total_indices == 0 {
            return;
        }

        // Ensure buffers have sufficient capacity
        self.ensure_buffer_capacity(device, total_vertices, total_indices);

        // Collect all vertices and indices
        let mut all_vertices = Vec::with_capacity(total_vertices);
        let mut all_indices = Vec::with_capacity(total_indices);
        let mut vertex_offset = 0u32;

        for batch in batches {
            all_vertices.extend_from_slice(&batch.vertices);
            
            // Offset indices for this batch
            for &index in &batch.indices {
                all_indices.push(vertex_offset + index);
            }
            
            vertex_offset += batch.vertices.len() as u32;
        }

        // Upload vertex data
        if let Some(ref vertex_buffer) = self.vertex_buffer {
            queue.write_buffer(
                vertex_buffer,
                0,
                bytemuck::cast_slice(&all_vertices),
            );
        }

        // Upload index data
        if let Some(ref index_buffer) = self.index_buffer {
            queue.write_buffer(
                index_buffer,
                0,
                bytemuck::cast_slice(&all_indices),
            );
        }

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Don't clear, render on top
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        if let Some(ref pipeline) = self.render_pipeline {
            render_pass.set_pipeline(pipeline);
        }

        if let Some(ref vertex_buffer) = self.vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        if let Some(ref index_buffer) = self.index_buffer {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }

        // Draw all batches
        // TODO: Set bind groups for textures per batch
        // For now, draw everything with a default white texture
        let mut index_offset = 0;
        for batch in batches {
            let index_count = batch.indices.len() as u32;
            if index_count > 0 {
                render_pass.draw_indexed(index_offset..index_offset + index_count, 0, 0..1);
                index_offset += index_count;
            }
        }
    }
}

impl Default for UIRenderPass {
    fn default() -> Self {
        Self::new()
    }
}

// Implement bytemuck traits for UIVertex to allow safe casting
unsafe impl bytemuck::Pod for UIVertex {}
unsafe impl bytemuck::Zeroable for UIVertex {}
