# Complete Migration Guide: Legacy HUD System → New UI System

## Table of Contents

1. [Overview](#overview)
2. [Before You Begin](#before-you-begin)
3. [Step-by-Step Migration Process](#step-by-step-migration-process)
4. [Code Examples: Before and After](#code-examples-before-and-after)
5. [Common Issues and Solutions](#common-issues-and-solutions)
6. [Testing Your Migration](#testing-your-migration)
7. [Rollback Procedure](#rollback-procedure)

---

## Overview

This guide walks you through migrating from the legacy HUD system to the new comprehensive UI system. The migration involves:

- Converting `.hud` files to `.uiprefab` format
- Updating Lua scripts to use the new UI API
- Refactoring custom HUD rendering code
- Testing and validating the migrated UI

**Estimated Time:** 2-4 hours for a typical game project

**Prerequisites:**
- Backup your project before starting
- Familiarity with the legacy HUD system
- Basic understanding of Lua scripting

---

## Before You Begin

### 1. Backup Your Project

Create a complete backup of your project:

```bash
# Create a backup directory
mkdir ../my_game_backup
cp -r . ../my_game_backup/

# Or use git
git checkout -b pre-ui-migration
git commit -am "Backup before UI migration"
```

### 2. Inventory Your HUD Files

List all HUD files in your project:

```bash
# On Windows (PowerShell)
Get-ChildItem -Recurse -Filter "*.hud" | Select-Object FullName

# On Linux/Mac
find . -name "*.hud" -type f
```

### 3. Review Your Lua Scripts

Identify Lua scripts that interact with the HUD system:

- Scripts that call `hud_manager` functions
- Scripts with data bindings (e.g., `bind("player_health", ...)`)
- Scripts that dynamically update HUD elements

---

## Step-by-Step Migration Process

### Step 1: Convert HUD Files to UIPrefab Format

#### Using the Migration Tool

The easiest way to convert your HUD files is using the automated migration tool:

```bash
# Navigate to your project root
cd path/to/your/project

# Run the migration tool
cargo run --package ui --bin hud_migrator -- --paths . --progress --verbose
```

**Tool Options:**
- `--paths <DIR>...` - Directories to search (default: current directory)
- `--progress` - Show detailed progress for each file
- `--dry-run` - Preview conversion without writing files
- `--no-backup` - Skip creating `.hud.backup` files (not recommended)
- `--verbose` - Show detailed conversion information

**Example Output:**
```
HUD to UIPrefab Migration Tool
============================================================

Discovering .hud files in:
  - .

✓ Found 3 .hud file(s)

Converting files...
[1/3] Processing: assets/ui/main_hud.hud ... ✓
[2/3] Processing: assets/ui/game_hud.hud ... ✓
[3/3] Processing: assets/ui/pause_menu.hud ... ✓

============================================================
Migration Summary
============================================================
Total .hud files found: 3
Successful conversions: 3
Failed conversions: 0
```

#### Manual Conversion (Advanced)

If you need fine-grained control, you can convert files programmatically:

```rust
use ui::{HudToUIPrefabConverter, HudAsset};
use std::fs;

fn convert_hud_file(hud_path: &str, output_path: &str) {
    // Load HUD file
    let hud_json = fs::read_to_string(hud_path).unwrap();
    let hud: HudAsset = serde_json::from_str(&hud_json).unwrap();
    
    // Convert to UIPrefab
    let prefab = HudToUIPrefabConverter::convert(&hud);
    
    // Save as .uiprefab
    let prefab_json = serde_json::to_string_pretty(&prefab).unwrap();
    fs::write(output_path, prefab_json).unwrap();
}
```

### Step 2: Review Conversion Notes

After conversion, check for elements that need manual attention:


```bash
# Search for conversion notes in .uiprefab files
grep -r "/*" assets/ui/*.uiprefab
```

**Common Conversion Notes:**

1. **DynamicText Elements**
   - Note: `/* DynamicText: Bind 'format_string' in Lua using set_text() */`
   - Action: Update Lua scripts to manually set text values

2. **HealthBar/ProgressBar Elements**
   - Note: `/* HealthBar: Add UIImage child with fill_method=Horizontal */`
   - Action: Add fill image children and bind fill_amount in Lua

3. **Minimap Elements**
   - Note: `/* Minimap: Custom component needed */`
   - Action: Implement custom minimap rendering

### Step 3: Update Lua Scripts

#### 3.1 Replace HUD Manager Calls

**Before (Legacy HUD System):**
```lua
-- Old: Automatic data binding
hud_manager.bind("player_health", function()
    return player.health / player.max_health
end)

hud_manager.bind("score", function()
    return game.score
end)
```

**After (New UI System):**
```lua
-- New: Manual UI updates in update loop
function update(dt)
    -- Update health bar
    local health_bar = ui.find_element("HealthBar_Fill")
    if health_bar then
        local health_percent = player.health / player.max_health
        ui.set_fill_amount(health_bar, health_percent)
    end
    
    -- Update score text
    local score_label = ui.find_element("ScoreLabel")
    if score_label then
        ui.set_text(score_label, "Score: " .. game.score)
    end
end
```

#### 3.2 Create UI Elements Dynamically

**Before (Legacy HUD System):**
```lua
-- Old: Limited to predefined HUD elements
-- No dynamic creation support
```

**After (New UI System):**
```lua
-- New: Full dynamic UI creation
function create_inventory_ui()
    local canvas = ui.create_canvas({
        render_mode = "ScreenSpaceOverlay",
        sort_order = 10
    })
    
    local panel = ui.create_panel(canvas, {
        name = "InventoryPanel",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, 0},
            size_delta = {400, 600}
        }
    })
    
    -- Add items dynamically
    for i, item in ipairs(player.inventory) do
        local item_button = ui.create_button(panel, {
            name = "Item_" .. i,
            text = item.name,
            on_click = "on_item_clicked"
        })
    end
end
```

#### 3.3 Handle UI Events

**Before (Legacy HUD System):**
```lua
-- Old: No event handling (HUD was read-only)
```

**After (New UI System):**
```lua
-- New: Full event handling
function on_button_clicked(button_entity)
    print("Button clicked: " .. ui.get_name(button_entity))
end

function on_slider_changed(slider_entity, value)
    print("Slider value: " .. value)
    -- Update game setting
    game.volume = value
end

-- Register callbacks
ui.register_callback("PlayButton", "on_click", "on_play_clicked")
ui.register_callback("VolumeSlider", "on_value_changed", "on_volume_changed")
```

### Step 4: Set Up Health Bars and Progress Bars

Health bars and progress bars require manual setup in the new system:

#### 4.1 Add Fill Images

Edit your `.uiprefab` file or use the UI Prefab Editor to add fill images:

```json
{
  "name": "HealthBar",
  "rect_transform": { ... },
  "panel": {
    "background": "health_bar_background.png"
  },
  "children": [
    {
      "name": "HealthBar_Fill",
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
        "color": [1.0, 0.2, 0.2, 1.0]
      }
    }
  ]
}
```

#### 4.2 Update Fill Amount in Lua

```lua
function update(dt)
    local health_fill = ui.find_element("HealthBar_Fill")
    if health_fill then
        local health_percent = player.health / player.max_health
        ui.set_fill_amount(health_fill, health_percent)
        
        -- Optional: Change color based on health
        if health_percent < 0.3 then
            ui.set_color(health_fill, {1.0, 0.0, 0.0, 1.0}) -- Red
        elseif health_percent < 0.6 then
            ui.set_color(health_fill, {1.0, 1.0, 0.0, 1.0}) -- Yellow
        else
            ui.set_color(health_fill, {0.0, 1.0, 0.0, 1.0}) -- Green
        end
    end
end
```

### Step 5: Implement Custom Components (Minimap, etc.)

For custom components like minimaps, you have two options:

#### Option A: Lua-Based Rendering

```lua
function update_minimap(dt)
    local minimap = ui.find_element("Minimap")
    if not minimap then return end
    
    -- Clear previous markers
    for _, child in ipairs(ui.get_children(minimap)) do
        ui.destroy(child)
    end
    
    -- Add player marker
    local player_marker = ui.create_image(minimap, {
        name = "PlayerMarker",
        sprite = "player_icon.png",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, 0},
            size_delta = {10, 10}
        }
    })
    
    -- Add enemy markers
    for _, enemy in ipairs(game.enemies) do
        local world_to_minimap = convert_world_to_minimap_pos(enemy.position)
        local enemy_marker = ui.create_image(minimap, {
            name = "EnemyMarker",
            sprite = "enemy_icon.png",
            rect_transform = {
                anchor_min = {0.5, 0.5},
                anchor_max = {0.5, 0.5},
                pivot = {0.5, 0.5},
                anchored_position = world_to_minimap,
                size_delta = {8, 8}
            },
            color = {1.0, 0.0, 0.0, 1.0}
        })
    end
end
```

#### Option B: Custom Rust Component

Create a custom UI component in Rust:

```rust
// In your game code
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIMinimapComponent {
    pub zoom: f32,
    pub follow_player: bool,
    pub show_enemies: bool,
    pub show_items: bool,
}

// Implement rendering in your game's update loop
fn update_minimap_system(world: &mut World) {
    // Query for minimap components
    // Render minimap content
    // Update UI elements
}
```

### Step 6: Test Your Migrated UI

Run your game and verify:

1. **Visual Appearance**
   - UI elements appear in correct positions
   - Anchoring works at different resolutions
   - Colors and sprites are correct

2. **Dynamic Updates**
   - Health bars update correctly
   - Score/text displays update
   - Progress bars animate smoothly

3. **Interactions**
   - Buttons respond to clicks
   - Sliders can be dragged
   - Dropdowns open and close

4. **Performance**
   - No frame rate drops
   - UI updates are smooth
   - No memory leaks

---

## Code Examples: Before and After

### Example 1: Simple HUD with Health and Score

**Before (Legacy .hud file):**
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
    },
    {
      "id": "ScoreLabel",
      "element_type": {
        "DynamicText": {
          "format": "Score: {score}",
          "font_size": 18.0,
          "color": [1.0, 1.0, 1.0, 1.0]
        }
      },
      "anchor": "TopCenter",
      "offset": [0.0, 10.0],
      "size": [150.0, 30.0],
      "visible": true
    }
  ]
}
```

**Before (Legacy Lua script):**
```lua
-- hud_manager.lua
function init()
    hud_manager.bind("player.health", function()
        return player.health / player.max_health
    end)
    
    hud_manager.bind("score", function()
        return game.score
    end)
end
```

**After (New .uiprefab file):**
```json
{
  "name": "GameHUD",
  "root": {
    "name": "Canvas",
    "rect_transform": {
      "anchor_min": [0.0, 0.0],
      "anchor_max": [1.0, 1.0],
      "pivot": [0.5, 0.5],
      "anchored_position": [0, 0],
      "size_delta": [0, 0]
    },
    "children": [
      {
        "name": "HealthBar",
        "rect_transform": {
          "anchor_min": [0.0, 1.0],
          "anchor_max": [0.0, 1.0],
          "pivot": [0.0, 1.0],
          "anchored_position": [10.0, -10.0],
          "size_delta": [200.0, 20.0]
        },
        "panel": {
          "background": "health_bar_bg.png"
        },
        "children": [
          {
            "name": "HealthBar_Fill",
            "image": {
              "sprite": "health_bar_fill.png",
              "image_type": "Filled",
              "fill_method": "Horizontal",
              "fill_amount": 1.0
            }
          }
        ]
      },
      {
        "name": "ScoreLabel",
        "rect_transform": {
          "anchor_min": [0.5, 1.0],
          "anchor_max": [0.5, 1.0],
          "pivot": [0.5, 1.0],
          "anchored_position": [0.0, -10.0],
          "size_delta": [150.0, 30.0]
        },
        "text": {
          "text": "Score: 0",
          "font": "default",
          "font_size": 18.0,
          "color": [1.0, 1.0, 1.0, 1.0],
          "alignment": "MiddleCenter"
        }
      }
    ]
  }
}
```

**After (New Lua script):**
```lua
-- game_ui.lua
local health_bar_fill = nil
local score_label = nil

function init()
    -- Cache UI element references
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

### Example 2: Interactive Pause Menu

**After (New UI with interactions):**
```lua
-- pause_menu.lua
local pause_menu = nil
local is_paused = false

function init()
    -- Create pause menu
    local canvas = ui.create_canvas({
        render_mode = "ScreenSpaceOverlay",
        sort_order = 100
    })
    
    pause_menu = ui.create_panel(canvas, {
        name = "PauseMenu",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, 0},
            size_delta = {400, 500}
        },
        background = "panel_bg.png"
    })
    
    -- Add title
    ui.create_text(pause_menu, {
        name = "Title",
        text = "PAUSED",
        font_size = 36.0,
        rect_transform = {
            anchor_min = {0.5, 1.0},
            anchor_max = {0.5, 1.0},
            pivot = {0.5, 1.0},
            anchored_position = {0, -50},
            size_delta = {300, 50}
        }
    })
    
    -- Add resume button
    local resume_btn = ui.create_button(pause_menu, {
        name = "ResumeButton",
        text = "Resume",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, 50},
            size_delta = {200, 50}
        }
    })
    ui.register_callback(resume_btn, "on_click", "on_resume_clicked")
    
    -- Add settings button
    local settings_btn = ui.create_button(pause_menu, {
        name = "SettingsButton",
        text = "Settings",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, -10},
            size_delta = {200, 50}
        }
    })
    ui.register_callback(settings_btn, "on_click", "on_settings_clicked")
    
    -- Add quit button
    local quit_btn = ui.create_button(pause_menu, {
        name = "QuitButton",
        text = "Quit to Menu",
        rect_transform = {
            anchor_min = {0.5, 0.5},
            anchor_max = {0.5, 0.5},
            pivot = {0.5, 0.5},
            anchored_position = {0, -70},
            size_delta = {200, 50}
        }
    })
    ui.register_callback(quit_btn, "on_click", "on_quit_clicked")
    
    -- Hide initially
    ui.set_active(pause_menu, false)
end

function update(dt)
    -- Toggle pause menu with ESC key
    if input.is_key_pressed("Escape") then
        is_paused = not is_paused
        ui.set_active(pause_menu, is_paused)
        game.set_paused(is_paused)
    end
end

function on_resume_clicked()
    is_paused = false
    ui.set_active(pause_menu, false)
    game.set_paused(false)
end

function on_settings_clicked()
    -- Open settings menu
    print("Opening settings...")
end

function on_quit_clicked()
    -- Return to main menu
    game.load_scene("MainMenu")
end
```

---

## Common Issues and Solutions

### Issue 1: UI Elements Not Appearing

**Symptoms:**
- Converted UI doesn't show up in game
- Elements are invisible or off-screen

**Solutions:**

1. **Check Canvas Creation:**
   ```lua
   -- Make sure canvas is created and active
   local canvas = ui.find_element("Canvas")
   if not canvas then
       print("ERROR: Canvas not found!")
   end
   ```

2. **Verify Anchor Settings:**
   ```lua
   -- Check if anchors are valid (0-1 range)
   local rect = ui.get_rect_transform(element)
   print("Anchor min:", rect.anchor_min)
   print("Anchor max:", rect.anchor_max)
   ```

3. **Check Z-Order:**
   ```lua
   -- Ensure element is not behind other elements
   ui.set_z_order(element, 10)
   ```

4. **Verify Visibility:**
   ```lua
   -- Make sure element is active
   ui.set_active(element, true)
   ```

### Issue 2: Health Bars Not Updating

**Symptoms:**
- Health bar fill doesn't change
- Fill amount stuck at 0 or 1

**Solutions:**

1. **Check Element Reference:**
   ```lua
   local health_fill = ui.find_element("HealthBar_Fill")
   if not health_fill then
       print("ERROR: Health bar fill not found!")
       -- Check the exact name in your .uiprefab file
   end
   ```

2. **Verify Fill Method:**
   ```json
   // In .uiprefab file
   "image": {
       "image_type": "Filled",  // Must be "Filled"
       "fill_method": "Horizontal",  // Or "Vertical", "Radial90", etc.
       "fill_amount": 1.0
   }
   ```

3. **Check Value Range:**
   ```lua
   -- fill_amount must be between 0.0 and 1.0
   local health_percent = math.max(0.0, math.min(1.0, player.health / player.max_health))
   ui.set_fill_amount(health_fill, health_percent)
   ```

### Issue 3: Text Not Updating

**Symptoms:**
- Dynamic text shows placeholder value
- Text doesn't change when game state updates

**Solutions:**

1. **Call set_text in Update Loop:**
   ```lua
   function update(dt)
       -- Must be called every frame for dynamic text
       local score_label = ui.find_element("ScoreLabel")
       ui.set_text(score_label, "Score: " .. game.score)
   end
   ```

2. **Check String Formatting:**
   ```lua
   -- Use Lua string concatenation or string.format
   ui.set_text(label, string.format("Health: %d/%d", player.health, player.max_health))
   ```

### Issue 4: Buttons Not Responding

**Symptoms:**
- Clicking buttons does nothing
- No callback is triggered

**Solutions:**

1. **Register Callback Correctly:**
   ```lua
   -- Make sure callback is registered
   ui.register_callback(button, "on_click", "my_callback_function")
   
   -- And function exists
   function my_callback_function(button_entity)
       print("Button clicked!")
   end
   ```

2. **Check Raycast Target:**
   ```json
   // In .uiprefab file
   "ui_element": {
       "raycast_target": true,  // Must be true
       "interactable": true     // Must be true
   }
   ```

3. **Verify Button Component:**
   ```json
   "button": {
       "transition": "ColorTint",
       "on_click": "my_callback_function"
   }
   ```

### Issue 5: Layout Not Working

**Symptoms:**
- Elements overlap incorrectly
- Layout groups don't arrange children

**Solutions:**

1. **Check Layout Component:**
   ```json
   "horizontal_layout": {
       "padding": [10, 10, 10, 10],
       "spacing": 5.0,
       "child_alignment": "MiddleCenter"
   }
   ```

2. **Verify Parent-Child Hierarchy:**
   ```lua
   -- Children must be direct children of layout group
   local parent = ui.get_parent(child)
   print("Parent:", ui.get_name(parent))
   ```

3. **Force Layout Rebuild:**
   ```lua
   -- Mark layout as dirty to force recalculation
   ui.mark_layout_dirty(layout_group)
   ```

### Issue 6: Performance Issues

**Symptoms:**
- Frame rate drops with UI visible
- Stuttering when updating UI

**Solutions:**

1. **Cache Element References:**
   ```lua
   -- BAD: Finding elements every frame
   function update(dt)
       local label = ui.find_element("ScoreLabel")  -- Slow!
       ui.set_text(label, "Score: " .. score)
   end
   
   -- GOOD: Cache references
   local score_label = nil
   function init()
       score_label = ui.find_element("ScoreLabel")
   end
   function update(dt)
       ui.set_text(score_label, "Score: " .. score)  -- Fast!
   end
   ```

2. **Batch UI Updates:**
   ```lua
   -- Update UI only when values change
   local last_health = -1
   function update(dt)
       if player.health ~= last_health then
           ui.set_fill_amount(health_fill, player.health / player.max_health)
           last_health = player.health
       end
   end
   ```

3. **Use UI Batching:**
   ```lua
   -- Group similar elements to enable batching
   -- Use same sprites/materials when possible
   ```

### Issue 7: Resolution Independence Not Working

**Symptoms:**
- UI looks wrong at different resolutions
- Elements are too small or too large

**Solutions:**

1. **Configure Canvas Scaler:**
   ```json
   "canvas": {
       "scaler": {
           "mode": "ScaleWithScreenSize",
           "reference_resolution": [1920.0, 1080.0],
           "match_width_or_height": 0.5
       }
   }
   ```

2. **Use Proper Anchoring:**
   ```lua
   -- For elements that should stay in corners
   -- Use corner anchors (0,0), (1,1), etc.
   
   -- For elements that should stretch
   -- Use stretched anchors
   rect_transform = {
       anchor_min = {0.0, 0.0},
       anchor_max = {1.0, 1.0},  -- Stretches to fill parent
       size_delta = {-20, -20}   -- With margins
   }
   ```

---

## Testing Your Migration

### Automated Testing

Create a test script to verify your migration:

```lua
-- test_ui_migration.lua
function test_all_elements_exist()
    local required_elements = {
        "HealthBar_Fill",
        "ScoreLabel",
        "Minimap",
        "InventoryPanel"
    }
    
    for _, name in ipairs(required_elements) do
        local element = ui.find_element(name)
        if not element then
            print("FAIL: Element not found: " .. name)
            return false
        end
    end
    
    print("PASS: All elements exist")
    return true
end

function test_health_bar_updates()
    local health_fill = ui.find_element("HealthBar_Fill")
    
    -- Test different values
    ui.set_fill_amount(health_fill, 1.0)
    assert(ui.get_fill_amount(health_fill) == 1.0)
    
    ui.set_fill_amount(health_fill, 0.5)
    assert(ui.get_fill_amount(health_fill) == 0.5)
    
    ui.set_fill_amount(health_fill, 0.0)
    assert(ui.get_fill_amount(health_fill) == 0.0)
    
    print("PASS: Health bar updates correctly")
    return true
end

function test_button_callbacks()
    local button = ui.find_element("TestButton")
    local callback_called = false
    
    function test_callback()
        callback_called = true
    end
    
    ui.register_callback(button, "on_click", "test_callback")
    -- Simulate click
    ui.simulate_click(button)
    
    assert(callback_called, "Callback was not called")
    print("PASS: Button callbacks work")
    return true
end

-- Run all tests
function run_migration_tests()
    print("Running UI Migration Tests...")
    print("================================")
    
    local tests = {
        test_all_elements_exist,
        test_health_bar_updates,
        test_button_callbacks
    }
    
    local passed = 0
    local failed = 0
    
    for _, test in ipairs(tests) do
        if test() then
            passed = passed + 1
        else
            failed = failed + 1
        end
    end
    
    print("================================")
    print(string.format("Results: %d passed, %d failed", passed, failed))
end
```

### Manual Testing Checklist

- [ ] All UI elements appear in correct positions
- [ ] UI scales correctly at different resolutions (test 1920x1080, 1280x720, 800x600)
- [ ] Health bars update smoothly
- [ ] Score/text displays update correctly
- [ ] Buttons respond to clicks
- [ ] Hover effects work
- [ ] Sliders can be dragged
- [ ] Dropdowns open and close
- [ ] Input fields accept text
- [ ] Scroll views scroll correctly
- [ ] Animations play smoothly
- [ ] No visual glitches or artifacts
- [ ] Performance is acceptable (60 FPS)
- [ ] No console errors or warnings

### Visual Regression Testing

Take screenshots before and after migration:

```lua
-- screenshot_comparison.lua
function capture_ui_screenshot(filename)
    -- Capture screenshot
    game.capture_screenshot(filename)
end

-- Before migration
capture_ui_screenshot("ui_before.png")

-- After migration
capture_ui_screenshot("ui_after.png")

-- Compare manually or use image diff tool
```

---

## Rollback Procedure

If you encounter critical issues and need to rollback:

### Option 1: Restore from Backup

```bash
# Restore entire project
rm -rf .
cp -r ../my_game_backup/* .

# Or use git
git checkout pre-ui-migration
```

### Option 2: Restore Individual Files

```bash
# Restore .hud files from backups
cp assets/ui/*.hud.backup assets/ui/*.hud

# Restore Lua scripts from git
git checkout HEAD -- scripts/
```

### Option 3: Gradual Rollback

Keep both systems running temporarily:

1. Keep `.hud` files alongside `.uiprefab` files
2. Use feature flag to switch between systems
3. Migrate one screen at a time
4. Test thoroughly before removing old files

---

## Next Steps

After successful migration:

1. **Remove Backup Files**
   ```bash
   find . -name "*.hud.backup" -delete
   find . -name "*.hud" -delete  # Only after verification!
   ```

2. **Update Documentation**
   - Update your game's README
   - Document any custom UI components
   - Create UI style guide for your team

3. **Explore New Features**
   - Try advanced components (Scroll View, Dropdown, etc.)
   - Experiment with UI animations
   - Create reusable UI prefabs
   - Implement UI themes

4. **Optimize Performance**
   - Profile UI rendering
   - Optimize Lua scripts
   - Use UI batching effectively

5. **Share Feedback**
   - Report any issues or bugs
   - Suggest improvements
   - Share your migration experience

---

## Additional Resources

- **UI System Documentation**: [README.md](README.md)
- **Lua API Reference**: [LUA_API.md](LUA_API.md)
- **API Changes**: [API_CHANGES.md](API_CHANGES.md)
- **Migration Tool Guide**: [MIGRATION_TOOL_GUIDE.md](MIGRATION_TOOL_GUIDE.md)
- **HUD Converter Guide**: [HUD_CONVERTER_GUIDE.md](HUD_CONVERTER_GUIDE.md)
- **Example Scripts**: [examples/](examples/)

---

## Support

If you encounter issues during migration:

1. Check the [Common Issues](#common-issues-and-solutions) section
2. Review the [API Changes](API_CHANGES.md) document
3. Look at example scripts in `ui/examples/`
4. Create an issue in the project tracker with:
   - Description of the problem
   - Steps to reproduce
   - Error messages or logs
   - Your `.uiprefab` file (if relevant)

---

**Migration Status:** ✅ Complete

**Last Updated:** December 2025

**Version:** 1.0.0
