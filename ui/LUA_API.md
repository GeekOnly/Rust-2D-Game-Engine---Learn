# UI System Lua API Documentation

This document describes the Lua API for the UI system, providing comprehensive bindings for creating and manipulating UI elements at runtime.

## Overview

The UI Lua API follows patterns similar to Unity's UI system, providing functions for:
- Canvas creation and management
- UI element creation (Image, Text, Button, Panel, etc.)
- Hierarchy operations (parenting, children queries, destruction)
- Property getters and setters for all components
- Animation functions
- Event callback registration
- Element queries (find by name, tag, etc.)

## Canvas Creation

### `ui_create_canvas(args)`

Creates a new Canvas entity.

**Parameters:**
- `args` (table):
  - `render_mode` (string, optional): "ScreenSpaceOverlay" (default), "ScreenSpaceCamera", or "WorldSpace"
  - `sort_order` (number, optional): Sort order for rendering (default: 0)

**Returns:** Entity ID

**Example:**
```lua
local canvas = ui_create_canvas({
    render_mode = "ScreenSpaceOverlay",
    sort_order = 0
})
```

## UI Element Creation

### `ui_create_image(args)`

Creates a new Image element.

**Parameters:**
- `args` (table):
  - `parent` (entity, optional): Parent entity
  - `sprite` (string, optional): Sprite/texture ID
  - `color` (table, optional): Color tint {r, g, b, a}

**Returns:** Entity ID

**Example:**
```lua
local image = ui_create_image({
    parent = canvas,
    sprite = "my_sprite",
    color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
})
```

### `ui_create_text(args)`

Creates a new Text element.

**Parameters:**
- `args` (table):
  - `parent` (entity, optional): Parent entity
  - `text` (string, optional): Text content
  - `font_size` (number, optional): Font size (default: 14)
  - `color` (table, optional): Text color {r, g, b, a}

**Returns:** Entity ID

**Example:**
```lua
local text = ui_create_text({
    parent = canvas,
    text = "Hello World",
    font_size = 24,
    color = {r = 0.0, g = 0.0, b = 0.0, a = 1.0}
})
```

### `ui_create_button(args)`

Creates a new Button element.

**Parameters:**
- `args` (table):
  - `parent` (entity, optional): Parent entity
  - `on_click` (string, optional): Lua callback function name

**Returns:** Entity ID

**Example:**
```lua
function on_button_click()
    log("Button clicked!")
end

local button = ui_create_button({
    parent = canvas,
    on_click = "on_button_click"
})
```

### `ui_create_panel(args)`

Creates a new Panel element.

**Parameters:**
- `args` (table):
  - `parent` (entity, optional): Parent entity
  - `background` (string, optional): Background sprite ID

**Returns:** Entity ID

**Example:**
```lua
local panel = ui_create_panel({
    parent = canvas,
    background = "panel_bg"
})
```

## Hierarchy Operations

### `ui_set_parent(entity, parent)`

Sets the parent of a UI element.

**Parameters:**
- `entity` (entity): The entity to reparent
- `parent` (entity or nil): The new parent entity

**Example:**
```lua
ui_set_parent(child_element, parent_element)
```

### `ui_get_parent(entity)`

Gets the parent of a UI element.

**Parameters:**
- `entity` (entity): The entity to query

**Returns:** Parent entity ID or nil

**Example:**
```lua
local parent = ui_get_parent(element)
```

### `ui_get_children(entity)`

Gets all children of a UI element.

**Parameters:**
- `entity` (entity): The entity to query

**Returns:** Table (array) of child entity IDs

**Example:**
```lua
local children = ui_get_children(parent)
for i, child in ipairs(children) do
    log("Child " .. i .. ": " .. child)
end
```

### `ui_destroy(entity)`

Destroys a UI element and all its children.

**Parameters:**
- `entity` (entity): The entity to destroy

**Example:**
```lua
ui_destroy(element)
```

