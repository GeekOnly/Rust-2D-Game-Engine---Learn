# 📚 Recommended Libraries for Rust 2D Game Engine

## 🎯 Priority Levels
- 🔴 **Critical** - ต้องมี
- 🟡 **High** - ควรมี
- 🟢 **Medium** - ดีถ้ามี
- 🔵 **Low** - ไว้ทีหลัง

---

## 🔴 Critical Libraries (ต้องมี)

### Audio
```toml
# audio/Cargo.toml
[dependencies]
kira = "0.8"  # Game audio engine
```

### Math
```toml
# engine_core/Cargo.toml
[dependencies]
glam = { version = "0.25", features = ["serde"] }  # Fast math
```

### Image Processing
```toml
# render/Cargo.toml
[dependencies]
image = "0.24"  # Image loading
```

### Font Rendering
```toml
# render/Cargo.toml
[dependencies]
fontdue = "0.8"  # Fast font rasterization
```

### Gamepad Support
```toml
# input/Cargo.toml
[dependencies]
gilrs = "0.10"  # Gamepad support
```

---

## 🟡 High Priority Libraries (ควรมี)

### Better Physics
```toml
# physics/Cargo.toml
[dependencies]
rapier2d = "0.18"
parry2d = "0.13"
```

### Animation & Tweening
```toml
# engine_core/Cargo.toml
[dependencies]
interpolation = "0.2"
ezing = "0.4"
```

### Tilemap Support
```toml
# render/Cargo.toml
[dependencies]
tiled = "0.11"  # Load Tiled maps
```

### Color Management
```toml
# render/Cargo.toml
[dependencies]
palette = "0.7"
```

### Asset Hot Reload
```toml
# editor/Cargo.toml
[dependencies]
notify = "6.1"  # File watcher
```

### Better Logging
```toml
# engine_core/Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 🟢 Medium Priority Libraries (ดีถ้ามี)

### Profiling
```toml
# editor/Cargo.toml
[dependencies]
puffin = "0.18"
puffin_egui = "0.25"
```

### Better Serialization
```toml
# engine_core/Cargo.toml
[dependencies]
bincode = "1.3"  # Binary format (save files)
ron = "0.8"      # Config files
```

### Random Generation
```toml
# engine_core/Cargo.toml
[dependencies]
fastrand = "2.0"
```

### Compression
```toml
# engine_core/Cargo.toml
[dependencies]
flate2 = "1.0"  # Asset bundling
```

---

## 🔵 Low Priority Libraries (ไว้ทีหลัง)

### Networking
```toml
# network/Cargo.toml (create new crate)
[dependencies]
renet = "0.0.15"
```

### Better ECS
```toml
# ecs/Cargo.toml
[dependencies]
hecs = "0.10"  # Replace current ECS
```

---

## 📦 Complete Example: Updated Workspace

### Root Cargo.toml
```toml
[workspace]
members = [
  "engine_core",
  "ecs",
  "render",
  "physics",
  "script",
  "input",
  "editor",
  "engine",
  "audio",      # NEW
]
resolver = "2"

[workspace.dependencies]
anyhow = "1.0"
log = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
glam = { version = "0.25", features = ["serde"] }
```

### audio/Cargo.toml (NEW)
```toml
[package]
name = "audio"
version = "0.1.0"
edition = "2021"

[dependencies]
kira = "0.8"
anyhow = "1.0"
```

### engine_core/Cargo.toml (UPDATED)
```toml
[package]
name = "engine_core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
glam = { version = "0.25", features = ["serde"] }
interpolation = "0.2"
ezing = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
fastrand = "2.0"
bincode = "1.3"
ron = "0.8"
flate2 = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

### render/Cargo.toml (UPDATED)
```toml
[package]
name = "render"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "0.19"
image = "0.24"
fontdue = "0.8"
palette = "0.7"
tiled = "0.11"
glam = { version = "0.25", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### physics/Cargo.toml (UPDATED)
```toml
[package]
name = "physics"
version = "0.1.0"
edition = "2021"

[dependencies]
ecs = { path = "../ecs" }
rapier2d = "0.18"
parry2d = "0.13"
glam = "0.25"
```

### input/Cargo.toml (UPDATED)
```toml
[package]
name = "input"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.29"
gilrs = "0.10"
serde = { version = "1.0", features = ["derive"] }
```

### editor/Cargo.toml (UPDATED)
```toml
[package]
name = "editor"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.27"
egui-wgpu = "0.27"
egui-winit = "0.27"
notify = "6.1"
puffin = "0.18"
puffin_egui = "0.25"
rfd = "0.14"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 🚀 Implementation Priority

### Phase 1: Essential (Week 1-2)
1. ✅ Add `glam` for math
2. ✅ Add `kira` for audio
3. ✅ Add `image` for textures
4. ✅ Add `fontdue` for fonts
5. ✅ Add `gilrs` for gamepad

### Phase 2: Quality of Life (Week 3-4)
1. ✅ Add `rapier2d` for better physics
2. ✅ Add `interpolation` + `ezing` for animations
3. ✅ Add `tiled` for tilemap support
4. ✅ Add `notify` for hot reload
5. ✅ Add `tracing` for better logging

