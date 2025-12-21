# Dual Physics System Implementation Plan (2D & 3D)

## Objective
Implement a robust physics architecture supporting both 2D (Rapier2D) and 3D (Rapier3D) simulations. This system will serve as the foundation for the "Mobile AAA" destruction system using asset swapping.

## Architecture Support

The engine will support two distinct physics modes, activated based on the components present or a global setting.

### 1. Component Separation
We will explicitly separate 2D and 3D physics components to avoid ambiguity and performance overhead.

| Feature | 2D Component | 3D Component |
| :--- | :--- | :--- |
| **Body** | `Rigidbody2D` (Existing) | `Rigidbody3D` (New) |
| **Collision** | `Collider` (Rename to `Collider2D` recommended, or alias) | `Collider3D` (Existing, needs expansion) |

### 2. Physics Backend
The `physics` crate will oversee two internal modules:
- `backend_2d`: Wraps `rapier2d`.
- `backend_3d`: Wraps `rapier3d`.

The `PhysicsSystem` in the ECS will decide which backend to tick:
- If `ViewMode` is 2D: Tick `backend_2d`.
- If `ViewMode` is 3D: Tick `backend_3d`.
- (Optional) Tick both if mixed content is required, but generally we toggle based on game mode.

---

## Implementation Steps

### Phase 1: Dependencies & Components

1.  **Update `physics/Cargo.toml`**:
    - Add `rapier3d` dependency (optional/feature-gated or standard).
    - Ensure `rapier2d` remains available.

2.  **Create `Rigidbody3D` Component** (`ecs/src/components/rigidbody_3d.rs`):
    ```rust
    pub struct Rigidbody3D {
        pub velocity: [f32; 3],
        pub angular_velocity: [f32; 3],
        pub gravity_scale: f32,
        pub mass: f32,
        pub is_kinematic: bool,
        pub drag: f32,
        pub angular_drag: f32,
        pub freeze_position: [bool; 3],
        pub freeze_rotation: [bool; 3],
    }
    ```

3.  **Enhance `Collider3D` Component**:
    - Add `Mesh` shape support (for complex destruction debris).
    - Add `convex_hull` support (optimized mesh collision).

4.  **Register in `CustomWorld`**:
    - Add `rigidbodies_3d: HashMap<Entity, Rigidbody3D>` to `ecs/src/lib.rs`.

### Phase 2: Physics Backend (The "Dual Core")

1.  **Refactor `physics/src/lib.rs`**:
    - Create a trait `PhysicsBackend` (optional, or just distinct structs).
    - Move existing `RapierPhysicsWorld` to `rapier_2d_backend.rs`.

2.  **Create `physics/src/rapier_3d_backend.rs`**:
    - Port the logic from the 2D backend but map to 3D.
    - **Crucial Change**: map Engine (X, Y, Z) directly to Rapier3D (X, Y, Z).
    - Handling coordinate systems:
        - Engine: Y-up (likely), Right-handed.
        - Rapier: Y-up (standard).
    - Syncing: Update `Transform.position` (Vec3) and `Transform.rotation` (Quat) from physics simulation.

### Phase 3: Physics System Integration

1.  **Update `engine/src/lib.rs` or `runtime/systems.rs`**:
    - Instantiate both `physics_world_2d` and `physics_world_3d`.
    - In the update loop:
        ```rust
        if game_mode == Mode2D {
            physics_world_2d.step(dt, world);
        } else {
            physics_world_3d.step(dt, world);
        }
        ```

### Phase 4: Destruction Logic (Pre-fractured Swap)

1.  **Create `Destructible` Component**:
    ```rust
    pub struct Destructible {
        pub broken_prefab_id: String, // e.g., "pot_broken"
        pub health: f32,
        pub debris_force: f32,
    }
    ```

2.  **Implement `DestructionSystem`**:
    - Check for events (e.g., Collision, Damage).
    - If `health <= 0`:
        - Get Transform of current entity.
        - Despawn current entity.
        - Instantiate `broken_prefab_id` at Transform.
        - Iterate through children of new instance (debris pieces).
        - Apply random `Impulse` via `Rigidbody3D` to simulate explosion.

---

## Roadmap Checklist

- [ ] Add `rapier3d` to `physics` crate dependencies.
- [ ] Implement `Rigidbody3D` struct in `ecs`.
- [ ] register `Rigidbody3D` in `World`.
- [ ] Implement `Rapier3DBackend` in `physics`.
- [ ] Update `Engine` update loop to support 3D physics ticking.
- [ ] Create `Destructible` component and system.

This plan ensures "Mobile AAA" quality by using the performant pre-fractured swap method and upgrading the engine to support true 3D physics.
