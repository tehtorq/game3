# Unused Code Analysis

## Summary
This analysis identifies all unused structs, modules, types, and files in the codebase that can be safely removed.

## Completely Unused Files
These files are not imported or used anywhere:

1. **src/terrain_cpu.rs** - Not imported in any module
2. **src/game.rs.bak** - Backup file
3. **src/terrain/mesh_refactored.rs** - Not imported in terrain/mod.rs
4. **src/terrain/gpu_batch.rs** - Only referenced by unused terrain_cpu.rs
5. **src/test_audio.rs** - Placeholder binary with no real code

## Unused Modules
These modules are declared but their contents are never used:

### In src/app/
- **AppState** struct - Never constructed
- **EventHandler** struct - Never constructed
- Entire module is imported in main.rs but not used

### In src/input/
- **InputHandler** struct - Never constructed
- **InputState** in input module (main.rs defines its own InputState)
- Entire module is imported in main.rs but not used

### In src/graphics/
- **GraphicsPipelines** struct - Never constructed
- **create_pipelines** function - Never called
- **ShaderSource** - Never used
- Entire module is imported in main.rs but not used

### In src/sounds/
- **ThrusterSound** struct - Never constructed (despite being re-exported)

## Unused Terrain Module Components

### Completely Unused Terrain Modules:
1. **terrain/batch.rs** - TerrainBatch never constructed
2. **terrain/chunk.rs** - TerrainChunk never constructed  
3. **terrain/clipmap.rs** - All structs (ClipRing, LShape, CenterPatch, TrimRegion, TerrainClipmap) never constructed
4. **terrain/lod_blend.rs** - LodBlendInfo never constructed
5. **terrain/mesh_builder.rs** - MeshBuilder never constructed
6. **terrain/predictive.rs** - PredictiveLoader never constructed
7. **terrain/texture_generator.rs** - TextureGenerator never constructed

### Partially Used Terrain Modules:
- **terrain/gpu_rings.rs** - TerrainGPURings is declared in main.rs but set to None and never used
- **terrain/mesh.rs** - Terrain is re-exported but never used
- **terrain/cache.rs** - TerrainCache re-exported but never used

### Used Terrain Modules:
- **terrain/gpu_complete.rs** - TerrainGPUComplete is actively used
- **terrain/generation.rs** - Provides height_at function used by gpu_complete
- **terrain/biome_heights.rs** - Used by generation module

## Unused Enemy System Components

### Duplicate Code:
1. **src/enemy.rs** - Old enemy implementation still imported but superseded by enemies/ module
   - Contains duplicate MovementPattern enum
   - Contains old Enemy struct implementation
   - Should be removed in favor of enemies/ module

### Unused in enemies/ module:
1. **enemies/factory.rs** - EnemyFactory never constructed
2. **enemies/core.rs** - Contains duplicate MovementPattern enum, FormationType never used

## Unused Types/Enums
1. **LodLevel** enum in terrain module - Never used
2. **MovementPattern** enum in enemy.rs - Duplicate of the one in enemies/movement.rs
3. **FormationType** enum in enemies/core.rs - Never used
4. **UniformsTerrain** struct - Never constructed

## Unused Constants
In terrain/mod.rs:
- CHUNK_SIZE
- TERRAIN_SCALE  
- MAX_HEIGHT
- WATER_LEVEL

## Recommendations for Removal

### High Priority (Safe to Remove):
1. Delete **src/terrain_cpu.rs**
2. Delete **src/game.rs.bak**
3. Delete **src/terrain/mesh_refactored.rs**
4. Delete **src/test_audio.rs**
5. Delete **src/enemy.rs** (replaced by enemies/ module)
6. Remove unused modules from **src/app/** (entire module can be removed)
7. Remove unused modules from **src/input/** (entire module can be removed)
8. Remove unused modules from **src/graphics/** (entire module can be removed)

### Medium Priority (Unused Terrain Components):
1. Delete **src/terrain/batch.rs**
2. Delete **src/terrain/chunk.rs**
3. Delete **src/terrain/clipmap.rs**
4. Delete **src/terrain/lod_blend.rs**
5. Delete **src/terrain/mesh_builder.rs**
6. Delete **src/terrain/predictive.rs**
7. Delete **src/terrain/texture_generator.rs**
8. Delete **src/terrain/gpu_batch.rs**
9. Remove TerrainGPURings code if not planning to use it

### Low Priority (May be kept for future use):
1. **enemies/factory.rs** - Might be useful for future enemy spawning systems
2. **terrain/cache.rs** - Might be useful if terrain caching is re-enabled

## Code to Clean Up

In **src/main.rs**:
- Remove imports for unused modules (app, graphics, input)
- Remove `mod enemy;` (use only enemies module)
- Remove `terrain_gpu_rings: Option<terrain::TerrainGPURings>` field from Stage
- Remove `terrain_bindings` field (noted as never read)

In **src/terrain/mod.rs**:
- Remove module declarations for unused modules
- Remove re-exports for unused types
- Remove unused constants

In **src/enemies/mod.rs**:
- Remove unused re-exports (MovementController not used outside module)