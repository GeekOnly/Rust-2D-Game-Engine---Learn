# Game View Resolution Settings

Game View now supports fixed resolutions for different platforms!

## 🎮 Features

### Resolution Presets

**PC (16:9)**
- Full HD: 1920x1080
- HD: 1280x720
- WXGA: 1366x768
- QHD: 2560x1440
- 4K UHD: 3840x2160

**Mobile (Portrait)**
- iPhone 14: 1170x2532
- iPhone 14 Pro: 1179x2556
- iPhone SE: 750x1334
- Pixel 7: 1080x2400
- Galaxy S23: 1080x2340

**Mobile (Landscape)**
- iPhone 14 Landscape: 2532x1170
- iPhone 14 Pro Landscape: 2556x1179
- Pixel 7 Landscape: 2400x1080

**Tablet**
- iPad Pro: 2048x2732
- iPad Air: 1640x2360

**Free Mode**
- Fit to Window (default)

## 🚀 How to Use

### 1. Open Game View

- Switch to **Game** tab (not Scene tab)
- You'll see a toolbar at the top

### 2. Select Resolution

- Click the **Resolution** dropdown
- Choose your target platform:
  - PC: Full HD (1920x1080) recommended
  - Mobile Portrait: iPhone 14 or Pixel 7
  - Mobile Landscape: iPhone 14 Landscape
  - Tablet: iPad Pro

### 3. Adjust Scale

- Use the **Scale** slider to fit the view
- 100% = Full resolution
- 50% = Half size (easier to see on small screens)
- 25% = Quarter size

### 4. Enable Guides

- **Safe Area**: Shows 5% margin for UI elements
- **Info**: Shows resolution and scale info

## 📐 Resolution Display

### Fixed Resolution Mode

When you select a specific resolution:
- Game view is **centered** in the panel
- **Aspect ratio is preserved**
- **Black bars** appear if needed
- **Border** shows the game area
- **Resolution info** displays in top-left

### Free Mode

When "Free (Fit to Window)" is selected:
- Game view **fills the entire panel**
- No fixed aspect ratio
- No borders or info overlay

## 🎨 Visual Guides

### Resolution Info Overlay

Shows in top-left corner:
```
Full HD (1920x1080)
1920x1080 (100%)
```

### Safe Area Guide

Green border showing safe area for UI:
- 5% margin from edges
- Important UI should stay inside
- Prevents cutoff on notched screens

### Game View Border

Gray border around game area:
- Shows exact game boundaries
- Helps visualize final output

## 💡 Use Cases

### PC Game Development
```
Resolution: Full HD (1920x1080)
Scale: 100%
Safe Area: Off
```

### Mobile Game (Portrait)
```
Resolution: iPhone 14 (1170x2532)
Scale: 50% (easier to see)
Safe Area: On (important!)
```

### Mobile Game (Landscape)
```
Resolution: iPhone 14 Landscape (2532x1170)
Scale: 75%
Safe Area: On
```

### Tablet Game
```
Resolution: iPad Pro (2048x2732)
Scale: 50%
Safe Area: On
```

### Testing Multiple Resolutions

1. Design UI in Full HD
2. Test in iPhone 14 Portrait
3. Test in iPhone 14 Landscape
4. Verify safe areas work
5. Adjust HUD positions if needed

## 🔧 Technical Details

### Aspect Ratio Preservation

Game view maintains aspect ratio:
- Fits to **width** if panel is tall
- Fits to **height** if panel is wide
- Always **centered** in available space

### Scaling

Scale affects render size:
- 100% = Native resolution
- 50% = Half resolution (faster)
- 25% = Quarter resolution (preview)

### HUD Rendering

HUD adapts to game view size:
- Anchor positions scale correctly
- Text remains readable
- Elements stay in correct positions

## 📊 Performance

### Resolution Impact

| Resolution | Pixels | Performance |
|------------|--------|-------------|
| HD (720p) | 921,600 | Fast |
| Full HD (1080p) | 2,073,600 | Good |
| QHD (1440p) | 3,686,400 | Medium |
| 4K (2160p) | 8,294,400 | Slow |

**Tip**: Use lower scale for better performance during development.

### Scale Impact

- 100% scale = Full quality, slower
- 50% scale = Good quality, faster
- 25% scale = Preview quality, fastest

## 🎯 Best Practices

### For PC Games
1. Design at Full HD (1920x1080)
2. Test at HD (1280x720)
3. Verify at 4K (3840x2160)

### For Mobile Games
1. Design at iPhone 14 (1170x2532)
2. Test landscape mode
3. Enable safe area guides
4. Check on different aspect ratios

### For Cross-Platform
1. Use Free mode for initial design
2. Test all target resolutions
3. Adjust HUD for each platform
4. Use safe areas consistently

## 🐛 Troubleshooting

### Game View Too Small?
- Increase **Scale** slider
- Or use **Free** mode

### Game View Too Large?
- Decrease **Scale** slider
- Or select smaller resolution

### HUD Cut Off?
- Enable **Safe Area** guide
- Move HUD elements inside safe area
- Adjust anchor positions

### Wrong Aspect Ratio?
- Check selected resolution
- Verify it matches your target platform
- Use Free mode if unsure

## 📖 Related Documentation

- [HUD System Guide](../../MD/HUD_SYSTEM_GUIDE.md)
- [HUD Layout Reference](HUD_LAYOUT.md)
- [Testing HUD](TEST_HUD.md)

---

**Now you can design for any platform! 🎮📱💻**
