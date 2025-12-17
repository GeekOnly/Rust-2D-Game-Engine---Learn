use ecs::World;
use render::{BatchRenderer, MeshRenderer, TextureManager, CameraBinding, LightBinding, Mesh, PbrMaterialUniform, ObjectUniform};
use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

// Simple mesh cache to avoid regenerating meshes every frame
static mut MESH_CACHE: Option<HashMap<String, Mesh>> = None;
static mut MATERIAL_CACHE: Option<wgpu::BindGroup> = None;

fn get_mesh_cache() -> &'static mut HashMap<String, Mesh> {
    unsafe {
        if MESH_CACHE.is_none() {
            MESH_CACHE = Some(HashMap::new());
        }
        MESH_CACHE.as_mut().unwrap()
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
    _screen_size: winit::dpi::PhysicalSize<u32>,
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
        
        let rot_rad = Vec3::new(
            transform.rotation[0].to_radians(),
            transform.rotation[1].to_radians(),
            transform.rotation[2].to_radians(),
        );
        let cam_rotation = Quat::from_euler(glam::EulerRot::YXZ, rot_rad.y, rot_rad.x, rot_rad.z);
        let cam_translation = Vec3::from(transform.position);

        let view = Mat4::from_rotation_translation(cam_rotation, cam_translation).inverse();
        
        let projection = match camera.projection {
             ecs::CameraProjection::Perspective => {
                let aspect = 16.0 / 9.0; // TODO: Pass actual screen size
                Mat4::perspective_rh(camera.fov.to_radians(), aspect, camera.near_clip, camera.far_clip)
             }
             ecs::CameraProjection::Orthographic => {
                 let half_height = camera.orthographic_size;
                 let aspect = 16.0 / 9.0;
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
    // Pass 1: Ensure all meshes are in cache (Mutable access)
    let mesh_cache = get_mesh_cache();

    for (entity, ecs_mesh) in &mesh_entities {
         if world.transforms.contains_key(*entity) {
            let cache_key = format!("{:?}", ecs_mesh.mesh_type);
            if !mesh_cache.contains_key(&cache_key) {
                let generated_mesh = render::mesh_generation::generate_mesh(device, &ecs_mesh.mesh_type);
                mesh_cache.insert(cache_key, generated_mesh);
            }
         }
    }

    // Pass 2: Render (Immutable access)
    // We need to create bind groups for each entity
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
             let cache_key = format!("{:?}", ecs_mesh.mesh_type);
             if mesh_cache.contains_key(&cache_key) {
                 // 1. Object Uniform (Model Matrix)
                if !entity_cache.contains_key(entity) {
                    let rot_rad = Vec3::new(
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians(),
                    );
                    let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                    let translation = Vec3::from(transform.position);
                    let scale = Vec3::from(transform.scale);
                    
                    let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, translation);
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
                        let rot_rad = Vec3::new(
                            transform.rotation[0].to_radians(),
                            transform.rotation[1].to_radians(),
                            transform.rotation[2].to_radians(),
                        );
                        let rotation = Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z);
                        let translation = Vec3::from(transform.position);
                        let scale = Vec3::from(transform.scale);
                        
                        let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, translation);
                        let object_uniform = ObjectUniform {
                            model: model_matrix.to_cols_array_2d(),
                        };
                         queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[object_uniform]));
                    }
                }

                 // 2. Material Uniform (PBR)
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
    for (entity, ecs_mesh) in mesh_entities {
         if world.transforms.contains_key(entity) {
            let cache_key = format!("{:?}", ecs_mesh.mesh_type);
            if let Some(mesh) = mesh_cache.get(&cache_key) {
                if let (Some((_, object_bg)), Some((_, material_bg))) = (entity_cache.get(entity), material_cache.get(entity)) {
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
