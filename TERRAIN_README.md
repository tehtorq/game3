# Terrain Rendering System

The game features a complete GPU-based terrain rendering system with rich biome variety.

## Controls

- **T** - Toggle between terrain rendering modes
- **W/A/S/D** - Move around
- **Space** - Move up
- **C** - Move down
- **Mouse** - Look around
- **Left Click** - Shoot
- **Shift** - Boost

## Terrain Modes

### 1. GPUComplete (Default)
- Full GPU terrain with texture support
- Cached height/biome textures
- All 14 biomes fully implemented
- Best visual quality
- ~80 FPS with full visual features

### 2. GPUGrid
- Simple GPU-based grid terrain
- Entire terrain generated on GPU
- High performance (~90 FPS)
- Basic height generation with biomes
- Lightweight alternative

## Biomes

The terrain system includes 14 distinct biomes:

1. **Plains** - Rolling hills with grass
2. **Canyon** - Deep river canyons with mesas
3. **Plateau** - Flat-topped mesas with cliffs
4. **Crystalline** - Sharp crystal formations
5. **Volcanic** - Craters and lava flows
6. **Mountains** - Dramatic peaks and valleys
7. **Desert** - Sand dunes and rock formations
8. **Arctic** - Glaciers and ice formations
9. **Badlands** - Eroded rock with hoodoos
10. **Floating** - Sky islands at high altitude
11. **Caverns** - Sinkholes and cave systems
12. **Swamp** - Wetlands with pools
13. **_More biomes..._**

## Features

- **Dynamic Lighting** - Sun and moon lighting with rim lighting
- **Biome Blending** - Smooth transitions between biomes
- **Height-based Coloring** - Terrain color varies with elevation
- **Water Effects** - Low areas appear as water
- **Wireframe Overlay** - Subtle edge highlighting

## Performance

- GPUComplete: ~80 FPS with full visual features and all biomes
- GPUGrid: ~90 FPS with 2M+ triangles, simplified rendering

## Technical Details

- Vertex shader generates all terrain heights on GPU
- No CPU terrain calculation in GPU modes
- Consistent height calculation across all modes
- Player collision works with all terrain types
- Infinite terrain through grid following