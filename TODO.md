# 📋 TODO List - Priority Fixes & Features

## 🔴 Critical Bugs (Fix First)

### 1. ✅ Startup Scene Not Loading
**Status:** Fixed
**Priority:** 🔴 Critical
**Effort:** 2 hours

**Problem:**
- Engine doesn't load the startup scene on launch
- No setting to configure which scene should start

**Solution:**
```rust
// Add to ProjectSettings
struct ProjectSettings {
    startup_scene: Option<PathBuf>,
}

// On project load, automatically load startup scene
if let Some(scene) = project_settings.startup_scene {
    editor_state.load_scene(&scene)?;
}
```

**Files to modify:**
- `engine_core/src/project.rs` - Add startup_scene field
- `game/src/main.rs` - Load startup scene on project open

---

### 2. ✅ Transform Gizmo Not Following Mouse
**Status:** Fixed
**Priority:** 🔴 Critical
**Effort:** 3 hours

**Problem:**
- Gizmo handles exist but drag doesn't follow mouse smoothly
- Position updates but feels laggy or jumpy

**Solution:**
```rust
// Use direct mouse position instead of drag_delta
if response.dragged() {
    if let Some(mouse_pos) = response.interact_pointer_pos() {
        // Convert screen to world coordinates
        let world_x = mouse_pos.x - center_x;
        let world_y = mouse_pos.y - center_y;

        // Apply axis constraints
        match drag_axis {
            0 => transform.x = world_x,  // X only
            1 => transform.y = world_y,  // Y only
            2 => {                       // Both
                transform.x = world_x;
                transform.y = world_y;
            }
        }
    }
}
```

**Files to modify:**
- `game/src/editor_ui.rs` - Fix gizmo interaction logic (lines 540-588)

---

## 🟡 High Priority Features

### 3. 🔧 Unity-Style Hierarchy Panel
**Status:** Not Implemented
**Priority:** 🟡 High
**Effort:** 4 hours

**Current:**
```
Hierarchy
├─ Item 2
├─ GameObject 1
└─ Player
```

**Target (Unity-style):**
```
Hierarchy
└─ 📄 Scene: main.json
   ├─ 🎮 Player
   ├─ 📦 Platform
   ├─ 🏠 Background
   └─ 📷 Main Camera
```

**Features Needed:**
- [x] Scene name as root node
- [ ] GameObject icons (🎮 player, 📦 object, 📷 camera, etc.)
- [ ] Parent-child hierarchy (drag to parent)
- [ ] Right-click context menu
  - Create GameObject → 2D Object
  - Create GameObject → Empty
  - Create GameObject → Empty Child
  - Delete GameObject
  - Duplicate GameObject

**Implementation:**
```rust
// Hierarchy tree structure
struct HierarchyNode {
    entity: Entity,
    name: String,
    children: Vec<Entity>,
    icon: String,  // Emoji or icon ID
}

// Right-click menu
if ui.button("Create 2D Object").clicked() {
    create_game_object_with_sprite();
}
if ui.button("Create Empty").clicked() {
    create_empty_game_object();
}
```

**Files to modify:**
- `game/src/editor_ui.rs` - Hierarchy panel rendering
- `ecs/src/lib.rs` - Add parent-child relationships

---

### 4. 🗂️ Unity-Style Project/Asset Manager
**Status:** Basic Grid View Only
**Priority:** 🟡 High
**Effort:** 8 hours

**Current Features:**
- ✅ Grid view folders/files
- ✅ Icons for different types
- ❌ Can't create folders
- ❌ Can't create scripts
- ❌ Can't drag GameObjects to save as prefabs
- ❌ No right-click menu

**Target (Unity-style):**
```
Project Window
├─ Assets/
│  ├─ Scenes/
│  ├─ Scripts/
│  ├─ Sprites/
│  ├─ Prefabs/     ← NEW
│  └─ Audio/       ← NEW
│
Right-click menu:
├─ Create → Folder
├─ Create → Lua Script
├─ Create → Scene
├─ Rename
├─ Delete
└─ Show in Explorer
```

