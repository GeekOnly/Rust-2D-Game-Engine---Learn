# ECS (Entity Component System)

High-performance Entity Component System for the Rust 2D/3D Game Engine.

## 📚 Overview

This ECS library provides entity management and component storage for our game engine. Currently using a custom HashMap-based implementation, with plans to support multiple backends (hecs, Specs, Bevy) in the future.

---

## 🎯 Current Status

**Implementation:** Custom HashMap-based ECS
**Performance:** Good for small-medium games (<5,000 entities)
**Benchmarked:** Yes ([BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md))
**Production Ready:** ✅ Yes

### Performance Baseline (10,000 entities):
- **Spawn:** 53 ns/entity
- **Query (single component):** 2.3 ns/entity
- **Query (multi-component):** 20.3 ns/entity
- **Game scenario (1,000 entities × 60 frames):** ~40 µs/frame

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) | Performance analysis and comparison with other ECS libraries |
| [MIGRATION_TO_HECS.md](MIGRATION_TO_HECS.md) | Step-by-step guide for migrating to hecs when needed |
| [examples/migration_example.rs](examples/migration_example.rs) | Code examples showing before/after migration patterns |

---

## 🚀 Quick Start

### Basic Usage

```rust
use ecs::{World, Transform, Sprite, Collider};

// Create world
let mut world = World::new();

// Spawn entity
let player = world.spawn();

// Add components
world.transforms.insert(player, Transform::default());
world.sprites.insert(player, Sprite {
    texture_id: "player.png".to_string(),
    width: 32.0,
    height: 32.0,
    color: [1.0, 1.0, 1.0, 1.0],
    billboard: false,
});

// Query entities
for (entity, transform) in &world.transforms {
    if let Some(sprite) = world.sprites.get(&entity) {
        // Render sprite at transform position
    }
}

// Remove components
world.sprites.remove(&player);

// Despawn entity
world.despawn(player);
```

### Using Prefabs

```rust
use ecs::{World, Prefab};

let mut world = World::new();

// Spawn predefined entity types
let player = Prefab::player().spawn(&mut world);
let item = Prefab::item().spawn(&mut world);
```

---

## 📊 Components

### Transform
Position, rotation, and scale in 3D space.

```rust
pub struct Transform {
    pub position: [f32; 3],  // X, Y, Z
    pub rotation: [f32; 3],  // Euler angles (degrees)
    pub scale: [f32; 3],     // X, Y, Z
}
```

### Sprite (2D Rendering)
2D sprite with texture and color.

```rust
pub struct Sprite {
    pub texture_id: String,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],    // RGBA
    pub billboard: bool,    // Face camera in 3D mode
}
```

### SpriteSheet
Sprite sheet component for managing sprite atlas data with multiple frames.

```rust
pub struct SpriteSheet {
    pub texture_path: String,
    pub texture_id: String,
    pub sheet_width: u32,
    pub sheet_height: u32,
    pub frames: Vec<SpriteFrame>,
}
```

**Loading from Sprite Editor files:**

```rust
use ecs::components::sprite_sheet::SpriteSheet;
use std::path::Path;

// Load sprite sheet from .sprite file created by the Sprite Editor
let sprite_sheet = SpriteSheet::from_sprite_file(Path::new("assets/characters/knight.sprite"))
    .expect("Failed to load sprite sheet");

// Access individual sprites by name
if let Some(frame) = sprite_sheet.get_frame_by_name("knight_idle_0") {
    println!("Frame at ({}, {}), size {}x{}", 
        frame.x, frame.y, frame.width, frame.height);
}

// Or by index
if let Some(frame) = sprite_sheet.get_frame(0) {
    // Use frame data for rendering
}
```

### Mesh (3D Rendering)
3D mesh with type and color.

```rust
pub struct Mesh {
    pub mesh_type: MeshType,  // Cube, Sphere, Cylinder, etc.
    pub color: [f32; 4],      // RGBA
}
```

### Collider
Physics collider (2D box collider).

```rust
pub struct Collider {
    pub width: f32,
    pub height: f32,
}
```

### Camera
Camera component for rendering (Unity-like).

```rust
pub struct Camera {
    pub projection: CameraProjection,  // Orthographic or Perspective
    pub fov: f32,                      // Field of view (degrees)
    pub orthographic_size: f32,        // Half-height in world units
    pub near_clip: f32,
    pub far_clip: f32,
    pub background_color: [f32; 4],
    pub depth: i32,                    // Render order
    // ...
}
```

### Script
Lua script component.

```rust
pub struct Script {
    pub script_name: String,
    pub enabled: bool,
    pub parameters: HashMap<String, ScriptParameter>,
}
```

