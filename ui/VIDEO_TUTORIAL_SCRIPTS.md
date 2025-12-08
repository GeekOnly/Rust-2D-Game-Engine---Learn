# Video Tutorial Scripts for UI System Migration

## Overview

This document contains detailed scripts for creating video tutorials about the new UI system and migration process. Each tutorial includes:

- Target audience
- Duration estimate
- Required materials
- Step-by-step script
- Key points to emphasize
- Common mistakes to avoid

---

## Tutorial 1: Introduction to the New UI System

**Duration:** 10-12 minutes  
**Target Audience:** All developers  
**Prerequisites:** None

### Script

**[0:00-0:30] Introduction**

"Welcome to the XS Game Engine UI System tutorial. In this video, we'll explore the new comprehensive UI system that replaces the legacy HUD system. This new system provides powerful features comparable to Unity's Canvas UI and Unreal Engine's UMG."

**[0:30-2:00] Overview of New Features**

"Let's start with what's new:

1. **Canvas-based rendering** - Multiple render modes for different use cases
2. **Flexible RectTransform** - Powerful anchoring and positioning system
3. **Rich component library** - 15+ UI components including buttons, sliders, scroll views
4. **Event system** - Full support for user interactions
5. **Animation system** - Smooth UI animations with easing functions
6. **Layout system** - Automatic positioning with layout groups
7. **Lua integration** - Complete API for runtime UI creation

Let me show you each of these in action."

**[2:00-4:00] Canvas System Demo**

"First, let's look at the Canvas system. A Canvas is the root container for all UI elements.

[Show code editor with canvas creation]

```lua
local canvas = ui.create_canvas({
    render_mode = "ScreenSpaceOverlay",
    sort_order = 0
})
```

The Canvas has three render modes:
- Screen Space Overlay: Renders on top of everything
- Screen Space Camera: Renders at a distance from the camera
- World Space: Renders as part of the 3D world

[Show visual examples of each mode]"

**[4:00-6:00] RectTransform and Anchoring**

"The RectTransform system is the heart of the UI positioning. Unlike the old 9-position anchor system, RectTransform gives you complete control.

[Show anchor visualization]

Anchors are normalized values from 0 to 1:
- (0, 0) is bottom-left
- (1, 1) is top-right
- (0.5, 0.5) is center

You can even stretch elements by setting different anchor_min and anchor_max values.

[Demonstrate stretching]"

**[6:00-8:00] UI Components**

"The new system includes a rich set of components:

[Show each component with quick demo]
- UIImage: Display sprites and textures
- UIText: Render text with alignment and overflow options
- UIButton: Interactive buttons with visual states
- UIPanel: Container with background
- UISlider: Value selection with draggable handle
- UIToggle: Checkbox/toggle switch
- UIDropdown: Selection from list of options
- UIInputField: Text input
- UIScrollView: Scrollable content with clipping

Each component is fully customizable and can be created in Lua or defined in prefab files."

**[8:00-10:00] Layout System**

"One of the most powerful features is the automatic layout system.

[Show layout group demo]

Layout groups automatically position their children:
- Horizontal Layout: Arranges in a row
- Vertical Layout: Arranges in a column
- Grid Layout: Arranges in a grid

You can control spacing, padding, and alignment. The layout updates automatically when you add or remove children."

**[10:00-12:00] Conclusion**

"That's a quick overview of the new UI system. In the next tutorials, we'll dive deeper into:
- Migrating from the legacy HUD system
- Using the UI Prefab Editor
- Creating interactive UIs with Lua

Thanks for watching, and happy coding!"

### Key Points to Emphasize

- The new system is much more powerful than the legacy HUD system
- RectTransform provides flexible positioning
- Layout groups save time on manual positioning
- Full Lua API for runtime UI creation

### Common Mistakes to Avoid

- Don't confuse anchor coordinates (0-1) with pixel coordinates
- Remember that Y-axis is inverted (0 is bottom, 1 is top)
- Always create a Canvas before adding UI elements

---

