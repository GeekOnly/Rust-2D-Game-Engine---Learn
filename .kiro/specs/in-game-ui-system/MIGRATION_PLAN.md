# UI System Migration Plan

## Overview

This document outlines the complete migration plan from the legacy HUD system (`engine/src/hud` + `engine/src/editor/widget_editor`) to the new comprehensive UI system (`ui` crate).

## Current State Analysis

### Legacy Systems

#### 1. `engine/src/hud` - Runtime HUD System
**Purpose:** Display in-game HUD elements (health bars, minimap, score, etc.)

**Components:**
- `hud_asset.rs` - HudAsset, HudElement, HudElementType definitions
- `hud_manager.rs` - HudManager for state management and data binding
- `hud_renderer.rs` - Rendering using egui

**Limitations:**
- ❌ Simple element types only (HealthBar, ProgressBar, Text, Minimap, Image, Container)
- ❌ No interaction/events (read-only display)
- ❌ No layout system (manual positioning)
- ❌ No animation support
- ❌ egui-based (not integrated with game rendering)
- ❌ Limited to HUD use cases

**File Format:** `.hud` JSON files

#### 2. `engine/src/editor/widget_editor` - Visual HUD Editor
**Purpose:** Visual editor for creating/editing HUD files

**Components:**
- `mod.rs` - Main WidgetEditor
- `canvas.rs` - Visual canvas with grid and safe area
- `properties.rs` - Properties panel for editing
- `state.rs` - Editor state management

**Features:**
- ✅ Visual drag-and-drop editing
- ✅ Grid and safe area display
- ✅ Element selection and manipulation
- ✅ Multiple resolution preview
- ⚠️ Limited property editing (mostly read-only)
- ⚠️ Works only with HudAsset format

### New System

#### `ui` crate - Complete UI System
**Purpose:** Production-ready UI system comparable to Unity Canvas UI / Unreal UMG

**Advantages:**
- ✅ ECS-based architecture
- ✅ RectTransform with flexible anchoring
- ✅ Comprehensive component library (15+ types)
- ✅ Layout system (Horizontal/Vertical/Grid)
- ✅ Event system (click, hover, drag, scroll)
- ✅ Animation system with easing
- ✅ Advanced components (Button, Slider, Toggle, Dropdown, InputField, ScrollView)
- ✅ 9-slice sprite support
- ✅ Masking and clipping
- ✅ Canvas scaler for resolution independence
- ✅ Prefab system
- ✅ Style and theme support
- ✅ Lua scripting integration
- ✅ WGPU-based rendering with batching

**File Format:** `.uiprefab` JSON files (UIPrefab)

---

## Migration Strategy

### Phase 1: Foundation (Week 1-2)
**Goal:** Complete core UI system implementation

**Tasks:**
1. ✅ Set up UI crate structure and core types (COMPLETED)
2. ⬜ Implement RectTransform system with anchoring
3. ⬜ Implement Canvas and CanvasScaler
4. ⬜ Implement UIElement hierarchy system
5. ⬜ Implement core UI components (Image, Text, Button, Panel)
6. ⬜ Implement layout system
7. ⬜ Implement event system
8. ⬜ Implement rendering pipeline integration

**Deliverables:**
- Functional UI system with basic components
- Integration with ECS and rendering pipeline
- Basic examples working

### Phase 2: Advanced Features (Week 3-4)
**Goal:** Implement advanced UI features

**Tasks:**
1. ⬜ Implement animation system
2. ⬜ Implement scroll view system
3. ⬜ Implement advanced components (Slider, Toggle, Dropdown, InputField)
4. ⬜ Implement masking system
5. ⬜ Implement prefab system
6. ⬜ Implement style and theme system
7. ⬜ Implement Lua bindings

**Deliverables:**
- Complete UI component library
- Prefab system for reusable UI
- Lua API for runtime UI creation

