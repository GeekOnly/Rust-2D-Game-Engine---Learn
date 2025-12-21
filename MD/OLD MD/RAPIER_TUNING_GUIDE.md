# Rapier Physics Tuning Guide

## การปรับแต่งพารามิเตอร์สำหรับ Platformer

คู่มือนี้จะช่วยให้คุณปรับแต่ง physics parameters ให้เหมาะกับเกม platformer แบบ Celeste

## 🎯 พารามิเตอร์หลัก

### 1. Gravity (แรงโน้มถ่วง)

```rust
// physics/src/rapier_backend.rs
impl Default for RapierPhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, 150.0], // Positive Y = down
            // ...
        }
    }
}
```

**แนะนำ:**
- **Floaty (ลอยนาน)**: 100-120 pixels/s²
- **Normal (ปกติ)**: 150-180 pixels/s²
- **Heavy (ตกเร็ว)**: 200-300 pixels/s²
- **Celeste-like**: 150-170 pixels/s²

### 2. Jump Force (แรงกระโดด)

```lua
-- player_controller.lua
local jump_force = 25.0  -- Negative = up
```

**แนะนำ:**
- **Low jump**: 15-20
- **Normal jump**: 25-30
- **High jump**: 35-45
- **Celeste-like**: 25-28

**สูตรคำนวณ:**
```
jump_height = (jump_force²) / (2 * gravity)

ตัวอย่าง:
jump_force = 25
gravity = 150
jump_height = (25²) / (2 * 150) = 625 / 300 = 2.08 units
```

### 3. Move Speed (ความเร็วเคลื่อนที่)

```lua
local move_speed = 3.0  -- Units per second
```

**แนะนำ:**
- **Slow**: 2-3 units/s
- **Normal**: 4-6 units/s
- **Fast**: 7-10 units/s
- **Celeste-like**: 3-4 units/s

### 4. Dash Speed (ความเร็ว dash)

```lua
local dash_speed = 10.0
local dash_duration = 0.2  -- seconds
```

**แนะนำ:**
- **Short dash**: speed=8, duration=0.15
- **Normal dash**: speed=10, duration=0.2
- **Long dash**: speed=12, duration=0.25
- **Celeste-like**: speed=10-12, duration=0.2

## 🎮 Feel Tuning

### Jump Feel

#### 1. Variable Jump Height

```lua
-- Release Space early = short jump
if not is_key_down("Space") and velocity_y < 0.0 then
    velocity_y = velocity_y * 0.5  -- Cut velocity
end
```

**ปรับ multiplier:**
- **0.3** - Very responsive (short jump)
- **0.5** - Balanced (Celeste-like)
- **0.7** - Less responsive (longer minimum jump)

#### 2. Coyote Time (ยังกระโดดได้หลังออกจากขอบ)

```lua
local coyote_time = 0.1  -- seconds
local time_since_grounded = 0.0

function Update(dt)
    if is_grounded_rapier then
        time_since_grounded = 0.0
    else
        time_since_grounded = time_since_grounded + dt
    end
    
    -- Can jump if recently grounded
    local can_jump = time_since_grounded < coyote_time
    
    if is_key_just_pressed("Space") and can_jump then
        velocity_y = -jump_force
    end
end
```

**แนะนำ:**
- **Tight**: 0.05-0.08s
- **Normal**: 0.1-0.15s (Celeste)
- **Forgiving**: 0.2-0.3s

#### 3. Jump Buffer (กด jump ก่อนถึงพื้น)

```lua
local jump_buffer_time = 0.1
local jump_buffer = 0.0

function Update(dt)
    -- Update buffer
    if is_key_just_pressed("Space") then
        jump_buffer = jump_buffer_time
    end
    jump_buffer = math.max(0, jump_buffer - dt)
    
    -- Jump if buffered and grounded
    if jump_buffer > 0 and is_grounded_rapier then
        velocity_y = -jump_force
        jump_buffer = 0
    end
end
```

**แนะนำ:**
- **Tight**: 0.05-0.08s
- **Normal**: 0.1-0.15s (Celeste)
- **Forgiving**: 0.2-0.3s

### Movement Feel

#### 1. Acceleration (ความเร่ง)