**Drag GameObject → Asset Manager:**
```rust
// When dragging GameObject from Hierarchy to Project
if let Some(entity) = dragged_entity {
    if drop_zone == "project_panel" {
        // Save as prefab
        let prefab = create_prefab_from_entity(entity);
        save_prefab_to_disk(prefab, "Assets/Prefabs/NewPrefab.prefab");
    }
}
```

**Features to Add:**
- [ ] Create Folder button
- [ ] Create Script button (template .lua file)
- [ ] Right-click context menu
- [ ] Drag GameObject → save as Prefab
- [ ] Drag Prefab → Hierarchy to instantiate
- [ ] Rename/Delete assets

**Files to modify:**
- `game/src/editor_ui.rs` - Project panel (lines 650-748)
- `ecs/src/lib.rs` - Prefab serialization

---

### 5. 🎯 Transform Tools: Move, Rotate, Scale
**Status:** Only Move Exists
**Priority:** 🟡 High
**Effort:** 6 hours

**Current:**
- ✅ Move tool (W key)
- ❌ Rotate tool (E key)
- ❌ Scale tool (R key)

**Target (Unity hotkeys):**
```
W - Move Tool    (current gizmo)
E - Rotate Tool  (circular handle)
R - Scale Tool   (corner handles)
Q - Hand Tool    (pan camera)
```

**Rotate Gizmo:**
```
        ↑ Y
        │
    ╱───┼───╲
   │    │    │
───┼────●────┼─── X
   │    │    │
    ╲───┼───╱
        │

- Circular handle around object
- Drag to rotate
- Show rotation angle while dragging
```

**Scale Gizmo:**
```
  ■────────■
  │        │
  │   ●    │  ← Center
  │        │
  ■────────■

- 4 corner handles
- Drag corner to scale
- Shift = uniform scaling
- Alt = scale from center
```

**Implementation:**
```rust
enum GizmoMode {
    Move,    // W
    Rotate,  // E
    Scale,   // R
}

struct EditorState {
    gizmo_mode: GizmoMode,
}

// Keyboard shortcuts
if ctx.input(|i| i.key_pressed(egui::Key::W)) {
    gizmo_mode = GizmoMode::Move;
}
if ctx.input(|i| i.key_pressed(egui::Key::E)) {
    gizmo_mode = GizmoMode::Rotate;
}
if ctx.input(|i| i.key_pressed(egui::Key::R)) {
    gizmo_mode = GizmoMode::Scale;
}
```

**Files to modify:**
- `game/src/main.rs` - Add gizmo_mode state
- `game/src/editor_ui.rs` - Add rotate/scale gizmo rendering + interaction

---

### 6. 🔧 Inspector with Full 3D Transform
**Status:** Basic 2D Only
**Priority:** 🟡 High
**Effort:** 3 hours

**Current:**
```
Transform
  Position X: [  21  ]
  Position Y: [  41  ]
  Rotation:   [  0   ]
  Scale:      [ 1.0  ]
```

**Target (Unity 3D-style):**
```
Transform
  Position
    X  [  0.00  ]
    Y  [  0.00  ]
    Z  [  0.00  ]

  Rotation
    X  [  0.00  ]
    Y  [  0.00  ]
    Z  [  0.00  ]

  Scale
    X  [  1.00  ]
    Y  [  1.00  ]
    Z  [  1.00  ]

  [Reset]
```

**Why 3D?**
- WGPU is 3D rendering system
- Future-proof for 2.5D games
- Z-order for layer sorting
- Z-rotation for billboards/effects

