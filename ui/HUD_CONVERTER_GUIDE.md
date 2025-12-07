# HUD to UIPrefab Converter Guide

This guide explains how to use the `HudToUIPrefabConverter` to migrate legacy HUD assets to the new UI prefab system.

## Overview

The `HudToUIPrefabConverter` provides a seamless migration path from the legacy HUD system to the new UI prefab system. It automatically converts:

- **Anchor positions** → RectTransform anchor points
- **HUD elements** → UI components (Image, Text, Panel, etc.)
- **Hierarchies** → Parent-child relationships
- **Properties** → Component configurations

## Basic Usage

```rust
use ui::{HudToUIPrefabConverter, HudAsset, HudElement, HudElementType, Anchor};

// Load or create a HUD asset
let hud = HudAsset {
    name: "PlayerHUD".to_string(),
    elements: vec![
        HudElement {
            id: "HealthLabel".to_string(),
            element_type: HudElementType::Text {
                text: "Health:".to_string(),
                font_size: 18.0,
                color: [1.0, 1.0, 1.0, 1.0],
            },
            anchor: Anchor::TopLeft,
            offset: [10.0, 10.0],
            size: [100.0, 30.0],
            visible: true,
        },
    ],
};

// Convert to UIPrefab
let prefab = HudToUIPrefabConverter::convert(&hud);

// Save to file
let json = serde_json::to_string_pretty(&prefab).unwrap();
std::fs::write("player_hud.uiprefab", json).unwrap();
```

## Anchor Conversion

The converter maps all 9 HUD anchor positions to RectTransform anchors:

| HUD Anchor | RectTransform Anchor | Pivot |
|------------|---------------------|-------|
| TopLeft | (0.0, 1.0) | (0.0, 1.0) |
| TopCenter | (0.5, 1.0) | (0.5, 1.0) |
| TopRight | (1.0, 1.0) | (1.0, 1.0) |
| CenterLeft | (0.0, 0.5) | (0.0, 0.5) |
| Center | (0.5, 0.5) | (0.5, 0.5) |
| CenterRight | (1.0, 0.5) | (1.0, 0.5) |
| BottomLeft | (0.0, 0.0) | (0.0, 0.0) |
| BottomCenter | (0.5, 0.0) | (0.5, 0.0) |
| BottomRight | (1.0, 0.0) | (1.0, 0.0) |

### Example

```rust
// HUD element anchored to top-left
HudElement {
    anchor: Anchor::TopLeft,
    offset: [20.0, 20.0],
    size: [100.0, 50.0],
    // ...
}

// Converts to RectTransform:
RectTransform {
    anchor_min: Vec2::new(0.0, 1.0),
    anchor_max: Vec2::new(0.0, 1.0),
    pivot: Vec2::new(0.0, 1.0),
    anchored_position: Vec2::new(20.0, 20.0),
    size_delta: Vec2::new(100.0, 50.0),
    // ...
}
```

## Component Mapping

### Text → UIText

Simple text elements convert directly:

```rust
HudElementType::Text {
    text: "Score: 100".to_string(),
    font_size: 20.0,
    color: [1.0, 1.0, 0.0, 1.0],
}

// Converts to:
UIText {
    text: "Score: 100".to_string(),
    font: "default".to_string(),
    font_size: 20.0,
    color: [1.0, 1.0, 0.0, 1.0],
    alignment: TextAlignment::MiddleCenter,
    // ... other defaults
}
```

### DynamicText → UIText (with notes)

Dynamic text includes conversion notes for Lua binding:

```rust
HudElementType::DynamicText {
    format: "Score: {score}".to_string(),
    font_size: 20.0,
    color: [1.0, 1.0, 0.0, 1.0],
}

// Converts to UIText with name annotation:
// Name: "ScoreLabel /* DynamicText: Bind 'Score: {score}' in Lua using set_text() */"
```

**Migration Note:** Update your Lua scripts to dynamically set the text:

```lua
-- Old HUD system (automatic binding)
-- No code needed, binding was automatic

-- New UI system (manual binding)
local score_label = ui.find_element("ScoreLabel")
ui.set_text(score_label, "Score: " .. player.score)
```

### Image → UIImage

Images convert with sprite reference:

```rust
HudElementType::Image {
    texture: "icon.png".to_string(),
    tint: [1.0, 0.5, 0.5, 1.0],
}

// Converts to:
UIImage {
    sprite: Some("icon.png".to_string()),
    image_type: ImageType::Simple,
    // ... other defaults
}
// Note: Tint is stored in UIElement.color
```

### HealthBar / ProgressBar → UIPanel (with notes)

Health bars and progress bars require manual setup of fill images:

```rust
HudElementType::HealthBar {
    binding: "player.health".to_string(),
    color: [1.0, 0.0, 0.0, 1.0],
    background_color: [0.2, 0.2, 0.2, 0.8],
}

// Converts to UIPanel with annotation:
// Name: "HealthBar /* HealthBar: Add UIImage child with fill_method=Horizontal,
//        bind 'player.health' to fill_amount in Lua. Colors: fg=[1.0, 0.0, 0.0, 1.0],
//        bg=[0.2, 0.2, 0.2, 0.8] */"
```

**Migration Steps:**

1. Create background UIImage child:
   ```rust
   UIImage {
       sprite: Some("bar_background.png".to_string()),
       image_type: ImageType::Sliced,
       // Use background_color from notes
   }
   ```

2. Create fill UIImage child:
   ```rust
   UIImage {
       sprite: Some("bar_fill.png".to_string()),
       image_type: ImageType::Filled,
       fill_method: FillMethod::Horizontal,
       fill_amount: 1.0, // Will be updated by Lua
       // Use color from notes
   }
   ```

3. Update Lua script:
   ```lua
   local health_bar_fill = ui.find_element("HealthBar_Fill")
   ui.set_fill_amount(health_bar_fill, player.health / player.max_health)
   ```

### Container → UIPanel (with children)

Containers preserve hierarchy:

```rust
HudElementType::Container {
    children: vec![
        // Child elements...
    ],
}

// Converts to UIPanel with children in prefab hierarchy
```

### Minimap → UIPanel (with notes)

Minimaps require custom implementation:

```rust
HudElementType::Minimap {
    zoom: 2.0,
    background_color: [0.1, 0.1, 0.1, 0.9],
}

// Converts to UIPanel with annotation:
// Name: "Minimap /* Minimap: Custom component needed. Zoom=2.0,
//        BG=[0.1, 0.1, 0.1, 0.9]. Implement in Lua or as custom UI component. */"
```

**Migration Note:** Implement minimap rendering in Lua or create a custom UI component.

## Complete Example

Here's a complete example converting a game HUD:

```rust
use ui::{HudToUIPrefabConverter, HudAsset, HudElement, HudElementType, Anchor};

fn main() {
    // Create a complex HUD
    let hud = HudAsset {
        name: "GameHUD".to_string(),
        elements: vec![
            // Health bar
            HudElement {
                id: "HealthBar".to_string(),
                element_type: HudElementType::HealthBar {
                    binding: "player.health".to_string(),
                    color: [1.0, 0.2, 0.2, 1.0],
                    background_color: [0.2, 0.2, 0.2, 0.8],
                },
                anchor: Anchor::TopLeft,
                offset: [10.0, 10.0],
                size: [200.0, 20.0],
                visible: true,
            },
            
            // Score display
            HudElement {
                id: "ScoreLabel".to_string(),
                element_type: HudElementType::DynamicText {
                    format: "Score: {score}".to_string(),
                    font_size: 18.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                anchor: Anchor::TopCenter,
                offset: [0.0, 10.0],
                size: [150.0, 30.0],
                visible: true,
            },
            
            // Minimap
            HudElement {
                id: "Minimap".to_string(),
                element_type: HudElementType::Minimap {
                    zoom: 2.0,
                    background_color: [0.1, 0.1, 0.1, 0.9],
                },
                anchor: Anchor::BottomRight,
                offset: [-150.0, 10.0],
                size: [140.0, 140.0],
                visible: true,
            },
            
            // Inventory panel
            HudElement {
                id: "InventoryPanel".to_string(),
                element_type: HudElementType::Container {
                    children: vec![
                        HudElement {
                            id: "InventoryTitle".to_string(),
                            element_type: HudElementType::Text {
                                text: "Inventory".to_string(),
                                font_size: 16.0,
                                color: [1.0, 1.0, 1.0, 1.0],
                            },
                            anchor: Anchor::TopCenter,
                            offset: [0.0, 5.0],
                            size: [100.0, 20.0],
                            visible: true,
                        },
                    ],
                },
                anchor: Anchor::BottomLeft,
                offset: [10.0, 10.0],
                size: [200.0, 300.0],
                visible: true,
            },
        ],
    };
    
    // Convert to UIPrefab
    let prefab = HudToUIPrefabConverter::convert(&hud);
    
    // Save to file
    let json = serde_json::to_string_pretty(&prefab).unwrap();
    std::fs::write("game_hud.uiprefab", json).unwrap();
    
    println!("Converted HUD to UIPrefab!");
    println!("Root element: {}", prefab.root.name);
    println!("Child elements: {}", prefab.root.children.len());
    
    // Print conversion notes
    for child in &prefab.root.children {
        if child.name.contains("/*") {
            println!("\nConversion note for {}:", child.id);
            println!("{}", child.name);
        }
    }
}
```

