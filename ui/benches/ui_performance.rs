//! UI System Performance Benchmarks
//!
//! This benchmark suite measures the performance of key UI system operations
//! to ensure the new system meets or exceeds the legacy HUD system performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ui::*;
use glam::Vec2;

/// Benchmark RectTransform calculations
fn bench_rect_transform_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("RectTransform");
    
    // Benchmark anchored positioning
    group.bench_function("anchored_position", |b| {
        let mut rt = RectTransform::anchored(Vec2::new(0.5, 0.5), Vec2::new(100.0, 50.0), Vec2::new(200.0, 100.0));
        b.iter(|| {
            rt.anchored_position = black_box(Vec2::new(150.0, 75.0));
            black_box(&rt);
        });
    });
    
    // Benchmark stretched sizing
    group.bench_function("stretched_sizing", |b| {
        let mut rt = RectTransform::stretched(
            Vec2::new(0.1, 0.1),
            Vec2::new(0.9, 0.9),
            [10.0, 10.0, 10.0, 10.0].into()
        );
        b.iter(|| {
            rt.size_delta = black_box(Vec2::new(20.0, 20.0));
            black_box(&rt);
        });
    });
    
    // Benchmark contains_point (raycasting)
    group.bench_function("contains_point", |b| {
        let rt = RectTransform {
            rect: Rect { x: 0.0, y: 0.0, width: 200.0, height: 100.0 },
            ..Default::default()
        };
        let point = Vec2::new(100.0, 50.0);
        b.iter(|| {
            black_box(rt.contains_point(black_box(point)));
        });
    });
    
    group.finish();
}

/// Benchmark Canvas scaling calculations
fn bench_canvas_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Canvas Scaling");
    
    for resolution in &[(1920, 1080), (2560, 1440), (3840, 2160)] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", resolution.0, resolution.1)),
            resolution,
            |b, &(width, height)| {
                let mut scaler = CanvasScaler {
                    mode: ScaleMode::ScaleWithScreenSize,
                    reference_resolution: (1920.0, 1080.0),
                    match_width_or_height: 0.5,
                    reference_dpi: 96.0,
                    min_scale: 0.5,
                    max_scale: 2.0,
                    scale_factor: 1.0,
                };
                
                b.iter(|| {
                    // Simulate scale factor calculation
                    let screen_size = (width as f32, height as f32);
                    let ref_size = scaler.reference_resolution;
                    let width_scale = screen_size.0 / ref_size.0;
                    let height_scale = screen_size.1 / ref_size.1;
                    let scale = width_scale.min(height_scale);
                    scaler.scale_factor = black_box(scale.clamp(scaler.min_scale, scaler.max_scale));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark UI element hierarchy operations
fn bench_hierarchy_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hierarchy");
    
    // Benchmark creating a simple hierarchy
    group.bench_function("create_simple_hierarchy", |b| {
        b.iter(|| {
            let child1 = UIPrefabElement {
                name: "Child1".to_string(),
                rect_transform: RectTransform::default(),
                ui_element: UIElement::default(),
                image: None,
                text: None,
                button: None,
                panel: None,
                slider: None,
                toggle: None,
                dropdown: None,
                input_field: None,
                scroll_view: None,
                mask: None,
                horizontal_layout: None,
                vertical_layout: None,
                grid_layout: None,
                children: vec![],
            };
            
            let child2 = UIPrefabElement {
                name: "Child2".to_string(),
                rect_transform: RectTransform::default(),
                ui_element: UIElement::default(),
                image: None,
                text: None,
                button: None,
                panel: None,
                slider: None,
                toggle: None,
                dropdown: None,
                input_field: None,
                scroll_view: None,
                mask: None,
                horizontal_layout: None,
                vertical_layout: None,
                grid_layout: None,
                children: vec![],
            };
            
            let root = UIPrefabElement {
                name: "Root".to_string(),
                rect_transform: RectTransform::default(),
                ui_element: UIElement::default(),
                image: None,
                text: None,
                button: None,
                panel: None,
                slider: None,
                toggle: None,
                dropdown: None,
                input_field: None,
                scroll_view: None,
                mask: None,
                horizontal_layout: None,
                vertical_layout: None,
                grid_layout: None,
                children: vec![child1, child2],
            };
            black_box(root);
        });
    });
    
    group.finish();
}

/// Benchmark layout calculations
fn bench_layout_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Layout");
    
    // Benchmark horizontal layout with varying child counts
    for child_count in &[5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("horizontal_layout", child_count),
            child_count,
            |b, &count| {
                let layout = HorizontalLayoutGroup {
                    padding: [10.0, 10.0, 10.0, 10.0].into(),
                    spacing: 5.0,
                    child_alignment: Alignment::MiddleCenter,
                    child_force_expand_width: false,
                    child_force_expand_height: false,
                    child_control_width: true,
                    child_control_height: true,
                };
                
                b.iter(|| {
                    // Simulate layout calculation
                    let available_width = 800.0 - layout.padding.x - layout.padding.z;
                    let total_spacing = layout.spacing * (count - 1) as f32;
                    let child_width = (available_width - total_spacing) / count as f32;
                    black_box(child_width);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark prefab serialization/deserialization
fn bench_prefab_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("Prefab Serialization");
    
    let prefab = UIPrefab {
        name: "TestPrefab".to_string(),
        root: UIPrefabElement {
            name: "Root".to_string(),
            rect_transform: RectTransform::default(),
            ui_element: UIElement::default(),
            image: None,
            text: Some(UIText {
                text: "Hello World".to_string(),
                font: "default".to_string(),
                font_size: 18.0,
                color: [1.0, 1.0, 1.0, 1.0],
                alignment: TextAlignment::MiddleCenter,
                horizontal_overflow: OverflowMode::Wrap,
                vertical_overflow: OverflowMode::Truncate,
                rich_text: false,
                line_spacing: 1.0,
                best_fit: false,
                best_fit_min_size: 10.0,
                best_fit_max_size: 40.0,
            }),
            button: None,
            panel: None,
            slider: None,
            toggle: None,
            dropdown: None,
            input_field: None,
            scroll_view: None,
            mask: None,
            horizontal_layout: None,
            vertical_layout: None,
            grid_layout: None,
            children: vec![],
        },
    };
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&prefab).unwrap();
            black_box(json);
        });
    });
    
    let json = serde_json::to_string(&prefab).unwrap();
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            let prefab: UIPrefab = serde_json::from_str(&json).unwrap();
            black_box(prefab);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_rect_transform_calculations,
    bench_canvas_scaling,
    bench_hierarchy_operations,
    bench_layout_calculations,
    bench_prefab_serialization
);
criterion_main!(benches);
