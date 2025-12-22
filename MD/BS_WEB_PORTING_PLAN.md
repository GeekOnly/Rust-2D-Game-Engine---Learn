# Rust Game Engine Web Porting Plan
**Project:** Rust 2D Game Engine (Port to WebAssembly/React)
**Date:** December 22, 2025
**Author:** Antigravity (Assistant)

## 1. Executive Summary

This document outlines the strategic roadmap for porting the existing Rust 2D Game Engine to WebAssembly (WASM) to enable integration with modern web applications (specifically React.js). The goal is to allow the engine to run natively in a browser environment with performance comparable to the desktop application, utilizing WebGPU/WebGL via the WGPU backend.

The primary challenge identified is the lack of a direct file system (`std::fs`) in the browser environment, necessitating a significant refactoring of the Asset Loading system.

## 2. Technical Goals

- **WASM Compatibility:** Compile the `engine` crate to a `.wasm` binary that runs without panics in a web browser.
- **React Integration:** Create a seamless bridge between the Rust Engine and React Components, allowing standard Web Developers to embed the game engine easily.
- **Async Asset Loading:** Transition from synchronous `std::fs` operations to an asynchronous, trait-based `AssetLoader` capable of `fetch` (HTTP) requests.
- **Performance:** Maintain 60 FPS performance on standard web browsers using WebGL2 or WebGPU.

---

## 3. Detailed Roadmap

### Phase 1: Environment & Toolchain Setup
**Objective:** Prepare the Rust development environment for WebAssembly targets.

1. **Install Build Tools**
   - Install `wasm-pack`: `cargo install wasm-pack`
   - Add WASM target: `rustup target add wasm32-unknown-unknown`

2. **Project Configuration (`Cargo.toml`)**
   - Modify `engine/Cargo.toml` to support the `cdylib` crate type (required for shared WASM libraries).
   - Add platform-specific dependencies:
     ```toml
     [target.'cfg(target_arch = "wasm32")'.dependencies]
     wasm-bindgen = "0.2"
     web-sys = { version = "0.3", features = ["Window", "Document", "Element", "HtmlCanvasElement", "Request", "Response"] }
     wasm-bindgen-futures = "0.4"
     console_error_panic_hook = "0.1"
     ```

### Phase 2: Core Refactoring - Async Asset System
**Objective:** Decouple the engine from the OS File System. This is the **most critical** phase.

1. **Define `AssetLoader` Trait**
   - Create a new trait in `engine_core` or `engine::assets`:
     ```rust
     #[async_trait]
     pub trait AssetLoader: Send + Sync {
         async fn load_text(&self, path: &str) -> anyhow::Result<String>;
         async fn load_binary(&self, path: &str) -> anyhow::Result<Vec<u8>>;
         fn get_base_path(&self) -> String;
     }
     ```

2. **Implement Platform-Specific Loaders**
   - **`NativeAssetLoader` (Desktop):** Wraps `std::fs::read_to_string` and `std::fs::read`.
   - **`WebAssetLoader` (Web):** Uses `web_sys::window().fetch_with_str()` and converts JS Promises to Rust Futures. Use `reqwest` (wasm-compatible) or raw `web-sys` calls.

3. **Refactor Subsystems**
   - **`SceneLoader`:** Update `load_from_json` to accept generic content strings loaded via the `AssetLoader`, rather than taking file paths directly.
   - **`TextureManager`:** Update image loading to handle raw byte arrays (`Vec<u8>`) instead of paths, as the texture must be decoded from bytes fetched over HTTP.
   - **`ScriptEngine`:** Update Lua script loading to `await` the script content before execution.

### Phase 3: Web Entry Point & Lifecycle
**Objective:** Create the "Front Door" for the web browser to launch the engine.

1. **Create Web Entry Point (`lib.rs` specific to WASM context)**
   - Implement a function tagged with `#[wasm_bindgen]` exposed to JS.
   - **Signature:** `pub async fn run_web(canvas_id: String, project_root: String)`.
   - **Logic:**
     1. Initialize panic hooks (for debugging in Console).
     2. Locate the HTML Canvas element by ID.
     3. Initialize WGPU/Winit with web specific surface configuration.
     4. Instantiate `WebAssetLoader` pointing to the `project_root` URL.
     5. Start the Event Loop (Note: Winit's `EventLoop::run` on web returns immediately or throws exception to unwind stack; careful handling required).

2. **Event Loop Adaptation**
   - Winit requires special handling on the web (`generic_event_loop` crate or `winit`'s own web extensions) to prevent blocking the browser's UI thread.

### Phase 4: React Client Integration
**Objective:** Create the user-facing web application.

1. **Project Scaffold**
   - Initialize a Vite project: `npm create vite@latest game-client -- --template react-ts`

2. **WASM Integration**
   - Add the compiled WASM package to `package.json` or link locally.
   - Create a React Hook/Component `useGameEngine`:
     ```typescript
     // Pseudocode
     useEffect(() => {
       initWasm().then(() => {
         run_web("canvas-id", "/assets");
       });
     }, []);
     ```

3. **Asset Deployment**
   - Configure the build pipeline to copy the game's `assets` folder to the React `public/` directory so they are accessible via HTTP fetch.

### Phase 5: Testing & Optimization
**Objective:** Ensure stability and performance.

1. **Validation Steps**
   - Test basic Scene Loading (JSON).
   - Test Texture rendering (PNG/JPG decoding).
   - Test Input mapping (Touch/Mouse events).

2. **Optimization**
   - **Binary Size:** Enable LTO (Link Time Optimization) and `opt-level = "z"` (size) for WASM builds to reduce download time.
   - **Asset Compression:** Ensure assets served are compressed (gzip/brotli) by the web server.

---

## 4. Risks and Mitigation

| Risk | Impact | Mitigation |
|:---|:---|:---|
| **`std::fs` Dependency Deep in Dependencies** | Blocking | Use `console_error_panic_hook` to identify exact crash locations. Replace incompatible crates or put them behind `cfg(not(target_arch = "wasm32"))`. |
| **Winit Event Loop Differences** | High | Use the latest pattern recommended by `winit` documentation for `requestAnimationFrame` integration. |
| **CORS (Cross-Origin) Issues** | Medium | Ensure local dev server (Vite) serves headers correctly. Assets must be on the same domain or have proper CORS headers. |

## 5. Conclusion

The current architecture of the XSGameStudio Engine is well-suited for this port. The clear separation of the `render` module using WGPU is the strongest asset for this transition. By addressing the File I/O abstraction layer systematically, the engine will gain powerful cross-platform capabilities, opening the door to instant web-based game demos and distribution.
