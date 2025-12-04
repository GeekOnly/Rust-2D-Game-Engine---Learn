# Tilemap System

ระบบ Tilemap สำหรับ Game Engine รองรับทั้ง **LDtk** และ **Tiled** พร้อม **Hot-Reload**

## 🎯 Features

### ✅ LDtk Support
- ✅ Load .ldtk files
- ✅ Multiple levels
- ✅ Multiple layers
- ✅ **Hot-reload แบบ real-time**
- ✅ Tile layers
- ✅ Entity layers

### ✅ Tiled Support
- ✅ Load .tmx files
- ✅ Multiple layers
- ✅ Tile layers
- 🚧 Hot-reload (coming soon)

## 🚀 Quick Start

### LDtk with Hot-Reload

```rust
use ecs::{World, loaders::LdtkHotReloader};

let mut world = World::new();
let mut reloader = LdtkHotReloader::new();

// Load and watch
reloader.watch("levels/world.ldtk", &mut world)?;

// Game loop
loop {
    // Check for updates
    if let Some(entities) = reloader.check_updates(&mut world) {
        println!("Reloaded {} entities!", entities.len());
    }
}
```

### Runtime API (High-level)

```rust
use engine::runtime::LdtkRuntime;

let mut ldtk = LdtkRuntime::new();
ldtk.load("levels/world.ldtk", &mut world)?;

// In game loop
if ldtk.update(&mut world) {
    println!("Level reloaded!");
}
```

## 📖 Documentation

- **[LDTK_HOT_RELOAD.md](LDTK_HOT_RELOAD.md)** - API Reference
- **[../MD/LDTK_INTEGRATION_GUIDE.md](../MD/LDTK_INTEGRATION_GUIDE.md)** - Integration Guide
- **[SPRITE_TILEMAP_USAGE.md](SPRITE_TILEMAP_USAGE.md)** - Sprite & Tilemap Usage

## 🎮 Examples

### Basic Example

```bash
cargo run --example ldtk_hot_reload -- levels/world.ldtk
```

### Game Example

```rust
struct Game {
    world: World,
    ldtk: LdtkRuntime,
}

impl Game {
    fn new() -> Self {
        let mut world = World::new();
        let mut ldtk = LdtkRuntime::new();
        ldtk.load("levels/level_1.ldtk", &mut world).unwrap();
        
        Self { world, ldtk }
    }
    
    fn update(&mut self) {
        // Hot-reload check
        if self.ldtk.update(&mut self.world) {
            self.on_level_reloaded();
        }
        
        // Game logic
        self.update_physics();
        self.update_player();
    }
    
    fn on_level_reloaded(&mut self) {
        println!("Level reloaded!");
        // Reset player position, etc.
    }
}
```

## 🔥 Hot-Reload Workflow

1. **เปิด LDtk Editor** - แก้ไข level
2. **เปิด Game** - run ด้วย hot-reload
3. **Save ใน LDtk** - กด Ctrl+S
4. **ดูผลทันที** - game reload อัตโนมัติ

## 🛠️ Architecture

```
ecs/
├── src/
│   ├── loaders/
│   │   ├── ldtk_loader.rs        # Basic LDtk loader
│   │   ├── ldtk_hot_reload.rs    # Hot-reload system
│   │   └── tiled_loader.rs       # Tiled loader
│   └── components/
│       └── tilemap.rs             # Tilemap components
└── examples/
    └── ldtk_hot_reload.rs         # Example usage

engine/
└── src/
    └── runtime/
        └── ldtk_runtime.rs        # High-level API
```

## 📦 Dependencies

```toml
[dependencies]
ldtk_rust = "0.6"      # LDtk file format
tiled = "0.11"         # Tiled file format
notify = "6.1"         # File watching
```

## 🎯 Use Cases

### 1. Platformer (Celeste-style)
- ใช้ LDtk ออกแบบ level
- Hot-reload เพื่อ iterate เร็ว
- Test gameplay ทันที

### 2. Top-Down RPG
- ใช้ LDtk สร้าง world map
- Hot-reload เพื่อปรับ layout
- Test collision และ navigation

### 3. Puzzle Game
- ใช้ LDtk ออกแบบ puzzle
- Hot-reload เพื่อ test difficulty
- Iterate ได้เร็ว

## 🐛 Troubleshooting

### ไฟล์ไม่ reload
```bash
# Enable logging
RUST_LOG=info cargo run
```

### Performance issues
- แบ่ง level เป็นหลายไฟล์
- ใช้ level streaming
- ปิด hot-reload ใน production

## 📚 Resources

- **LDtk**: https://ldtk.io/
- **Tiled**: https://www.mapeditor.org/
- **notify crate**: https://docs.rs/notify/

## 🚀 Roadmap

- [x] LDtk basic loading
- [x] LDtk hot-reload
- [x] Multiple levels support
- [ ] Tiled hot-reload
- [ ] Partial reload (layers only)
- [ ] Entity properties
- [ ] Auto-tiling support
- [ ] Level streaming
- [ ] Editor integration

## 💡 Tips

1. **ใช้ relative paths** - ง่ายต่อการ share
2. **Organize levels** - แยก folder ตาม world
3. **Version control** - commit .ldtk files
4. **Dual monitor** - LDtk + Game side-by-side
5. **Auto-save** - เปิดใน LDtk settings

---

Happy Level Designing! 🎮✨