### Phase 3: Migration Tools (Week 5)
**Goal:** Create tools to migrate existing HUD files

**Tasks:**
1. ⬜ Create HUD → UIPrefab converter
2. ⬜ Create migration script for existing .hud files
3. ⬜ Create compatibility layer (optional, for gradual migration)
4. ⬜ Document migration process

**Deliverables:**
- Automated migration tool
- Migration guide
- Converted prefab files

### Phase 4: UI Prefab Editor (Week 6-8)
**Goal:** Create visual editor for UI prefabs

**Tasks:**
1. ⬜ Refactor widget_editor to work with UIPrefab
2. ⬜ Implement prefab loading/saving
3. ⬜ Implement component property editing
4. ⬜ Implement RectTransform visual editing
5. ⬜ Implement layout preview
6. ⬜ Implement component palette
7. ⬜ Implement hierarchy view
8. ⬜ Implement undo/redo system

**Deliverables:**
- Functional UI Prefab Editor
- Support for all UI components
- Visual editing of RectTransform and layouts

### Phase 5: Cleanup and Documentation (Week 9)
**Goal:** Remove legacy systems and finalize documentation

**Tasks:**
1. ⬜ Remove `engine/src/hud` module
2. ⬜ Update all references to use `ui` crate
3. ⬜ Update examples and tutorials
4. ⬜ Create comprehensive documentation
5. ⬜ Create video tutorials (optional)

**Deliverables:**
- Clean codebase without legacy systems
- Complete documentation
- Migration complete

---

## Detailed Migration Steps

### Step 1: HUD Asset → UI Prefab Conversion

#### Mapping Table

| HudElement | UIPrefab Equivalent | Notes |
|------------|---------------------|-------|
| `HudElement` | `UIPrefabElement` | Base element |
| `Anchor` | `RectTransform.anchor_min/max` | More flexible anchoring |
| `offset` | `RectTransform.anchored_position` | Position offset |
| `size` | `RectTransform.size_delta` | Size |
| `HudElementType::Text` | `UIText` | Direct mapping |
| `HudElementType::DynamicText` | `UIText` + Lua binding | Use Lua for dynamic updates |
| `HudElementType::HealthBar` | `UIImage` (9-slice) + `UIImage` (fill) | Two images: background + fill |
| `HudElementType::ProgressBar` | `UIImage` (9-slice) + `UIImage` (fill) | Same as HealthBar |
| `HudElementType::Minimap` | `UIPanel` + custom rendering | Custom component or render texture |
| `HudElementType::Image` | `UIImage` | Direct mapping |
| `HudElementType::Container` | Parent-child hierarchy | Use ECS hierarchy |

#### Anchor Conversion

```rust
// Old: HUD Anchor
Anchor::TopLeft → RectTransform {
    anchor_min: Vec2::new(0.0, 1.0),  // Top-left in Unity coordinates
    anchor_max: Vec2::new(0.0, 1.0),
    pivot: Vec2::new(0.0, 1.0),
    ...
}

Anchor::Center → RectTransform {
    anchor_min: Vec2::new(0.5, 0.5),
    anchor_max: Vec2::new(0.5, 0.5),
    pivot: Vec2::new(0.5, 0.5),
    ...
}

Anchor::BottomRight → RectTransform {
    anchor_min: Vec2::new(1.0, 0.0),
    anchor_max: Vec2::new(1.0, 0.0),
    pivot: Vec2::new(1.0, 0.0),
    ...
}
```

#### Data Binding Migration

**Old System:**
```rust
// HudManager with bindings
hud_manager.bind("player_health", |world| {
    // Get health from world
    0.75
});
```

**New System:**
```lua
-- Lua script updates UI
function update()
    local health = get_player_health()
    ui.set_fill_amount("health_bar", health)
end
```

### Step 2: Widget Editor → UI Prefab Editor

#### Architecture Changes

