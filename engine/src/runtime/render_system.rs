// Force update
use ecs::World;
use render::{BatchRenderer, MeshRenderer, TilemapRenderer, TextureManager, CameraBinding, LightBinding, Mesh, PbrMaterial};
use glam::{Vec3, Mat4, Quat};
use std::collections::HashMap;
use std::sync::Arc;
use crate::assets::model_manager::get_model_manager;
use crate::runtime::extraction_system::ExtractionSystem;
use anyhow;

// Simple mesh cache to avoid regenerating meshes every frame
// Render Cache Struct to replace static mut (Global State)
pub struct RenderCache {
    pub mesh_cache: HashMap<String, Mesh>,
    pub mesh_assets: HashMap<String, Arc<Mesh>>,
    pub material_assets: HashMap<String, Arc<PbrMaterial>>,
    pub material_bind_group_cache: HashMap<String, wgpu::BindGroup>,
    
    // Tilemap Cache: Entity -> (Vertex Buffer, Index Buffer, Index Count)
    pub tilemap_cache: HashMap<ecs::Entity, (wgpu::Buffer, wgpu::Buffer, u32)>,
    
    // Entity Object Uniform Cache: Entity ID -> (Buffer, BindGroup)
    pub entity_cache: HashMap<u32, (wgpu::Buffer, wgpu::BindGroup)>,
    
    // Entity Material Uniform Cache: Entity ID -> (Buffer, BindGroup)
    pub entity_material_cache: HashMap<u32, (wgpu::Buffer, wgpu::BindGroup)>,
    
    // Model3D Node Cache: (Entity ID, Node Index) -> (Buffer, BindGroup)
    pub model_node_cache: HashMap<(u32, u32), (wgpu::Buffer, wgpu::BindGroup)>,

    // Instance Buffer Cache: BatchKey(String) -> Buffer
    pub instance_buffers: HashMap<String, wgpu::Buffer>,
}

impl RenderCache {
    pub fn new() -> Self {
        Self {
            mesh_cache: HashMap::new(),
            mesh_assets: HashMap::new(),
            material_assets: HashMap::new(),
            material_bind_group_cache: HashMap::new(),
            tilemap_cache: HashMap::new(),
            entity_cache: HashMap::new(),
            entity_material_cache: HashMap::new(),
            model_node_cache: HashMap::new(),
            instance_buffers: HashMap::new(),
        }
    }
}

pub fn register_mesh_asset(render_cache: &mut RenderCache, name: String, mesh: Arc<Mesh>) {
    render_cache.mesh_assets.insert(name, mesh);
}

pub fn register_material_asset(render_cache: &mut RenderCache, name: String, material: Arc<PbrMaterial>) {
    render_cache.material_assets.insert(name, material);
}