### Phase 3: Polish (Week 5-6)
1. ✅ Add `puffin` for profiling
2. ✅ Add `palette` for colors
3. ✅ Add `bincode` + `ron` for serialization
4. ✅ Add `flate2` for compression

### Phase 4: Advanced (Later)
1. ⏳ Add `renet` for networking
2. ⏳ Replace ECS with `hecs`
3. ⏳ Add particle system
4. ⏳ Add shader support

---

## 💡 Usage Examples

### Audio System
```rust
// audio/src/lib.rs
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

pub struct AudioSystem {
    manager: AudioManager,
}

impl AudioSystem {
    pub fn new() -> anyhow::Result<Self> {
        let manager = AudioManager::new(AudioManagerSettings::default())?;
        Ok(Self { manager })
    }

    pub fn play_sound(&mut self, path: &str) -> anyhow::Result<()> {
        let sound = StaticSoundData::from_file(path)?;
        self.manager.play(sound)?;
        Ok(())
    }

    pub fn play_music(&mut self, path: &str, looping: bool) -> anyhow::Result<()> {
        let mut settings = StaticSoundSettings::default();
        settings.loop_behavior = if looping {
            kira::sound::static_sound::LoopBehavior::default()
        } else {
            kira::sound::static_sound::LoopBehavior::default()
        };
        let sound = StaticSoundData::from_file(path)?.with_settings(settings);
        self.manager.play(sound)?;
        Ok(())
    }
}
```

### Math with glam
```rust
use glam::{Vec2, Vec3, Mat4};

// Transform component
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(1.0),
            glam::Quat::from_rotation_z(self.rotation),
            self.position.extend(0.0),
        )
    }
}
```

### Animation with ezing
```rust
use ezing::*;

pub struct Tween {
    start: f32,
    end: f32,
    duration: f32,
    elapsed: f32,
    easing: fn(f32) -> f32,
}

impl Tween {
    pub fn new(start: f32, end: f32, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing: quad_inout,
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.elapsed += dt;
        let t = (self.elapsed / self.duration).min(1.0);
        let eased = (self.easing)(t);
        self.start + (self.end - self.start) * eased
    }
}
```

### Hot Reload with notify
```rust
use notify::{Watcher, RecursiveMode, Event};

pub struct AssetWatcher {
    watcher: notify::RecommendedWatcher,
}

impl AssetWatcher {
    pub fn new() -> anyhow::Result<Self> {
        let watcher = notify::recommended_watcher(|res: Result<Event, _>| {
            match res {
                Ok(event) => {
                    println!("Asset changed: {:?}", event);
                    // Reload asset here
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        })?;

        Ok(Self { watcher })
    }

    pub fn watch(&mut self, path: &str) -> anyhow::Result<()> {
        self.watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        Ok(())
    }
}
```

---

## 📊 Size Impact

| Library | Binary Size | Compile Time | Worth It? |
|---------|-------------|--------------|-----------|
| glam | +50 KB | +2s | ✅ Yes |
| kira | +500 KB | +10s | ✅ Yes |
| rapier2d | +2 MB | +30s | ✅ Yes |
| image | +300 KB | +8s | ✅ Yes |
| fontdue | +100 KB | +3s | ✅ Yes |
| gilrs | +200 KB | +5s | ✅ Yes |
| tiled | +150 KB | +4s | ✅ Yes |
| puffin | +100 KB | +3s | 🟡 Maybe |
| renet | +400 KB | +12s | 🟡 If needed |

**Total Impact:** ~4 MB, +80s compile time
**Worth it?** Absolutely! คุณภาพดีขึ้นมาก

---

## ⚠️ Common Pitfalls

### 1. Don't Mix Math Libraries
```rust
// ❌ BAD
use nalgebra::Vector2;
use glam::Vec2;

// ✅ GOOD - Pick one
use glam::Vec2;
```

### 2. Audio on Separate Thread
```rust
// ✅ GOOD
std::thread::spawn(move || {
    audio_system.update();
});
```

### 3. Hot Reload Can Be Slow
```rust
// ✅ GOOD - Debounce file changes
let mut last_reload = Instant::now();
if last_reload.elapsed() > Duration::from_millis(500) {
    reload_asset();
    last_reload = Instant::now();
}
```

---

## 🎯 Final Recommendations

### Must Have (เริ่มเลย):
1. `glam` - Math
2. `kira` - Audio
3. `image` - Textures
4. `gilrs` - Gamepad

### Should Have (สัปดาห์หน้า):
1. `rapier2d` - Physics
2. `fontdue` - Fonts
3. `ezing` - Animations
4. `notify` - Hot reload

### Nice to Have (ทีหลัง):
1. `puffin` - Profiling
2. `tiled` - Tilemap
3. `renet` - Networking

---

**Last Updated:** 2025-11-26
**Status:** Ready to implement
**Estimated Time:** 2-3 weeks for all critical libraries
