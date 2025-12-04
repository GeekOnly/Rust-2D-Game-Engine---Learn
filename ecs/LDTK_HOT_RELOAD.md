# LDtk Hot-Reload System

ระบบ hot-reload สำหรับไฟล์ LDtk ที่ช่วยให้คุณแก้ไข level ใน LDtk editor และเห็นผลลัพธ์ทันทีใน game engine โดยไม่ต้อง restart

## ✨ Features

- 🔥 **Hot-reload แบบ real-time** - แก้ไขไฟล์ .ldtk และเห็นผลทันที
- 🎮 **ใช้งานง่าย** - API แค่ 3-4 บรรทัด
- 🔍 **Watch หลายไฟล์** - รองรับการ watch หลาย .ldtk พร้อมกัน
- 🛡️ **Error handling** - จัดการ error อย่างปลอดภัย ไม่ crash game
- 🎯 **Production-ready** - ปิด hot-reload ได้ใน production build

## 📦 Installation

Hot-reload system มาพร้อมกับ `ecs` crate แล้ว ไม่ต้องติดตั้งเพิ่ม

## 🚀 Quick Start

### 1. Basic Usage (ECS Level)

```rust
use ecs::{World, LdtkHotReloader};

fn main() {
    let mut world = World::new();
    let mut reloader = LdtkHotReloader::new();
    
    // Load และ watch ไฟล์
    reloader.watch("levels/world.ldtk", &mut world).unwrap();
    
    // ใน game loop
    loop {
        // Check for updates
        if let Some(entities) = reloader.check_updates(&mut world) {
            println!("Reloaded {} entities!", entities.len());
        }
        
        // ... game logic ...
    }
}
```

### 2. Runtime Usage (Engine Level)

```rust
use engine::runtime::LdtkRuntime;
use ecs::World;

fn main() {
    let mut world = World::new();
    let mut ldtk = LdtkRuntime::new();
    
    // Load level
    ldtk.load("levels/world.ldtk", &mut world).unwrap();
    
    // Game loop
    loop {
        // Update hot-reload (returns true if reloaded)
        if ldtk.update(&mut world) {
            println!("Level reloaded!");
        }
        
        // ... game logic ...
    }
}
```

## 📖 API Reference

### `LdtkHotReloader`

Low-level API สำหรับ hot-reload

#### Methods

```rust
// สร้าง reloader ใหม่
pub fn new() -> Self

// Watch ไฟล์และ load เข้า world
pub fn watch(&mut self, path: impl AsRef<Path>, world: &mut World) 
    -> Result<Vec<Entity>, String>

// หยุด watch ไฟล์
pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), String>

// Check updates (เรียกทุก frame)
pub fn check_updates(&mut self, world: &mut World) -> Option<Vec<Entity>>

// ดูรายการไฟล์ที่กำลัง watch
pub fn watched_files(&self) -> Vec<PathBuf>

// ดู entities ของไฟล์เฉพาะ
pub fn get_entities(&self, path: impl AsRef<Path>) -> Option<&[Entity]>
```

### `LdtkRuntime`

High-level API สำหรับใช้ใน game

#### Methods

```rust
// สร้าง runtime ใหม่
pub fn new() -> Self

// Load และ watch ไฟล์
pub fn load(&mut self, path: impl AsRef<Path>, world: &mut World) 
    -> Result<(), String>

// Update (เรียกทุก frame)
pub fn update(&mut self, world: &mut World) -> bool

// เปิด/ปิด hot-reload
pub fn set_enabled(&mut self, enabled: bool)

// Check ว่า hot-reload เปิดอยู่หรือไม่
pub fn is_enabled(&self) -> bool

// ดูรายการไฟล์ที่กำลัง watch
pub fn watched_files(&self) -> Vec<PathBuf>

// หยุด watch ไฟล์
pub fn unload(&mut self, path: impl AsRef<Path>) -> Result<(), String>
```

## 🎯 Use Cases

### 1. Level Design Workflow

```rust
// Designer แก้ไข level ใน LDtk
// Game reload อัตโนมัติ
// ทดสอบได้ทันที ไม่ต้อง restart

let mut ldtk = LdtkRuntime::new();
ldtk.load("levels/level_1.ldtk", &mut world)?;

loop {
    if ldtk.update(&mut world) {
        // Reset player position หรือ game state
        reset_game_state(&mut world);
    }
    
    update_game(&mut world);
    render(&world);
}
```

