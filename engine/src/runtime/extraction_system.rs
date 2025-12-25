use ecs::World;
use crate::runtime::render_frame::RenderFrame;
use render::{MeshInstance, LightUniform};

pub struct ExtractionSystem;

impl ExtractionSystem {
    pub fn extract(world: &World, _dt: f32) -> RenderFrame {
        let mut frame = RenderFrame::new();

        // 1. Extract Camera
        // Ideally we find the active camera. For now, assumes Main Camera logic is elsewhere or we extract first camera.
        // Actually, renderer usually holds camera binding. 
        // But for "Extraction", we might want to capture camera state from ECS.
        // For phase 1/2 refactor, we might skip camera extraction if it's handled by 'app' or 'render_system' legacy logic,
        // We need active camera. For now, picking first one or default.
        if let Some((_, _camera)) = world.cameras.iter().next() {
            // Calculate view/proj
            // frame.camera.update(...)
            // For now, let's leave camera default or assume it's set by the caller (render_system) before draw?
            // Or we extract it here.
        }
        
        // 2. Extract Meshes (Component)
        for (entity, mesh_component) in &world.meshes {
            if let Some(transform) = world.transforms.get(entity) {
                // Visibility Check
                    if let Some(active) = world.active.get(entity) { if !active { continue; } }
                if let Some(visible) = world.visibles.get(entity) { if !visible.is_visible { continue; } }

                let model_matrix = if let Some(global) = world.global_transforms.get(entity) {
                    glam::Mat4::from_cols_array(&global.matrix)
                } else {
                     let t = glam::Vec3::from_array(transform.position);
                     let r = glam::Quat::from_euler(
                        glam::EulerRot::XYZ,
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians()
                     );
                     let s = glam::Vec3::from_array(transform.scale);
                     glam::Mat4::from_scale_rotation_translation(s, r, t)
                };
                
                // Convert to MeshInstance
                let instance = MeshInstance {
                    model: model_matrix.to_cols_array_2d(),
                    color: mesh_component.color,
                };

                let mesh_id = match &mesh_component.mesh_type {
                    ecs::MeshType::Cube => "Cube".to_string(),
                    ecs::MeshType::Sphere => "Sphere".to_string(),
                    ecs::MeshType::Cylinder => "Cylinder".to_string(),
                    ecs::MeshType::Plane => "Plane".to_string(),
                    ecs::MeshType::Capsule => "Capsule".to_string(),
                    ecs::MeshType::Asset(id) => id.clone(),
                };
                // If Asset is just a file path (e.g. .xsg), we skip it for now as it usually has children?
                // Actually, mesh_type::Asset usually refers to a specific mesh asset name, NOT the .xsg file itself.
                // But in existing code, .xsg loading creates children with Asset mesh types?
                // Revisit: "mesh_renderer.render_pbr" uses "mesh_cache.get".
                
                let mat_id = mesh_component.material_id.clone().unwrap_or_else(|| "default".to_string());

                frame.push_instance(mesh_id, mat_id, instance);
            }
        }
        
        // 3. Extract Model3D (Hierarchy)
        // Access ModelManager
        let model_manager = crate::assets::model_manager::get_model_manager();
        
        for (entity, model_3d) in &world.model_3ds {
            if let Some(visible) = world.active.get(entity) { if !visible { continue; } }
            
             if let Some(xsg) = model_manager.get_model(&model_3d.asset_id) {
                 // Calculate Root Transform
                 let root_transform = if let Some(global) = world.global_transforms.get(entity) {
                     glam::Mat4::from_cols_array(&global.matrix)
                 } else if let Some(transform) = world.transforms.get(entity) {
                      let t = glam::Vec3::from_array(transform.position);
                      let r = glam::Quat::from_euler(glam::EulerRot::XYZ, 
                          transform.rotation[0].to_radians(), 
                          transform.rotation[1].to_radians(), 
                          transform.rotation[2].to_radians());
                      let s = glam::Vec3::from_array(transform.scale);
                      glam::Mat4::from_scale_rotation_translation(s, r, t)
                 } else {
                      glam::Mat4::IDENTITY
                 };

                 let mut stack = Vec::new();
                 for root_idx in &xsg.root_nodes {
                     stack.push((*root_idx, root_transform));
                 }

                 while let Some((node_idx, parent_mat)) = stack.pop() {
                     let node = &xsg.nodes[node_idx as usize];
                     let q = glam::Quat::from_array(node.transform.rotation);
                     let t = glam::Vec3::from(node.transform.position);
                     let s = glam::Vec3::from(node.transform.scale);
                     let local_mat = glam::Mat4::from_scale_rotation_translation(s, q, t);
                     let global_mat = parent_mat * local_mat;

                     if let Some(mesh_idx) = node.mesh {
                          let mesh_name = &xsg.meshes[mesh_idx as usize].name;
                          // Iterate primitives
                          for (prim_idx, prim) in xsg.meshes[mesh_idx as usize].primitives.iter().enumerate() {
                                let mesh_id = format!("{}_mesh_{}_{}_{}", model_3d.asset_id, mesh_name, mesh_idx, prim_idx);
                                let mat_id = prim.material_index.and_then(|mi| {
                                     let mname = &xsg.materials[mi as usize].name;
                                     Some(format!("{}_mat_{}_{}", model_3d.asset_id, mname, mi))
                                }).unwrap_or("default".to_string());
                                
                                let instance = MeshInstance {
                                    model: global_mat.to_cols_array_2d(),
                                    color: [1.0; 4], // Model3D doesn't have per-instance color override yet
                                };
                                
                                frame.push_instance(mesh_id, mat_id, instance);
                          }
                     }

                     for child_idx in &node.children {
                         stack.push((*child_idx, global_mat));
                     }
                 }
            }
        }


        // 4. Extract Lights
        for (entity, light) in &world.lights {
            if let Some(visible) = world.active.get(entity) { if !visible { continue; } }
            
            let position = if let Some(global) = world.global_transforms.get(entity) {
                // Extract translation from matrix
                let col3 = &global.matrix[12..15];
                [col3[0], col3[1], col3[2]]
            } else if let Some(transform) = world.transforms.get(entity) {
                transform.position
            } else {
                [0.0, 0.0, 0.0]
            };

            let uniform = LightUniform::new(position, light.color, light.intensity);
            frame.lights.push(uniform);
        }

        frame
    }
}
