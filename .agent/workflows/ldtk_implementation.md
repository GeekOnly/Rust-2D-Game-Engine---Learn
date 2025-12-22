---
description: Workflow for implementing and verifying LDtk integration changes
---

1. **Review Plan**: 
   - Check `MD/LDTK_IMPROVEMENT_PLAN.md` for current objectives.

2. **Implement Core Logic (ECS)**:
   - Update `ecs/src/components/ldtk_map.rs` for data structures.
   - Update `ecs/src/loaders/ldtk_loader.rs` for loading logic.
   - Ensure `LdtkJson` alias and fields (like `file_path`, `auto_reload`) are preserved.

3. **Update Editor Integration**:
   - **Maps Panel**: Update `editor/src/ui/panels/maps_panel.rs` to match new `LdtkMap`/`LdtkJson` fields.
   - **Loader Calls**: Search for `LdtkLoader::` usages in `editor/` (e.g., `map_manager.rs`, `map_inspector.rs`, `map_view.rs`) and update arguments.
   - **Colliders**: check `editor/src/systems/generators/collider_generator.rs`.

4. **Verify Compilation**:
   - Run `cargo check -p editor`.
   - Address errors in the `editor` crate caused by `ecs` changes.

5. **Runtime Validation**:
   - Run `cargo run -p editor`.
   - Open `Celeste Demo` project.
   - Open "Maps" panel and verify map details.
   - Test "Generate Colliders" button.

6. **Documentation**:
   - Update `MD/LDTK_IMPROVEMENT_PLAN.md` marking tasks as complete.
