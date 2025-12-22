# 🔧 Scene View Refactoring Progress

## ✅ Completed (Phase 1)

### 1. Directory Structure ✅
```
engine/src/editor/ui/scene_view/
├── mod.rs ✅
├── types.rs ✅
├── rendering/
│   └── mod.rs ✅
└── interaction/
    └── mod.rs ✅
```

### 2. Files Created ✅
- ✅ `scene_view/mod.rs` - Main module with render_scene_view()
- ✅ `scene_view/types.rs` - All types, enums, Point3D, helpers
- ✅ `scene_view/rendering/mod.rs` - Rendering module declaration
- ✅ `scene_view/interaction/mod.rs` - Interaction module declaration

## 🚧 Remaining Work

### Phase 2: Create Placeholder Files (15 min)

Create these empty files with basic structure:

```bash
# Rendering modules
touch engine/src/editor/ui/scene_view/rendering/grid.rs
touch engine/src/editor/ui/scene_view/rendering/entity.rs
touch engine/src/editor/ui/scene_view/rendering/gizmos.rs

# Interaction modules
touch engine/src/editor/ui/scene_view/interaction/camera.rs
touch engine/src/editor/ui/scene_view/interaction/transform.rs

# UI modules
touch engine/src/editor/ui/scene_view/toolbar.rs
touch engine/src/editor/ui/scene_view/shortcuts.rs
```

### Phase 3: Move Code (2-3 hours)

#### 3.1 Grid Rendering → `rendering/grid.rs`

**Functions to move:**
- `render_grid_2d()`
- `render_grid_3d()`

**Template:**
```rust
//! Grid Rendering
//!
//! 2D and 3D grid rendering functions.

use egui;
use crate::editor::{SceneCamera, SceneGrid};

pub fn render_grid_2d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    // Copy code from scene_view.rs lines ~600-650
}

pub fn render_grid_3d(
    painter: &egui::Painter,
    rect: egui::Rect,
    scene_camera: &SceneCamera,
    scene_grid: &SceneGrid,
) {
    // Copy code from scene_view.rs lines ~650-800
}
```

#### 3.2 Entity Rendering → `rendering/entity.rs`

**Functions to move:**
- `render_all_entities()` (new wrapper function)
- `render_entity()`
- `render_mesh_entity()`
- `render_3d_cube()`
- `calculate_3d_cube_bounds()`
- `render_camera_gizmo()`

**Template:**
```rust
//! Entity Rendering
//!
//! Functions for rendering entities (sprites, meshes, cameras).

use ecs::{World, Entity};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;

pub fn render_all_entities(
    painter: &egui::Painter,
    world: &mut World,
    scene_camera: &SceneCamera,
    scene_view_mode: &SceneViewMode,
    projection_mode: &ProjectionMode,
    center: egui::Pos2,
    selected_entity: &Option<Entity>,
    show_colliders: &bool,
    show_velocities: &bool,
    hovered_entity: &mut Option<Entity>,
    response: &egui::Response,
) {
    // Main entity rendering loop
    // Copy code from scene_view.rs lines ~200-500
}

pub fn render_entity(...) {
    // Copy code from scene_view.rs
}

pub fn render_mesh_entity(...) {
    // Copy code from scene_view.rs
}

// ... other functions
```

#### 3.3 Gizmos → `rendering/gizmos.rs`

**Functions to move:**
- `render_scene_gizmo_visual()`
- `render_transform_gizmo()`
- `render_collider_gizmo()`
- `render_velocity_gizmo()`
- `render_camera_gizmo()`

#### 3.4 Camera Controls → `interaction/camera.rs`

**Functions to move:**
- `handle_camera_controls()`
- `handle_gizmo_axis_clicks()`

#### 3.5 Transform Interaction → `interaction/transform.rs`

**Functions to move:**
- `handle_gizmo_interaction_stateful()`

#### 3.6 Toolbar → `toolbar.rs`

**Functions to move:**
- `render_scene_toolbar()`

#### 3.7 Shortcuts → `shortcuts.rs`

**Functions to move:**
- `handle_keyboard_shortcuts()`

### Phase 4: Update scene_view.rs (30 min)

Replace `engine/src/editor/ui/scene_view.rs` with a simple re-export:

```rust
//! Scene View (Re-export)
//!
//! This file is kept for backward compatibility.
//! The actual implementation is in the scene_view/ module.

// Re-export everything from the new module
pub use self::scene_view::*;

// Module declaration
mod scene_view;
```

### Phase 5: Test & Fix (30 min)

1. Run `cargo build --package engine`
2. Fix any compilation errors
3. Test all features:
   - Move tool
   - Rotate tool
   - Scale tool
   - Camera controls
   - Grid rendering
   - Gizmo rendering

## 📋 Checklist

- [x] Create directory structure
- [x] Create types.rs
- [x] Create mod.rs
- [x] Create rendering/mod.rs
- [x] Create interaction/mod.rs
- [ ] Create all placeholder files
- [ ] Move grid rendering code
- [ ] Move entity rendering code
- [ ] Move gizmo rendering code
- [ ] Move camera interaction code
- [ ] Move transform interaction code
- [ ] Move toolbar code
- [ ] Move shortcuts code
- [ ] Update scene_view.rs to re-export
- [ ] Test compilation
- [ ] Test all features
- [ ] Update documentation

## 🎯 Quick Commands

### Create Remaining Files
```bash
cd engine/src/editor/ui/scene_view

# Rendering
echo "//! Grid rendering" > rendering/grid.rs
echo "//! Entity rendering" > rendering/entity.rs
echo "//! Gizmo rendering" > rendering/gizmos.rs

# Interaction
echo "//! Camera controls" > interaction/camera.rs
echo "//! Transform interaction" > interaction/transform.rs

# UI
echo "//! Toolbar" > toolbar.rs
echo "//! Keyboard shortcuts" > shortcuts.rs
```

### Build & Test
```bash
cargo build --package engine
cargo test --package engine
```

## 💡 Tips

1. **Move one module at a time** - Don't try to move everything at once
2. **Test after each move** - Make sure it compiles
3. **Keep git commits small** - One module per commit
4. **Use search & replace** - For updating imports
5. **Don't rush** - Take breaks between modules

## 🚀 Next Steps

1. Create all placeholder files (15 min)
2. Move grid.rs (30 min)
3. Move entity.rs (45 min)
4. Move gizmos.rs (30 min)
5. Move camera.rs (30 min)
6. Move transform.rs (30 min)
7. Move toolbar.rs (15 min)
8. Move shortcuts.rs (15 min)
9. Update scene_view.rs (15 min)
10. Test everything (30 min)

**Total remaining:** ~4 hours

---

**Current Status:** Phase 1 Complete ✅
**Next:** Create placeholder files
