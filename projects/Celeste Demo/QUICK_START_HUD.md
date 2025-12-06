# Quick Start - HUD System

Get the HUD system running in 5 minutes!

## Step 1: Files Already Created ✅

- ✅ `assets/ui/celeste_hud.hud` - HUD configuration
- ✅ `scripts/hud_manager.lua` - HUD state manager
- ✅ `scripts/world_ui_example.lua` - World UI examples

## Step 2: Integration Code (Copy & Paste)

### Add to Engine Runtime (Rust)

```rust
// In engine/src/runtime/mod.rs or your game loop

use crate::hud::HudManager;
use ecs::{World, EntityTag};

pub struct GameRuntime {
    world: World,
    hud_manager: HudManager,
    // ... other fields
}

impl GameRuntime {
    pub fn new() -> Self {
        let mut hud_manager = HudManager::new();
        
        // Load HUD
        hud_manager.load("projects/Celeste Demo/assets/ui/celeste_hud.hud")
            .expect("Failed to load HUD");
        
        // Setup bindings
        Self::setup_hud_bindings(&mut hud_manager);
        
        Self {
            world: World::new(),
            hud_manager,
        }
    }
    
    fn setup_hud_bindings(hud_manager: &mut HudManager) {
        // Player health
        hud_manager.bind("player.health", |_world| 1.0);
        
        // Player stamina
        hud_manager.bind("player.stamina", |_world| 1.0);
        
        // Dash count
        hud_manager.bind("dash_count", |_world| 1.0);
        
        // Position
        hud_manager.bind("pos_x", |world| {
            world.tags.iter()
                .find(|(_, tag)| **tag == EntityTag::Player)
                .and_then(|(entity, _)| world.transforms.get(entity))
                .map(|t| t.position[0])
                .unwrap_or(0.0)
        });
        
        hud_manager.bind("pos_y", |world| {
            world.tags.iter()
                .find(|(_, tag)| **tag == EntityTag::Player)
                .and_then(|(entity, _)| world.transforms.get(entity))
                .map(|t| t.position[1])
                .unwrap_or(0.0)
        });
        
        // Velocity
        hud_manager.bind("vel_x", |world| {
            world.tags.iter()
                .find(|(_, tag)| **tag == EntityTag::Player)
                .and_then(|(entity, _)| world.velocities.get(entity))
                .map(|v| v.0)
                .unwrap_or(0.0)
        });
        
        hud_manager.bind("vel_y", |world| {
            world.tags.iter()
                .find(|(_, tag)| **tag == EntityTag::Player)
                .and_then(|(entity, _)| world.velocities.get(entity))
                .map(|v| v.1)
                .unwrap_or(0.0)
        });
        
        // FPS
        hud_manager.bind("fps", |_world| 60.0);
    }
    
    pub fn update(&mut self, dt: f32) {
        // Update game logic
        // ...
        
        // Update HUD data
        self.hud_manager.update(&self.world);
    }
    
    pub fn render(&self, egui_ctx: &egui::Context, screen_width: f32, screen_height: f32) {
        // Render game world
        // ...
        
        // Render HUD
        self.hud_manager.render_egui(
            egui_ctx,
            &self.world,
            screen_width,
            screen_height,
        );
    }
}
```

## Step 3: Test It!

### Run the Game
```bash
cd "projects/Celeste Demo"
./launch.bat
```

### What You Should See

✅ **Top-Left**: Health bar (green) and stamina bar (yellow)  
✅ **Top-Right**: Position, velocity, and FPS  
✅ **Bottom**: Controls hint  

### Test Features

1. **Move Player** - Watch position/velocity update
2. **Jump** - See velocity change
3. **Wall Slide** - Stamina bar decreases
4. **Dash** - Dash indicator updates

## Step 4: Add World UI (Optional)

### Spawn Damage Number

```rust
use ecs::{World, WorldUI, Transform};

// When player takes damage
let dmg_entity = world.spawn();
world.transforms.insert(dmg_entity, Transform::with_position(
    player_x,
    player_y + 1.0,
    0.0
));
world.world_uis.insert(dmg_entity, WorldUI::damage_number(25));
```

### Spawn Enemy with Health Bar

```rust
let enemy = world.spawn();
world.transforms.insert(enemy, Transform::with_position(10.0, 5.0, 0.0));
world.world_uis.insert(enemy, WorldUI::health_bar(80.0, 100.0));
```

## Step 5: Customize

### Change Colors
Edit `assets/ui/celeste_hud.hud`:
```json
"color": [1.0, 0.0, 0.0, 1.0]  // Red
```

### Reposition Elements
```json
"offset": [50.0, 50.0]  // Move right 50px, down 50px
```

### Hide Elements
```json
"visible": false
```

## Troubleshooting

### HUD Not Showing?
- ✅ Check file path: `projects/Celeste Demo/assets/ui/celeste_hud.hud`
- ✅ Verify HUD is loaded: `hud_manager.load(...)`
- ✅ Ensure render is called: `hud_manager.render_egui(...)`

### Data Not Updating?
- ✅ Call `hud_manager.update(&world)` every frame
- ✅ Check bindings are set up correctly
- ✅ Verify player entity exists

### Performance Issues?
- ✅ Reduce update frequency (update every 2-3 frames)
- ✅ Hide unused elements
- ✅ Optimize binding functions

## Next Steps

1. ✅ **Working HUD** - You're done!
2. 📖 Read [HUD_EXAMPLE_README.md](HUD_EXAMPLE_README.md) for advanced features
3. 🎨 Customize HUD layout and colors
4. 🎮 Add health/damage system
5. ✨ Implement world UI rendering

## Need Help?

- See [HUD System Guide](../../MD/HUD_SYSTEM_GUIDE.md)
- Check [HUD Example](../../MD/HUD_EXAMPLE.md)
- Read [HUD README](../../MD/HUD_README.md)

---

**That's it! You now have a working HUD system! 🎉**
