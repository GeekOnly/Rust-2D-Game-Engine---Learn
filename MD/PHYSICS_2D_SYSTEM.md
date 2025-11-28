# ระบบ Physics 2D

## ✅ สถานะ: ใช้งานได้แล้ว!

ระบบ Physics 2D สำหรับ XS Game Engine พร้อมใช้งานแล้ว รองรับ:
- ✅ Gravity (แรงโน้มถ่วง)
- ✅ Velocity-based Movement (การเคลื่อนที่ด้วยความเร็ว)
- ✅ AABB Collision Detection (ตรวจจับการชน)
- ✅ Physics Helpers (ฟังก์ชันช่วยเหลือ)

## 📦 Components ที่เกี่ยวข้อง

### 1. Rigidbody 2D (Velocity Component)
```rust
ComponentType::Rigidbody
```
- เก็บความเร็ว (velocity_x, velocity_y)
- รับผลกระทบจากแรงโน้มถ่วง
- ใช้สำหรับ Entity ที่ต้องการเคลื่อนที่

### 2. Box Collider 2D
```rust
ComponentType::BoxCollider
```
- กำหนดขอบเขตการชน (width, height)
- ใช้สำหรับตรวจจับการชนกัน (AABB)

## 🎮 การใช้งาน

### 1. สร้าง Physics World
```rust
use physics::PhysicsWorld;

let mut physics = PhysicsWorld::new();
// Gravity เริ่มต้น: 980 pixels/s² (เหมือน Unity)
```

### 2. เพิ่ม Rigidbody ให้ Entity
```rust
use ecs::{World, ComponentType, ComponentManager};

let mut world = World::new();
let player = world.spawn();

// เพิ่ม Components
world.add_component(player, ComponentType::Transform)?;
world.add_component(player, ComponentType::Rigidbody)?;

// ตั้งค่าความเร็วเริ่มต้น
world.velocities.insert(player, (100.0, 0.0)); // เคลื่อนที่ไปทางขวา
```

### 3. Update Physics ทุก Frame
```rust
let dt = 0.016; // 60 FPS (16ms per frame)
physics.step(dt, &mut world);
```

### 4. ตรวจจับการชน
```rust
// ตรวจสอบการชนระหว่าง 2 Entity
if PhysicsWorld::check_collision(&world, entity1, entity2) {
    println!("Collision detected!");
}
```

## 🔧 Physics Helpers

### Apply Impulse (กระแทก)
```rust
// กระโดด
if let Some(vel) = world.velocities.get_mut(&player) {
    vel.1 += 300.0; // กระโดดขึ้น
}
```

### Apply Force (แรง)
```rust
// เดินไปทางขวา
if let Some(vel) = world.velocities.get_mut(&player) {
    vel.0 += 100.0 * dt; // เพิ่มความเร็วทางขวา
}
```

### Clamp Velocity (จำกัดความเร็ว)
```rust
if let Some(vel) = world.velocities.get_mut(&player) {
    let speed = (vel.0 * vel.0 + vel.1 * vel.1).sqrt();
    if speed > max_speed {
        let scale = max_speed / speed;
        vel.0 *= scale;
        vel.1 *= scale;
    }
}
```

### Apply Damping (ความต้านทาน)
```rust
// ลดความเร็วลงเรื่อยๆ (เหมือนแรงเสียดทาน)
if let Some(vel) = world.velocities.get_mut(&player) {
    let factor = 1.0 - (damping * dt).min(1.0);
    vel.0 *= factor;
    vel.1 *= factor;
}
```

### Stop (หยุด)
```rust
world.velocities.insert(player, (0.0, 0.0));
```

## 🎯 ตัวอย่างการใช้งาน

### ตัวอย่าง 1: Player ที่กระโดดได้
```rust
let mut world = World::new();
let mut physics = PhysicsWorld::new();

// สร้าง Player
let player = world.spawn();
world.add_component(player, ComponentType::Transform)?;
world.add_component(player, ComponentType::Sprite)?;
world.add_component(player, ComponentType::BoxCollider)?;
world.add_component(player, ComponentType::Rigidbody)?;

// ตั้งค่า
world.transforms.get_mut(&player).unwrap().position = [0.0, 100.0, 0.0];
world.velocities.insert(player, (0.0, 0.0));

// Game Loop
loop {
    let dt = 0.016; // 60 FPS

    // กดปุ่มกระโดด
    if jump_pressed {
        if let Some(vel) = world.velocities.get_mut(&player) {
            vel.1 = 300.0; // กระโดด
        }
    }

    // Update physics
    physics.step(dt, &mut world);

    // Render...
}
```

