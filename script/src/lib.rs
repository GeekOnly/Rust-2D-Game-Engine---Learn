use mlua::{Lua, Function};
use anyhow::Result;
use ecs::{World, Entity, EntityTag};
use input::{InputSystem, Key, MouseButton, GamepadButton};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct ScriptEngine {
    lua: Lua,
    // Per-entity Lua states for proper lifecycle management
    entity_states: HashMap<Entity, Lua>,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { 
            lua,
            entity_states: HashMap::new(),
        })
    }

    pub fn exec(&self, src: &str) -> Result<()> {
        self.lua.load(src).exec()?;
        Ok(())
    }

    /// Load a script for a specific entity (Unity-style with backward compatibility)
    /// This creates a separate Lua state for each entity to properly manage lifecycle
    pub fn load_script_for_entity(&mut self, entity: Entity, content: &str, world: &World) -> Result<()> {
        // Create a new Lua state for this entity
        let lua = Lua::new();
        
        // Load the script content
        lua.load(content).exec()?;

        // Inject script parameters as globals before calling Awake
        if let Some(script) = world.scripts.get(&entity) {
            {
                let globals = lua.globals();
                for (name, value) in &script.parameters {
                    match value {
                        ecs::ScriptParameter::Float(v) => globals.set(name.as_str(), *v)?,
                        ecs::ScriptParameter::Int(v) => globals.set(name.as_str(), *v)?,
                        ecs::ScriptParameter::String(v) => globals.set(name.as_str(), v.clone())?,
                        ecs::ScriptParameter::Bool(v) => globals.set(name.as_str(), *v)?,
                    }
                }
            } // Drop globals here
        }

        // Call Awake() if it exists (Unity-style)
        {
            let globals = lua.globals();
            if let Ok(awake) = globals.get::<_, Function>("Awake") {
                awake.call::<_, ()>(())?;
            }
            // Backward compatibility: call on_start
            else if let Ok(on_start) = globals.get::<_, Function>("on_start") {
                on_start.call::<_, ()>((entity,))?;
            }
        } // Drop globals here

        // Store the Lua state for this entity
        self.entity_states.insert(entity, lua);

        Ok(())
    }

    /// Call Start() for an entity (should be called after all Awake() calls)
    /// This needs world access to inject API functions
    pub fn call_start_for_entity(&self, entity: Entity, world: &mut World) -> Result<()> {
        if let Some(lua) = self.entity_states.get(&entity) {
            // Use RefCell to work around borrow checker
            let world_cell = RefCell::new(&mut *world);
            
            lua.scope(|scope| {
                let globals = lua.globals();
                
                // Inject essential API functions for Start()
                let set_velocity = scope.create_function_mut(|_, (vx, vy): (f32, f32)| {
                    world_cell.borrow_mut().velocities.insert(entity, (vx, vy));
                    if let Some(rigidbody) = world_cell.borrow_mut().rigidbodies.get_mut(&entity) {
                        rigidbody.velocity = (vx, vy);
                    }
                    Ok(())
                })?;
                globals.set("set_velocity", set_velocity)?;
                
                let set_gravity_scale = scope.create_function_mut(|_, scale: f32| {
                    if let Some(rigidbody) = world_cell.borrow_mut().rigidbodies.get_mut(&entity) {
                        rigidbody.gravity_scale = scale;
                    }
                    Ok(())
                })?;
                globals.set("set_gravity_scale", set_gravity_scale)?;
                
                // Call Start() if it exists
                if let Ok(start) = globals.get::<_, Function>("Start") {
                    start.call::<_, ()>(())?;
                }
                
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Remove entity's Lua state when entity is destroyed
    pub fn remove_entity_state(&mut self, entity: Entity) {
        self.entity_states.remove(&entity);
    }

    pub fn call_update(&self, name: &str, dt: f32, world: &mut World) -> Result<()> {
        let world_cell = RefCell::new(&mut *world);
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<_, Function>(name) {
            self.lua.scope(|scope| {
                let spawn = scope.create_function_mut(move |_, ()| {
                    Ok(world_cell.borrow_mut().spawn())
                })?;

                // Pass spawn directly
                func.call::<_, ()>((dt, spawn))?;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Update a script (call Update or on_update) - script should already be loaded
    /// Now uses InputSystem instead of HashMap<String, bool>
    pub fn run_script(
        &mut self,
        _script_path: &std::path::Path,
        entity: Entity,
        world: &mut World,
        input: &InputSystem,
        dt: f32,
    ) -> Result<()> {
        // Get the entity's Lua state
        let lua = match self.entity_states.get(&entity) {
            Some(lua) => lua,
            None => return Ok(()), // Entity has no loaded script
        };

        // Use RefCell to work around borrow checker in scope
        let world_cell = RefCell::new(&mut *world);

        lua.scope(|scope| {
            let globals = lua.globals();

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
                    "South" | "A" => GamepadButton::South,
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
                // Set velocity in both legacy and rigidbody systems
                world_cell.borrow_mut().velocities.insert(entity, (vx, vy));
                
                // Sync with rigidbody if it exists
                if let Some(rigidbody) = world_cell.borrow_mut().rigidbodies.get_mut(&entity) {
                    rigidbody.velocity = (vx, vy);
                }
                Ok(())
            })?;
            globals.set("set_velocity", set_velocity)?;

            let get_velocity = scope.create_function(|lua, ()| {
                // Try rigidbody first, then fall back to legacy velocity
                let velocity = if let Some(rigidbody) = world_cell.borrow().rigidbodies.get(&entity) {
                    rigidbody.velocity
                } else {
                    world_cell.borrow().velocities.get(&entity).copied().unwrap_or((0.0, 0.0))
                };
                
                let table = lua.create_table()?;
                table.set("x", velocity.0)?;
                table.set("y", velocity.1)?;
                Ok(Some(table))
            })?;
            globals.set("get_velocity", get_velocity)?;

            let set_gravity_scale = scope.create_function_mut(|_, scale: f32| {
                if let Some(rigidbody) = world_cell.borrow_mut().rigidbodies.get_mut(&entity) {
                    rigidbody.gravity_scale = scale;
                }
                Ok(())
            })?;
            globals.set("set_gravity_scale", set_gravity_scale)?;

            let get_gravity_scale = scope.create_function(|_, ()| {
                if let Some(rigidbody) = world_cell.borrow().rigidbodies.get(&entity) {
                    Ok(Some(rigidbody.gravity_scale))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_gravity_scale", get_gravity_scale)?;

            let get_position = scope.create_function(|lua, ()| {
                if let Some(transform) = world_cell.borrow().transforms.get(&entity) {
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
                if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
                    transform.position[0] = x;
                    transform.position[1] = y;
                }
                Ok(())
            })?;
            globals.set("set_position", set_position)?;

            let get_rotation = scope.create_function(|_, ()| {
                if let Some(transform) = world_cell.borrow().transforms.get(&entity) {
                    Ok(Some(transform.rotation[2]))  // Z-axis rotation for 2D
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_rotation", get_rotation)?;

            let set_rotation = scope.create_function_mut(|_, rotation: f32| {
                if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
                    transform.rotation[2] = rotation;  // Z-axis rotation for 2D
                }
                Ok(())
            })?;
            globals.set("set_rotation", set_rotation)?;

            let get_scale = scope.create_function(|lua, ()| {
                if let Some(transform) = world_cell.borrow().transforms.get(&entity) {
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
                if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
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
                if let Some(tag) = world_cell.borrow().tags.get(&query_entity) {
                    let tag_str = match tag {
                        EntityTag::Player => "Player",
                        EntityTag::Item => "Item",
                    };
                    Ok(Some(tag_str.to_string()))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_tag", get_tag)?;

            let set_tag = scope.create_function_mut(|_, tag: String| {
                let entity_tag = match tag.as_str() {
                    "Player" => Some(EntityTag::Player),
                    "Item" => Some(EntityTag::Item),
                    _ => None,
                };
                if let Some(t) = entity_tag {
                    world_cell.borrow_mut().tags.insert(entity, t);
                }
                Ok(())
            })?;
            globals.set("set_tag", set_tag)?;

            // TODO: get_name requires entity_names to be in World
            // let get_name = scope.create_function(|_, query_entity: Entity| {
            //     Ok(Some("GameObject".to_string()))
            // })?;
            // globals.set("get_name", get_name)?;

            let destroy_entity = scope.create_function_mut(|_, target_entity: Entity| {
                world_cell.borrow_mut().despawn(target_entity);
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
                log::info!("[Lua] {}", msg);
                Ok(())
            })?;
            globals.set("log", print_log)?;

            // ================================================================
            // INJECT SCRIPT PARAMETERS AS GLOBALS
            // ================================================================

            // Inject script parameters into Lua globals
            if let Some(script) = world_cell.borrow().scripts.get(&entity) {
                for (name, value) in &script.parameters {
                    match value {
                        ecs::ScriptParameter::Float(v) => globals.set(name.as_str(), *v)?,
                        ecs::ScriptParameter::Int(v) => globals.set(name.as_str(), *v)?,
                        ecs::ScriptParameter::String(v) => globals.set(name.as_str(), v.clone())?,
                        ecs::ScriptParameter::Bool(v) => globals.set(name.as_str(), *v)?,
                    }
                }
            }

            // ================================================================
            // CALL LIFECYCLE FUNCTIONS (Unity-style with backward compatibility)
            // ================================================================

            // Try Unity-style Update() first, then fall back to on_update()
            if let Ok(update_func) = globals.get::<_, Function>("Update") {
                // Unity-style: Update(dt)
                update_func.call::<_, ()>(dt)?;
            } else if let Ok(on_update) = globals.get::<_, Function>("on_update") {
                // Backward compatibility: on_update(entity, dt)
                on_update.call::<_, ()>((entity, dt))?;
            }

            Ok(())
        })?;

        Ok(())
    }

    /// Call on_collision callback for a script
    pub fn call_collision(
        &mut self,
        _script_path: &std::path::Path,
        entity: Entity,
        other_entity: Entity,
        world: &mut World,
    ) -> Result<()> {
        // Get the entity's Lua state
        let lua = match self.entity_states.get(&entity) {
            Some(lua) => lua,
            None => return Ok(()), // Entity has no loaded script
        };

        // Use RefCell to work around borrow checker in scope
        let world_cell = RefCell::new(&mut *world);

        lua.scope(|scope| {
            let globals = lua.globals();

            // ================================================================
            // ENTITY QUERY API (for collision callback)
            // ================================================================

            let get_tag = scope.create_function(|_, query_entity: Entity| {
                if let Some(tag) = world_cell.borrow().tags.get(&query_entity) {
                    let tag_str = match tag {
                        EntityTag::Player => "Player",
                        EntityTag::Item => "Item",
                    };
                    Ok(Some(tag_str.to_string()))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_tag", get_tag)?;

            let destroy_entity = scope.create_function_mut(|_, target_entity: Entity| {
                world_cell.borrow_mut().despawn(target_entity);
                Ok(())
            })?;
            globals.set("destroy_entity", destroy_entity)?;

            // ================================================================
            // CALL COLLISION CALLBACKS (Unity-style with backward compatibility)
            // ================================================================

            // Try Unity-style OnCollisionEnter first
            if let Ok(on_collision_enter) = globals.get::<_, Function>("OnCollisionEnter") {
                on_collision_enter.call::<_, ()>(other_entity)?;
            }
            // Backward compatibility: call on_collision
            else if let Ok(on_collision) = globals.get::<_, Function>("on_collision") {
                on_collision.call::<_, ()>(other_entity)?;
            }

            Ok(())
        })?;

        Ok(())
    }
}