## Tutorial 2: Migrating from Legacy HUD System

**Duration:** 15-20 minutes  
**Target Audience:** Developers with existing HUD files  
**Prerequisites:** Tutorial 1

### Script

**[0:00-1:00] Introduction**

"In this tutorial, we'll walk through migrating from the legacy HUD system to the new UI system. We'll convert a real game HUD step by step, showing you exactly what changes and how to update your code."

**[1:00-3:00] Before We Begin**

"Before starting the migration, always:

1. Backup your project
2. Inventory your HUD files
3. Review your Lua scripts

[Show terminal commands for finding HUD files]

```bash
find . -name '*.hud' -type f
```

This shows all HUD files that need conversion."

**[3:00-6:00] Using the Migration Tool**

"The easiest way to migrate is using the automated tool.

[Show terminal]

```bash
cargo run --package ui --bin hud_migrator -- --paths . --progress
```

[Run the tool and show output]

The tool:
1. Finds all .hud files
2. Creates backups (.hud.backup)
3. Converts to .uiprefab format
4. Adds conversion notes for manual steps

[Show the generated .uiprefab file]"

**[6:00-10:00] Understanding Conversion Notes**

"After conversion, check for elements that need manual attention.

[Open converted .uiprefab file]

Look for comments like:
- `/* DynamicText: Bind in Lua */`
- `/* HealthBar: Add fill image */`
- `/* Minimap: Custom component needed */`

Let's handle each of these.

[Show examples of fixing each type]"

**[10:00-15:00] Updating Lua Scripts**

"The biggest change is how we update UI elements. The old system used automatic data binding. The new system requires manual updates.

[Show side-by-side comparison]

**Old way:**
```lua
hud_manager.bind('player.health', function()
    return player.health / player.max_health
end)
```

**New way:**
```lua
function update(dt)
    local health_fill = ui.find_element('HealthBar_Fill')
    ui.set_fill_amount(health_fill, player.health / player.max_health)
end
```

[Demonstrate updating a complete Lua script]"

**[15:00-18:00] Testing the Migration**

"After migration, thoroughly test your UI:

[Show game running]

1. Check visual appearance at different resolutions
2. Verify dynamic updates work
3. Test all interactions
4. Check performance

[Demonstrate each test]"

**[18:00-20:00] Conclusion**

"That's the complete migration process! Key takeaways:

1. Use the migration tool for automatic conversion
2. Review and handle conversion notes
3. Update Lua scripts for manual UI updates
4. Test thoroughly before removing old files

For detailed information, check the Migration Guide in the documentation."

### Key Points to Emphasize

- Always backup before migrating
- The migration tool handles most of the work
- Manual updates are needed for dynamic content
- Test thoroughly at different resolutions

### Common Mistakes to Avoid

- Don't delete .hud files until migration is verified
- Don't forget to update Lua scripts
- Don't skip testing at different resolutions

---

## Tutorial 3: Using the UI Prefab Editor

**Duration:** 12-15 minutes  
**Target Audience:** All developers  
**Prerequisites:** Tutorial 1

### Script

**[0:00-1:00] Introduction**

"In this tutorial, we'll explore the UI Prefab Editor - a visual tool for creating and editing UI layouts. Think of it as a Unity-style UI editor built right into the engine."

**[1:00-3:00] Opening the Editor**

"To open the UI Prefab Editor:

[Show menu navigation]

1. Go to Window → UI Prefab Editor
2. Or press Ctrl+Shift+U

[Show editor interface]

The editor has four main panels:
- Canvas: Visual editing area
- Hierarchy: Tree view of UI elements
- Inspector: Component properties
- Component Palette: Available UI components"

**[3:00-6:00] Creating a Simple UI**

"Let's create a simple game HUD from scratch.

[Start with empty canvas]

1. First, create a Canvas
   [Click 'New Canvas' button]

2. Add a health bar
   [Drag Panel from palette]
   [Position in top-left]
   [Add Image child for fill]

3. Add a score label
   [Drag Text from palette]
   [Position in top-center]
   [Set text properties]

