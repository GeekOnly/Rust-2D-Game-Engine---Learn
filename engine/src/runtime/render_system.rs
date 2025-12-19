use ecs::World;
use render::{BatchRenderer, MeshRenderer, TextureManager, CameraBinding, LightBinding, Mesh, PbrMaterialUniform, ObjectUniform, PbrMaterial, TilemapGpuResources};
use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use crate::assets::model_manager::get_model_manager;
use anyhow;

// Simple mesh cache to avoid regenerating meshes every frame
static mut MESH_CACHE: Option<HashMap<String, Mesh>> = None;
// Asset caches (Loaded from files)
static mut MESH_ASSETS: Option<HashMap<String, Arc<Mesh>>> = None;
static mut MATERIAL_ASSETS: Option<HashMap<String, Arc<PbrMaterial>>> = None;
static mut MATERIAL_BIND_GROUP_CACHE: Option<HashMap<String, wgpu::BindGroup>> = None;

static mut MATERIAL_CACHE: Option<wgpu::BindGroup> = None;

fn get_mesh_cache() -> &'static mut HashMap<String, Mesh> {
    unsafe {
        if MESH_CACHE.is_none() {
            MESH_CACHE = Some(HashMap::new());
        }
        MESH_CACHE.as_mut().unwrap()
    }
}

fn get_mesh_assets() -> &'static mut HashMap<String, Arc<Mesh>> {
    unsafe {
        if MESH_ASSETS.is_none() {
            MESH_ASSETS = Some(HashMap::new());
        }
        MESH_ASSETS.as_mut().unwrap()
    }
}

fn get_material_assets() -> &'static mut HashMap<String, Arc<PbrMaterial>> {
    unsafe {
        if MATERIAL_ASSETS.is_none() {
            MATERIAL_ASSETS = Some(HashMap::new());
        }
        MATERIAL_ASSETS.as_mut().unwrap()
    }
}

fn get_material_bind_group_cache() -> &'static mut HashMap<String, wgpu::BindGroup> {
    unsafe {
        if MATERIAL_BIND_GROUP_CACHE.is_none() {
            MATERIAL_BIND_GROUP_CACHE = Some(HashMap::new());
        }
        MATERIAL_BIND_GROUP_CACHE.as_mut().unwrap()
    }
}

pub fn register_mesh_asset(name: String, mesh: Arc<Mesh>) {
    get_mesh_assets().insert(name, mesh);
}

pub fn register_material_asset(name: String, material: Arc<PbrMaterial>) {
    get_material_assets().insert(name, material);
}



