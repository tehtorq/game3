#!/bin/bash
# Script to remove unused files from the codebase
# Run with: bash cleanup_unused.sh

echo "This script will remove unused files from the codebase."
echo "It's recommended to commit your changes first!"
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

# Remove completely unused files
echo "Removing unused files..."

# Backup files
[ -f "src/game.rs.bak" ] && rm -v "src/game.rs.bak"

# Unused standalone files  
[ -f "src/terrain_cpu.rs" ] && rm -v "src/terrain_cpu.rs"
[ -f "src/test_audio.rs" ] && rm -v "src/test_audio.rs"
[ -f "src/enemy.rs" ] && echo "NOTE: src/enemy.rs should be removed after updating main.rs imports"

# Unused terrain files
[ -f "src/terrain/mesh_refactored.rs" ] && rm -v "src/terrain/mesh_refactored.rs"
[ -f "src/terrain/gpu_batch.rs" ] && rm -v "src/terrain/gpu_batch.rs"
[ -f "src/terrain/batch.rs" ] && rm -v "src/terrain/batch.rs"
[ -f "src/terrain/chunk.rs" ] && rm -v "src/terrain/chunk.rs"
[ -f "src/terrain/clipmap.rs" ] && rm -v "src/terrain/clipmap.rs"
[ -f "src/terrain/lod_blend.rs" ] && rm -v "src/terrain/lod_blend.rs"
[ -f "src/terrain/mesh_builder.rs" ] && rm -v "src/terrain/mesh_builder.rs"
[ -f "src/terrain/predictive.rs" ] && rm -v "src/terrain/predictive.rs"
[ -f "src/terrain/texture_generator.rs" ] && rm -v "src/terrain/texture_generator.rs"

# Remove entire unused module directories
[ -d "src/app" ] && echo "NOTE: src/app/ directory should be removed after updating main.rs imports"
[ -d "src/input" ] && echo "NOTE: src/input/ directory should be removed after updating main.rs imports"  
[ -d "src/graphics" ] && echo "NOTE: src/graphics/ directory should be removed after updating main.rs imports"

echo ""
echo "File cleanup complete!"
echo ""
echo "Next steps:"
echo "1. Update src/main.rs to remove imports for:"
echo "   - mod enemy; (use only 'mod enemies;')"
echo "   - mod app;"
echo "   - mod input;" 
echo "   - mod graphics;"
echo ""
echo "2. Update src/terrain/mod.rs to remove module declarations for deleted files"
echo ""
echo "3. Run 'cargo build' to ensure everything still compiles"
echo ""
echo "4. Consider removing the following low-priority items if not needed:"
echo "   - src/enemies/factory.rs"
echo "   - src/enemies/core.rs"
echo "   - src/terrain/cache.rs"
echo "   - src/sounds/thruster.rs"