### ตัวอย่าง 2: Platform Game
```rust
// สร้าง Ground
let ground = world.spawn();
world.add_component(ground, ComponentType::Transform)?;
world.add_component(ground, ComponentType::BoxCollider)?;
world.transforms.get_mut(&ground).unwrap().position = [0.0, -50.0, 0.0];
world.colliders.get_mut(&ground).unwrap().width = 200.0;
world.colliders.get_mut(&ground).unwrap().height = 20.0;

// ตรวจสอบการชนกับพื้น
if PhysicsWorld::check_collision(&world, player, ground) {
    // Player อยู่บนพื้น - สามารถกระโดดได้
    can_jump = true;
    
    // หยุดตกลงไป
    if let Some(vel) = world.velocities.get_mut(&player) {
        if vel.1 < 0.0 {
            vel.1 = 0.0;
        }
    }
}
```

### ตัวอย่าง 3: Moving Platform
```rust
// สร้าง Moving Platform
let platform = world.spawn();
world.add_component(platform, ComponentType::Transform)?;
world.add_component(platform, ComponentType::BoxCollider)?;
world.add_component(platform, ComponentType::Rigidbody)?;

// ตั้งค่าความเร็วแนวนอน
world.velocities.insert(platform, (50.0, 0.0));

// ปิด Gravity สำหรับ Platform
// (ต้องแก้ไข PhysicsWorld ให้รองรับ gravity_scale)
```

## 🔬 การทดสอบ

### รัน Unit Tests
```bash
cargo test -p engine runtime::physics
```

ผลลัพธ์:
```
running 4 tests
test runtime::physics::tests::test_collision_detection ... ok
test runtime::physics::tests::test_physics_helpers ... ok
test runtime::physics::tests::test_gravity_application ... ok
test runtime::physics::tests::test_position_update ... ok

test result: ok. 4 passed
```

### รัน Demo
```bash
cargo run --example physics_demo
```

## 📊 Physics Properties

| Property | ค่าเริ่มต้น | หน่วย | คำอธิบาย |
|----------|------------|-------|----------|
| Gravity | 980.0 | pixels/s² | แรงโน้มถ่วง (เหมือน Unity 9.8 m/s²) |
| Velocity | (0.0, 0.0) | pixels/s | ความเร็ว (X, Y) |
| Collider Size | 32x32 | pixels | ขนาด Collider เริ่มต้น |

## 🎓 เทคนิคขั้นสูง

### 1. Variable Jump Height
```rust
// กระโดดสูงต่ำตามระยะเวลากดปุ่ม
if jump_button_released && vel.1 > 0.0 {
    vel.1 *= 0.5; // ลดความเร็วขึ้นลง
}
```

### 2. Coyote Time (ยังกระโดดได้หลังออกจากพื้น)
```rust
let mut coyote_timer = 0.1; // 0.1 วินาที

if !on_ground {
    coyote_timer -= dt;
}

if jump_pressed && coyote_timer > 0.0 {
    // ยังกระโดดได้
}
```

### 3. Jump Buffer (กดกระโดดก่อนถึงพื้น)
```rust
let mut jump_buffer = 0.0;

if jump_pressed {
    jump_buffer = 0.1; // เก็บคำสั่งกระโดดไว้ 0.1 วินาที
}

if on_ground && jump_buffer > 0.0 {
    // กระโดด
    jump_buffer = 0.0;
}
```

## 🚀 Features ที่จะเพิ่มในอนาคต

- [ ] Gravity Scale (ปรับแรงโน้มถ่วงแต่ละ Entity)
- [ ] Friction (แรงเสียดทาน)
- [ ] Bounciness (การกระเด้ง)
- [ ] One-Way Platforms (แพลตฟอร์มทางเดียว)
- [ ] Trigger Colliders (Collider ที่ไม่บล็อก)
- [ ] Collision Layers (Layer-based collision)
- [ ] Collision Callbacks (Event เมื่อชนกัน)
- [ ] Raycast (ยิงเส้นตรวจจับ)

## 📚 เอกสารเพิ่มเติม

- `physics/src/lib.rs` - Physics crate (เดิม)
- `engine/src/runtime/physics.rs` - Physics system ใหม่
- `engine/examples/physics_demo.rs` - ตัวอย่างการใช้งาน
- `MD/COMPONENT_MANAGEMENT.md` - การจัดการ Component

## 🎮 การใช้งานใน Inspector

1. เลือก Entity ใน Hierarchy
2. คลิก "Add Component"
3. เลือก "Rigidbody 2D" จากหมวด Physics
4. ปรับค่า Velocity ใน Inspector
5. เพิ่ม "Box Collider 2D" สำหรับการชน
6. กด Play เพื่อทดสอบ

## ✅ สรุป

ระบบ Physics 2D ของ XS Game Engine **ใช้งานได้แล้ว** และพร้อมสำหรับการพัฒนาเกม 2D แบบ Platform, Puzzle, หรือ Action ได้ทันที!

**ความสามารถหลัก:**
- ✅ Gravity simulation
- ✅ Velocity-based movement  
- ✅ AABB collision detection
- ✅ Physics helpers (impulse, force, damping)
- ✅ Component-based architecture
- ✅ Unity-like API

**ทดสอบแล้ว:**
- ✅ 4/4 Unit tests ผ่าน
- ✅ Demo program รันได้
- ✅ Collision detection ทำงานถูกต้อง
- ✅ Gravity และ velocity ทำงานถูกต้อง
