use engine_core::EngineContext;
use physics::PhysicsWorld;
use crate::states::EditorState;
use script::ScriptEngine;
#[cfg(feature = "rapier")]
use physics::rapier_backend::RapierPhysicsWorld;
#[cfg(not(feature = "rapier"))]
use physics::PhysicsWorld;

pub struct PlayModeSystem;

impl PlayModeSystem {
    pub fn update(
        editor_state: &mut EditorState,
        ctx: &mut EngineContext,
        script_engine: &mut ScriptEngine,
        physics: &mut dyn std::any::Any,
        physics_accumulator: &mut f32,
        fixed_time_step: f32,
        dt: f32,
        asset_loader: &dyn engine_core::assets::AssetLoader,
    ) {
        if !editor_state.is_playing {
            return;
        }

        // Update gamepads (but don't clear input yet - scripts need to read it first)
        ctx.input.update_gamepads();
        
        // Update debug draw system
        editor_state.debug_draw.update(dt);

        // Update ground states for Rapier (before running scripts)
        #[cfg(feature = "rapier")]
        {
            if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                let entities_with_rigidbodies: Vec<_> = editor_state.world.rigidbodies.keys().cloned().collect();
                for entity in entities_with_rigidbodies {
                    // Cast ray 0.15 units down from player center
                    let is_grounded = rapier_world.raycast_ground(entity, &editor_state.world, 0.15);
                    script_engine.set_ground_state(entity, is_grounded);
                    
                    // Debug draw raycast
                    if let Some(transform) = editor_state.world.transforms.get(&entity) {
                        let collider_half_height = if let Some(collider) = editor_state.world.colliders.get(&entity) {
                            collider.get_world_height(transform.scale[1]) / 2.0
                        } else {
                            0.0
                        };
                        
                        let ray_start = [
                            transform.position[0],
                            transform.position[1] - collider_half_height,
                            transform.position[2],
                        ];
                        let ray_end = [
                            transform.position[0],
                            transform.position[1] - collider_half_height - 0.15,
                            transform.position[2],
                        ];
                        
                        // Green if grounded, Red if not
                        if is_grounded {
                            editor_state.debug_draw.draw_line_green(ray_start, ray_end, 0.0);
                        } else {
                            editor_state.debug_draw.draw_line_red(ray_start, ray_end, 0.0);
                        }
                    }
                }
            }
        }
        
        // Run scripts FIRST (before physics) so they can set velocities
        {
            let entities: Vec<ecs::Entity> = editor_state.world.scripts.keys().cloned().collect();

            for entity in entities {
                let should_run = if let Some(script) = editor_state.world.scripts.get(&entity) {
                    script.enabled
                } else {
                    false
                };

                if should_run {
                    let mut log_callback = |msg: String| {
                        log::info!("[Lua] {}", msg);
                    };

                    let result = script_engine.run_script(
                        std::path::Path::new(""),
                        entity,
                        &mut editor_state.world,
                        &ctx.input,
                        dt,
                        &mut log_callback,
                    );

                    if let Err(e) = result {
                        editor_state.console.error(format!("Script error for entity {}: {}", entity, e));
                    }
                }
            }
        }

        // Transfer debug lines from script engine to debug_draw manager
        let script_debug_lines = script_engine.take_debug_lines();
        for line in script_debug_lines {
            // Convert script DebugLine to editor DebugLine
            let color = egui::Color32::from_rgba_premultiplied(
                (line.color[0] * 255.0) as u8,
                (line.color[1] * 255.0) as u8,
                (line.color[2] * 255.0) as u8,
                (line.color[3] * 255.0) as u8,
            );
            editor_state.debug_draw.draw_line(line.start, line.end, color, line.duration);
        }
        
