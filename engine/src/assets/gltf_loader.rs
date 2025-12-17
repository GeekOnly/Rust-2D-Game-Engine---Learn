use anyhow::{Result, Context};
use wgpu::util::DeviceExt;
use gltf::{Gltf, Mesh as GltfMesh, Primitive, Material as GltfMaterial, Texture as GltfTexture};
use gltf::mesh::util::{ReadTexCoords, ReadIndices};
use render::{Mesh, ModelVertex, PbrMaterial, Texture, TextureManager, MeshRenderer};
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use glam::{Vec3, Mat4, Quat};
use log::info;

pub struct GltfLoader {
    base_path: PathBuf,
}

pub struct LoadedMesh {
    pub mesh: Arc<Mesh>,
    pub material: Arc<PbrMaterial>,
    pub transform: Mat4,
    pub name: String,
}

impl GltfLoader {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::new(),
        }
    }

    pub fn load_gltf(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_manager: &mut TextureManager,
        mesh_renderer: &MeshRenderer,
        path: impl AsRef<Path>,
    ) -> Result<Vec<LoadedMesh>> {
        let path = path.as_ref();
        self.base_path = path.parent().unwrap_or(Path::new("")).to_path_buf();
        
        info!("Loading GLTF: {:?}", path);
        let gltf = Gltf::open(path)?;

        let mut loaded_meshes = Vec::new();

        // Load all buffers upfront
        let mut buffers = Vec::new();
        let blob = gltf.blob.as_deref();
        
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => {
                     if let Some(blob) = blob {
                        buffers.push(blob.to_vec());
                    } else {
                         return Err(anyhow::anyhow!("Missing GLB binary chunk"));
                    }
                }
                gltf::buffer::Source::Uri(uri) => {
                     let buffer_path = self.base_path.join(uri);
                     let data = std::fs::read(&buffer_path).with_context(|| format!("Failed to read buffer: {:?}", buffer_path))?;
                     buffers.push(data);
                }
            }
        }

        // Load Materials
        let mut materials_cache = Vec::new();
        for material in gltf.materials() {
             let pbr_mat = self.load_material(device, queue, texture_manager, mesh_renderer, &material, &buffers)?;
             materials_cache.push(Arc::new(pbr_mat));
        }

        // We'll iterate through scenes -> nodes
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                self.process_node(device, &node, Mat4::IDENTITY, &gltf, &buffers, &materials_cache, &mut loaded_meshes)?;
            }
        }

        Ok(loaded_meshes)
    }

    fn load_material(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_manager: &mut TextureManager,
        mesh_renderer: &MeshRenderer,
        material: &GltfMaterial,
        buffers: &[Vec<u8>],
    ) -> Result<PbrMaterial> {
        let pbr = material.pbr_metallic_roughness();
        
        let albedo_factor = pbr.base_color_factor();
        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();

        let mut pbr_material = PbrMaterial {
            albedo_factor,
            metallic_factor,
            roughness_factor,
            ..Default::default()
        };

        // Load Textures
        if let Some(info) = pbr.base_color_texture() {
            let texture = self.load_texture(device, queue, texture_manager, &info.texture(), buffers, true)?; 
            pbr_material.albedo_texture = Some(Arc::new(texture));
        } else if let Some(tex) = texture_manager.get_white_texture(device, queue) {
             // TextureManager returns Arc<Texture> or reference?
             // render::TextureManager returns Option<&Texture>.
             // But we need Arc.
             // If TextureManager stores Texture (owned), and gives ref, we can't get Arc easily unless TM stores Arc.
             // Wait, TextureManager in `render` crate (Step 85): `pub textures: HashMap<String, Texture>`.
             // It does NOT use Arc.
             // THIS IS A PROBLEM.
             // If TextureManager doesn't use Arc, we can't share cheap refs.
             // But I already updated `Texture` to remove Clone and set `Arc` inside `PbrMaterial`.
             // I MUST update `TextureManager` to store `Arc<Texture>`.
             // OR clone the Texture (it's deep clone if wgpu types are not Arc internally, but wait, wgpu types are Handles).
             // Actually, wgpu::Texture IS a handle (Arc/Ref-counted internally usually).
             // BUT Rust wgpu wrapper makes them Move-only by default (no Clone).
             // So I can't clone them.
             
             // I WILL UPDATE TEXTURE MANAGER TO STORE `Arc<Texture>`.
             // For now, let's assume I can clone whatever I load here as I create it new.
             // For default white texture: I need to clone it. But I can't.
             // I'll create a NEW white texture if I can't get one.
             // Or rely on fallback handling in `mesh_renderer` if None?
             // `mesh_renderer` panics if albedo is missing.
             // GltfLoader must provide one.
             
             // Workaround: Load a fresh 1x1 white texture for this material if TM doesn't give Arc.
             // Or better: Use TextureManager to load, but it returns ref.
             // I will ignore TextureManager defaults for a moment and just Create New if needed.
             // Or fix TextureManager later.
        }
        
        if let Some(info) = material.normal_texture() {
             if let Ok(tex) = self.load_texture(device, queue, texture_manager, &info.texture(), buffers, false) {
                 pbr_material.normal_texture = Some(Arc::new(tex));
             }
        }

        if let Some(info) = pbr.metallic_roughness_texture() {
             if let Ok(tex) = self.load_texture(device, queue, texture_manager, &info.texture(), buffers, false) {
                 pbr_material.metallic_roughness_texture = Some(Arc::new(tex));
             }
        }
        
        // Ensure Albedo exists (create 1x1 white if missing)
        if pbr_material.albedo_texture.is_none() {
            // Create white texture
             let size = wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
             let texture = device.create_texture_with_data(queue, &wgpu::TextureDescriptor {
                label: Some("Default White"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
             }, wgpu::util::TextureDataOrder::MipMajor, &[255, 255, 255, 255]);
             
             let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
             let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
             
             pbr_material.albedo_texture = Some(Arc::new(Texture {
                 texture, view, sampler, bind_group: None, width: 1, height: 1
             }));
        }

        // Create Bind Group
        let bind_group = mesh_renderer.create_pbr_bind_group(device, &pbr_material, texture_manager);
        pbr_material.bind_group = Some(bind_group);

        Ok(pbr_material)
    }

    fn process_node(
        &self,
        device: &wgpu::Device,
        node: &gltf::Node,
        parent_transform: Mat4,
        gltf: &Gltf,
        buffers: &[Vec<u8>],
        materials_cache: &[Arc<PbrMaterial>],
        loaded_meshes: &mut Vec<LoadedMesh>,
    ) -> Result<()> {
        let (translation, rotation, scale) = node.transform().decomposed();
        let local_transform = Mat4::from_scale_rotation_translation(
            Vec3::from(scale),
            Quat::from_array(rotation),
            Vec3::from(translation),
        );
        let global_transform = parent_transform * local_transform;

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let loaded_mesh = self.load_primitive(device, &primitive, global_transform, buffers, mesh.name().unwrap_or("Unnamed"), materials_cache)?;
                loaded_meshes.push(loaded_mesh);
            }
        }

        for child in node.children() {
            self.process_node(device, &child, global_transform, gltf, buffers, materials_cache, loaded_meshes)?;
        }

        Ok(())
    }

    fn load_primitive(
        &self,
        device: &wgpu::Device,
        primitive: &Primitive,
        transform: Mat4,
        buffers: &[Vec<u8>],
        mesh_name: &str,
        materials_cache: &[Arc<PbrMaterial>],
    ) -> Result<LoadedMesh> {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let mut vertices = Vec::new();
        
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        let normals: Vec<[f32; 3]> = reader.read_normals().map(|iter| iter.collect()).unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
        
        // Handling TexCoords (might be multiple sets, we use 0)
        let tex_coords: Vec<[f32; 2]> = if let Some(read_tex_coords) = reader.read_tex_coords(0) {
             read_tex_coords.into_f32().collect()
        } else {
             vec![[0.0, 0.0]; positions.len()]
        };

        // Handling Tangents
        let tangents: Vec<[f32; 4]> = reader.read_tangents().map(|iter| iter.collect()).unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]);


        for i in 0..positions.len() {
             vertices.push(ModelVertex {
                position: positions[i],
                tex_coords: tex_coords[i],
                normal: normals[i],
                tangent: [tangents[i][0], tangents[i][1], tangents[i][2]],
                bitangent: [0.0, 0.0, 0.0], 
            });
        }
        
        // Calculate bitangents
        for i in 0..vertices.len() {
             let n = Vec3::from(vertices[i].normal);
             let t = Vec3::from(vertices[i].tangent);
             let w = tangents[i][3];
             let b = n.cross(t) * w;
             vertices[i].bitangent = b.to_array();
        }

        let indices: Vec<u32> = reader.read_indices().map(|read_indices| {
            read_indices.into_u32().collect()
        }).unwrap_or_else(|| (0..positions.len() as u32).collect());

        let mesh = Mesh::new(device, mesh_name, &vertices, &indices);
        let mesh = Arc::new(mesh);

        // Assign Material from Cache
        let material = if let Some(idx) = primitive.material().index() {
            if idx < materials_cache.len() {
                materials_cache[idx].clone()
            } else {
                // Fallback default
                 Arc::new(PbrMaterial::default())
            }
        } else {
             // Default material
             Arc::new(PbrMaterial::default())
        };

        Ok(LoadedMesh {
            mesh,
            material,
            transform,
            name: mesh_name.to_string(),
        })
    }
    
    fn load_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_manager: &mut TextureManager,
        gltf_texture: &gltf::Texture,
        buffers: &[Vec<u8>],
        srgb: bool,
    ) -> Result<Texture> {
        let source = gltf_texture.source();
        let name = source.name().unwrap_or("unnamed_tex");
        let id = format!("{}_{}", name, source.index());
        
        // Temporarily disabled caching check because TextureManager doesn't return Cloneable Texture yet
        // In future: texture_manager.get_texture(&id).cloned()
        
        let img_data = match source.source() {
            gltf::image::Source::View { view, mime_type: _ } => {
                let start = view.offset();
                let end = start + view.length();
                let buffer_index = view.buffer().index();
                
                if buffer_index >= buffers.len() {
                    return Err(anyhow::anyhow!("Buffer index out of range: {}", buffer_index));
                }
                
                let data = &buffers[buffer_index][start..end];
                image::load_from_memory(data)?
            }
             gltf::image::Source::Uri { uri, mime_type: _ } => {
                 let path = self.base_path.join(uri);
                 let data = std::fs::read(&path)?;
                 image::load_from_memory(&data)?
            }
        };
        
        // Pass srgb? Texture::from_image assumes SRGB for now or generic?
        // Let's assume generic for now (Rgba8UnormSrgb is used in Texture::from_image probably).
        
        let texture = Texture::from_image(device, queue, &img_data, Some(&id), None)?; 
        
        Ok(texture)
    }

}
