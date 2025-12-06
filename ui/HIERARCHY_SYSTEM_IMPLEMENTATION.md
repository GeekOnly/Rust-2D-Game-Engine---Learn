# UI Hierarchy System Implementation

## Overview

This document describes the implementation of the UI hierarchy system for the in-game UI system (Task 4).

## Completed Tasks

### Task 4.1: Create UIElement base component ✅

The `UIElement` component was already implemented in `ui/src/components/ui_element.rs`. It provides the base properties for all UI elements:

- **raycast_target**: Whether the element can receive raycast events
- **blocks_raycasts**: Whether the element blocks raycasts to elements behind it
- **z_order**: Rendering order within siblings (higher renders on top)
- **color**: Color tint applied to the element
- **alpha**: Alpha transparency (0.0 = fully transparent, 1.0 = fully opaque)
- **interactable**: Whether the element is interactable
- **ignore_layout**: Whether to ignore parent layout groups
- **canvas_entity**: Cached canvas entity reference (updated by hierarchy system)

### Task 4.2: Implement hierarchy propagation systems ✅

Created a new `UIHierarchySystem` in `ui/src/hierarchy_system.rs` that handles:

#### 1. Canvas Entity Cache Propagation

The system propagates canvas entity references down the hierarchy so each UI element knows which canvas it belongs to. This is essential for:
- Determining render order across multiple canvases
- Applying canvas-specific settings (scale factor, render mode)
- Event routing and raycasting

**Method**: `update_canvas_cache()`

#### 2. Visibility Propagation

The system calculates effective visibility for each UI element based on:
- The element's own active state
- The visibility of all ancestor elements
- A UI element is only visible if it and all its ancestors are active

This implements **Property 9: Hierarchical visibility propagation** from the design document.

**Method**: `update_visibility()`

#### 3. Render Order Calculation

The system determines the correct render order for UI elements based on:
1. Canvas sort order (for elements in different canvases)
2. Hierarchy depth (parents render before children)
3. Sibling index (order within the same parent)
4. Z-order (within siblings)

This implements **Property 11: Sibling index determines render order** from the design document.

**Method**: `get_render_order()`

#### 4. Destruction Propagation

When a UI element is destroyed, the system identifies all descendant entities that should also be destroyed. This ensures:
- No orphaned UI elements remain in the hierarchy
- Resources are properly cleaned up
- The hierarchy remains consistent

This implements **Property 10: Hierarchical destruction propagation** from the design document.

**Method**: `get_descendants_for_destruction()`

## Integration with ECS

The hierarchy system integrates seamlessly with the existing ECS architecture:

- Uses the `parents` and `children` maps from the ECS world
- Works with the existing `active` state tracking
- Complements the `RectTransformSystem` for transform propagation
- Provides data for the rendering pipeline

## Testing

All systems are thoroughly tested with unit tests:

1. **test_canvas_cache_propagation**: Verifies canvas entity references propagate correctly
2. **test_visibility_propagation**: Tests visibility calculation with various hierarchy configurations
3. **test_render_order_sibling_index**: Validates render order based on z-order
4. **test_descendants_for_destruction**: Tests complete hierarchy destruction
5. **test_descendants_for_destruction_partial**: Tests partial hierarchy destruction

All tests pass successfully.

## Requirements Validation

This implementation satisfies the following requirements from the design document:

- **Requirement 3.1**: Transform propagation (handled by RectTransformSystem + UIHierarchySystem)
- **Requirement 3.2**: Parent movement updates children (RectTransformSystem)
- **Requirement 3.3**: Parent scale applies to children (RectTransformSystem)
- **Requirement 3.4**: Parent rotation applies to children (RectTransformSystem)
- **Requirement 3.5**: Parent hiding hides children (UIHierarchySystem.update_visibility)
- **Requirement 3.6**: Parent destruction destroys children (UIHierarchySystem.get_descendants_for_destruction)
- **Requirement 3.7**: Sibling index determines render order (UIHierarchySystem.get_render_order)
- **Requirement 6.1**: Raycast target marking (UIElement.raycast_target)
- **Requirement 6.7**: Raycast blocking (UIElement.blocks_raycasts)

## Usage Example

```rust
use ui::{UIHierarchySystem, UIElement, Canvas};
use std::collections::HashMap;

// Create the hierarchy system
let mut hierarchy_system = UIHierarchySystem::new();

// Update canvas cache (call once per frame or when hierarchy changes)
hierarchy_system.update_canvas_cache(
    &mut ui_elements,
    &canvases,
    &parents,
);

// Calculate visibility (call once per frame)
let visibility = hierarchy_system.update_visibility(
    &ui_elements,
    &parents,
    &active_states,
);

// Get render order (call once per frame before rendering)
let render_order = hierarchy_system.get_render_order(
    &ui_elements,
    &canvases,
    &parents,
    &children,
);

// When destroying an entity
let descendants = hierarchy_system.get_descendants_for_destruction(
    entity_to_destroy,
    &children,
);
// Destroy entity and all descendants
```

## Next Steps

The following tasks remain in the implementation plan:

- Task 5: Implement core UI components (UIImage, UIText, UIButton, UIPanel)
- Task 6: Implement layout system
- Task 7: Checkpoint - Ensure all tests pass
- Task 8: Implement event system
- And more...

The hierarchy system provides the foundation for these upcoming features by ensuring proper parent-child relationships, visibility, and render order.
