# HUD System Guide

Complete guide for using the Hybrid HUD System (World-Space + Screen-Space)

## Overview

The HUD system provides two types of UI:

1. **World-Space UI** - UI elements attached to entities in the game world
   - Health bars above enemies
   - Damage numbers
   - Interaction prompts
   - Quest markers

2. **Screen-Space HUD** - UI elements fixed to screen positions
   - Player health/mana bars
   - Minimap
   - Score display
   - Inventory menus

## World-Space UI (ECS Component)

### Adding World UI to Entities

```rust
use ecs::{World, WorldUI, WorldUIType};

// Add health bar above enemy
let enemy = world.spawn();
world.transforms.insert(enemy, Transform::with_position(100.0, 200.0, 0.0));
world.world_uis.insert(enemy, WorldUI::health_bar(80.0, 100.0));

// Add damage number
let damage_entity = world.spawn();
world.transforms.insert(damage_entity, Transform::with_position(150.0, 250.0, 0.0));
world.world_uis.insert(damage_entity, WorldUI::damage_number(25));

// Add interaction prompt
let chest = world.spawn();
world.transforms.insert(chest, Transform::with_position(200.0, 100.0, 0.0));
world.world_uis.insert(chest, WorldUI::interaction_prompt("Open Chest", "E"));
```

### Updating World UI

```rust
// Update health bar
if let Some(world_ui) = world.world_uis.get_mut(&enemy) {
    world_ui.update_health(50.0, 100.0);
}

// Update damage number (returns false when expired)
if let Some(world_ui) = world.world_uis.get_mut(&damage_entity) {
    if !world_ui.update_damage_number(delta_time) {
        world.despawn(damage_entity); // Remove when expired
    }
}
```

### World UI Types

```rust
// Health Bar
WorldUI::health_bar(current: f32, max: f32)

// Damage Number (floats upward)
WorldUI::damage_number(value: i32)

// Interaction Prompt
WorldUI::interaction_prompt(text: &str, key: &str)

// Quest Marker
WorldUI::quest_marker(QuestMarkerType::Objective)

// Custom configuration
WorldUI {
    ui_type: WorldUIType::TextLabel {
        text: "Custom Label".to_string(),
        color: [1.0, 1.0, 0.0, 1.0],
    },
    offset: [0.0, 50.0],
    billboard: true,
    scale: 1.5,
}
```

## Screen-Space HUD (Asset System)

### Loading HUD

```rust
use engine::hud::HudManager;

let mut hud_manager = HudManager::new();

// Load HUD from file
hud_manager.load("assets/ui/main_hud.hud")?;

// Or create programmatically
let mut hud = HudAsset::new("Game HUD");
hud.add_element(HudElement::new(
    "health",
    HudElementType::HealthBar {
        binding: "player.health".to_string(),
        color: [1.0, 0.2, 0.2, 1.0],
        background_color: [0.2, 0.2, 0.2, 0.8],
    },
    Anchor::TopLeft,
    [20.0, 20.0],
    [200.0, 30.0],
));
hud_manager.set_hud(hud);
```

### Data Binding

```rust
// Bind data sources
hud_manager.bind("player.health", |world| {
    if let Some(player) = find_player(world) {
        if let Some(health) = world.get_health(player) {
            return health.current / health.max;
        }
    }
    0.0
});

hud_manager.bind("player.mana", |world| {
    // Return 0.0 to 1.0 (percentage)
    0.75
});

hud_manager.bind("score", |world| {
    // Return any numeric value
    1234.0
});

hud_manager.bind("fps", |world| {
    60.0 // Get from game state
});
```

### Rendering HUD

```rust
// In game loop
fn render(&mut self) {
    // Update HUD data
    self.hud_manager.update(&self.world);
    
    // Render HUD with egui
    self.hud_manager.render_egui(
        &self.egui_ctx,
        &self.world,
        self.screen_width,
        self.screen_height,
    );
}
```

### Controlling HUD Elements

```rust
// Show/hide elements
hud_manager.set_element_visible("minimap", false);
hud_manager.set_element_visible("interaction_hint", true);

// Get current HUD
if let Some(hud) = hud_manager.get_hud() {
    println!("Current HUD: {}", hud.name);
}

// Modify HUD
if let Some(hud) = hud_manager.get_hud_mut() {
    hud.elements[0].offset = [30.0, 30.0];
}
```

## HUD Asset Format

### Example HUD File (JSON)

```json
{
  "name": "Main HUD",
  "elements": [
    {
      "id": "player_health",
      "element_type": {
        "type": "HealthBar",
        "binding": "player.health",
        "color": [1.0, 0.2, 0.2, 1.0],
        "background_color": [0.2, 0.2, 0.2, 0.8]
      },
      "anchor": "TopLeft",
      "offset": [20.0, 20.0],
      "size": [200.0, 30.0],
      "visible": true
    },
    {
      "id": "score",
      "element_type": {
        "type": "DynamicText",
        "format": "Score: {score}",
        "font_size": 24.0,
        "color": [1.0, 1.0, 1.0, 1.0]
      },
      "anchor": "TopCenter",
      "offset": [-50.0, 20.0],
      "size": [100.0, 40.0],
      "visible": true
    }
  ]
}
```

