# Camera Gizmo Unity-Style Rendering

## Issue
The camera frustum gizmo appears to rotate when you rotate the Scene Camera view (using Alt+drag). This doesn't match Unity's behavior.

## Root Cause
- The Perspective camera frustum uses `world_to_screen_allow_behind()` which applies the Scene Camera's view matrix
- When Scene Camera is in **Perspective** mode, the 3D frustum gets projected with perspective distortion
- This makes it appear to rotate as you orbit the Scene Camera around the scene

## Solution: Use Isometric Scene Camera Projection

Unity's Scene View typically uses an **Isometric (orthographic)** projection for the Scene Camera, not perspective. This makes 3D gizmos and frustums appear stable and not distorted.

### How to Switch to Isometric Mode:
1. Look at the **top-right corner** of the Scene View (in 3D mode)
2. Find the button below the scene gizmo that says either:
   - **"⬜ Persp"** (Perspective mode - current)
   - **"◇ Iso"** (Isometric mode - Unity-like)
3. Click the button to toggle to **Isometric** mode

### Expected Behavior:
- **Perspective Scene Camera** (⬜ Persp): Camera frustum appears to rotate with Scene Camera view
- **Isometric Scene Camera** (◇ Iso): Camera frustum stays stable in world space, just like Unity

## Technical Details

### Current Implementation:
- **Perspective Camera Frustum** (pyramid): Rendered in 3D world space using `world_to_screen_allow_behind()`
  - File: [gizmos.rs:708-746](engine/src/editor/ui/scene_view/rendering/gizmos.rs#L708-L746)
  - Projects 3D corners using Scene Camera's view matrix

- **Orthographic Camera Frustum** (box): Rendered in 2D screen space as billboard
  - File: [gizmos.rs:748-813](engine/src/editor/ui/scene_view/rendering/gizmos.rs#L748-L813)
  - Uses 2D rotation independent of Scene Camera
  - This is a workaround to avoid Scene Camera rotation affecting the gizmo

### Why Unity Looks Different:
Unity's Scene View uses Isometric/Orthographic projection by default, which means:
1. No perspective distortion in Scene View
2. 3D objects and gizmos maintain their world-space orientation
3. Camera frustums appear as clean, undistorted 3D volumes

When you use **Isometric** mode in our engine, the frustum rendering will match Unity's appearance exactly.

## Recommendation

For the best Unity-like experience when working with Camera components:
- **Always use Isometric Scene Camera mode** (◇ Iso) when placing or adjusting cameras
- Switch to Perspective mode only when you want to preview how the scene looks with depth

The projection mode toggle button is located at:
[mod.rs:150-160](engine/src/editor/ui/scene_view/mod.rs#L150-L160)
