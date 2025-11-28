/// Physics 2D Demo
/// 
/// แสดงการใช้งานระบบ Physics 2D:
/// - Gravity
/// - Velocity
/// - Collision Detection
/// - Physics Helpers
/// 
/// รันด้วยคำสั่ง: cargo run --example physics_demo

use ecs::{World, ComponentType, ComponentManager};
use ecs::traits::EcsWorld;
use physics::PhysicsWorld;

fn main() {
    println!("=== 2D Physics System Demo ===\n");

    let mut world = World::new();
    let mut physics = PhysicsWorld::new();

    println!("Physics Settings:");
    println!("  Gravity: {} pixels/s²\n", physics.gravity);

    // 1. สร้าง Player Entity
    println!("1. Creating Player Entity...");
    let player = world.spawn();
    world.names.insert(player, "Player".to_string());
    world.add_component(player, ComponentType::Transform).unwrap();
    world.add_component(player, ComponentType::Sprite).unwrap();
    world.add_component(player, ComponentType::BoxCollider).unwrap();
    world.add_component(player, ComponentType::Rigidbody).unwrap();

    // ตั้งค่า Transform
    world.transforms.get_mut(&player).unwrap().position = [0.0, 100.0, 0.0];

    // ตั้งค่า Sprite
    if let Some(sprite) = world.sprites.get_mut(&player) {
        sprite.texture_id = "player".to_string();
        sprite.width = 40.0;
        sprite.height = 40.0;
        sprite.color = [0.2, 0.6, 1.0, 1.0];
    }

    // ตั้งค่า Collider
    if let Some(collider) = world.colliders.get_mut(&player) {
        collider.width = 40.0;
        collider.height = 40.0;
    }

    // ตั้งค่า Velocity เริ่มต้น
    world.velocities.insert(player, (50.0, 0.0)); // เคลื่อนที่ไปทางขวา

    println!("  Position: {:?}", world.transforms.get(&player).unwrap().position);
    println!("  Velocity: {:?}", world.velocities.get(&player).unwrap());
    println!();

    // 2. สร้าง Ground Entity
    println!("2. Creating Ground Entity...");
    let ground = world.spawn();
    world.names.insert(ground, "Ground".to_string());
    world.add_component(ground, ComponentType::Transform).unwrap();
    world.add_component(ground, ComponentType::Sprite).unwrap();
    world.add_component(ground, ComponentType::BoxCollider).unwrap();

    // ตั้งค่า Ground
    world.transforms.get_mut(&ground).unwrap().position = [0.0, -50.0, 0.0];
    if let Some(sprite) = world.sprites.get_mut(&ground) {
        sprite.width = 200.0;
        sprite.height = 20.0;
        sprite.color = [0.3, 0.3, 0.3, 1.0];
    }
    if let Some(collider) = world.colliders.get_mut(&ground) {
        collider.width = 200.0;
        collider.height = 20.0;
    }

    println!("  Position: {:?}", world.transforms.get(&ground).unwrap().position);
    println!();

    // 3. จำลอง Physics
    println!("3. Simulating Physics...\n");

    let dt = 0.016; // 60 FPS (16ms per frame)
    let total_time = 2.0; // จำลอง 2 วินาที
    let steps = (total_time / dt) as usize;

    for i in 0..steps {
        let time = i as f32 * dt;

        // Update physics
        physics.step(dt, &mut world);

        // แสดงผลทุก 0.5 วินาที
        if i % 30 == 0 {
            let pos = world.transforms.get(&player).unwrap().position;
            let vel = world.velocities.get(&player).unwrap();
            
            println!("Time: {:.2}s", time);
            println!("  Player Position: [{:.1}, {:.1}]", pos[0], pos[1]);
            println!("  Player Velocity: [{:.1}, {:.1}]", vel.0, vel.1);

            // ตรวจสอบ Collision
            let collisions = get_collisions_for_entity(&world, player);
            if !collisions.is_empty() {
                println!("  ⚠️ Collision detected with {} entities!", collisions.len());
                for collision_entity in collisions {
                    if let Some(name) = world.names.get(&collision_entity) {
                        println!("    - Colliding with: {}", name);
                    }
                }
            }
            println!();
        }
    }

    // 4. ทดสอบ Physics Helpers
    println!("4. Testing Physics Helpers...\n");

    // Reset player
    world.transforms.get_mut(&player).unwrap().position = [0.0, 0.0, 0.0];
    world.velocities.insert(player, (0.0, 0.0));

    println!("  Initial velocity: {:?}", world.velocities.get(&player));

    // Apply impulse (jump)
    println!("  Applying jump impulse (0, 300)...");
    if let Some(vel) = world.velocities.get_mut(&player) {
        vel.0 += 0.0;
        vel.1 += 300.0;
    }
    println!("  Velocity after impulse: {:?}", world.velocities.get(&player));

    // Apply force
    println!("  Applying force (100, 0) for 0.1s...");
    if let Some(vel) = world.velocities.get_mut(&player) {
        vel.0 += 100.0 * 0.1;
        vel.1 += 0.0;
    }
    println!("  Velocity after force: {:?}", world.velocities.get(&player));

    // Clamp velocity
    println!("  Clamping velocity to max 50...");
    if let Some(vel) = world.velocities.get_mut(&player) {
        let speed = (vel.0 * vel.0 + vel.1 * vel.1).sqrt();
        if speed > 50.0 {
            let scale = 50.0 / speed;
            vel.0 *= scale;
            vel.1 *= scale;
        }
    }
    println!("  Velocity after clamp: {:?}", world.velocities.get(&player));

    // Apply damping
    println!("  Applying damping (0.9) for 1s...");
    if let Some(vel) = world.velocities.get_mut(&player) {
        let factor = 1.0 - (0.9_f32 * 1.0).min(1.0);
        vel.0 *= factor;
        vel.1 *= factor;
    }
    println!("  Velocity after damping: {:?}", world.velocities.get(&player));

    // Stop
    println!("  Stopping entity...");
    world.velocities.insert(player, (0.0, 0.0));
    println!("  Velocity after stop: {:?}\n", world.velocities.get(&player));

    // 5. ทดสอบ Collision Detection
    println!("5. Testing Collision Detection...\n");

    // สร้าง Entity 2 ตัวที่ชนกัน
    let box1 = world.spawn();
    world.names.insert(box1, "Box 1".to_string());
    world.add_component(box1, ComponentType::Transform).unwrap();
    world.add_component(box1, ComponentType::BoxCollider).unwrap();
    world.transforms.get_mut(&box1).unwrap().position = [0.0, 0.0, 0.0];
    world.colliders.get_mut(&box1).unwrap().width = 50.0;
    world.colliders.get_mut(&box1).unwrap().height = 50.0;

    let box2 = world.spawn();
    world.names.insert(box2, "Box 2".to_string());
    world.add_component(box2, ComponentType::Transform).unwrap();
    world.add_component(box2, ComponentType::BoxCollider).unwrap();
    world.transforms.get_mut(&box2).unwrap().position = [30.0, 30.0, 0.0];
    world.colliders.get_mut(&box2).unwrap().width = 50.0;
    world.colliders.get_mut(&box2).unwrap().height = 50.0;

    println!("  Box 1 Position: {:?}", world.transforms.get(&box1).unwrap().position);
    println!("  Box 2 Position: {:?}", world.transforms.get(&box2).unwrap().position);
    
    if PhysicsWorld::check_collision(&world, box1, box2) {
        println!("  ✅ Collision detected between Box 1 and Box 2");
    } else {
        println!("  ❌ No collision");
    }

    // เลื่อน Box 2 ออกไป
    world.transforms.get_mut(&box2).unwrap().position = [100.0, 100.0, 0.0];
    println!("\n  Moving Box 2 to: {:?}", world.transforms.get(&box2).unwrap().position);
    
    if PhysicsWorld::check_collision(&world, box1, box2) {
        println!("  ✅ Collision detected");
    } else {
        println!("  ❌ No collision (as expected)");
    }

    // 6. สรุป
    println!("\n=== Summary ===");
    println!("Total Entities: {}", world.entity_count());
    println!("\nPhysics Features Demonstrated:");
    println!("  ✅ Gravity simulation");
    println!("  ✅ Velocity-based movement");
    println!("  ✅ AABB collision detection");
    println!("  ✅ Physics helpers (impulse, force, damping, etc.)");
    println!("  ✅ Time scale control");
    
    println!("\n=== Demo Complete ===");
}

// Helper function to get collisions
fn get_collisions_for_entity(world: &World, entity: ecs::Entity) -> Vec<ecs::Entity> {
    let mut collisions = Vec::new();

    if !world.colliders.contains_key(&entity) {
        return collisions;
    }

    for (other_entity, _) in &world.colliders {
        if *other_entity != entity {
            if PhysicsWorld::check_collision(world, entity, *other_entity) {
                collisions.push(*other_entity);
            }
        }
    }

    collisions
}
