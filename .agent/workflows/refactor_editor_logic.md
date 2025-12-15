---
description: Plan for refactoring Editor logic from Engine crate to Editor crate
---

# Editor Refactoring Plan

The goal of this refactor is to decouple the Editor logic from the core Engine runtime. This will allow for cleaner builds, better separation of concerns, and a lightweight runtime for exported games.

## Phase 1: Preparation & Dependencies

1.  **Analyze Dependencies:**
    *   Identify all dependencies in `engine/Cargo.toml` that are strictly for the Editor (e.g., `egui`, `rfd`, `notify`).
    *   Update `editor/Cargo.toml` to include these dependencies.
    *   Ensure `editor` crate depends on `engine` crate (presumably with a `"lib"` feature or public API exposure).

2.  **Expose Engine Internals (Temporary):**
    *   Since the Editor needs deep access to Engine structures (World, Resources, Systems), we may need to make certain private fields/methods in `engine` public or add accessors.
    *   *Action:* Review `engine/src/lib.rs` and module visibilities.

## Phase 2: Migration of Core Editor Modules

We will move modules one by one or in related groups. For each group:
    a. Move the file(s) from `engine/src/editor/...` to `editor/src/...`.
    b. Fix imports in the moved files (change `crate::...` to `engine::...`).
    c. Re-export the module in `editor/src/lib.rs` (if public).
    d. Verify `engine` crate still compiles (by removing the old module usage or gating it).

**Order of Migration:**

1.  **Independent Utilities:**
    *   `shortcuts.rs`
    *   `theme.rs`
    *   `clipboard.rs` (if not dependent on engine internals heavily)
    *   `undo.rs`

2.  **UI Components (The heavy lifting):**
    *   `ui/` directory (Inspector, Hierarchy, Console, Asset Browser, etc.). This is the biggest chunk.
    *   `console.rs`
    *   `toolbar.rs`

3.  **Scene Interaction & Rendering:**
    *   `camera.rs` (SceneCamera)
    *   `grid.rs`
    *   `selection.rs`
    *   `snapping.rs`
    *   `rendering_3d.rs` (Gizmos might be tricky if heavily coupled with wgpu/renderer).

4.  **Editor State & Management:**
    *   `states.rs` (EditorState) - *Critical*: heavily couples everything.
    *   `asset_manager.rs`
    *   `map_manager.rs`
    *   `prefab.rs`
    *   `widget_editor.rs`

## Phase 3: Application Entry Point Restructuring

1.  **Create Editor Entry Point:**
    *   Currently, `engine/src/main.rs` likely acts as both the Game Runtime and the Editor Launcher based on args or config.
    *   *Goal:* Create a new binary target in `editor` (e.g., `editor/src/bin/editor.rs`) or make the `editor` crate the primary executable for development.
    *   Refactor `engine/src/main.rs` to purely start the *Game Runtime* (or keep it as a "Launcher" that decides which to load, but delegates logic to the crates).

2.  **Feature Gating in Engine:**
    *   Add an `editor` feature flag to `engine/Cargo.toml`.
    *   Gate any remaining editor-support code in `engine` behind `#[cfg(feature = "editor")]`.
    *   This ensures `cargo build --release --bin player` (runtime) doesn't include editor bloat.

## Phase 4: Cleanup & Verification

1.  **Fix Warnings:** Clean up the unused imports resulting from the move.
2.  **Testing:**
    *   Verify Editor UI still loads.
    *   Verify Scene Save/Load works.
    *   Verify Play Mode (Hot Reloading/Scripting) still works.
3.  **Documentation:** Update README/docs on how to build the Editor vs. the Game Runtime.

## Immediate First Steps (Active Checklist)

- [ ] Update `editor/Cargo.toml` with necessary dependencies from `engine`.
- [ ] Move `theme.rs` and `shortcuts.rs` to `editor/src` as a trial run.
- [ ] Establish `editor` depending on `engine` and ensure cross-crate access works.