**Old Widget Editor:**
```
WidgetEditor
├── WidgetCanvas (renders HudElements)
├── PropertiesPanel (edits HudElement properties)
└── WidgetEditorState (manages HudAsset)
```

**New UI Prefab Editor:**
```
UIPrefabEditor
├── PrefabCanvas (renders UIPrefabElements with RectTransform)
├── ComponentPanel (add/remove components)
├── PropertiesPanel (edit component properties)
├── HierarchyPanel (tree view of elements)
└── PrefabEditorState (manages UIPrefab)
```

#### New Features to Add

1. **Component Palette**
   - Drag-and-drop components onto canvas
   - All UI component types available

2. **Hierarchy View**
   - Tree view of UI element hierarchy
   - Drag-and-drop to reparent
   - Show/hide elements

3. **RectTransform Editor**
   - Visual anchor editing
   - Pivot point manipulation
   - Size and position handles

4. **Layout Preview**
   - Preview different resolutions
   - Show layout group effects
   - Safe area visualization

5. **Component Inspector**
   - Edit all component properties
   - Color pickers
   - Sprite selectors
   - Event callback editors

### Step 3: File Format Migration

#### Converter Implementation

```rust
// Pseudo-code for converter
pub struct HudToUIPrefabConverter;

impl HudToUIPrefabConverter {
    pub fn convert(hud: HudAsset) -> UIPrefab {
        let mut prefab = UIPrefab {
            name: hud.name,
            root: Self::convert_element(&hud.elements[0]),
        };
        
        // Convert all elements
        for element in &hud.elements[1..] {
            // Add to hierarchy
        }
        
        prefab
    }
    
    fn convert_element(element: &HudElement) -> UIPrefabElement {
        UIPrefabElement {
            name: element.id.clone(),
            rect_transform: Self::convert_anchor_to_rect_transform(
                &element.anchor,
                element.offset,
                element.size,
            ),
            ui_element: UIElement::default(),
            image: Self::convert_image(&element.element_type),
            text: Self::convert_text(&element.element_type),
            // ... other components
            children: vec![],
        }
    }
    
    fn convert_anchor_to_rect_transform(
        anchor: &Anchor,
        offset: [f32; 2],
        size: [f32; 2],
    ) -> RectTransform {
        let (anchor_min, anchor_max, pivot) = match anchor {
            Anchor::TopLeft => (Vec2::new(0.0, 1.0), Vec2::new(0.0, 1.0), Vec2::new(0.0, 1.0)),
            Anchor::TopCenter => (Vec2::new(0.5, 1.0), Vec2::new(0.5, 1.0), Vec2::new(0.5, 1.0)),
            Anchor::TopRight => (Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0)),
            Anchor::CenterLeft => (Vec2::new(0.0, 0.5), Vec2::new(0.0, 0.5), Vec2::new(0.0, 0.5)),
            Anchor::Center => (Vec2::new(0.5, 0.5), Vec2::new(0.5, 0.5), Vec2::new(0.5, 0.5)),
            Anchor::CenterRight => (Vec2::new(1.0, 0.5), Vec2::new(1.0, 0.5), Vec2::new(1.0, 0.5)),
            Anchor::BottomLeft => (Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
            Anchor::BottomCenter => (Vec2::new(0.5, 0.0), Vec2::new(0.5, 0.0), Vec2::new(0.5, 0.0)),
            Anchor::BottomRight => (Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 0.0)),
        };
        
        RectTransform {
            anchor_min,
            anchor_max,
            pivot,
            anchored_position: Vec2::new(offset[0], offset[1]),
            size_delta: Vec2::new(size[0], size[1]),
            rotation: 0.0,
            scale: Vec2::ONE,
            world_corners: [Vec2::ZERO; 4],
            rect: Rect::default(),
            dirty: true,
        }
    }
}
```

#### Migration Script

