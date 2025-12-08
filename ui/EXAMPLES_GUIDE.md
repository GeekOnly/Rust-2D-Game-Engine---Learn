# UI System Examples Guide

This guide provides an overview of all available examples for the UI system.

## Running Examples

All Rust examples can be run using:
```bash
cargo run --example <example_name> --manifest-path ui/Cargo.toml
```

## Available Examples

### 1. basic_ui.rs - Core UI Fundamentals

**What it demonstrates:**
- Canvas creation with different render modes
- RectTransform anchoring (fixed, stretched, corner anchoring)
- Creating basic UI components (Image, Text, Button, Panel)
- Building UI hierarchies (parent-child relationships)
- Different anchor modes and their use cases

**Key concepts:**
- Screen Space Overlay vs World Space canvases
- Canvas scaler modes
- Fixed position anchoring (anchor min == anchor max)
- Stretched anchoring (anchor min != anchor max)
- Pivot points and their effect on positioning

**Run with:**
```bash
cargo run --example basic_ui --manifest-path ui/Cargo.toml
```

**Output:** Console output showing canvas configurations, anchor modes, button creation, and UI hierarchy structure.

---

### 2. advanced_ui.rs - Advanced Components & Features

**What it demonstrates:**
- UIScrollView with viewport clipping and scrollbars
- UISlider with value clamping and normalization
- UIToggle for boolean options
- UIDropdown for option selection
- UIInputField with different content types
- UI animations with easing functions
- Event system and callbacks

**Key concepts:**
- Scroll view movement types (elastic, clamped, unrestricted)
- Slider value ranges and normalization
- Toggle state management
- Dropdown option lists
- Input field validation (standard, number, password, email, etc.)
- Animation properties (position, scale, rotation, color, alpha)
- Easing functions (Linear, EaseOut, EaseIn, Bounce, etc.)
- Event types (click, hover, drag, scroll)

**Run with:**
```bash
cargo run --example advanced_ui --manifest-path ui/Cargo.toml
```

**Output:** Console output demonstrating all advanced components with their configurations and use cases.

---

### 3. layout_demo.rs - Automatic Layout System

**What it demonstrates:**
- Horizontal Layout Group
- Vertical Layout Group
- Grid Layout Group
- Layout spacing and padding
- Child alignment options

**Key concepts:**
- Automatic child positioning
- Spacing between elements
- Padding around layout container
- Child alignment (upper, middle, lower, left, center, right)
- Force expand options
- Grid constraints (flexible, fixed column/row count)

**Run with:**
```bash
cargo run --example layout_demo --manifest-path ui/Cargo.toml
```

**Output:** Console output showing calculated positions for children in each layout type.

---

### 4. prefab_demo.rs - Reusable UI Templates

**What it demonstrates:**
- Creating UI prefabs
- Instantiating prefabs
- Parameterized instantiation (overriding values)
- Multiple instances from same prefab
- Destroying prefab instances
- Complex hierarchical prefabs

**Key concepts:**
- UIPrefab structure
- UIPrefabElement hierarchy
- PrefabInstantiator
- PrefabParameters for customization
- Named entities in prefabs
- Prefab reusability

**Run with:**
```bash
cargo run --example prefab_demo --manifest-path ui/Cargo.toml
```

**Output:** Console output showing prefab instantiation with and without parameters, multiple instances, and entity management.

---

### 5. style_demo.rs - Styling & Theming

**What it demonstrates:**
- Creating UI styles
- Applying styles to elements
- Style inheritance from parent
- Theme changes affecting all elements
- Smooth style transitions with animations
- Color interpolation

**Key concepts:**
- UIStyle configuration (colors, fonts, sprites, spacing)
- UITheme management
- Active style selection
- Style application to different component types
- StyleTransition for animated theme changes
- Style inheritance rules

**Run with:**
```bash
cargo run --example style_demo --manifest-path ui/Cargo.toml
```

**Output:** Console output showing style creation, application, inheritance, theme changes, and transition animations.

---

### 6. lua_ui_example.lua - Lua API Demonstration

**What it demonstrates:**
- Creating UI from Lua scripts
- Main menu with buttons
- Settings panel with sliders and toggles
- Game HUD with health bar, score, minimap, abilities
- Animated notification system
- Event handlers in Lua
- Dynamic UI updates

**Key concepts:**
- Lua API functions (ui_create_*, ui_set_*, ui_get_*)
- Canvas creation from Lua
- Element hierarchy in Lua
- Property manipulation
- Animation from Lua
- Event callback registration
- Finding elements by name
- Dynamic UI updates (health, score)

**Usage:**
This is a Lua script that demonstrates the API. It would be loaded and executed by the engine's Lua runtime. The script includes:
- `create_main_menu()` - Creates a menu with buttons
- `create_settings_panel()` - Creates settings UI
- `create_game_hud()` - Creates in-game HUD
- `show_notification()` - Shows animated notifications
- `update_health()` - Updates health bar
- `update_score()` - Updates score display

