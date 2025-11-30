# Jump System Analysis & Issues

## Current Problems

### Issue 1: Inconsistent Jump Behavior
- Jump force < 44: Player doesn't jump at all
- Jump force > 44: Player jumps too high and disappears
- Very narrow working range (44-45)

### Issue 2: Root Cause - Timing Problem

**Game Loop Order (Current):**
```
1. Scripts run → set velocity_y = -jump_force
2. Physics applies gravity → velocity_y -= 300 * dt
3. Physics updates position → position.y += velocity_y * dt
4. Collision resolution → if (colliding && moving_into) { velocity = 0 }
```

**The Problem:**
- Player is ALWAYS colliding with ground when grounded
- When jump is triggered, velocity is set but player hasn't moved yet
- Collision resolution sees: "player colliding + velocity pointing away" → should allow
- BUT: After gravity is applied, velocity might become positive (down) → collision stops it

**Math:**
```
Gravity per frame = 300 * 0.016 = 4.8 units/frame
Jump force needed = Must overcome gravity + separate from collision

If jump_force = 20:
  Frame 1: velocity = -20 (up)
  After gravity: velocity = -20 + 4.8 = -15.2 (still up) ✓
  
If jump_force = 10:
  Frame 1: velocity = -10 (up)
  After gravity: velocity = -10 + 4.8 = -5.2 (still up) ✓
  But collision resolution might reset it because overlap is too small
```

## Solutions Attempted

### ❌ Solution 1: Increase Jump Force
- Problem: Makes jump too high
- Why it fails: Doesn't address root cause

### ❌ Solution 2: Change Execution Order
- Scripts run before physics ✓
- Problem: Collision resolution still runs after and resets velocity

### ❌ Solution 3: Fix Collision Resolution Logic
- Added check to not reset velocity when moving away
- Problem: Player still touching ground, so "moving away" detection fails

### ❌ Solution 4: Add Extra Jump Boost
- `velocity_y = -jump_force - 5.0`
- Problem: Inconsistent, doesn't solve timing issue

## Recommended Solutions

### ✅ Solution A: Separate Player from Ground Before Jump (Best)

**In Script:**
```lua
function handle_jump()
    if is_key_just_pressed("Space") and is_grounded then
        -- Move player up slightly FIRST
        local pos = get_position()
        set_position(pos.x, pos.y - 0.1, pos.z)  -- Move up 0.1 units
        
        -- THEN apply jump velocity
        velocity_y = -jump_force
        is_grounded = false
    end
end
```

**Problem:** `set_position()` doesn't exist in Lua API!

### ✅ Solution B: Add "Just Jumped" Flag to Physics

**In Physics System:**
```rust
pub struct Rigidbody {
    pub velocity: (f32, f32),
    pub just_jumped: bool,  // NEW: Skip collision for 1 frame
    // ...
}

// In collision resolution:
if !rb.just_jumped && direction < 0.0 && rb.velocity.1 > 0.0 {
    rb.velocity.1 = 0.0;
}

// Clear flag after physics step
rb.just_jumped = false;
```

**In Lua API:**
```lua
set_just_jumped(true)  -- NEW function
```

### ✅ Solution C: Use Impulse Instead of Velocity

**Current:** `set_velocity()` - overwrites velocity
**Better:** `add_impulse()` - adds to velocity

```lua
function handle_jump()
    if is_key_just_pressed("Space") and is_grounded then
        add_impulse(0, -jump_force)  -- Add upward impulse
        is_grounded = false
    end
end
```

### ✅ Solution D: Reduce Gravity (Simplest)

**Current:** gravity = 300
**Better:** gravity = 100-150

This makes jump force requirements less strict.

## Immediate Fix

Let me implement Solution D (reduce gravity) + add proper Lua API functions:

1. Reduce gravity to 150
2. Set jump_force to 12-15
3. Add `set_position()` to Lua API
4. Test and iterate

## Long-term Fix

Implement proper physics system with:
1. Impulse-based forces
2. Proper collision callbacks
3. Coyote time (grace period for jumping after leaving ground)
4. Jump buffering (remember jump input for a few frames)