[Show the completed simple HUD]"

**[6:00-9:00] Working with RectTransform**

"The RectTransform editor is powerful. Let me show you the key features.

[Select an element]

1. Anchor presets
   [Show anchor preset buttons]
   [Click different presets to demonstrate]

2. Pivot point
   [Show pivot visualization]
   [Drag pivot point]

3. Size handles
   [Drag corner and edge handles]

4. Position dragging
   [Drag element around canvas]

The visual feedback makes it easy to understand how anchoring works."

**[9:00-12:00] Advanced Features**

"The editor has several advanced features:

1. **Layout Groups**
   [Add Horizontal Layout Group]
   [Add children and show automatic positioning]

2. **Multi-resolution Preview**
   [Switch between different resolutions]
   [Show how anchored elements adapt]

3. **Undo/Redo**
   [Make changes and undo them]
   [Keyboard shortcuts: Ctrl+Z, Ctrl+Y]

4. **Hierarchy Management**
   [Drag elements to reparent]
   [Show/hide elements]
   [Duplicate elements]"

**[12:00-15:00] Saving and Using Prefabs**

"Once you've created your UI, save it as a prefab.

[File → Save Prefab]

The prefab can be:
1. Loaded in other scenes
2. Instantiated from Lua
3. Used as a template

[Show Lua code to load prefab]

```lua
local prefab = ui.load_prefab('game_hud.uiprefab')
local instance = ui.instantiate_prefab(prefab)
```

That's the UI Prefab Editor! Practice with it to become proficient."

### Key Points to Emphasize

- Visual editing is faster than manual JSON editing
- Anchor presets make positioning easy
- Multi-resolution preview ensures responsive design
- Prefabs are reusable across scenes

### Common Mistakes to Avoid

- Don't forget to save your work frequently
- Use anchor presets instead of manual anchor values
- Test at multiple resolutions before finalizing

---

## Tutorial 4: Creating Interactive UIs with Lua

**Duration:** 15-18 minutes  
**Target Audience:** Developers comfortable with Lua  
**Prerequisites:** Tutorials 1 and 2

### Script

**[0:00-1:00] Introduction**

"In this tutorial, we'll create a complete interactive UI system using Lua. We'll build a pause menu with buttons, a settings screen with sliders, and an inventory system - all created dynamically at runtime."

**[1:00-4:00] Creating a Pause Menu**

"Let's start with a pause menu.

[Show code editor]

```lua
function create_pause_menu()
    -- Create canvas
    local canvas = ui.create_canvas({
        render_mode = 'ScreenSpaceOverlay',
        sort_order = 100
    })
    
    -- Create panel
    local panel = ui.create_panel(canvas, {
        name = 'PauseMenu',
        background = 'panel_bg.png',
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, 0},
            size_delta = {400, 500}
        }
    })
    
    -- Add buttons
    create_button(panel, 'Resume', {0, 50}, 'on_resume')
    create_button(panel, 'Settings', {0, -10}, 'on_settings')
    create_button(panel, 'Quit', {0, -70}, 'on_quit')
    
    return panel
end
```

[Run the code and show the menu]"

**[4:00-7:00] Handling Button Events**

"Now let's handle button clicks.

[Show callback functions]

```lua
function on_resume()
    ui.set_active(pause_menu, false)
    game.set_paused(false)
end

function on_settings()
    ui.set_active(pause_menu, false)
    ui.set_active(settings_menu, true)
end

function on_quit()
    game.load_scene('MainMenu')
end
```

[Demonstrate clicking buttons]

The event system makes it easy to create responsive UIs."

**[7:00-11:00] Creating a Settings Screen**

"Let's create a settings screen with sliders.

[Show code]

