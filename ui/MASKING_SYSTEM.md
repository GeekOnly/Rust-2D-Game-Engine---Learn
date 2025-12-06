# UI Masking System

The UI Masking System provides comprehensive clipping and masking capabilities for UI elements, supporting both stencil-based rectangular clipping and sprite alpha-based masking.

## Overview

The masking system allows you to:
- Clip UI elements to rectangular regions (stencil-based clipping)
- Mask UI elements using sprite alpha channels (alpha-based masking)
- Nest masks for complex clipping effects (intersection of all parent masks)
- Control whether mask graphics are rendered or hidden

## Components

### UIMask Component

The `UIMask` component is attached to UI elements that should act as masks:

```rust
pub struct UIMask {
    /// Whether to show the mask graphic
    pub show_mask_graphic: bool,
    
    /// Whether to use sprite alpha for masking
    pub use_sprite_alpha: bool,
}
```

**Properties:**
- `show_mask_graphic`: When `true`, the mask's visual representation (sprite/image) is rendered. When `false`, only the clipping effect is applied.
- `use_sprite_alpha`: When `true`, uses the sprite's alpha channel for masking. When `false`, uses rectangular bounds clipping.

## Usage

### Basic Rectangular Masking

To create a simple rectangular mask:

```rust
use ui::{UIMask, MaskingSystem, Rect};

let mut masking_system = MaskingSystem::new();

// Create a mask component
let mask = UIMask {
    show_mask_graphic: true,  // Show the mask graphic
    use_sprite_alpha: false,  // Use rectangular clipping
};

// Define the mask bounds
let bounds = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 200.0,
};

// Register the mask
let mask_entity = 1;
masking_system.register_mask(mask_entity, mask, bounds);

// Push the mask to activate it
masking_system.push_mask(mask_entity);

// Now all child elements will be clipped to this region
// Check if a point is masked
let point = Vec2::new(100.0, 100.0);
let is_masked = masking_system.is_point_masked(point);

// Pop the mask when done
masking_system.pop_mask();
```

### Sprite Alpha Masking

To use sprite alpha for masking:

```rust
use ui::{UIMask, MaskingSystem, Rect};

let mut masking_system = MaskingSystem::new();

// Create a mask component with alpha masking enabled
let mask = UIMask {
    show_mask_graphic: false,  // Don't show the mask graphic
    use_sprite_alpha: true,    // Use sprite alpha for masking
};

// Define the mask bounds
let bounds = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 200.0,
};

// Register the mask with a sprite texture
let mask_entity = 1;
let sprite_texture = Some("mask_sprite.png".to_string());
masking_system.register_mask_with_sprite(mask_entity, mask, bounds, sprite_texture);

// Push the mask to activate it
masking_system.push_mask(mask_entity);

// Check if a point passes the alpha mask
let point = Vec2::new(100.0, 100.0);
let alpha_threshold = 0.5;
let passes = masking_system.check_alpha_mask(mask_entity, point, alpha_threshold);
```

### Nested Masks

Masks can be nested to create complex clipping regions. The final clipping region is the intersection of all active masks:

```rust
use ui::{UIMask, MaskingSystem, Rect};

let mut masking_system = MaskingSystem::new();

// First mask: 0,0 to 200,200
let mask1 = UIMask::default();
let bounds1 = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 200.0,
};
masking_system.register_mask(1, mask1, bounds1);
masking_system.push_mask(1);

// Second mask: 50,50 to 150,150 (inside first)
let mask2 = UIMask::default();
let bounds2 = Rect {
    x: 50.0,
    y: 50.0,
    width: 100.0,
    height: 100.0,
};
masking_system.register_mask(2, mask2, bounds2);
masking_system.push_mask(2);

// The effective clipping region is now 50,50 to 150,150
let intersection = masking_system.get_intersection_bounds().unwrap();
assert_eq!(intersection.x, 50.0);
assert_eq!(intersection.y, 50.0);
assert_eq!(intersection.width, 100.0);
assert_eq!(intersection.height, 100.0);

// Pop masks in reverse order
masking_system.pop_mask();
masking_system.pop_mask();
```

### Controlling Mask Graphic Visibility

You can control whether the mask graphic itself is rendered:

