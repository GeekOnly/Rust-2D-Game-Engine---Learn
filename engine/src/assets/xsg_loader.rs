use crate::assets::xsg::*;
use ecs::{World, Entity};
use render::{MeshRenderer, Mesh, PbrMaterial, ModelVertex, Texture};
use crate::texture_manager::TextureManager;
use crate::runtime::render_system::{register_mesh_asset, register_material_asset};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use ecs::traits::ComponentAccess;
use log::{info, warn};
use image;
use crate::assets::model_manager::get_model_manager;

pub struct XsgLoader;

impl XsgLoader {

    pub fn load_to_manager(
        xsg: XsgFile,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path_id: &str,
    ) -> anyhow::Result<()> {
        Self::load_resources(&xsg, device, queue, path_id)?;
        get_model_manager().add_model(path_id.to_string(), xsg);
        Ok(())
    }

    pub fn load_resources(
         xsg: &XsgFile,
         device: &wgpu::Device,
         queue: &wgpu::Queue,
         path_id: &str,
    ) -> anyhow::Result<(std::collections::HashMap<u32, String>, std::collections::HashMap<u32, String>)> {
         let mut mesh_map = std::collections::HashMap::new(); // Index -> Asset ID
         let mut material_map = std::collections::HashMap::new(); // Index -> Asset ID

         // 1. Load Textures
        let mut texture_map = std::collections::HashMap::new();
        for (i, xsg_tex) in xsg.textures.iter().enumerate() {
            let tex_id = format!("{}_tex_{}_{}", path_id, xsg_tex.name, i);
            let mut loaded_texture = None;
            
            if let Some(uri) = &xsg_tex.uri {
                 let xsg_path = std::path::Path::new(path_id);
                 // If path_id is just a name, parent might be empty.
                 let parent = xsg_path.parent().unwrap_or(std::path::Path::new("."));
                 let full_path = parent.join(uri);
                 
                 if let Ok(img) = image::open(&full_path) {
                     // Use Texture directly as it is imported
                     if let Ok(tex) = Texture::from_image(device, queue, &img, Some(&tex_id), None) {
                         loaded_texture = Some(Arc::new(tex));
                     }
                 } else {
                     warn!("Failed to load texture at {:?}", full_path);
                 }
            } else if let Some(data) = &xsg_tex.data {
                 if let Ok(img) = image::load_from_memory(data) {
                     if let Ok(tex) = Texture::from_image(device, queue, &img, Some(&tex_id), None) {
                         loaded_texture = Some(Arc::new(tex));
                     }
                 }
            }
            
            if let Some(tex) = loaded_texture {
                texture_map.insert(i as u32, tex);
            }
        }

        // 2. Load Materials
        for (i, xsg_mat) in xsg.materials.iter().enumerate() {
             let mat_id = format!("{}_mat_{}_{}", path_id, xsg_mat.name, i);
             
             // Resolve textures
             let albedo = xsg_mat.base_color_texture.and_then(|idx| texture_map.get(&idx).cloned());
             let normal = xsg_mat.normal_texture.and_then(|idx| texture_map.get(&idx).cloned());
             
             let material = Arc::new(PbrMaterial {
                 albedo_texture: albedo,
                 normal_texture: normal,
                 albedo_factor: xsg_mat.base_color_factor,
                 metallic_factor: xsg_mat.metallic_factor,
                 roughness_factor: xsg_mat.roughness_factor,
                 bind_group: None, 
                 ..Default::default()
             });
             
             register_material_asset(mat_id.clone(), material);
             material_map.insert(i as u32, mat_id);
        }

        // 3. Load Meshes (Create GPU Buffers)
        for (i, xsg_mesh) in xsg.meshes.iter().enumerate() {
            // A mesh can have multiple primitives. 
            // In our simple ECS, 1 Entity = 1 Mesh.
            
            for (prim_idx, prim) in xsg_mesh.primitives.iter().enumerate() {
                let mesh_id = format!("{}_mesh_{}_{}_{}", path_id, xsg_mesh.name, i, prim_idx);
                
                // Interleave vertices
                let vertex_count = prim.positions.len();
                let mut vertices = Vec::with_capacity(vertex_count);
                
                for v_idx in 0..vertex_count {
                    let pos = prim.positions[v_idx];
                    let normal = if v_idx < prim.normals.len() { prim.normals[v_idx] } else { [0.0, 1.0, 0.0] };
                    let uv = if v_idx < prim.uvs.len() { prim.uvs[v_idx] } else { [0.0, 0.0] };
                    
                    vertices.push(ModelVertex {
                        position: pos,
                        tex_coords: uv,
                        normal,
                        tangent: [0.0; 3], // TODO: Calculate tangents
                        bitangent: [0.0; 3],
                    });
                }
                
                // Create Mesh using helper that handles buffer creation
                let mesh = Mesh::new(
                    device, 
                    &mesh_id, 
                    &vertices, 
                    &prim.indices
                );
                
                register_mesh_asset(mesh_id.clone(), Arc::new(mesh));
                
                // Store mapping: (MeshIndex, PrimIndex) -> MeshID
                // For now just storing mapped to MeshIndex assuming 1 primitive
                if prim_idx == 0 {
                    mesh_map.insert(i as u32, mesh_id);
                }
            }
        }
        
        Ok((mesh_map, material_map))
    }

