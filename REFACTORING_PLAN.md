# Refactoring Plan for Vector Shooter 3D

## Current Issues

1. **Large monolithic files**:
   - `main.rs` - 1353 lines (handles rendering, input, game loop, terrain rendering)
   - `shader.rs` - 1244 lines (contains all shader code as strings)
   - `game.rs` - 817 lines (handles all game logic, collision, spawning)

2. **Mixed responsibilities**:
   - `main.rs` handles rendering, input, window management, and game state
   - `game.rs` handles enemy spawning, collision detection, base management, wave system
   - `shader.rs` contains all shaders as inline strings

3. **Code duplication**:
   - Terrain height calculations (already partially addressed)
   - Similar rendering patterns repeated across different entity types

## Proposed Module Structure

```
src/
├── main.rs (minimal - just entry point)
├── app.rs (main application state/event handler)
├── game/
│   ├── mod.rs
│   ├── state.rs (Game struct)
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── collision.rs
│   │   ├── spawning.rs
│   │   ├── wave_manager.rs
│   │   ├── base_manager.rs
│   │   └── combat.rs
│   └── entities/ (already exists as individual files)
├── rendering/
│   ├── mod.rs
│   ├── renderer.rs (move from current renderer.rs)
│   ├── pipelines.rs (shader pipeline setup)
│   ├── draw_calls.rs (actual drawing logic)
│   └── mesh_generation.rs
├── shaders/
│   ├── mod.rs
│   ├── terrain/
│   │   ├── vertex.glsl
│   │   ├── fragment.glsl
│   │   └── mod.rs
│   ├── basic/
│   │   ├── vertex.glsl
│   │   ├── fragment.glsl
│   │   └── mod.rs
│   ├── volumetric_laser/
│   │   ├── vertex.glsl
│   │   ├── fragment.glsl
│   │   └── mod.rs
│   └── loader.rs (shader loading utilities)
├── input/
│   ├── mod.rs
│   ├── keyboard.rs
│   ├── mouse.rs
│   └── state.rs
├── ui/
│   ├── mod.rs
│   ├── hud.rs (move current HUD)
│   └── debug_overlay.rs
└── terrain/ (already refactored)
```

## Refactoring Steps

### Phase 1: Extract Systems from game.rs
1. Create `game/systems/` directory
2. Extract collision detection to `collision.rs`
3. Extract enemy spawning logic to `spawning.rs`
4. Extract wave management to `wave_manager.rs`
5. Extract base-related logic to `base_manager.rs`
6. Extract combat logic (damage, projectiles) to `combat.rs`

### Phase 2: Split main.rs
1. Create `app.rs` for the main Stage struct and EventHandler
2. Create `rendering/` module structure
3. Move all rendering logic to appropriate modules
4. Keep only the main() function in main.rs

### Phase 3: Modularize Shaders
1. Create `shaders/` directory structure
2. Move shader strings to actual .glsl files
3. Create a shader loader that reads files at compile time
4. Group related shaders together

### Phase 4: Extract Input Handling
1. Create `input/` module
2. Move InputState and input handling logic
3. Separate keyboard and mouse handling

### Phase 5: Clean up UI
1. Move HUD to ui/ module
2. Add debug overlay capabilities
3. Separate UI rendering from game rendering

## Benefits

1. **Maintainability**: Easier to find and modify specific functionality
2. **Testability**: Smaller modules are easier to unit test
3. **Reusability**: Systems can be reused or replaced independently
4. **Compilation**: Changes to one module won't require recompiling everything
5. **Code clarity**: Each file has a single, clear responsibility

## Implementation Priority

1. **High Priority**: Extract systems from game.rs (Phase 1)
2. **High Priority**: Split main.rs (Phase 2)
3. **Medium Priority**: Modularize shaders (Phase 3)
4. **Low Priority**: Extract input handling and UI (Phases 4-5)

## Notes

- Each phase should maintain full functionality
- Use `pub(crate)` for internal APIs
- Add documentation to each module
- Consider adding integration tests after refactoring