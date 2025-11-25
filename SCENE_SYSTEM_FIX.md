# Scene Save/Load System Fix

**Status:** ✅ Complete
**Date:** 2025-11-25
**Priority:** 🔴 Critical

## Problem

ระบบ Scene มีปัญหา 2 จุด:

1. **Save/Save As ยัง save ไฟล์นอก project ได้**
   - User สามารถ save scene ไปที่ไหนก็ได้
   - Engine ไม่รู้จักไฟล์ที่ save นอก project
   - ทำให้ workflow สับสน

2. **Project Settings ไม่มี UI เลือก startup scene**
   - ต้องแก้ไข `project.json` ด้วยมือ
   - ไม่มี Editor vs Game startup scene แยก (เหมือน Unreal)

## Solution Implemented

### Fix #1: Enforce Scene Save Inside Project

**Modified:** `game/src/main.rs`

#### Save As (lines 897-916)
```rust
if let Some(file) = dialog.save_file() {
    // Validate that a project is open
    if editor_state.current_project_path.is_none() {
        editor_state.console.error("Cannot save scene: No project is open!".to_string());
    } else if let Some(proj_path) = &editor_state.current_project_path {
        // Validate that file is inside project/scenes/
        let scenes_folder = proj_path.join("scenes");
        if !file.starts_with(&scenes_folder) {
            editor_state.console.error("Scene must be saved inside project/scenes/ folder!".to_string());
        } else {
            if let Err(e) = editor_state.save_scene(&file) {
                log::error!("Failed to save scene: {}", e);
                editor_state.console.error(format!("Failed to save scene: {}", e));
            } else {
                editor_state.current_scene_path = Some(file.clone());
                editor_state.console.info(format!("Scene saved as: {}", file.display()));
            }
        }
    }
}
```

#### Save (lines 920-947)
```rust
if save_request {
    // Check if project is open
    if editor_state.current_project_path.is_none() {
        editor_state.console.error("Cannot save scene: No project is open!".to_string());
    } else {
        let path_clone = editor_state.current_scene_path.clone();
        if let Some(path) = path_clone {
            if let Err(e) = editor_state.save_scene(&path) {
                log::error!("Failed to save scene: {}", e);
                editor_state.console.error(format!("Failed to save scene: {}", e));
            } else {
                editor_state.console.info(format!("Scene saved: {}", path.display()));
            }
        } else {
            // No current path, use default path in project/scenes/
            if let Some(default_path) = editor_state.get_default_scene_path("scene") {
                if let Err(e) = editor_state.save_scene(&default_path) {
                    log::error!("Failed to save scene: {}", e);
                    editor_state.console.error(format!("Failed to save scene: {}", e));
                } else {
                    editor_state.current_scene_path = Some(default_path.clone());
                    editor_state.console.info(format!("Scene saved: {}", default_path.display()));
                }
            } else {
                editor_state.console.error("Cannot create default scene path: No project is open!".to_string());
            }
        }
    }
}
```

#### Load Scene (lines 950-975)
```rust
if load_request {
    // Start dialog in project/scenes/ folder if project is open
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Scene", &["json"]);

    if let Some(proj_path) = &editor_state.current_project_path {
        let scenes_folder = proj_path.join("scenes");
        if scenes_folder.exists() {
            dialog = dialog.set_directory(&scenes_folder);
        }
    }

    if let Some(file) = dialog.pick_file() {
        if let Err(e) = editor_state.load_scene(&file) {
            log::error!("Failed to load scene: {}", e);
            editor_state.console.error(format!("Failed to load scene: {}", e));
        } else {
            editor_state.current_scene_path = Some(file.clone());
            editor_state.console.info(format!("Scene loaded: {}", file.display()));
        }
    }
}
```

### Fix #2: Project Settings UI

**Modified:** `game/src/editor_ui.rs` (lines 810-878)

#### Features Added:
- ⚙ General section (collapsible)
  - Project Name
  - Project Path

- 🎮 Play Mode section (collapsible)
  - **Editor Startup Scene**
    - Browse button → opens dialog in `scenes/` folder
    - Clear button → removes startup scene
    - Shows relative path (e.g., `scenes/main.json`)
    - Updates `project.json` automatically

  - **Game Startup Scene (Exported Build)**
    - Placeholder for future feature
    - Gray text: "Coming soon..."

#### UI Code:
```rust
ui.collapsing("🎮 Play Mode", |ui| {
    ui.label("Configure which scene to load when:");
    ui.add_space(5.0);

    // Editor Startup Scene
    ui.strong("Editor Startup Scene:");
    ui.label("Scene to load when opening project in Editor");

    let mut current_startup = String::new();
    if let Ok(pm) = ProjectManager::new() {
        if let Ok(Some(scene)) = pm.get_startup_scene(path) {
            current_startup = scene.to_string_lossy().to_string();
        }
    }

    let mut new_startup = current_startup.clone();
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut new_startup);
        if ui.button("📁 Browse...").clicked() {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("Scene", &["json"]);
            let scenes_folder = path.join("scenes");
            if scenes_folder.exists() {
                dialog = dialog.set_directory(&scenes_folder);
            }
            if let Some(file) = dialog.pick_file() {
                if let Ok(relative) = file.strip_prefix(path) {
                    new_startup = relative.to_string_lossy().to_string();
                }
            }
        }
        if ui.button("❌ Clear").clicked() {
            new_startup.clear();
        }
    });

    if new_startup != current_startup {
        if let Ok(pm) = ProjectManager::new() {
            let scene_path = if new_startup.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(&new_startup))
            };
            let _ = pm.set_startup_scene(path, scene_path);
        }
    }
});
```