```lua
local target_velocity_x = 0.0
local current_velocity_x = 0.0
local acceleration = 20.0  -- Units/s²

function Update(dt)
    -- Set target
    if is_key_down("A") then
        target_velocity_x = -move_speed
    elseif is_key_down("D") then
        target_velocity_x = move_speed
    else
        target_velocity_x = 0.0
    end
    
    -- Lerp to target
    current_velocity_x = lerp(current_velocity_x, target_velocity_x, acceleration * dt)
    velocity_x = current_velocity_x
end
```

**แนะนำ:**
- **Instant**: ไม่ใช้ acceleration (ตั้งค่าตรง ๆ)
- **Responsive**: 15-25 units/s²
- **Smooth**: 8-12 units/s²
- **Celeste-like**: Instant (no acceleration)

#### 2. Air Control (ควบคุมในอากาศ)

```lua
local ground_move_speed = 5.0
local air_move_speed = 4.0  -- Slower in air

function Update(dt)
    local current_move_speed = is_grounded_rapier and ground_move_speed or air_move_speed
    
    if is_key_down("A") then
        velocity_x = -current_move_speed
    elseif is_key_down("D") then
        velocity_x = current_move_speed
    end
end
```

**แนะนำ:**
- **Full control**: air_speed = ground_speed
- **Reduced control**: air_speed = ground_speed * 0.7-0.9
- **Celeste-like**: Full control

#### 3. Friction/Deceleration

```lua
local ground_friction = 0.8  -- 0-1
local air_friction = 0.95    -- Less friction in air

function Update(dt)
    local friction = is_grounded_rapier and ground_friction or air_friction
    
    if not is_key_down("A") and not is_key_down("D") then
        velocity_x = velocity_x * friction
        if math.abs(velocity_x) < 0.1 then
            velocity_x = 0.0
        end
    end
end
```

**แนะนำ:**
- **Instant stop**: friction = 0.0 (set to 0 directly)
- **Quick stop**: friction = 0.5-0.7
- **Smooth stop**: friction = 0.8-0.9
- **Ice**: friction = 0.95-0.99

## 🔧 Rapier-Specific Tuning

### 1. Ground Detection Threshold

```rust
// physics/src/rapier_backend.rs
pub fn is_grounded(&self, entity: Entity, _world: &World) -> bool {
    for contact in self.contacts_with(entity) {
        if contact.normal.y < -0.7 {  // ← ปรับค่านี้
            return true;
        }
    }
    false
}
```

**แนะนำ:**
- **Strict (flat ground only)**: -0.9 to -1.0
- **Normal**: -0.7 to -0.8 (Celeste-like)
- **Forgiving (slopes OK)**: -0.5 to -0.6

### 2. CCD (Continuous Collision Detection)

```rust
// Enable for fast-moving objects
rigidbody.ccd_enabled = true;
```

**เมื่อไหร่ควรเปิด:**
- ✅ Player (ถ้า dash เร็ว)
- ✅ Bullets/projectiles
- ✅ Fast-moving platforms
- ❌ Static objects
- ❌ Slow-moving objects

### 3. Gravity Scale

```lua
-- Per-entity gravity multiplier
set_gravity_scale(1.0)  -- Normal
set_gravity_scale(0.5)  -- Floaty
set_gravity_scale(2.0)  -- Heavy
```

**Use cases:**
- **Feather/balloon**: 0.2-0.5
- **Normal character**: 1.0
- **Heavy object**: 1.5-2.0
- **Zero gravity**: 0.0

## 📊 Preset Configurations

### Celeste-like

```lua
-- Gravity
physics.set_gravity(160.0)

-- Player
local move_speed = 3.5
local jump_force = 27.0
local dash_speed = 12.0
local dash_duration = 0.2

-- Feel
local coyote_time = 0.15
local jump_buffer_time = 0.1
local air_control = 1.0  -- Full control
```

### Super Meat Boy-like

```lua
-- Gravity
physics.set_gravity(200.0)

-- Player
local move_speed = 6.0
local jump_force = 30.0
local dash_speed = 15.0

-- Feel
local coyote_time = 0.08
local jump_buffer_time = 0.08
local air_control = 1.0
```

### Hollow Knight-like

