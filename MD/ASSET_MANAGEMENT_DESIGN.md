# Unified Asset Inspector Design

## Overview
Currently, the XS Editor has a hardcoded check in `dock_layout.rs`:
```rust
if texture_selected { show_texture_inspector() } else { show_entity_inspector() }
```
This is not scalable. As we add Audio, 3D Models (.gltf), and Materials, we need a generic system.

## 1. The Problem
*   **No Model Settings:** You cannot change the Import Scale or Forward Axis of a GLTF model.
*   **No Audio Settings:** You cannot set an MP3 to "Stream from Disk" vs "Decompress on Load".
*   **Hardcoded Logic:** `dock_layout.rs` will grow indefinitely if we keep adding `if/else`.

## 2. Proposed Architecture

### 2.1 The `AssetInspector` Trait
We introduce a trait that all asset inspectors must implement.

```rust
pub trait AssetInspector {
    /// Return true if this inspector handles this file extension
    fn can_inspect(&self, path: &Path) -> bool;
    
    /// Draw the UI. Return `true` if changed.
    fn render(&mut self, ui: &mut egui::Ui, path: &Path, asset_manager: &AssetManager) -> bool;
    
    /// Apply changes to .meta file
    fn save(&mut self, path: &Path);
}
```

### 2.2 The `InspectorRegistry`
A central registry in `EditorState`.

```rust
pub struct InspectorRegistry {
    inspectors: Vec<Box<dyn AssetInspector>>,
}

impl InspectorRegistry {
    pub fn render_for_path(&mut self, ui: &mut egui::Ui, path: &Path) {
        if let Some(inspector) = self.inspectors.iter_mut().find(|i| i.can_inspect(path)) {
            inspector.render(ui, path, ...);
        } else {
            ui.label("No inspector for this asset type.");
        }
    }
}
```

## 3. New Inspectors Needed

### 3.1 `ModelInspector` (.gltf, .glb, .xsg)
*   **Scale Factor:** Global scale multiplier (0.01 vs 1.0).
*   **Axis Conversion:** Blender (Z-up) vs Unity (Y-up) correction.
*   **Material Remapping:** Override internal materials with project materials.
*   **Collider Gen:** "Generate Convex Mesh Collider" checkbox.

### 3.2 `AudioInspector` (.wav, .mp3)
*   **Force To Mono:** For 3D positional audio.
*   **Compression:** `PCM`, `Vorbis`, `ADPCM`.
*   **Load Type:** `DecompressOnLoad` (SFX), `CompressedInMemory`, `Streaming` (Music).

### 3.3 `MaterialInspector` (.mat)
*   **Shader Selection:** Dropdown to pick `.wgsl` shader.
*   **Properties:** Auto-generated UI based on Shader Uniforms (Reflect from WGSL).
    *   `float` -> Slider
    *   `vec3` -> Color Picker
    *   `texture` -> Texture Slot

---

## 4. Implementation Steps

1.  **Refactor `dock_layout.rs`:** Remove the hardcoded `texture_inspector` check.
2.  **Create `InspectorRegistry`:** In `editor/src/ui/inspector/registry.rs`.
3.  **Port `TextureInspector`:** Make it implement `AssetInspector` trait.
4.  **Implement `ModelInspector`:** Basic scale settings for GLTF.

This system is required before we can seriously import 3D assets for the Physics/Voxel work.
