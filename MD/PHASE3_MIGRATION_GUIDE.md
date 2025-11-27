# 🔧 Phase 3: Code Migration Guide

## 📋 Overview

This guide will help you move the remaining ~1,150 lines of code from `scene_view.rs` to the new module structure.

**Estimated Time:** 2-3 hours
**Difficulty:** Medium
**Approach:** Move one module at a time, test after each move

## 🎯 Migration Order

1. ✅ **Gizmos** (30 min) - Independent, no dependencies
2. ✅ **Camera Interaction** (30 min) - Independent
3. ✅ **Transform Interaction** (30 min) - Independent
4. ✅ **Entity Rendering** (45 min) - Uses gizmos
5. ✅ **Update mod.rs** (15 min) - Wire everything together
6. ✅ **Test** (30 min) - Verify everything works

---

## Step 1: Move Gizmo Rendering (30 min)

### File: `scene_view/rendering/gizmos.rs`

**What to move:**
- `render_scene_gizmo_visual()` - Scene gizmo (XYZ axes)
- `render_transform_gizmo()` - Transform handles
- `render_collider_gizmo()` - Collider outlines
- `render_velocity_gizmo()` - Velocity arrows

**How to find in scene_view.rs:**
```bash
# Search for these functions:
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_scene_gizmo_visual"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_transform_gizmo"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_collider_gizmo"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_velocity_gizmo"
```

**Steps:**
1. Open `scene_view.rs` and find `fn render_scene_gizmo_visual`
2. Copy the entire function (including doc comments)
3. Paste into `gizmos.rs`, replacing the placeholder
4. Repeat for other 3 functions
5. Add necessary imports at top of `gizmos.rs`
6. Build: `cargo build --package engine`

**Required imports for gizmos.rs:**
```rust
use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
use super::super::types::*;
```

---

## Step 2: Move Camera Interaction (30 min)

### File: `scene_view/interaction/camera.rs`

**What to move:**
- `handle_camera_controls()` - Pan/orbit/zoom
- `handle_gizmo_axis_clicks()` - Preset camera views

**How to find:**
```bash
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn handle_camera_controls"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn handle_gizmo_axis_clicks"
```

**Steps:**
1. Find `fn handle_camera_controls` in `scene_view.rs`
2. Copy entire function
3. Paste into `camera.rs`, replacing placeholder
4. Repeat for `handle_gizmo_axis_clicks`
5. Add imports
6. Build and test

**Required imports:**
```rust
use ecs::{World, Entity};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;
```

---

## Step 3: Move Transform Interaction (30 min)

### File: `scene_view/interaction/transform.rs`

**What to move:**
- `handle_gizmo_interaction_stateful()` - Main gizmo interaction

**How to find:**
```bash
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn handle_gizmo_interaction_stateful"
```

**Steps:**
1. Find `fn handle_gizmo_interaction_stateful` in `scene_view.rs`
2. This is a LARGE function (~250 lines)
3. Copy entire function carefully
4. Paste into `transform.rs`
5. Add imports
6. Build and test

**Required imports:**
```rust
use ecs::{World, Entity};
use egui;
use crate::editor::ui::TransformTool;
use crate::editor::SceneCamera;
use super::super::types::*;
```

---

## Step 4: Move Entity Rendering (45 min)

### File: `scene_view/rendering/entity.rs`

**What to move:**
- Main entity rendering loop (from `render_scene_view`)
- `render_entity()` - Sprite rendering
- `render_mesh_entity()` - Mesh rendering
- `render_3d_cube()` - 3D cube
- `calculate_3d_cube_bounds()` - Bounds
- `render_camera_gizmo()` - Camera icon

**This is the MOST COMPLEX step!**

**How to find:**
```bash
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_entity"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_mesh_entity"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_3d_cube"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn calculate_3d_cube_bounds"
Select-String -Path engine/src/editor/ui/scene_view.rs -Pattern "^fn render_camera_gizmo"
```

**Steps:**

### 4.1: Move Helper Functions First
1. Copy `render_camera_gizmo()` → `entity.rs`
2. Copy `calculate_3d_cube_bounds()` → `entity.rs`
3. Copy `render_3d_cube()` → `entity.rs`
4. Copy `render_mesh_entity()` → `entity.rs`
5. Copy `render_entity()` → `entity.rs`

### 4.2: Create Main Rendering Loop
In `entity.rs`, implement `render_all_entities()`:

```rust
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
    // Copy the entity rendering loop from render_scene_view()
    // Look for: "// === ENTITIES ===" comment
    // Copy from there until "// === SELECTION OUTLINES ===" comment
    
    // This includes:
    // 1. Collecting entities
    // 2. Sorting by Z position
    // 3. Separating opaque/transparent
    // 4. Rendering loop
    // 5. Selection outlines
}
```

**Required imports:**
```rust
use ecs::{World, Entity};
use egui;
use crate::editor::SceneCamera;
use super::super::types::*;
use super::gizmos::{render_collider_gizmo, render_velocity_gizmo};
```

---

## Step 5: Update mod.rs (15 min)

### File: `scene_view/mod.rs`

The `mod.rs` already has the structure, but you need to verify all function calls are correct.