## Property Getters and Setters

### RectTransform Properties

#### Position

```lua
-- Get position
local pos = ui_get_position(entity)
log("Position: " .. pos.x .. ", " .. pos.y)

-- Set position
ui_set_position(entity, 100, 200)
```

#### Size

```lua
-- Get size
local size = ui_get_size(entity)
log("Size: " .. size.x .. ", " .. size.y)

-- Set size
ui_set_size(entity, 200, 100)
```

#### Anchor Min

```lua
-- Get anchor min
local anchor_min = ui_get_anchor_min(entity)

-- Set anchor min
ui_set_anchor_min(entity, 0.0, 0.0)  -- Bottom-left
```

#### Anchor Max

```lua
-- Get anchor max
local anchor_max = ui_get_anchor_max(entity)

-- Set anchor max
ui_set_anchor_max(entity, 1.0, 1.0)  -- Top-right
```

#### Pivot

```lua
-- Get pivot
local pivot = ui_get_pivot(entity)

-- Set pivot
ui_set_pivot(entity, 0.5, 0.5)  -- Center
```

#### Rotation

```lua
-- Get rotation (in degrees)
local rotation = ui_get_rotation(entity)

-- Set rotation
ui_set_rotation(entity, 45.0)
```

#### Scale

```lua
-- Get scale
local scale = ui_get_scale(entity)

-- Set scale
ui_set_scale(entity, 1.5, 1.5)
```

### UIElement Properties

#### Color

```lua
-- Get color
local color = ui_get_color(entity)
log("Color: " .. color.r .. ", " .. color.g .. ", " .. color.b .. ", " .. color.a)

-- Set color
ui_set_color({
    entity = entity,
    r = 1.0,
    g = 0.5,
    b = 0.0,
    a = 1.0
})
```

#### Alpha

```lua
-- Get alpha
local alpha = ui_get_alpha(entity)

-- Set alpha
ui_set_alpha(entity, 0.5)
```

#### Interactable

```lua
-- Get interactable state
local interactable = ui_get_interactable(entity)

-- Set interactable
ui_set_interactable(entity, false)
```

#### Raycast Target

```lua
-- Get raycast target state
local raycast_target = ui_get_raycast_target(entity)

-- Set raycast target
ui_set_raycast_target(entity, true)
```

### UIText Properties

#### Text Content

```lua
-- Get text
local text = ui_get_text(entity)

-- Set text
ui_set_text(entity, "New text")
```

#### Font Size

```lua
-- Get font size
local font_size = ui_get_font_size(entity)

-- Set font size
ui_set_font_size(entity, 24)
```

#### Text Alignment

```lua
-- Get text alignment
local alignment = ui_get_text_alignment(entity)

-- Set text alignment
-- Options: "TopLeft", "TopCenter", "TopRight",
--          "MiddleLeft", "MiddleCenter", "MiddleRight",
--          "BottomLeft", "BottomCenter", "BottomRight"
ui_set_text_alignment(entity, "MiddleCenter")
```

### UIImage Properties

#### Sprite

```lua
-- Get sprite
local sprite = ui_get_sprite(entity)

-- Set sprite
ui_set_sprite(entity, "new_sprite_id")
```

#### Fill Amount

```lua
-- Get fill amount (0.0 to 1.0)
local fill_amount = ui_get_fill_amount(entity)

-- Set fill amount (for filled image types)
ui_set_fill_amount(entity, 0.75)
```

### UISlider Properties

#### Slider Value

```lua
-- Get slider value
local value = ui_get_slider_value(entity)

-- Set slider value
ui_set_slider_value(entity, 0.5)
```

#### Slider Min/Max

```lua
-- Get min value
local min = ui_get_slider_min(entity)

-- Set min value
ui_set_slider_min(entity, 0.0)

-- Get max value
local max = ui_get_slider_max(entity)

-- Set max value
ui_set_slider_max(entity, 100.0)
```

### UIToggle Properties

