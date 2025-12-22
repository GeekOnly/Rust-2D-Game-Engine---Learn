# คู่มือการใช้งาน LDtk กับ Game Engine

## 🎮 Overview

Engine รองรับการใช้งาน [LDtk](https://ldtk.io/) (Level Designer Toolkit) แบบ **hybrid workflow**:
- ✏️ ออกแบบ level ใน LDtk Editor (external tool)
- 🔥 Hot-reload แบบ real-time ใน engine
- 🎯 ไม่ต้อง restart game เมื่อแก้ไข level

## 📋 Prerequisites

1. **ติดตั้ง LDtk Editor**
   - Download: https://ldtk.io/
   - รองรับ Windows, Mac, Linux

2. **Project Structure**
   ```
   your-game/
   ├── levels/
   │   ├── world.ldtk      # LDtk project file
   │   └── tilesets/       # Tileset images
   ├── scenes/
   │   └── main.json       # Game scene
   └── scripts/
       └── level_loader.lua
   ```

## 🚀 Quick Start

### 1. สร้าง Level ใน LDtk

1. เปิด LDtk Editor
2. สร้าง project ใหม่: `File > New Project`
3. ตั้งชื่อ: `world.ldtk`
4. เพิ่ม Tileset และ Layers
5. วาด level ของคุณ
6. Save (Ctrl+S)

### 2. Load Level ใน Engine (Rust)

```rust
use engine::runtime::LdtkRuntime;
use ecs::World;

fn main() {
    let mut world = World::new();
    let mut ldtk = LdtkRuntime::new();
    
    // Load level with hot-reload
    ldtk.load("levels/world.ldtk", &mut world)
        .expect("Failed to load level");
    
    // Game loop
    loop {
        // Check for hot-reload
        if ldtk.update(&mut world) {
            println!("Level reloaded!");
        }
        
        // Update game
        update_game(&mut world);
        render(&world);
    }
}
```

### 3. Load Level ใน Lua Script

```lua
-- level_loader.lua
local ldtk_runtime = nil

function on_start()
    -- สร้าง LDtk runtime
    ldtk_runtime = LdtkRuntime.new()
    
    -- Load level
    local success = ldtk_runtime:load("levels/world.ldtk")
    if success then
        print("Level loaded!")
    else
        print("Failed to load level")
    end
end

function on_update(dt)
    -- Check for hot-reload
    if ldtk_runtime:update() then
        print("Level hot-reloaded!")
        -- Reset game state if needed
        reset_player_position()
    end
end
```

## 🔥 Hot-Reload Workflow

### การใช้งานแบบ Designer

1. **เปิด 2 หน้าต่าง**:
   - หน้าต่างที่ 1: LDtk Editor
   - หน้าต่างที่ 2: Game Engine (running)

2. **แก้ไข Level**:
   - แก้ไขใน LDtk
   - กด Save (Ctrl+S)
   - ดูผลทันทีใน Game

3. **Iterate ได้เร็ว**:
   - ไม่ต้อง restart game
   - ไม่ต้อง recompile
   - เห็นผลทันที

### Tips สำหรับ Hot-Reload

```rust
// ปิด hot-reload ใน production
#[cfg(not(debug_assertions))]
ldtk.set_enabled(false);

// Reset game state เมื่อ reload
if ldtk.update(&mut world) {
    reset_player_position(&mut world);
    reset_enemies(&mut world);
    play_sound("level_reload");
}
```

## 📦 LDtk Features Support

### ✅ รองรับแล้ว

- ✅ Multiple levels
- ✅ Multiple layers
- ✅ Tile layers
- ✅ Entity layers
- ✅ Level positioning
- ✅ Hot-reload

### 🚧 กำลังพัฒนา

- 🚧 Auto-tiling
- 🚧 Entity properties
- 🚧 Level references
- 🚧 Custom fields

## 🎯 Use Cases

### 1. Platformer Game (Celeste-style)

```rust
struct Game {
    world: World,
    ldtk: LdtkRuntime,
    current_level: usize,
}

impl Game {
    fn load_level(&mut self, level_index: usize) {
        let path = format!("levels/level_{}.ldtk", level_index);
        self.ldtk.load(&path, &mut self.world)
            .expect("Failed to load level");
        self.current_level = level_index;
    }
    
    fn update(&mut self) {
        // Hot-reload
        if self.ldtk.update(&mut self.world) {
            self.respawn_player();
        }
        
        // Game logic
        self.update_physics();
        self.check_level_complete();
    }
}
```

### 2. Top-Down RPG

```rust
struct RPGGame {
    world: World,
    ldtk: LdtkRuntime,
    current_map: String,
}

impl RPGGame {
    fn change_map(&mut self, map_name: &str) {
        // Unload old map
        if !self.current_map.is_empty() {
            self.ldtk.unload(&self.current_map).ok();
        }
        
        // Load new map
        let path = format!("levels/{}.ldtk", map_name);
        self.ldtk.load(&path, &mut self.world)
            .expect("Failed to load map");
        self.current_map = map_name.to_string();
    }
}
```

### 3. Puzzle Game

```lua
-- puzzle_game.lua
local levels = {
    "levels/tutorial.ldtk",
    "levels/easy_1.ldtk",
    "levels/easy_2.ldtk",
    "levels/medium_1.ldtk",
    -- ...
}

local current_level = 1
local ldtk = LdtkRuntime.new()

function load_level(index)
    ldtk:load(levels[index])
    current_level = index
end

function next_level()
    current_level = current_level + 1
    if current_level <= #levels then
        load_level(current_level)
    else
        print("Game completed!")
    end
end

function restart_level()
    load_level(current_level)
end
```

## 🛠️ Advanced Usage

### Multiple Levels

```rust
let mut ldtk = LdtkRuntime::new();

// Load multiple levels
ldtk.load("levels/world_1.ldtk", &mut world)?;
ldtk.load("levels/world_2.ldtk", &mut world)?;
ldtk.load("levels/ui.ldtk", &mut world)?;

// All files will hot-reload automatically
```

### Conditional Hot-Reload

```rust
let mut ldtk = LdtkRuntime::new();

// Enable only in debug mode
ldtk.set_enabled(cfg!(debug_assertions));

// Or based on settings
ldtk.set_enabled(game_settings.developer_mode);
```

### Custom Reload Handler

```rust
if ldtk.update(&mut world) {
    // Save player state
    let player_pos = get_player_position(&world);
    let player_health = get_player_health(&world);
    
    // Level reloaded automatically
    
    // Restore player state
    set_player_position(&mut world, player_pos);
    set_player_health(&mut world, player_health);
    
    // Show notification
    show_notification("Level reloaded!");
}
```

## 🐛 Troubleshooting

### ไฟล์ไม่ reload

**ปัญหา**: แก้ไขไฟล์แล้วแต่ game ไม่ reload

**แก้ไข**:
1. ตรวจสอบว่า save สำเร็จใน LDtk
2. ตรวจสอบ path ของไฟล์
3. ดู console log (`RUST_LOG=info`)
4. ตรวจสอบว่า hot-reload เปิดอยู่

```rust
// Debug hot-reload
println!("Hot-reload enabled: {}", ldtk.is_enabled());
println!("Watched files: {:?}", ldtk.watched_files());
```

### Performance ช้าตอน reload

**ปัญหา**: Level ใหญ่ reload ช้า

**แก้ไข**:
1. แบ่ง level เป็นหลายไฟล์เล็กๆ
2. ใช้ level streaming
3. Load เฉพาะส่วนที่ต้องการ

### Entities หาย

**ปัญหา**: Entities บางตัวหายหลัง reload

**แก้ไข**:
```rust
// Save runtime entities before reload
let runtime_entities = save_runtime_entities(&world);

if ldtk.update(&mut world) {
    // Restore runtime entities
    restore_runtime_entities(&mut world, runtime_entities);
}
```

## 📚 LDtk Resources

- **Official Website**: https://ldtk.io/
- **Documentation**: https://ldtk.io/docs/
- **Discord**: https://discord.gg/ldtk
- **Examples**: https://ldtk.io/gallery/

## 🎓 Learning Path

1. **เริ่มต้น**: ทำ tutorial ใน LDtk
2. **ทดลอง**: สร้าง level เล็กๆ
3. **Hot-reload**: ทดสอบ hot-reload workflow
4. **Production**: ใช้ใน project จริง

## 💡 Best Practices

### 1. File Organization

```
levels/
├── world_1/
│   ├── level_1.ldtk
│   ├── level_2.ldtk
│   └── tilesets/
├── world_2/
│   ├── level_1.ldtk
│   └── tilesets/
└── shared/
    └── tilesets/
```

### 2. Version Control

```gitignore
# Commit .ldtk files
*.ldtk

# Ignore backup files
*.ldtk.backup
*.ldtk~
```

### 3. Team Workflow

- ใช้ Git สำหรับ version control
- แยก level ตาม designer
- Review changes ก่อน merge
- ใช้ LDtk's external levels feature

### 4. Performance

```rust
// Disable hot-reload in production
#[cfg(not(debug_assertions))]
ldtk.set_enabled(false);

// Unload unused levels
ldtk.unload("levels/old_level.ldtk")?;

// Load levels on-demand
if player_near_door {
    ldtk.load("levels/next_room.ldtk", &mut world)?;
}
```

## 🚀 Next Steps

1. ลองสร้าง level แรกใน LDtk
2. ทดสอบ hot-reload
3. ดู example: `cargo run --example ldtk_hot_reload`
4. อ่าน [LDTK_HOT_RELOAD.md](../ecs/LDTK_HOT_RELOAD.md) สำหรับ API details

---

Happy Level Designing! 🎮✨
