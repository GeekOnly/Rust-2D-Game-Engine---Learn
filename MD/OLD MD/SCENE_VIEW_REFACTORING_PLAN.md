# 🔧 Scene View Refactoring Plan

## 📊 Current Status

**File:** `engine/src/editor/ui/scene_view.rs`
**Size:** **1,992 lines** 🔴 (TOO LARGE!)

**Recommended:** < 500 lines per file

## 🎯 Refactoring Goals

1. **Maintainability** - ง่ายต่อการแก้ไขและเพิ่มฟีเจอร์
2. **Readability** - อ่านและเข้าใจง่าย
3. **Testability** - แยก logic ออกมา test ได้
4. **Reusability** - ใช้ซ้ำได้ในส่วนอื่น
5. **Performance** - ไม่กระทบ performance

## 📁 Proposed Structure

```
engine/src/editor/ui/
├── scene_view/
│   ├── mod.rs                    # Main module (200 lines)
│   ├── types.rs                  # Types & Enums (100 lines)
│   ├── rendering/
│   │   ├── mod.rs               # Rendering module
│   │   ├── entity.rs            # Entity rendering (200 lines)
│   │   ├── sprite.rs            # Sprite rendering (150 lines)
│   │   ├── mesh.rs              # Mesh rendering (200 lines)
│   │   ├── grid.rs              # Grid rendering (200 lines)
│   │   └── gizmos.rs            # Gizmo rendering (300 lines)
│   ├── interaction/
│   │   ├── mod.rs               # Interaction module
│   │   ├── camera.rs            # Camera controls (150 lines)
│   │   ├── selection.rs         # Entity selection (100 lines)
│   │   └── transform.rs         # Transform gizmo interaction (200 lines)
│   ├── toolbar.rs               # Toolbar UI (100 lines)
│   └── shortcuts.rs             # Keyboard shortcuts (100 lines)
└── scene_view.rs                # Re-export (deprecated, for compatibility)
```

**Total:** ~2,000 lines → 8-10 files (~200 lines each)

## 🔍 Code Analysis

### Current File Breakdown

| Section | Lines | Should Move To |
|---------|-------|----------------|
| Types & Enums | ~100 | `types.rs` |
| Main render function | ~200 | `mod.rs` |
| Toolbar | ~100 | `toolbar.rs` |
| Camera controls | ~150 | `interaction/camera.rs` |
| Grid rendering (2D) | ~100 | `rendering/grid.rs` |
| Grid rendering (3D) | ~150 | `rendering/grid.rs` |
| Scene gizmo | ~150 | `rendering/gizmos.rs` |
| Entity rendering | ~200 | `rendering/entity.rs` |
| Sprite rendering | ~150 | `rendering/sprite.rs` |
| Mesh rendering | ~300 | `rendering/mesh.rs` |
| Collider gizmo | ~100 | `rendering/gizmos.rs` |
| Velocity gizmo | ~50 | `rendering/gizmos.rs` |
| Transform gizmo | ~150 | `rendering/gizmos.rs` |
| Gizmo interaction | ~200 | `interaction/transform.rs` |
| Keyboard shortcuts | ~100 | `shortcuts.rs` |
| Helper functions | ~100 | `mod.rs` or `types.rs` |

## 📝 Refactoring Steps

### Phase 1: Create Module Structure (30 min)

1. Create `scene_view/` directory
2. Create `mod.rs` with module declarations
3. Create empty files for each module

### Phase 2: Move Types & Enums (15 min)

**File:** `scene_view/types.rs`

```rust
// Move these types:
- SceneViewMode
- ProjectionMode
- TransformSpace
- SnapMode
- SnapSettings
- Point3D
- Helper functions (snap_to_grid, rotate_point_2d)
```

### Phase 3: Move Rendering Code (60 min)

#### 3.1 Grid Rendering
**File:** `scene_view/rendering/grid.rs`

```rust
pub fn render_grid_2d(...) { }
pub fn render_grid_3d(...) { }
```

#### 3.2 Entity Rendering
**File:** `scene_view/rendering/entity.rs`

```rust
pub fn render_entity(...) { }
pub fn render_mesh_entity(...) { }
```

#### 3.3 Sprite Rendering
**File:** `scene_view/rendering/sprite.rs`

```rust
pub fn render_sprite(...) { }
pub fn render_rotated_sprite(...) { }
fn calculate_sprite_corners(...) { }
```

#### 3.4 Mesh Rendering
**File:** `scene_view/rendering/mesh.rs`

```rust
pub fn render_3d_cube(...) { }
pub fn calculate_3d_cube_bounds(...) { }
```

#### 3.5 Gizmos
**File:** `scene_view/rendering/gizmos.rs`

```rust
pub fn render_scene_gizmo_visual(...) { }
pub fn render_transform_gizmo(...) { }
pub fn render_collider_gizmo(...) { }
pub fn render_velocity_gizmo(...) { }
pub fn render_camera_gizmo(...) { }
```

### Phase 4: Move Interaction Code (45 min)

#### 4.1 Camera Controls
**File:** `scene_view/interaction/camera.rs`

```rust
pub fn handle_camera_controls(...) { }
pub fn handle_gizmo_axis_clicks(...) { }
```

#### 4.2 Transform Interaction
**File:** `scene_view/interaction/transform.rs`

```rust
pub fn handle_gizmo_interaction_stateful(...) { }
pub fn detect_gizmo_handle(...) { }
pub fn apply_transform_delta(...) { }
```

