use ecs::World;
use render::{BatchRenderer, MeshRenderer, TilemapRenderer, TextureManager, CameraBinding, LightBinding, Mesh, PbrMaterialUniform, ObjectUniform, PbrMaterial};
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

// Tilemap Cache: Entity -> (Vertex Buffer, Index Buffer, Index Count)
static mut TILEMAP_CACHE: Option<HashMap<ecs::Entity, (wgpu::Buffer, wgpu::Buffer, u32)>> = None;

fn get_tilemap_cache() -> &'static mut HashMap<ecs::Entity, (wgpu::Buffer, wgpu::Buffer, u32)> {
    unsafe {
        if TILEMAP_CACHE.is_none() {
            TILEMAP_CACHE = Some(HashMap::new());
        }
        TILEMAP_CACHE.as_mut().unwrap()
    }
}

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
    asset_loader: &dyn engine_core::assets::AssetLoader,
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
        let asset_path_str = asset_path.to_str().unwrap_or("");
        println!("DEBUG: Attempting to load asset from: {:?}", asset_path);

        // Ideally we should check if asset exists using asset_loader, but it doesn't have `exists()`.
        // We just try to load.
            
        // Check file extension to determine loader
        // Simple check on string
        let extension = asset_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let load_result = match extension.to_lowercase().as_str() {
            "xsg" => {
                println!("DEBUG: Loading XSG file");
                // Load XSG file via AssetLoader
                match crate::assets::xsg_importer::XsgImporter::load_from_asset(asset_loader, asset_path_str) {
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
                            asset_loader
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
    }
}

pub fn render_game_world<'a>(
    world: &'a World,
    tilemap_renderer: &'a TilemapRenderer,
    batch_renderer: &'a mut BatchRenderer,
    mesh_renderer: &'a mut MeshRenderer,
    camera_binding: &'a CameraBinding,
    light_binding: &'a LightBinding,
    texture_manager: &'a mut TextureManager,
    queue: &wgpu::Queue,
    device: &wgpu::Device,
    _screen_size: winit::dpi::PhysicalSize<u32>, // Unused now that projection is passed in
    render_pass: &mut wgpu::RenderPass<'a>,
    view_proj: Mat4, // <--- Added Argument
) {
    // 0. Update Light (Simple directional light for now)
    // TODO: Find Light component in world
    // Default light at (2.0, 5.0, 2.0) with white color
    light_binding.update(queue, [2.0, 5.0, 2.0], [1.0, 1.0, 1.0], 1.0);

    // Ensure default textures exist before any immutable borrows (Fixes E0502)
    let _ = texture_manager.get_white_texture(device, queue);
    let _ = texture_manager.get_normal_texture(device, queue);

    // 1. Update Camera Uniform for Sprites
    // REMOVED: batch_renderer.update_camera(queue, view_proj);
    // Reason: Buffer reuse race-condition. We use the updated CameraBinding passed via logic instead.

    // Debug: Print view_proj matrix once
    static mut VIEWPROJ_LOGGED: bool = false;
    unsafe {
        if !VIEWPROJ_LOGGED {
            println!("DEBUG: view_proj matrix = {:?}", view_proj);
            VIEWPROJ_LOGGED = true;
        }
    }

    // ------------------------------------------------------------------------
    // 0. Render Tilemaps (Background)
    // ------------------------------------------------------------------------
    let tilemap_cache = get_tilemap_cache();
    
    // Pass 1: Ensure geometry is cached
    for (entity, tilemap) in &world.tilemaps {
        if !tilemap.visible {
            continue;
        }

        if !tilemap_cache.contains_key(entity) {
             // Find corresponding Tileset to generate mesh
            let tileset = world.tilesets.values().find(|ts| ts.texture_id == tilemap.tileset_id);
            if let Some(tileset) = tileset {
                // Get Transform for offset (default to Zero if missing)
                 let pos = if let Some(transform) = world.transforms.get(entity) {
                    glam::Vec3::from(transform.position)
                } else {
                    glam::Vec3::ZERO
                };
                
                // Prepare Mesh (Geometry) with Scale and Offset
                // Default pixels_per_unit = 8.0 (1 tile = 1 unit)
                let mesh_data = tilemap_renderer.prepare_mesh(device, tilemap, tileset, pos, 8.0);
                tilemap_cache.insert(*entity, mesh_data);
            }
        }
    }

    // Pass 2: Render
    for (entity, tilemap) in &world.tilemaps {
        if !tilemap.visible {
            continue;
        }

        if let Some((vertex_buffer, index_buffer, index_count)) = tilemap_cache.get(entity) {
            // Find tileset to get texture
            let tileset = world.tilesets.values().find(|ts| ts.texture_id == tilemap.tileset_id);
            if let Some(tileset) = tileset {
                if let Some(texture) = texture_manager.get_texture(&tileset.texture_path) {
                     tilemap_renderer.render(
                        render_pass,
                        vertex_buffer,
                        index_buffer,
                        *index_count,
                        texture,
                        &camera_binding.bind_group
                    );
                } else {
                    static mut TEX_FAIL_LOGGED: bool = false;
                    unsafe {
                        if !TEX_FAIL_LOGGED {
                            println!("DEBUG: Rendering FAILED for tilemap {:?}. Texture '{}' not found in TextureManager. Make sure it is loaded.", entity, tileset.texture_path);
                            TEX_FAIL_LOGGED = true;
                        }
                    }
                }
            } else {
                static mut TS_FAIL_LOGGED: bool = false;
                unsafe {
                    if !TS_FAIL_LOGGED {
                        println!("DEBUG: Rendering FAILED for tilemap {:?}. Tileset '{}' not found in World.", entity, tilemap.tileset_id);
                        TS_FAIL_LOGGED = true;
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // 2. Sort/Group Sprites by Texture and Unity-style Sorting
    // ------------------------------------------------------------------------
    // To minimize draw calls and state changes, we group sprites that share the same texture.

    // Start by clearing transient buffers from previous frame
    batch_renderer.start_frame();

    // Debug: Count sprites (commented out - too spammy)
    // let sprite_count = world.sprites.len();
    // if sprite_count > 0 {
    //     println!("DEBUG: Rendering {} sprites", sprite_count);
    // }

    struct SpriteInfo<'a> {
        _entity: ecs::Entity,
        sprite: &'a ecs::Sprite,
        transform: &'a ecs::Transform,
    }

    let mut visible_sprites = Vec::new();

    for (entity, sprite) in &world.sprites {
        // TODO: Add visible field to Sprite component
        if let Some(transform) = world.transforms.get(entity) {
             visible_sprites.push(SpriteInfo {
                 _entity: *entity,
                 sprite,
                 transform,
             });
        }
    }
    
    // Sort logic: Sorting Layer -> Order in Layer -> Z Depth (Back to Front)
    visible_sprites.sort_by(|a, b| {
        // 1. Sorting Layer (String comparison for now)
        let layer_cmp = a.sprite.sorting_layer.cmp(&b.sprite.sorting_layer);
        if layer_cmp != std::cmp::Ordering::Equal { return layer_cmp; }
        
        // 2. Order in Layer (Higher order = On top)
        let order_cmp = a.sprite.order_in_layer.cmp(&b.sprite.order_in_layer);
        if order_cmp != std::cmp::Ordering::Equal { return order_cmp; }
        
        // 3. Z-Depth (Back to Front) aka Painter's Algorithm
        // For standard 2D with -Z camera, Back (-Z) to Front (+Z)?
        // Wait, if Camera is at +Z looking at -Z.
        // Far (-100) -> Near (0).
        // Standard sort is Ascending. (-100, -99, ...).
        // Draw -100 first, then -99. 
        // So Ascending Z is correct if "Lower is Farther".
        a.transform.position[2].partial_cmp(&b.transform.position[2]).unwrap_or(std::cmp::Ordering::Equal)
    });



    // 3. Prepare Batches
    let mut current_texture_id = String::new();
    
    batch_renderer.begin_frame(); 
    
    for info in visible_sprites {
        // Check for texture change
        if info.sprite.texture_id != current_texture_id {
            if !current_texture_id.is_empty() {
                // Finish previous batch
                batch_renderer.finish_batch(device, current_texture_id.clone());
            }
            current_texture_id = info.sprite.texture_id.clone();
        }
        
        if let Some(texture) = texture_manager.get_texture(&info.sprite.texture_id) {
            let tex_w = texture.width as f32;
            let tex_h = texture.height as f32;

            // Draw Sprite
            let sprite = info.sprite;
            let transform = info.transform;

            // Debug: Print first sprite being rendered
            static mut FIRST_SPRITE_LOGGED: bool = false;
            unsafe {
                if !FIRST_SPRITE_LOGGED {
                    println!("DEBUG: First sprite - pos: {:?}, texture: {}", transform.position, sprite.texture_id);
                    FIRST_SPRITE_LOGGED = true;
                }
            }
            
            let rect = sprite.sprite_rect.unwrap_or([0, 0, sprite.width as u32, sprite.height as u32]);
            let u_min = rect[0] as f32 / tex_w;
            let v_min = rect[1] as f32 / tex_h;
            let u_scale = rect[2] as f32 / tex_w;
            let v_scale = rect[3] as f32 / tex_h;
            
            let pos = Vec3::new(transform.position[0], transform.position[1], transform.position[2]);
            
            // [Fix] Handle 3D Rotation and Billboarding
            // If billboard is true, we keep original 2D behavior (Z-rotation only, facing Z plane)
            // If billboard is false, we apply full 3D rotation so it can be placed as a floor/wall in 3D
            let rot = if sprite.billboard {
                 Quat::from_rotation_z(transform.rotation[2].to_radians())
            } else {
                 let rot_rad = Vec3::new(
                    transform.rotation[0].to_radians(),
                    transform.rotation[1].to_radians(),
                    transform.rotation[2].to_radians(),
                );
                Quat::from_euler(glam::EulerRot::XYZ, rot_rad.x, rot_rad.y, rot_rad.z)
            };

            // Convert pixel size to world units using pixels_per_unit
            let world_width = sprite.width / sprite.pixels_per_unit;
            let world_height = sprite.height / sprite.pixels_per_unit;
            let scale = Vec3::new(transform.scale[0] * world_width, transform.scale[1] * world_height, 1.0);

            batch_renderer.draw_sprite(pos, rot, scale, sprite.color, [u_min, v_min], [u_scale, v_scale]);
        }
    }
    
    // Flush final batch
    if !current_texture_id.is_empty() {
         batch_renderer.finish_batch(device, current_texture_id);
    }

    // 4. Render All Batches
    // Pass external camera binding (Scene Camera or Game Camera)
    batch_renderer.render(render_pass, texture_manager, &camera_binding.bind_group);

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
}
