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
    window::WindowBuilder,
};

// Use the engine library
use engine::runtime;
use engine::texture_manager::TextureManager;
use engine::ui_manager::UIManager;

// Map winit keycode to our input system key
fn map_winit_keycode(keycode: winit::keyboard::KeyCode) -> Option<input::Key> {
    use winit::keyboard::KeyCode;
    use input::Key;
    
    match keycode {
        KeyCode::KeyA => Some(Key::A),
        KeyCode::KeyB => Some(Key::B),
        KeyCode::KeyC => Some(Key::C),
        KeyCode::KeyD => Some(Key::D),
        KeyCode::KeyE => Some(Key::E),
        KeyCode::KeyF => Some(Key::F),
        KeyCode::KeyG => Some(Key::G),
        KeyCode::KeyH => Some(Key::H),
        KeyCode::KeyI => Some(Key::I),
        KeyCode::KeyJ => Some(Key::J),
        KeyCode::KeyK => Some(Key::K),
        KeyCode::KeyL => Some(Key::L),
        KeyCode::KeyM => Some(Key::M),
        KeyCode::KeyN => Some(Key::N),
        KeyCode::KeyO => Some(Key::O),
        KeyCode::KeyP => Some(Key::P),
        KeyCode::KeyQ => Some(Key::Q),
        KeyCode::KeyR => Some(Key::R),
        KeyCode::KeyS => Some(Key::S),
        KeyCode::KeyT => Some(Key::T),
        KeyCode::KeyU => Some(Key::U),
        KeyCode::KeyV => Some(Key::V),
        KeyCode::KeyW => Some(Key::W),
        KeyCode::KeyX => Some(Key::X),
        KeyCode::KeyY => Some(Key::Y),
        KeyCode::KeyZ => Some(Key::Z),
        KeyCode::Digit0 => Some(Key::Num0),
        KeyCode::Digit1 => Some(Key::Num1),
        KeyCode::Digit2 => Some(Key::Num2),
        KeyCode::Digit3 => Some(Key::Num3),
        KeyCode::Digit4 => Some(Key::Num4),
        KeyCode::Digit5 => Some(Key::Num5),
        KeyCode::Digit6 => Some(Key::Num6),
        KeyCode::Digit7 => Some(Key::Num7),
        KeyCode::Digit8 => Some(Key::Num8),
        KeyCode::Digit9 => Some(Key::Num9),
        KeyCode::ArrowUp => Some(Key::Up),
        KeyCode::ArrowDown => Some(Key::Down),
        KeyCode::ArrowLeft => Some(Key::Left),
        KeyCode::ArrowRight => Some(Key::Right),
        KeyCode::Space => Some(Key::Space),
        KeyCode::Enter => Some(Key::Enter),
        KeyCode::Escape => Some(Key::Escape),
        KeyCode::Tab => Some(Key::Tab),
        KeyCode::Backspace => Some(Key::Backspace),
        KeyCode::Delete => Some(Key::Delete),
        KeyCode::ShiftLeft => Some(Key::LShift),
        KeyCode::ShiftRight => Some(Key::RShift),
        KeyCode::ControlLeft => Some(Key::LCtrl),
        KeyCode::ControlRight => Some(Key::RCtrl),
        KeyCode::AltLeft => Some(Key::LAlt),
        KeyCode::AltRight => Some(Key::RAlt),
        _ => None,
    }
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("=== Game Player Runtime Starting ===");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rust 2D Game Engine Player")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)?;

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

    // Initialize egui for rendering the game view (reuse renderer logic)
    let egui_ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        Some(wgpu::TextureFormat::Depth32Float),
        1,
    );
 
    // Load Game Project
    // In a real export, these paths would be relative to the executable
    let mut project_path = std::env::current_dir()?;

    // If we're in the root directory, look for FPS 3D Example project
    if !project_path.join("scenes").exists() {
        let fps_path = project_path.join("projects/FPS 3D Example");
        if fps_path.exists() {
            project_path = fps_path;
        }
    }

    log::info!("Loading project from: {:?}", project_path);

    // Initial World
    let mut world = World::new();
    let mut scene_path = project_path.join("scenes/main.json");
    
    // Check if defaults exist, otherwise try to find scene in arguments?
    // For now we assume typical export structure
    if !scene_path.exists() {
        log::warn!("Scene not found at {:?}, trying 'scenes/main.scene'", scene_path);
        let alt_path = project_path.join("scenes/main.scene");
        if alt_path.exists() {
            scene_path = alt_path;
        } else {
             log::error!("No scene file found!");
        }
    }

    if scene_path.exists() {
        if let Ok(json) = std::fs::read_to_string(&scene_path) {
            if let Err(e) = world.load_from_json(&json) {
                log::error!("Failed to load scene: {}", e);
            } else {
                log::info!("Scene loaded successfully");
                
                // Load scripts after scene is loaded
                // scripts_folder is no longer needed as argument, simpler call:
                if let Err(e) = runtime::script_loader::load_all_scripts(&mut world, &mut script_engine) {
                    log::error!("Failed to load scripts: {}", e);
                } else {
                    log::info!("Scripts loaded successfully");
                }
            }
        }
    }

    // Set texture base path
    texture_manager.set_base_path(project_path.join("assets"));

    // [SCENE POST-PROCESSING] Load External Assets (GLTF)
    // Use shared function explicitly
    runtime::render_system::post_process_asset_meshes(
        &project_path,
        &mut world,
        &renderer.device,
        &renderer.queue,
        &mut renderer.texture_manager,
        &renderer.mesh_renderer,
        &*asset_loader,
    );

    let mut last_frame_time = std::time::Instant::now();
    const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
    let mut physics_accumulator: f32 = 0.0;

    // Start scripts (Init) - call for all entities with scripts
    let entities_with_scripts: Vec<_> = world.scripts.keys().copied().collect();
    for entity in entities_with_scripts {
        if let Err(e) = script_engine.call_start_for_entity(entity, &mut world) {
            log::error!("Script start error for entity {:?}: {}", entity, e);
        }
    }

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                let _ = egui_state.on_window_event(&window, event);
                
                // Handle input events
                match event {
                    WindowEvent::KeyboardInput { event, .. } => {
                        use winit::keyboard::PhysicalKey;
                        if let PhysicalKey::Code(keycode) = event.physical_key {
                            if let Some(key) = map_winit_keycode(keycode) {
                                match event.state {
                                    winit::event::ElementState::Pressed => {
                                        ctx.input.press_key(key);
                                    }
                                    winit::event::ElementState::Released => {
                                        ctx.input.release_key(key);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_frame_time).as_secs_f32();
                        last_frame_time = now;

                        // Scripts Update - use proper script system (before clearing input)
                        runtime::script_system::update_scripts(&mut script_engine, &mut world, &ctx.input, dt);

                        // Process UI commands from Lua scripts
                        let ui_commands = script_engine.take_ui_commands();
                        for command in ui_commands {
                            use script::UICommand;
                            match command {
                                UICommand::LoadPrefab { path } => {
                                    if let Err(e) = ui_manager.load_prefab(&path) {
                                        log::error!("Failed to load prefab '{}': {}", path, e);
                                    }
                                }
                                UICommand::ActivatePrefab { path, instance_name } => {
                                    if let Err(e) = ui_manager.activate_prefab(&path, &instance_name) {
                                        log::error!("Failed to activate prefab '{}': {}", path, e);
                                    }
                                }
                                UICommand::DeactivatePrefab { instance_name } => {
                                    ui_manager.deactivate_prefab(&instance_name);
                                }
                                UICommand::SetText { element_path, text } => {
                                    ui_manager.set_ui_data(&element_path, text);
                                }
                                UICommand::SetImageFill { element_path, fill_amount } => {
                                    if let Some((instance, element)) = element_path.split_once('/') {
                                        if let Err(e) = ui_manager.set_element_fill(instance, element, fill_amount) {
                                            log::error!("Failed to set fill: {}", e);
                                        }
                                    }
                                }
                                UICommand::SetColor { element_path, r, g, b, a } => {
                                    if let Some((instance, element)) = element_path.split_once('/') {
                                        if let Err(e) = ui_manager.set_element_color(instance, element, r, g, b, a) {
                                            log::error!("Failed to set color: {}", e);
                                        }
                                    }
                                }
                                UICommand::ShowElement { element_path } => {
                                    if let Some((instance, element)) = element_path.split_once('/') {
                                        if let Err(e) = ui_manager.show_element(instance, element) {
                                            log::error!("Failed to show element: {}", e);
                                        }
                                    }
                                }
                                UICommand::HideElement { element_path } => {
                                    if let Some((instance, element)) = element_path.split_once('/') {
                                        if let Err(e) = ui_manager.hide_element(instance, element) {
                                            log::error!("Failed to hide element: {}", e);
                                        }
                                    }
                                }
                            }
                        }

                        // Clear per-frame input state AFTER scripts have read it
                        ctx.input.begin_frame();

                        // Physics
                        physics_accumulator += dt;
                        while physics_accumulator >= FIXED_TIMESTEP {
                            physics.step(FIXED_TIMESTEP, &mut world);
                            physics_accumulator -= FIXED_TIMESTEP;
                        }

                        // Render
                        let raw_input = egui_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        egui::CentralPanel::default().show(&egui_ctx, |ui| {
                            // Render the game view fullscreen
                            runtime::render_game_view(
                                ui,
                                &world,
                                &mut texture_manager,
                                Some(&mut ui_manager),
                                None, // Default settings (fullscreen)
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

                        let res = renderer.render_with_callback(|device, queue, encoder, view, depth_view, texture_manager, tilemap_renderer, batch_renderer, mesh_renderer, camera_binding, light_binding| {
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
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0), // Standard Z: clear to 1.0 (Matches BatchRenderer)
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                }),
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                            
                            // Find Main Camera and Calculate ViewProj
                            let mut view_proj = glam::Mat4::IDENTITY;
                            if let Some(main_camera) = world.cameras.iter()
                                .min_by_key(|(_, camera)| camera.depth)
                            {
                                let (entity, camera) = main_camera;
                                if let Some(transform) = world.transforms.get(entity) {
                                     use glam::{Vec3, Quat, Mat4, EulerRot};
                                     let rot_rad = Vec3::new(
                                        transform.rotation[0].to_radians(),
                                        transform.rotation[1].to_radians(),
                                        transform.rotation[2].to_radians(),
                                    );
                                    let cam_rotation = Quat::from_euler(EulerRot::YXZ, rot_rad.y, rot_rad.x, rot_rad.z);
                                    let cam_pos = Vec3::from(transform.position);
                                    let forward = cam_rotation * Vec3::Z;
                                    let up = cam_rotation * Vec3::Y;
                                    let view = Mat4::look_at_rh(cam_pos, cam_pos + forward, up);
                                    let max_dim = screen_width.max(1) as f32;
                                    let aspect = screen_width as f32 / screen_height.max(1) as f32;
                                    let projection = match camera.projection {
                                        ecs::CameraProjection::Perspective => {
                                            Mat4::perspective_rh(camera.fov.to_radians(), aspect, camera.near_clip, camera.far_clip)
                                        }
                                        ecs::CameraProjection::Orthographic => {
                                            let half_height = camera.orthographic_size;
                                            let half_width = half_height * aspect;
                                            Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, camera.far_clip, camera.near_clip)
                                        }
                                    };
                                    view_proj = projection * view;
                                }
                            } else {
                                // Fallback default camera
                                let projection = glam::Mat4::orthographic_rh(-8.8, 8.8, -5.0, 5.0, 50.0, 0.1);
                                view_proj = projection;
                            }

                            // Render Game World (3D / WGPU)
                            runtime::render_system::render_game_world(
                                &world,
                                tilemap_renderer,
                                batch_renderer,
                                mesh_renderer,
                                camera_binding,
                                light_binding,
                                texture_manager,
                                queue,
                                device,
                                window.inner_size(),
                                &mut rpass,
                                view_proj,
                            );

                            // Render UI on top
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
