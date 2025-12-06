# Unity-like Scene Editor Analysis
## วิเคราะห์และแนะนำสิ่งที่ต้องเพิ่มเติมเพื่อให้เหมือน Unity

---

## 📊 สรุปภาพรวม

**คำตอบ: เป็นไปได้แน่นอน!** 

โครงสร้างปัจจุบันของคุณมีพื้นฐานที่ดีมากแล้ว และสามารถพัฒนาต่อเป็น Unity-like Scene Editor ได้เต็มรูปแบบ

---

## ✅ สิ่งที่มีอยู่แล้ว (Foundation)

### 1. **Core Architecture** ✓
- ✅ Modular structure (types, rendering, interaction, toolbar, shortcuts)
- ✅ 2D และ 3D view modes แยกกันชัดเจน
- ✅ Scene camera system พร้อม pan/zoom/orbit
- ✅ Entity-Component-System (ECS) architecture

### 2. **Camera System** ✓
- ✅ 2D camera (pan, zoom)
- ✅ 3D camera (orbit, rotation, pitch)
- ✅ Perspective และ Isometric projection
- ✅ Camera view presets (Front, Back, Top, Bottom, Left, Right)
- ✅ Focus on selected entity (F key)
- ✅ Smooth camera transitions

### 3. **Transform Tools** ✓
- ✅ View tool (Q)
- ✅ Move tool (W) - with X/Y axis handles
- ✅ Rotate tool (E) - with rotation circle
- ✅ Scale tool (R) - with X/Y axis handles
- ✅ Transform space (Local/World)
- ✅ Unity-like keyboard shortcuts (Q/W/E/R)

### 4. **Gizmos** ✓
- ✅ Transform gizmos (Move/Rotate/Scale)
- ✅ Scene gizmo (XYZ axes visualization)
- ✅ Collider gizmos (green outline)
- ✅ Velocity gizmos (yellow arrows)
- ✅ Camera gizmos (yellow trapezoid)
- ✅ Camera viewport bounds

### 5. **Grid System** ✓
- ✅ 2D grid rendering
- ✅ 3D grid rendering
- ✅ Grid toggle on/off

### 6. **Selection & Interaction** ✓
- ✅ Entity selection (click)
- ✅ Hover detection
- ✅ Drag-and-drop from asset browser
- ✅ Gizmo interaction (stateful dragging)

### 7. **Visual Debugging** ✓
- ✅ Show colliders toggle
- ✅ Show velocities toggle
- ✅ Show debug lines toggle
- ✅ Debug draw manager

---

## 🎯 สิ่งที่ต้องเพิ่มเติมเพื่อให้เหมือน Unity

### **Priority 1: Critical Features** 🔴

#### 1.1 **Enhanced Camera System**
**Status:** 🟡 Partially implemented (in progress in spec)

**ต้องเพิ่ม:**
- [ ] Smooth camera damping/inertia (กำลังทำใน task 1)
- [ ] Cursor-based zoom (zoom to mouse position) (task 2)
- [ ] Configurable sensitivity settings (task 3)
- [ ] Camera speed multiplier (Shift = faster, Ctrl = slower)
- [ ] Flythrough mode (WASD + mouse look) สำหรับ 3D
- [ ] Frame selected (F key) - มีแล้วแต่ต้องปรับปรุง
- [ ] Frame all (A key)

**Unity Comparison:**
```
Unity:
- Right-click + WASD = Flythrough
- Alt + Left-click = Orbit
- Alt + Right-click = Zoom
- Middle-click = Pan
- Scroll = Zoom to cursor
- F = Frame selected
- Shift = Speed up
```

**Current:**
```
Your Engine:
- Middle-click = Pan ✓
- Right-click = Rotate ✓
- Alt + Left-click = Orbit ✓
- Scroll = Zoom (not to cursor) 🟡
- F = Frame selected ✓
- Numpad 1/3/7 = View presets ✓
```

---

#### 1.2 **Infinite Grid System**
**Status:** 🔴 Not implemented (planned in task 4-7)

**ต้องเพิ่ม:**
- [ ] Multi-level grid (minor, major, axis lines)
- [ ] Adaptive grid scaling based on zoom
- [ ] Smooth fade-in/fade-out transitions
- [ ] Distance-based alpha fading
- [ ] Proper perspective convergence (vanishing points)
- [ ] Grid caching for performance
- [ ] Line batching for efficient rendering

