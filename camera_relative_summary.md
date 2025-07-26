# Camera-Relative Rendering Implementation

## Summary

Camera-relative rendering is an attempted solution to eliminate floating-point precision issues when flying far from the origin. However, the current implementation has issues with terrain rendering.

## The Problem

When implementing camera-relative rendering:
1. The terrain inverts or moves incorrectly when the camera moves
2. Specifically, when flying forward/backward, the terrain moves up/down
3. Other objects (player, enemies, bullets) render correctly

## Technical Approach

The implementation attempts to separate translation and rotation:

1. **View Matrix Creation**:
   ```rust
   // Create full view matrix normally
   let full_view = Mat4::look_at_lh(camera_pos, look_target, up);
   
   // Extract only rotation (zero out translation)
   Mat4::from_cols(
       full_view.x_axis,
       full_view.y_axis, 
       full_view.z_axis,
       Vec4::new(0.0, 0.0, 0.0, 1.0)
   )
   ```

2. **In vertex shaders**:
   ```glsl
   vec3 relative_pos = world_pos - camera_pos;
   gl_Position = mvp * vec4(relative_pos, 1.0);
   ```

## Current Issues

1. **Terrain Movement**: The terrain moves vertically when the camera moves forward/backward
2. **View Matrix Construction**: Extracting rotation from a look_at matrix doesn't produce the correct transformation
3. **Coordinate System**: The interaction between camera-relative positioning and the view matrix is not working as expected

## Why It's Difficult

The challenge is that:
- Standard view matrices include both rotation and translation
- Extracting just the rotation component doesn't account for how the view direction was calculated
- The terrain's procedural height generation interacts poorly with camera-relative positioning

## Status

**NOT FULLY WORKING** - While the approach eliminates precision issues, it causes incorrect terrain rendering. A different approach may be needed, such as:
- Using a world origin shifting technique instead
- Implementing double precision for world positions
- Using a different view matrix construction method