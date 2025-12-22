# Unity Lifecycle Implementation - Quick Plan

## Current Status

✅ Created specification document
✅ Added ScriptLifecycleState to Script struct
⏳ Need to implement actual lifecycle calls

## Quick Fix for Jump Issue

**Problem:** Jump doesn't work because velocity is set in Update() but physics runs first and overrides it.

**Solution:** Add FixedUpdate() callback that runs BEFORE physics.

## Minimal Implementation Steps

### Step 1: Add FixedUpdate support to ScriptEngine

```rust
// In script/src/lib.rs

impl ScriptEngine {
    // Add fixed timestep accumulator
    fixed_accumulator: f32,
    
    pub fn update_with_fixed(&mut self, entity: Entity, world: &mut World, input: &InputSystem, dt: f32) {
        // Accumulate time
        self.fixed_accumulator += dt;
        
        // Run FixedUpdate multiple times if needed
        while self.fixed_accumulator >= 0.02 {
            self.call_fixed_update(entity, world, input);
            self.fixed_accumulator -= 0.02;
        }
        
        // Run normal Update
        self.run_script(entity, world, input, dt);
    }
    
    fn call_fixed_update(&self, entity: Entity, world: &mut World, input: &InputSystem) {
        // Try FixedUpdate() first, then on_update() for compatibility
        let globals = self.lua.globals();
        if let Ok(func) = globals.get::<_, Function>("FixedUpdate") {
            // Call with full context
            func.call(());
        }
    }
}
```

### Step 2: Update game loop order

```rust
// In engine/src/main.rs or wherever game loop is

// 1. Run FixedUpdate (scripts modify velocity)
for (entity, engine) in &mut script_engines {
    engine.call_fixed_update(entity, world, input);
}

// 2. Run Physics (uses velocity from scripts)
physics_world.update(dt, world);

// 3. Run Update (normal per-frame logic)
for (entity, engine) in &mut script_engines {
    engine.call_update(entity, world, input, dt);
}
```

### Step 3: Update player_controller.lua

```lua
-- Move physics code to FixedUpdate
function FixedUpdate()
    -- Get input
    local input_x = 0
    if is_key_down("A") then input_x = -1 end
    if is_key_down("D") then input_x = 1 end
    
    -- Set velocity (will be used by physics)
    local vel = get_velocity()
    if vel then
        velocity_x = input_x * move_speed
        velocity_y = vel.y
    end
    
    -- Jump
    if is_key_just_pressed("Space") then
        velocity_y = -jump_force
        print("Jump!")
    end
    
    -- Apply velocity
    set_velocity(velocity_x, velocity_y)
end

-- Keep non-physics logic in Update
function Update(dt)
    -- Animation, visual effects, etc.
end
```

## Full Implementation (Future)

For complete Unity-style lifecycle, implement:

1. **Awake()** - Called when script loads
2. **Start()** - Called before first frame
3. **FixedUpdate()** - Fixed timestep (0.02s)
4. **Update(dt)** - Per frame
5. **LateUpdate(dt)** - After all Update
6. **OnCollisionEnter/Stay/Exit(other)** - Collision callbacks
7. **OnDestroy()** - Cleanup

## Benefits

- ✅ Fixes jump issue immediately
- ✅ Familiar to Unity developers
- ✅ Better physics integration
- ✅ Backward compatible (old scripts still work)

## Next Actions

1. Implement FixedUpdate in ScriptEngine
2. Update game loop to call FixedUpdate before physics
3. Convert player_controller.lua to use FixedUpdate
4. Test jump functionality
5. Gradually add other lifecycle callbacks

## Estimated Time

- Quick fix (FixedUpdate only): 30-60 minutes
- Full implementation: 2-4 hours