**Unity Comparison:**
```
Unity Grid:
- Infinite grid that extends to horizon
- Multiple grid levels (1, 10, 100 units)
- Smooth transitions between levels
- Fades out with distance
- Always visible at any zoom level
```

---

#### 1.3 **Snapping System**
**Status:** 🟡 Partially implemented (SnapSettings struct exists but not used)

**ต้องเพิ่ม:**
- [ ] Grid snapping (Ctrl key toggle)
- [ ] Vertex snapping (V key)
- [ ] Surface snapping
- [ ] Configurable snap increments
- [ ] Visual snap indicators
- [ ] Snap settings UI panel

**Unity Comparison:**
```
Unity Snapping:
- Ctrl + Move = Snap to grid
- V + Move = Vertex snapping
- Shift + Ctrl + Move = Surface snapping
- Edit > Snap Settings = Configure increments
```

---

#### 1.4 **2.5D Support**
**Status:** 🟡 Partially implemented (Isometric projection exists)

**ต้องเพิ่ม:**
- [ ] True 2.5D mode (orthographic 3D)
- [ ] Sprite sorting layers
- [ ] Z-depth visualization
- [ ] Billboard sprites in 3D
- [ ] Parallax layers
- [ ] 2.5D-specific gizmos

**Unity Comparison:**
```
Unity 2.5D:
- Orthographic camera in 3D space
- Sorting layers for sprites
- Z-position affects rendering order
- Can mix 2D sprites with 3D objects
```

---

### **Priority 2: Important Features** 🟠

#### 2.1 **Multi-Selection**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Box selection (drag to select multiple)
- [ ] Ctrl+Click to add/remove from selection
- [ ] Shift+Click to select range
- [ ] Select all (Ctrl+A)
- [ ] Deselect all (Ctrl+D)
- [ ] Invert selection
- [ ] Selection outline/highlight
- [ ] Multi-entity transform gizmo

---

#### 2.2 **Gizmo Enhancements**
**Status:** 🟡 Basic gizmos exist

**ต้องเพิ่ม:**
- [ ] Gizmo size scaling (independent of zoom)
- [ ] Gizmo color customization
- [ ] Hover highlighting on gizmo handles
- [ ] Planar movement handles (XY, XZ, YZ planes)
- [ ] Uniform scale handle (center cube)
- [ ] Gizmo visibility toggle
- [ ] Custom gizmo icons for different entity types
- [ ] 3D transform gizmos (not just 2D)

**Unity Comparison:**
```
Unity Gizmos:
- Move: 3 arrows + 3 planes + center cube
- Rotate: 3 circles (X/Y/Z) + outer circle (screen space)
- Scale: 3 lines with cubes + center cube (uniform)
- Gizmos maintain constant screen size
- Hover = highlight in yellow
```

---

#### 2.3 **Scene View Toolbar Enhancements**
**Status:** 🟡 Basic toolbar exists

**ต้องเพิ่ม:**
- [ ] Shading mode dropdown (Wireframe, Shaded, Textured)
- [ ] Render mode dropdown (RGB, Alpha, Overdraw, Mipmaps)
- [ ] Audio toggle
- [ ] Effects toggle (particles, post-processing)
- [ ] Gizmos dropdown menu
- [ ] Camera settings dropdown
- [ ] Scene view options menu

**Unity Comparison:**
```
Unity Toolbar:
[2D/3D] [Shading▼] [2D▼] [Gizmos▼] [Search] ... [Audio] [Effects]
```

---

#### 2.4 **Scene Gizmo Enhancements**
**Status:** 🟡 Basic scene gizmo exists

**ต้องเพิ่ม:**
- [ ] Clickable axis labels (X/Y/Z text)
- [ ] Perspective/Orthographic toggle on gizmo
- [ ] Smooth camera transitions when clicking axes
- [ ] Cone shapes for axis arrows (not just circles)
- [ ] Center cube for perspective toggle
- [ ] Tooltips on hover

---

#### 2.5 **Viewport Overlays**
**Status:** 🟡 Basic camera controls overlay exists

**ต้องเพิ่ม:**
- [ ] Stats overlay (FPS, triangles, draw calls, batches)
- [ ] Grid settings overlay
- [ ] Camera settings overlay (FOV, near/far clip)
- [ ] Render settings overlay
- [ ] Customizable overlay positions
- [ ] Show/hide overlays toggle

---

### **Priority 3: Nice-to-Have Features** 🟢

