# Tilemap Management User Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Loading Maps](#loading-maps)
4. [Managing Layers](#managing-layers)
5. [Collider Configuration](#collider-configuration)
6. [Hot-Reload Workflow](#hot-reload-workflow)
7. [Performance Monitoring](#performance-monitoring)
8. [Keyboard Shortcuts](#keyboard-shortcuts)
9. [Troubleshooting](#troubleshooting)

## Introduction

The Tilemap Management System provides professional-grade tools for working with LDtk tilemap files in your game engine. It features automatic collider generation, layer management, hot-reload support, and real-time performance monitoring.

### Key Features
- 🗺️ Load and manage multiple LDtk maps simultaneously
- 🎨 Edit layer properties (position, rotation, scale, visibility)
- 🔨 Automatic collider generation from IntGrid layers
- 🔄 Hot-reload support for seamless LDtk workflow
- 📊 Real-time performance monitoring
- ⚙️ Configurable collider generation settings

## Getting Started

### Prerequisites
- LDtk editor installed (https://ldtk.io/)
- Project with .ldtk files in the project directory

### Opening the Tilemap Panels

The tilemap management system consists of 4 main panels:

1. **Maps Panel** (🗺️): Load and manage map files
2. **Layer Properties Panel** (🎨): Edit individual layer properties
3. **Layer Ordering Panel** (📑): Reorder layers with drag & drop
4. **Performance Panel** (📊): Monitor performance metrics

All panels are available in the default dock layout. You can also access them from the View menu.

## Loading Maps

### Method 1: Browse for Files

1. Open the **Maps Panel** (🗺️)
2. Click **"Add Map"** button
3. Browse to your .ldtk file
4. The map will load automatically with:
   - Grid Entity as root
   - All layers as children
   - Auto-generated colliders from IntGrid layers

### Method 2: Auto-Scan Project

1. Place .ldtk files anywhere in your project directory
2. Click **"Refresh"** in the Maps Panel
3. Available files will appear in the list
4. Click on a file to load it

### Understanding the Hierarchy

When a map loads, it creates this structure:
```
Grid Entity (root)
├── Layer 1 (IntGrid)
├── Layer 2 (Tiles)
├── Layer 3 (Entities)
└── Collider Entities (auto-generated)
```

The Grid Entity acts as a container. Despawning it automatically removes all children.

## Managing Layers

### Visibility Control

**From Maps Panel**:
- Click the eye icon (👁) next to a layer to toggle visibility
- Hidden layers show as grayed out

**From Layer Properties Panel**:
- Select a layer
- Use the visibility checkbox in the Rendering section

**Keyboard Shortcut**:
- Select a layer
- Press **Ctrl+H** to toggle visibility

### Editing Layer Properties

1. Select a layer in the Maps Panel or Hierarchy
2. Open the **Layer Properties Panel** (🎨)
3. Edit properties:
   - **Transform**: Position, rotation, scale
   - **Rendering**: Visibility, Z-Order, opacity, color tint
   - **Info**: View tilemap size, tileset, memory usage

### Reordering Layers

**Method 1: Drag & Drop**
1. Open the **Layer Ordering Panel** (📑)
2. Select a map from the dropdown
3. Click and drag layers to reorder
4. Z-Orders update automatically

**Method 2: Move Buttons**
1. Select a layer in the Layer Ordering Panel
2. Click **⬆ Move Up** to increment Z-Order by 1
3. Click **⬇ Move Down** to decrement Z-Order by 1
4. Minimum Z-Order: -100

**Z-Order Rules**:
- Higher Z-Order renders on top
- Layers are ordered bottom to top in the list
- Z-Orders are monotonically increasing

## Collider Configuration

### Understanding Collider Types

**Composite (Recommended)**:
- Merges adjacent tiles into larger shapes
- Reduces collider count by 70-90%
- Best performance for most use cases

**Individual**:
- Creates one collider per tile
- Higher collider count
- Use for precise per-tile collision

**Polygon** (Not Yet Implemented):
- Advanced polygon shapes
- Coming in future update

### Configuring Colliders

1. Open the **Collider Settings Panel** (⚙️)
2. Choose collider type
3. Set collision value (default: 1)
   - Only IntGrid tiles with this value generate colliders
   - Set different values in LDtk for different behaviors
4. Toggle auto-regenerate on reload
5. Click **"Apply Settings"**

### Manual Collider Operations

**Regenerate Colliders**:
- Select a map in Maps Panel
- Click **"Regenerate Colliders"**
- Or press **Ctrl+Shift+R**

**Clean Up Colliders**:
- Remove colliders for specific map: Click **"Clean Up Colliders"**
- Remove all colliders: Click **"Clean Up All"**

## Hot-Reload Workflow

Hot-reload automatically detects and reloads changed .ldtk files.

### Enabling Hot-Reload

1. Open **Collider Settings Panel** (⚙️)
2. Enable **"Auto-regenerate on reload"**
3. Hot-reload is now active

### Workflow

1. Load a map in the editor
2. Edit the map in LDtk
3. Save in LDtk (Ctrl+S)
4. Map automatically reloads in editor
5. Layer visibility states are preserved
6. Colliders regenerate automatically (if enabled)

### Hot-Reload Features

- **Detection**: Changes detected within 1 second
- **State Preservation**: Visibility and Z-Order maintained
- **Error Recovery**: Corrupted files preserve last valid state
- **Debouncing**: Ignores rapid successive changes

## Performance Monitoring

### Opening Performance Panel

1. Open the **Performance Panel** (📊)
2. View real-time metrics:
   - Draw calls, triangles, vertices
   - Memory usage (tilemap, texture, collider)
   - Entity counts

### Understanding Metrics

**Rendering Metrics**:
- **Draw Calls**: Number of render batches (lower is better)
- **Triangles/Vertices**: Geometry complexity

**Memory Metrics**:
- **Tilemap Data**: Memory used by tilemap components
- **Texture Memory**: Memory used by tileset textures
- **Collider Memory**: Memory used by physics colliders

**Warning Indicators**:
- ⚠ Yellow warning appears when threshold exceeded
- Adjust thresholds in the settings section

### Optimization Tips

1. **Use Composite Colliders**: Reduces collider count by 70-90%
2. **Unload Unused Maps**: Free memory by unloading maps you're not editing
3. **Hide Layers**: Hide layers you're not currently working on
4. **Limit Loaded Maps**: Keep number of loaded maps reasonable (10 or fewer)
5. **Optimize Tilesets**: Use smaller tileset textures when possible

## Keyboard Shortcuts

### Tilemap Management
- **Ctrl+R**: Reload selected map
- **Ctrl+Shift+R**: Regenerate colliders for selected map
- **Ctrl+H**: Toggle visibility of selected layer

### General Editor
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Ctrl+A**: Select all
- **Escape**: Clear selection
- **Delete**: Delete selected entities
- **Ctrl+C**: Copy
- **Ctrl+V**: Paste
- **Ctrl+D**: Duplicate
- **Ctrl+X**: Cut
- **Ctrl+G**: Toggle snapping
- **Ctrl+Shift+G**: Toggle grid

## Troubleshooting

### Map Won't Load

**Problem**: "File not found" error

**Solution**:
- Verify .ldtk file exists in project directory
- Check file path is correct
- Click "Refresh" to rescan project

**Problem**: "Invalid format" error

**Solution**:
- Verify file is a valid .ldtk file
- Check LDtk version compatibility (1.5.3+)
- Try opening file in LDtk to verify it's not corrupted

### Colliders Not Generating

**Problem**: No colliders appear after loading map

**Solution**:
- Check IntGrid layer has tiles with collision value 1
- Verify collision value in Collider Settings matches LDtk
- Try manually regenerating with "Regenerate Colliders"

**Problem**: Too many colliders generated

**Solution**:
- Switch to Composite collider type
- Check collision value is correct (not generating for wrong tiles)

### Hot-Reload Not Working

**Problem**: Changes in LDtk don't reload automatically

**Solution**:
- Verify hot-reload is enabled in Collider Settings
- Check file watcher is active (look for console messages)
- Try manual reload with Ctrl+R

**Problem**: Hot-reload fails with error

**Solution**:
- Check console for error message
- Verify .ldtk file is not corrupted
- Last valid state is preserved - fix file and reload manually

### Performance Issues

**Problem**: Low frame rate with multiple maps

**Solution**:
- Check Performance Panel for metrics
- Unload unused maps
- Use Composite colliders
- Hide layers you're not editing
- Reduce number of loaded maps

**Problem**: High memory usage

**Solution**:
- Unload unused maps
- Check for memory leaks (entity counts should decrease after unload)
- Optimize tileset texture sizes

## Advanced Topics

### Extending the System

The tilemap management system is designed to be extensible. See `API_DOCUMENTATION.md` for details on:
- Custom collider generation algorithms
- Custom layer property editors
- Integration with other systems

### Project Settings

Tilemap settings are stored in `.kiro/settings/tilemap.json`:
```json
{
  "auto_generate_colliders": true,
  "collision_value": 1,
  "collider_type": "Composite",
  "hot_reload_enabled": true,
  "pixels_per_unit": 8.0
}
```

These settings persist across editor sessions.

## Getting Help

- Check the in-panel help sections (ℹ️ Help)
- Review keyboard shortcuts (see above)
- Check console for error messages
- Refer to `API_DOCUMENTATION.md` for technical details

## Conclusion

The Tilemap Management System provides a complete workflow for working with LDtk maps. With automatic collider generation, hot-reload support, and real-time performance monitoring, you can iterate quickly and efficiently on your levels.

Happy level designing! 🎮