#### Toggle Value

```lua
-- Get toggle state
local is_on = ui_get_toggle_value(entity)

-- Set toggle state
ui_set_toggle_value(entity, true)
```

### UIDropdown Properties

#### Dropdown Value

```lua
-- Get selected index
local index = ui_get_dropdown_value(entity)

-- Set selected index
ui_set_dropdown_value(entity, 2)
```

#### Dropdown Options

```lua
-- Get options
local options = ui_get_dropdown_options(entity)

-- Set options
ui_set_dropdown_options(entity, {
    {text = "Option 1"},
    {text = "Option 2"},
    {text = "Option 3"}
})
```

### UIInputField Properties

#### Input Text

```lua
-- Get input text
local text = ui_get_input_text(entity)

-- Set input text
ui_set_input_text(entity, "New input")
```

#### Input Placeholder

```lua
-- Get placeholder
local placeholder = ui_get_input_placeholder(entity)

-- Set placeholder
ui_set_input_placeholder(entity, "Enter text...")
```

#### Character Limit

```lua
-- Get character limit
local limit = ui_get_input_character_limit(entity)

-- Set character limit (0 = unlimited)
ui_set_input_character_limit(entity, 50)
```

### UIScrollView Properties

#### Scroll Position

```lua
-- Get scroll position (normalized 0-1)
local pos = ui_get_scroll_position(entity)

-- Set scroll position
ui_set_scroll_position(entity, 0.5, 0.5)
```

## Animation Functions

### `ui_animate_position(args)`

Animates the position of a UI element.

**Parameters:**
- `args` (table):
  - `entity` (entity): The entity to animate
  - `to_x` (number): Target X position
  - `to_y` (number): Target Y position
  - `duration` (number): Animation duration in seconds
  - `easing` (string, optional): Easing function name (default: "Linear")
  - `on_complete` (string, optional): Callback function name

**Example:**
```lua
ui_animate_position({
    entity = element,
    to_x = 500,
    to_y = 300,
    duration = 1.0,
    easing = "EaseOutQuad",
    on_complete = "on_animation_complete"
})
```

### `ui_animate_scale(args)`

Animates the scale of a UI element.

**Example:**
```lua
ui_animate_scale({
    entity = element,
    to_x = 1.5,
    to_y = 1.5,
    duration = 0.5,
    easing = "EaseInOutBack"
})
```

### `ui_animate_rotation(args)`

Animates the rotation of a UI element.

**Example:**
```lua
ui_animate_rotation({
    entity = element,
    to = 180,
    duration = 1.0,
    easing = "EaseInOutQuad"
})
```

### `ui_animate_color(args)`

Animates the color of a UI element.

**Example:**
```lua
ui_animate_color({
    entity = element,
    to_r = 1.0,
    to_g = 0.0,
    to_b = 0.0,
    to_a = 1.0,
    duration = 0.5
})
```

### `ui_animate_alpha(args)`

Animates the alpha of a UI element.

**Example:**
```lua
ui_animate_alpha({
    entity = element,
    to = 0.0,
    duration = 1.0,
    easing = "EaseOutQuad"
})
```

### `ui_stop_animation(entity)`

Stops all animations on a UI element.

**Example:**
```lua
ui_stop_animation(element)
```

## Event Callbacks

### `ui_on_click(entity, callback)`

Registers a click event callback.

**Parameters:**
- `entity` (entity): The UI element
- `callback` (string): Name of the Lua function to call

**Example:**
```lua
function my_click_handler()
    log("Element clicked!")
end

ui_on_click(button, "my_click_handler")
```

### `ui_on_pointer_enter(entity, callback)`

Registers a pointer enter event callback (when mouse enters element).

**Example:**
```lua
function on_hover_start()
    log("Mouse entered element")
end

ui_on_pointer_enter(element, "on_hover_start")
```

### `ui_on_pointer_exit(entity, callback)`