**Key functions demonstrated:**
```lua
ui_create_canvas()
ui_create_panel()
ui_create_text()
ui_create_button()
ui_set_position()
ui_set_size()
ui_set_anchor_min()
ui_set_anchor_max()
ui_set_color()
ui_set_name()
ui_find_by_name()
ui_animate_position()
ui_animate_alpha()
ui_animate_scale()
ui_on_pointer_enter()
ui_on_pointer_exit()
```

---

## Example Progression

We recommend exploring the examples in this order:

1. **basic_ui.rs** - Start here to understand core concepts
2. **layout_demo.rs** - Learn automatic layouts
3. **advanced_ui.rs** - Explore advanced components
4. **prefab_demo.rs** - Understand reusable templates
5. **style_demo.rs** - Learn styling and theming
6. **lua_ui_example.lua** - See how to use the Lua API

## Common Patterns

### Creating a Button with Text

```rust
// Rust
let button = UIButton::default();
let button_transform = RectTransform::anchored(
    Vec2::new(0.5, 0.5),
    Vec2::ZERO,
    Vec2::new(200.0, 50.0)
);

let text = UIText {
    text: "Click Me!".to_string(),
    ..Default::default()
};
let text_transform = RectTransform::stretched(
    Vec2::ZERO,
    Vec2::ONE,
    [5.0, 5.0, 5.0, 5.0].into()
);
```

```lua
-- Lua
local button = ui_create_button({parent = canvas})
ui_set_position(button, 0, 0)
ui_set_size(button, 200, 50)

local text = ui_create_text({
    parent = button,
    text = "Click Me!"
})
ui_set_anchor_min(text, 0.0, 0.0)
ui_set_anchor_max(text, 1.0, 1.0)
```

### Animating UI Elements

```rust
// Rust
let animation = UIAnimation {
    entity: button_entity,
    property: AnimatedProperty::Scale,
    from: AnimationValue::Vec2(Vec2::ONE),
    to: AnimationValue::Vec2(Vec2::new(1.2, 1.2)),
    duration: 0.2,
    easing: EasingFunction::EaseOutBack,
    ..Default::default()
};
```

```lua
-- Lua
ui_animate_scale({
    entity = button,
    to_x = 1.2,
    to_y = 1.2,
    duration = 0.2,
    easing = "EaseOutBack"
})
```

### Creating Responsive Layouts

```rust
// Rust
let layout = VerticalLayoutGroup {
    spacing: 10.0,
    padding: Vec4::new(10.0, 10.0, 10.0, 10.0),
    child_alignment: Alignment::MiddleCenter,
    child_force_expand_width: true,
    ..Default::default()
};
```

## Tips & Best Practices

1. **Use Anchoring for Responsiveness**: Always use appropriate anchor modes to ensure UI adapts to different screen sizes.

2. **Leverage Layout Groups**: Use layout groups instead of manual positioning when arranging multiple elements.

3. **Prefabs for Reusability**: Create prefabs for commonly used UI patterns (buttons, dialogs, etc.).

4. **Style Consistency**: Use the styling system to maintain consistent look across your UI.

5. **Event Callbacks**: Register event callbacks for interactive elements to handle user input.

6. **Animations for Polish**: Add subtle animations to make UI feel more responsive and polished.

7. **Lua for Dynamic UI**: Use Lua API for UI that needs to be created or modified at runtime.

## Next Steps

After exploring the examples:

1. Read the [README.md](README.md) for comprehensive API documentation
2. Check [LUA_API.md](LUA_API.md) for complete Lua API reference
3. Review implementation docs for specific systems:
   - [LAYOUT_SYSTEM.md](LAYOUT_SYSTEM.md)
   - [SCROLL_VIEW_IMPLEMENTATION.md](SCROLL_VIEW_IMPLEMENTATION.md)
   - [MASKING_SYSTEM.md](MASKING_SYSTEM.md)
   - [PREFAB_INSTANTIATION.md](PREFAB_INSTANTIATION.md)
   - [STYLE_SYSTEM.md](STYLE_SYSTEM.md)

## Troubleshooting

**Example won't compile:**
- Ensure you're in the project root directory
- Check that all dependencies are up to date: `cargo update`
- Try cleaning and rebuilding: `cargo clean && cargo build`

**Example runs but shows warnings:**
- Warnings about unused variables are normal in examples
- These don't affect functionality

**Want to modify an example:**
- Copy the example to your own project
- Modify as needed
- Examples are meant to be learning resources and starting points

## Contributing Examples

If you create a useful example, consider contributing it:
1. Follow the existing example structure
2. Add comprehensive comments
3. Update this guide with your example
4. Submit a pull request