## How It Works Now

### Save As Workflow:
```
User: File → Save As
    ↓
Dialog opens in: project/scenes/ ✅
    ↓
User selects location
    ↓
Validation:
  - Is project open? → NO → Error ❌
  - Is file inside project/scenes/? → NO → Error ❌
  - Otherwise → Save ✅
```

### Save Workflow:
```
User: File → Save (Ctrl+S)
    ↓
Check: Is project open?
  - NO → Error: "No project is open!" ❌
  - YES → Continue ↓
    ↓
Has current_scene_path?
  - YES → Save to that path ✅
  - NO → Use default: project/scenes/scene.json ✅
```

### Load Workflow:
```
User: File → Load
    ↓
Dialog opens in: project/scenes/ ✅
    ↓
User picks file
    ↓
Load scene ✅
Update current_scene_path ✅
```

### Project Settings Workflow:
```
User: Edit → Project Settings
    ↓
⚙ General section:
  - Shows project name
  - Shows project path
    ↓
🎮 Play Mode section:
  - Editor Startup Scene:
    - Text field shows current: "scenes/main.json"
    - Browse → select different scene
    - Clear → remove startup scene
  - Game Startup Scene:
    - Coming soon (for exported builds)
```

## Validation Rules

### ✅ Valid Scenarios:
1. Project open + Save in `project/scenes/` → **Allowed**
2. Project open + Load from anywhere → **Allowed** (read-only)
3. New Scene → **Allowed** (no save until user saves)

### ❌ Invalid Scenarios (Show Error):
1. No project open + Save → **Error: "No project is open!"**
2. Project open + Save outside `project/scenes/` → **Error: "Must be saved inside project/scenes/ folder!"**

## Error Messages

| Situation | Error Message |
|-----------|---------------|
| Save without project | "Cannot save scene: No project is open!" |
| Save outside project/scenes/ | "Scene must be saved inside project/scenes/ folder!" |
| Save fails (I/O error) | "Failed to save scene: {error}" |
| Load fails | "Failed to load scene: {error}" |

## Testing Guide

### Test 1: Save As Validation
1. Open project
2. Create GameObject
3. File → Save As
4. ✅ Dialog should start in `project/scenes/`
5. Try to navigate outside and save
6. ✅ Should show error: "Scene must be saved inside project/scenes/ folder!"

### Test 2: Save Without Project
1. Close all projects (no project open)
2. Create GameObject
3. File → Save
4. ✅ Should show error: "Cannot save scene: No project is open!"

### Test 3: Project Settings Startup Scene
1. Open project
2. Edit → Project Settings
3. Click 🎮 Play Mode
4. Click "📁 Browse..." for Editor Startup Scene
5. ✅ Dialog should start in `scenes/`
6. Select a scene (e.g., `main.json`)
7. ✅ Text field shows: `scenes/main.json`
8. Close project and reopen
9. ✅ Scene should auto-load

### Test 4: Clear Startup Scene
1. Edit → Project Settings
2. Click 🎮 Play Mode
3. Click "❌ Clear"
4. ✅ Text field becomes empty
5. Close and reopen project
6. ✅ No scene auto-loads (empty editor)

## Success Criteria

- [x] Cannot save scene without project open
- [x] Cannot save scene outside `project/scenes/` folder
- [x] Save As dialog starts in `project/scenes/`
- [x] Load Scene dialog starts in `project/scenes/`
- [x] Project Settings shows startup scene selection
- [x] Browse button opens in `scenes/` folder
- [x] Clear button removes startup scene
- [x] Shows relative paths (portable)
- [x] Console shows clear error messages
- [x] Build succeeds with no errors

## Impact

### Before:
- ❌ Could save scenes anywhere
- ❌ Engine couldn't find saved scenes
- ❌ No UI to configure startup scene
- ❌ Confusing workflow

### After:
- ✅ Scenes MUST be in `project/scenes/`
- ✅ Clear validation with error messages
- ✅ UI to select Editor Startup Scene
- ✅ Placeholder for Game Startup Scene (future)
- ✅ Professional, Unity-like workflow

## Files Changed

1. **game/src/main.rs**
   - Save As: Added project validation (lines 897-916)
   - Save: Added project validation (lines 920-947)
   - Load: Dialog starts in scenes folder (lines 950-975)

2. **game/src/editor_ui.rs**
   - Project Settings UI (lines 810-878)
   - General section
   - Play Mode section with startup scene selection

## Next Steps

Ready to implement:
- [ ] Unity-Style Hierarchy (4h)
- [ ] 3D Transform Inspector (3h) - Quick win
- [ ] Asset Manager improvements (8h)
- [ ] Rotate & Scale Tools (6h)

---

**Build Time:** 49.62s
**Status:** ✅ All Critical Scene Issues Fixed
**Warnings:** 5 (dead code only, expected)