    pub fn load_into_world(
        xsg: &XsgFile,
        world: &mut World,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _texture_manager: &mut TextureManager, // Unused now
        path_id: &str,
    ) -> anyhow::Result<Vec<Entity>> {
        let (mesh_map, material_map) = Self::load_resources(xsg, device, queue, path_id)?;
        let mut created_entities = Vec::new();
        
        // 4. Create Entities (Nodes)
        // We map XSG Node Index -> ECS Entity
        let mut node_to_entity = std::collections::HashMap::new();
        
        for (i, node) in xsg.nodes.iter().enumerate() {
            let entity = world.spawn();
            node_to_entity.insert(i as u32, entity);
            created_entities.push(entity);
            
            // Name
            let _ = ComponentAccess::<String>::insert(world, entity, node.name.clone());
            
            // Transform
            // Convert start logic...
            let q = glam::Quat::from_array(node.transform.rotation);
            let e = q.to_euler(glam::EulerRot::XYZ);
            let e_deg = [e.0.to_degrees(), e.1.to_degrees(), e.2.to_degrees()];
            let transform = ecs::Transform {
                position: node.transform.position,
                rotation: e_deg,
                scale: node.transform.scale,
            };
            
            let _ = ComponentAccess::<ecs::Transform>::insert(world, entity, transform);
            
            // Initialize GlobalTransform (will be updated by system)
            let _ = ComponentAccess::<ecs::GlobalTransform>::insert(world, entity, ecs::GlobalTransform::default());
            
            // Mesh
            if let Some(mesh_idx) = node.mesh {
                if let Some(mesh_id) = mesh_map.get(&mesh_idx) {
                    // Find material ID
                    let mat_id = xsg.meshes[mesh_idx as usize].primitives.first()
                        .and_then(|p| p.material_index)
                        .and_then(|mi| material_map.get(&mi))
                        .cloned();

                    let _ = ComponentAccess::<ecs::Mesh>::insert(world, entity, ecs::Mesh {
                        mesh_type: ecs::MeshType::Asset(mesh_id.clone()),
                        color: [1.0, 1.0, 1.0, 1.0], 
                        material_id: mat_id,
                    });
                }
            }
        }
        
        // 5. Restore Hierarchy
        for (i, node) in xsg.nodes.iter().enumerate() {
            let parent_entity = node_to_entity[&(i as u32)];
            for child_idx in &node.children {
                if let Some(child_entity) = node_to_entity.get(child_idx) {
                    world.set_parent(*child_entity, Some(parent_entity));
                }
            }
        }
        
        Ok(created_entities)
    }
}
