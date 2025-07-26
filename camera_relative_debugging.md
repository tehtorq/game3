# Camera-Relative Rendering Debug Analysis

## The Problem
When camera-relative rendering is enabled, the terrain exhibits incorrect behavior:
1. Terrain moves up/down when flying forward/backward
2. Terrain may appear inverted or at wrong position

## Current Implementation Details

### 1. Coordinate System
- **Engine**: Left-handed coordinate system
- **Y-axis**: Points UP (positive Y = up)
- **Z-axis**: Points FORWARD (positive Z = into screen)
- **X-axis**: Points RIGHT

### 2. Camera Setup
```rust
// Camera position behind and above player
camera_offset.x = sin(rotation) * distance
camera_offset.y = height  // Fixed height above player
camera_offset.z = cos(rotation) * distance

camera_pos = player_pos + camera_offset
```

### 3. View Matrix Construction (Current)
```rust
// Step 1: Create normal view matrix
let full_view = Mat4::look_at_lh(camera_pos, look_target, up);

// Step 2: Extract rotation only (remove translation)
Mat4::from_cols(
    full_view.x_axis,
    full_view.y_axis,
    full_view.z_axis,
    Vec3::ZERO.extend(1.0)  // Sets translation to (0, 0, 0)
)
```

### 4. Shader Implementation
```glsl
// Terrain vertex shader (current with Y-fix attempt)
vec3 relative_pos = vec3(
    v_world_pos.x - camera_pos.x,
    v_world_pos.y,  // Not subtracting Y
    v_world_pos.z - camera_pos.z
);
gl_Position = mvp * vec4(relative_pos, 1.0);

// Other objects (player, enemies, etc)
vec3 relative_pos = pos - camera_pos;
gl_Position = mvp * vec4(relative_pos, 1.0);
```

### 5. Terrain Specifics
- Terrain grid starts with Y=0
- Height is added procedurally: `world_pos.y = get_blended_biome_height(world_pos.xz)`
- Grid follows camera: `chunk_offset = floor(camera_pos.xz / grid_spacing) * grid_spacing`

## Analysis of the Issue

### What Should Happen
1. Camera-relative rendering should make world coordinates relative to camera position
2. View matrix should only contain rotation (no translation)
3. Result: Objects stay in correct relative positions but with better floating-point precision

### What's Actually Happening
1. When we subtract `camera_pos` in the shader, we're moving the world by `-camera_pos`
2. The rotation-only view matrix then rotates this shifted world
3. This is causing the terrain to move incorrectly

### The Core Problem
The issue appears to be in how we're constructing the rotation-only view matrix. When we extract just the rotation from a `look_at` matrix, we might not be getting the correct transformation.

## Hypothesis

The `look_at_lh` function creates a view matrix that:
1. Translates by `-camera_pos`
2. Rotates to align with the view direction

When we extract only the rotation part and apply it to already-translated vertices (via `world_pos - camera_pos`), we're not getting the correct transformation.

## Potential Solutions

### Solution 1: Use Full View Matrix Without Camera Subtraction
- Don't subtract camera_pos in shader
- Use normal view matrix
- This is the standard approach but loses precision benefits

### Solution 2: Construct Rotation Matrix Differently
Instead of extracting from `look_at`, build rotation matrix directly:
```rust
// Calculate view direction vectors
let forward = (look_target - camera_pos).normalize();
let right = forward.cross(up).normalize();
let up = right.cross(forward);

// Build rotation matrix directly
Mat4::from_cols(
    right.extend(0.0),
    up.extend(0.0),
    -forward.extend(0.0),  // Negative for view matrix
    Vec4::W
)
```

### Solution 3: Apply Camera-Relative Only to Large Objects
- Keep terrain using standard rendering
- Apply camera-relative only to small objects where precision matters

### Solution 4: Investigate Matrix Order
The order might be wrong. Instead of:
```
(world - camera) * rotation_view * projection
```
We might need:
```
world * rotation_view * projection - camera_projection_offset
```

## Next Steps to Debug

1. **Test with identity view matrix** - See if terrain renders at correct position
2. **Log matrix values** - Print out view matrix to see what we're getting
3. **Test without Y subtraction** - Already tried, still inverted
4. **Build rotation matrix manually** - Don't extract from look_at
5. **Check if other engines handle this differently** - Research camera-relative implementations