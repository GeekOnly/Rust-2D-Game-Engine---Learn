use mlua::{Lua, Function, Table};
use anyhow::Result;
use ecs::{World, Entity};
use input::{InputSystem, Key, MouseButton, GamepadButton};

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
    /// Now uses InputSystem instead of HashMap<String, bool>
    pub fn run_script(
        &mut self,
        _script_path: &std::path::Path,
        entity: Entity,
        world: &mut World,
        input: &InputSystem,
        dt: f32,
    ) -> Result<()> {
        // Don't reload script - just call on_update with current state
        // Use scope to safely pass world reference
        self.lua.scope(|scope| {
            let globals = self.lua.globals();

            // ================================================================
            // KEYBOARD INPUT
            // ================================================================

            let is_key_pressed = scope.create_function(|_, key: String| {
                if let Some(key_enum) = Key::from_str(&key) {
                    Ok(input.is_key_down(key_enum))
                } else {
                    Ok(false)
                }
            })?;
            globals.set("is_key_pressed", is_key_pressed)?;

            let is_key_down = scope.create_function(|_, key: String| {
                if let Some(key_enum) = Key::from_str(&key) {
                    Ok(input.is_key_down(key_enum))
                } else {
                    Ok(false)
                }
            })?;
            globals.set("is_key_down", is_key_down)?;

            let is_key_just_pressed = scope.create_function(|_, key: String| {
                if let Some(key_enum) = Key::from_str(&key) {
                    Ok(input.is_key_pressed(key_enum))
                } else {
                    Ok(false)
                }
            })?;
            globals.set("is_key_just_pressed", is_key_just_pressed)?;

            let is_key_just_released = scope.create_function(|_, key: String| {
                if let Some(key_enum) = Key::from_str(&key) {
                    Ok(input.is_key_released(key_enum))
                } else {
                    Ok(false)
                }
            })?;
            globals.set("is_key_just_released", is_key_just_released)?;

            // ================================================================
            // MOUSE INPUT
            // ================================================================

            let is_mouse_button_pressed = scope.create_function(|_, button: String| {
                let btn = match button.as_str() {
                    "Left" => MouseButton::Left,
                    "Right" => MouseButton::Right,
                    "Middle" => MouseButton::Middle,
                    _ => return Ok(false),
                };
                Ok(input.is_mouse_button_down(btn))
            })?;
            globals.set("is_mouse_button_pressed", is_mouse_button_pressed)?;

            let get_mouse_position = scope.create_function(|lua, ()| {
                let pos = input.mouse_position();
                let table = lua.create_table()?;
                table.set("x", pos.x)?;
                table.set("y", pos.y)?;
                Ok(table)
            })?;
            globals.set("get_mouse_position", get_mouse_position)?;

            let get_mouse_delta = scope.create_function(|lua, ()| {
                let delta = input.mouse_delta();
                let table = lua.create_table()?;
                table.set("x", delta.x)?;
                table.set("y", delta.y)?;
                Ok(table)
            })?;
            globals.set("get_mouse_delta", get_mouse_delta)?;

            let get_mouse_scroll = scope.create_function(|lua, ()| {
                let scroll = input.mouse_scroll_delta();
                let table = lua.create_table()?;
                table.set("x", scroll.x)?;
                table.set("y", scroll.y)?;
                Ok(table)
            })?;
            globals.set("get_mouse_scroll", get_mouse_scroll)?;

            // ================================================================
            // GAMEPAD INPUT
            // ================================================================

            let is_gamepad_button_pressed = scope.create_function(|_, (gamepad_id, button): (usize, String)| {
                let btn = match button.as_str() {
                    "South" | "A" | "X" => GamepadButton::South,
                    "East" | "B" | "Circle" => GamepadButton::East,
                    "North" | "Y" | "Triangle" => GamepadButton::North,
                    "West" | "X" | "Square" => GamepadButton::West,
                    "L1" | "LB" => GamepadButton::L1,
                    "R1" | "RB" => GamepadButton::R1,
                    "L2" | "LT" => GamepadButton::L2,
                    "R2" | "RT" => GamepadButton::R2,
                    "Start" => GamepadButton::Start,
                    "Select" | "Back" => GamepadButton::Select,
                    "DPadUp" => GamepadButton::DPadUp,
                    "DPadDown" => GamepadButton::DPadDown,
                    "DPadLeft" => GamepadButton::DPadLeft,
                    "DPadRight" => GamepadButton::DPadRight,
                    _ => return Ok(false),
                };
                Ok(input.is_gamepad_button_down(gamepad_id, btn))
            })?;
            globals.set("is_gamepad_button_pressed", is_gamepad_button_pressed)?;

            let get_gamepad_left_stick = scope.create_function(|lua, gamepad_id: usize| {
                let stick = input.gamepad_left_stick(gamepad_id);
                let table = lua.create_table()?;
                table.set("x", stick.x)?;
                table.set("y", stick.y)?;
                Ok(table)
            })?;
            globals.set("get_gamepad_left_stick", get_gamepad_left_stick)?;

            let get_gamepad_right_stick = scope.create_function(|lua, gamepad_id: usize| {
                let stick = input.gamepad_right_stick(gamepad_id);
                let table = lua.create_table()?;
                table.set("x", stick.x)?;
                table.set("y", stick.y)?;
                Ok(table)
            })?;
            globals.set("get_gamepad_right_stick", get_gamepad_right_stick)?;

            let is_gamepad_connected = scope.create_function(|_, gamepad_id: usize| {
                Ok(input.is_gamepad_connected(gamepad_id))
            })?;
            globals.set("is_gamepad_connected", is_gamepad_connected)?;

            // ================================================================
            // VIRTUAL INPUT (cross-platform)
            // ================================================================

            let get_movement_input = scope.create_function(|lua, gamepad_id: Option<usize>| {
                let movement = input.get_movement_input(gamepad_id.unwrap_or(0));
                let table = lua.create_table()?;
                table.set("x", movement.x)?;
                table.set("y", movement.y)?;
                Ok(table)
            })?;
            globals.set("get_movement_input", get_movement_input)?;

            let get_action_button = scope.create_function(|_, gamepad_id: Option<usize>| {
                Ok(input.get_action_button(gamepad_id.unwrap_or(0)))
            })?;
            globals.set("get_action_button", get_action_button)?;

            let get_action_button_pressed = scope.create_function(|_, gamepad_id: Option<usize>| {
                Ok(input.get_action_button_pressed(gamepad_id.unwrap_or(0)))
            })?;
            globals.set("get_action_button_pressed", get_action_button_pressed)?;

            // ================================================================
            // ENTITY/WORLD MANIPULATION
            // ================================================================

            let set_velocity = scope.create_function_mut(|_, (vx, vy): (f32, f32)| {
                // Initialize velocity if it doesn't exist
                if !world.velocities.contains_key(&entity) {
                    world.velocities.insert(entity, (0.0, 0.0));
                }

                if let Some(velocity) = world.velocities.get_mut(&entity) {
                    velocity.0 = vx;
                    velocity.1 = vy;
                }
                Ok(())
            })?;
            globals.set("set_velocity", set_velocity)?;

            let get_velocity = scope.create_function(|lua, ()| {
                if let Some(velocity) = world.velocities.get(&entity) {
                    let table = lua.create_table()?;
                    table.set("x", velocity.0)?;
                    table.set("y", velocity.1)?;
                    Ok(Some(table))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_velocity", get_velocity)?;

            let get_position = scope.create_function(|lua, ()| {
                if let Some(transform) = world.transforms.get(&entity) {
                    let table = lua.create_table()?;
                    table.set("x", transform.position[0])?;
                    table.set("y", transform.position[1])?;
                    Ok(Some(table))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_position", get_position)?;

            let set_position = scope.create_function_mut(|_, (x, y): (f32, f32)| {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.position[0] = x;
                    transform.position[1] = y;
                }
                Ok(())
            })?;
            globals.set("set_position", set_position)?;

            let get_rotation = scope.create_function(|_, ()| {
                if let Some(transform) = world.transforms.get(&entity) {
                    Ok(Some(transform.rotation))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_rotation", get_rotation)?;

            let set_rotation = scope.create_function_mut(|_, rotation: f32| {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.rotation = rotation;
                }
                Ok(())
            })?;
            globals.set("set_rotation", set_rotation)?;

            let get_scale = scope.create_function(|lua, ()| {
                if let Some(transform) = world.transforms.get(&entity) {
                    let table = lua.create_table()?;
                    table.set("x", transform.scale[0])?;
                    table.set("y", transform.scale[1])?;
                    Ok(Some(table))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_scale", get_scale)?;

            let set_scale = scope.create_function_mut(|_, (x, y): (f32, f32)| {
                if let Some(transform) = world.transforms.get_mut(&entity) {
                    transform.scale[0] = x;
                    transform.scale[1] = y;
                }
                Ok(())
            })?;
            globals.set("set_scale", set_scale)?;

            // ================================================================
            // TAG & ENTITY QUERIES
            // ================================================================

            let get_tag = scope.create_function(|_, query_entity: Entity| {
                if let Some(tag) = world.tags.get(&query_entity) {
                    Ok(Some(tag.clone()))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_tag", get_tag)?;

            let set_tag = scope.create_function_mut(|_, tag: String| {
                world.tags.insert(entity, tag);
                Ok(())
            })?;
            globals.set("set_tag", set_tag)?;

            let get_name = scope.create_function(|_, query_entity: Entity| {
                if let Some(name) = world.entity_names.get(&query_entity) {
                    Ok(Some(name.clone()))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_name", get_name)?;

            let destroy_entity = scope.create_function_mut(|_, target_entity: Entity| {
                // Mark for destruction (actual removal should happen after script execution)
                world.destroy(target_entity);
                Ok(())
            })?;
            globals.set("destroy_entity", destroy_entity)?;

            // ================================================================
            // UTILITY FUNCTIONS
            // ================================================================

            let get_delta_time = scope.create_function(|_, ()| {
                Ok(dt)
            })?;
            globals.set("get_delta_time", get_delta_time)?;

            let print_log = scope.create_function(|_, msg: String| {
                println!("[Lua] {}", msg);
                Ok(())
            })?;
            globals.set("log", print_log)?;

            // ================================================================
            // CALL ON_UPDATE
            // ================================================================

            // Call on_update if it exists
            if let Ok(on_update) = globals.get::<_, Function>("on_update") {
                on_update.call::<_, ()>(dt)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