Registers a pointer exit event callback (when mouse leaves element).

**Example:**
```lua
function on_hover_end()
    log("Mouse left element")
end

ui_on_pointer_exit(element, "on_hover_end")
```

### `ui_on_pointer_down(entity, callback)`

Registers a pointer down event callback (when mouse button is pressed).

**Example:**
```lua
function on_press()
    log("Mouse button pressed")
end

ui_on_pointer_down(element, "on_press")
```

### `ui_on_pointer_up(entity, callback)`

Registers a pointer up event callback (when mouse button is released).

**Example:**
```lua
function on_release()
    log("Mouse button released")
end

ui_on_pointer_up(element, "on_release")
```

### `ui_on_drag(entity, callback)`

Registers a drag event callback (when element is being dragged).

**Example:**
```lua
function on_dragging()
    log("Element is being dragged")
end

ui_on_drag(element, "on_dragging")
```

### `ui_on_begin_drag(entity, callback)`

Registers a begin drag event callback (when drag starts).

**Example:**
```lua
function on_drag_start()
    log("Drag started")
end

ui_on_begin_drag(element, "on_drag_start")
```

### `ui_on_end_drag(entity, callback)`

Registers an end drag event callback (when drag ends).

**Example:**
```lua
function on_drag_end()
    log("Drag ended")
end

ui_on_end_drag(element, "on_drag_end")
```

### `ui_on_scroll(entity, callback)`

Registers a scroll event callback (for scroll views).

**Example:**
```lua
function on_scrolling()
    log("Scroll view scrolled")
end

ui_on_scroll(scroll_view, "on_scrolling")
```

### `ui_on_value_changed(entity, callback)`

Registers a value changed callback (for sliders, toggles, dropdowns, input fields).

**Example:**
```lua
function on_slider_changed()
    local value = ui_get_slider_value(slider)
    log("Slider value: " .. value)
end

ui_on_value_changed(slider, "on_slider_changed")
```

### `ui_remove_event_callback(entity, event_type)`

Removes a specific event callback from an element.

**Parameters:**
- `entity` (entity): The UI element
- `event_type` (string): The event type to remove

**Example:**
```lua
ui_remove_event_callback(element, "OnPointerClick")
```

### `ui_remove_all_callbacks(entity)`

Removes all event callbacks from an element.

**Parameters:**
- `entity` (entity): The UI element

**Example:**
```lua
ui_remove_all_callbacks(element)
```

## Element Queries

### `ui_find_by_name(name)`

Finds a UI element by name.

**Parameters:**
- `name` (string): The name to search for

**Returns:** Entity ID or nil

**Example:**
```lua
ui_set_name(element, "my_button")
local found = ui_find_by_name("my_button")
if found then
    log("Found element: " .. found)
end
```

### `ui_find_by_tag(tag)`

Finds all UI elements with a specific tag.

**Parameters:**
- `tag` (string): The tag to search for

**Returns:** Table (array) of entity IDs

**Example:**
```lua
ui_set_tag(element1, "menu_item")
ui_set_tag(element2, "menu_item")
local items = ui_find_by_tag("menu_item")
for i, item in ipairs(items) do
    log("Item " .. i .. ": " .. item)
end
```

### `ui_set_name(entity, name)`

Sets the name of a UI element for lookup.

**Parameters:**
- `entity` (entity): The UI element
- `name` (string): The name to assign

**Example:**
```lua
ui_set_name(button, "start_button")
```

### `ui_set_tag(entity, tag)`

Sets a tag on a UI element for lookup. Multiple elements can share the same tag.

**Parameters:**
- `entity` (entity): The UI element
- `tag` (string): The tag to assign

**Example:**
```lua
ui_set_tag(button1, "menu_button")
ui_set_tag(button2, "menu_button")
```

### `ui_get_active(entity)`

Checks if a UI element is active (visible and interactable).

**Parameters:**
- `entity` (entity): The UI element

**Returns:** boolean

