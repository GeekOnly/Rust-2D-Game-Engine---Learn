use anyhow::Result;
use engine_core::EngineContext;
use ecs::World;
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;
use render::RenderModule;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// Use the engine library
use engine::runtime;
use engine::texture_manager::TextureManager;
use engine::ui_manager::UIManager;
use ecs::traits::ComponentAccess;
use ecs::{Transform, Camera, Mesh};

fn map_winit_keycode(keycode: winit::keyboard::KeyCode) -> Option<input::Key> {
    use winit::keyboard::KeyCode;
    use input::Key;
    match keycode {
        KeyCode::Escape => Some(Key::Escape),
        _ => None,
    }
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("=== Stress Test Starting ===");

    let event_loop = EventLoop::new()?;
    let window_attributes = Window::default_attributes()
        .with_title("Stress Test 5000 Entities")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
    let window = event_loop.create_window(window_attributes)?;

    // Initialize systems
    let asset_loader: std::sync::Arc<dyn engine_core::assets::AssetLoader> = std::sync::Arc::new(engine::assets::native_loader::NativeAssetLoader::new("."));
    let mut ctx = EngineContext::new(asset_loader.clone());
    let mut script_engine = ScriptEngine::new(asset_loader.clone())?;
    
    #[cfg(feature = "rapier")]
    let mut physics = RapierPhysicsWorld::new();
    #[cfg(not(feature = "rapier"))]
    let mut physics = PhysicsWorld::new();

    // Init Renderer
    let mut renderer = pollster::block_on(RenderModule::new(&window))?;
    let mut texture_manager = TextureManager::new();
    let mut ui_manager = UIManager::new();
    let mut render_cache = engine::runtime::render_system::RenderCache::new();

    // Initialize egui for rendering the game view (reuse renderer logic)
    let egui_ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
        None,
    );
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        egui_wgpu::RendererOptions {
            depth_stencil_format: Some(wgpu::TextureFormat::Depth32Float),
            ..Default::default()
        },
    );

    // Initial World
    let mut world = World::new();

    // Spawn Camera
    let cam_entity = world.spawn();
    // Position camera to view the grid
    let _ = ComponentAccess::<Transform>::insert(&mut world, cam_entity, Transform::with_position(0.0, 50.0, 150.0));
    let mut cam = Camera::perspective_3d();
    cam.far_clip = 1000.0;
    // Rotate camera to look down slightly?
    // Rotation is Euler angles in degrees. (Pitch, Yaw, Roll) (X, Y, Z)
    // Pitch down 20 degrees
    let mut transform = Transform::with_position(0.0, 50.0, 150.0);
    transform.rotation = [-20.0, 0.0, 0.0];
    let _ = ComponentAccess::<Transform>::insert(&mut world, cam_entity, transform);
    let _ = ComponentAccess::<Camera>::insert(&mut world, cam_entity, cam);

    // Spawn 5000 Cubes (50 x 10 x 10)
    let count_x = 50;
    let count_y = 10;
    let count_z = 10;
    
    log::info!("Spawning {}x{}x{} = {} entities...", count_x, count_y, count_z, count_x*count_y*count_z);

    for x in 0..count_x {
        for y in 0..count_y {
            for z in 0..count_z {
                 let e = world.spawn();
                 let pos = [(x as f32 - 25.0) * 2.5, (y as f32) * 2.5, (z as f32 - 5.0) * 2.5];
                 let _ = ComponentAccess::<Transform>::insert(&mut world, e, Transform {
                     position: pos,
                     rotation: [x as f32 * 10.0, y as f32 * 10.0, 0.0],
                     scale: [1.0, 1.0, 1.0],
                 });
                 
                 let mesh = Mesh {
                     mesh_type: ecs::MeshType::Cube,
                     color: [x as f32 / 50.0, y as f32 / 10.0, z as f32 / 10.0, 1.0],
                     material_id: None,
                 };
                 let _ = ComponentAccess::<Mesh>::insert(&mut world, e, mesh);
                 let _ = ComponentAccess::<bool>::insert(&mut world, e, true); // Active
            }
        }
    }
    
    // Add Light
    let light_entity = world.spawn();
    let light = ecs::components::Light {
        color: [1.0, 1.0, 0.8], // Warm white
        intensity: 2.0,
        ..Default::default()
    };
    let _ = ComponentAccess::<ecs::components::Light>::insert(&mut world, light_entity, light);
    // Light Position/Direction
    let _ = ComponentAccess::<Transform>::insert(&mut world, light_entity, Transform::with_position(50.0, 100.0, 50.0));
    let _ = ComponentAccess::<bool>::insert(&mut world, light_entity, true);


    let mut last_frame_time = std::time::Instant::now();
    const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
    let mut physics_accumulator: f32 = 0.0;

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                let _ = egui_state.on_window_event(&window, event);
                
                // Handle close
                 match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_frame_time).as_secs_f32();
                        last_frame_time = now;
                        
                        // Rotate cubes
                         for (entity, transform) in world.transforms.iter_mut() {
                             // Skip camera (check if has camera component)
                             if !world.cameras.contains_key(entity) {
                                 transform.rotation[1] += 90.0 * dt; // Rotate Y
                             }
                         }

                        // Physics (Empty for now)
                        physics_accumulator += dt;
                        while physics_accumulator >= FIXED_TIMESTEP {
                            physics.step(FIXED_TIMESTEP, &mut world);
                            physics_accumulator -= FIXED_TIMESTEP;
                        }

                        // Render
                        let raw_input = egui_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        egui::CentralPanel::default().show(&egui_ctx, |ui| {
                            ui.label(format!("FPS: {:.1}", 1.0 / dt));
                            // Render game view (fullscreenish)
                             runtime::render_game_view(
                                ui,
                                &world,
                                &mut texture_manager,
                                Some(&mut ui_manager),
                                None, 
                            );
                        });

                        let full_output = egui_ctx.end_frame();
                        
                        // Frame rendering using egui_wgpu
                         let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [renderer.config.width, renderer.config.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        for (id, image_delta) in &full_output.textures_delta.set {
                            egui_renderer.update_texture(&renderer.device, &renderer.queue, *id, image_delta);
                        }

                        let screen_width = renderer.config.width;
                        let screen_height = renderer.config.height;
                        let renderer_size = renderer.size;

                        let res = renderer.render_with_callback(|device, queue, encoder, view, depth_view, texture_manager, tilemap_renderer, batch_renderer, mesh_renderer, camera_binding, light_binding, depth_texture, scene_depth_texture, _scene_depth_view, config| {
                            egui_renderer.update_buffers(
                                device,
                                queue,
                                encoder,
                                &paint_jobs,
                                &screen_descriptor,
                            );

                            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui_render"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                }),
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                            
                            // Calculate ViewProj (Simple LookAt 0,0,0)
                            let mut view_proj = glam::Mat4::IDENTITY;
                             if let Some(main_camera) = world.cameras.iter().min_by_key(|(_, camera)| camera.depth) {
                                let (entity, camera) = main_camera;
                                if let Some(transform) = world.transforms.get(entity) {
                                     use glam::{Vec3, Quat, Mat4, EulerRot};
                                     let rot_rad = Vec3::new(transform.rotation[0].to_radians(), transform.rotation[1].to_radians(), transform.rotation[2].to_radians());
                                    let cam_rotation = Quat::from_euler(EulerRot::YXZ, rot_rad.y, rot_rad.x, rot_rad.z);
                                    let cam_pos = Vec3::from(transform.position);
                                    let forward = cam_rotation * Vec3::Z; // -Z is forward in basic OpenGL, but here we might have configured differently.
                                    // Actually, standard LookAt takes target.
                                    // Let's use standard cam logic from player.rs
                                    // forward = cam_rotation * Vec3::NEG_Z?
                                    // Let's stick to what player.rs had:
                                    let forward = cam_rotation * Vec3::Z; // Wait, player.rs used Vec3::Z? 
                                    // "cam_pos + forward"
                                    // If Z is forward, then it's looking int +Z. 
                                    // Usually "Forward" is -Z in RH.
                                    // Let's verify `player.rs`.
                                    
                                    // player.rs:
                                    // let forward = cam_rotation * Vec3::Z;
                                    // let view = Mat4::look_at_rh(cam_pos, cam_pos + forward, up);
                                    
                                    // If I look at 0,0,0 from 0,50,150.
                                    // I want to look at (0,0,0).
                                    // Let's just hardcode LookAt for stress test to be safe.
                                    let view = Mat4::look_at_rh(cam_pos, Vec3::ZERO, Vec3::Y);
                                    
                                    let aspect = screen_width as f32 / screen_height.max(1) as f32;
                                     let projection = Mat4::perspective_rh(camera.fov.to_radians(), aspect, camera.near_clip, camera.far_clip);
                                    view_proj = projection * view;
                                }
                             }

                            // Prepare frame and shadows
                            let frame = runtime::render_system::prepare_frame_and_shadows(
                                &mut render_cache,
                                &world,
                                device,
                                queue,
                                texture_manager,
                                light_binding,
                                camera_binding,
                                mesh_renderer,
                                depth_view,
                                depth_texture,
                                scene_depth_texture,
                                config.width,
                                config.height,
                            );

                            // Render scene
                            runtime::render_system::render_scene(
                                &frame,
                                &mut render_cache,
                                &world,
                                tilemap_renderer,
                                batch_renderer,
                                mesh_renderer,
                                camera_binding,
                                light_binding,
                                texture_manager,
                                queue,
                                device,
                                renderer_size,
                                &mut rpass,
                                view_proj,
                            );

                            // Render UI on top
                            let mut rpass: wgpu::RenderPass<'static> = unsafe { std::mem::transmute(rpass) };
                            egui_renderer.render(
                                &mut rpass,
                                &paint_jobs,
                                &screen_descriptor,
                            );
                        });

                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        match res {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    })?;

    Ok(())
}
