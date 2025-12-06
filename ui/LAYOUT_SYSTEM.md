# UI Layout System

The Layout System provides automatic arrangement of UI elements using three layout types: Horizontal, Vertical, and Grid layouts.

## Overview

The layout system automatically positions and sizes child UI elements based on layout group components attached to their parent. This eliminates the need for manual positioning and makes UIs responsive to size changes.

## Features

- **Horizontal Layout**: Arranges children in a horizontal row
- **Vertical Layout**: Arranges children in a vertical column
- **Grid Layout**: Arranges children in a grid with configurable rows/columns
- **Padding**: Adds space around the layout container
- **Spacing**: Adds space between child elements
- **Alignment**: Controls how children are aligned within their allocated space
- **Force Expand**: Makes children expand to fill available space
- **Child Control**: Controls whether the layout manages child sizes
- **Ignore Layout**: Children can opt-out of layout with the `ignore_layout` flag

## Layout Types

### Horizontal Layout

Arranges children in a horizontal row from left to right.

```rust
use ui::{HorizontalLayoutGroup, Alignment};
use glam::Vec4;

let mut layout = HorizontalLayoutGroup::default();
layout.spacing = 10.0;                          // 10px between children
layout.padding = Vec4::new(5.0, 5.0, 5.0, 5.0); // 5px padding on all sides
layout.child_alignment = Alignment::MiddleCenter;
layout.child_force_expand_width = false;        // Don't expand children
layout.child_force_expand_height = true;        // Fill parent height
layout.child_control_width = false;             // Keep child widths
layout.child_control_height = true;             // Control child heights
```

**Properties:**
- `spacing`: Space between adjacent children (horizontal)
- `padding`: Space around the layout (left, bottom, right, top)
- `child_alignment`: How children are aligned vertically
- `child_force_expand_width`: Expand children to fill available width
- `child_force_expand_height`: Expand children to fill available height
- `child_control_width`: Whether layout controls child widths
- `child_control_height`: Whether layout controls child heights

### Vertical Layout

Arranges children in a vertical column from top to bottom.

```rust
use ui::{VerticalLayoutGroup, Alignment};
use glam::Vec4;

let mut layout = VerticalLayoutGroup::default();
layout.spacing = 10.0;                          // 10px between children
layout.padding = Vec4::new(5.0, 5.0, 5.0, 5.0); // 5px padding on all sides
layout.child_alignment = Alignment::UpperCenter;
layout.child_force_expand_width = true;         // Fill parent width
layout.child_force_expand_height = false;       // Don't expand children
layout.child_control_width = true;              // Control child widths
layout.child_control_height = false;            // Keep child heights
```

**Properties:**
- `spacing`: Space between adjacent children (vertical)
- `padding`: Space around the layout (left, bottom, right, top)
- `child_alignment`: How children are aligned horizontally
- `child_force_expand_width`: Expand children to fill available width
- `child_force_expand_height`: Expand children to fill available height
- `child_control_width`: Whether layout controls child widths
- `child_control_height`: Whether layout controls child heights

### Grid Layout

Arranges children in a grid with configurable cell size and constraints.

```rust
use ui::{GridLayoutGroup, GridConstraint, Corner, Axis, Alignment};
use glam::{Vec2, Vec4};

let mut layout = GridLayoutGroup::default();
layout.cell_size = Vec2::new(100.0, 100.0);     // 100x100 cells
layout.spacing = Vec2::new(10.0, 10.0);         // 10px spacing
layout.padding = Vec4::new(5.0, 5.0, 5.0, 5.0); // 5px padding
layout.constraint = GridConstraint::FixedColumnCount;
layout.constraint_count = 3;                     // 3 columns
layout.start_corner = Corner::UpperLeft;         // Start from top-left
layout.start_axis = Axis::Horizontal;            // Fill horizontally first
layout.child_alignment = Alignment::MiddleCenter;
```

**Properties:**
- `cell_size`: Size of each grid cell
- `spacing`: Space between cells (horizontal, vertical)
- `padding`: Space around the layout (left, bottom, right, top)
- `constraint`: How grid dimensions are determined
  - `Flexible`: Fit as many columns as possible
  - `FixedColumnCount`: Fixed number of columns
  - `FixedRowCount`: Fixed number of rows