pub fn post_process_asset_meshes(
    project_path: &std::path::Path,
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    _texture_manager: &mut TextureManager,
    _mesh_renderer: &MeshRenderer,
) {
    // [SCENE POST-PROCESSING] Load External Assets (XSG)
    // Iterate over all entities with MeshType::Asset and load the referenced XSG files
    // Post-process Asset meshes (Load XSG)
    // Find entities with MeshType::Asset
    let mut assets_to_load = Vec::new();
    for (entity, mesh) in world.meshes.iter() {
        if let ecs::MeshType::Asset(path) = &mesh.mesh_type {
            // Only load supported file formats (.xsg)
            // This prevents trying to load internal mesh IDs (like "Unnamed_0" or "sponza.xsg_mesh_Mesh_0_0") as files
            let path_lower = path.to_lowercase();
            if path_lower.ends_with(".xsg") {
                assets_to_load.push((*entity, path.clone()));
            }
        }
    }
    
    // Also check Model3D components
    for (entity, model) in world.model_3ds.iter() {
        let path = &model.asset_id;
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".xsg") {
            assets_to_load.push((*entity, path.clone()));
        }
    }

    // Load and attach
    use ecs::traits::ComponentAccess;

    for (parent_entity, asset_rel_path) in assets_to_load {
        // Check if entity already has children (already loaded)
        let has_children = world.parents.iter().any(|(_, &parent)| parent == parent_entity);
        if has_children {
            // Already loaded, skip
            continue;
        }

        let asset_path = project_path.join(&asset_rel_path);
        println!("DEBUG: Attempting to load asset from: {:?}", asset_path);

        if asset_path.exists() {
            println!("DEBUG: File exists! Loading...");
            
            // Check file extension to determine loader
            let extension = asset_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            let load_result = match extension.to_lowercase().as_str() {
                "xsg" => {
                    println!("DEBUG: Loading XSG file");
                    // Load XSG file
                    match crate::assets::xsg_importer::XsgImporter::load_from_file(&asset_path) {
                        Ok(xsg) => {
                            // Create a dummy texture manager for XSG loader
                            let mut dummy_texture_manager = crate::texture_manager::TextureManager::new();
                            let base_path = asset_path.parent().unwrap_or(std::path::Path::new("."));
                            
                            crate::assets::xsg_loader::XsgLoader::load_into_world(
                                &xsg,
                                world,
                                device,
                                queue,
                                &mut dummy_texture_manager,
                                &asset_rel_path,
                                base_path,
                            )
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to load XSG file: {}", e))
                    }
                }
                _ => {
                    Err(anyhow::anyhow!("Unsupported asset format: {}. Only XSG files are supported.", extension))
                }
            };
            
            match load_result {
                Ok(loaded_entities) => {
                    println!("DEBUG: Successfully loaded {} entities from asset", loaded_entities.len());
                    // Parent loaded entities to the container
                    for child in loaded_entities {
                        if world.parents.get(&child).is_none() {
                             world.set_parent(child, Some(parent_entity));
                        }
                    }

                    // NOTE: We keep the Mesh component with MeshType::Asset so it can be serialized
                    // and reloaded. The rendering system will skip it since the asset path won't
                    // match any mesh in the asset cache (child entities do the actual rendering).
                    // world.meshes.remove(&parent_entity);  // <-- DO NOT REMOVE
                    println!("DEBUG: Attached entities to parent {:?}", parent_entity);
                },
                Err(e) => {
                    log::error!("Failed to load Asset: {:?}", e);
                    println!("DEBUG: Failed to load Asset: {:?}", e);
                },
            }
        } else {
             log::error!("Asset not found: {:?}", asset_path);
             println!("DEBUG: Asset NOT FOUND at: {:?}", asset_path);
        }
    }
}

