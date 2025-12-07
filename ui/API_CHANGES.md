# API Changes: Legacy HUD System → New UI System

## Overview

This document provides a comprehensive mapping between the legacy HUD system API and the new UI system API. Use this as a reference when migrating your code.

---

## Table of Contents

1. [Breaking Changes Summary](#breaking-changes-summary)
2. [Rust API Changes](#rust-api-changes)
3. [Lua API Changes](#lua-api-changes)
4. [File Format Changes](#file-format-changes)
5. [Component Mapping](#component-mapping)
6. [Deprecation Notices](#deprecation-notices)

---

## Breaking Changes Summary

### Critical Breaking Changes

⚠️ **These changes require immediate action:**

1. **HUD Manager Removed**
   - `HudManager` struct no longer exists
   - Data binding system removed
   - Manual UI updates required in Lua

2. **File Format Changed**
   - `.hud` files replaced with `.uiprefab` files
   - JSON structure completely different
   - Automatic conversion tool available

3. **Anchor System Changed**
   - Simple 9-position anchors replaced with flexible RectTransform
   - Anchor values now normalized (0.0-1.0 range)
   - Separate anchor_min and anchor_max for stretching

4. **Component Types Changed**
   - `HudElementType` enum replaced with individual component structs
   - Each component type is now a separate struct
   - More granular control over properties

5. **Rendering System Changed**
   - egui-based rendering replaced with WGPU sprite rendering
   - UI now integrated with game rendering pipeline
   - Better performance and visual quality

### Non-Breaking Changes

✅ **These features are new or enhanced:**

- Interactive UI components (buttons, sliders, etc.)
- Event system for user input
- Animation system with easing
- Layout system for automatic positioning
- Scroll views with clipping
- Masking and stencil operations
- Resolution-independent scaling
- UI prefab system
- Style and theme support

---

## Rust API Changes

### Module Structure

**Before:**
```rust
use engine::hud::{HudManager, HudAsset, HudElement, HudElementType, Anchor};
```

**After:**
```rust
use ui::{
    Canvas, CanvasScaler, RectTransform, UIElement,
    UIImage, UIText, UIButton, UIPanel,
    UISlider, UIToggle, UIDropdown, UIInputField, UIScrollView,
    HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup,
    UIPrefab, UIStyle, UITheme
};
```

### HUD Manager

**Before:**
```rust
// Create HUD manager
let mut hud_manager = HudManager::new();

// Load HUD asset
let hud = HudAsset::load("assets/ui/game_hud.hud")?;
hud_manager.set_hud(hud);

// Bind data
hud_manager.bind("player_health", Box::new(|world| {
    // Get health from world
    0.75
}));

// Update (automatic)
hud_manager.update(world);

// Render
hud_manager.render(ui_context);
```

**After:**
```rust
// UI is now ECS-based, no manager needed
// Create UI entities directly in ECS world

// Load UI prefab
let prefab = UIPrefab::load("assets/ui/game_hud.uiprefab")?;

// Instantiate prefab
let mut instantiator = PrefabInstantiator::new();
let result = instantiator.instantiate(&prefab, &mut world);

// Update UI in Lua scripts (no automatic binding)
// Rendering happens automatically through UI systems
```

### Creating UI Elements

**Before:**
```rust
// HUD elements were defined in JSON only
// No programmatic creation in Rust
```

**After:**
```rust
use ui::*;
use ecs::World;

fn create_health_bar(world: &mut World, parent: Entity) -> Entity {
    let entity = world.spawn();
    
    // Add RectTransform
    world.insert_one(entity, RectTransform {
        anchor_min: Vec2::new(0.0, 1.0),
        anchor_max: Vec2::new(0.0, 1.0),
        pivot: Vec2::new(0.0, 1.0),
        anchored_position: Vec2::new(10.0, -10.0),
        size_delta: Vec2::new(200.0, 20.0),
        ..Default::default()
    }).unwrap();
    
    // Add UIElement
    world.insert_one(entity, UIElement {
        raycast_target: false,
        ..Default::default()
    }).unwrap();
    
    // Add UIImage
    world.insert_one(entity, UIImage {
        sprite: Some("health_bar_fill.png".to_string()),
        image_type: ImageType::Filled,
        fill_method: FillMethod::Horizontal,
        fill_amount: 1.0,
        ..Default::default()
    }).unwrap();
    
    // Set parent
    world.insert_one(entity, Parent(parent)).unwrap();
    
    entity
}
```

### Anchor Conversion

**Before:**
```rust
use engine::hud::Anchor;

let anchor = Anchor::TopLeft;
let offset = [10.0, 10.0];
let size = [100.0, 50.0];
```

**After:**
```rust
use ui::RectTransform;
use glam::Vec2;

let rect_transform = RectTransform {
    anchor_min: Vec2::new(0.0, 1.0),  // Top-left
    anchor_max: Vec2::new(0.0, 1.0),
    pivot: Vec2::new(0.0, 1.0),
    anchored_position: Vec2::new(10.0, -10.0),  // Note: Y is negative
    size_delta: Vec2::new(100.0, 50.0),
    ..Default::default()
};
```

### Component Type Mapping

**Before:**
```rust
use engine::hud::HudElementType;

let element_type = HudElementType::Text {
    text: "Hello".to_string(),
    font_size: 18.0,
    color: [1.0, 1.0, 1.0, 1.0],
};
```

**After:**
```rust
use ui::UIText;

let text_component = UIText {
    text: "Hello".to_string(),
    font: "default".to_string(),
    font_size: 18.0,
    color: [1.0, 1.0, 1.0, 1.0],
    alignment: TextAlignment::MiddleCenter,
    horizontal_overflow: OverflowMode::Wrap,
    vertical_overflow: OverflowMode::Truncate,
    ..Default::default()
};
```

---

## Lua API Changes

### Module Import

**Before:**
```lua
-- No explicit import needed
-- HUD manager was global
```

**After:**
```lua
-- UI functions are in the 'ui' module
-- Available globally, no import needed
```

### Creating UI Elements

**Before:**
```lua
-- Not supported in legacy system
-- UI could only be defined in .hud files
```

**After:**
```lua
-- Create canvas
local canvas = ui.create_canvas({
    render_mode = "ScreenSpaceOverlay",
    sort_order = 0
})

-- Create image
local image = ui.create_image(canvas, {
    name = "MyImage",
    sprite = "icon.png",
    color = {1.0, 1.0, 1.0, 1.0},
    rect_transform = {
        anchor_min = {0.5, 0.5},
        anchor_max = {0.5, 0.5},
        pivot = {0.5, 0.5},
        anchored_position = {0, 0},
        size_delta = {100, 100}
    }
})

-- Create text
local text = ui.create_text(canvas, {
    name = "MyText",
    text = "Hello World",
    font_size = 24.0,
    color = {1.0, 1.0, 1.0, 1.0}
})

-- Create button
local button = ui.create_button(canvas, {
    name = "MyButton",
    text = "Click Me",
    on_click = "on_button_clicked"
})
```

### Data Binding

**Before:**
```lua
-- Automatic data binding
hud_manager.bind("player_health", function()
    return player.health / player.max_health
end)

hud_manager.bind("score", function()
    return game.score
end)

-- Updates happened automatically
```

**After:**
```lua
-- Manual updates in update loop
local health_bar_fill = nil
local score_label = nil

function init()
    health_bar_fill = ui.find_element("HealthBar_Fill")
    score_label = ui.find_element("ScoreLabel")
end

function update(dt)
    -- Update health bar
    if health_bar_fill then
        local health_percent = player.health / player.max_health
        ui.set_fill_amount(health_bar_fill, health_percent)
    end
    
    -- Update score
    if score_label then
        ui.set_text(score_label, "Score: " .. game.score)
    end
end
```

### Finding Elements

**Before:**
```lua
-- Not supported
-- Elements were accessed through bindings only
```

**After:**
```lua
-- Find by name
local element = ui.find_element("ElementName")

-- Find by tag
local elements = ui.find_elements_by_tag("enemy_marker")

-- Get children
local children = ui.get_children(parent_element)

-- Get parent
local parent = ui.get_parent(element)
```

### Modifying Properties

**Before:**
```lua
-- Not supported
-- Properties were read-only
```

**After:**
```lua
-- Modify text
ui.set_text(text_element, "New Text")

-- Modify color
ui.set_color(element, {1.0, 0.0, 0.0, 1.0})

-- Modify position
ui.set_anchored_position(element, {100, 50})

-- Modify size
ui.set_size_delta(element, {200, 100})

-- Modify fill amount (for filled images)
ui.set_fill_amount(image_element, 0.75)

-- Modify alpha
ui.set_alpha(element, 0.5)

-- Show/hide
ui.set_active(element, true)
ui.set_active(element, false)
```

### Event Handling

**Before:**
```lua
-- Not supported
-- HUD was read-only
```

**After:**
```lua
-- Register click callback
ui.register_callback(button, "on_click", "on_button_clicked")

function on_button_clicked(button_entity)
    print("Button clicked!")
end

-- Register value changed callback
ui.register_callback(slider, "on_value_changed", "on_slider_changed")

function on_slider_changed(slider_entity, value)
    print("Slider value:", value)
    game.volume = value
end

-- Register hover callbacks
ui.register_callback(element, "on_pointer_enter", "on_hover_start")
ui.register_callback(element, "on_pointer_exit", "on_hover_end")

function on_hover_start(element)
    ui.set_color(element, {1.0, 1.0, 0.0, 1.0})  -- Yellow
end

function on_hover_end(element)
    ui.set_color(element, {1.0, 1.0, 1.0, 1.0})  -- White
end
```

### Animations

**Before:**
```lua
-- Not supported
```

**After:**
```lua
-- Animate position
ui.animate_position(element, {
    from = {0, 0},
    to = {100, 50},
    duration = 1.0,
    easing = "EaseOutQuad",
    on_complete = "on_animation_complete"
})

-- Animate color
ui.animate_color(element, {
    from = {1.0, 1.0, 1.0, 1.0},
    to = {1.0, 0.0, 0.0, 1.0},
    duration = 0.5,
    easing = "Linear"
})

-- Animate scale
ui.animate_scale(element, {
    from = {1.0, 1.0},
    to = {1.5, 1.5},
    duration = 0.3,
    easing = "EaseOutElastic"
})

-- Animate alpha (fade)
ui.animate_alpha(element, {
    from = 1.0,
    to = 0.0,
    duration = 1.0,
    easing = "EaseInQuad",
    on_complete = "on_fade_complete"
})

function on_fade_complete(element)
    ui.destroy(element)
end
```

### Destroying Elements

**Before:**
```lua
-- Not supported
```

**After:**
```lua
-- Destroy single element
ui.destroy(element)

-- Destroy element and all children
ui.destroy_recursive(element)
```

---

## File Format Changes

### HUD Asset Format (.hud)

**Before:**
```json
{
  "name": "GameHUD",
  "elements": [
    {
      "id": "HealthBar",
      "element_type": {
        "HealthBar": {
          "binding": "player.health",
          "color": [1.0, 0.0, 0.0, 1.0],
          "background_color": [0.2, 0.2, 0.2, 0.8]
        }
      },
      "anchor": "TopLeft",
      "offset": [10.0, 10.0],
      "size": [200.0, 20.0],
      "visible": true
    }
  ]
}
```

### UI Prefab Format (.uiprefab)

**After:**
```json
{
  "name": "GameHUD",
  "root": {
    "name": "Canvas",
    "id": "canvas_root",
    "rect_transform": {
      "anchor_min": [0.0, 0.0],
      "anchor_max": [1.0, 1.0],
      "pivot": [0.5, 0.5],
      "anchored_position": [0, 0],
      "size_delta": [0, 0],
      "rotation": 0.0,
      "scale": [1.0, 1.0]
    },
    "ui_element": {
      "raycast_target": false,
      "blocks_raycasts": false,
      "z_order": 0,
      "color": [1.0, 1.0, 1.0, 1.0],
      "alpha": 1.0,
      "interactable": true,
      "ignore_layout": false
    },
    "children": [
      {
        "name": "HealthBar",
        "id": "health_bar",
        "rect_transform": {
          "anchor_min": [0.0, 1.0],
          "anchor_max": [0.0, 1.0],
          "pivot": [0.0, 1.0],
          "anchored_position": [10.0, -10.0],
          "size_delta": [200.0, 20.0],
          "rotation": 0.0,
          "scale": [1.0, 1.0]
        },
        "ui_element": {
          "raycast_target": false,
          "color": [1.0, 1.0, 1.0, 1.0],
          "alpha": 1.0
        },
        "panel": {
          "background": "health_bar_bg.png",
          "use_nine_slice": true,
          "slice_borders": [4, 4, 4, 4],
          "padding": [0, 0, 0, 0]
        },
        "children": [
          {
            "name": "HealthBar_Fill",
            "id": "health_bar_fill",
            "rect_transform": {
              "anchor_min": [0.0, 0.0],
              "anchor_max": [1.0, 1.0],
              "pivot": [0.0, 0.5],
              "anchored_position": [0, 0],
              "size_delta": [0, 0]
            },
            "image": {
              "sprite": "health_bar_fill.png",
              "image_type": "Filled",
              "fill_method": "Horizontal",
              "fill_amount": 1.0,
              "color": [1.0, 0.0, 0.0, 1.0]
            }
          }
        ]
      }
    ]
  }
}
```

---

## Component Mapping

### Complete Component Mapping Table

| Legacy HudElementType | New UI Component(s) | Notes |
|----------------------|---------------------|-------|
| `Text` | `UIText` | Direct mapping |
| `DynamicText` | `UIText` + Lua script | Manual updates required |
| `Image` | `UIImage` | Direct mapping |
| `HealthBar` | `UIPanel` + `UIImage` (fill) | Requires two elements |
| `ProgressBar` | `UIPanel` + `UIImage` (fill) | Requires two elements |
| `Container` | Parent-child hierarchy | Use ECS hierarchy |
| `Minimap` | `UIPanel` + custom rendering | Custom implementation needed |
| N/A | `UIButton` | New component |
| N/A | `UISlider` | New component |
| N/A | `UIToggle` | New component |
| N/A | `UIDropdown` | New component |
| N/A | `UIInputField` | New component |
| N/A | `UIScrollView` | New component |
| N/A | `HorizontalLayoutGroup` | New component |
| N/A | `VerticalLayoutGroup` | New component |
| N/A | `GridLayoutGroup` | New component |
| N/A | `UIMask` | New component |

### Anchor Mapping Table

| Legacy Anchor | RectTransform Anchor | Pivot |
|--------------|---------------------|-------|
| `TopLeft` | `anchor_min: (0.0, 1.0)`, `anchor_max: (0.0, 1.0)` | `(0.0, 1.0)` |
| `TopCenter` | `anchor_min: (0.5, 1.0)`, `anchor_max: (0.5, 1.0)` | `(0.5, 1.0)` |
| `TopRight` | `anchor_min: (1.0, 1.0)`, `anchor_max: (1.0, 1.0)` | `(1.0, 1.0)` |
| `CenterLeft` | `anchor_min: (0.0, 0.5)`, `anchor_max: (0.0, 0.5)` | `(0.0, 0.5)` |
| `Center` | `anchor_min: (0.5, 0.5)`, `anchor_max: (0.5, 0.5)` | `(0.5, 0.5)` |
| `CenterRight` | `anchor_min: (1.0, 0.5)`, `anchor_max: (1.0, 0.5)` | `(1.0, 0.5)` |
| `BottomLeft` | `anchor_min: (0.0, 0.0)`, `anchor_max: (0.0, 0.0)` | `(0.0, 0.0)` |
| `BottomCenter` | `anchor_min: (0.5, 0.0)`, `anchor_max: (0.5, 0.0)` | `(0.5, 0.0)` |
| `BottomRight` | `anchor_min: (1.0, 0.0)`, `anchor_max: (1.0, 0.0)` | `(1.0, 0.0)` |

### Property Mapping Table

| Legacy Property | New Property | Component | Notes |
|----------------|--------------|-----------|-------|
| `id` | `name` | `UIPrefabElement` | Element identifier |
| `anchor` | `anchor_min`, `anchor_max` | `RectTransform` | More flexible |
| `offset` | `anchored_position` | `RectTransform` | Position offset |
| `size` | `size_delta` | `RectTransform` | Size |
| `visible` | `alpha` / `active` | `UIElement` | Use `set_active()` |
| `text` | `text` | `UIText` | Direct mapping |
| `font_size` | `font_size` | `UIText` | Direct mapping |
| `color` | `color` | `UIElement` / `UIText` | Applied to all components |
| `texture` | `sprite` | `UIImage` | Sprite/texture reference |
| `binding` | Lua script | N/A | Manual updates required |

---

## Deprecation Notices

### Deprecated Modules

❌ **Removed in this version:**

- `engine::hud::HudManager` - Use UI system directly
- `engine::hud::HudAsset` - Use `ui::UIPrefab`
- `engine::hud::HudElement` - Use `ui::UIPrefabElement`
- `engine::hud::HudElementType` - Use individual component types
- `engine::hud::Anchor` - Use `ui::RectTransform`
- `engine::hud::hud_renderer` - Use UI rendering system

### Deprecated Functions

❌ **Removed in this version:**

**Rust:**
- `HudManager::new()` - No replacement (use ECS directly)
- `HudManager::set_hud()` - Use `PrefabInstantiator::instantiate()`
- `HudManager::bind()` - Update UI manually in Lua
- `HudManager::update()` - UI updates automatically
- `HudManager::render()` - Rendering is automatic
- `HudAsset::load()` - Use `UIPrefab::load()`
- `HudAsset::save()` - Use `UIPrefab::save()`

**Lua:**
- `hud_manager.bind()` - Update UI manually in `update()`
- `hud_manager.unbind()` - No longer needed
- `hud_manager.get_value()` - Use `ui.get_*()` functions
- `hud_manager.set_value()` - Use `ui.set_*()` functions

### Migration Timeline

- **Version 0.9.x**: Legacy HUD system (deprecated)
- **Version 1.0.0**: New UI system introduced, legacy system still available
- **Version 1.1.0**: Legacy HUD system removed (current version)

### Compatibility

⚠️ **No backward compatibility:**

The new UI system is not backward compatible with the legacy HUD system. All `.hud` files must be converted to `.uiprefab` format using the migration tool.

**Migration Tool:**
```bash
cargo run --package ui --bin hud_migrator -- --paths . --progress
```

---

## Quick Reference

### Common Migration Patterns

#### Pattern 1: Simple Text Display

**Before:**
```lua
-- .hud file defines text
-- Lua binds data
hud_manager.bind("score", function() return game.score end)
```

**After:**
```lua
-- .uiprefab file defines text
-- Lua updates manually
function update(dt)
    local label = ui.find_element("ScoreLabel")
    ui.set_text(label, "Score: " .. game.score)
end
```

#### Pattern 2: Health Bar

**Before:**
```lua
-- .hud file defines HealthBar
-- Lua binds health value
hud_manager.bind("player.health", function()
    return player.health / player.max_health
end)
```

**After:**
```lua
-- .uiprefab file defines Panel + Image (fill)
-- Lua updates fill amount
function update(dt)
    local fill = ui.find_element("HealthBar_Fill")
    ui.set_fill_amount(fill, player.health / player.max_health)
end
```

#### Pattern 3: Dynamic UI Creation

**Before:**
```lua
-- Not supported
```

**After:**
```lua
-- Create UI dynamically
function create_item_slot(parent, item, index)
    local slot = ui.create_button(parent, {
        name = "ItemSlot_" .. index,
        sprite = item.icon,
        rect_transform = {
            anchor_min = {0, 1},
            anchor_max = {0, 1},
            pivot = {0, 1},
            anchored_position = {10 + (index * 60), -10},
            size_delta = {50, 50}
        }
    })
    ui.register_callback(slot, "on_click", "on_item_clicked")
    return slot
end
```

---

## Additional Resources

- **Migration Guide**: [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)
- **UI System Documentation**: [README.md](README.md)
- **Lua API Reference**: [LUA_API.md](LUA_API.md)
- **Migration Tool Guide**: [MIGRATION_TOOL_GUIDE.md](MIGRATION_TOOL_GUIDE.md)
- **Example Scripts**: [examples/](examples/)

---

**Last Updated:** December 2025

**Version:** 1.0.0
