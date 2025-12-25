# AAA Tilemap & LDtk Integration Workflow

This document outlines the standard workflow for creating, importing, and managing tilemaps in the engine, designed to mimic AAA industry standards (similar to Unity/Godot).

## 1. Core Architecture

The engine treats Tilemaps as a composition of specialized components organized in a specific hierarchy.

### 1.1 Entity Hierarchy
When a Map is loaded, it creates a structured hierarchy:

*   📦 **Root Entity ("Level_Name")**
    *   `Grid Component`: Manages cell size, orientation, and coordinate conversion for all child layers.
    *   `LdtkAssetRef`: Links the entity to the source `.ldtk` file for hot-reloading.
    *   `Transform`: World position (usually 0,0,0).
    *   📂 **Child Entity ("Visual_Layer")**
        *   `Tilemap`: Stores raw tile data (positions, tile IDs).
        *   `TilemapRenderer`: Handles rendering, referencing the Tileset/Atlas.
    *   📂 **Child Entity ("Collisions")**
        *   `TilemapCollider` / `CompositeCollider2D`: Manages physics boundaries.
    *   📂 **Child Entity ("Entities")**
        *   Standard GameObjects spawned from LDtk Entity markers.

## 2. LDtk Setup Guide

### 2.1 Project Settings
*   **Grid Size**: Standardize your grid (e.g., 16px, 32px).
*   **Extension**: Save as `.ldtk`.
*   **Path**: Save in `tilemaps/` folder. Texture assets should be in `assets/`.

### 2.2 Layer Configuration
To ensure the engine interprets your map correctly, follow these layer conventions:

1.  **Physics Layers (IntGrid)**
    *   **Type**: IntGrid
    *   **Name**: Any (e.g., "Collisions", "Ground")
    *   **Rule**: Mark cells as Value `1` to create **Solid Colliders**.
    *   **Result**: Generates logic-only entities with Collider components.

2.  **Visual Layers (Tiles)**
    *   **Type**: Tiles
    *   **Name**: Any (e.g., "Foreground", "Background")
    *   **Result**: Generates entities with `TilemapRenderer`.

3.  **Gameplay Layers (Entities)**
    *   **Type**: Entities
    *   **Usage**: Spawn points, enemies, items.
    *   **Result**: Instantiates GameObjects with `LdtkEntity` data fields.

## 3. Import Workflow

### 3.1 Importing via Editor
*Current Implementation via Maps Panel*

1.  **Open Maps Panel**: Go to **View** -> **Panels** -> **Maps**.
2.  **Add Map**: Click "Add Map" and select your `.ldtk` file.
    *   *Note: This registers the file as a map asset.*
3.  **Instantiate**: Click **Load** (📂) next to the Import entry.
    *   This spawns the **Root Grid Entity** and its children into the Scene.

### 3.2 Asset Configuration (Inspector)
*Future Implementation Goal*

*   Select the `.ldtk` file in the **Asset Browser**.
*   The **Inspector** will show Import Settings:
    *   **Pixels Per Unit**: Define world-scale (e.g., 16 or 100).
    *   **Generate Colliders**: Toggle physics generation.
    *   **Sorting Order**: Base Render Layer.

## 4. Hot Reloading
The engine enables rapid iteration:
1.  Keep the Game/Editor running.
2.  Modify the map in **LDtk**.
3.  **Save** (Ctrl+S).
4.  The Engine detects the file change and **automatically reloads** the Grid and all layers.

## 5. Troubleshooting
*   **Scaling Issues**: Ensure the LDtk `Grid Size` matches your intended engine PPU settings.
*   **Missing Colliders**: Verify IntGrid Value is exactly `1`.
*   **Black Screen**: Ensure the `TilemapRenderer` layers are not obstructed or at wrong Z-depth.
