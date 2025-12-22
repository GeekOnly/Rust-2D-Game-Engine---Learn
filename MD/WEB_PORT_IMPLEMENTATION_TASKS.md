# Web Porting Implementation Tasks (Phase 2 & 3)

This document tracks the specific implementation steps required to verify the Web Porting Plan.

## 📦 Phase 2.1: Integrate AssetLoader (The "Async" Refactor)

The goal is to replace all direct `std::fs` calls in the engine with `AssetLoader` traits.

### Core Engine Updates
- [ ] **Inject AssetLoader into EngineContext**
    - Modify `EngineContext` in `engine_core/src/lib.rs` to hold `pub asset_loader: Arc<dyn AssetLoader>`.
    - Update `engine/src/lib.rs` (or main entry point) to initialize the correct loader (`NativeAssetLoader` vs `WebAssetLoader`) and pass it to context.

### Subsystem Refactoring

#### 1. Texture Manager 🖼️
- [ ] **Refactor `load_texture` to be Async**
    - **Current:** `load_texture(path: &str) -> AssetId` (Synchronous)
    - **New:** `load_texture(loader: &dyn AssetLoader, path: &str) -> AssetId` (returns placeholder immediately, loads in background?) OR `async fn load_texture(...) -> Result<AssetId>`
    - **Strategy:**
        1. Keep `load_texture` returning an `AssetId` immediately so we don't break ECS flow.
        2. Spawn the async load task.
        3. Use a "Pink Texture" (placeholder) until the real data arrives.
        4. When `AssetLoader::load_binary` returns `Vec<u8>`, use `image::load_from_memory` instead of `image::open`.

#### 2. Scene Loader 🎬
- [ ] **Refactor `SceneLoader` / `SceneManager`**
    - **Target File:** `engine/src/assets/manager.rs` or `runtime/scene_manager.rs` (check codebase).
    - **Action:**
        - Change `load_scene_from_file(path)` to use `loader.load_text(path).await`.
        - Parse the returned JSON string.

#### 3. GLTF / Model Loader 🧊
- [ ] **Update `gltf_loader.rs`**
    - GLTF loading is complex because `.gltf` files reference external `.bin` and texture files.
    - **Challenge:** The `gltf` crate typically expects a file path.
    - **Solution:** We might need to read the `.gltf` JSON manually or use the `gltf::import_slice` API if we load the `.glb` (binary) completely into memory first.
    - **Task:** Switch to loading `.glb` (single file) via `AssetLoader::load_binary` and use `gltf::import_slice`.

#### 4. Script Engine (Lua) 📜
- [ ] **Update Lua Script Loading**
    - **Target:** `script/src/lib.rs` or wherever `mlua` loads files.
    - **Action:** Instead of `lua.load(Path)`, use `loader.load_text(path).await` to get the script content, then `lua.load(&script_content).exec()`.

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