### 2. Multiple Levels

```rust
let mut reloader = LdtkHotReloader::new();

// Watch หลาย level พร้อมกัน
reloader.watch("levels/world_1.ldtk", &mut world)?;
reloader.watch("levels/world_2.ldtk", &mut world)?;
reloader.watch("levels/ui_layout.ldtk", &mut world)?;

// ทุกไฟล์จะ reload อัตโนมัติเมื่อมีการแก้ไข
```

### 3. Production Build

```rust
let mut ldtk = LdtkRuntime::new();

// ปิด hot-reload ใน production
#[cfg(not(debug_assertions))]
ldtk.set_enabled(false);

ldtk.load("levels/world.ldtk", &mut world)?;
```

## 🔧 Integration with LDtk Editor

### Workflow แนะนำ:

1. **เปิด LDtk Editor** - แก้ไข level ของคุณ
2. **เปิด Game Engine** - run game ด้วย hot-reload
3. **แก้ไขและ Save** - กด Ctrl+S ใน LDtk
4. **ดูผลทันที** - game reload อัตโนมัติ

### Tips:

- ใช้ **Auto-save** ใน LDtk เพื่อ reload บ่อยขึ้น
- ใช้ **Dual monitor** - LDtk ฝั่งหนึ่ง, Game อีกฝั่ง
- ใช้ **Git** เพื่อ track การเปลี่ยนแปลง level

## 🎮 Example: Celeste-style Platformer

```rust
use engine::runtime::LdtkRuntime;
use ecs::World;

struct Game {
    world: World,
    ldtk: LdtkRuntime,
    current_level: usize,
}

impl Game {
    fn new() -> Self {
        let mut world = World::new();
        let mut ldtk = LdtkRuntime::new();
        
        // Load first level
        ldtk.load("levels/level_1.ldtk", &mut world)
            .expect("Failed to load level");
        
        Self {
            world,
            ldtk,
            current_level: 1,
        }
    }
    
    fn update(&mut self) {
        // Hot-reload check
        if self.ldtk.update(&mut self.world) {
            println!("Level {} reloaded!", self.current_level);
            self.respawn_player();
        }
        
        // Game logic
        self.update_player();
        self.update_physics();
    }
    
    fn respawn_player(&mut self) {
        // Reset player to spawn point
        // ...
    }
}
```

## 🐛 Troubleshooting

### ไฟล์ไม่ reload

- ตรวจสอบว่าไฟล์ path ถูกต้อง
- ตรวจสอบว่า LDtk save ไฟล์สำเร็จ
- ดู log messages (`RUST_LOG=info cargo run`)

### Performance issues

- Hot-reload check ใช้ CPU น้อยมาก
- ถ้ามี lag ตอน reload = level ใหญ่เกินไป
- พิจารณาแบ่ง level เป็นหลายไฟล์

### Entities หาย

- Hot-reload จะ despawn entities เก่าทั้งหมด
- ถ้าต้องการเก็บ state = save ก่อน reload

## 📚 Related

- [LDtk Editor](https://ldtk.io/)
- [LDtk Documentation](https://ldtk.io/docs/)
- [ldtk_rust crate](https://crates.io/crates/ldtk_rust)

## 🎉 Example Project

ดู example ที่ `ecs/examples/ldtk_hot_reload.rs`:

```bash
cargo run --example ldtk_hot_reload -- path/to/your/level.ldtk
```

## 💡 Tips & Best Practices

1. **ใช้ relative paths** - ง่ายต่อการ share project
2. **Organize levels** - แยก folder ตาม world/chapter
3. **Version control** - commit .ldtk files
4. **Backup** - LDtk มี auto-backup แต่ควร commit บ่อยๆ
5. **Test hot-reload** - ทดสอบว่า reload ถูกต้องก่อน production

## 🚀 Next Steps

- [ ] รองรับ Tiled hot-reload
- [ ] Partial reload (reload เฉพาะ layer ที่เปลี่ยน)
- [ ] Editor integration (reload button ใน editor)
- [ ] Network sync (multiplayer level editing)