**Example:**
```lua
if ui_get_active(element) then
    log("Element is active")
end
```

### `ui_set_active(entity, active)`

Sets whether a UI element is active.

**Parameters:**
- `entity` (entity): The entity
- `active` (boolean): Active state

**Example:**
```lua
ui_set_active(menu, false)  -- Hide menu
```

### `ui_get_child(parent, index)`

Gets a child element by index.

**Parameters:**
- `parent` (entity): The parent entity
- `index` (number): The child index (1-based)

**Returns:** Entity ID or nil

**Example:**
```lua
local first_child = ui_get_child(panel, 1)
```

### `ui_get_child_count(entity)`

Gets the number of children an element has.

**Parameters:**
- `entity` (entity): The parent entity

**Returns:** number

**Example:**
```lua
local count = ui_get_child_count(panel)
log("Panel has " .. count .. " children")
```

### `ui_get_sibling_index(entity)`

Gets the sibling index of an element (its position among siblings).

**Parameters:**
- `entity` (entity): The UI element

**Returns:** number (0-based index)

**Example:**
```lua
local index = ui_get_sibling_index(button)
```

### `ui_set_sibling_index(entity, index)`

Sets the sibling index of an element (reorders it among siblings).

**Parameters:**
- `entity` (entity): The UI element
- `index` (number): The new sibling index (0-based)

**Example:**
```lua
ui_set_sibling_index(button, 0)  -- Move to first position
```

### `ui_exists(entity)`

Checks if an entity exists in the world.

**Parameters:**
- `entity` (entity): The entity to check

**Returns:** boolean

**Example:**
```lua
if ui_exists(element) then
    log("Element exists")
end
```

### `ui_get_canvas(entity)`

Gets the canvas that contains this UI element.

**Parameters:**
- `entity` (entity): The UI element

**Returns:** Canvas entity ID or nil

**Example:**
```lua
local canvas = ui_get_canvas(button)
```

## Easing Functions

Available easing functions for animations:
- `Linear`
- `EaseInQuad`, `EaseOutQuad`, `EaseInOutQuad`
- `EaseInCubic`, `EaseOutCubic`, `EaseInOutCubic`
- `EaseInSine`, `EaseOutSine`, `EaseInOutSine`
- `EaseInBack`, `EaseOutBack`, `EaseInOutBack`
- `EaseInBounce`, `EaseOutBounce`, `EaseInOutBounce`

## Complete Example

```lua
-- Create a canvas
local canvas = ui_create_canvas({
    render_mode = "ScreenSpaceOverlay"
})

-- Create a panel
local panel = ui_create_panel({
    parent = canvas,
    background = "panel_bg"
})

-- Create a title text
local title = ui_create_text({
    parent = panel,
    text = "Main Menu",
    font_size = 32,
    color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
})
ui_set_position(title, 0, 100)

-- Create a button
function on_start_click()
    log("Start button clicked!")
    ui_animate_alpha({
        entity = panel,
        to = 0.0,
        duration = 0.5,
        on_complete = "on_fade_complete"
    })
end

local start_button = ui_create_button({
    parent = panel,
    on_click = "on_start_click"
})
ui_set_position(start_button, 0, 0)

-- Add button text
local button_text = ui_create_text({
    parent = start_button,
    text = "Start Game",
    font_size = 18
})

-- Animate button on hover
function on_button_hover()
    ui_animate_scale({
        entity = start_button,
        to_x = 1.1,
        to_y = 1.1,
        duration = 0.2,
        easing = "EaseOutQuad"
    })
end

ui_on_pointer_enter(start_button, "on_button_hover")
```

## Notes

- All entity IDs are numbers (u32 from the ECS system)
- Colors are tables with `r`, `g`, `b`, `a` fields (0.0 to 1.0)
- Positions and sizes are in pixels (or canvas units depending on scaler)
- Callback functions must be defined in the global scope
- The actual ECS integration is pending - current implementation provides the API structure

