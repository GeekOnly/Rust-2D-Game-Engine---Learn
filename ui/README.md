# UI System

A comprehensive in-game UI system for the XS 2D Game Engine, providing capabilities comparable to Unity's Canvas UI and Unreal Engine's UMG.

> **🔄 Migrating from Legacy HUD System?**  
> See the [Migration Guide](MIGRATION_GUIDE.md) for step-by-step instructions on converting your `.hud` files to the new `.uiprefab` format and updating your Lua scripts.

## Features

- **Canvas-based Rendering**: Multiple render modes (Screen Space Overlay, Screen Space Camera, World Space)
- **Flexible Anchoring**: RectTransform system for resolution-independent positioning
- **Rich Components**: Image, Text, Button, Panel, Slider, Toggle, Dropdown, Input Field, Scroll View
- **Automatic Layouts**: Horizontal, Vertical, and Grid layout groups
- **Event System**: Comprehensive input handling with callbacks
- **Animations**: Tween-based animations with easing functions
- **Masking & Clipping**: Stencil-based and sprite alpha masking
- **Lua Integration**: Full Lua API for dynamic UI creation
- **Prefab System**: Reusable UI templates
- **Styling**: Theme-based styling with smooth transitions

## Quick Start

### Basic Example

```rust
use ui::{
    Canvas, CanvasRenderMode, CanvasScaler, ScaleMode,
    RectTransform, UIElement, UIText, UIButton,
    Vec2,
};

// Create a canvas
let canvas = Canvas {
    render_mode: CanvasRenderMode::ScreenSpaceOverlay,
    sort_order: 0,
    scaler: CanvasScaler {
        mode: ScaleMode::ScaleWithScreenSize,
        reference_resolution: (1920.0, 1080.0),
        ..Default::default()
    },
    ..Default::default()
};

// Create a button
let button_transform = RectTransform::anchored(
    Vec2::new(0.5, 0.5),  // Center anchor
    Vec2::ZERO,            // No offset
    Vec2::new(200.0, 50.0) // Size
);

let button = UIButton {
    on_click: Some("on_button_clicked".to_string()),
    ..Default::default()
};

// Create button text
let text_transform = RectTransform::stretched(
    Vec2::ZERO,
    Vec2::ONE,
    [5.0, 5.0, 5.0, 5.0].into() // 5px padding
);

let text = UIText {
    text: "Click Me!".to_string(),
    font_size: 16.0,
    ..Default::default()
};
```

### Lua Example

```lua
-- Create a canvas
local canvas = ui_create_canvas({
    render_mode = "ScreenSpaceOverlay"
})

-- Create a button
local button = ui_create_button({
    parent = canvas,
    on_click = "on_button_clicked"
})
ui_set_position(button, 0, 0)
ui_set_size(button, 200, 50)

-- Create button text
local text = ui_create_text({
    parent = button,
    text = "Click Me!",
    font_size = 16
})

-- Event handler
function on_button_clicked()
    print("Button clicked!")
    ui_animate_scale({
        entity = button,
        to_x = 1.2,
        to_y = 1.2,
        duration = 0.2,
        easing = "EaseOutBack"
    })
end
```

## Core Concepts

### Canvas

The Canvas is the root container for all UI elements. It defines the rendering space and coordinate system.

**Render Modes:**
- **Screen Space Overlay**: UI rendered on top of everything in screen coordinates
- **Screen Space Camera**: UI rendered in screen space at a distance from camera
- **World Space**: UI rendered as part of the 3D world

**Canvas Scaler:**
- **Constant Pixel Size**: UI maintains pixel dimensions
- **Scale With Screen Size**: UI scales proportionally to reference resolution
- **Constant Physical Size**: UI maintains physical dimensions based on DPI

### RectTransform

RectTransform defines position, size, and anchoring of UI elements.

**Anchoring Modes:**

```rust
// Fixed position (anchor min == anchor max)
let fixed = RectTransform::anchored(
    Vec2::new(0.5, 0.5), // Center anchor
    Vec2::new(0.0, 100.0), // 100px above center
    Vec2::new(200.0, 50.0) // 200x50 size
);

// Stretched horizontally
let stretched_h = RectTransform::stretched(
    Vec2::new(0.0, 0.5), // Left-center to right-center
    Vec2::new(1.0, 0.5),
    Vec4::new(20.0, 0.0, 20.0, 0.0) // 20px margins
);

// Fully stretched (fills parent)
let stretched_full = RectTransform::stretched(
    Vec2::ZERO, // Bottom-left to top-right
    Vec2::ONE,
    Vec4::new(10.0, 10.0, 10.0, 10.0) // 10px margins
);
```

### UI Components

#### UIImage
Displays sprites or textures with support for:
- Simple rendering
- 9-slice scaling
- Tiled rendering
- Filled rendering (for progress bars)

#### UIText
Renders text with:
- Font selection and sizing
- 9-point alignment
- Overflow modes (wrap, truncate, overflow)
- Rich text support