### Element Types

#### HealthBar / ProgressBar
```json
{
  "type": "HealthBar",
  "binding": "player.health",
  "color": [1.0, 0.2, 0.2, 1.0],
  "background_color": [0.2, 0.2, 0.2, 0.8]
}
```

#### Text (Static)
```json
{
  "type": "Text",
  "text": "Press E to interact",
  "font_size": 18.0,
  "color": [1.0, 1.0, 1.0, 1.0]
}
```

#### DynamicText (With Bindings)
```json
{
  "type": "DynamicText",
  "format": "Score: {score} | Level: {level}",
  "font_size": 20.0,
  "color": [1.0, 1.0, 1.0, 1.0]
}
```

#### Minimap
```json
{
  "type": "Minimap",
  "zoom": 2.0,
  "background_color": [0.1, 0.1, 0.1, 0.9]
}
```

#### Image
```json
{
  "type": "Image",
  "texture": "icon_sword.png",
  "tint": [1.0, 1.0, 1.0, 1.0]
}
```

### Anchor Points

- `TopLeft` - Top-left corner
- `TopCenter` - Top center
- `TopRight` - Top-right corner
- `CenterLeft` - Middle left
- `Center` - Screen center
- `CenterRight` - Middle right
- `BottomLeft` - Bottom-left corner
- `BottomCenter` - Bottom center
- `BottomRight` - Bottom-right corner

## Complete Example

```rust
use engine::hud::{HudManager, HudAsset, HudElement, HudElementType, Anchor};
use ecs::{World, WorldUI, EntityTag};

fn setup_hud_system(world: &mut World) -> HudManager {
    let mut hud_manager = HudManager::new();
    
    // Load HUD asset
    hud_manager.load("assets/ui/main_hud.hud")
        .expect("Failed to load HUD");
    
    // Setup data bindings
    hud_manager.bind("player.health", |world| {
        world.tags.iter()
            .find(|(_, tag)| **tag == EntityTag::Player)
            .and_then(|(entity, _)| {
                // Get health component (you'll need to implement this)
                Some(0.8) // 80% health
            })
            .unwrap_or(0.0)
    });
    
    hud_manager.bind("score", |_world| {
        // Get from game state
        1234.0
    });
    
    hud_manager.bind("fps", |_world| {
        60.0 // Calculate actual FPS
    });
    
    // Add world-space UI to enemies
    for (entity, tag) in &world.tags {
        if *tag == EntityTag::Item {
            world.world_uis.insert(*entity, WorldUI::health_bar(100.0, 100.0));
        }
    }
    
    hud_manager
}

fn update_hud(hud_manager: &mut HudManager, world: &World, dt: f32) {
    // Update HUD data bindings
    hud_manager.update(world);
    
    // Update world-space UI (damage numbers, etc.)
    let mut expired_entities = Vec::new();
    for (entity, world_ui) in &mut world.world_uis {
        if let WorldUIType::DamageNumber { .. } = world_ui.ui_type {
            if !world_ui.update_damage_number(dt) {
                expired_entities.push(*entity);
            }
        }
    }
    
    // Remove expired entities
    for entity in expired_entities {
        world.despawn(entity);
    }
}

fn render_hud(hud_manager: &HudManager, ctx: &egui::Context, world: &World, width: f32, height: f32) {
    hud_manager.render_egui(ctx, world, width, height);
}
```

## Best Practices

### When to Use World-Space UI
- ✅ Health bars above enemies
- ✅ Damage/healing numbers
- ✅ Interaction prompts near objects
- ✅ Quest markers in world
- ✅ Nameplates

### When to Use Screen-Space HUD
- ✅ Player status (health, mana, stamina)
- ✅ Minimap
- ✅ Score/currency display
- ✅ Inventory UI
- ✅ Menu systems
- ✅ Dialog boxes

### Performance Tips

1. **Batch World UI** - Group similar UI elements for efficient rendering
2. **Cull Off-Screen** - Don't render world UI that's outside camera view
3. **Update Frequency** - Update HUD data only when changed, not every frame
4. **Reuse Entities** - Pool damage number entities instead of spawning/despawning

### Future Enhancements

- [ ] Custom shader rendering for fancy effects
- [ ] Animation system for HUD elements
- [ ] Layout containers (horizontal/vertical)
- [ ] HUD editor in engine
- [ ] Image/icon support
- [ ] Rich text formatting
- [ ] Localization support

## Troubleshooting

**HUD not showing?**
- Check if HUD file path is correct
- Verify data bindings are set up
- Ensure elements are visible (visible: true)
- Check screen dimensions are correct

**World UI not appearing?**
- Verify entity has Transform component
- Check if entity is in camera view
- Ensure WorldUI component is added
- Check offset values

**Performance issues?**
- Reduce number of world UI elements
- Implement culling for off-screen UI
- Use object pooling for damage numbers
- Optimize data binding functions
