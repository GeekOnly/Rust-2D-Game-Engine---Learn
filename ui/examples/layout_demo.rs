//! Layout system demonstration
//!
//! This example demonstrates the three layout types:
//! - Horizontal Layout
//! - Vertical Layout  
//! - Grid Layout

use std::collections::HashMap;
use ui::{
    RectTransform, UIElement, LayoutSystem,
    HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup,
    Alignment, Corner, Axis, GridConstraint,
};
use glam::{Vec2, Vec4};

type Entity = u64;

fn main() {
    println!("=== UI Layout System Demo ===\n");

    // Demo 1: Horizontal Layout
    demo_horizontal_layout();
    
    // Demo 2: Vertical Layout
    demo_vertical_layout();
    
    // Demo 3: Grid Layout
    demo_grid_layout();
}

fn demo_horizontal_layout() {
    println!("--- Horizontal Layout Demo ---");
    
    let mut system = LayoutSystem::new();
    let mut rect_transforms = HashMap::new();
    let mut ui_elements = HashMap::new();
    let mut horizontal_layouts = HashMap::new();
    let mut children_map = HashMap::new();

    // Create parent container (400x100)
    let parent = 1;
    let mut parent_rt = RectTransform::default();
    parent_rt.rect = ui::Rect::new(0.0, 0.0, 400.0, 100.0);
    parent_rt.size_delta = Vec2::new(400.0, 100.0);
    rect_transforms.insert(parent, parent_rt);
    ui_elements.insert(parent, UIElement::default());

    // Create 3 children (80x60 each)
    let children = vec![2, 3, 4];
    for &child in &children {
        let mut child_rt = RectTransform::default();
        child_rt.size_delta = Vec2::new(80.0, 60.0);
        rect_transforms.insert(child, child_rt);
        ui_elements.insert(child, UIElement::default());
    }

    // Configure horizontal layout with spacing
    let mut layout = HorizontalLayoutGroup::default();
    layout.spacing = 20.0;
    layout.padding = Vec4::new(10.0, 10.0, 10.0, 10.0); // left, bottom, right, top
    layout.child_alignment = Alignment::MiddleCenter;
    layout.child_force_expand_width = false;
    layout.child_force_expand_height = false;
    horizontal_layouts.insert(parent, layout);
    children_map.insert(parent, children.clone());

    // Apply layout
    system.update_layouts(
        &mut rect_transforms,
        &ui_elements,
        &horizontal_layouts,
        &HashMap::new(),
        &HashMap::new(),
        &children_map,
    );

    // Print results
    println!("Parent: 400x100 at (0, 0)");
    println!("Layout: Horizontal with 20px spacing, 10px padding");
    for (i, &child) in children.iter().enumerate() {
        let rt = rect_transforms.get(&child).unwrap();
        println!("  Child {}: {}x{} at ({:.1}, {:.1})", 
            i + 1, rt.rect.width, rt.rect.height, rt.rect.x, rt.rect.y);
    }
    println!();
}

fn demo_vertical_layout() {
    println!("--- Vertical Layout Demo ---");
    
    let mut system = LayoutSystem::new();
    let mut rect_transforms = HashMap::new();
    let mut ui_elements = HashMap::new();
    let mut vertical_layouts = HashMap::new();
    let mut children_map = HashMap::new();

    // Create parent container (200x400)
    let parent = 1;
    let mut parent_rt = RectTransform::default();
    parent_rt.rect = ui::Rect::new(0.0, 0.0, 200.0, 400.0);
    parent_rt.size_delta = Vec2::new(200.0, 400.0);
    rect_transforms.insert(parent, parent_rt);
    ui_elements.insert(parent, UIElement::default());

    // Create 4 children (180x80 each)
    let children = vec![2, 3, 4, 5];
    for &child in &children {
        let mut child_rt = RectTransform::default();
        child_rt.size_delta = Vec2::new(180.0, 80.0);
        rect_transforms.insert(child, child_rt);
        ui_elements.insert(child, UIElement::default());
    }

    // Configure vertical layout
    let mut layout = VerticalLayoutGroup::default();
    layout.spacing = 10.0;
    layout.padding = Vec4::new(10.0, 10.0, 10.0, 10.0);
    layout.child_alignment = Alignment::UpperCenter;
    layout.child_force_expand_width = false;
    layout.child_force_expand_height = false;
    vertical_layouts.insert(parent, layout);
    children_map.insert(parent, children.clone());

    // Apply layout
    system.update_layouts(
        &mut rect_transforms,
        &ui_elements,
        &HashMap::new(),
        &vertical_layouts,
        &HashMap::new(),
        &children_map,
    );

    // Print results
    println!("Parent: 200x400 at (0, 0)");
    println!("Layout: Vertical with 10px spacing, 10px padding");
    for (i, &child) in children.iter().enumerate() {
        let rt = rect_transforms.get(&child).unwrap();
        println!("  Child {}: {}x{} at ({:.1}, {:.1})", 
            i + 1, rt.rect.width, rt.rect.height, rt.rect.x, rt.rect.y);
    }
    println!();
}

fn demo_grid_layout() {
    println!("--- Grid Layout Demo ---");
    
    let mut system = LayoutSystem::new();
    let mut rect_transforms = HashMap::new();
    let mut ui_elements = HashMap::new();
    let mut grid_layouts = HashMap::new();
    let mut children_map = HashMap::new();

    // Create parent container (400x400)
    let parent = 1;
    let mut parent_rt = RectTransform::default();
    parent_rt.rect = ui::Rect::new(0.0, 0.0, 400.0, 400.0);
    parent_rt.size_delta = Vec2::new(400.0, 400.0);
    rect_transforms.insert(parent, parent_rt);
    ui_elements.insert(parent, UIElement::default());

    // Create 9 children for a 3x3 grid
    let children: Vec<Entity> = (2..=10).collect();
    for &child in &children {
        let mut child_rt = RectTransform::default();
        child_rt.size_delta = Vec2::new(80.0, 80.0);
        rect_transforms.insert(child, child_rt);
        ui_elements.insert(child, UIElement::default());
    }

    // Configure grid layout (3 columns)
    let mut layout = GridLayoutGroup::default();
    layout.cell_size = Vec2::new(80.0, 80.0);
    layout.spacing = Vec2::new(10.0, 10.0);
    layout.padding = Vec4::new(20.0, 20.0, 20.0, 20.0);
    layout.constraint = GridConstraint::FixedColumnCount;
    layout.constraint_count = 3;
    layout.start_corner = Corner::UpperLeft;
    layout.start_axis = Axis::Horizontal;
    layout.child_alignment = Alignment::UpperLeft;
    grid_layouts.insert(parent, layout);
    children_map.insert(parent, children.clone());

    // Apply layout
    system.update_layouts(
        &mut rect_transforms,
        &ui_elements,
        &HashMap::new(),
        &HashMap::new(),
        &grid_layouts,
        &children_map,
    );

    // Print results
    println!("Parent: 400x400 at (0, 0)");
    println!("Layout: 3x3 Grid with 80x80 cells, 10px spacing, 20px padding");
    for (i, &child) in children.iter().enumerate() {
        let rt = rect_transforms.get(&child).unwrap();
        let row = i / 3;
        let col = i % 3;
        println!("  Cell [{},{}]: {}x{} at ({:.1}, {:.1})", 
            row, col, rt.rect.width, rt.rect.height, rt.rect.x, rt.rect.y);
    }
    println!();
}
