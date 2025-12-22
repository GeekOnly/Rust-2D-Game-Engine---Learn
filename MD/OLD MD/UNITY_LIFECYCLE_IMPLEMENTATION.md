# Unity-Style Script Lifecycle Implementation

## Overview
Successfully implemented Unity-style script lifecycle management with proper Awake → Start → Update execution order.

## Implementation Date
November 30, 2025

## Changes Made

### 1. Script Engine Architecture (script/src/lib.rs)

**Problem**: Previously used a single global Lua state for all entities, causing lifecycle functions to be called globally instead of per-entity.

**Solution**: Implemented per-entity Lua states using a HashMap:

```rust
pub struct ScriptEngine {
    lua: Lua,  // Keep for backward compatibility
    entity_states: HashMap<Entity, Lua>,  // Per-entity Lua states
}
```

### 2. New API Methods

#### `load_script_for_entity()`
- Creates a separate Lua state for each entity
- Loads the script content
- Injects script parameters as globals
- Calls `Awake()` immediately (Unity-style)
- Stores the Lua state in the entity_states HashMap

#### `call_start_for_entity()`
- Called after all `Awake()` calls are complete
- Calls `Start()` for a specific entity
- Follows Unity's execution order: all Awake() → all Start()

#### `remove_entity_state()`
- Cleans up entity's Lua state when entity is destroyed
- Prevents memory leaks

### 3. Updated Script Execution

**run_script()**: Now retrieves the entity's specific Lua state and calls `Update(dt)` or `on_update(entity, dt)` for backward compatibility.

**call_collision()**: Now uses the entity's specific Lua state for collision callbacks.

### 4. Main Game Loop Updates (engine/src/main.rs)

When entering play mode:

```rust
// Phase 1: Load scripts and call Awake() for all entities
for entity in &entities_with_scripts {
    script_engine.load_script_for_entity(*entity, &content, &world)?;
    // Awake() is called inside load_script_for_entity
}

// Phase 2: Call Start() for all entities (after all Awake() calls)
for entity in &entities_with_scripts {
    script_engine.call_start_for_entity(*entity)?;
}
```

During play mode:
- `Update(dt)` is called every frame for each entity's script
- Physics runs at fixed timestep (60 Hz)
- Collision callbacks trigger `OnCollisionEnter(other)`

### 5. Player Controller Script Updates

Updated to use Unity-style lifecycle:

```lua
-- Unity-style lifecycle: Awake is called when script is loaded
function Awake()
    print("Player Controller: Awake() called")
end

-- Unity-style lifecycle: Start is called before first Update
function Start()
    print("Player Controller: Start() called")
    set_velocity(0.0, 0.0)
    set_gravity_scale(gravity_scale)
end

-- Unity-style lifecycle: Update is called every frame
function Update(dt)
    -- Game logic here
end

-- Unity-style collision callback
function OnCollisionEnter(other)
    print("Collision detected with entity: " .. tostring(other))
    is_grounded = true
end
```

## Unity Lifecycle Order Implemented

✅ **Awake()** - Called when script is loaded (once per entity)
✅ **Start()** - Called before first Update (once per entity, after all Awake calls)
✅ **Update(dt)** - Called every frame
✅ **OnCollisionEnter(other)** - Called when collision detected

## Not Yet Implemented

⏳ **FixedUpdate()** - For physics calculations (would run at fixed timestep)
⏳ **LateUpdate()** - For camera follow and post-update logic
⏳ **OnEnable/OnDisable** - For component enable/disable
⏳ **OnDestroy** - For cleanup when entity is destroyed
⏳ **OnCollisionStay/OnCollisionExit** - For continuous collision tracking

## Testing Results

✅ Jump functionality working correctly
✅ Grounded detection working
✅ Velocity setting and getting working
✅ Per-entity script state isolation working
✅ No interference between entity scripts

## Benefits

1. **Proper Initialization Order**: Awake → Start → Update matches Unity's behavior
2. **Entity Isolation**: Each entity has its own Lua state, preventing variable conflicts
3. **Unity Familiarity**: Developers familiar with Unity can use the same patterns
4. **Backward Compatibility**: Still supports old `on_start()` and `on_update()` functions
5. **Clean Architecture**: Clear separation of concerns with per-entity states

## Performance Considerations

- Each entity creates its own Lua state (small memory overhead)
- Lua states are lightweight (~few KB each)
- Proper cleanup when entities are destroyed prevents memory leaks
- Trade-off: Slightly more memory for much better isolation and correctness

## Future Enhancements

1. Implement FixedUpdate() for physics-based movement
2. Add LateUpdate() for camera systems
3. Implement OnDestroy() for cleanup
4. Add OnCollisionStay() and OnCollisionExit()
5. Consider Lua state pooling for frequently spawned/destroyed entities
6. Add OnEnable/OnDisable for component toggling

## Migration Guide

### Old Style (Deprecated but still works)
```lua
function on_start(entity)
    -- Initialization
end

function on_update(entity, dt)
    -- Update logic
end

function on_collision(entity, other)
    -- Collision handling
end
```

### New Style (Recommended)
```lua
function Awake()
    -- Early initialization (called first)
end

function Start()
    -- Initialization (called after all Awake)
end

function Update(dt)
    -- Update logic (called every frame)
end

function OnCollisionEnter(other)
    -- Collision handling
end
```

## Conclusion

The Unity-style script lifecycle is now fully functional with proper Awake → Start → Update execution order. Each entity has its own isolated Lua state, preventing conflicts and enabling proper per-entity initialization. The system maintains backward compatibility while providing a familiar API for Unity developers.
