use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ecs::{World, Transform, Sprite, Collider};

/// Benchmark: Spawning entities
fn bench_spawn_entities(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    black_box(world.spawn());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Inserting Transform components
fn bench_insert_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_transform");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();

                for &entity in &entities {
                    world.transforms.insert(entity, Transform::default());
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Inserting multiple components (Transform + Sprite + Collider)
fn bench_insert_multi_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_multi_component");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();

                for &entity in &entities {
                    world.transforms.insert(entity, Transform::default());
                    world.sprites.insert(entity, Sprite {
                        texture_id: "test".to_string(),
                        width: 32.0,
                        height: 32.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        billboard: false,
                    });
                    world.colliders.insert(entity, Collider {
                        width: 32.0,
                        height: 32.0,
                    });
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Querying single component (Transform)
fn bench_query_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_transform");

    for count in [100, 1000, 10000].iter() {
        // Setup world with entities
        let mut world = World::new();
        for _ in 0..*count {
            let entity = world.spawn();
            world.transforms.insert(entity, Transform::default());
        }

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _count| {
            b.iter(|| {
                let mut sum = 0.0;
                for (_entity, transform) in &world.transforms {
                    sum += transform.position[0];
                    sum += transform.position[1];
                    sum += transform.position[2];
                }
                black_box(sum);
            });
        });
    }

    group.finish();
}

/// Benchmark: Querying multiple components (Transform + Sprite)
fn bench_query_multi_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_multi_component");

    for count in [100, 1000, 10000].iter() {
        // Setup world with entities
        let mut world = World::new();
        for _ in 0..*count {
            let entity = world.spawn();
            world.transforms.insert(entity, Transform::default());
            world.sprites.insert(entity, Sprite {
                texture_id: "test".to_string(),
                width: 32.0,
                height: 32.0,
                color: [1.0, 1.0, 1.0, 1.0],
                billboard: false,
            });
        }

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _count| {
            b.iter(|| {
                let mut sum = 0.0;
                for (entity, transform) in &world.transforms {
                    if let Some(sprite) = world.sprites.get(&entity) {
                        sum += transform.position[0];
                        sum += sprite.width;
                    }
                }
                black_box(sum);
            });
        });
    }

    group.finish();
}

/// Benchmark: Mutating components (Transform position update)
fn bench_mutate_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutate_transform");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    let entity = world.spawn();
                    world.transforms.insert(entity, Transform::default());
                }

                // Mutate all transforms
                for (_entity, transform) in world.transforms.iter_mut() {
                    transform.position[0] += 1.0;
                    transform.position[1] += 1.0;
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Removing components
fn bench_remove_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_component");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<_> = (0..count).map(|_| {
                    let entity = world.spawn();
                    world.transforms.insert(entity, Transform::default());
                    entity
                }).collect();

                // Remove all transforms
                for &entity in &entities {
                    world.transforms.remove(&entity);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Despawning entities
fn bench_despawn_entities(c: &mut Criterion) {
    let mut group = c.benchmark_group("despawn");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                let entities: Vec<_> = (0..count).map(|_| {
                    let entity = world.spawn();
                    world.transforms.insert(entity, Transform::default());
                    world.sprites.insert(entity, Sprite {
                        texture_id: "test".to_string(),
                        width: 32.0,
                        height: 32.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        billboard: false,
                    });
                    entity
                }).collect();

                // Despawn all entities
                for &entity in &entities {
                    world.despawn(entity);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Realistic game scenario (spawning + querying + mutating)
fn bench_game_scenario(c: &mut Criterion) {
    c.bench_function("game_scenario_1000_entities_60_frames", |b| {
        b.iter(|| {
            let mut world = World::new();

            // Spawn 1000 entities with Transform, Sprite, Velocity
            for i in 0..1000 {
                let entity = world.spawn();
                world.transforms.insert(entity, Transform {
                    position: [i as f32, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                });
                world.sprites.insert(entity, Sprite {
                    texture_id: "test".to_string(),
                    width: 32.0,
                    height: 32.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    billboard: false,
                });
                world.velocities.insert(entity, (1.0, 0.0)); // (vx, vy) tuple
            }

            // Simulate 60 frames of updates (1 second at 60 FPS)
            for _frame in 0..60 {
                // Update physics: apply velocity to transform
                let entities_with_velocity: Vec<_> = world.velocities.keys().copied().collect();
                for entity in entities_with_velocity {
                    if let (Some(&(vx, vy)), Some(transform)) =
                        (world.velocities.get(&entity), world.transforms.get_mut(&entity)) {
                        transform.position[0] += vx * 0.016; // 60 FPS delta
                        transform.position[1] += vy * 0.016;
                    }
                }

                // Render query: iterate all entities with Transform + Sprite
                let mut render_count = 0;
                for (entity, transform) in &world.transforms {
                    if world.sprites.contains_key(&entity) {
                        black_box(transform.position);
                        render_count += 1;
                    }
                }
                black_box(render_count);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_spawn_entities,
    bench_insert_transform,
    bench_insert_multi_component,
    bench_query_transform,
    bench_query_multi_component,
    bench_mutate_transform,
    bench_remove_component,
    bench_despawn_entities,
    bench_game_scenario,
);

criterion_main!(benches);