**Implementation:**
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    // Position
    pub x: f32,
    pub y: f32,
    pub z: f32,  // Z-order / depth

    // Rotation (Euler angles)
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,

    // Scale
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0, y: 0.0, z: 0.0,
            rotation_x: 0.0, rotation_y: 0.0, rotation_z: 0.0,
            scale_x: 1.0, scale_y: 1.0, scale_z: 1.0,
        }
    }
}
```

**Inspector UI:**
```rust
// Inspector rendering
ui.label("Transform");
ui.indent("transform", |ui| {
    ui.label("Position");
    ui.horizontal(|ui| {
        ui.label("X");
        ui.add(egui::DragValue::new(&mut transform.x).speed(0.1));
        ui.label("Y");
        ui.add(egui::DragValue::new(&mut transform.y).speed(0.1));
        ui.label("Z");
        ui.add(egui::DragValue::new(&mut transform.z).speed(0.1));
    });

    ui.label("Rotation");
    ui.horizontal(|ui| {
        ui.label("X");
        ui.add(egui::DragValue::new(&mut transform.rotation_x).speed(0.5));
        ui.label("Y");
        ui.add(egui::DragValue::new(&mut transform.rotation_y).speed(0.5));
        ui.label("Z");
        ui.add(egui::DragValue::new(&mut transform.rotation_z).speed(0.5));
    });

    ui.label("Scale");
    ui.horizontal(|ui| {
        ui.label("X");
        ui.add(egui::DragValue::new(&mut transform.scale_x).speed(0.01));
        ui.label("Y");
        ui.add(egui::DragValue::new(&mut transform.scale_y).speed(0.01));
        ui.label("Z");
        ui.add(egui::DragValue::new(&mut transform.scale_z).speed(0.01));
    });

    if ui.button("Reset").clicked() {
        *transform = Transform::default();
    }
});
```

**Files to modify:**
- `ecs/src/lib.rs` - Update Transform struct to 3D
- `game/src/editor_ui.rs` - Update Inspector Transform UI
- `game/src/main.rs` - Update Transform creation/usage

---

## 🟢 Nice to Have (Later)

### 7. GameObject Types
**Status:** Not Implemented
**Priority:** 🟢 Medium
**Effort:** 2 hours

**Types:**
- **2D Object** - Sprite + Collider + Rigidbody
- **Empty** - Just Transform
- **Empty Child** - Empty parented to selected object
- **Camera** - Camera component
- **Light** - Light component (future)
- **UI Element** - UI component (future)

### 8. Hierarchy Parent-Child Relationships
**Status:** Not Implemented
**Priority:** 🟢 Medium
**Effort:** 5 hours

**Features:**
- Drag GameObject onto another to parent
- Indentation shows hierarchy depth
- Expand/collapse parent nodes
- Local vs world transform

### 9. Prefab System
**Status:** Basic Prefab Exists
**Priority:** 🟢 Medium
**Effort:** 4 hours

**Improvements:**
- Drag GameObject to Project → save as Prefab
- Drag Prefab to Hierarchy → instantiate
- Prefab overrides (modify instance)
- Prefab variants

---

## 📊 Priority Summary

### Must Fix Today (Critical) 🔴
1. ✅ Startup Scene Loading (2h)
2. ✅ Transform Gizmo Mouse Tracking (3h)

**Total: 5 hours**

### Should Do This Week (High) 🟡
3. ✅ Unity-Style Hierarchy (4h)
4. ✅ Unity-Style Asset Manager (8h)
5. ✅ Rotate & Scale Tools (6h)
6. ✅ 3D Transform Inspector (3h)

**Total: 21 hours**

### Can Do Later (Medium) 🟢
7. GameObject Types (2h)
8. Parent-Child Hierarchy (5h)
9. Prefab Improvements (4h)

**Total: 11 hours**

---

## 🚀 Implementation Order

### Day 1: Critical Fixes (5h)
```
Morning:
[x] 1. Fix Startup Scene (2h)
[ ] 2. Fix Gizmo Mouse Tracking (3h)

Afternoon:
[ ] Test both fixes
[ ] Commit changes
```

### Day 2: Hierarchy & Inspector (7h)
```
Morning:
[ ] 3. Unity-Style Hierarchy (4h)

Afternoon:
[ ] 6. 3D Transform Inspector (3h)
[ ] Test & commit
```

### Day 3: Transform Tools (6h)
```
All Day:
[ ] 5. Rotate & Scale Gizmos (6h)
[ ] Test all 3 tools (W/E/R)
[ ] Commit
```

### Day 4-5: Asset Manager (8h)
```
Day 4:
[ ] 4a. Create Folder/Script (4h)