#### 4.3 Selection
**File:** `scene_view/interaction/selection.rs`

```rust
pub fn handle_entity_selection(...) { }
pub fn check_entity_hover(...) { }
```

### Phase 5: Move UI Code (30 min)

#### 5.1 Toolbar
**File:** `scene_view/toolbar.rs`

```rust
pub fn render_scene_toolbar(...) { }
```

#### 5.2 Shortcuts
**File:** `scene_view/shortcuts.rs`

```rust
pub fn handle_keyboard_shortcuts(...) { }
```

### Phase 6: Update Main Module (30 min)

**File:** `scene_view/mod.rs`

```rust
// Module declarations
pub mod types;
pub mod rendering;
pub mod interaction;
pub mod toolbar;
pub mod shortcuts;

// Re-exports
pub use types::*;

// Main render function (simplified)
pub fn render_scene_view(...) {
    // Use functions from submodules
    toolbar::render_scene_toolbar(...);
    shortcuts::handle_keyboard_shortcuts(...);
    
    // Rendering
    rendering::grid::render_grid_2d(...);
    rendering::entity::render_entity(...);
    
    // Interaction
    interaction::camera::handle_camera_controls(...);
    interaction::transform::handle_gizmo_interaction(...);
}
```

### Phase 7: Update Imports (15 min)

Update all files that import from `scene_view.rs`:

```rust
// Old
use crate::editor::ui::scene_view::{render_scene_view, SceneViewMode};

// New
use crate::editor::ui::scene_view::{render_scene_view, SceneViewMode};
// (Same! Thanks to re-exports)
```

### Phase 8: Testing (30 min)

1. Compile and fix any errors
2. Test all features:
   - Move tool
   - Rotate tool
   - Scale tool
   - Camera controls
   - Grid rendering
   - Gizmo rendering
3. Run tests: `cargo test`

## 🎯 Benefits After Refactoring

### Before
```
scene_view.rs (1,992 lines)
├── Everything mixed together
├── Hard to find specific code
├── Difficult to test
└── Scary to modify
```

### After
```
scene_view/
├── mod.rs (200 lines) ✅
├── types.rs (100 lines) ✅
├── rendering/ (1,000 lines split into 5 files) ✅
├── interaction/ (450 lines split into 3 files) ✅
├── toolbar.rs (100 lines) ✅
└── shortcuts.rs (100 lines) ✅

Total: Same functionality, better organization!
```

**Benefits:**
- ✅ Each file < 300 lines
- ✅ Clear separation of concerns
- ✅ Easy to find and modify code
- ✅ Better for team collaboration
- ✅ Easier to test individual components
- ✅ Can reuse rendering functions elsewhere

## 📊 Estimated Time

| Phase | Time | Difficulty |
|-------|------|------------|
| 1. Create structure | 30 min | Easy |
| 2. Move types | 15 min | Easy |
| 3. Move rendering | 60 min | Medium |
| 4. Move interaction | 45 min | Medium |
| 5. Move UI | 30 min | Easy |
| 6. Update main | 30 min | Medium |
| 7. Update imports | 15 min | Easy |
| 8. Testing | 30 min | Medium |
| **Total** | **~4 hours** | **Medium** |

## 🚀 Quick Start Commands

```bash
# Create directory structure
mkdir -p engine/src/editor/ui/scene_view/rendering
mkdir -p engine/src/editor/ui/scene_view/interaction

# Create files
touch engine/src/editor/ui/scene_view/mod.rs
touch engine/src/editor/ui/scene_view/types.rs
touch engine/src/editor/ui/scene_view/toolbar.rs
touch engine/src/editor/ui/scene_view/shortcuts.rs

# Rendering modules
touch engine/src/editor/ui/scene_view/rendering/mod.rs
touch engine/src/editor/ui/scene_view/rendering/entity.rs
touch engine/src/editor/ui/scene_view/rendering/sprite.rs
touch engine/src/editor/ui/scene_view/rendering/mesh.rs
touch engine/src/editor/ui/scene_view/rendering/grid.rs
touch engine/src/editor/ui/scene_view/rendering/gizmos.rs

# Interaction modules
touch engine/src/editor/ui/scene_view/interaction/mod.rs
touch engine/src/editor/ui/scene_view/interaction/camera.rs
touch engine/src/editor/ui/scene_view/interaction/selection.rs
touch engine/src/editor/ui/scene_view/interaction/transform.rs
```

## 💡 Alternative: Gradual Refactoring

If 4 hours is too much, do it gradually:

### Week 1: Extract Types
- Move types to `types.rs`
- Keep everything else in `scene_view.rs`

### Week 2: Extract Rendering
- Move rendering functions to `rendering/` modules
- Keep interaction in `scene_view.rs`

### Week 3: Extract Interaction
- Move interaction to `interaction/` modules
- Keep UI in `scene_view.rs`

### Week 4: Extract UI
- Move toolbar and shortcuts
- Clean up `mod.rs`

## 🎊 Recommendation

**Should you refactor now?**

**YES, if:**
- ✅ You plan to add more features
- ✅ Multiple people work on this code
- ✅ You want better code organization
- ✅ You have 4 hours available

**NO, if:**
- ❌ Code is working and won't change much
- ❌ You're in a rush to ship
- ❌ Only you work on this code
- ❌ No time for refactoring

**My Recommendation:** **YES, DO IT!** 

The code is already large and will only grow. Better to refactor now than when it's 3,000+ lines!

---

**Want me to start the refactoring?** Just say "เริ่มเลย!" and I'll begin! 🚀