```lua
function create_settings_menu()
    local panel = ui.create_panel(canvas, {...})
    
    -- Volume slider
    local volume_slider = ui.create_slider(panel, {
        name = 'VolumeSlider',
        min_value = 0.0,
        max_value = 1.0,
        value = game.volume
    })
    ui.register_callback(volume_slider, 'on_value_changed', 'on_volume_changed')
    
    -- Brightness slider
    local brightness_slider = ui.create_slider(panel, {...})
    ui.register_callback(brightness_slider, 'on_value_changed', 'on_brightness_changed')
    
    return panel
end

function on_volume_changed(slider, value)
    game.volume = value
    audio.set_master_volume(value)
end
```

[Demonstrate adjusting sliders]"

**[11:00-15:00] Dynamic Inventory System**

"Finally, let's create a dynamic inventory that updates as items change.

[Show code]

```lua
function update_inventory()
    -- Clear existing items
    local container = ui.find_element('InventoryContainer')
    for _, child in ipairs(ui.get_children(container)) do
        ui.destroy(child)
    end
    
    -- Add current items
    for i, item in ipairs(player.inventory) do
        local slot = ui.create_button(container, {
            name = 'ItemSlot_' .. i,
            sprite = item.icon,
            rect_transform = {
                anchor_min = {0, 1},
                anchor_max = {0, 1},
                pivot = {0, 1},
                anchored_position = {10 + ((i-1) % 5) * 60, -10 - math.floor((i-1) / 5) * 60},
                size_delta = {50, 50}
            }
        })
        ui.register_callback(slot, 'on_click', 'on_item_clicked')
    end
end

function on_item_clicked(button)
    local item_index = tonumber(string.match(ui.get_name(button), '%d+'))
    use_item(player.inventory[item_index])
end
```

[Demonstrate adding/removing items]"

**[15:00-18:00] Best Practices and Tips**

"Here are some best practices for Lua UI:

1. **Cache element references**
   ```lua
   -- Cache in init()
   local health_bar = ui.find_element('HealthBar')
   
   -- Use in update()
   ui.set_fill_amount(health_bar, health)
   ```

2. **Batch updates**
   ```lua
   -- Only update when values change
   if health ~= last_health then
       ui.set_fill_amount(health_bar, health)
       last_health = health
   end
   ```

3. **Clean up properly**
   ```lua
   function on_scene_unload()
       ui.destroy_recursive(main_canvas)
   end
   ```

That's everything you need to create interactive UIs with Lua!"

### Key Points to Emphasize

- Lua provides complete control over UI creation
- Event callbacks make interactions easy
- Cache element references for performance
- Clean up UI when changing scenes

### Common Mistakes to Avoid

- Don't call find_element() every frame
- Don't forget to register callbacks
- Don't forget to clean up UI when done

---

## Production Notes

### Equipment Needed

- Screen recording software (OBS Studio, Camtasia, etc.)
- Microphone for clear audio
- Code editor with syntax highlighting
- Running instance of the game engine
- Sample project with HUD files

### Recording Tips

1. **Resolution**: Record at 1920x1080 for best quality
2. **Frame Rate**: 30 or 60 FPS
3. **Audio**: Use a good microphone, minimize background noise
4. **Pacing**: Speak clearly and not too fast
5. **Editing**: Add captions for key points
6. **Length**: Keep videos under 20 minutes

### Post-Production

1. Add intro/outro graphics
2. Add chapter markers for easy navigation
3. Include links to documentation in description
4. Add captions/subtitles
5. Create thumbnail images

### Publishing

1. Upload to YouTube or similar platform
2. Create playlist for all tutorials
3. Link from documentation
4. Share on project website/forum
5. Include in release notes

---

## Additional Tutorial Ideas

### Tutorial 5: Advanced UI Animations
- Tween animations
- Easing functions
- Chaining animations
- Animation callbacks

### Tutorial 6: Responsive UI Design
- Canvas scaler modes
- Anchor strategies
- Safe area handling
- Multi-resolution testing

### Tutorial 7: UI Performance Optimization
- Batching strategies
- Caching element references
- Efficient updates
- Profiling UI performance

### Tutorial 8: Custom UI Components
- Creating custom components in Rust
- Integrating with Lua
- Custom rendering
- Reusable patterns

---

**Last Updated:** December 2025

**Version:** 1.0.0