pub fn post_process_asset_meshes(
    render_cache: &mut RenderCache,
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
    // use ecs::traits::ComponentAccess;

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
                            render_cache,
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
    render_cache: &'a mut RenderCache,
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
    // 0. Extract Frame Data
    let frame = ExtractionSystem::extract(world, 0.0);

    // 1. Update Light
    if let Some(light) = frame.lights.first() {
        // Use position from uniform (x, y, z) - w is separate
        let pos = [light.position[0], light.position[1], light.position[2]];
        let color = [light.color[0], light.color[1], light.color[2]];
        let intensity = light.color[3]; // We packed intensity into alpha
        light_binding.update(queue, pos, color, intensity);
    } else {
        // Default light if none exists
        light_binding.update(queue, [2.0, 5.0, 2.0], [1.0, 1.0, 1.0], 1.0);
    }

    // Ensure default textures exist before any immutable borrows (Fixes E0502)
    let _ = texture_manager.get_white_texture(device, queue);
    let _ = texture_manager.get_normal_texture(device, queue);

    // 1. Update Camera Uniform for Sprites
    // REMOVED: batch_renderer.update_camera(queue, view_proj);
    // Reason: Buffer reuse race-condition. We use the updated CameraBinding passed via logic instead.



    // ------------------------------------------------------------------------
    // 0. Render Tilemaps (Background)
    // ------------------------------------------------------------------------
    
    // Pass 1: Ensure geometry is cached
    for (entity, tilemap) in &world.tilemaps {
        if !tilemap.visible {
            continue;
        }

        if !render_cache.tilemap_cache.contains_key(entity) {
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
                render_cache.tilemap_cache.insert(*entity, mesh_data);
            }
        }
    }

    // ------------------------------------------------------------------------
    // PREPARATION PHASE: Update all Caches (Mutable Access)
    // ------------------------------------------------------------------------
    
    // 1. Prepare/Sort Meshes
    // Collect and sort meshes by Z position (back to front for transparency, front to back for opaque)
    let mut mesh_entities: Vec<_> = world.meshes.iter().collect();
    
    // Sort by Z position (front to back for better depth testing)
    mesh_entities.sort_by(|a, b| {
        let z_a = world.transforms.get(a.0).map(|t| t.position[2]).unwrap_or(0.0);
        let z_b = world.transforms.get(b.0).map(|t| t.position[2]).unwrap_or(0.0);
        z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // 2. Update Procedural Mesh Cache
    for (entity, ecs_mesh) in &mesh_entities {
         if world.transforms.contains_key(*entity) {
            match &ecs_mesh.mesh_type {
                ecs::MeshType::Asset(_) => {
                    // Assets are pre-loaded, no generation needed here
                },
                _ => {
                    let cache_key = format!("{:?}", ecs_mesh.mesh_type);
                    if !render_cache.mesh_cache.contains_key(&cache_key) {
                        let generated_mesh = render::mesh_generation::generate_mesh(device, &ecs_mesh.mesh_type);
                        render_cache.mesh_cache.insert(cache_key, generated_mesh);
                    }
                }
            }
         }
    }

    // 3. Update Entity & Material Caches (Meshes) - Only Material Buffers now
    // Create material bind groups for assets that need them
    for (_, ecs_mesh) in &mesh_entities {
            if let Some(mat_id) = &ecs_mesh.material_id {
                if let Some(mat_asset) = render_cache.material_assets.get(mat_id) {
                    if mat_asset.bind_group.is_none() && !render_cache.material_bind_group_cache.contains_key(mat_id) {
                        let bind_group = mesh_renderer.create_pbr_bind_group(device, queue, mat_asset, texture_manager);
                        render_cache.material_bind_group_cache.insert(mat_id.clone(), bind_group);
                    }
                }
            }
    }
    



    
    // 4. Update Model3D Cache (Materials & Nodes)
    let model_manager = get_model_manager();
    
    for (_, model_3d) in &world.model_3ds {
        if let Some(xsg) = model_manager.get_model(&model_3d.asset_id) {
            for material in &xsg.materials {
                let mat_id = format!("{}_mat_{}_{}", model_3d.asset_id, material.name, 0);
                if let Some(mat_asset) = render_cache.material_assets.get(&mat_id) {
                    if mat_asset.bind_group.is_none() && !render_cache.material_bind_group_cache.contains_key(&mat_id) {
                        let bind_group = mesh_renderer.create_pbr_bind_group(device, queue, mat_asset, texture_manager);
                        render_cache.material_bind_group_cache.insert(mat_id, bind_group);
                    }
                }
            }
        }
    }


    // ------------------------------------------------------------------------
    // RENDER PHASE (Immutable Access)
    // ------------------------------------------------------------------------

    // Pass 2: Render
    for (entity, tilemap) in &world.tilemaps {
        if !tilemap.visible {
            continue;
        }

        if let Some((vertex_buffer, index_buffer, index_count)) = render_cache.tilemap_cache.get(entity) {
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





    // Pass 2: Ensure Bind Groups exist (Mutable access)
    // We cannot render here because RenderPass needs immutable access to the binds, 
    // but creation needs mutable access to the cache.
    // ENTITY_CACHE and ENTITY_MATERIAL_CACHE are now in render_cache





    // 5. EXTRACT & RENDER (Instanced)
    // Frame extracted at step 0
    // let frame = ExtractionSystem::extract(world, 0.0);

    // Ensure 'default' material exists for untextured meshes
    if !render_cache.material_bind_group_cache.contains_key("default") {
        let default_material = render::PbrMaterial {
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            albedo_factor: [1.0; 4],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            bind_group: None,
            // Assuming these might exist or derived default. 
            // If explicit construction fails due to missing fields, 
            // we might need ..Default::default() if PbrMaterial implements it, 
            // but it has Option fields so maybe not fully default?
            // Let's try to include known fields.
            // If PbrMaterial is defined in render crate, I assume I can construct it.
        };
        // We wrap in Arc to match expected API if needed, but create_pbr_bind_group takes &PbrMaterial
        let bg = mesh_renderer.create_pbr_bind_group(device, queue, &default_material, texture_manager);
        render_cache.material_bind_group_cache.insert("default".to_string(), bg);
    }

    // Opaque Batches (Instancing)
    // PRE-PASS: Prepare Instance Buffers
    for (key, instances) in &frame.opaque_batches {
         if instances.is_empty() { continue; }
         
         let cache_key = format!("{}_{}", key.mesh_id, key.material_id);
         let required_size = (instances.len() * std::mem::size_of::<render::MeshInstance>()) as u64;
         
         let buffer_valid = if let Some(buffer) = render_cache.instance_buffers.get(&cache_key) {
             buffer.size() >= required_size
         } else {
             false
         };

         if !buffer_valid {
              let new_buffer = mesh_renderer.create_instance_buffer(device, instances);
              render_cache.instance_buffers.insert(cache_key, new_buffer);
         } else {
              let buffer = render_cache.instance_buffers.get(&cache_key).unwrap();
              queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
         }
    }

    // RENDER PASS
    for (key, instances) in &frame.opaque_batches {
         let mesh_id = &key.mesh_id;
         let mat_id = &key.material_id;
         
         // Resolve Mesh
         let mesh_opt = if render_cache.mesh_assets.contains_key(mesh_id.as_str()) {
             render_cache.mesh_assets.get(mesh_id.as_str()).map(|m| m.as_ref())
         } else {
             render_cache.mesh_cache.get(mesh_id.as_str())
         };

         if let Some(mesh) = mesh_opt {
              // Resolve Material Bind Group
              // Try asset map first, then cache
              let material_bg_opt = if let Some(mat_asset) = render_cache.material_assets.get(mat_id.as_str()) {
                   mat_asset.bind_group.as_ref().or_else(|| render_cache.material_bind_group_cache.get(mat_id.as_str()))
              } else {
                   // Fallback for "default" or other manual keys
                   render_cache.material_bind_group_cache.get(mat_id.as_str())
              };

              if let Some(material_bg) = material_bg_opt {
                  if instances.is_empty() { continue; }
                  
                  let cache_key = format!("{}_{}", mesh_id, mat_id);
                  if let Some(instance_buffer) = render_cache.instance_buffers.get(&cache_key) {
                        mesh_renderer.render_instanced(
                            render_pass,
                            mesh,
                            material_bg,
                            &camera_binding.bind_group,
                            &light_binding.bind_group,
                            instance_buffer,
                            instances.len() as u32
                        );
                  }
              }
         }
    }
}
