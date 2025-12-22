# Rapier Physics Migration Guide

## ทำไมต้องเปลี่ยนไปใช้ Rapier?

### ปัญหาของ Simple Physics (Custom)

1. **Jump ไม่ทำงาน** - collision resolution ทำงานในเฟรมเดียวกับ jump ทำให้ velocity ถูก reset
2. **Ground detection ไม่แม่นยำ** - ใช้การเช็คตำแหน่ง Y แบบ manual
3. **Tunneling** - วัตถุเคลื่อนที่เร็วทะลุผ่านกำแพงได้
4. **ไม่มี contact information** - ไม่รู้ว่าชนด้านไหน, แรงเท่าไหร่
5. **Performance ไม่ดีกับวัตถุเยอะ** - O(n²) collision detection

### ข้อดีของ Rapier Physics

✅ **Production-ready** - ใช้ในเกมจริงหลายเกม (Bevy, Fyrox)  
✅ **Accurate collision** - มี CCD, contact normals, penetration depth  
✅ **Proper ground detection** - ใช้ contact normal vector  
✅ **Better performance** - spatial partitioning, SIMD optimization  
✅ **Deterministic** - เหมาะกับ multiplayer/replay  
✅ **Rich features** - joints, sensors, collision groups, events  
✅ **Well-documented** - มี docs และ examples ดี  

## การเปรียบเทียบ

### Simple Physics (เดิม)

```rust
// Ground check - ไม่แม่นยำ
if pos.y >= -1.6 && pos.y <= -1.4 && velocity_y < 1.0 {
    is_grounded = true;
}

// Jump - มีปัญหา velocity ถูก reset
if is_grounded {
    velocity_y = -jump_force;
    is_grounded = false;
}
```

**ปัญหา:**
- ต้อง hardcode ตำแหน่ง ground
- ไม่รู้ว่าชนพื้นจริงหรือแค่อยู่ใกล้
- Collision resolution อาจ reset velocity ทันที

### Rapier Physics (ใหม่)

```rust
// Ground check - แม่นยำด้วย contact normal
fn is_grounded(&self, entity: Entity) -> bool {
    for contact in self.contacts_with(entity) {
        // Check if normal points upward (we're standing on something)
        if contact.normal.y < -0.7 {
            return true;
        }
    }
    false
}

// Jump - ทำงานได้ถูกต้อง
if physics.is_grounded(player, &world) {
    rigidbody.velocity.1 = -jump_force;
    // Rapier จะไม่ reset velocity ในเฟรมเดียวกัน
}
```

**ข้อดี:**
- ตรวจสอบจาก contact จริง
- รู้ว่าชนด้านไหน (normal vector)
- ไม่มีปัญหา velocity reset

## วิธีการ Migrate

### 1. เพิ่ม Rapier dependency

```toml
# physics/Cargo.toml
[dependencies]
rapier2d = "0.22"

[features]
default = ["rapier"]
rapier = []
simple = []  # Keep old physics as fallback
```

### 2. สร้าง Rapier backend

```rust
// physics/src/rapier_backend.rs
pub struct RapierPhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    // ... other Rapier components
    
    // Mapping ECS <-> Rapier
    entity_to_body: HashMap<Entity, RigidBodyHandle>,
}

impl RapierPhysicsWorld {
    pub fn step(&mut self, dt: f32, world: &mut World) {
        // 1. Sync ECS -> Rapier
        self.sync_from_ecs(world);
        
        // 2. Run physics
        self.physics_pipeline.step(...);
        
        // 3. Sync Rapier -> ECS
        self.sync_to_ecs(world);
    }
    
    pub fn is_grounded(&self, entity: Entity) -> bool {
        // Check contact normals
        for contact in self.contacts_with(entity) {
            if contact.normal.y < -0.7 {
                return true;
            }
        }
        false
    }
}
```

### 3. อัพเดท Player Controller

**เดิม (Lua):**
```lua
-- Ground check แบบเดิม - ไม่แม่นยำ
if pos and math.abs(velocity_y) < 1.0 and pos.y >= -1.6 and pos.y <= -1.4 then
    is_grounded = true
end

-- Jump
if is_key_just_pressed("Space") and is_grounded then
    velocity_y = -jump_force
    is_grounded = false
end
```