        // Process UI commands from Lua scripts
        let ui_commands = script_engine.take_ui_commands();
        for command in ui_commands {
            use script::UICommand;
            match command {
                UICommand::LoadPrefab { path } => {
                    if let Err(e) = editor_state.ui_manager.load_prefab(&path) {
                        editor_state.console.error(format!("Failed to load prefab '{}': {}", path, e));
                    }
                }
                UICommand::ActivatePrefab { path, instance_name } => {
                    if let Err(e) = editor_state.ui_manager.activate_prefab(&path, &instance_name) {
                        editor_state.console.error(format!("Failed to activate prefab '{}': {}", path, e));
                    }
                }
                UICommand::DeactivatePrefab { instance_name } => {
                    editor_state.ui_manager.deactivate_prefab(&instance_name);
                }
                UICommand::SetText { element_path, text } => {
                    editor_state.ui_manager.set_ui_data(&element_path, text);
                }
                UICommand::SetImageFill { element_path, fill_amount } => {
                    if let Some((instance, element)) = element_path.split_once('/') {
                        if let Err(e) = editor_state.ui_manager.set_element_fill(instance, element, fill_amount) {
                            editor_state.console.error(format!("Failed to set fill: {}", e));
                        }
                    }
                }
                UICommand::SetColor { element_path, r, g, b, a } => {
                    if let Some((instance, element)) = element_path.split_once('/') {
                        if let Err(e) = editor_state.ui_manager.set_element_color(instance, element, r, g, b, a) {
                            editor_state.console.error(format!("Failed to set color: {}", e));
                        }
                    }
                }
                UICommand::ShowElement { element_path } => {
                    if let Some((instance, element)) = element_path.split_once('/') {
                        if let Err(e) = editor_state.ui_manager.show_element(instance, element) {
                            editor_state.console.error(format!("Failed to show element: {}", e));
                        }
                    }
                }
                UICommand::HideElement { element_path } => {
                    if let Some((instance, element)) = element_path.split_once('/') {
                        if let Err(e) = editor_state.ui_manager.hide_element(instance, element) {
                            editor_state.console.error(format!("Failed to hide element: {}", e));
                        }
                    }
                }
            }
        }

        // Accumulate frame time for fixed timestep physics
        *physics_accumulator += dt;
        
        // Update physics with fixed timestep (may run multiple times per frame)
        let mut physics_steps = 0;
        while *physics_accumulator >= fixed_time_step {
            #[cfg(feature = "rapier")]
            {
                if let Some(rapier_world) = physics.downcast_mut::<RapierPhysicsWorld>() {
                    rapier_world.step(fixed_time_step, &mut editor_state.world);
                }
            }
            #[cfg(not(feature = "rapier"))]
            {
                if let Some(simple_world) = physics.downcast_mut::<PhysicsWorld>() {
                    simple_world.step(fixed_time_step, &mut editor_state.world);
                }
            }
            
            *physics_accumulator -= fixed_time_step;
            physics_steps += 1;
            
            // Safety: prevent spiral of death (too many physics steps)
            if physics_steps >= 5 {
                *physics_accumulator = 0.0;
                break;
            }
        }
        
        // Check collisions and call collision callbacks (using simple fallback for now or Rapier events if implemented)
        // Note: For Rapier, we should arguably use its EventQueue, but for now maintaining simple check compatibility
        // This is O(N^2) and should be optimized or replaced by physics engine events
        let entities_with_colliders: Vec<_> = editor_state.world.colliders.keys().cloned().collect();
        for i in 0..entities_with_colliders.len() {
            for j in (i + 1)..entities_with_colliders.len() {
                let e1 = entities_with_colliders[i];
                let e2 = entities_with_colliders[j];

                let collision = {
                        #[cfg(feature = "rapier")]
                        {
                            // TODO: Use Rapier contact events
                            PhysicsWorld::check_collision(&editor_state.world, e1, e2) 
                        }
                        #[cfg(not(feature = "rapier"))]
                        {
                            PhysicsWorld::check_collision(&editor_state.world, e1, e2) 
                        }
                };

                if collision {
                    // Call on_collision for e1's script
                    if let Some(script) = editor_state.world.scripts.get(&e1).filter(|s| s.enabled) {
                        let script_name = script.script_name.clone();
                        // Call collision (path ignored by engine, resolved via entity state)
                        if let Err(e) = script_engine.call_collision(&std::path::Path::new(""), e1, e2, &mut editor_state.world) {
                             editor_state.console.error(format!("Collision error {}: {}", script_name, e));
                        }
                    }

                    // Call on_collision for e2's script
                    if let Some(script) = editor_state.world.scripts.get(&e2).filter(|s| s.enabled) {
                        let script_name = script.script_name.clone();
                        if let Err(e) = script_engine.call_collision(&std::path::Path::new(""), e2, e1, &mut editor_state.world) {
                             editor_state.console.error(format!("Collision error {}: {}", script_name, e));
                        }
                    }
                }
            }
        }

        // Clear per-frame input state AFTER scripts have run
        ctx.input.begin_frame();
    }
}
