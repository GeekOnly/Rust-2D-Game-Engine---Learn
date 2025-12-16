use ecs::World;
use render::{BatchRenderer, MeshRenderer, TextureManager, CameraBinding, LightBinding};
use glam::{Vec3, Quat, Mat4};
use std::collections::HashMap;

pub fn render_game_world<'a>(
    world: &World,
    batch_renderer: &'a mut BatchRenderer,
    _mesh_renderer: &mut MeshRenderer,
    _camera_binding: &CameraBinding,
    _light_binding: &LightBinding,
    texture_manager: &'a mut TextureManager,
    queue: &wgpu::Queue,
    _device: &wgpu::Device,
    _screen_size: winit::dpi::PhysicalSize<u32>,
    render_pass: &mut wgpu::RenderPass<'a>,
) {
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

    // Camera View-Projection Matrix
    let view_proj = if let (Some(_camera), Some(transform)) = (main_camera, camera_transform) {
        // Simple Orthographic setup for 2D
        // TODO: Use actual Camera component properties (projection size, fov)
        let half_height = 5.0; // Fixed size for now (Zoom level)
        // We'd need accurate aspect ratio here. Passing 16/9 generic for now, 
        // real implementation should pass actual surface size into this function.
        let aspect = 1.777; 
        let half_width = half_height * aspect;
        
        let projection = Mat4::orthographic_rh(
            -half_width, half_width, 
            -half_height, half_height, 
            -100.0, 100.0 // Near/Far
        );
        
        let eye = Vec3::new(transform.position[0], transform.position[1], 10.0); // Z=10 for camera
        let target = Vec3::new(transform.position[0], transform.position[1], 0.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        let view = Mat4::look_at_rh(eye, target, up);
        
        projection * view
    } else {
        // Fallback default camera
        let projection = Mat4::orthographic_rh(-8.8, 8.8, -5.0, 5.0, -100.0, 100.0);
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

    // 4. Render Meshes
    // Note: Camera and light uniforms should be updated by the caller before calling this function
    // since the bindings are immutable references here

    // TODO: Implement proper mesh rendering
    // For now, skip mesh rendering until proper mesh loading is implemented
    // The ecs::Mesh component needs to be converted to render::Mesh with GPU buffers
    for (_entity, _mesh) in &world.meshes {
        // Skip mesh rendering for now
        // TODO: Load or create render::Mesh from ecs::Mesh and render it
    }
}