**ใหม่ (Lua with Rapier):**
```lua
-- Ground check ผ่าน Rapier API
is_grounded = is_grounded_rapier()  -- ใช้ contact normals

-- Jump - ทำงานได้ถูกต้อง
if is_key_just_pressed("Space") and is_grounded then
    velocity_y = -jump_force
    -- ไม่ต้อง set is_grounded = false
    -- Rapier จะจัดการให้อัตโนมัติ
end
```

### 4. เพิ่ม Lua bindings สำหรับ Rapier

```rust
// script/src/lib.rs
fn is_grounded_rapier(lua: &Lua, _: ()) -> LuaResult<bool> {
    let entity = get_current_entity(lua)?;
    let physics = get_physics_world(lua)?;
    Ok(physics.is_grounded(entity, &world))
}

// Register function
lua.globals().set("is_grounded_rapier", lua.create_function(is_grounded_rapier)?)?;
```

## ตัวอย่างการใช้งาน

### Basic Setup

```rust
use physics::rapier_backend::RapierPhysicsWorld;

let mut physics = RapierPhysicsWorld::new();
physics.set_gravity(150.0); // Pixels per second²

// Game loop
loop {
    let dt = calculate_delta_time();
    physics.step(dt, &mut world);
}
```

### Player Controller

```rust
// Check if can jump
if input.just_pressed(KeyCode::Space) {
    if physics.is_grounded(player_entity, &world) {
        // Apply jump
        if let Some(rb) = world.rigidbodies.get_mut(&player_entity) {
            rb.velocity.1 = -25.0; // Jump force
        }
    }
}

// Horizontal movement
if let Some(rb) = world.rigidbodies.get_mut(&player_entity) {
    rb.velocity.0 = input.axis() * move_speed;
}
```

### Advanced: Raycast Ground Check

```rust
// Alternative ground check using raycast
let is_grounded = physics.raycast_ground(
    player_entity,
    &world,
    0.1 // Distance to check (slightly below player)
);
```

## Performance Tips

1. **Enable CCD for fast objects**
   ```rust
   rigidbody.ccd_enabled = true;
   ```

2. **Use collision groups**
   ```rust
   collider.collision_groups = InteractionGroups::new(
       Group::GROUP_1,  // This object's group
       Group::GROUP_2,  // Groups it collides with
   );
   ```

3. **Disable unnecessary features**
   ```rust
   rigidbody.set_enabled_rotations(false, false, false); // 2D platformer
   ```

## Migration Checklist

- [ ] เพิ่ม rapier2d dependency
- [ ] สร้าง RapierPhysicsWorld backend
- [ ] เพิ่ม entity <-> handle mapping
- [ ] Implement sync_from_ecs และ sync_to_ecs
- [ ] Implement is_grounded ด้วย contact normals
- [ ] อัพเดท player controller ใช้ is_grounded ใหม่
- [ ] เพิ่ม Lua bindings สำหรับ Rapier functions
- [ ] ทดสอบ jump, movement, collision
- [ ] ปรับ gravity และ jump_force ให้เหมาะสม
- [ ] เพิ่ม CCD สำหรับวัตถุเคลื่อนที่เร็ว

## Troubleshooting

### Jump ยังไม่ทำงาน

1. ตรวจสอบ contact normal threshold
   ```rust
   if contact.normal.y < -0.7 { // ลองปรับค่า
   ```

2. ตรวจสอบว่า CCD เปิดอยู่
   ```rust
   rigidbody.ccd_enabled = true;
   ```

3. ตรวจสอบ gravity scale
   ```rust
   rigidbody.gravity_scale = 1.0; // ไม่ใช่ 0
   ```

### Performance ไม่ดี

1. ใช้ collision groups
2. ลด integration steps
3. ใช้ simplified colliders (box แทน polygon)

### Determinism issues

1. ใช้ fixed timestep
   ```rust
   let fixed_dt = 1.0 / 60.0;
   physics.step(fixed_dt, &mut world);
   ```

2. เปิด deterministic mode
   ```rust
   integration_parameters.deterministic = true;
   ```

## สรุป

การเปลี่ยนไปใช้ Rapier จะแก้ปัญหา jump และทำให้ physics system production-ready:

✅ Ground detection แม่นยำด้วย contact normals  
✅ ไม่มีปัญหา velocity reset  
✅ Performance ดีกว่า  
✅ Features ครบครัน  
✅ Deterministic สำหรับ multiplayer  

**แนะนำให้ migrate ทันที** สำหรับ production project!
