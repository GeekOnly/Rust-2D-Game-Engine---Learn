# Web Assembly (WASM) & React Integration Plan

This plan outlines the steps required to compile the Rust Game Engine to WebAssembly (WASM) and integrate it with a React.js application.

## Phase 1: Environment & Dependency Setup

### 1.1 Install Prerequisites
- **Tool**: `wasm-pack` is required to build Rust into a standard NPM package.
- **Action**: Run `cargo install wasm-pack`.

### 1.2 Update Project Configuration (`engine/Cargo.toml`)
- **Goal**: Enable WASM compilation and include web-specific dependencies.
- **Changes**:
    - Set `crate-type` to `["cdylib", "rlib"]` (cdylib for WASM, rlib for Native).
    - Add `target.'cfg(target_arch = "wasm32")'.dependencies`:
        - `wasm-bindgen`: Interfacing with JS.
        - `web-sys`: Access to browser APIs (Canvas, Window, Fetch).
        - `console_error_panic_hook`: Friendly panic messages in browser console.
        - `wasm-bindgen-futures`: Utilizing Rust Futures in JS.
        - `js-sys`: Bindings for JS global objects.
    - Ensure `winit`, `wgpu`, and `image` dependencies have WASM feature flags enabled if necessary (usually `winit` handles this automatically, but `wgpu` needs `webgl` feature if WebGPU isn't assumed).

## Phase 2: Core Refactoring - Async Asset System

The most critical change is handling File I/O, as `std::fs` does not exist in the browser.

### 2.1 Abstract Asset Loading
- **Task**: Create an `AssetLoader` trait in `engine_core` or `engine::assets`.
- **Interface**:
  ```rust
  #[async_trait]
  pub trait AssetLoader {
      async fn load_text(&self, path: &str) -> anyhow::Result<String>;
      async fn load_binary(&self, path: &str) -> anyhow::Result<Vec<u8>>;
  }
  ```

### 2.2 Implement Native Loader (`native_loader.rs`)
- **Strategy**: Wrap `std::fs` calls.
- **Implementation**: Simple blocking I/O (wrapped in `async` block or using `tokio::fs` if we switch to async runtime later, but for now blocking in async wrapper is fine for `pollster`).

### 2.3 Implement Web Loader (`web_loader.rs`)
- **Strategy**: Use `web_sys::window().unwrap().fetch_with_str(path)`.
- **Implementation**:
    - Convert relative paths to URL paths (e.g., `assets/texture.png` -> `http://localhost:port/assets/texture.png`).
    - Handle Javascript Promises using `wasm_bindgen_futures::JsFuture`.

### 2.4 Update Engine Systems
- **Task**: Refactor `SceneLoader`, `TextureManager`, and `ScriptLoader` to accept `Arc<dyn AssetLoader>`.
- **Challenge**: Initial loading must become asynchronous. The `engine::runtime` startup sequence will need to `await` asset loading.

## Phase 3: Web Entry Point

### 3.1 Create WASM Entry Point
- **Location**: `engine/src/lib.rs` (or a dedicated `engine/src/web_lib.rs`).
- **Function**: `#[wasm_bindgen] pub async fn start_game(canvas_id: &str) -> Result<(), JsValue>`.
- **Logic**:
    1. Initialize `console_error_panic_hook`.
    2. Get the Canvas element by ID.
    3. Initialize `Window` and `EventLoop` (Winit specific setup for Web).
    4. Initialize `EngineContext` with `WebLoader`.
    5. Start the Game Loop.

### 3.2 Main Loop Adaptation
- **Winit**: `EventLoop::run` works differently on Web (it throws an exception to break the stack).
- **Refactoring**: Ensure the `run()` function in `player.rs` can be reused or is abstract enough to be called from the WASM entry point. Ideally, move the run logic into `engine::runtime::app_runner`.

## Phase 4: React Application Setup (The "Client")

### 4.1 Initialize React Project
- **Tool**: Vite (Recommended for speed and WASM support).
- **Command**: `npm create vite@latest web-client -- --template react-ts`.

### 4.2 Build WASM Package
- **Command**: `wasm-pack build --target web --out-dir ../web-client/public/pkg` (or inside a generic pkg folder and linked).

### 4.3 Develop React Components
- **Component**: `GameCanvas.tsx`.
- **Logic**:
    - Render a `<canvas id="game-canvas">`.
    - Use `useEffect` to import the WASM module (`init`).
    - Call `start_game("game-canvas")`.
    - Handle cleanup (if possible, though Rust WASM threads often live as long as the page).

### 4.4 Asset Serving
- **Action**: Copy the `assets` folder and `projects` folder to `web-client/public`.
- **Result**: The React dev server will serve these files, allowing `fetch("/assets/...")` to work.

## Phase 5: Verification & Optimization

### 5.1 Testing
- Verify Scene Loading (JSON fetching).
- Verify Texture Loading (Image decoding in WASM).
- Verify Input Handling (Keyboard/Mouse maps).

### 5.2 Optimization
- Ensure `release` profile builds are small enough.
- Check `wgpu` Backend (WebGPU vs WebGL2 fallback).

---

## Action Items Checklist

- [ ] Modify `engine/Cargo.toml` dependencies.
- [ ] Create `AssetLoader` trait and implementations.
- [ ] Refactor `player.rs` logic into a reusable `App` struct or function.
- [ ] Implement `start_game` entry point for WASM.
- [ ] Create React App scaffold.
- [ ] Test Build & Run.
