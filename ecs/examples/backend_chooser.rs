//! ECS Backend Chooser Example
//!
//! Demonstrates how to choose and switch between different ECS backends.

use ecs::{
    EcsBackendType, DynamicWorld, BenchmarkRunner,
    Transform, Sprite, Collider
};
use ecs::traits::{EcsWorld, ComponentAccess};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ECS Backend Chooser Example");
    println!("===========================\n");
    
    // List available backends
    println!("Available ECS backends:");
    for (i, backend) in EcsBackendType::available_backends().iter().enumerate() {
        let default_marker = if *backend == EcsBackendType::default() { " (default)" } else { "" };
        println!("  {}. {}{}", i + 1, backend, default_marker);
        println!("     {}", backend.description());
        
        let perf_info = backend.performance_info();
        println!("     Performance: Entity Spawn: {}, Query: {}, Parallel: {}",
            perf_info.entity_spawn_speed,
            perf_info.query_speed,
            perf_info.parallel_systems
        );
        println!();
    }
    
    // Demonstrate basic usage with each backend
    for backend_type in EcsBackendType::available_backends() {
        println!("Testing {} backend:", backend_type);
        println!("{}", "-".repeat(30));
        
        match demonstrate_backend(backend_type) {
            Ok(_) => println!("✓ Backend test successful\n"),
            Err(e) => println!("✗ Backend test failed: {}\n", e),
        }
    }
    
    // Run a quick benchmark comparison
    println!("Quick Benchmark Comparison:");
    println!("==========================");
    
    let runner = BenchmarkRunner::new(100, 10); // Small numbers for demo
    
    for backend_type in EcsBackendType::available_backends() {
        print!("Benchmarking {}... ", backend_type);
        
        match runner.bench_entity_spawn(backend_type, 1000) {
            Ok(result) => {
                println!("{:.0} entities/sec", result.operations_per_second);
            },
            Err(e) => {
                println!("Failed: {}", e);
            }
        }
    }
    
    // Demonstrate backend switching
    println!("\nBackend Switching Demo:");
    println!("======================");
    
    let mut world = DynamicWorld::new(EcsBackendType::default())?;
    println!("Started with: {}", world.backend_type());
    
    // Add some entities
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    world.set_parent(entity2, Some(entity1))?;
    
    println!("Created {} entities", world.entity_count());
    
    // Switch to a different backend (this clears the world)
    let available_backends = EcsBackendType::available_backends();
    if available_backends.len() > 1 {
        let new_backend = available_backends.iter()
            .find(|&&b| b != world.backend_type())
            .copied()
            .unwrap_or(EcsBackendType::default());
        
        println!("Switching to: {}", new_backend);
        world.switch_backend(new_backend)?;
        println!("New backend: {}", world.backend_type());
        println!("Entity count after switch: {}", world.entity_count());
    }
    
    Ok(())
}

fn demonstrate_backend(backend_type: EcsBackendType) -> Result<(), Box<dyn std::error::Error>> {
    let mut world = DynamicWorld::new(backend_type)?;
    
    // Create some entities
    let player = world.spawn();
    let enemy = world.spawn();
    let item = world.spawn();
    
    println!("  Created {} entities", world.entity_count());
    
    // Test hierarchy
    world.set_parent(item, Some(player))?;
    println!("  Set up parent-child relationship");
    
    // Verify hierarchy
    assert_eq!(world.get_parent(item), Some(player));
    assert_eq!(world.get_children(player), vec![item]);
    println!("  Hierarchy verified");
    
    // Test entity lifecycle
    world.despawn(enemy)?;
    println!("  Despawned enemy entity");
    
    assert!(!world.is_alive(enemy));
    assert!(world.is_alive(player));
    assert!(world.is_alive(item));
    
    // Test recursive despawn (despawning parent should despawn children)
    world.despawn(player)?;
    assert!(!world.is_alive(player));
    assert!(!world.is_alive(item)); // Should be despawned with parent
    
    println!("  Recursive despawn verified");
    
    // Test world clear
    let _new_entity = world.spawn();
    world.clear();
    assert_eq!(world.entity_count(), 0);
    println!("  World clear verified");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_backends() {
        for backend_type in EcsBackendType::available_backends() {
            demonstrate_backend(backend_type).unwrap();
        }
    }
}