#### 3.1 **Scene View Modes**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Wireframe mode
- [ ] Shaded mode
- [ ] Textured mode
- [ ] Overdraw visualization
- [ ] Lightmap preview
- [ ] Shadow cascades visualization
- [ ] Occlusion culling visualization

---

#### 3.2 **Scene View Camera**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Scene camera as a component (can be saved)
- [ ] Multiple scene view tabs with independent cameras
- [ ] Align scene camera to game camera
- [ ] Align game camera to scene camera
- [ ] Copy camera settings
- [ ] Camera bookmarks (save/load positions)

---

#### 3.3 **Measurement Tools**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Distance measurement tool
- [ ] Angle measurement tool
- [ ] Area measurement tool
- [ ] Ruler overlay
- [ ] Grid unit display

---

#### 3.4 **Scene View Effects**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Skybox rendering
- [ ] Fog rendering
- [ ] Post-processing preview
- [ ] Particle system preview
- [ ] Lighting preview
- [ ] Shadow preview

---

#### 3.5 **Advanced Selection**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Select by type
- [ ] Select by layer
- [ ] Select by tag
- [ ] Select children
- [ ] Select parent
- [ ] Select siblings
- [ ] Grow selection
- [ ] Shrink selection

---

#### 3.6 **Scene View Search**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Search bar in scene view
- [ ] Search by name
- [ ] Search by component
- [ ] Search by tag/layer
- [ ] Highlight search results
- [ ] Navigate between results

---

#### 3.7 **Handles & Manipulators**
**Status:** 🔴 Not implemented

**ต้องเพิ่ม:**
- [ ] Custom handles API
- [ ] Position handle
- [ ] Rotation handle
- [ ] Scale handle
- [ ] Free move handle
- [ ] Radius handle (for circles/spheres)
- [ ] Bounds handle (for boxes)
- [ ] Arc handle
- [ ] Slider handle

---

## 🏗️ Architecture Recommendations

### 1. **Separate Scene View State**
```rust
pub struct SceneViewState {
    pub camera: SceneCamera,
    pub grid: SceneGrid,
    pub selection: Selection,
    pub gizmo_settings: GizmoSettings,
    pub viewport_settings: ViewportSettings,
    pub overlay_settings: OverlaySettings,
    pub shading_mode: ShadingMode,
    pub render_mode: RenderMode,
}
```

### 2. **Selection System**
```rust
pub struct Selection {
    pub entities: Vec<Entity>,
    pub active_entity: Option<Entity>,
}

impl Selection {
    pub fn add(&mut self, entity: Entity);
    pub fn remove(&mut self, entity: Entity);
    pub fn toggle(&mut self, entity: Entity);
    pub fn clear(&mut self);
    pub fn is_selected(&self, entity: Entity) -> bool;
    pub fn get_bounds(&self, world: &World) -> Option<Bounds>;
}
```

### 3. **Gizmo System**
```rust
pub trait Gizmo {
    fn render(&self, painter: &Painter, context: &GizmoContext);
    fn handle_input(&mut self, response: &Response, context: &GizmoContext) -> Option<GizmoResult>;
    fn get_bounds(&self) -> Bounds;
}

pub struct GizmoContext {
    pub camera: &SceneCamera,
    pub transform: &Transform,
    pub space: TransformSpace,
    pub snap_settings: &SnapSettings,
}
```

### 4. **Grid System**
```rust
pub struct InfiniteGrid {
    pub levels: Vec<GridLevel>,
    pub cache: Option<GridGeometry>,
    pub settings: GridSettings,
}

pub struct GridLevel {
    pub spacing: f32,
    pub color: Color32,
    pub alpha: f32,
    pub line_width: f32,
}
```

---

## 📋 Implementation Roadmap

### **Phase 1: Core Improvements** (1-2 weeks)
1. ✅ Enhanced camera system (tasks 1-3 in current spec)
2. ✅ Infinite grid system (tasks 4-7 in current spec)
3. ⬜ Snapping system
4. ⬜ Multi-selection

### **Phase 2: Gizmo & Interaction** (1-2 weeks)
5. ⬜ Enhanced gizmos (3D, planar handles)
6. ⬜ Gizmo size scaling
7. ⬜ Hover highlighting
8. ⬜ Box selection

### **Phase 3: 2.5D Support** (1 week)
9. ⬜ True 2.5D mode
10. ⬜ Sprite sorting layers
11. ⬜ Z-depth visualization

### **Phase 4: Polish & Features** (2-3 weeks)
12. ⬜ Scene view modes (wireframe, shaded, etc.)
13. ⬜ Viewport overlays
14. ⬜ Toolbar enhancements
15. ⬜ Scene gizmo improvements

