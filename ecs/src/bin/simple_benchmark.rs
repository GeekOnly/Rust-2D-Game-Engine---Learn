//! Simple ECS Benchmark - Core Operations Only
//!
//! Tests only the core ECS operations without dependencies on loaders

use std::time::{Duration, Instant};
use ecs::{EcsBackendType, DynamicWorld, BenchmarkRunner};
use ecs::traits::EcsWorld;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple ECS Backend Comparison");
    println!("================================\n");

    // Test available backends
    let backends = EcsBackendType::available_backends();
    println!("Available backends: {:?}\n", backends);

    for backend_type in backends {
        println!("🧪 Testing {} Backend:", backend_type);
        println!("{}", "-".repeat(40));
        
        match test_backend_performance(backend_type) {
            Ok(results) => {
                println!("  ✅ Entity Spawn: {:.1}M/sec", results.spawn_ops_per_sec / 1_000_000.0);
                println!("  ✅ Entity Despawn: {:.1}M/sec", results.despawn_ops_per_sec / 1_000_000.0);
                println!("  ✅ Hierarchy Ops: {:.1}M/sec", results.hierarchy_ops_per_sec / 1_000_000.0);
                println!("  ✅ Mixed Ops: {:.1}K/sec", results.mixed_ops_per_sec / 1000.0);
                println!("  ⏱️  Total Time: {:.2}ms", results.total_time.as_millis());
                
                let score = calculate_performance_score(&results);
                println!("  🏆 Score: {}/100", score);
            },
            Err(e) => {
                println!("  ❌ Failed: {}", e);
            }
        }
        println!();
    }

    // Detailed comparison
    println!("📊 Detailed Performance Comparison:");
    println!("===================================");
    
    let mut all_results = Vec::new();
    
    for backend_type in EcsBackendType::available_backends() {
        if let Ok(results) = test_backend_performance(backend_type) {
            all_results.push((backend_type, results));
        }
    }
    
    // Sort by overall performance score
    all_results.sort_by(|a, b| {
        let score_a = calculate_performance_score(&a.1);
        let score_b = calculate_performance_score(&b.1);
        score_b.cmp(&score_a)
    });
    
    println!("🏆 Performance Ranking:");
    for (i, (backend, results)) in all_results.iter().enumerate() {
        let medal = match i {
            0 => "🥇",
            1 => "🥈", 
            2 => "🥉",
            _ => "📊",
        };
        let score = calculate_performance_score(results);
        println!("  {} {}: {} points", medal, backend, score);
    }
    
    println!("\n💡 Recommendations:");
    println!("===================");
    
    if let Some((best_backend, best_results)) = all_results.first() {
        println!("🏆 Best Overall: {}", best_backend);
        println!("   - Entity Spawn: {:.1}M/sec", best_results.spawn_ops_per_sec / 1_000_000.0);
        println!("   - Use for: High-performance games, many entities");
        
        if all_results.len() > 1 {
            let (second_backend, second_results) = &all_results[1];
            println!("🥈 Good Alternative: {}", second_backend);
            println!("   - Entity Spawn: {:.1}M/sec", second_results.spawn_ops_per_sec / 1_000_000.0);
            println!("   - Use for: Balanced performance and simplicity");
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct BenchmarkResults {
    spawn_ops_per_sec: f64,
    despawn_ops_per_sec: f64,
    hierarchy_ops_per_sec: f64,
    mixed_ops_per_sec: f64,
    total_time: Duration,
}

fn test_backend_performance(backend_type: EcsBackendType) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Test entity spawn performance
    let spawn_result = {
        let mut world = DynamicWorld::new(backend_type)?;
        let iterations = 1000;
        let entities_per_iteration = 1000;
        
        let start = Instant::now();
        for _ in 0..iterations {
            let mut entities = Vec::new();
            for _ in 0..entities_per_iteration {
                entities.push(world.spawn());
            }
            // Clean up
            for entity in entities {
                let _ = world.despawn(entity);
            }
        }
        let duration = start.elapsed();
        
        (iterations * entities_per_iteration) as f64 / duration.as_secs_f64()
    };
    
    // Test entity despawn performance
    let despawn_result = {
        let mut world = DynamicWorld::new(backend_type)?;
        let iterations = 1000;
        let entities_per_iteration = 1000;
        
        let start = Instant::now();
        for _ in 0..iterations {
            // Spawn entities
            let mut entities = Vec::new();
            for _ in 0..entities_per_iteration {
                entities.push(world.spawn());
            }
            // Despawn entities (this is what we're measuring)
            for entity in entities {
                let _ = world.despawn(entity);
            }
        }
        let duration = start.elapsed();
        
        (iterations * entities_per_iteration) as f64 / duration.as_secs_f64()
    };
    
    // Test hierarchy operations
    let hierarchy_result = {
        let mut world = DynamicWorld::new(backend_type)?;
        let iterations = 1000;
        let children_per_iteration = 100;
        
        let start = Instant::now();
        for _ in 0..iterations {
            let parent = world.spawn();
            let mut children = Vec::new();
            
            // Create hierarchy
            for _ in 0..children_per_iteration {
                let child = world.spawn();
                let _ = world.set_parent(child, Some(parent));
                children.push(child);
            }
            
            // Test hierarchy queries
            let _ = world.get_children(parent);
            for child in &children {
                let _ = world.get_parent(*child);
            }
            
            // Clean up
            let _ = world.despawn(parent); // Should recursively despawn children
        }
        let duration = start.elapsed();
        
        (iterations * children_per_iteration * 2) as f64 / duration.as_secs_f64() // *2 for set_parent + get_parent
    };
    
    // Test mixed operations (simulated game frame)
    let mixed_result = {
        let mut world = DynamicWorld::new(backend_type)?;
        let iterations = 1000;
        
        let start = Instant::now();
        for _ in 0..iterations {
            // Spawn some entities
            let mut entities = Vec::new();
            for _ in 0..20 {
                entities.push(world.spawn());
            }
            
            // Create some hierarchy
            if entities.len() >= 2 {
                let _ = world.set_parent(entities[1], Some(entities[0]));
            }
            
            // Query operations
            for &entity in &entities {
                let _ = world.is_alive(entity);
            }
            
            // Despawn some entities
            for entity in entities.into_iter().take(10) {
                let _ = world.despawn(entity);
            }
        }
        let duration = start.elapsed();
        
        iterations as f64 / duration.as_secs_f64()
    };
    
    let total_time = start_time.elapsed();
    
    Ok(BenchmarkResults {
        spawn_ops_per_sec: spawn_result,
        despawn_ops_per_sec: despawn_result,
        hierarchy_ops_per_sec: hierarchy_result,
        mixed_ops_per_sec: mixed_result,
        total_time,
    })
}

fn calculate_performance_score(results: &BenchmarkResults) -> u32 {
    // Weighted scoring system
    let spawn_score = (results.spawn_ops_per_sec / 100_000.0).min(30.0) as u32;
    let despawn_score = (results.despawn_ops_per_sec / 100_000.0).min(25.0) as u32;
    let hierarchy_score = (results.hierarchy_ops_per_sec / 100_000.0).min(25.0) as u32;
    let mixed_score = (results.mixed_ops_per_sec / 1000.0).min(20.0) as u32;
    
    spawn_score + despawn_score + hierarchy_score + mixed_score
}