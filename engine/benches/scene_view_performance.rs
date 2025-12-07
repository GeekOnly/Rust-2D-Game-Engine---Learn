// Feature: scene-view-improvements, Task 12.1: Performance benchmarks
// Benchmarks for grid rendering, cache hit rate, and frame time consistency

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use glam::Vec2;
use engine::editor::{SceneCamera, grid::{InfiniteGrid, CameraState}};

/// Benchmark grid rendering time for various camera positions
fn bench_grid_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_rendering");
    
    // Test different zoom levels
    let zoom_levels = vec![0.1, 1.0, 5.0, 20.0, 100.0];
    
    for zoom in zoom_levels {
        group.bench_with_input(BenchmarkId::new("generate_geometry", zoom), &zoom, |b, &zoom| {
            let mut grid = InfiniteGrid::new();
            let camera = CameraState {
                position: Vec2::ZERO,
                rotation: 45.0,
                pitch: 30.0,
                zoom,
            };
            let viewport_size = Vec2::new(1920.0, 1080.0);
            
            b.iter(|| {
                black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
            });
        });
    }
    
    group.finish();
}

/// Benchmark cache hit rate
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");
    
    group.bench_function("cache_hit", |b| {
        let mut grid = InfiniteGrid::new();
        let camera = CameraState {
            position: Vec2::ZERO,
            rotation: 45.0,
            pitch: 30.0,
            zoom: 2.0,
        };
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        // Prime the cache
        grid.generate_geometry(&camera, viewport_size);
        
        b.iter(|| {
            // Should hit cache since camera hasn't moved significantly
            black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.bench_function("cache_miss", |b| {
        let mut grid = InfiniteGrid::new();
        let viewport_size = Vec2::new(1920.0, 1080.0);
        let mut position = Vec2::ZERO;
        
        b.iter(|| {
            // Move camera significantly to force cache miss
            position.x += 10.0;
            let camera = CameraState {
                position,
                rotation: 45.0,
                pitch: 30.0,
                zoom: 2.0,
            };
            black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.finish();
}

/// Benchmark frame time consistency
fn bench_frame_time_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_time_consistency");
    
    group.bench_function("static_camera", |b| {
        let mut grid = InfiniteGrid::new();
        let camera = CameraState {
            position: Vec2::ZERO,
            rotation: 45.0,
            pitch: 30.0,
            zoom: 2.0,
        };
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        b.iter(|| {
            black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.bench_function("moving_camera", |b| {
        let mut grid = InfiniteGrid::new();
        let viewport_size = Vec2::new(1920.0, 1080.0);
        let mut frame = 0;
        
        b.iter(|| {
            // Simulate camera movement
            let t = frame as f32 * 0.016; // 60 FPS
            let camera = CameraState {
                position: Vec2::new(t * 10.0, t * 5.0),
                rotation: 45.0 + t * 2.0,
                pitch: 30.0,
                zoom: 2.0,
            };
            frame += 1;
            black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.bench_function("zooming_camera", |b| {
        let mut grid = InfiniteGrid::new();
        let viewport_size = Vec2::new(1920.0, 1080.0);
        let mut frame = 0;
        
        b.iter(|| {
            // Simulate zoom changes
            let t = frame as f32 * 0.016; // 60 FPS
            let zoom = 2.0 + (t * 0.5).sin() * 1.5; // Oscillate between 0.5 and 3.5
            let camera = CameraState {
                position: Vec2::ZERO,
                rotation: 45.0,
                pitch: 30.0,
                zoom,
            };
            frame += 1;
            black_box(grid.generate_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.finish();
}

/// Benchmark line batching efficiency
fn bench_line_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_batching");
    
    group.bench_function("generate_batched_geometry", |b| {
        let mut grid = InfiniteGrid::new();
        let camera = CameraState {
            position: Vec2::ZERO,
            rotation: 45.0,
            pitch: 30.0,
            zoom: 2.0,
        };
        let viewport_size = Vec2::new(1920.0, 1080.0);
        
        b.iter(|| {
            black_box(grid.generate_batched_geometry(black_box(&camera), black_box(viewport_size)))
        });
    });
    
    group.finish();
}

/// Benchmark grid level calculation
fn bench_grid_level_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_level_calculation");
    
    let zoom_levels = vec![0.01, 0.1, 1.0, 10.0, 100.0];
    
    for zoom in zoom_levels {
        group.bench_with_input(BenchmarkId::new("calculate_grid_level", zoom), &zoom, |b, &zoom| {
            let grid = InfiniteGrid::new();
            
            b.iter(|| {
                black_box(grid.calculate_grid_level(black_box(zoom)))
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_grid_rendering,
    bench_cache_hit_rate,
    bench_frame_time_consistency,
    bench_line_batching,
    bench_grid_level_calculation
);
criterion_main!(benches);
