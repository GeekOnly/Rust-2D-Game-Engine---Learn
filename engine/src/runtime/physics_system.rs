// Physics system for runtime
use ecs::World;
use physics::PhysicsWorld;

pub fn update_physics(physics_world: &mut PhysicsWorld, world: &mut World, delta_time: f32) {
    physics_world.step(delta_time, world);
}