Day 5:
[ ] 4b. Right-click menu (2h)
[ ] 4c. Drag GameObject to Prefab (2h)
[ ] Test & commit
```

**Total: 26 hours (~3.5 days)**

---

## 📝 Notes

### Technical Decisions

**Q: Why 3D Transform for 2D engine?**
A:
- WGPU is inherently 3D
- Z-order for layer sorting (background, midground, foreground)
- Future 2.5D support (isometric, parallax)
- Z-rotation for effects
- Industry standard (Unity 2D also uses 3D transforms)

**Q: Should we add parent-child transforms?**
A:
- YES for full Unity parity
- Needed for character rigs (body + arms + legs)
- Needed for UI hierarchies
- Adds complexity but essential for real games

**Q: Prefab vs GameObject?**
A:
- GameObject = instance in scene
- Prefab = template/blueprint
- Prefab lives in Assets folder
- Can instantiate Prefab many times
- Essential for level design workflow

### Code Impact

**Files that will change:**
- `ecs/src/lib.rs` - Transform to 3D, parent-child
- `game/src/main.rs` - Gizmo modes, startup scene
- `game/src/editor_ui.rs` - Hierarchy, Inspector, Asset Manager, Gizmos
- `engine_core/src/project.rs` - Project settings

**New files needed:**
- None (all modifications to existing)

**Breaking changes:**
- Transform struct changes (2D → 3D)
- Need to update all Transform usages
- Scene files will need migration

**Migration strategy:**
```rust
// Old Transform (2D)
Transform { x: 10.0, y: 20.0, rotation: 0.0, scale: 1.0 }

// New Transform (3D)
Transform {
    x: 10.0, y: 20.0, z: 0.0,
    rotation_x: 0.0, rotation_y: 0.0, rotation_z: 0.0,
    scale_x: 1.0, scale_y: 1.0, scale_z: 1.0,
}

// Auto-migration on load
if old_format {
    transform.z = 0.0;
    transform.rotation_x = 0.0;
    transform.rotation_y = 0.0;
    transform.rotation_z = old_transform.rotation;
    transform.scale_x = old_transform.scale;
    transform.scale_y = old_transform.scale;
    transform.scale_z = 1.0;
}
```

---

## ✅ Acceptance Criteria

### Done When:

**Startup Scene:**
- [ ] Can set startup scene in Project Settings
- [ ] Startup scene loads automatically on project open
- [ ] Can change startup scene anytime

**Gizmo Mouse Tracking:**
- [ ] Gizmo follows mouse smoothly (no lag)
- [ ] Axis constraints work (X-only, Y-only, Both)
- [ ] Visual feedback during drag

**Hierarchy:**
- [ ] Scene name shows as root
- [ ] GameObjects show as children
- [ ] Icons for different types
- [ ] Right-click → Create GameObject works
- [ ] Can delete/duplicate

**Asset Manager:**
- [ ] Can create folders
- [ ] Can create Lua scripts (template)
- [ ] Right-click menu works
- [ ] Drag GameObject → saves as Prefab
- [ ] Drag Prefab → instantiates in scene

**Transform Tools:**
- [ ] W = Move (existing, improved)
- [ ] E = Rotate (new)
- [ ] R = Scale (new)
- [ ] Visual feedback for current tool

**3D Transform:**
- [ ] Inspector shows X/Y/Z for position
- [ ] Inspector shows X/Y/Z for rotation
- [ ] Inspector shows X/Y/Z for scale
- [ ] Reset button works
- [ ] All tools respect 3D transform

---

## 🎯 Success Metrics

**User can:**
1. ✅ Open project and see startup scene automatically
2. ✅ Move GameObject smoothly with mouse
3. ✅ Rotate GameObject with E key
4. ✅ Scale GameObject with R key
5. ✅ Create folders in Asset Manager
6. ✅ Create Lua scripts from Asset Manager
7. ✅ Drag GameObject to save as Prefab
8. ✅ Right-click Hierarchy to add GameObjects
9. ✅ See full 3D Transform in Inspector
10. ✅ Use Unity-like workflow

**Performance:**
- No lag during gizmo drag
- Smooth 60 FPS
- Quick startup (<2s)

**Quality:**
- No crashes
- Clear visual feedback
- Intuitive controls
- Professional appearance

---

**Last Updated:** 2025-11-25
**Assigned To:** Development Team
**Estimated Completion:** 3-5 days
**Status:** 🟡 In Progress
