# 📦 Asset Management System Design

## 🎯 Overview
The Asset Management System (AMS) is the backbone of the game engine, responsible for importing, processing, tracking, and loading game assets (textures, models, audio, scripts, scenes, etc.).

This document outlines the architecture for a robust, Unity/Unreal-style asset system that supports:
1.  **Unique Identification (UUIDs)** for stable references.
2.  **Metadata & Import Settings** via sidecar `.meta` files.
3.  **Async Loading & Caching** for performance.
4.  **Hot Reloading** for rapid iteration.
5.  **Editor Integration** (Content Browser).

---

## 🏗 Architecture

### 1. The Asset Database (Editor-Side)
The **Asset Database** is responsible for tracking all files in the `assets/` folder. It maps file paths to UUIDs and maintains the state of imported assets.

-   **Responsibility**:
    -   Scanning the `assets/` directory.
    -   Generating/Reading `.meta` files.
    -   Detecting file changes (watchdog).
    -   Triggering importers when source files change.

### 2. The Runtime Asset Manager
The **Runtime Manager** handles loading assets into memory for the game.

-   **Responsibility**:
    -   Loading assets by UUID or Path.
    -   Caching loaded assets (ref-counting).
    -   Unloading unused assets.
    -   Handling async loading requests.

---

## 📂 File Structure & Metadata

### The `.meta` File
Every source asset (e.g., `player.png`) will have a generated sidecar file (e.g., `player.png.meta`).

**Why?**
-   **Stable References**: If `player.png` is moved to a different folder, the UUID in the `.meta` file stays the same. Other assets (like Scenes) reference the UUID, not the path.
-   **Import Settings**: Stores settings specific to that asset (e.g., Texture filtering mode, Audio compression quality).

#### Example: `player.png.meta`
```yaml
guid: 5a2b3c4d-8e9f-1g2h-3i4j-5k6l7m8n9o0p
importer: TextureImporter
settings:
  read_write: false
  generate_mipmaps: true
  filter_mode: Bilinear
  wrap_mode: Clamp
```

---

## 🔄 Asset Workflow

### 1. Import Process
When a new file is dropped into `assets/`:
1.  **Detection**: `AssetDatabase` detects the new file.
2.  **Meta Generation**: A unique UUID is generated, and a `.meta` file is created.
3.  **Processing**: The appropriate `Importer` (e.g., `TextureImporter`) processes the raw file into an engine-friendly format if necessary (or loads it directly).

### 2. Loading (Runtime)
```rust
// Identifying an asset
let player_sprite_id = AssetId::from("5a2b3c4d-...");

// Requesting load (Async)
let handle = asset_manager.load::<Texture>(player_sprite_id);

// Accessing (once loaded)
if let Some(texture) = asset_manager.get(handle) {
    renderer.draw(texture);
}
```

---

## 🧩 Core Components

### `AssetId` (UUID)
A 128-bit unique identifier referencing a specific asset.

### `AssetHandle<T>`
A lightweight handle used by game objects to reference an asset. It allows the underlying asset to be swapped (hot-reloaded) without updating every game object.

### `AssetCache`
A map of `AssetId -> LoadedAsset`. Uses Reference Counting (Arc) to keep assets alive while in use.

---

## 🗓 Roadmap

### Phase 1: Foundation (Current Status)
- [x] Basic File Browser (Editor).
- [x] `AssetMetadata` struct (Size, Type, Modified Time).
- [ ] UUID Generation for all assets.
- [ ] `.meta` file creation/parsing.

### Phase 2: Runtime Integration
- [ ] `AssetManager` struct with `load()` and `unload()`.
- [ ] Integration with `TextureManager`, `ModelManager`, etc.
- [ ] Scene format update to use UUIDs instead of paths.

### Phase 3: Advanced Features
- [ ] Hot Reloading (Update GPU resources when file changes).
- [ ] Asset Bundles (Packing for release).
- [ ] Async Loading (Background thread pool).

---

## 📝 Usage Examples

### Defining a Component with an Asset Ref
```rust
struct SpriteComponent {
    texture: AssetHandle<Texture>,
    // stored as UUID in serialization
}
```

### Drag & Drop in Editor
1.  User drags `jump.wav` from Content Browser to a Button in Inspector.
2.  Editor looks up `.meta` for `jump.wav`.
3.  Inspector receives the UUID.
4.  Component saves the UUID.

---
*Created by XS Game Studio Architecture Team*