pub fn render_game_world<'a>(
    world: &World,
    batch_renderer: &'a mut BatchRenderer,
    mesh_renderer: &'a mut MeshRenderer,
    tilemap_renderer: &'a mut render::TilemapRenderer,
    camera_binding: &'a CameraBinding,
    light_binding: &'a LightBinding,
    texture_manager: &'a mut TextureManager,
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    screen_size: winit::dpi::PhysicalSize<u32>,
    render_pass: &mut wgpu::RenderPass<'a>,
) {
    // 0. Update Light (Simple directional light for now)
    // TODO: Find Light component in world
    // Default light at (2.0, 5.0, 2.0) with white color
    light_binding.update(queue, [2.0, 5.0, 2.0], [1.0, 1.0, 1.0], 1.0);

    // 1. Find the active Main Camera
    let mut main_camera = None;
    let mut camera_transform = None;

    for (entity, camera) in &world.cameras {
        // For now, just use the first camera found
        // TODO: Add enabled and is_main fields to Camera component
        if let Some(transform) = world.transforms.get(entity) {
            main_camera = Some(camera);
            camera_transform = Some(transform);
            break; 
        }
    }

    let view_proj = if let (Some(camera), Some(transform)) = (main_camera, camera_transform) {
        
        // Convert Euler angles (Degrees) to Radians
        let rot_rad = Vec3::new(
            transform.rotation[0].to_radians(),
            transform.rotation[1].to_radians(),
            transform.rotation[2].to_radians(),
        );
        
        // Reconstruct rotation quaternion (YXZ order: Yaw -> Pitch -> Roll)
        let cam_rotation = Quat::from_euler(glam::EulerRot::YXZ, rot_rad.y, rot_rad.x, rot_rad.z);
        let cam_pos = Vec3::from(transform.position);

        // Calculate Forward and Up vectors
        // NOTE: We assume +Z is Forward to match the Editor/Gizmo convention 
        // (where Camera at -Z looks at Origin).
        // Standard View Projection (RH) expects -Z forward, so `look_at_rh` handles the conversion.
        let forward = cam_rotation * Vec3::Z; // +Z Forward
        let up = cam_rotation * Vec3::Y;      // +Y Up

        let view = Mat4::look_at_rh(cam_pos, cam_pos + forward, up);
        
        let projection = match camera.projection {
             ecs::CameraProjection::Perspective => {
                let aspect = screen_size.width as f32 / screen_size.height.max(1) as f32;
                Mat4::perspective_rh(camera.fov.to_radians(), aspect, camera.near_clip, camera.far_clip)
             }
             ecs::CameraProjection::Orthographic => {
                 let half_height = camera.orthographic_size;
                 let aspect = screen_size.width as f32 / screen_size.height.max(1) as f32;
                 let half_width = half_height * aspect;
                 Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, camera.far_clip, camera.near_clip)
             }
        };

        projection * view
    } else {
        // Fallback default camera with Reverse-Z
        let projection = Mat4::orthographic_rh(-8.8, 8.8, -5.0, 5.0, 50.0, 0.1);
        Mat4::IDENTITY * projection
    };

    // Update Camera Uniform ONCE per frame
    batch_renderer.update_camera(queue, view_proj);

    // 2. Sort/Group Sprites by Texture
    // To minimize draw calls and state changes, we group sprites that share the same texture.
    // We store the data needed for drawing.
    struct DrawCommand {
        texture_id: String,
        pos: Vec3,
        rot: Quat,
        scale: Vec3,
        color: [f32; 4],
        rect: [u32; 4],
    }

    let mut commands: HashMap<String, Vec<DrawCommand>> = HashMap::new();

    for (entity, sprite) in &world.sprites {
        // TODO: Add visible field to Sprite component
        
        if let Some(transform) = world.transforms.get(entity) {
            let cmd = DrawCommand {
                texture_id: sprite.texture_id.clone(),
                pos: Vec3::new(transform.position[0], transform.position[1], transform.position[2]),
                rot: Quat::from_rotation_z(transform.rotation[2].to_radians()),
                scale: Vec3::new(transform.scale[0] * sprite.width, transform.scale[1] * sprite.height, 1.0),
                color: sprite.color,
                rect: sprite.sprite_rect.unwrap_or([0, 0, sprite.width as u32, sprite.height as u32]),
            };
            
            commands.entry(sprite.texture_id.clone()).or_default().push(cmd);
        }
    }

    // Ensure default textures exist before any immutable borrows
    let _ = texture_manager.get_white_texture(device, queue);
    let _ = texture_manager.get_normal_texture(device, queue);





    // 4. Render Meshes FIRST (Opaque, writes depth)
    // Collect and sort meshes by Z position (back to front for transparency, front to back for opaque)
    let mut mesh_entities: Vec<_> = world.meshes.iter().collect();
    
    // Sort by Z position (front to back for better depth testing)
    mesh_entities.sort_by(|a, b| {
        let z_a = world.transforms.get(a.0).map(|t| t.position[2]).unwrap_or(0.0);
        let z_b = world.transforms.get(b.0).map(|t| t.position[2]).unwrap_or(0.0);
        z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // 4. Render Meshes with proper GPU rendering
    // Pass 1: Ensure Procedural Meshes are in cache (Mutable access)
    let mesh_cache = get_mesh_cache();

    for (entity, ecs_mesh) in &mesh_entities {
         if world.transforms.contains_key(*entity) {
            match &ecs_mesh.mesh_type {
                ecs::MeshType::Asset(_) => {
                    // Assets are pre-loaded, no generation needed here
                },
                _ => {
                    let cache_key = format!("{:?}", ecs_mesh.mesh_type);
                    if !mesh_cache.contains_key(&cache_key) {
                        let generated_mesh = render::mesh_generation::generate_mesh(device, &ecs_mesh.mesh_type);
                        mesh_cache.insert(cache_key, generated_mesh);
                    }
                }
            }
         }
    }

    // Pass 2: Ensure Bind Groups exist (Mutable access)
    // We cannot render here because RenderPass needs immutable access to the binds, 
    // but creation needs mutable access to the cache.
    static mut ENTITY_CACHE: Option<HashMap<u32, (wgpu::Buffer, wgpu::BindGroup)>> = None;
    static mut ENTITY_MATERIAL_CACHE: Option<HashMap<u32, (wgpu::Buffer, wgpu::BindGroup)>> = None;

    let entity_cache = unsafe {
        if ENTITY_CACHE.is_none() {
            ENTITY_CACHE = Some(HashMap::new());
        }
        ENTITY_CACHE.as_mut().unwrap()
    };
    
    let material_cache = unsafe {
        if ENTITY_MATERIAL_CACHE.is_none() {
             ENTITY_MATERIAL_CACHE = Some(HashMap::new());
        }
        ENTITY_MATERIAL_CACHE.as_mut().unwrap()
    };



    // Pass 2: Ensure Bind Groups exist (Mutable access)
    // We cannot render here because RenderPass needs immutable access to the binds, 
    // but creation needs mutable access to the cache.
    
    // First, create material bind groups for assets that need them
    {
        let bind_group_cache = get_material_bind_group_cache();
        let material_assets = get_material_assets();
        for (_, ecs_mesh) in &mesh_entities {
            if let Some(mat_id) = &ecs_mesh.material_id {
                if let Some(mat_asset) = material_assets.get(mat_id) {
                    if mat_asset.bind_group.is_none() && !bind_group_cache.contains_key(mat_id) {
                        // println!("DEBUG: Creating BindGroup for Material Asset {}", mat_id);
                        let bind_group = mesh_renderer.create_pbr_bind_group(device, queue, mat_asset, texture_manager);
                        bind_group_cache.insert(mat_id.clone(), bind_group);
                    }
                }
            }
        }
    }
    
    for (entity, ecs_mesh) in &mesh_entities {
        if let Some(transform) = world.transforms.get(entity) {
             // 1. Object Uniform (Model Matrix)
            if !entity_cache.contains_key(entity) {
                let model_matrix = if let Some(global) = world.global_transforms.get(entity) {
                    Mat4::from_cols_array(&global.matrix)
                } else {
                    // Fallback to local transform
                    let rot_rad = Vec3::new(
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians(),
                    );
                    let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                    let translation = Vec3::from(transform.position);
                    let scale = Vec3::from(transform.scale);
                    
                    Mat4::from_scale_rotation_translation(scale, rotation, translation)
                };

                let object_uniform = ObjectUniform {
                    model: model_matrix.to_cols_array_2d(),
                };
                
                 let buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Object Uniform Buffer"),
                        contents: bytemuck::cast_slice(&[object_uniform]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    }
                );
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &mesh_renderer.object_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                    ],
                    label: Some("object_bind_group"),
                });
                entity_cache.insert(**entity, (buffer, bind_group));
            } else {
                // Update existing buffer
                if let Some((buffer, _)) = entity_cache.get(entity) {
                    let model_matrix = if let Some(global) = world.global_transforms.get(entity) {
                        Mat4::from_cols_array(&global.matrix)
                    } else {
                        // Fallback to local transform
                        let rot_rad = Vec3::new(
                            transform.rotation[0].to_radians(),
                            transform.rotation[1].to_radians(),
                            transform.rotation[2].to_radians(),
                        );
                        let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                        let translation = Vec3::from(transform.position);
                        let scale = Vec3::from(transform.scale);
                        
                        Mat4::from_scale_rotation_translation(scale, rotation, translation)
                    };

                    let object_uniform = ObjectUniform {
                        model: model_matrix.to_cols_array_2d(),
                    };
                     queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[object_uniform]));
                }
            }

             // 2. Material Uniform (PBR)
             // Check if custom material ID is present
             if let Some(mat_id) = &ecs_mesh.material_id {
                 // Asset-based materials are pre-created bind groups?
                 // Wait, PbrMaterial struct has `bind_group: Option<wgpu::BindGroup>`.
                 // If we have an asset material, we should use its bind group!
                 // See below in Render pass.
                 // We don't need to generate a dynamic buffer/bindgroup here if it's an asset.
                 // But for consistency we might.
                 // Actually, let's skip generating dynamic material for Asset materials.
             } else {
                 if !material_cache.contains_key(entity) {
                     let pbr_material = PbrMaterialUniform {
                        albedo_factor: ecs_mesh.color, 
                        metallic_factor: 0.0, 
                        roughness_factor: 0.5, 
                        padding: [0.0; 2],
                    };
                    
                    // We can safely unwrap because we initialized them above
                    let white = texture_manager.get_texture("default_white").unwrap();
                    let normal = texture_manager.get_texture("default_normal").unwrap();
                    
                     let buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("PBR Material Buffer"),
                            contents: bytemuck::cast_slice(&[pbr_material]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        }
                    );
                    
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &mesh_renderer.material_layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&white.view) },
                            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&white.sampler) },
                            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&normal.view) },
                            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&normal.sampler) },
                            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&white.view) },
                            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&white.sampler) },
                        ],
                        label: Some("pbr_material_bind_group"),
                    });
                    
                     material_cache.insert(**entity, (buffer, bind_group));
                 } else {
                     // Update existing material buffer (in case color changes)
                     if let Some((buffer, _)) = material_cache.get(entity) {
                          let pbr_material = PbrMaterialUniform {
                            albedo_factor: ecs_mesh.color, 
                            metallic_factor: 0.0, 
                            roughness_factor: 0.5, 
                            padding: [0.0; 2],
                        };
                        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[pbr_material]));
                     }
                 }
             }
        }
    }

    // Pass 3: Render (Immutable access)
    // Now caches are ready, we can borrow them fully for the render pass
    // Get asset caches
    let mesh_assets = get_mesh_assets();
    let material_assets = get_material_assets();

    for (entity, ecs_mesh) in mesh_entities {
         if world.transforms.contains_key(entity) {
            // Find Mesh
            let mesh_to_render = match &ecs_mesh.mesh_type {
                ecs::MeshType::Asset(id) => {
                    // Skip rendering parent Asset entities - their children will render the actual meshes
                    if id.ends_with(".xsg") {
                        None
                    } else {
                        // println!("DEBUG: Looking for mesh asset: {}", id);
                        if let Some(mesh) = mesh_assets.get(id) {
                            // println!("DEBUG: Found mesh asset: {}", id);
                            Some(mesh.as_ref())
                        } else {
                            // println!("DEBUG: Available mesh assets: {:?}", mesh_assets.keys().collect::<Vec<_>>());
                            None
                        }
                    }
                },
                _ => {
                    let cache_key = format!("{:?}", ecs_mesh.mesh_type);
                    mesh_cache.get(&cache_key)
                }
            };

            if let Some(mesh) = mesh_to_render {
                // Find Material Bind Group
                let material_bind_group = if let Some(mat_id) = &ecs_mesh.material_id {
                    let mat = material_assets.get(mat_id);
                     // If material loaded successfully, use it
                     if let Some(mat_asset) = mat {
                         if let Some(ref bind_group) = mat_asset.bind_group {
                             Some(bind_group)
                         } else {
                             // Get from cache (should exist from Pass 2)
                             get_material_bind_group_cache().get(mat_id)
                         }
                     } else {
                         println!("DEBUG: Material Asset {} NOT FOUND in cache!", mat_id);
                         None
                     }
                } else {
                     material_cache.get(entity).map(|(_, bg)| bg)
                };
                
                // Fallback to dynamic material if asset material missing/invalid
                let final_material_bg = material_bind_group.or_else(|| material_cache.get(entity).map(|(_, bg)| bg));

                if let (Some(material_bg), Some((_, object_bg))) = (final_material_bg, entity_cache.get(entity)) {
                     mesh_renderer.render_pbr(
                        render_pass,
                        mesh,
                        material_bg,
                        &camera_binding.bind_group,
                        &light_binding.bind_group,
                        object_bg,
                    );
                }
            }
         }
    }

    // 5. Render Model3D Components
    let model_manager = get_model_manager();
    
    static mut MODEL_NODE_CACHE: Option<HashMap<(u32, u32), (wgpu::Buffer, wgpu::BindGroup)>> = None;
    let model_node_cache = unsafe {
        if MODEL_NODE_CACHE.is_none() {
            MODEL_NODE_CACHE = Some(HashMap::new());
        }
        MODEL_NODE_CACHE.as_mut().unwrap()
    };
    
    // Pass A: Update Cache (Mutable access)
    // First, create material bind groups for Model3D assets that need them
    {
        let bind_group_cache = get_material_bind_group_cache();
        let material_assets = get_material_assets();
        for (_, model_3d) in &world.model_3ds {
            if let Some(xsg) = model_manager.get_model(&model_3d.asset_id) {
                for material in &xsg.materials {
                    let mat_id = format!("{}_mat_{}_{}", model_3d.asset_id, material.name, 0); // Simplified
                    if let Some(mat_asset) = material_assets.get(&mat_id) {
                        if mat_asset.bind_group.is_none() && !bind_group_cache.contains_key(&mat_id) {
                            let bind_group = mesh_renderer.create_pbr_bind_group(device, queue, mat_asset, texture_manager);
                            bind_group_cache.insert(mat_id, bind_group);
                        }
                    }
                }
            }
        }
    }
    
    for (entity, model_3d) in &world.model_3ds {
        if let Some(xsg) = model_manager.get_model(&model_3d.asset_id) {
             let root_transform = if let Some(global) = world.global_transforms.get(entity) {
                 Mat4::from_cols_array(&global.matrix)
             } else if let Some(transform) = world.transforms.get(entity) {
                  let rot_rad = Vec3::new(
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians(),
                  );
                  let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                  let translation = Vec3::from(transform.position);
                  let scale = Vec3::from(transform.scale);
                  Mat4::from_scale_rotation_translation(scale, rotation, translation)
             } else {
                  Mat4::IDENTITY
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
                 let local_mat = Mat4::from_scale_rotation_translation(s, q, t);
                 let global_mat = parent_mat * local_mat;

                 if let Some(_) = node.mesh {
                      // We only care about updating buffer here
                      let cache_key = (*entity, node_idx);
                      let object_uniform = ObjectUniform { model: global_mat.to_cols_array_2d() };
                      
                      if !model_node_cache.contains_key(&cache_key) {
                            let buffer = device.create_buffer_init(
                            &wgpu::util::BufferInitDescriptor {
                                label: Some(&format!("Model3D Node Buffer {}-{}", entity, node_idx)),
                                contents: bytemuck::cast_slice(&[object_uniform]),
                                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            }
                            );
                            
                            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &mesh_renderer.object_layout,
                                entries: &[
                                    wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                                ],
                                label: Some("model_node_bind_group"),
                            });
                            
                            model_node_cache.insert(cache_key, (buffer, bg));
                      } else {
                            let (buffer, _) = model_node_cache.get(&cache_key).unwrap();
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[object_uniform]));
                      }
                 }

                 for child_idx in &node.children {
                     stack.push((*child_idx, global_mat));
                 }
             }
        }
    }

    // Pass B: Render (Immutable access)
    // We traverse again to submit draw calls
    for (entity, model_3d) in &world.model_3ds {
        if let Some(xsg) = model_manager.get_model(&model_3d.asset_id) {
             let root_transform = if let Some(global) = world.global_transforms.get(entity) {
                 Mat4::from_cols_array(&global.matrix)
             } else if let Some(transform) = world.transforms.get(entity) {
                  let rot_rad = Vec3::new(
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians(),
                  );
                  let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                  let translation = Vec3::from(transform.position);
                  let scale = Vec3::from(transform.scale);
                  Mat4::from_scale_rotation_translation(scale, rotation, translation)
             } else {
                  Mat4::IDENTITY
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
                 let local_mat = Mat4::from_scale_rotation_translation(s, q, t);
                 let global_mat = parent_mat * local_mat;

                 if let Some(mesh_idx) = node.mesh {
                      let mesh_name = &xsg.meshes[mesh_idx as usize].name;
                      for (prim_idx, prim) in xsg.meshes[mesh_idx as usize].primitives.iter().enumerate() {
                           let mesh_id = format!("{}_mesh_{}_{}_{}", model_3d.asset_id, mesh_name, mesh_idx, prim_idx);
                           if let Some(mesh) = mesh_assets.get(&mesh_id) {
                                let mat_id = prim.material_index.and_then(|mi| {
                                     let mname = &xsg.materials[mi as usize].name;
                                     Some(format!("{}_mat_{}_{}", model_3d.asset_id, mname, mi))
                                });
                                let material_bg = mat_id.as_ref().and_then(|id| {
                                     if let Some(mat_asset) = material_assets.get(id) {
                                         if let Some(ref bind_group) = mat_asset.bind_group {
                                             Some(bind_group)
                                         } else {
                                             // Get from cache (should exist from Pass A)
                                             get_material_bind_group_cache().get(id)
                                         }
                                     } else {
                                         None
                                     }
                                 });

                                if let Some(mat_bg) = material_bg {
                                     let cache_key = (*entity, node_idx);
                                     if let Some((_, object_bg)) = model_node_cache.get(&cache_key) {
                                         mesh_renderer.render_pbr(
                                             render_pass,
                                             mesh,
                                             mat_bg,
                                             &camera_binding.bind_group,
                                             &light_binding.bind_group,
                                             object_bg
                                         );
                                     }
                                }
                           }
                      }
                 }

                 for child_idx in &node.children {
                     stack.push((*child_idx, global_mat));
                 }
             }
        }
    }



    // 6. Render Tilemaps (Transparent, background usually)
    // Add logic to render tilemaps
    // 6. Render Tilemaps (Transparent, background usually)
    // Add logic to render tilemaps
    static mut TILEMAP_CACHE: Option<HashMap<u32, TilemapGpuResources>> = None;
    let tilemap_cache = unsafe {
        if TILEMAP_CACHE.is_none() {
            TILEMAP_CACHE = Some(HashMap::new());
        }
        TILEMAP_CACHE.as_mut().unwrap()
    };

    // Pass A: Update Cache
    for (entity, tilemap) in &world.tilemaps {
        if world.transforms.contains_key(entity) {
             if !tilemap_cache.contains_key(entity) {
                // Generate GPU resources
                let tileset = world.tilesets.values().next(); // Hack: Just take first tileset
                
                if let Some(tileset) = tileset {
                    // We need the texture to be loaded
                    if let Some(texture) = texture_manager.get_texture(&tileset.texture_path) {
                        if let Some(resources) = tilemap_renderer.prepare_gpu_data(device, queue, tilemap, tileset, texture) {
                            tilemap_cache.insert(*entity, resources);
                        }
                    }
                }
            }
        }
    }

    // Pass B: Render
    for (entity, _tilemap) in &world.tilemaps {
        if world.transforms.contains_key(entity) {
            // Render
            if let Some(resources) = tilemap_cache.get(entity) {
                tilemap_renderer.render(
                    render_pass,
                    resources,
                    &camera_binding.bind_group
                );
            }
        }
    }


    // 7. Render Sprites (Transparent, Drawn LAST to blend correctly over meshes)
    // For now, just render the first batch to avoid borrowing issues
    // TODO: Implement proper multi-texture batching
    if let Some((texture_id, batch)) = commands.into_iter().next() {
        if let Some(texture) = texture_manager.get_texture(&texture_id) {
            // Clone texture data to avoid borrowing issues
            let tex_w = texture.width as f32;
            let tex_h = texture.height as f32;
            
            // Prepare sprite data
            batch_renderer.begin_frame(); // Clears instance buffer for this batch
            
            for cmd in batch {
                // Calculate UVs
                let u_min = cmd.rect[0] as f32 / tex_w;
                let v_min = cmd.rect[1] as f32 / tex_h;
                let u_scale = cmd.rect[2] as f32 / tex_w;
                let v_scale = cmd.rect[3] as f32 / tex_h;

                batch_renderer.draw_sprite(cmd.pos, cmd.rot, cmd.scale, cmd.color, [u_min, v_min], [u_scale, v_scale]);
            }

            // Draw sprites
            batch_renderer.end_frame(queue, render_pass, texture);
        }
    }
}
