# Unity-Style Script Lifecycle

## Overview

Implement Unity-style script lifecycle callbacks to make the engine more familiar to Unity developers and provide better control over script execution order.

## Current System

**ปัจจุบัน:**
- `on_start(entity)` - เรียกครั้งเดียวเมื่อเริ่มต้น
- `on_update(entity, dt)` - เรียกทุก frame
- `on_collision(entity, other)` - เรียกเมื่อเกิด collision

**ปัญหา:**
- ไม่มี FixedUpdate สำหรับ physics
- ไม่มี LateUpdate สำหรับ camera
- ไม่มี lifecycle callbacks อื่นๆ
- Physics และ script update ไม่ sync กัน

## Proposed Unity-Style Lifecycle

### Initialization Phase

```lua
function Awake()
    -- Called when script is first loaded
    -- Use for initialization that doesn't depend on other scripts
    -- Called BEFORE Start()
end

function OnEnable()
    -- Called when script/entity is enabled
    -- Can be called multiple times if entity is disabled/enabled
end

function Start()
    -- Called before first frame update
    -- Use for initialization that depends on other scripts
    -- Called AFTER Awake() and OnEnable()
end
```

### Update Phase

```lua
function FixedUpdate()
    -- Called every fixed timestep (default: 50 times per second)
    -- Use for physics calculations and rigidbody manipulation
    -- Guaranteed consistent timing
    -- Called BEFORE physics simulation
end

function Update()
    -- Called every frame (variable timestep)
    -- Use for input handling, non-physics movement
    -- Called AFTER physics simulation
end

function LateUpdate()
    -- Called after all Update() calls
    -- Use for camera following, final adjustments
    -- Guaranteed to run after all Update() calls
end
```

### Cleanup Phase

```lua
function OnDisable()
    -- Called when script/entity is disabled
end

function OnDestroy()
    -- Called when entity is destroyed
    -- Use for cleanup
end
```

### Physics Callbacks

```lua
function OnCollisionEnter(other)
    -- Called when collision starts
end

function OnCollisionStay(other)
    -- Called every frame while colliding
end

function OnCollisionExit(other)
    -- Called when collision ends
end
```

## Execution Order

```
=== Initialization ===
1. Awake()           (all scripts)
2. OnEnable()        (all scripts)
3. Start()           (all scripts)

=== Game Loop ===
Every Fixed Timestep (0.02s):
4. FixedUpdate()     (all scripts)
5. Physics Simulation
6. Collision Detection
7. OnCollisionEnter/Stay/Exit()

Every Frame:
8. Update()          (all scripts)
9. LateUpdate()      (all scripts)
10. Render

=== Cleanup ===
11. OnDisable()
12. OnDestroy()
```

## Implementation Plan

### Phase 1: Core Lifecycle

1. **Update script/src/lib.rs:**
   - Add lifecycle tracking per script
   - Implement Awake, Start, OnEnable
   - Implement FixedUpdate, Update, LateUpdate
   - Add fixed timestep accumulator

2. **Update engine game loop:**
   - Separate fixed update from variable update
   - Call FixedUpdate before physics
   - Call Update after physics
   - Call LateUpdate last

### Phase 2: Physics Callbacks

3. **Enhance collision detection:**
   - Track collision state (enter/stay/exit)
   - Call appropriate callbacks
   - Pass collision info to scripts

### Phase 3: Cleanup

4. **Add cleanup callbacks:**
   - OnDisable when entity disabled
   - OnDestroy when entity destroyed

## Example: Player Controller with Unity Lifecycle

```lua
-- Player Controller (Unity-style)

local move_speed = 5.0
local jump_force = 10.0
local velocity = {x = 0, y = 0}

function Awake()
    print("Player Awake")
    -- Initialize variables
end

function Start()
    print("Player Start")
    -- Get references to other components
    set_gravity_scale(0.5)
end

function FixedUpdate()
    -- Physics calculations (runs at fixed 50 FPS)
    -- This is where we should modify velocity!
    
    -- Get input
    local input_x = 0
    if is_key_down("A") then input_x = -1 end
    if is_key_down("D") then input_x = 1 end
    
    -- Set velocity
    velocity.x = input_x * move_speed
    
    -- Jump
    if is_key_just_pressed("Space") then
        velocity.y = -jump_force
    end
    
    -- Apply velocity (will be used by physics)
    set_velocity(velocity.x, velocity.y)
end

function Update()
    -- Non-physics updates (runs every frame)
    -- Animation, visual effects, etc.
end

function LateUpdate()
    -- Camera follow, final adjustments
end

function OnCollisionEnter(other)
    print("Collision with: " .. get_name(other))
end
```

## Benefits

### For Unity Developers
- **Familiar API** - Same lifecycle as Unity
- **Easy migration** - Copy-paste Unity scripts with minimal changes
- **Predictable behavior** - Same execution order

### For Physics
- **FixedUpdate for physics** - Consistent timestep
- **Proper execution order** - Physics runs after FixedUpdate
- **No more velocity override** - Scripts modify velocity before physics

### For Gameplay
- **LateUpdate for camera** - Smooth camera follow
- **Collision callbacks** - Better collision handling
- **Cleanup callbacks** - Proper resource management

## Migration Guide

### Old System → New System

```lua
-- OLD
function on_start(entity)
    -- initialization
end

function on_update(entity, dt)
    -- everything
end

-- NEW
function Start()
    -- initialization
end

function FixedUpdate()
    -- physics, velocity changes
end

function Update()
    -- input, non-physics
end

function LateUpdate()
    -- camera, final adjustments
end
```

## Technical Details

### Fixed Timestep

```rust
const FIXED_TIMESTEP: f32 = 0.02; // 50 FPS

struct ScriptSystem {
    accumulator: f32,
}

impl ScriptSystem {
    fn update(&mut self, dt: f32, world: &mut World) {
        // Accumulate time
        self.accumulator += dt;
        
        // Run FixedUpdate multiple times if needed
        while self.accumulator >= FIXED_TIMESTEP {
            self.run_fixed_update(world);
            self.accumulator -= FIXED_TIMESTEP;
        }
        
        // Run Update once per frame
        self.run_update(dt, world);
        
        // Run LateUpdate once per frame
        self.run_late_update(dt, world);
    }
}
```

### Script State Tracking

```rust
struct ScriptState {
    awake_called: bool,
    start_called: bool,
    enabled: bool,
}
```

## Compatibility

- **Backward compatible** - Old scripts still work
- **Gradual migration** - Can mix old and new style
- **Optional** - Scripts can use only the callbacks they need

## Next Steps

1. Create spec document ✅
2. Implement core lifecycle (Awake, Start, FixedUpdate, Update, LateUpdate)
3. Update player_controller.lua to use new lifecycle
4. Test jump with FixedUpdate
5. Implement collision callbacks
6. Implement cleanup callbacks
7. Update documentation
