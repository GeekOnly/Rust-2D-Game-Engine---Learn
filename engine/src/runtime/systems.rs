// Runtime systems module
// This module contains the core systems that run during game execution

use anyhow::Result;
use ecs::World;
use physics::PhysicsWorld;
use script::ScriptEngine;
use input::InputSystem;

// Re-export specific functions if needed, but GameSystems struct is the main entry point
pub use super::render_system;
pub use super::physics_system;
pub use super::script_system;

pub struct GameSystems {
    pub physics_world: PhysicsWorld,
    pub script_engine: ScriptEngine,
}

impl GameSystems {
    pub fn new() -> Result<Self> {
        Ok(Self {
            physics_world: PhysicsWorld::new(),
            script_engine: ScriptEngine::new()?,
        })
    }

    pub fn update(&mut self, world: &mut World, input: &InputSystem, dt: f32) {
        // 1. Update Scripts (Game Logic)
        // Scripts might modify transform or velocity, so they run before physics
        script_system::update_scripts(&mut self.script_engine, world, input, dt);

        // 2. Update Physics
        // Physics applies forces and resolves collisions
        physics_system::update_physics(&mut self.physics_world, world, dt);
    }
}