#### UIButton
Interactive button with:
- Visual states (normal, highlighted, pressed, disabled)
- Transition types (color tint, sprite swap, animation)
- Click callbacks

#### UIPanel
Background panel with:
- 9-slice sprite support
- Padding configuration

#### Advanced Components
- **UISlider**: Value selection with draggable handle
- **UIToggle**: Boolean checkbox/toggle
- **UIDropdown**: Option selection from list
- **UIInputField**: Text input with validation
- **UIScrollView**: Scrollable content with viewport clipping

### Layout System

Automatic layout groups arrange children without manual positioning:

```rust
// Horizontal layout
let horizontal = HorizontalLayoutGroup {
    spacing: 10.0,
    padding: Vec4::new(10.0, 10.0, 10.0, 10.0),
    child_alignment: Alignment::MiddleCenter,
    ..Default::default()
};

// Vertical layout
let vertical = VerticalLayoutGroup {
    spacing: 10.0,
    child_force_expand_height: false,
    ..Default::default()
};

// Grid layout
let grid = GridLayoutGroup {
    cell_size: Vec2::new(100.0, 100.0),
    spacing: Vec2::new(10.0, 10.0),
    constraint: GridConstraint::FixedColumnCount,
    constraint_count: 3,
    ..Default::default()
};
```

### Event System

UI elements can respond to user input:

```rust
let element = UIElement {
    raycast_target: true,
    blocks_raycasts: true,
    interactable: true,
    ..Default::default()
};

// In Lua:
ui_on_click(button, "on_button_clicked")
ui_on_pointer_enter(button, "on_hover_start")
ui_on_pointer_exit(button, "on_hover_end")
ui_on_drag(element, "on_dragging")
```

**Event Types:**
- OnPointerEnter / OnPointerExit
- OnPointerDown / OnPointerUp
- OnPointerClick
- OnBeginDrag / OnDrag / OnEndDrag
- OnScroll

### Animations

Tween-based animations with easing functions:

```rust
let animation = UIAnimation {
    entity: 1,
    property: AnimatedProperty::AnchoredPosition,
    from: AnimationValue::Vec2(Vec2::ZERO),
    to: AnimationValue::Vec2(Vec2::new(100.0, 50.0)),
    duration: 1.0,
    easing: EasingFunction::EaseOutQuad,
    loop_mode: LoopMode::Once,
    ..Default::default()
};

// In Lua:
ui_animate_position({
    entity = element,
    to_x = 100,
    to_y = 50,
    duration = 1.0,
    easing = "EaseOutQuad"
})
```

**Animated Properties:**
- Position, Scale, Rotation
- Color, Alpha
- Size

**Easing Functions:**
- Linear
- Quad, Cubic, Quart, Quint, Sine
- Expo, Circ, Back, Elastic, Bounce
- In, Out, InOut variants

### Masking & Clipping

Control visibility of UI elements:

```rust
let mask = UIMask {
    show_mask_graphic: false,
    use_sprite_alpha: false,
};

// Stencil-based clipping: clips children to mask bounds
// Sprite alpha masking: uses sprite alpha channel
// Nested masks: intersection of all parent masks
```

### Prefab System

Create reusable UI templates:

```rust
let prefab = UIPrefab {
    name: "Button".to_string(),
    root: UIPrefabElement {
        name: "ButtonBackground".to_string(),
        rect_transform: RectTransform::anchored(
            Vec2::new(0.5, 0.5),
            Vec2::ZERO,
            Vec2::new(100.0, 40.0),
        ),
        button: Some(UIButton::default()),
        children: vec![
            UIPrefabElement {
                name: "ButtonText".to_string(),
                text: Some(UIText {
                    text: "Button".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    },
};

// Instantiate prefab
let instantiator = PrefabInstantiator::new();
let result = instantiator.instantiate(&prefab);
```

### Styling System

Apply consistent styling across UI elements:

```rust
let style = UIStyle {
    name: "dark".to_string(),
    primary_color: [0.2, 0.2, 0.3, 1.0],
    text_color: [0.9, 0.9, 0.9, 1.0],
    default_font_size: 16.0,
    ..Default::default()
};

let mut style_system = StyleSystem::new();
style_system.theme_mut().add_style(style);
style_system.set_active_style("dark".to_string());

// Apply style to elements
style_system.apply_style_to_button(&style, &mut button);
style_system.apply_style_to_text(&style, &mut text);
```

## Examples

The `examples/` directory contains comprehensive examples:

- **basic_ui.rs**: Canvas, Image, Text, Button with anchoring
- **advanced_ui.rs**: Scroll View, Slider, Toggle, Dropdown, Input Field, animations, events
- **layout_demo.rs**: Horizontal, Vertical, and Grid layouts
- **prefab_demo.rs**: Prefab creation and instantiation
- **style_demo.rs**: Styling and theme system
- **lua_ui_example.lua**: Complete Lua API demonstration

Run examples with:
```bash
cargo run --example basic_ui --manifest-path ui/Cargo.toml
cargo run --example advanced_ui --manifest-path ui/Cargo.toml
cargo run --example layout_demo --manifest-path ui/Cargo.toml
```

