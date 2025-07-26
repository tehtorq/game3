# Coordinate System Analysis for Terrain Rendering Issue

## Current Situation
The terrain is rendering incorrectly when camera-relative rendering is applied. Specifically:
- When flying forward/backward, the terrain moves up/down
- The terrain appears inverted or in the wrong position

## Coordinate System Assumptions

### 1. World Coordinate System
- **Handedness**: Left-handed (based on `Mat4::look_at_lh` and `Mat4::perspective_lh` usage)
- **Axes**:
  - X: Right (positive X goes right)
  - Y: Up (positive Y goes up)
  - Z: Forward (positive Z goes into the screen in left-handed system)
- **Origin**: World center at (0, 0, 0)

### 2. Camera System
- **Camera Position**: Calculated as `player_pos + camera_offset`
  - `camera_offset.x = sin(rotation) * distance`
  - `camera_offset.y = height` (constant height above player)
  - `camera_offset.z = cos(rotation) * distance`
- **Look Target**: `player_pos + look_ahead`
  - `look_ahead.x = -sin(rotation) * 50.0`
  - `look_ahead.y = 0.0`
  - `look_ahead.z = -cos(rotation) * 50.0`
- **Up Vector**: (0, 1, 0) - Y is up

### 3. View Matrix Construction
- Using `Mat4::look_at_lh(camera_pos, look_target, up)`
- For camera-relative rendering, attempting to extract rotation-only component

### 4. Projection Matrix
- Using `Mat4::perspective_lh(fov, aspect, near, far)`
- Left-handed perspective projection

### 5. Terrain Generation
- **Grid Generation**: 
  - Vertices start at Y=0 in object space
  - Grid centered at origin: `(x - half_size) * grid_spacing`
  - Height added procedurally in vertex shader
- **Height Range**: Approximately -500 to +500 units
- **Coordinate Space**: World space positions calculated in shader

### 6. Camera-Relative Rendering Approach
- **Goal**: Avoid floating-point precision issues at large distances
- **Method**: Subtract camera position from vertex positions before MVP transform
- **Current Implementation**:
  ```glsl
  vec3 relative_pos = world_pos - camera_pos;
  gl_Position = mvp * vec4(relative_pos, 1.0);
  ```

## Potential Issues

### 1. View Matrix Translation Removal
When extracting rotation-only from view matrix:
```rust
Mat4::from_cols(
    full_view.x_axis,
    full_view.y_axis,
    full_view.z_axis,
    Vec3::ZERO.extend(1.0)  // Translation removed
)
```
This might not be the correct approach for camera-relative rendering.

### 2. Double Transformation
- Camera position is subtracted in shader
- View matrix also contains camera transformation
- This could lead to double transformation or incorrect results

### 3. Terrain Height Application
- Terrain starts at Y=0 and height is added
- When camera Y is subtracted, it affects the base height
- This could cause the terrain to move vertically

### 4. Coordinate Space Mismatch
- View matrix expects world-space positions
- But we're feeding it camera-relative positions
- The rotation-only view matrix might not handle this correctly

## Questions to Investigate

1. **Is the view matrix construction correct for camera-relative rendering?**
   - Should we use identity matrix instead?
   - Should we construct view matrix differently?
   - Is the rotation extraction correct?

2. **Is the shader transformation correct?**
   - Should we subtract camera_pos before or after other transformations?
   - Should we handle Y differently than X and Z?

3. **What is the correct order of operations?**
   - Model transform → Camera-relative → View → Projection?
   - Model transform → View → Camera-relative → Projection?

4. **Are we in the correct coordinate system throughout?**
   - Is terrain generation assuming a different coordinate system?
   - Are all matrices using the same handedness?

## Test Cases to Verify

1. **Without camera-relative rendering**: Does terrain render correctly?
2. **With identity view matrix**: What happens?
3. **With full view matrix and no camera_pos subtraction**: Baseline behavior
4. **With different Y handling**: Only subtract X,Z vs subtract all

## Mathematical Analysis

### Standard Rendering Pipeline
```
world_pos → view_matrix → projection_matrix → clip_space
```

### Camera-Relative Rendering Pipeline (Current)
```
world_pos → (world_pos - camera_pos) → rotation_only_view → projection → clip_space
```

### Expected Camera-Relative Pipeline
```
world_pos → (world_pos - camera_pos) → rotation_view → projection → clip_space
```

Where `rotation_view` should represent only the camera's orientation, not position.