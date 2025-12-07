use mlua::{Lua, Function, Table};
use anyhow::Result;
use ecs::{World, Entity, EntityTag};
use input::{InputSystem, Key, MouseButton, GamepadButton};
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(feature = "rapier")]
mod rapier_bindings;

// Debug draw structures (simple versions for Lua)
#[derive(Clone, Debug)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4], // RGBA
    pub duration: f32,
}

// UI command types for Lua -> Engine communication
#[derive(Clone, Debug)]
pub enum UICommand {
    LoadPrefab { path: String },
    ActivatePrefab { path: String, instance_name: String },
    DeactivatePrefab { instance_name: String },
    SetText { element_path: String, text: String },
    SetImageFill { element_path: String, fill_amount: f32 },
    SetColor { element_path: String, r: f32, g: f32, b: f32, a: f32 },
    ShowElement { element_path: String },
    HideElement { element_path: String },
}

pub struct ScriptEngine {
    lua: Lua,
    // Per-entity Lua states for proper lifecycle management
    entity_states: HashMap<Entity, Lua>,
    // Store ground state for Rapier (temporary solution)
    pub ground_states: HashMap<Entity, bool>,
    // Debug draw queue (accessible from Lua scripts)
    pub debug_lines: RefCell<Vec<DebugLine>>,
    // UI command queue (Lua -> Engine)
    pub ui_commands: RefCell<Vec<UICommand>>,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { 
            lua,
            entity_states: HashMap::new(),
            ground_states: HashMap::new(),
            debug_lines: RefCell::new(Vec::new()),
            ui_commands: RefCell::new(Vec::new()),
        })
    }
    
    /// Get and clear debug lines (called by engine after rendering)
    pub fn take_debug_lines(&self) -> Vec<DebugLine> {
        self.debug_lines.borrow_mut().drain(..).collect()
    }
    
    /// Get and clear UI commands (called by engine to process UI updates)
    pub fn take_ui_commands(&self) -> Vec<UICommand> {
        self.ui_commands.borrow_mut().drain(..).collect()
    }
    
    /// Set ground state for entity (called by engine with Rapier result)
    pub fn set_ground_state(&mut self, entity: Entity, is_grounded: bool) {
        self.ground_states.insert(entity, is_grounded);
    }

    pub fn exec(&self, src: &str) -> Result<()> {
        self.lua.load(src).exec()?;
        Ok(())
    }

    /// Load a script for a specific entity (Unity-style with backward compatibility)
    /// This creates a separate Lua state for each entity to properly manage lifecycle
    pub fn load_script_for_entity(&mut self, entity: Entity, content: &str, world: &mut World) -> Result<()> {
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
                        ecs::ScriptParameter::Entity(Some(e)) => globals.set(name.as_str(), *e)?,
                        ecs::ScriptParameter::Entity(None) => globals.set(name.as_str(), mlua::Nil)?,
                    }
                }
            } // Drop globals here
        }

        // Inject basic API functions and call on_start within the same scope
        {
            let world_cell = RefCell::new(&mut *world);
            
            lua.scope(|scope| {
                let globals = lua.globals();
                
                // Entity query functions
                let get_all_entities = scope.create_function(|lua, ()| {
                    let entities: Vec<Entity> = world_cell.borrow().transforms.keys().copied().collect();
                    let table = lua.create_table()?;
                    for (i, ent) in entities.iter().enumerate() {
                        table.set(i + 1, *ent)?;
                    }
                    Ok(table)
                })?;
                globals.set("get_all_entities", get_all_entities)?;
                
                let get_tags = scope.create_function(|lua, query_entity: Entity| {
                    let table = lua.create_table()?;
                    if let Some(tag) = world_cell.borrow().tags.get(&query_entity) {
                        let tag_str = match tag {
                            EntityTag::Player => "Player",
                            EntityTag::Item => "Item",
                        };
                        table.set(1, tag_str)?;
                    }
                    Ok(table)
                })?;
                globals.set("get_tags", get_tags)?;
                
                let get_position_of = scope.create_function(|lua, query_entity: Entity| {
                    if let Some(transform) = world_cell.borrow().transforms.get(&query_entity) {
                        let table = lua.create_table()?;
                        table.set("x", transform.position[0])?;
                        table.set("y", transform.position[1])?;
                        table.set("z", transform.position[2])?;
                        Ok(Some(table))
                    } else {
                        Ok(None)
                    }
                })?;
                globals.set("get_position_of", get_position_of)?;
                
                let get_position = scope.create_function(|lua, ()| {
                    if let Some(transform) = world_cell.borrow().transforms.get(&entity) {
                        let table = lua.create_table()?;
                        table.set("x", transform.position[0])?;
                        table.set("y", transform.position[1])?;
                        table.set("z", transform.position[2])?;
                        Ok(Some(table))
                    } else {
                        Ok(None)
                    }
                })?;
                globals.set("get_position", get_position)?;
                
                let set_position = scope.create_function_mut(|_, (x, y, z): (f32, f32, f32)| {
                    if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
                        transform.position[0] = x;
                        transform.position[1] = y;
                        transform.position[2] = z;
                    }
                    Ok(())
                })?;
                globals.set("set_position", set_position)?;
                
                // Call Awake() or on_start() within the scope while functions are still valid
                if let Ok(awake) = globals.get::<_, Function>("Awake") {
                    awake.call::<_, ()>(())?;
                } else if let Ok(on_start) = globals.get::<_, Function>("on_start") {
                    on_start.call::<_, ()>(())?;
                }
                
                Ok(())
            })?;
        }

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
                
                // Add entity query functions for camera follow
                let get_all_entities = scope.create_function(|lua, ()| {
                    let entities: Vec<Entity> = world_cell.borrow().transforms.keys().copied().collect();
                    let table = lua.create_table()?;
                    for (i, ent) in entities.iter().enumerate() {
                        table.set(i + 1, *ent)?;
                    }
                    Ok(table)
                })?;
                globals.set("get_all_entities", get_all_entities)?;
                
                let get_tags = scope.create_function(|lua, query_entity: Entity| {
                    let table = lua.create_table()?;
                    if let Some(tag) = world_cell.borrow().tags.get(&query_entity) {
                        let tag_str = match tag {
                            EntityTag::Player => "Player",
                            EntityTag::Item => "Item",
                        };
                        table.set(1, tag_str)?;
                    }
                    Ok(table)
                })?;
                globals.set("get_tags", get_tags)?;
                
                let get_position = scope.create_function(|lua, ()| {
                    if let Some(transform) = world_cell.borrow().transforms.get(&entity) {
                        let table = lua.create_table()?;
                        table.set("x", transform.position[0])?;
                        table.set("y", transform.position[1])?;
                        table.set("z", transform.position[2])?;
                        Ok(Some(table))
                    } else {
                        Ok(None)
                    }
                })?;
                globals.set("get_position", get_position)?;
                
                let get_position_of = scope.create_function(|lua, query_entity: Entity| {
                    if let Some(transform) = world_cell.borrow().transforms.get(&query_entity) {
                        let table = lua.create_table()?;
                        table.set("x", transform.position[0])?;
                        table.set("y", transform.position[1])?;
                        table.set("z", transform.position[2])?;
                        Ok(Some(table))
                    } else {
                        Ok(None)
                    }
                })?;
                globals.set("get_position_of", get_position_of)?;
                
                let set_position = scope.create_function_mut(|_, (x, y, z): (f32, f32, f32)| {
                    if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
                        transform.position[0] = x;
                        transform.position[1] = y;
                        transform.position[2] = z;
                    }
                    Ok(())
                })?;
                globals.set("set_position", set_position)?;
                
                let get_velocity_of = scope.create_function(|lua, query_entity: Entity| {
                    if let Some(vel) = world_cell.borrow().velocities.get(&query_entity) {
                        let table = lua.create_table()?;
                        table.set("x", vel.0)?;
                        table.set("y", vel.1)?;
                        Ok(Some(table))
                    } else if let Some(rb) = world_cell.borrow().rigidbodies.get(&query_entity) {
                        let table = lua.create_table()?;
                        table.set("x", rb.velocity.0)?;
                        table.set("y", rb.velocity.1)?;
                        Ok(Some(table))
                    } else {
                        Ok(None)
                    }
                })?;
                globals.set("get_velocity_of", get_velocity_of)?;
                
                // Call Start() or on_start() if it exists (Unity-style with backward compatibility)
                if let Ok(start) = globals.get::<_, Function>("Start") {
                    start.call::<_, ()>(())?;
                } else if let Ok(on_start) = globals.get::<_, Function>("on_start") {
                    on_start.call::<_, ()>(())?;
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
        log_callback: &mut dyn FnMut(String),
    ) -> Result<()> {
        // Get the entity's Lua state
        let lua = match self.entity_states.get(&entity) {
            Some(lua) => lua,
            None => return Ok(()), // Entity has no loaded script
        };

        // Use RefCell to work around borrow checker in scope
        let world_cell = RefCell::new(&mut *world);
        let log_callback_cell = RefCell::new(log_callback);

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
                    let result = input.is_key_pressed(key_enum);
                    if result {
                        log::info!("🔍 Key '{}' just pressed!", key);
                    }
                    Ok(result)
                } else {
                    log::warn!("⚠️ Unknown key: '{}'", key);
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

            // Removed 2-parameter set_position - use 3-parameter version below instead

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
            // SPRITE CONTROL
            // ================================================================

            let set_sprite_flip_x = scope.create_function_mut(|_, flip: bool| {
                if let Some(sprite) = world_cell.borrow_mut().sprites.get_mut(&entity) {
                    sprite.flip_x = flip;
                }
                Ok(())
            })?;
            globals.set("set_sprite_flip_x", set_sprite_flip_x)?;

            let set_sprite_flip_y = scope.create_function_mut(|_, flip: bool| {
                if let Some(sprite) = world_cell.borrow_mut().sprites.get_mut(&entity) {
                    sprite.flip_y = flip;
                }
                Ok(())
            })?;
            globals.set("set_sprite_flip_y", set_sprite_flip_y)?;

            let get_sprite_flip_x = scope.create_function(|_, ()| {
                if let Some(sprite) = world_cell.borrow().sprites.get(&entity) {
                    Ok(Some(sprite.flip_x))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_sprite_flip_x", get_sprite_flip_x)?;

            let get_sprite_flip_y = scope.create_function(|_, ()| {
                if let Some(sprite) = world_cell.borrow().sprites.get(&entity) {
                    Ok(Some(sprite.flip_y))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_sprite_flip_y", get_sprite_flip_y)?;

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
            // ENTITY QUERIES (for camera follow, etc.)
            // ================================================================
            
            // Get all entities
            let get_all_entities = scope.create_function(|lua, ()| {
                let entities: Vec<Entity> = world_cell.borrow().transforms.keys().copied().collect();
                let table = lua.create_table()?;
                for (i, ent) in entities.iter().enumerate() {
                    table.set(i + 1, *ent)?;
                }
                Ok(table)
            })?;
            globals.set("get_all_entities", get_all_entities)?;
            
            // Get tags for an entity (returns array of tag strings)
            let get_tags = scope.create_function(|lua, query_entity: Entity| {
                let table = lua.create_table()?;
                if let Some(tag) = world_cell.borrow().tags.get(&query_entity) {
                    let tag_str = match tag {
                        EntityTag::Player => "Player",
                        EntityTag::Item => "Item",
                    };
                    table.set(1, tag_str)?;
                }
                Ok(table)
            })?;
            globals.set("get_tags", get_tags)?;
            
            // Get position of another entity (separate function name)
            let get_position_of = scope.create_function(|lua, query_entity: Entity| {
                if let Some(transform) = world_cell.borrow().transforms.get(&query_entity) {
                    let table = lua.create_table()?;
                    table.set("x", transform.position[0])?;
                    table.set("y", transform.position[1])?;
                    table.set("z", transform.position[2])?;
                    Ok(Some(table))
                } else {
                    Ok(None)
                }
            })?;
            // Note: get_position() already exists for current entity
            // This is get_position_of(entity) for querying other entities
            globals.set("get_position_of", get_position_of)?;
            
            // Set position with z parameter
            let set_position_xyz = scope.create_function_mut(|_, (x, y, z): (f32, f32, f32)| {
                if let Some(transform) = world_cell.borrow_mut().transforms.get_mut(&entity) {
                    transform.position[0] = x;
                    transform.position[1] = y;
                    transform.position[2] = z;
                }
                Ok(())
            })?;
            // Override set_position to accept 3 parameters
            globals.set("set_position", set_position_xyz)?;
            
            // Get velocity of another entity (separate function to avoid conflict)
            let get_velocity_of = scope.create_function(|lua, query_entity: Entity| {
                if let Some(vel) = world_cell.borrow().velocities.get(&query_entity) {
                    let table = lua.create_table()?;
                    table.set("x", vel.0)?;
                    table.set("y", vel.1)?;
                    Ok(Some(table))
                } else if let Some(rb) = world_cell.borrow().rigidbodies.get(&query_entity) {
                    let table = lua.create_table()?;
                    table.set("x", rb.velocity.0)?;
                    table.set("y", rb.velocity.1)?;
                    Ok(Some(table))
                } else {
                    Ok(None)
                }
            })?;
            globals.set("get_velocity_of", get_velocity_of)?;

            // ================================================================
            // UTILITY FUNCTIONS
            // ================================================================

            let get_delta_time = scope.create_function(|_, ()| {
                Ok(dt)
            })?;
            globals.set("get_delta_time", get_delta_time)?;

            let print_log = scope.create_function_mut(|_, msg: String| {
                log_callback_cell.borrow_mut()(format!("[Lua] {}", msg));
                Ok(())
            })?;
            globals.set("log", print_log)?;

            // ================================================================
            // DEBUG DRAW (Unity/Unreal style)
            // ================================================================
            
            let debug_lines_ref = &self.debug_lines;
            
            // debug_draw_line(start_x, start_y, start_z, end_x, end_y, end_z, r, g, b, a, duration)
            let debug_draw_line = scope.create_function_mut(move |_, args: (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)| {
                let (sx, sy, sz, ex, ey, ez, r, g, b, a, duration) = args;
                debug_lines_ref.borrow_mut().push(DebugLine {
                    start: [sx, sy, sz],
                    end: [ex, ey, ez],
                    color: [r, g, b, a],
                    duration,
                });
                Ok(())
            })?;
            globals.set("debug_draw_line", debug_draw_line)?;
            
            // Simplified version for 2D: debug_draw_line_2d(start_x, start_y, end_x, end_y, r, g, b, duration)
            let debug_lines_ref2 = &self.debug_lines;
            let debug_draw_line_2d = scope.create_function_mut(move |_, args: (f32, f32, f32, f32, f32, f32, f32, f32)| {
                let (sx, sy, ex, ey, r, g, b, duration) = args;
                debug_lines_ref2.borrow_mut().push(DebugLine {
                    start: [sx, sy, 0.0],
                    end: [ex, ey, 0.0],
                    color: [r, g, b, 1.0],
                    duration,
                });
                Ok(())
            })?;
            globals.set("debug_draw_line_2d", debug_draw_line_2d)?;
            
            // Helper: debug_draw_ray(origin_x, origin_y, origin_z, dir_x, dir_y, dir_z, length, r, g, b, duration)
            let debug_lines_ref3 = &self.debug_lines;
            let debug_draw_ray = scope.create_function_mut(move |_, args: (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)| {
                let (ox, oy, oz, dx, dy, dz, length, r, g, b, duration) = args;
                let end_x = ox + dx * length;
                let end_y = oy + dy * length;
                let end_z = oz + dz * length;
                debug_lines_ref3.borrow_mut().push(DebugLine {
                    start: [ox, oy, oz],
                    end: [end_x, end_y, end_z],
                    color: [r, g, b, 1.0],
                    duration,
                });
                Ok(())
            })?;
            globals.set("debug_draw_ray", debug_draw_ray)?;

            // ================================================================
            // PHYSICS - GROUND CHECK (Rapier support)
            // ================================================================
            
            // Get ground state from script engine (set by engine with Rapier result)
            let is_grounded = self.ground_states.get(&entity).copied().unwrap_or(false);
            globals.set("is_grounded_rapier", is_grounded)?;

            // ================================================================
            // UI SYSTEM API
            // ================================================================
            
            // UI functions - queue commands for engine to process
            let ui_commands_clone = self.ui_commands.clone();
            
            // UI.load_prefab(path) -> boolean
            let ui_load_prefab = scope.create_function(move |_, path: String| {
                ui_commands_clone.borrow_mut().push(UICommand::LoadPrefab { path });
                Ok(true)
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.activate_prefab(path, instance_name) -> boolean
            let ui_activate_prefab = scope.create_function(move |_, (path, instance_name): (String, String)| {
                ui_commands_clone.borrow_mut().push(UICommand::ActivatePrefab { path, instance_name });
                Ok(true)
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.deactivate_prefab(instance_name)
            let ui_deactivate_prefab = scope.create_function(move |_, instance_name: String| {
                ui_commands_clone.borrow_mut().push(UICommand::DeactivatePrefab { instance_name });
                Ok(())
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.set_text(element_path, text)
            let ui_set_text = scope.create_function(move |_, (element_path, text): (String, String)| {
                ui_commands_clone.borrow_mut().push(UICommand::SetText { element_path, text });
                Ok(())
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.set_image_fill(element_path, fill_amount)
            let ui_set_image_fill = scope.create_function(move |_, (element_path, fill_amount): (String, f32)| {
                ui_commands_clone.borrow_mut().push(UICommand::SetImageFill { element_path, fill_amount });
                Ok(())
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.set_color(element_path, {r, g, b, a})
            let ui_set_color = scope.create_function(move |_, (element_path, color): (String, Table)| {
                let r = color.get::<_, f32>("r").unwrap_or(1.0);
                let g = color.get::<_, f32>("g").unwrap_or(1.0);
                let b = color.get::<_, f32>("b").unwrap_or(1.0);
                let a = color.get::<_, f32>("a").unwrap_or(1.0);
                ui_commands_clone.borrow_mut().push(UICommand::SetColor { element_path, r, g, b, a });
                Ok(())
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.show_element(element_path)
            let ui_show_element = scope.create_function(move |_, element_path: String| {
                ui_commands_clone.borrow_mut().push(UICommand::ShowElement { element_path });
                Ok(())
            })?;
            
            let ui_commands_clone = self.ui_commands.clone();
            // UI.hide_element(element_path)
            let ui_hide_element = scope.create_function(move |_, element_path: String| {
                ui_commands_clone.borrow_mut().push(UICommand::HideElement { element_path });
                Ok(())
            })?;
            
            // Create UI table and add functions
            let ui_table = lua.create_table()?;
            ui_table.set("load_prefab", ui_load_prefab)?;
            ui_table.set("activate_prefab", ui_activate_prefab)?;
            ui_table.set("deactivate_prefab", ui_deactivate_prefab)?;
            ui_table.set("set_text", ui_set_text)?;
            ui_table.set("set_image_fill", ui_set_image_fill)?;
            ui_table.set("set_color", ui_set_color)?;
            ui_table.set("show_element", ui_show_element)?;
            ui_table.set("hide_element", ui_hide_element)?;
            globals.set("UI", ui_table)?;

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
                        ecs::ScriptParameter::Entity(Some(e)) => globals.set(name.as_str(), *e)?,
                        ecs::ScriptParameter::Entity(None) => globals.set(name.as_str(), mlua::Nil)?,
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
