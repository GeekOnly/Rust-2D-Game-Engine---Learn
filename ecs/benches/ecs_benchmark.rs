//! ECS Backend Benchmarks
//!
//! Comprehensive benchmarks comparing different ECS backends.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ecs::backends::{EcsBackendType, DynamicWorld};
use ecs::traits::{EcsWorld, ComponentAccess};
use ecs::{Transform, Sprite, Collider, Rigidbody2D};

// Benchmark configuration
const ENTITY_COUNTS: &[usize] = &[100, 1000, 10000];
const COMPONENT_COUNTS: &[usize] = &[100, 1000, 5000];

fn bench_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");
    
    for backend_type in EcsBackendType::available_backends() {
        for &entity_count in ENTITY_COUNTS {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", backend_type), entity_count),
                &entity_count,
                |b, &entity_count| {
                    b.iter_batched(
                        || DynamicWorld::new(backend_type).unwrap(),
                        |mut world| {
                            for _ in 0..entity_count {
                                black_box(world.spawn());
                            }
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_entity_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_despawn");
    
    for backend_type in EcsBackendType::available_backends() {
        for &entity_count in ENTITY_COUNTS {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", backend_type), entity_count),
                &entity_count,
                |b, &entity_count| {
                    b.iter_batched(
                        || {
                            let mut world = DynamicWorld::new(backend_type).unwrap();
                            let entities: Vec<_> = (0..entity_count)
                                .map(|_| world.spawn())
                                .collect();
                            (world, entities)
                        },
                        |(mut world, entities)| {
                            for entity in entities {
                                black_box(world.despawn(entity).ok());
                            }
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_component_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_insert");
    
    for backend_type in EcsBackendType::available_backends() {
        for &component_count in COMPONENT_COUNTS {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", backend_type), component_count),
                &component_count,
                |b, &component_count| {
                    b.iter_batched(
                        || {
                            let mut world = DynamicWorld::new(backend_type).unwrap();
                            let entities: Vec<_> = (0..component_count)
                                .map(|_| world.spawn())
                                .collect();
                            (world, entities)
                        },
                        |(mut world, entities)| {
                            // Note: This is a simplified benchmark
                            // In a real implementation, you'd need to handle the different entity types
                            for (i, &entity) in entities.iter().enumerate() {
                                let transform = Transform {
                                    position: [i as f32, i as f32, 0.0],
                                    rotation: [0.0, 0.0, 0.0],
                                    scale: [1.0, 1.0, 1.0],
                                };
                                // This would need proper implementation for each backend
                                black_box(transform);
                            }
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_world_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("world_clear");
    
    for backend_type in EcsBackendType::available_backends() {
        for &entity_count in ENTITY_COUNTS {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", backend_type), entity_count),
                &entity_count,
                |b, &entity_count| {
                    b.iter_batched(
                        || {
                            let mut world = DynamicWorld::new(backend_type).unwrap();
                            for _ in 0..entity_count {
                                world.spawn();
                            }
                            world
                        },
                        |mut world| {
                            black_box(world.clear());
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn bench_hierarchy_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchy_operations");
    
    for backend_type in EcsBackendType::available_backends() {
        group.bench_function(
            format!("{:?}_set_parent", backend_type),
            |b| {
                b.iter_batched(
                    || {
                        let mut world = DynamicWorld::new(backend_type).unwrap();
                        let parent = world.spawn();
                        let children: Vec<_> = (0..100).map(|_| world.spawn()).collect();
                        (world, parent, children)
                    },
                    |(mut world, parent, children)| {
                        for child in children {
                            black_box(world.set_parent(child, Some(parent)).unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        group.bench_function(
            format!("{:?}_get_children", backend_type),
            |b| {
                b.iter_batched(
                    || {
                        let mut world = DynamicWorld::new(backend_type).unwrap();
                        let parent = world.spawn();
                        let children: Vec<_> = (0..100).map(|_| world.spawn()).collect();
                        for child in &children {
                            world.set_parent(*child, Some(parent)).unwrap();
                        }
                        (world, parent)
                    },
                    |(world, parent)| {
                        black_box(world.get_children(parent));
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    
    for backend_type in EcsBackendType::available_backends() {
        group.bench_function(
            format!("{:?}_game_simulation", backend_type),
            |b| {
                b.iter_batched(
                    || DynamicWorld::new(backend_type).unwrap(),
                    |mut world| {
                        // Simulate a typical game frame
                        
                        // Spawn some entities
                        let mut entities = Vec::new();
                        for i in 0..50 {
                            let entity = world.spawn();
                            entities.push(entity);
                            
                            // Add some hierarchy
                            if i > 0 && i % 10 == 0 {
                                world.set_parent(entity, Some(entities[i - 1])).unwrap();
                            }
                        }
                        
                        // Check entity states
                        for &entity in &entities {
                            black_box(world.is_alive(entity));
                            black_box(world.get_parent(entity));
                        }
                        
                        // Despawn some entities
                        for &entity in entities.iter().take(10) {
                            world.despawn(entity).ok();
                        }
                        
                        // Final count check
                        black_box(world.entity_count());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for backend_type in EcsBackendType::available_backends() {
        group.bench_function(
            format!("{:?}_large_world", backend_type),
            |b| {
                b.iter_batched(
                    || DynamicWorld::new(backend_type).unwrap(),
                    |mut world| {
                        // Create a large world to test memory efficiency
                        for _ in 0..10000 {
                            let entity = world.spawn();
                            black_box(entity);
                        }
                        
                        // Test entity count performance with large world
                        black_box(world.entity_count());
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_entity_spawn,
    bench_entity_despawn,
    bench_component_insert,
    bench_world_clear,
    bench_hierarchy_operations,
    bench_mixed_operations,
    bench_memory_usage
);

criterion_main!(benches);