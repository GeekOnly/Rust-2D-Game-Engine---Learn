use std::sync::Arc;
use crate::texture::Texture;

pub struct PbrMaterial {
    pub albedo_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub metallic_roughness_texture: Option<Arc<Texture>>,
    pub occlusion_texture: Option<Arc<Texture>>,
    
    // Factors if textures are missing
    pub albedo_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            albedo_factor: [1.0; 4],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            bind_group: None,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrMaterialUniform {
    pub albedo_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub padding: [f32; 2], // Pad to 16-byte alignment (vec4 is 16 bytes, f32 is 4. Total 16+4+4 = 24. Needed 32)
}

impl PbrMaterialUniform {
    pub fn new(material: &PbrMaterial) -> Self {
        Self {
            albedo_factor: material.albedo_factor,
            metallic_factor: material.metallic_factor,
            roughness_factor: material.roughness_factor,
            padding: [0.0; 2],
        }
    }
}

pub struct ToonMaterial {
    pub color: [f32; 4],
    pub outline_width: f32, // For inverted hull
    pub outline_color: [f32; 4],
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Default for ToonMaterial {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            outline_width: 0.02,
            outline_color: [0.0, 0.0, 0.0, 1.0],
            bind_group: None,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ToonMaterialUniform {
    pub color: [f32; 4],
    pub outline_color: [f32; 4],
    pub params: [f32; 4], // x: outline_width, y,z,w: padding/unused
}

impl ToonMaterialUniform {
    pub fn new(material: &ToonMaterial) -> Self {
        Self {
            color: material.color,
            outline_color: material.outline_color,
            params: [material.outline_width, 0.0, 0.0, 0.0],
        }
    }
}
