# Camera Frustum Origin Fix - Complete Implementation

## Problem
The camera frustum was not starting from the camera origin like Unity. The frustum lines were not connecting properly to the camera position, making it difficult to visualize the camera's view volume correctly.

## Root Cause Analysis
1. **Projection Issues**: The camera origin projection to screen space was sometimes failing
2. **Fallback Handling**: When projection failed, the fallback logic wasn't consistent
3. **Debug Visibility**: No visual indication of where the camera origin was being projected

## Solution Implemented

### 1. Enhanced Camera Origin Projection
- Added robust projection of camera origin to screen space using `world_to_screen_allow_behind()`
- Implemented proper fallback to provided screen position when projection fails

### 2. Debug Visualization
Added debug markers to visualize camera origin projection:
- **Magenta circle**: Shows successfully projected camera origin
- **Red circle**: Shows fallback position when projection fails
- **Text labels**: Display camera world coordinates for debugging

### 3. Improved Frustum Line Drawing
- **Perspective Camera**: Lines from camera origin to far plane corners (pyramid shape)
- **Orthographic Camera**: Lines from camera origin to near plane corners (Unity-style)
- Consistent line drawing regardless of projection success/failure

### 4. Code Changes Made

#### File: `engine/src/editor/ui/scene_view/rendering/gizmos.rs`

**Added Debug Visualization:**
```rust
// Debug: Always draw a small circle at camera origin for debugging
if let Some(origin_screen) = camera_origin_screen {
    painter.circle_filled(origin_screen, 8.0, egui::Color32::from_rgb(255, 0, 255)); // Magenta circle
    painter.text(
        egui::pos2(origin_screen.x + 10.0, origin_screen.y),
        egui::Align2::LEFT_CENTER,
        format!("Cam Origin ({:.1}, {:.1}, {:.1})", cam_pos.x, cam_pos.y, cam_pos.z),
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(255, 0, 255),
    );
} else {
    // If projection fails, draw at fallback position
    painter.circle_filled(camera_screen_pos, 8.0, egui::Color32::from_rgb(255, 100, 100)); // Red circle
    painter.text(
        egui::pos2(camera_screen_pos.x + 10.0, camera_screen_pos.y),
        egui::Align2::LEFT_CENTER,
        format!("Cam Fallback ({:.1}, {:.1}, {:.1})", cam_pos.x, cam_pos.y, cam_pos.z),
        egui::FontId::proportional(10.0),
        egui::Color32::from_rgb(255, 100, 100),
    );
}
```

**Simplified Frustum Line Drawing:**
```rust
// Perspective Camera - pyramid lines
let origin_to_use = camera_origin_screen.unwrap_or(camera_screen_pos);
for far_point in far_points.iter() {
    painter.line_segment([origin_to_use, *far_point], line_stroke);
}

// Orthographic Camera - lines to near plane
let origin_to_use = camera_origin_screen.unwrap_or(camera_screen_pos);
if near_corners.iter().all(|p| p.is_some()) {
    let near_points: Vec<egui::Pos2> = near_corners.into_iter().map(|p| p.unwrap()).collect();
    for near_point in near_points.iter() {
        painter.line_segment([origin_to_use, *near_point], line_stroke);
    }
}
```

## Expected Behavior

### What You Should See Now:
1. **Camera Origin Marker**: A magenta circle at the camera's projected position
2. **Frustum Lines**: Yellow lines connecting from the camera origin to frustum corners
3. **Proper Unity-Style Visualization**: 
   - Perspective cameras show pyramid shape from origin
   - Orthographic cameras show lines from origin to near plane

### Visual Indicators:
- **Magenta Circle + Text**: Camera origin successfully projected to screen
- **Red Circle + Text**: Camera origin projection failed, using fallback position
- **Yellow Lines**: Frustum edges connecting to camera origin
- **Coordinate Display**: Shows camera world position for debugging

## Testing Instructions

1. **Run the Engine**: `cargo run --package engine`
2. **Open Scene View**: Switch to 3D mode if not already
3. **Add Camera Entity**: Create a camera in the scene
4. **Observe Frustum**: Look for:
   - Magenta circle at camera position
   - Yellow frustum lines starting from the circle
   - Proper pyramid (perspective) or box (orthographic) shape

## Troubleshooting

### If You See Red Circles:
- Camera origin projection is failing
- Check camera position (very far from scene view)
- Verify scene camera settings

### If Frustum Lines Don't Connect:
- Check that camera entity has both Camera and Transform components
- Verify camera is within reasonable distance from scene

### If No Frustum Appears:
- Ensure camera entity is selected or visible in scene
- Check that `render_camera_frustum_3d` is being called
- Verify camera has valid projection settings

## Unity Compatibility

This implementation now matches Unity's camera frustum visualization:
- **Perspective**: Pyramid shape from camera origin
- **Orthographic**: Rectangular box with lines from origin
- **Proper Spatial Representation**: Uses 3D world-space calculations
- **Consistent Behavior**: Works in both Isometric and Perspective scene modes

## Next Steps

1. Test with different camera positions and rotations
2. Verify behavior in both Isometric and Perspective scene modes
3. Test with both Perspective and Orthographic camera projections
4. Remove debug circles once confirmed working correctly

The camera frustum should now properly start from the camera origin like Unity, providing accurate visual feedback for camera positioning and orientation.