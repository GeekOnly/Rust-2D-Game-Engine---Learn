# Widget Editor Specification

Visual UI editor for creating and editing HUD/UI widgets (Unreal UMG style)

## Overview

A WYSIWYG editor for creating game UI without manually editing JSON files.

## Features

### Phase 1: Basic Visual Editor ⭐ PRIORITY

#### 1.1 Widget Canvas
- Visual representation of HUD
- Show all elements in their positions
- Click to select element
- Drag to move element
- Show selection outline

#### 1.2 Properties Inspector
- Show selected element properties
- Edit position (X, Y)
- Edit size (Width, Height)
- Edit anchor point
- Edit colors
- Edit text content
- Edit font size

#### 1.3 Element List (Hierarchy)
- List all elements
- Click to select
- Show/hide elements
- Reorder elements (z-order)

#### 1.4 Save/Load
- Load existing .hud files
- Save changes back to file
- Auto-save support

### Phase 2: Widget Creation

#### 2.1 Widget Palette
```
Common:
- Text
- Image
- Button
- Panel (Container)

Game UI:
- Health Bar
- Progress Bar
- Minimap
- Damage Number

Custom:
- User-defined widgets
```

#### 2.2 Creation Flow
1. Select widget from palette
2. Click on canvas to place
3. Or drag from palette to canvas
4. Auto-generate unique ID
5. Set default properties

### Phase 3: Advanced Editing

#### 3.1 Transform Tools
- Move tool (drag)
- Resize tool (handles)
- Rotate tool (future)

#### 3.2 Alignment
- Align left/center/right
- Align top/middle/bottom
- Distribute evenly
- Snap to grid

#### 3.3 Layout Helpers
- Show safe area
- Show anchor points
- Show guidelines
- Ruler/Grid

#### 3.4 Undo/Redo
- Command pattern
- Undo stack
- Redo stack

### Phase 4: Script Integration

#### 4.1 Widget Scripts
```lua
-- widget_script.lua
function on_create(widget)
    -- Initialize widget
end

function on_update(widget, dt)
    -- Update logic
end

function on_click(widget)
    -- Handle click
end
```

#### 4.2 Data Binding
```lua
-- Bind to game data
widget:bind("health", function()
    return player.health / player.max_health
end)
```

#### 4.3 Events
- OnClick
- OnHover
- OnShow
- OnHide
- OnUpdate

#### 4.4 Animations
```lua
-- Fade in animation
widget:animate({
    property = "alpha",
    from = 0.0,
    to = 1.0,
    duration = 0.5,
    easing = "ease_in_out"
})
```

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│ File  Edit  View  Widget                                │
├──────────┬────────────────────────────┬─────────────────┤
│          │                            │                 │
│ Palette  │      Canvas                │   Properties    │
│          │                            │                 │
│ □ Text   │  ┌──────────────────────┐  │ Selected: Text  │
│ □ Image  │  │                      │  │                 │
│ □ Button │  │   [Health Bar]       │  │ Position:       │
│ □ Panel  │  │                      │  │   X: 20         │
│          │  │   "Player Name"      │  │   Y: 20         │
│ ▼ Game   │  │                      │  │                 │
│ □ Health │  │                      │  │ Size:           │
│ □ Progr. │  │                      │  │   W: 200        │
│          │  └──────────────────────┘  │   H: 30         │
│          │                            │                 │
│          │  Resolution: 1920x1080    │ Anchor: TopLeft │
│          │  Scale: 100%              │                 │
├──────────┴────────────────────────────┴─────────────────┤
│ Hierarchy                                                │
│ ├─ player_health (HealthBar)                            │
│ ├─ player_name (Text)                                   │
│ └─ minimap (Container)                                  │
│     ├─ minimap_bg (Image)                               │
│     └─ minimap_player (Image)                           │
└─────────────────────────────────────────────────────────┘
```

## File Format

Widget Editor works with .hud files (JSON):

```json
{
  "name": "Main HUD",
  "elements": [
    {
      "id": "health_bar",
      "element_type": {
        "type": "HealthBar",
        "binding": "player.health"
      },
      "anchor": "TopLeft",
      "offset": [20.0, 20.0],
      "size": [200.0, 30.0],
      "script": "widgets/health_bar.lua"
    }
  ]
}
```

## Technical Architecture

### Components

**1. WidgetEditor (Main)**
```rust
pub struct WidgetEditor {
    canvas: WidgetCanvas,
    palette: WidgetPalette,
    properties: PropertiesPanel,
    hierarchy: HierarchyPanel,
    current_hud: Option<HudAsset>,
    selected_element: Option<String>,
    undo_stack: Vec<EditorCommand>,
}
```

**2. WidgetCanvas**
```rust
pub struct WidgetCanvas {
    zoom: f32,
    pan: Vec2,
    grid_size: f32,
    show_grid: bool,
    show_safe_area: bool,
    resolution: GameViewResolution,
}
```

**3. WidgetPalette**
```rust
pub struct WidgetPalette {
    categories: Vec<WidgetCategory>,
    search_filter: String,
}

pub enum WidgetCategory {
    Common,
    GameUI,
    Custom,
}
```

**4. PropertiesPanel**
```rust
pub struct PropertiesPanel {
    element: Option<HudElement>,
    dirty: bool,
}
```

### Interaction Flow

```
User Action → Editor State → Update Canvas → Save to File

1. Click element → Select
2. Drag element → Update position
3. Edit property → Update element
4. Save → Write to .hud file
```

## Comparison with Other Engines

| Feature | Unreal UMG | Unity UI | This Engine |
|---------|-----------|----------|-------------|
| Visual Editor | ✅ | ✅ | 🔨 Phase 1 |
| Drag & Drop | ✅ | ✅ | 🔨 Phase 1 |
| Anchors | ✅ | ✅ | ✅ Done |
| Scripting | Blueprint | C# | 🔨 Lua (Phase 4) |
| Animation | ✅ | ✅ | 🔨 Phase 4 |
| Data Binding | ✅ | ❌ | ✅ Done |
| Hot Reload | ✅ | ✅ | ✅ Done |

## Development Roadmap

### Milestone 1: Basic Editor (2-3 weeks)
- [ ] Widget Editor tab
- [ ] Canvas rendering
- [ ] Element selection
- [ ] Drag to move
- [ ] Properties panel
- [ ] Save/Load

### Milestone 2: Creation Tools (1-2 weeks)
- [ ] Widget palette
- [ ] Create new elements
- [ ] Delete elements
- [ ] Duplicate elements

### Milestone 3: Advanced Tools (2-3 weeks)
- [ ] Resize handles
- [ ] Alignment tools
- [ ] Grid snapping
- [ ] Undo/Redo

### Milestone 4: Scripting (3-4 weeks)
- [ ] Lua integration
- [ ] Event system
- [ ] Animation system
- [ ] Custom widgets

## Success Criteria

✅ Can create a complete HUD without editing JSON
✅ Can move and resize elements visually
✅ Can preview in different resolutions
✅ Can add scripts to widgets
✅ Changes save correctly to .hud files

## Next Steps

1. Create WidgetEditor tab in editor
2. Implement basic canvas rendering
3. Add element selection
4. Add drag to move
5. Add properties panel
6. Test with existing HUD files

---

**Status**: Specification Complete
**Ready for**: Phase 1 Implementation