## Batch Conversion

For converting multiple HUD files, see the migration script in task 22.

## Conversion Notes

The converter adds notes to element names when manual intervention is needed:

- **DynamicText**: Indicates Lua binding is required
- **HealthBar/ProgressBar**: Indicates fill image setup is needed
- **Minimap**: Indicates custom component implementation is needed
- **Image tint**: Indicates non-white tint values

These notes are embedded in the element name as comments: `"ElementName /* Note */"`

## Testing Converted Prefabs

After conversion, test your prefabs:

```rust
use ui::{PrefabInstantiator, UIPrefab};

// Load converted prefab
let json = std::fs::read_to_string("game_hud.uiprefab").unwrap();
let prefab: UIPrefab = serde_json::from_str(&json).unwrap();

// Instantiate
let mut instantiator = PrefabInstantiator::new();
let result = instantiator.instantiate(&prefab);

println!("Instantiated prefab with root entity: {}", result.root_entity);
println!("Named entities: {:?}", result.named_entities);
```

## Migration Checklist

- [ ] Convert HUD asset using `HudToUIPrefabConverter::convert()`
- [ ] Review conversion notes in element names
- [ ] Set up fill images for health/progress bars
- [ ] Update Lua scripts for dynamic text bindings
- [ ] Implement custom components for minimaps
- [ ] Test converted prefab in-game
- [ ] Update documentation and examples
- [ ] Remove old HUD files after verification

## Next Steps

After converting your HUD assets:

1. **Task 22**: Use the batch conversion script to convert all HUD files
2. **Task 23**: Use the UI Prefab Editor to refine converted prefabs
3. **Task 24**: Update engine integration to use new UI system
4. **Task 25**: Update documentation and examples

## Troubleshooting

### Issue: Converted anchors don't match original layout

**Solution**: The HUD system used screen-space coordinates, while the UI system uses normalized anchors. Verify that:
- Anchor positions are correct (0-1 range)
- Offsets are relative to anchor points
- Pivot points match the original anchor behavior

### Issue: Dynamic text not updating

**Solution**: Update your Lua scripts to manually set text values. The new system doesn't have automatic binding.

### Issue: Health bars missing fill graphics

**Solution**: Manually add UIImage children with `fill_method=Horizontal` and bind `fill_amount` in Lua.

### Issue: Minimap not rendering

**Solution**: Implement custom minimap rendering logic in Lua or create a custom UI component.

## API Reference

### `HudToUIPrefabConverter`

```rust
pub struct HudToUIPrefabConverter;

impl HudToUIPrefabConverter {
    /// Convert a HudAsset to a UIPrefab
    pub fn convert(hud: &HudAsset) -> UIPrefab;
    
    /// Convert a single HudElement to a UIPrefabElement
    fn convert_element(hud_element: &HudElement) -> UIPrefabElement;
    
    /// Convert HUD Anchor to RectTransform
    fn convert_anchor(anchor: &Anchor, offset: [f32; 2], size: [f32; 2]) -> RectTransform;
    
    /// Convert HudElementType to UI components
    fn convert_element_type(element_type: &HudElementType) 
        -> (Option<UIImage>, Option<UIText>, Option<UIPanel>, Vec<HudElement>, String);
}
```

## See Also

- [UI Prefab System](PREFAB_INSTANTIATION.md)
- [UI System Guide](README.md)
- [Lua API Reference](LUA_API.md)
- [Migration Plan](../.kiro/specs/in-game-ui-system/MIGRATION_PLAN.md)
