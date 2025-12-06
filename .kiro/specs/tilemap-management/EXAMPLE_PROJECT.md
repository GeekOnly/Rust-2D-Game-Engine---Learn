# Example Project: Platformer Level

This guide walks you through creating a simple platformer level using the Tilemap Management System.

## Prerequisites

- LDtk editor installed (https://ldtk.io/)
- Game engine with tilemap management system
- Basic understanding of LDtk

## Step 1: Create LDtk Project

1. Open LDtk
2. Create new project: `File > New Project`
3. Save as `platformer_level.ldtk` in your project directory

## Step 2: Configure Project Settings

In LDtk:

1. Set grid size to 8x8 pixels
2. Create tileset:
   - `Project > Tilesets > Add Tileset`
   - Select your tileset image (e.g., `platformer_tiles.png`)
   - Set tile size to 8x8

## Step 3: Create Layers

Create these layers in order (bottom to top):

### Background Layer (Tiles)
- Type: Tiles
- Grid size: 8x8
- Tileset: platformer_tiles
- Purpose: Decorative background

### Collision Layer (IntGrid)
- Type: IntGrid
- Grid size: 8x8
- IntGrid values:
  - 0: Empty (no collision)
  - 1: Solid (generates collider)
- Purpose: Collision detection

### Foreground Layer (Tiles)
- Type: Tiles
- Grid size: 8x8
- Tileset: platformer_tiles
- Purpose: Foreground decoration

### Entities Layer (Entities)
- Type: Entities
- Purpose: Spawn points, items, enemies

## Step 4: Paint Your Level

### Paint Collision Layer

1. Select Collision Layer
2. Select IntGrid value 1 (Solid)
3. Paint platforms and walls
4. Leave gaps for player movement

Example layout:
```
0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0
0 0 1 1 1 0 0 0 0 0
0 0 0 0 0 0 1 1 1 0
1 1 1 1 1 1 1 1 1 1
```

### Paint Tile Layers

1. Select Background Layer
2. Paint background tiles
3. Select Foreground Layer
4. Paint foreground decoration

### Add Entities

1. Select Entities Layer
2. Add player spawn point
3. Add items, enemies, etc.

## Step 5: Save and Export

1. Save in LDtk: `Ctrl+S`
2. LDtk automatically saves to .ldtk file
3. No manual export needed!

## Step 6: Load in Game Engine

### Method 1: Auto-Load

1. Open game engine
2. Open Maps Panel (🗺️)
3. Click "Refresh"
4. Click on `platformer_level.ldtk`
5. Map loads with auto-generated colliders!

### Method 2: Manual Load

1. Open Maps Panel
2. Click "Add Map"
3. Browse to `platformer_level.ldtk`
4. Click "Open"

## Step 7: Verify Colliders

1. Check Maps Panel
2. Expand the loaded map
3. Verify colliders were generated
4. Should see: "Colliders: X" (where X is the number of composite colliders)

Example:
- 100 solid tiles → ~15 composite colliders (85% reduction!)

## Step 8: Adjust Layer Properties

### Hide Background Layer

1. Open Layer Properties Panel (🎨)
2. Select Background layer
3. Uncheck "Visible"
4. Or press `Ctrl+H`

### Adjust Layer Z-Order

1. Open Layer Ordering Panel (📑)
2. Select `platformer_level.ldtk` from dropdown
3. Drag layers to reorder
4. Verify rendering order in Scene View

### Adjust Layer Transform

1. Select a layer
2. Open Layer Properties Panel
3. Adjust position, rotation, or scale
4. Changes apply immediately

## Step 9: Test Hot-Reload

1. Keep map loaded in game engine
2. Switch to LDtk
3. Make changes to the level
4. Save in LDtk (`Ctrl+S`)
5. Switch back to game engine
6. Map automatically reloads!
7. Layer visibility preserved
8. Colliders regenerated

## Step 10: Monitor Performance

1. Open Performance Panel (📊)
2. Check metrics:
   - Draw Calls: Should be low (< 100)
   - Triangles: Depends on level size
   - Memory: Should be reasonable (< 50MB for medium level)
3. If warnings appear:
   - Use composite colliders (already default)
   - Hide unused layers
   - Optimize tileset size

## Advanced Techniques

### Multiple Collision Values

In LDtk, use different IntGrid values for different collision types:

- 0: Empty
- 1: Solid (generates collider)
- 2: One-way platform (custom logic)
- 3: Hazard (custom logic)

Configure in Collider Settings:
- Set collision value to 1 for solid platforms
- Implement custom logic for values 2 and 3

### Layer Effects

1. Select a layer
2. Open Layer Properties Panel
3. Adjust:
   - Opacity: For transparency effects
   - Color Tint: For atmospheric effects
   - Z-Order: For parallax scrolling

### Multiple Levels

1. Create multiple .ldtk files:
   - `level_01.ldtk`
   - `level_02.ldtk`
   - `level_03.ldtk`
2. Load all in Maps Panel
3. Toggle visibility to switch between levels
4. Or unload/load as needed

## Keyboard Shortcuts Reference

While working on your level:

- `Ctrl+R`: Reload current map
- `Ctrl+Shift+R`: Regenerate colliders
- `Ctrl+H`: Toggle layer visibility
- `Ctrl+S`: Save scene (in game engine)

## Troubleshooting

### Colliders Not Generating

**Problem**: No colliders appear

**Solution**:
1. Check IntGrid layer has tiles with value 1
2. Verify collision value in Collider Settings is 1
3. Click "Regenerate Colliders" manually

### Hot-Reload Not Working

**Problem**: Changes don't reload automatically

**Solution**:
1. Check Collider Settings → Auto-regenerate is enabled
2. Verify file is saved in LDtk
3. Try manual reload with `Ctrl+R`

### Performance Issues

**Problem**: Low frame rate

**Solution**:
1. Check Performance Panel
2. Unload unused maps
3. Hide unused layers
4. Use composite colliders (default)

## Example Project Files

The example project includes:

```
platformer_level/
├── platformer_level.ldtk          # Main level file
├── platformer_tiles.png           # Tileset image
└── README.md                      # This file
```

## Next Steps

1. Create more complex levels
2. Add multiple layers for parallax
3. Use different collision values for gameplay mechanics
4. Experiment with layer properties
5. Build a complete game!

## Resources

- LDtk Documentation: https://ldtk.io/docs/
- Tilemap Management User Guide: `USER_GUIDE.md`
- API Documentation: `API_DOCUMENTATION.md`

## Conclusion

You now have a complete platformer level with:
- ✅ Collision detection (auto-generated)
- ✅ Multiple layers (background, collision, foreground)
- ✅ Hot-reload workflow
- ✅ Performance monitoring

Happy level designing! 🎮