- `constraint_count`: Number of columns/rows (for fixed constraints)
- `start_corner`: Which corner to start placing children
  - `UpperLeft`, `UpperRight`, `LowerLeft`, `LowerRight`
- `start_axis`: Which direction to fill first
  - `Horizontal`: Fill rows first
  - `Vertical`: Fill columns first
- `child_alignment`: How children are aligned within cells

## Alignment Options

The `Alignment` enum controls how children are positioned within their allocated space:

```rust
pub enum Alignment {
    UpperLeft,      // Top-left corner
    UpperCenter,    // Top-center
    UpperRight,     // Top-right corner
    MiddleLeft,     // Middle-left
    MiddleCenter,   // Center
    MiddleRight,    // Middle-right
    LowerLeft,      // Bottom-left corner
    LowerCenter,    // Bottom-center
    LowerRight,     // Bottom-right corner
}
```

## Usage Example

```rust
use std::collections::HashMap;
use ui::{
    LayoutSystem, RectTransform, UIElement,
    HorizontalLayoutGroup, Alignment,
};
use glam::{Vec2, Vec4};

// Create layout system
let mut layout_system = LayoutSystem::new();

// Create parent with horizontal layout
let parent = 1;
let mut parent_rt = RectTransform::default();
parent_rt.rect = ui::Rect::new(0.0, 0.0, 400.0, 100.0);
parent_rt.size_delta = Vec2::new(400.0, 100.0);

let mut layout = HorizontalLayoutGroup::default();
layout.spacing = 10.0;
layout.padding = Vec4::new(5.0, 5.0, 5.0, 5.0);
layout.child_alignment = Alignment::MiddleCenter;

// Create children
let children = vec![2, 3, 4];
let mut rect_transforms = HashMap::new();
let mut ui_elements = HashMap::new();
let mut horizontal_layouts = HashMap::new();
let mut children_map = HashMap::new();

rect_transforms.insert(parent, parent_rt);
ui_elements.insert(parent, UIElement::default());
horizontal_layouts.insert(parent, layout);

for &child in &children {
    let mut child_rt = RectTransform::default();
    child_rt.size_delta = Vec2::new(80.0, 60.0);
    rect_transforms.insert(child, child_rt);
    ui_elements.insert(child, UIElement::default());
}

children_map.insert(parent, children);

// Apply layout
layout_system.update_layouts(
    &mut rect_transforms,
    &ui_elements,
    &horizontal_layouts,
    &HashMap::new(),
    &HashMap::new(),
    &children_map,
);

// Children are now positioned automatically!
```

## Ignoring Layout

Individual children can opt-out of layout by setting the `ignore_layout` flag:

```rust
let mut ui_element = UIElement::default();
ui_element.ignore_layout = true;
```

This is useful for:
- Absolutely positioned overlays
- Custom positioned elements
- Elements that manage their own positioning

## Integration with ECS

The layout system integrates with the ECS architecture:

1. Layout components are attached to parent entities
2. The layout system queries for entities with layout components
3. Child entities are positioned based on their parent's layout
4. RectTransforms are updated with new positions and sizes

## Performance Considerations

- Layout calculations are performed only when needed
- The system filters out children with `ignore_layout = true`
- Dirty flagging ensures efficient updates
- Layout cache (future optimization) will reduce redundant calculations

## Requirements Validated

This implementation validates the following requirements:

- **5.1**: Horizontal layout arranges children in a row with spacing
- **5.2**: Vertical layout arranges children in a column with spacing
- **5.3**: Grid layout arranges children in a grid with cell size and spacing
- **5.4**: Layout padding is applied to all edges
- **5.5**: Child alignment controls positioning within allocated space
- **5.6**: Force expand makes children fill available space
- **5.7**: Layout recalculates when called (dirty flagging for future optimization)

## Future Enhancements

- Content size fitter (auto-size parent to fit children)
- Aspect ratio fitter (maintain aspect ratio)
- Layout element component (per-child layout preferences)
- Min/max size constraints
- Flexible width/height (weighted distribution)
- Layout animation (smooth transitions)
