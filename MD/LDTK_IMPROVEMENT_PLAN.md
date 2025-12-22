# LDtk Integration Improvement Plan

## Overview
This plan outlines the steps to refactor and improve the LDtk integration in the Game Engine. The main goals are to switch from manual JSON parsing to a type-safe struct-based approach, enable Entity loading, and support dynamic grid sizing.

## Phase 1: Refactoring Core Loader
- [ ] **Review `LdtkMap` Structs**: Verify `ecs/src/components/ldtk_map.rs` matches the latest LDtk JSON schema, particularly for `LayerInstance` and `EntityInstance`.
- [ ] **Refactor `ldtk_loader.rs`**: 
    - Rewrite `load_project_with_grid` to deserialize the JSON into the `LdtkMap` struct immediately using `serde_json::from_str::<LdtkMap>`.
    - Replace all `project["key"]` manual lookups with safe struct field access.
- [ ] **Dynamic Scale**: 
    - Remove hardcoded `pixels_per_unit = 8.0`.
    - Use `LdtkMap.default_grid_size` to determine the world scale (`pixels_per_unit`).

## Phase 2: Entity Support Implementation
- [ ] **Create `LdtkEntity` Component**: 
    - Define a new component in `ecs/src/components/ldtk_entity.rs` (or similar) to store raw entity data (`identifier`, `iid`, `fields`).
- [ ] **Implement Entity Layer Parsing**:
    - In `ldtk_loader.rs`, add a loop to process layers of type `Entities`.
    - Iterate over `entityInstances` and spawn generic Entities with:
        - `Transform` (converted to engine coordinates).
        - `LdtkEntity` component containing metadata.
        - `Name` component (e.g., "PlayerStart", "Enemy").
- [ ] **Tagging System**: Add a mechanism to tag these entities so systems can process them (e.g., a `SpawnPoint` system can look for entities with `LdtkEntity` identifier="PlayerStart").

## Phase 3: Physics & Collider Improvements
- [ ] **Refactor Collider Generation**:
    - Update `generate_composite_colliders_from_intgrid` to use the `LdtkMap` struct.
    - Ensure it respects the dynamic `pixels_per_unit` calculated from the map.
- [ ] **Layer Filtering**:
    - Ensure the loader can distinguish between "Visual" IntGrids and "Collider" IntGrids (e.g., by matching Layer name "IntGrid_Collider").

## Phase 4: Validation & Workflow
- [ ] **Test Loading**: Load a sample map containing Tilesets, IntGrid Colliders, and Entities.
- [ ] **Test Hot Reload**: Verify that modifying an Entity position in LDtk and saving updates the position in the engine runtime immediately.
- [ ] **Coordinate Check**: Verify specifically that Y-coordinates align correctly between LDtk (Top-Left origin) and Engine (Bottom-Left origin).
- [ ] **Update Celeste Demo**:
    - Update `projects/Celeste Demo/scenes/main.json` to properly invoke the new LDtk loader logic.
    - Ensure the scene defines the correct LDtk file path and settings to demonstrate the new capabilities (Entities, Colliders).

## Future Consideration (Post-Refactor)
- **Entity Factory Pattern**: Implement a registry to map LDtk Entity Identifiers to specific Rust constructor functions (e.g., `PlayerStart` -> `components::Player::spawn`).