## Lua API

The UI system provides a comprehensive Lua API for dynamic UI creation. See [LUA_API.md](LUA_API.md) for complete documentation.

**Key Functions:**
- `ui_create_canvas()` - Create canvas
- `ui_create_image()`, `ui_create_text()`, `ui_create_button()`, `ui_create_panel()` - Create elements
- `ui_set_position()`, `ui_set_size()`, `ui_set_color()` - Set properties
- `ui_animate_position()`, `ui_animate_scale()`, `ui_animate_color()` - Animate properties
- `ui_on_click()`, `ui_on_pointer_enter()`, `ui_on_drag()` - Register event callbacks

## Architecture

The UI system is built on top of the ECS (Entity Component System) architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                     Game Application                         │
│                    (Lua Scripts / Rust)                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    UI System API                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Canvas       │  │ UI Builder   │  │ Event        │      │
│  │ Manager      │  │              │  │ Dispatcher   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    ECS World (ecs crate)                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  UI Components:                                       │   │
│  │  - Canvas, RectTransform, UIElement                  │   │
│  │  - UIImage, UIText, UIButton, UIPanel                │   │
│  │  - Layout Groups, Scroll View, Mask                  │   │
│  │  - UISlider, UIToggle, UIDropdown, UIInputField     │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rendering Pipeline                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ UI Batch     │  │ Text         │  │ Clipping &   │      │
│  │ Builder      │  │ Renderer     │  │ Masking      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Systems

The UI system consists of several specialized systems:

- **CanvasSystem**: Manages canvases and resolution changes
- **RectTransformSystem**: Calculates world positions and sizes
- **UIHierarchySystem**: Propagates transforms, visibility, and destruction
- **LayoutSystem**: Arranges children in layout groups
- **ScrollViewSystem**: Handles scrolling and viewport clipping
- **SliderSystem**, **ToggleSystem**, **DropdownSystem**, **InputFieldSystem**: Manage advanced components
- **UIRaycastSystem**: Performs raycasting for input events
- **UIInputHandler**: Processes mouse/touch input
- **UIEventDispatcher**: Dispatches events to elements
- **MaskingSystem**: Handles clipping and masking
- **StyleSystem**: Applies styles and themes

## Performance

The UI system is designed for high performance:

- **Dirty Flagging**: Only recalculates layouts and rebuilds batches for dirty elements
- **Batch Rendering**: Groups elements with same material/texture into single draw calls
- **Culling**: Off-screen elements are excluded from rendering
- **Spatial Hashing**: Efficient raycasting with many elements
- **Object Pooling**: Frequently created/destroyed elements are pooled

## Testing

The UI system includes comprehensive tests:

- **Unit Tests**: Component creation, calculations, serialization
- **Property-Based Tests**: Universal properties verified across random inputs
- **Integration Tests**: Full workflows and system interactions

Run tests with:
```bash
cargo test --manifest-path ui/Cargo.toml
```

## Documentation

### Getting Started

- [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) - **Complete migration guide from legacy HUD system**
- [API_CHANGES.md](API_CHANGES.md) - **API changes and breaking changes reference**
- [EXAMPLES_GUIDE.md](EXAMPLES_GUIDE.md) - Guide to all examples

### Migration Resources

- [MIGRATION_TOOL_GUIDE.md](MIGRATION_TOOL_GUIDE.md) - Using the HUD migration tool
- [HUD_CONVERTER_GUIDE.md](HUD_CONVERTER_GUIDE.md) - HUD to UIPrefab converter details
- [VIDEO_TUTORIAL_SCRIPTS.md](VIDEO_TUTORIAL_SCRIPTS.md) - Video tutorial scripts

### System Documentation

- [LUA_API.md](LUA_API.md) - Complete Lua API reference
- [LAYOUT_SYSTEM.md](LAYOUT_SYSTEM.md) - Layout system details
- [HIERARCHY_SYSTEM_IMPLEMENTATION.md](HIERARCHY_SYSTEM_IMPLEMENTATION.md) - Hierarchy system
- [SCROLL_VIEW_IMPLEMENTATION.md](SCROLL_VIEW_IMPLEMENTATION.md) - Scroll view details
- [MASKING_SYSTEM.md](MASKING_SYSTEM.md) - Masking and clipping
- [TEXT_RENDERING_IMPLEMENTATION.md](TEXT_RENDERING_IMPLEMENTATION.md) - Text rendering
- [PREFAB_INSTANTIATION.md](PREFAB_INSTANTIATION.md) - Prefab system
- [STYLE_SYSTEM.md](STYLE_SYSTEM.md) - Styling and themes
- [LUA_BINDINGS_IMPLEMENTATION.md](LUA_BINDINGS_IMPLEMENTATION.md) - Lua integration

## Contributing

When contributing to the UI system:

1. Follow the existing code style and patterns
2. Add tests for new features
3. Update documentation
4. Run `cargo fmt` and `cargo clippy`
5. Ensure all tests pass

## License

This UI system is part of the XS 2D Game Engine project.

