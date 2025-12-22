# Grid Component - Unity Compatibility Updates

## ✅ Changes Completed

### 1. Property Names (Unity Standard)
**Before** → **After**:
- ❌ "Layout" → ✅ "Cell Layout"
- ❌ "Swizzle" → ✅ "Cell Swizzle"
- ✅ "Cell Size" (already correct)
- ✅ "Cell Gap" (already correct)

### 2. Cell Gap Validation (Unity Behavior)
**Implemented**: ✅ Auto-clamp negative Cell Gap values

**Unity Rule**:
> "If a negative number with an absolute value higher than the Cell Size is entered, Unity will automatically change the absolute value to match the Cell Size instead."

**Example**:
```
Cell Size: (1.0, 1.0, 0.0)
User enters Cell Gap: (-2.0, -2.0)
System auto-clamps to: (-1.0, -1.0)
```

**Implementation**:
```rust
// In Inspector when Cell Gap changes:
if gap_x < 0.0 && gap_x.abs() > grid.cell_size.0 {
    gap_x = -grid.cell_size.0;  // Clamp to -cell_size
}
```

### 3. Cell Swizzle in Inspector
**Added**: ✅ Cell Swizzle dropdown in Inspector

**Options**:
- XYZ (default)
- XZY
- YXZ
- YZX
- ZXY
- ZYX

## 📋 Inspector Layout (Unity-style)

```
Grid Component
├─ Cell Size
│  ├─ X: [0.08]
│  ├─ Y: [0.08]
│  └─ Z: [0.00]
├─ Cell Gap (with validation)
│  ├─ X: [0.00]
│  └─ Y: [0.00]
├─ Cell Layout: [Rectangle ▼]
│  ├─ Rectangle
│  ├─ Hexagon (Flat Top)
│  ├─ Hexagon (Pointy Top)
│  └─ Isometric
├─ Cell Swizzle: [XYZ ▼]
│  ├─ XYZ
│  ├─ XZY
│  ├─ YXZ
│  ├─ YZX
│  ├─ ZXY
│  └─ ZYX
└─ Plane: [XY (Horizontal) ▼]
   ├─ XY (Horizontal)
   ├─ XZ (Vertical)
   └─ YZ (Side)
```

## 🎯 Testing

### Test Cell Gap Validation:
1. Select Grid entity
2. Inspector > Cell Gap > X
3. Enter `-2.0` (when Cell Size X = 0.08)
4. Value should auto-clamp to `-0.08`

### Test Cell Swizzle:
1. Select Grid entity
2. Inspector > Cell Swizzle
3. Change from XYZ to XZY
4. Verify coordinate transformation works

### Test Cell Layout:
1. Select Grid entity
2. Inspector > Cell Layout
3. Change between Rectangle, Hexagon, Isometric
4. Verify grid visualization updates

## 🔄 Compatibility

**Backward Compatible**: ✅ Yes
- Old scene files will load correctly
- Default values maintained
- No breaking changes

**Unity Compatible**: ✅ Yes
- Property names match Unity
- Validation behavior matches Unity
- UI layout similar to Unity

## 📝 Next Steps (Optional)

### Priority 2 Features:
1. ❌ Grid Snapping (Ctrl+Drag to snap)
2. ❌ Isometric Z as Y layout
3. ❌ Grid Settings Panel (Edit > Grid and Snap Settings)
4. ❌ Snap Guides visualization

These can be added later without breaking existing functionality.