**Check these calls:**
- `rendering::grid::render_grid_2d()` ✅
- `rendering::grid::render_grid_3d()` ✅
- `rendering::gizmos::render_scene_gizmo_visual()` - Update after Step 1
- `rendering::gizmos::render_transform_gizmo()` - Update after Step 1
- `rendering::entity::render_all_entities()` - Update after Step 4
- `interaction::camera::handle_camera_controls()` - Update after Step 2
- `interaction::camera::handle_gizmo_axis_clicks()` - Update after Step 2
- `interaction::transform::handle_gizmo_interaction_stateful()` - Update after Step 3

---

## Step 6: Update scene_view.rs (15 min)

After moving all code, update `scene_view.rs` to be a simple re-export:

```rust
//! Scene View Module (Re-export)
//!
//! This file is kept for backward compatibility.
//! The actual implementation is in the scene_view/ submodule.
//!
//! ## Migration Notice
//! This module has been refactored into smaller, more manageable files.
//! See the `scene_view/` directory for the new structure.

// Re-export everything from the new module
mod scene_view;
pub use scene_view::*;
```

**OR** if you want to keep the old file temporarily:

1. Rename `scene_view.rs` to `scene_view_old.rs`
2. Create new `scene_view.rs` with just re-exports
3. Test everything
4. Delete `scene_view_old.rs` when confident

---

## 🧪 Testing Checklist

After each step, run these tests:

### Build Test
```bash
cargo build --package engine
```

### Feature Tests
1. ✅ Open editor
2. ✅ Scene view displays
3. ✅ Grid renders (2D and 3D)
4. ✅ Entities render
5. ✅ Select entity
6. ✅ Move tool (W) works
7. ✅ Rotate tool (E) works
8. ✅ Scale tool (R) works
9. ✅ Camera pan (middle mouse)
10. ✅ Camera orbit (Alt + left mouse)
11. ✅ Camera zoom (scroll)
12. ✅ Keyboard shortcuts (Q, W, E, R)
13. ✅ Numpad camera views (1, 3, 7, 0)
14. ✅ Collider gizmos show
15. ✅ Velocity gizmos show

---

## 🚨 Common Issues & Solutions

### Issue 1: "Cannot find function in this scope"
**Solution:** Add the correct import at the top of the file

### Issue 2: "Private function used in public interface"
**Solution:** Make sure functions are `pub fn` not just `fn`

### Issue 3: "Circular dependency"
**Solution:** Check your module imports, might need to restructure

### Issue 4: "Type not found"
**Solution:** Add `use super::super::types::*;` or specific type import

### Issue 5: Build succeeds but features don't work
**Solution:** Check that `mod.rs` is calling the right functions

---

## 📊 Progress Tracking

Use this checklist:

```
Phase 3 Progress:
[ ] Step 1: Move gizmos.rs (30 min)
    [ ] render_scene_gizmo_visual
    [ ] render_transform_gizmo
    [ ] render_collider_gizmo
    [ ] render_velocity_gizmo
    [ ] Build test
    
[ ] Step 2: Move camera.rs (30 min)
    [ ] handle_camera_controls
    [ ] handle_gizmo_axis_clicks
    [ ] Build test
    
[ ] Step 3: Move transform.rs (30 min)
    [ ] handle_gizmo_interaction_stateful
    [ ] Build test
    
[ ] Step 4: Move entity.rs (45 min)
    [ ] render_camera_gizmo
    [ ] calculate_3d_cube_bounds
    [ ] render_3d_cube
    [ ] render_mesh_entity
    [ ] render_entity
    [ ] render_all_entities (main loop)
    [ ] Build test
    
[ ] Step 5: Update mod.rs (15 min)
    [ ] Verify all function calls
    [ ] Build test
    
[ ] Step 6: Update scene_view.rs (15 min)
    [ ] Create re-export
    [ ] Build test
    
[ ] Final Testing (30 min)
    [ ] All 15 feature tests pass
    [ ] No regressions
    [ ] Performance is good
```

---

## 💡 Pro Tips

1. **Work in small chunks** - Move one function at a time
2. **Build frequently** - After each function move
3. **Keep notes** - Write down what you moved
4. **Use git** - Commit after each successful step
5. **Take breaks** - This is tedious work!
6. **Don't rush** - Better to be slow and correct

---

## 🎯 Expected Results

### Before
```
scene_view.rs: 1,992 lines 🔴
```

### After
```
scene_view.rs: ~50 lines (re-export) ✅
scene_view/
├── mod.rs: 250 lines
├── types.rs: 180 lines
├── toolbar.rs: 60 lines
├── shortcuts.rs: 60 lines
├── rendering/
│   ├── grid.rs: 200 lines
│   ├── entity.rs: 400 lines
│   └── gizmos.rs: 450 lines
└── interaction/
    ├── camera.rs: 200 lines
    └── transform.rs: 250 lines

Total: ~2,050 lines across 10 files
Average: ~205 lines per file ✅
```

---

## 🚀 Ready to Start?

1. Open `scene_view.rs` in your editor
2. Open this guide in a browser
3. Start with Step 1 (Gizmos)
4. Follow the checklist
5. Test after each step
6. Celebrate when done! 🎉

**Good luck!** You've got this! 💪
