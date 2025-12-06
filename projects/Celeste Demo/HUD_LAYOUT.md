# Celeste Demo - HUD Layout Reference

Visual reference for HUD element positions.

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Health Bar ████████░░] (20, 20)                    (X: 2.7 Y: -9.8)│
│ [Stamina ██████████░░] (20, 50)                     (VX: 0.0 VY: 0.0)│
│ Dash: 1 (20, 75)                                         FPS: 60     │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                          [DASHING!]                                  │
│                         (Center, -100)                               │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│                                                                       │
│ [GROUNDED] (20, -30)                                                 │
│ [WALL SLIDE] (20, -55)                                               │
│                                                                       │
│              WASD: Move | Space: Jump | Shift: Dash                  │
│                        (Bottom Center, -15)                          │
└─────────────────────────────────────────────────────────────────────┘
```

## Element Positions

### Top-Left Corner
| Element | Position | Size | Anchor |
|---------|----------|------|--------|
| Health Bar | (20, 20) | 180x24 | TopLeft |
| Stamina Bar | (20, 50) | 180x16 | TopLeft |
| Dash Indicator | (20, 75) | 120x30 | TopLeft |

### Top-Right Corner
| Element | Position | Size | Anchor |
|---------|----------|------|--------|
| Position Debug | (-180, 10) | 170x25 | TopRight |
| Velocity Debug | (-180, 35) | 170x25 | TopRight |
| FPS Counter | (-100, 60) | 90x30 | TopRight |

### Bottom-Left Corner
| Element | Position | Size | Anchor |
|---------|----------|------|--------|
| Grounded Indicator | (20, -30) | 100x25 | BottomLeft |
| Wall Slide Indicator | (20, -55) | 100x25 | BottomLeft |

### Center
| Element | Position | Size | Anchor |
|---------|----------|------|--------|
| Dashing Indicator | (-50, -100) | 100x35 | Center |

### Bottom-Center
| Element | Position | Size | Anchor |
|---------|----------|------|--------|
| Controls Hint | (-180, -15) | 360x25 | BottomCenter |

## Color Scheme

### Health Bar
- **Foreground**: RGB(0.2, 1.0, 0.3) - Bright Green
- **Background**: RGB(0.15, 0.15, 0.15) - Dark Gray
- **Alpha**: 0.9

### Stamina Bar
- **Foreground**: RGB(1.0, 0.8, 0.2) - Yellow/Gold
- **Background**: RGB(0.15, 0.15, 0.15) - Dark Gray
- **Alpha**: 0.9

### Dash Indicator
- **Color**: RGB(0.3, 0.8, 1.0) - Cyan/Blue
- **Font Size**: 18px

### Debug Info
- **Color**: RGB(0.7, 0.7, 0.7) - Light Gray
- **Alpha**: 0.8
- **Font Size**: 14px

### FPS Counter
- **Color**: RGB(0.5, 1.0, 0.5) - Light Green
- **Font Size**: 16px

### State Indicators
- **Grounded**: RGB(0.3, 1.0, 0.3) - Green
- **Wall Slide**: RGB(0.3, 0.6, 1.0) - Blue
- **Dashing**: RGB(1.0, 0.3, 0.3) - Red (20px)

### Controls Hint
- **Color**: RGB(0.8, 0.8, 0.8) - Light Gray
- **Alpha**: 0.7
- **Font Size**: 14px

## Visibility States

### Always Visible
- ✅ Health Bar
- ✅ Stamina Bar
- ✅ Dash Indicator
- ✅ Position Debug
- ✅ Velocity Debug
- ✅ FPS Counter
- ✅ Controls Hint

### Conditional Visibility
- 🔄 Grounded Indicator (when `is_grounded = true`)
- 🔄 Wall Slide Indicator (when `is_touching_wall = true`)
- 🔄 Dashing Indicator (when `is_dashing = true`)

## Responsive Layout

All elements use anchor-based positioning, so they automatically adjust to different screen sizes:

### 1920x1080 (Full HD)
- Health Bar: (20, 20) from top-left
- FPS Counter: (1820, 60) absolute position

### 1280x720 (HD)
- Health Bar: (20, 20) from top-left
- FPS Counter: (1180, 60) absolute position

### 800x600 (Small)
- Health Bar: (20, 20) from top-left
- FPS Counter: (700, 60) absolute position

## World-Space UI (Not in HUD Asset)

These are spawned dynamically in the game world:

### Damage Numbers
- **Position**: Above entity (offset Y +1.0)
- **Lifetime**: 1.0 second
- **Animation**: Float upward
- **Color**: Red for damage, Green for healing

### Enemy Health Bars
- **Position**: Above enemy (offset Y +0.5)
- **Size**: 50x8 pixels
- **Color**: Red (health), Dark gray (background)

### Interaction Prompts
- **Position**: Above object (offset Y +0.3)
- **Text**: "Press E to interact"
- **Color**: White with slight transparency

## Customization Tips

### Make HUD Smaller
```json
"size": [150.0, 20.0]  // Reduce from 180x24
```

### Move to Different Corner
```json
"anchor": "TopRight",
"offset": [-200.0, 20.0]  // Negative X for right side
```

### Change Transparency
```json
"color": [1.0, 0.2, 0.2, 0.5]  // Last value is alpha (0.5 = 50%)
```

### Hide Debug Info
```json
"visible": false  // In position_debug, velocity_debug, fps_counter
```

## Performance Impact

| Element Type | Cost | Notes |
|--------------|------|-------|
| Progress Bar | Low | Simple rect drawing |
| Text | Low | egui text rendering |
| Dynamic Text | Low | String formatting per frame |
| Conditional Visibility | Minimal | Just a boolean check |

**Total HUD Cost**: < 0.1ms per frame (< 1% of 16.67ms budget @ 60 FPS)
