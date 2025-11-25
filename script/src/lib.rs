use mlua::{Lua, Function};
use anyhow::Result;
use ecs::{World, Entity};

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    pub fn exec(&self, src: &str) -> Result<()> {
        self.lua.load(src).exec()?;
        Ok(())
    }

    /// Load a script file and call on_start if it exists
    pub fn load_script(&self, content: &str) -> Result<()> {
        self.lua.load(content).exec()?;

        // Call on_start if it exists
        let globals = self.lua.globals();
        if let Ok(on_start) = globals.get::<_, Function>("on_start") {
            on_start.call::<_, ()>(())?;
        }

        Ok(())
    }

    pub fn call_update(&self, name: &str, dt: f32, world: &mut World) -> Result<()> {
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<_, Function>(name) {
            self.lua.scope(|scope| {
                let spawn = scope.create_function_mut(move |_, ()| {
                    Ok(world.spawn())
                })?;

                // Pass spawn directly
                func.call::<_, ()>((dt, spawn))?;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Update a script (call on_update) - script should already be loaded
    pub fn run_script(&mut self, _script_path: &std::path::Path, entity: Entity, world: &mut World, keyboard_state: &std::collections::HashMap<String, bool>) -> Result<()> {
        // Don't reload script - just call on_update with current state
        // Use scope to safely pass world reference
        self.lua.scope(|scope| {
            let globals = self.lua.globals();

            // is_key_pressed function - check keyboard_state
            let is_key_pressed = scope.create_function(|_, key: String| {
                // Map common key names to KeyCode format
                let key_code = match key.as_str() {
                    "W" => "KeyW",
                    "A" => "KeyA",
                    "S" => "KeyS",
                    "D" => "KeyD",
                    "Up" => "ArrowUp",
                    "Down" => "ArrowDown",
                    "Left" => "ArrowLeft",
                    "Right" => "ArrowRight",
                    _ => &key,
                };
                Ok(keyboard_state.get(key_code).copied().unwrap_or(false))
            })?;
            globals.set("is_key_pressed", is_key_pressed)?;

            // set_velocity function - FIXED: Only affects THIS entity
            let set_velocity = scope.create_function_mut(|_, (vx, vy): (f32, f32)| {
                if let Some(velocity) = world.velocities.get_mut(&entity) {
                    velocity.0 = vx;
                    velocity.1 = vy;
                }
                Ok(())
            })?;
            globals.set("set_velocity", set_velocity)?;

            // Call on_update if it exists
            if let Ok(on_update) = globals.get::<_, Function>("on_update") {
                let dt = 0.016; // Approximate 60 FPS for now
                on_update.call::<_, ()>(dt)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
