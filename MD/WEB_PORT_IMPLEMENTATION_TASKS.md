# Web Porting Implementation Tasks (Phase 2 & 3)

This document tracks the specific implementation steps required to verify the Web Porting Plan.

## 📦 Phase 2.1: Integrate AssetLoader (The "Async" Refactor)

The goal is to replace all direct `std::fs` calls in the engine with `AssetLoader` traits.

### Core Engine Updates
- [x] **Inject AssetLoader into EngineContext**
    - Modify `EngineContext` in `engine_core/src/lib.rs` to hold `pub asset_loader: Arc<dyn AssetLoader>`.
    - Update `engine/src/lib.rs` (or main entry point) to initialize the correct loader (`NativeAssetLoader` vs `WebAssetLoader`) and pass it to context.

### Subsystem Refactoring

#### 1. Texture Manager 🖼️
- [x] **Refactor `load_texture` to support `Vec<u8>` loading**
    - **Status:** Implemented `load_texture_from_bytes` in both `engine` and `render` crates.
    - **Current:** `load_texture(path: &str) -> AssetId` (Synchronous wrapper around bytes loader)
    - **New:** `load_texture(loader: &dyn AssetLoader, path: &str) -> AssetId` (returns placeholder immediately, loads in background?) OR `async fn load_texture(...) -> Result<AssetId>`
    - **Strategy:**
        1. Keep `load_texture` returning an `AssetId` immediately so we don't break ECS flow.
        2. Spawn the async load task.
        3. Use a "Pink Texture" (placeholder) until the real data arrives.
        4. When `AssetLoader::load_binary` returns `Vec<u8>`, use `image::load_from_memory` instead of `image::open`.
    - **Status Check:** `load_texture_from_bytes` is implemented. Editor logic uses `asset_loader.load_binary` -> `texture_manager.load_texture_from_bytes`.

#### 2. Scene Loader 🎬
- [x] **Refactor `SceneLoader` / `SceneManager`**
    - **Target File:** `editor/src/states.rs` (EditorState).
    - **Action:**
        - Change `load_scene_from_file(path)` to use `loader.load_text(path).await`.
        - Parse the returned JSON string.
    - **Status:** Done. `EditorState::load_scene` uses `asset_loader` to read the scene JSON.

#### 3. GLTF / Model Loader 🧊
- [x] **Update `gltf_loader.rs`**
    - GLTF loading is complex because `.gltf` files reference external `.bin` and texture files.
    - **Challenge:** The `gltf` crate typically expects a file path.
    - **Solution:** We might need to read the `.gltf` JSON manually or use the `gltf::import_slice` API if we load the `.glb` (binary) completely into memory first.
    - **Task:** Switch to loading `.glb` (single file) via `AssetLoader::load_binary` and use `gltf::import_slice`.
    - **Status:** Done. `XsgImporter` and `GltfLoader` refactored to use `AssetLoader`.

#### 4. Script Engine (Lua) 📜
- [x] **Update Lua Script Loading**
    - **Target:** `script/src/lib.rs` (`ScriptEngine`), `engine/src/runtime/script_loader.rs`.
    - **Action:** Instead of `std::fs`, use `loader.load_text(path)` to get the script content.
    - **Implementation Details:**
        - `ScriptEngine` now stores `Arc<dyn AssetLoader>`.
        - `ScriptEngine` registers a custom `package.searcher` for Lua `require` calls, which uses `pollster::block_on(asset_loader.load_text(...))`.
        - `load_all_scripts` refactored to use `asset_loader`.
        - `PlayModeSystem` now uses `ScriptEngine`'s stored loader/state instead of filesystem checks for collision callbacks.

---

## 🌐 Phase 3: Web Entry Point & Build

### Winit & Event Loop
- [ ] **Create `lib_wasm.rs` or similar entry point**
    - `#[wasm_bindgen(start)]`
    - Initialize `console_error_panic_hook`.
    - Create `WebAssetLoader` with base URL (e.g., `./assets`).
    - Setup `winit` event loop (using `winit::platform::web::EventLoopExtWebSys` if needed, or just standard 0.29+ winit usage).

### Rendering (WGPU)
- [ ] **Canvas Binding**
    - Ensure `wgpu::Surface` is created from the HTML Canvas element correctly.
    - Handle window resize events from the browser.

---

## 📱 Android/Native Validation
- [ ] **Verify Native Build Still Works**
    - Ensure `NativeAssetLoader` correctly resolves paths on Windows.
    - Ensure `std::fs` fallback works 100%.

## 📝 Next Immediate Actions
1. **Pass `Arc<dyn AssetLoader>`** into the `Engine` struct.
2. **Refactor `TextureManager`** to accept `Vec<u8>` or use the loader.