```rust
use ui::{UIMask, MaskingSystem, Rect};

let mut masking_system = MaskingSystem::new();

// Create a mask that clips but doesn't show its graphic
let mask = UIMask {
    show_mask_graphic: false,  // Hide the mask graphic
    use_sprite_alpha: false,
};

let bounds = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 200.0,
};

let mask_entity = 1;
masking_system.register_mask(mask_entity, mask, bounds);

// Check if the mask graphic should be rendered
let should_render = masking_system.should_render_mask_graphic(mask_entity);
assert!(!should_render);

// Change the setting dynamically
masking_system.set_show_mask_graphic(mask_entity, true);
assert!(masking_system.should_render_mask_graphic(mask_entity));
```

## API Reference

### MaskingSystem

The main system for managing masks.

#### Methods

- `new() -> Self`: Create a new masking system
- `register_mask(entity, mask, bounds)`: Register a mask component
- `register_mask_with_sprite(entity, mask, bounds, sprite_texture)`: Register a mask with sprite texture for alpha masking
- `unregister_mask(entity)`: Unregister a mask component
- `update_mask_bounds(entity, bounds)`: Update a mask's bounds
- `push_mask(entity) -> Option<MaskState>`: Push a mask onto the stack (activate it)
- `pop_mask() -> Option<(Entity, MaskState)>`: Pop a mask from the stack (deactivate it)
- `get_active_mask() -> Option<&MaskState>`: Get the current active mask
- `get_active_masks() -> &[(Entity, MaskState)]`: Get all active masks in the stack
- `get_intersection_bounds() -> Option<Rect>`: Get the intersection of all active mask bounds
- `is_point_masked(point) -> bool`: Check if a point is masked by active masks
- `is_rect_completely_masked(rect) -> bool`: Check if a rectangle is completely masked
- `is_rect_partially_masked(rect) -> bool`: Check if a rectangle is partially masked
- `clip_rect(rect) -> Option<Rect>`: Get the clipped bounds of a rectangle
- `get_stencil_depth() -> usize`: Get the current stencil depth (number of active masks)
- `clear_stack()`: Clear all active masks
- `should_render_mask_graphic(entity) -> bool`: Check if a mask should render its graphic
- `set_show_mask_graphic(entity, show)`: Set whether a mask should render its graphic
- `uses_sprite_alpha(entity) -> bool`: Check if a mask uses sprite alpha
- `get_sprite_texture(entity) -> Option<&str>`: Get the sprite texture for alpha masking
- `update_sprite_texture(entity, sprite_texture)`: Update the sprite texture for a mask
- `check_alpha_mask(entity, point, alpha_threshold) -> bool`: Check if a point passes alpha masking
- `point_to_uv(entity, point) -> Option<Vec2>`: Get UV coordinates for a point within a mask's bounds
- `get_mask_state(entity) -> Option<&MaskState>`: Get the mask state for rendering decisions

## Integration with Rendering

The masking system is designed to integrate with the UI rendering pipeline:

1. **Before rendering a masked region:**
   - Call `push_mask(entity)` to activate the mask
   - Use `get_mask_state(entity)` to get rendering information
   - Set up stencil buffer or alpha testing based on mask settings

2. **While rendering child elements:**
   - Use `is_rect_completely_masked(rect)` to cull completely masked elements
   - Use `clip_rect(rect)` to get clipped bounds for partially masked elements
   - Use `check_alpha_mask(entity, point, threshold)` for alpha-based masking

3. **After rendering a masked region:**
   - Call `pop_mask()` to deactivate the mask

4. **Rendering the mask graphic:**
   - Check `should_render_mask_graphic(entity)` to decide if the mask graphic should be rendered
   - If `true`, render the mask's sprite/image
   - If `false`, skip rendering the mask graphic (but still apply clipping)

## Requirements Validation

This implementation satisfies the following requirements:

- **Requirement 11.1**: Stencil-based clipping clips all child elements to mask bounds
- **Requirement 11.2**: Sprite alpha masking uses sprite alpha channel for clipping
- **Requirement 11.3**: Nested masks apply intersection of all parent masks
- **Requirement 11.4**: Show Mask Graphic option controls mask graphic rendering
- **Requirement 11.5**: Hide Mask Graphic maintains clipping while hiding graphic

## Performance Considerations

- **Stencil Depth**: The system supports up to 255 nested masks (8-bit stencil buffer)
- **Mask Stack**: Use `clear_stack()` at the start of each frame to reset the mask stack
- **Culling**: Use `is_rect_completely_masked()` to cull elements that are completely outside all masks
- **Clipping**: Use `clip_rect()` to reduce the rendering area for partially masked elements

## Examples

See the test cases in `ui/src/rendering/mask_system.rs` for comprehensive examples of all masking features.
