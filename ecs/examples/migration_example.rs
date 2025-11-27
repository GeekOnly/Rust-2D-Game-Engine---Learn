//! Migration Example: Custom ECS vs hecs
//!
//! This file shows side-by-side comparison of code patterns.
//! Run with: cargo run --example migration_example

// ============================================================================
// BEFORE: Custom HashMap ECS
// ============================================================================

#[cfg(feature = "show_custom_ecs")]
mod custom_ecs_example {
    use std::collections::HashMap;

    pub type Entity = u32;

    pub struct World {
        next_entity: Entity,
        transforms: HashMap<Entity, Transform>,
        sprites: HashMap<Entity, Sprite>,
        velocities: HashMap<Entity, (f32, f32)>,
    }

    #[derive(Clone)]
    pub struct Transform {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Clone)]
    pub struct Sprite {
        pub texture: String,
    }

    impl World {
        pub fn new() -> Self {
            Self {
                next_entity: 0,
                transforms: HashMap::new(),
                sprites: HashMap::new(),
                velocities: HashMap::new(),
            }
        }

        pub fn spawn(&mut self) -> Entity {
            let id = self.next_entity;
            self.next_entity += 1;
            id
        }
    }

    pub fn example_spawn(world: &mut World) {
        // Spawn player - components added separately
        let player = world.spawn();
        world.transforms.insert(player, Transform { x: 0.0, y: 0.0 });
        world.sprites.insert(player, Sprite {
            texture: "player.png".to_string(),
        });
        world.velocities.insert(player, (0.0, 0.0));

        println!("Custom ECS: Spawned player {:?}", player);
    }

    pub fn example_query(world: &World) {
        // Query entities with Transform + Sprite (slow - requires HashMap lookup)
        for (entity, transform) in &world.transforms {
            if let Some(sprite) = world.sprites.get(entity) {
                println!(
                    "Custom ECS: Entity {} at ({}, {}) with texture {}",
                    entity, transform.x, transform.y, sprite.texture
                );
            }
        }
    }

    pub fn example_update(world: &mut World, dt: f32) {
        // Update physics - requires collecting keys first to avoid borrow issues
        let entities: Vec<_> = world.velocities.keys().copied().collect();

        for entity in entities {
            if let (Some(&(vx, vy)), Some(transform)) = (
                world.velocities.get(&entity),
                world.transforms.get_mut(&entity),
            ) {
                transform.x += vx * dt;
                transform.y += vy * dt;
            }
        }
    }
}

// ============================================================================
// AFTER: hecs
// ============================================================================

#[cfg(feature = "show_hecs")]
mod hecs_example {
    use hecs::{World, Entity};

    #[derive(Clone)]
    pub struct Transform {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Clone)]
    pub struct Sprite {
        pub texture: String,
    }

    #[derive(Clone)]
    pub struct Velocity {
        pub vx: f32,
        pub vy: f32,
    }

    pub fn example_spawn(world: &mut World) {
        // Spawn player - components bundled together (faster!)
        let player = world.spawn((
            Transform { x: 0.0, y: 0.0 },
            Sprite {
                texture: "player.png".to_string(),
            },
            Velocity { vx: 0.0, vy: 0.0 },
        ));

        println!("hecs: Spawned player {:?}", player);
    }

    pub fn example_query(world: &World) {
        // Query entities with Transform + Sprite (fast - archetype-based!)
        for (entity, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
            println!(
                "hecs: Entity {:?} at ({}, {}) with texture {}",
                entity, transform.x, transform.y, sprite.texture
            );
        }
    }

    pub fn example_update(world: &mut World, dt: f32) {
        // Update physics - direct mutable iteration (no collecting needed!)
        for (_entity, (transform, velocity)) in world.query::<(&mut Transform, &Velocity)>().iter()
        {
            transform.x += velocity.vx * dt;
            transform.y += velocity.vy * dt;
        }
    }
}

// ============================================================================
// Main: Side-by-side Comparison
// ============================================================================

fn main() {
    println!("=".repeat(60));
    println!("ECS Migration Example: Custom HashMap vs hecs");
    println!("=".repeat(60));
    println!();

    #[cfg(feature = "show_custom_ecs")]
    {
        println!("--- CUSTOM HASHMAP ECS ---");
        let mut world = custom_ecs_example::World::new();
        custom_ecs_example::example_spawn(&mut world);
        custom_ecs_example::example_query(&world);
        custom_ecs_example::example_update(&mut world, 0.016);
        println!();
    }

    #[cfg(feature = "show_hecs")]
    {
        println!("--- HECS (ARCHETYPE-BASED) ---");
        let mut world = hecs::World::new();
        hecs_example::example_spawn(&mut world);
        hecs_example::example_query(&world);
        hecs_example::example_update(&mut world, 0.016);
        println!();
    }

    println!("=".repeat(60));
    println!("Key Differences:");
    println!("=".repeat(60));
    println!("1. Spawn:");
    println!("   Custom: Separate inserts (3 HashMap operations)");
    println!("   hecs:   Bundle spawn (1 archetype insert)");
    println!();
    println!("2. Query:");
    println!("   Custom: Iterate HashMap + lookup other components");
    println!("   hecs:   Direct archetype iteration (5-10x faster)");
    println!();
    println!("3. Mutation:");
    println!("   Custom: Collect keys to avoid borrow checker issues");
    println!("   hecs:   Direct mutable iteration");
    println!("=".repeat(60));
}
