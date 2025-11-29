# Celeste Demo - Platformer Movement

Demo project showcasing Celeste-style platformer movement mechanics.

## Features

- **Run**: A/D or Arrow Keys
- **Jump**: Space (when grounded)
- **Dash**: Left Shift (8-directional)
  - Can dash in any direction (WASD + Shift)
  - Dash recharges when touching ground
  - Default dash direction is right if no input

## Scene Layout

- **Player** (Blue square) - Controllable character
- **Ground** (Gray platform) - Main floor
- **Platforms** - Jump targets
- **Walls** - Side boundaries

## Controls

### Movement
- `A` / `Left Arrow` - Move Left
- `D` / `Right Arrow` - Move Right
- `Space` - Jump
- `Left Shift` - Dash

### Dash Directions
- `Shift` - Dash right (default)
- `W + Shift` - Dash up
- `S + Shift` - Dash down
- `A + Shift` - Dash left
- `D + Shift` - Dash right
- Diagonal dashes: `W+A+Shift`, `W+D+Shift`, etc.

## How to Play

1. Open the project in the editor
2. Load `scenes/main.json`
3. Press Play (Ctrl+P)
4. Use WASD/Arrows to move
5. Space to jump
6. Shift to dash

## Script: player_controller.lua

The player controller implements:
- Smooth horizontal movement with deceleration
- Jump mechanics (only when grounded)
- 8-directional dash system
- Dash cooldown (recharges on ground)
- Physics-based movement

## Future Enhancements

Potential additions:
- Wall slide
- Wall jump
- Coyote time (jump grace period)
- Jump buffering
- Variable jump height
- Stamina system
- Climbing mechanics

## Tips

- Dash can be used in mid-air
- Combine jump + dash for maximum distance
- Dash recharges when you touch the ground
- Experiment with diagonal dashes for precise movement