## API Summary

### Canvas & Element Creation (Task 18.1 - Complete)
- ✅ `ui_create_canvas` - Create canvas
- ✅ `ui_create_image` - Create image element
- ✅ `ui_create_text` - Create text element
- ✅ `ui_create_button` - Create button element
- ✅ `ui_create_panel` - Create panel element

### Hierarchy Operations (Task 18.1 - Complete)
- ✅ `ui_set_parent` - Set parent
- ✅ `ui_get_parent` - Get parent
- ✅ `ui_get_children` - Get children
- ✅ `ui_destroy` - Destroy element

### Property Getters/Setters (Task 18.4 - Complete)
- ✅ RectTransform: position, size, anchor_min, anchor_max, pivot, rotation, scale
- ✅ UIElement: color, alpha, interactable, raycast_target
- ✅ UIText: text, font_size, text_alignment
- ✅ UIImage: sprite, fill_amount
- ✅ UISlider: value, min, max
- ✅ UIToggle: value
- ✅ UIDropdown: value, options
- ✅ UIInputField: text, placeholder, character_limit
- ✅ UIScrollView: scroll_position

### Animation Functions (Task 18.4 - Complete)
- ✅ `ui_animate_position` - Animate position
- ✅ `ui_animate_scale` - Animate scale
- ✅ `ui_animate_rotation` - Animate rotation
- ✅ `ui_animate_color` - Animate color
- ✅ `ui_animate_alpha` - Animate alpha
- ✅ `ui_stop_animation` - Stop animations

### Event Callbacks (Task 18.4 - Complete)
- ✅ `ui_on_click` - Click event
- ✅ `ui_on_pointer_enter` - Pointer enter event
- ✅ `ui_on_pointer_exit` - Pointer exit event
- ✅ `ui_on_pointer_down` - Pointer down event
- ✅ `ui_on_pointer_up` - Pointer up event
- ✅ `ui_on_drag` - Drag event
- ✅ `ui_on_begin_drag` - Begin drag event
- ✅ `ui_on_end_drag` - End drag event
- ✅ `ui_on_scroll` - Scroll event
- ✅ `ui_on_value_changed` - Value changed event
- ✅ `ui_remove_event_callback` - Remove specific callback
- ✅ `ui_remove_all_callbacks` - Remove all callbacks

### Element Queries (Task 18.4 - Complete)
- ✅ `ui_find_by_name` - Find by name
- ✅ `ui_find_by_tag` - Find by tag
- ✅ `ui_set_name` - Set name
- ✅ `ui_set_tag` - Set tag
- ✅ `ui_get_active` - Get active state
- ✅ `ui_set_active` - Set active state
- ✅ `ui_get_child` - Get child by index
- ✅ `ui_get_child_count` - Get child count
- ✅ `ui_get_sibling_index` - Get sibling index
- ✅ `ui_set_sibling_index` - Set sibling index
- ✅ `ui_exists` - Check if entity exists
- ✅ `ui_get_canvas` - Get canvas for element

## Implementation Status

### Completed (Task 18.1 & 18.4)
- ✅ Canvas creation API
- ✅ UI element creation API (Image, Text, Button, Panel)
- ✅ Hierarchy operations API
- ✅ Comprehensive property getters/setters for all component types
- ✅ Animation API with easing functions
- ✅ Event callback registration API
- ✅ Element query API (find by name/tag, hierarchy queries)

### Pending ECS Integration
- ⏳ All functions have API structure but need actual ECS component access
- ⏳ Animation system needs to store and update UIAnimation components
- ⏳ Event system needs to invoke registered Lua callbacks
- ⏳ Property getters/setters need to read/write actual component data

### Future Enhancements
- 🔲 Layout group creation functions
- 🔲 Prefab instantiation functions
- 🔲 Style application functions
- 🔲 Advanced component creation (Slider, Toggle, Dropdown, InputField, ScrollView)