### **Phase 5: Advanced Features** (2-3 weeks)
16. ⬜ Measurement tools
17. ⬜ Scene view effects
18. ⬜ Advanced selection
19. ⬜ Custom handles API

---

## 🎨 Visual Comparison

### Unity Scene View Layout:
```
┌─────────────────────────────────────────────────────────┐
│ [2D/3D] [Shading▼] [Gizmos▼] ... [Audio] [Effects]    │ Toolbar
├─────────────────────────────────────────────────────────┤
│                                              ┌────────┐ │
│                                              │  XYZ   │ │ Scene Gizmo
│                                              │ Gizmo  │ │
│                                              └────────┘ │
│                                                         │
│                  Scene View                             │
│                  (Grid + Entities)                      │
│                                                         │
│                                                         │
│ ┌─────────────────────────────────────────┐            │
│ │ Camera: Position, Rotation, FOV         │            │ Overlays
│ │ Stats: FPS, Tris, Draw Calls            │            │
│ └─────────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────┘
```

### Your Current Layout:
```
┌─────────────────────────────────────────────────────────┐
│ [View] [Move] [Rotate] [Scale] [2D/3D] [Local/World]  │ Toolbar ✓
├─────────────────────────────────────────────────────────┤
│                                              ┌────────┐ │
│                                              │  XYZ   │ │ Scene Gizmo ✓
│                                              │ Gizmo  │ │
│                                              └────────┘ │
│                                                         │
│                  Scene View                             │
│                  (Grid + Entities)                      │
│                                                         │
│                                                         │
│ ┌─────────────────────────────────────────┐            │
│ │ Camera Controls                         │            │ Basic Overlay ✓
│ └─────────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────┘
```

---

## 💡 Quick Wins (Easy to Implement)

1. **Gizmo Size Scaling** - Make gizmos maintain constant screen size
2. **Hover Highlighting** - Highlight gizmo handles on hover
3. **Box Selection** - Drag to select multiple entities
4. **Ctrl+Click Multi-Select** - Add/remove from selection
5. **Frame All (A key)** - Frame all entities in view
6. **Stats Overlay** - Show FPS, entity count, etc.
7. **Shading Mode Toggle** - Wireframe/Shaded/Textured
8. **Grid Settings UI** - Panel to configure grid appearance

---

## 🔧 Technical Considerations

### Performance:
- Grid caching is critical for 60 FPS
- Line batching reduces draw calls
- Spatial culling for off-screen entities
- LOD for distant objects

### Usability:
- Keyboard shortcuts must be consistent
- Visual feedback for all interactions
- Undo/redo support for transforms
- Smooth animations for camera transitions

### Compatibility:
- Support both 2D and 3D workflows
- Seamless switching between modes
- Preserve camera state per mode
- Handle edge cases (extreme zoom, NaN values)

---

## 📚 Reference Resources

### Unity Documentation:
- [Scene View Navigation](https://docs.unity3d.com/Manual/SceneViewNavigation.html)
- [Scene View Control Bar](https://docs.unity3d.com/Manual/ViewModes.html)
- [Gizmos](https://docs.unity3d.com/Manual/GizmosMenu.html)

### Similar Engines:
- **Godot**: Similar scene view with 2D/3D modes
- **Unreal**: More complex but similar concepts
- **Bevy Editor**: Rust-based, good reference

---

## ✅ Conclusion

**คุณมีพื้นฐานที่ดีมากแล้ว!** 

ระบบปัจจุบันของคุณมี:
- ✅ 70% ของ core features
- ✅ Architecture ที่ดี (modular, extensible)
- ✅ Camera system ที่ใช้งานได้
- ✅ Basic gizmos และ interaction

**สิ่งที่ต้องเพิ่มหลักๆ:**
1. 🔴 Infinite grid system (กำลังทำอยู่)
2. 🔴 Enhanced camera (กำลังทำอยู่)
3. 🔴 Snapping system
4. 🔴 Multi-selection
5. 🟡 Enhanced gizmos (3D, planar handles)
6. 🟡 2.5D support improvements

**Timeline ประมาณ:** 6-8 สัปดาห์สำหรับ Unity-like experience ที่สมบูรณ์

**แนะนำ:** ทำตาม spec ที่มีอยู่ก่อน (scene-view-improvements) แล้วค่อยเพิ่ม features อื่นๆ ตาม priority