---

## 🔧 Running Benchmarks

```bash
# Run all benchmarks
cd ecs
cargo bench

# Run specific benchmark
cargo bench spawn

# View HTML report
# Open: target/criterion/report/index.html
```

---

## 📈 When to Migrate to hecs?

Consider migrating to [hecs](https://github.com/Ralith/hecs) when:

✅ You have **>5,000 active entities**
✅ Multi-component queries are **slow** (profiling shows ECS bottleneck)
✅ You need **better memory efficiency**
✅ You want **2-10x faster queries**

See [MIGRATION_TO_HECS.md](MIGRATION_TO_HECS.md) for complete migration guide.

### Expected Performance Gains:
- Query 10k entities: **23 µs → 5-8 µs** (2-5x faster)
- Multi-component query: **203 µs → 20-40 µs** (5-10x faster)
- Insert components: **895 µs → 200-300 µs** (2-4x faster)

---

## 🏗️ Architecture

### Current Structure:
```
ecs/
├── src/
│   ├── lib.rs                 # Main World struct + components
│   ├── traits.rs              # Abstraction traits (for future backends)
│   ├── components/            # Component definitions (future)
│   └── backends/              # Backend implementations (future)
│       ├── custom.rs          # HashMap-based (current)
│       ├── hecs_backend.rs    # hecs (future)
│       ├── specs_backend.rs   # Specs (future)
│       └── bevy_backend.rs    # Bevy ECS (future)
├── benches/
│   └── ecs_benchmark.rs       # Performance benchmarks
└── examples/
    └── migration_example.rs   # Migration examples
```

---

## 🎮 Features

- [x] Entity spawning and despawning
- [x] Component add/remove/query
- [x] Entity hierarchies (parent/child)
- [x] Entity tags and layers
- [x] Prefab system
- [x] Serialization (JSON)
- [x] Performance benchmarks
- [ ] System scheduling (future)
- [ ] Multiple backend support (future)
- [ ] Change detection (future)

---

## 🔬 Performance Tips

### DO:
✅ Use `for (entity, component) in &world.components` for iteration
✅ Batch component additions/removals
✅ Use prefabs for common entity types
✅ Profile before optimizing

### DON'T:
❌ Don't iterate all entities then filter by component
❌ Don't spawn/despawn entities in hot loops
❌ Don't use `world.clone()` (expensive!)
❌ Don't add components one-by-one in inner loops

---

## 📝 Example: Physics System

```rust
pub fn physics_system(world: &mut World, dt: f32) {
    // Collect entities with velocity first (avoid borrow issues)
    let entities: Vec<_> = world.velocities.keys().copied().collect();

    for entity in entities {
        if let (Some(&(vx, vy)), Some(transform)) = (
            world.velocities.get(&entity),
            world.transforms.get_mut(&entity),
        ) {
            // Update position based on velocity
            transform.position[0] += vx * dt;
            transform.position[1] += vy * dt;

            // Optional: Apply collision detection
            if let Some(collider) = world.colliders.get(&entity) {
                // Check collisions...
            }
        }
    }
}
```

---

## 🆘 Common Issues

### Issue: "Borrow checker errors when iterating"
**Solution:** Collect entity IDs first, then iterate:
```rust
let entities: Vec<_> = world.transforms.keys().copied().collect();
for entity in entities {
    // Now you can borrow world mutably
}
```

### Issue: "Performance is slow with many entities"
**Solution:**
1. Run benchmarks to identify bottleneck
2. See [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) for analysis
3. Consider migrating to hecs (see [MIGRATION_TO_HECS.md](MIGRATION_TO_HECS.md))

### Issue: "Entity not found after despawn"
**Solution:** Entities are removed immediately. Store entity references carefully and check `world.is_alive()`.

---

## 📜 License

Part of the Rust 2D/3D Game Engine project.

---

## 🤝 Contributing

This ECS is designed for our game engine. If you find bugs or have suggestions:
1. Check [BENCHMARK_RESULTS.md](BENCHMARK_RESULTS.md) for performance baseline
2. Read [MIGRATION_TO_HECS.md](MIGRATION_TO_HECS.md) for future plans
3. Submit issues or suggestions

---

## 🔗 Related Projects

- [hecs](https://github.com/Ralith/hecs) - High-performance archetype-based ECS
- [Bevy ECS](https://github.com/bevyengine/bevy) - Bevy's powerful ECS
- [Specs](https://github.com/amethyst/specs) - Specs parallel ECS
- [Legion](https://github.com/amethyst/legion) - High-performance ECS with scheduling
