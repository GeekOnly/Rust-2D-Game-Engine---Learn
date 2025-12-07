# HUD System Migration Note

## Old HUD System Removed

The old HUD system (`hud_manager.lua` and related `.hud` files) has been deprecated and removed from the engine.

## New UI System

The engine now uses a comprehensive UI system with full Lua bindings. See the UI crate documentation for details.

### Migration Path

To migrate from the old HUD system to the new UI system:

1. **Use the new UI Lua API** - The new system provides comprehensive Lua bindings for creating and managing UI elements programmatically.

2. **Convert `.hud` files to `.uiprefab` files** - Use the migration tool:
   ```bash
   cargo run --bin hud_migrator -- --input path/to/file.hud --output path/to/file.uiprefab
   ```

3. **Update Lua scripts** - Replace old HUD manager code with new UI system calls. See `ui/examples/lua_ui_example.lua` for examples.

### Example: Creating a Health Bar

Old system (deprecated):
```lua
-- Old HUD system - NO LONGER SUPPORTED
HudManager.player_health = 1.0
```

New system:
```lua
-- New UI system
local canvas = ui_create_canvas({
    render_mode = "ScreenSpaceOverlay",
    sort_order = 0
})

local health_bar = ui_create_image(canvas, {
    name = "HealthBar",
    sprite = "health_bar_fill",
    color = {1.0, 0.0, 0.0, 1.0},
    rect_transform = {
        anchor_min = {0.0, 1.0},
        anchor_max = {0.0, 1.0},
        pivot = {0.0, 1.0},
        anchored_position = {20.0, -20.0},
        size_delta = {200.0, 30.0}
    }
})

-- Update health bar fill
function update_health(current, max)
    local fill_amount = current / max
    ui_set_fill_amount(health_bar, fill_amount)
end
```

### Resources

- UI System Documentation: `ui/README.md`
- Lua API Reference: `ui/LUA_API.md`
- Example Scripts: `ui/examples/lua_ui_example.lua`
- Migration Tool Guide: `ui/MIGRATION_TOOL_GUIDE.md`

### Deprecated Files

The following files are deprecated and should be migrated:
- `hud_manager.lua` - Replace with new UI Lua API
- `assets/ui/*.hud` - Convert to `.uiprefab` format