```rust
// Migration script to convert all .hud files
use std::fs;
use std::path::Path;

fn migrate_all_hud_files(project_path: &Path) {
    let hud_files = find_hud_files(project_path);
    
    for hud_file in hud_files {
        println!("Migrating: {:?}", hud_file);
        
        // Load HUD
        let hud = HudAsset::load(hud_file.to_str().unwrap())
            .expect("Failed to load HUD");
        
        // Convert to UIPrefab
        let prefab = HudToUIPrefabConverter::convert(hud);
        
        // Save as .uiprefab
        let prefab_path = hud_file.with_extension("uiprefab");
        let json = serde_json::to_string_pretty(&prefab)
            .expect("Failed to serialize prefab");
        fs::write(&prefab_path, json)
            .expect("Failed to write prefab");
        
        println!("  → Created: {:?}", prefab_path);
    }
    
    println!("Migration complete!");
}

fn find_hud_files(path: &Path) -> Vec<std::path::PathBuf> {
    // Recursively find all .hud files
    // Implementation...
    vec![]
}
```

---

## Testing Strategy

### Unit Tests
- Test HUD → UIPrefab conversion
- Test anchor conversion
- Test component mapping

### Integration Tests
- Load converted prefabs
- Verify rendering matches original
- Test all component types

### Visual Regression Tests
- Screenshot comparison
- Before/after migration
- Multiple resolutions

---

## Rollback Plan

In case of issues during migration:

1. **Keep Legacy Code** (temporarily)
   - Don't delete `engine/src/hud` until migration is verified
   - Keep original .hud files as backup

2. **Feature Flag**
   ```rust
   #[cfg(feature = "legacy_hud")]
   mod hud;
   ```

3. **Gradual Migration**
   - Migrate one HUD file at a time
   - Test thoroughly before proceeding
   - Keep both systems running in parallel

---

## Success Criteria

Migration is considered complete when:

- ✅ All .hud files converted to .uiprefab
- ✅ UI Prefab Editor fully functional
- ✅ All UI features working (rendering, events, animations)
- ✅ Performance equal or better than legacy system
- ✅ Documentation complete
- ✅ Examples updated
- ✅ Legacy code removed
- ✅ No regressions in existing functionality

---

## Timeline

| Phase | Duration | Completion Date |
|-------|----------|-----------------|
| Phase 1: Foundation | 2 weeks | Week 2 |
| Phase 2: Advanced Features | 2 weeks | Week 4 |
| Phase 3: Migration Tools | 1 week | Week 5 |
| Phase 4: UI Prefab Editor | 3 weeks | Week 8 |
| Phase 5: Cleanup | 1 week | Week 9 |
| **Total** | **9 weeks** | **Week 9** |

---

## Resources

### Documentation
- Unity Canvas UI: https://docs.unity3d.com/Manual/UICanvas.html
- Unreal UMG: https://docs.unrealengine.com/en-US/umg-ui-designer/

### Code References
- Current HUD system: `engine/src/hud/`
- Current Widget Editor: `engine/src/editor/widget_editor/`
- New UI system: `ui/`

### Tools
- Migration converter: `tools/hud_to_prefab_converter.rs` (to be created)
- Migration script: `tools/migrate_all_huds.rs` (to be created)

---

## Notes

- This is a major refactoring that will take significant time
- Thorough testing is essential at each phase
- Keep communication open with team members
- Document any issues or blockers immediately
- Consider creating a migration branch for safety

---

## Questions & Decisions

### Open Questions
1. Should we support both systems during transition?
2. How to handle custom HUD rendering (e.g., minimap)?
3. What to do with Lua scripts that use old HUD API?

### Decisions Made
1. ✅ Full migration (not gradual)
2. ✅ New file format (.uiprefab)
3. ✅ Refactor widget_editor → UI Prefab Editor
4. ✅ Remove legacy code after migration

---

## Contact

For questions or issues during migration:
- Create issue in project tracker
- Tag with `migration` label
- Assign to UI system lead
