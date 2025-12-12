# XS Rust 2D Game Engine — Development Guidelines

This document captures project-specific knowledge to speed up advanced development on this workspace. It is intentionally concise and focused on non-obvious, repo-specific details.

## Workspace and Build

- This is a Cargo workspace with resolver = 2. The default member is `engine` (the runnable app/editor), while most crates are libraries:
  - Core libs: `engine_core`, `ecs`, `render`, `physics`, `script`, `input`, `editor`
  - App: `engine` (selected by `[workspace] default-members`)

- Windows-optimized dev profile in root `Cargo.toml`:
  - `split-debuginfo = "unpacked"` → faster linking on Windows
  - `incremental = true`, `opt-level = 1` for a good dev-time/run-time tradeoff
  - Release uses `lto = "thin"`, `codegen-units = 1`, `strip = true`

- Build targets:
  - Workspace-wide build: `cargo build`
  - Specific crate build: `cargo build -p <crate>` (e.g., `-p ecs`)
  - Run the engine app: `cargo run -p engine`

- Feature flags: Some optional functionality (e.g., Rapier, clipboard) may need features enabled per hints found in `build_errors.txt`. If you see missing-symbol errors referencing rapier/clipboard, either enable or add the feature in the relevant crate’s `Cargo.toml` and propagate via the workspace as needed.

## Testing

Because `engine` and `editor` depend on graphics/OS integrations, fast tests should target library crates (prefer `engine_core`; `ecs` is larger and currently contains multiple internal `mod tests` blocks that can cause conflicts).

- Run tests for a single crate:
  - `cargo test -p engine_core` (recommended quick target)
  - `cargo test -p ecs` (heavier; may fail if internal test modules conflict)

- Run tests workspace-wide:
  - `cargo test` (may be slower due to heavier crates)

- Add a unit test inside a module file (example skeleton):
  ```rust
  // Inside some src/*.rs file in a library crate
  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn it_works() {
          assert_eq!(2 + 2, 4);
      }
  }
  ```

- Add an integration test without touching source (preferred for quick experiments):
  - Create `tests/smoke_test.rs` in the target crate directory.
  - Example that compiles against this repo (validated on `engine_core`):
    ```rust
    // engine_core/tests/smoke_test.rs
    use engine_core::{EngineContext, EngineModule};
    use anyhow::Result;

    struct DummyModule;

    impl EngineModule for DummyModule {
        fn name(&self) -> &str { "dummy" }
        fn on_load(&mut self, _ctx: &mut EngineContext) -> Result<()> { Ok(()) }
        fn on_update(&mut self, ctx: &mut EngineContext, _dt: f32) { ctx.should_quit = false; }
        fn on_unload(&mut self, _ctx: &mut EngineContext) {}
        fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    }

    #[test]
    fn can_register_and_update_module() {
        let mut ctx = EngineContext::new();
        ctx.register_module(DummyModule);
        let before = ctx.modules.len();
        ctx.update(1.0 / 60.0);
        let after = ctx.modules.len();
        assert_eq!(before, after);
    }
    ```
  - Run it: `cargo test -p engine_core`

Notes:
- The `ecs` crate exposes a minimal API suitable for deterministic tests (`World`, entities, basic components). Prefer it for CI-fast checks.
- If adding property tests, `ecs` already includes `proptest` in `[dev-dependencies]` and `criterion` for benches (see `[[bench]]`).

## Benchmarks (ecs)

- Criterion is configured for `ecs` with HTML reports:
  - Run: `cargo bench -p ecs`
  - Reports are generated in `target/criterion`; open the HTML files to inspect results.

## Code Style and Conventions

- Follow existing module layout and naming. High-level pattern:
  - `engine` orchestrates runtime/editor; UI is under `engine/src/editor/ui/...`
  - `ecs` defines `World`, components, serialization helpers, and prefab utilities.
  - Renderer/physics/script/input are separated crates to keep compile times manageable.

- Keep comments sparse and practical, mirroring current code. Favor descriptive enums and method names over extensive inline commentary.

- Serialization: `serde` is used across the workspace. When extending scene or component data, ensure `serde` derives are added consistently and migrations exist if JSON compatibility is needed (see `World::save_to_json`/`load_from_json`).

## Debugging and Editor Tips

- Editor/Scene View:
  - The scene view is modularized (`engine/src/editor/ui/scene_view/{types,rendering,interaction,toolbar,shortcuts}`) with a single entry `render_scene_view`. When altering interaction or projection logic, keep `SceneViewMode`, `SceneProjectionMode`, and `TransformSpace` flows in sync with `SceneCamera`.

- Runtime data:
  - For LDtk/Tiled flows, consult `engine/src/runtime` and `ecs/src/loaders`. When changing asset pipelines, validate entity parenting and component migration paths.

- Windows specifics:
  - Large link times are mitigated via `split-debuginfo = "unpacked"`. If you see very slow links, confirm you’re building in dev profile and that incremental builds are enabled.

## Verified Test Example (what we ran)

To prove the testing instructions above, we temporarily added `engine_core/tests/smoke_test.rs` with the exact contents shown in the Integration Test example and executed:

- `cargo test -p engine_core`

The test passed locally. The temporary test file was then removed to keep the repository clean, as requested.