```lua
-- Gravity
physics.set_gravity(140.0)

-- Player
local move_speed = 4.0
local jump_force = 25.0
local dash_speed = 10.0

-- Feel
local coyote_time = 0.12
local jump_buffer_time = 0.12
local air_control = 0.8  -- Reduced in air
```

### Floaty/Casual

```lua
-- Gravity
physics.set_gravity(100.0)

-- Player
local move_speed = 3.0
local jump_force = 20.0

-- Feel
local coyote_time = 0.2
local jump_buffer_time = 0.2
local air_control = 1.0
```

## 🎨 Advanced Techniques

### 1. Wall Jump

```lua
local is_touching_wall = false
local wall_jump_force_x = 15.0
local wall_jump_force_y = 25.0

function Update(dt)
    -- Check wall (simplified)
    is_touching_wall = check_wall_collision()
    
    if is_key_just_pressed("Space") and is_touching_wall and not is_grounded_rapier then
        -- Jump away from wall
        velocity_x = wall_direction * wall_jump_force_x
        velocity_y = -wall_jump_force_y
    end
end
```

### 2. Wall Slide

```lua
local wall_slide_speed = 1.0

function Update(dt)
    if is_touching_wall and not is_grounded_rapier and velocity_y > 0 then
        -- Slow down fall
        velocity_y = math.min(velocity_y, wall_slide_speed)
    end
end
```

### 3. Double Jump

```lua
local jumps_remaining = 2
local max_jumps = 2

function Update(dt)
    if is_grounded_rapier then
        jumps_remaining = max_jumps
    end
    
    if is_key_just_pressed("Space") and jumps_remaining > 0 then
        velocity_y = -jump_force
        jumps_remaining = jumps_remaining - 1
    end
end
```

### 4. Fast Fall

```lua
local fast_fall_multiplier = 2.0

function Update(dt)
    -- Hold down to fall faster
    if is_key_down("S") and velocity_y > 0 then
        set_gravity_scale(fast_fall_multiplier)
    else
        set_gravity_scale(1.0)
    end
end
```

## 🧪 Testing Checklist

- [ ] Jump height feels right
- [ ] Jump is responsive (no delay)
- [ ] Can jump consistently from ground
- [ ] Can't jump in air (unless double jump)
- [ ] Variable jump height works
- [ ] Movement speed feels good
- [ ] Dash distance is appropriate
- [ ] Coyote time is forgiving enough
- [ ] Jump buffer prevents missed jumps
- [ ] No tunneling through walls
- [ ] Ground detection is accurate
- [ ] Slopes work correctly (if applicable)

## 📈 Iteration Process

1. **Start with defaults** (Celeste-like preset)
2. **Test jump** - adjust jump_force and gravity
3. **Test movement** - adjust move_speed
4. **Add feel** - coyote time, jump buffer
5. **Fine-tune** - acceleration, friction
6. **Playtest** - get feedback
7. **Iterate** - repeat 2-6

## 🎯 Common Issues & Solutions

### Jump feels floaty
- ✅ Increase gravity (150 → 180)
- ✅ Reduce jump_force (25 → 22)
- ✅ Reduce variable jump multiplier (0.5 → 0.3)

### Jump feels too heavy
- ✅ Decrease gravity (150 → 120)
- ✅ Increase jump_force (25 → 28)
- ✅ Increase variable jump multiplier (0.5 → 0.7)

### Can't jump consistently
- ✅ Check ground detection threshold (-0.7 → -0.5)
- ✅ Add coyote time (0.1s)
- ✅ Add jump buffer (0.1s)
- ✅ Enable CCD

### Movement feels sluggish
- ✅ Increase move_speed (3 → 5)
- ✅ Remove acceleration (instant response)
- ✅ Increase friction (0.8 → 0.5)

### Dash feels weak
- ✅ Increase dash_speed (10 → 15)
- ✅ Increase dash_duration (0.2 → 0.3)
- ✅ Disable gravity during dash

## 📚 Resources

- **Celeste Movement Analysis**: https://maddythorson.medium.com/celeste-and-towerfall-physics-d24bd2ae0fc5
- **Platformer Controls**: https://www.youtube.com/watch?v=yorTG9at90g
- **Game Feel**: https://www.youtube.com/watch?v=216_5nu4aVQ

---

**Happy tuning!** 🎮✨
