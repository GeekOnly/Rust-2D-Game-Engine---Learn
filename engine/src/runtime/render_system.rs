use ecs::World;
use render::{BatchRenderer, MeshRenderer, TextureManager, CameraBinding, LightBinding, Mesh, PbrMaterialUniform, ObjectUniform, PbrMaterial};
use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

// Simple mesh cache to avoid regenerating meshes every frame
static mut MESH_CACHE: Option<HashMap<String, Mesh>> = None;
// Asset caches (Loaded from files)
static mut MESH_ASSETS: Option<HashMap<String, Arc<Mesh>>> = None;
static mut MATERIAL_ASSETS: Option<HashMap<String, Arc<PbrMaterial>>> = None;

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

pub fn register_mesh_asset(name: String, mesh: Arc<Mesh>) {
    get_mesh_assets().insert(name, mesh);
}

pub fn register_material_asset(name: String, material: Arc<PbrMaterial>) {
    get_material_assets().insert(name, material);
}

pub fn load_gltf_into_world(
    path: &std::path::Path,
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_manager: &mut TextureManager,
    mesh_renderer: &MeshRenderer,
) -> anyhow::Result<Vec<ecs::Entity>> {
     use crate::assets::gltf_loader::GltfLoader;
     
     let mut loader = GltfLoader::new();
     let loaded_meshes = loader.load_gltf(device, queue, texture_manager, mesh_renderer, path)?;
     let mut entities = Vec::new();

     for (i, loaded_mesh) in loaded_meshes.into_iter().enumerate() {
         // Register Mesh
         // Assuming name is unique or we make it unique
         let mesh_id = format!("{}_{}", loaded_mesh.name, i); 
         register_mesh_asset(mesh_id.clone(), loaded_mesh.mesh);
         
         // Register Material
         // Since loaded_mesh.material has bind group already
         let mat_id = format!("{}_mat", mesh_id);
         
         register_material_asset(mat_id.clone(), loaded_mesh.material);
         
         // Spawn Entity
         let entity = world.spawn();
         
         // Add Transform
         let (scale, rot, pos) = loaded_mesh.transform.to_scale_rotation_translation();
         let transform = ecs::Transform {
             position: pos.into(),
             rotation: rot.to_euler(glam::EulerRot::XYZ).into(), // Check Euler order match ECS
             scale: scale.into(),
         };
         // Note: ECS Transform Rotation is stored in euler degrees?
         // ECS Transform: `pub rotation: [f32; 3]` (Euler angles in degrees)
         // `rot.to_euler` returns radians.
         let rot_euler = rot.to_euler(glam::EulerRot::XYZ);
         let rot_deg = [rot_euler.0.to_degrees(), rot_euler.1.to_degrees(), rot_euler.2.to_degrees()];
          
         // We need ComponentAccess to insert.
         // But `World` (CustomWorld) exposes `transforms` map directly.
         // world.transforms.insert(entity, ..);
         // However, ECS `World` might be `HecsMinimal` or `CustomWorld`.
         // `render_system.rs` uses `ecs::World` type alias.
         // If `hecs` feature is off, `World` is `CustomWorld`.
         // `CustomWorld` has public fields.
         // If `hecs` is on, we need trait.
         // `ecs/src/lib.rs` says: `pub use backends::hecs_minimal::HecsMinimal as World;` or `CustomWorld as World`.
         // We should use `ComponentAccess` trait to be safe/generic.
         use ecs::traits::ComponentAccess;
         
         let _ = ComponentAccess::<ecs::Transform>::insert(world, entity, ecs::Transform {
             position: pos.into(),
             rotation: rot_deg,
             scale: scale.into(),
         });
         
         let _ = ComponentAccess::<ecs::Mesh>::insert(world, entity, ecs::Mesh {
             mesh_type: ecs::MeshType::Asset(mesh_id),
             color: [1.0, 1.0, 1.0, 1.0], // Driven by material
             material_id: Some(mat_id),
         });
         
         entities.push(entity);
     }
     
     Ok(entities)
}

pub fn post_process_asset_meshes(
    project_path: &std::path::Path,
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_manager: &mut TextureManager,
    mesh_renderer: &MeshRenderer,
) {
    // [SCENE POST-PROCESSING] Load External Assets (GLTF)
    // Iterate over all entities with MeshType::Asset and load the referenced files
    // Post-process Asset meshes (Load GLTF)
    // Find entities with MeshType::Asset
    let mut assets_to_load = Vec::new();
    for (entity, mesh) in world.meshes.iter() {
        if let ecs::MeshType::Asset(path) = &mesh.mesh_type {
            // Only load if it looks like a file path (has extension)
            // This prevents trying to load internal mesh IDs (like "Unnamed_0") as files
            if std::path::Path::new(path).extension().is_some() {
                assets_to_load.push((*entity, path.clone()));
                // println!("DEBUG: Found Asset Mesh for entity {:?}: {}", entity, path);
            }
        }
    }

    // Load and attach
    use ecs::traits::ComponentAccess;

    for (parent_entity, asset_rel_path) in assets_to_load {
        // First, remove any existing children to prevent duplicates on reload
        let existing_children: Vec<_> = world.parents.iter()
            .filter(|(_, &parent)| parent == parent_entity)
            .map(|(child, _)| *child)
            .collect();

        for child in existing_children {
            println!("DEBUG: Removing existing child {:?} before reloading", child);
            world.despawn(child);
        }

        let asset_path = project_path.join(&asset_rel_path);
        println!("DEBUG: Attempting to load GLTF from: {:?}", asset_path);

        if asset_path.exists() {
            println!("DEBUG: File exists! Loading...");
            match load_gltf_into_world(
                &asset_path,
                world,
                device,
                queue,
                texture_manager,
                mesh_renderer,
            ) {
                Ok(loaded_entities) => {
                    println!("DEBUG: Successfully loaded {} entities from GLTF", loaded_entities.len());
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
                    log::error!("Failed to load GLTF Asset: {:?}", e);
                    println!("DEBUG: Failed to load GLTF Asset: {:?}", e);
                },
            }
        } else {
             log::error!("GLTF Asset not found: {:?}", asset_path);
             println!("DEBUG: GLTF Asset NOT FOUND at: {:?}", asset_path);
        }
    }
}

pub fn render_game_world<'a>(
    world: &World,
    batch_renderer: &'a mut BatchRenderer,
    mesh_renderer: &'a mut MeshRenderer,
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

    // 3. Render Batches
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

    // 4. Render Meshes with Depth Sorting
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
                ecs::MeshType::Asset(id) => mesh_assets.get(id).map(|m| m.as_ref()),
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
                         if mat_asset.bind_group.is_none() {
                             println!("DEBUG: Material Asset {} found but has NO BindGroup!", mat_id);
                         }
                         mat_asset.bind_group.as_ref()
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
}
