# HUD System - Quick Start Example

## Minimal Working Example

```rust
use engine::hud::{HudManager, HudAsset, HudElement, HudElementType, Anchor};
use ecs::{World, WorldUI, EntityTag, Transform};

fn main() {
    let mut world = World::new();
    let mut hud_manager = HudManager::new();
    
    // === 1. Setup Screen-Space HUD ===
    setup_screen_hud(&mut hud_manager);
    
    // === 2. Setup World-Space UI ===
    setup_world_ui(&mut world);
    
    // === 3. Game Loop ===
    // In your render loop:
    // hud_manager.update(&world);
    // hud_manager.render_egui(&egui_ctx, &world, screen_width, screen_height);
}

fn setup_screen_hud(hud_manager: &mut HudManager) {
    // Create HUD programmatically
    let mut hud = HudAsset::new("Game HUD");
    
    // Player health bar (top-left)
    hud.add_element(HudElement::new(
        "player_health",
        HudElementType::HealthBar {
            binding: "player.health".to_string(),
            color: [1.0, 0.2, 0.2, 1.0],
            background_color: [0.2, 0.2, 0.2, 0.8],
        },
        Anchor::TopLeft,
        [20.0, 20.0],
        [200.0, 30.0],
    ));
    
    // Score display (top-center)
    hud.add_element(HudElement::new(
        "score",
        HudElementType::DynamicText {
            format: "Score: {score}".to_string(),
            font_size: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
        Anchor::TopCenter,
        [-50.0, 20.0],
        [100.0, 40.0],
    ));
    
    hud_manager.set_hud(hud);
    
    // Setup data bindings
    hud_manager.bind("player.health", |world| {
        // Find player and return health percentage (0.0 to 1.0)
        world.tags.iter()
            .find(|(_, tag)| **tag == EntityTag::Player)
            .map(|_| 0.8) // 80% health
            .unwrap_or(0.0)
    });
    
    hud_manager.bind("score", |_world| {
        1234.0 // Get from game state
    });
}

fn setup_world_ui(world: &mut World) {
    // Spawn enemy with health bar
    let enemy = world.spawn();
    world.transforms.insert(enemy, Transform::with_position(200.0, 150.0, 0.0));
    world.world_uis.insert(enemy, WorldUI::health_bar(75.0, 100.0));
    
    // Spawn chest with interaction prompt
    let chest = world.spawn();
    world.transforms.insert(chest, Transform::with_position(300.0, 200.0, 0.0));
    world.world_uis.insert(chest, WorldUI::interaction_prompt("Open", "E"));
}
```

## Loading HUD from File

```rust
// Load pre-made HUD
hud_manager.load("assets/ui/main_hud.hud")
    .expect("Failed to load HUD");

// Setup bindings
hud_manager.bind("player.health", |world| {
    get_player_health_percentage(world)
});

hud_manager.bind("player.mana", |world| {
    get_player_mana_percentage(world)
});

hud_manager.bind("score", |world| {
    get_game_score(world) as f32
});

hud_manager.bind("fps", |_world| {
    60.0 // Calculate actual FPS
});
```

## Spawning Damage Numbers

```rust
fn spawn_damage_number(world: &mut World, position: [f32; 3], damage: i32) {
    let entity = world.spawn();
    world.transforms.insert(entity, Transform {
        position,
        rotation: [0.0, 0.0, 0.0],
        scale: [1.0, 1.0, 1.0],
    });
    world.world_uis.insert(entity, WorldUI::damage_number(damage));
}

// Usage
spawn_damage_number(&mut world, [player_x, player_y, 0.0], 25);
```

## Updating World UI

```rust
fn update_world_ui(world: &mut World, dt: f32) {
    let mut expired = Vec::new();
    
    for (entity, world_ui) in &mut world.world_uis {
        match &mut world_ui.ui_type {
            WorldUIType::DamageNumber { .. } => {
                if !world_ui.update_damage_number(dt) {
                    expired.push(*entity);
                }
            }
            WorldUIType::HealthBar { current, max } => {
                // Update health from entity component
                // *current = get_entity_health(world, *entity);
            }
            _ => {}
        }
    }
    
    // Remove expired entities
    for entity in expired {
        world.despawn(entity);
    }
}
```

## Controlling HUD Visibility

```rust
// Show/hide elements dynamically
hud_manager.set_element_visible("minimap", false);
hud_manager.set_element_visible("interaction_hint", true);

// Toggle based on game state
if player_near_chest {
    hud_manager.set_element_visible("interaction_hint", true);
} else {
    hud_manager.set_element_visible("interaction_hint", false);
}
```

## Complete Game Loop Integration

```rust
struct GameState {
    world: World,
    hud_manager: HudManager,
    // ... other state
}

impl GameState {
    fn new() -> Self {
        let mut world = World::new();
        let mut hud_manager = HudManager::new();
        
        // Setup HUD
        hud_manager.load("assets/ui/main_hud.hud").unwrap();
        setup_hud_bindings(&mut hud_manager);
        
        Self {
            world,
            hud_manager,
        }
    }
    
    fn update(&mut self, dt: f32) {
        // Update game logic
        update_game_logic(&mut self.world, dt);
        
        // Update world UI (damage numbers, etc.)
        update_world_ui(&mut self.world, dt);
        
        // Update HUD data
        self.hud_manager.update(&self.world);
    }
    
    fn render(&self, egui_ctx: &egui::Context, screen_width: f32, screen_height: f32) {
        // Render game world
        // ...
        
        // Render world-space UI
        render_world_ui(&self.world);
        
        // Render screen-space HUD
        self.hud_manager.render_egui(
            egui_ctx,
            &self.world,
            screen_width,
            screen_height,
        );
    }
}

fn setup_hud_bindings(hud_manager: &mut HudManager) {
    hud_manager.bind("player.health", |world| {
        world.tags.iter()
            .find(|(_, tag)| **tag == EntityTag::Player)
            .and_then(|(entity, _)| {
                // Get health component
                Some(0.8) // Return 0.0 to 1.0
            })
            .unwrap_or(0.0)
    });
    
    hud_manager.bind("score", |_world| {
        // Get from game state
        0.0
    });
}
```

## Testing HUD

```rust
#[test]
fn test_hud_system() {
    let mut world = World::new();
    let mut hud_manager = HudManager::new();
    
    // Create simple HUD
    let mut hud = HudAsset::new("Test HUD");
    hud.add_element(HudElement::new(
        "test_health",
        HudElementType::HealthBar {
            binding: "health".to_string(),
            color: [1.0, 0.0, 0.0, 1.0],
            background_color: [0.2, 0.2, 0.2, 0.8],
        },
        Anchor::TopLeft,
        [10.0, 10.0],
        [100.0, 20.0],
    ));
    
    hud_manager.set_hud(hud);
    hud_manager.bind("health", |_| 0.75);
    
    // Update and verify
    hud_manager.update(&world);
    assert_eq!(hud_manager.get_value("health"), Some(0.75));
}
```

## Next Steps

1. **Add Health Component** - Create a proper health component in ECS
2. **Implement Custom Shaders** - Add fancy effects to HUD elements
3. **Create HUD Editor** - Visual editor for designing HUDs
4. **Add Animations** - Animate HUD elements (fade in/out, slide, etc.)
5. **Localization** - Support multiple languages

See `MD/HUD_SYSTEM_GUIDE.md` for complete documentation.
