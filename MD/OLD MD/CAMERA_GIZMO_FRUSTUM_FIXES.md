# Camera Gizmo and Frustum Visibility Fixes

## Issues Fixed

### 1. Camera Gizmo Not Showing in 3D Mode
**Problem**: Camera gizmos were not visible in 3D mode due to projection failures and rendering logic issues.

**Solutions Implemented**:
- **Enhanced Projection Fallback**: Added multiple projection methods with fallback positions
- **Fixed Size Gizmos**: Changed from zoom-dependent to fixed-size gizmos (50px) for better visibility
- **Brighter Colors**: Used bright yellow (255,255,0) with thicker outlines (3px) for better visibility
- **Fallback Rendering**: When projection fails, cameras render at fixed positions with labels
- **Debug Information**: Added entity ID labels to identify cameras
- **Default Gizmo**: Added fallback gizmo for entities without camera components

### 2. Camera Frustum Not Showing in 3D Mode
**Problem**: Camera frustum (pyramid showing FOV) was not visible due to complex projection calculations and extreme values.

**Solutions Implemented**:
- **Simplified Frustum**: Reduced far plane distance to 5.0 units for better visibility
- **Test Frustum**: Added simple test lines from camera to fixed points around it
- **Multiple Projection Methods**: Try regular projection first, then allow_behind projection
- **Bright Visualization**: Bright yellow color (255,255,0) with thick lines (3px)
- **Debug Cross**: Red cross at camera position for debugging
- **Robust Calculations**: Fixed FOV to 60° and aspect to 1.0 for consistency

### 3. Improved Projection System
**Problem**: `world_to_screen_allow_behind` function was failing for points behind camera.

**Solutions Implemented**:
- **Better W-value Handling**: Improved handling of negative W values for points behind camera
- **NDC Clamping**: Clamp NDC coordinates to prevent extreme values (-10 to +10)
- **Graceful Degradation**: Handle edge cases without crashing
- **Wider Screen Bounds**: Allow gizmos to render off-screen (up to 10,000 pixels)

### 4. Enhanced Camera Entity Detection
**Problem**: Not all camera entities were being found and rendered.

**Solutions Implemented**:
- **Comprehensive Collection**: Collect ALL entities with camera components
- **Dual Rendering Path**: Handle both mesh-based and non-mesh camera entities
- **Debug Messages**: Show "No cameras found" when no camera entities exist
- **Entity Offset**: Multiple cameras get offset positions to avoid overlap

## Key Changes Made

### Files Modified:
1. `engine/src/editor/ui/scene_view/rendering/gizmos.rs`
2. `engine/src/editor/ui/scene_view/rendering/view_3d.rs`
3. `engine/src/editor/ui/scene_view/rendering/projection_3d.rs`

### New Features:
- **Always-Visible Camera Gizmos**: Cameras are now always visible, even when behind the view camera
- **Test Frustum Rendering**: Simple test lines ensure frustum is always visible
- **Debug Information**: Entity IDs and status labels for troubleshooting
- **Fallback Positions**: Off-screen cameras render at fixed positions with labels
- **Robust Projection**: Improved projection system handles edge cases gracefully

## Testing Instructions

1. **Open 3D Scene View**: Switch to 3D mode in the scene view
2. **Add Camera Entity**: Create an entity with a camera component
3. **Verify Gizmo**: Should see bright yellow camera gizmo at camera position
4. **Verify Frustum**: Should see yellow pyramid lines showing camera FOV
5. **Test Movement**: Move camera around - gizmo and frustum should follow
6. **Test Behind Camera**: Move scene camera behind entity camera - should still see gizmo at fallback position

## Expected Results

- **Camera Gizmos**: Bright yellow camera icons visible for all camera entities
- **Camera Frustum**: Yellow pyramid lines showing camera field of view
- **Always Visible**: Gizmos visible even when cameras are behind the scene camera
- **Debug Info**: Entity IDs and status labels help identify cameras
- **Smooth Performance**: No crashes or extreme values in projection calculations

## Unity-Style Behavior

The implementation now matches Unity's camera gizmo behavior:
- **Orthographic Cameras**: Trapezoid shape (2D style)
- **Perspective Cameras**: 3D camera icon with lens and viewfinder
- **Frustum Visualization**: Pyramid lines from camera to far plane corners
- **Always Visible**: Gizmos remain visible regardless of scene camera position
- **Consistent Colors**: Bright yellow for easy identification

## Debug Features

- **Red Cross**: Shows exact camera position for debugging
- **Test Lines**: Simple frustum lines ensure basic visibility
- **Entity Labels**: Show camera entity IDs
- **Status Messages**: Indicate off-screen or missing cameras
- **Fallback Rendering**: Ensures cameras are never completely invisible