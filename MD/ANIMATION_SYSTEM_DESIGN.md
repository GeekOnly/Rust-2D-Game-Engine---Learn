# Unified Animation System Design (2D + 3D)

## Overview
This document outlines the design for a **Unified Animation System** that powers both high-performance 2D Sprite animations and complex 3D Skeletal animations.
The design draws inspiration from **Unity's Animator Controller** (for its clean State Machine flow) and **Unreal's Blend Spaces** (for smooth parameter blending), adapted for a **Rust ECS** architecture.

---

## 1. Core Architecture: Separation of Concerns

To support BOTH 2D and 3D equally well, we separate the **"Navigation"** (Logic) from the **"Presentation"** (Pose).

### 1.1 The Logic Layer: `AnimatorController`
This is the "Brain". It looks exactly like Unity's Animator window.
*   **State Machine:** Nodes (States) connected by Transitions.
*   **Parameters:** `Speed (float)`, `IsGrounded (bool)`, `Attack (trigger)`.
*   **Output:** It does NOT output bones. It outputs a **"Sampling Request"**.
    *   *Example:* "We are in State 'Run', at time 0.5s, with weight 1.0."
    *   *Example (Blending):* "We are blending 'Walk' (0.3) and 'Run' (0.7)."

### 1.2 The Data Layer: `AnimationClip`
Abstracted storage for animation data.
*   **3D Skeletal Clip:** Stores curves (`Translation`, `Rotation`, `Scale`) for each bone.
*   **2D Sprite Clip:** Stores a list of `SpriteIndex` and `Duration`.
*   **Property Clip:** Stores generic curves for float properties (e.g., Light Intensity, Alpha).

### 1.3 The Presentation Layer: `AnimationBackend`
Systems that read the "Sampling Request" and apply it to components.
*   **SpriteBackend:** Changes the `Sprite` component's texture/index.
*   **SkeletalBackend:** Calculates Bone Matrices, sends them to the GPU Skinning Buffer.

---

## 2. Feature Deep Dive

### 2.1 State Machines (Unity Style)
We use a Graph-based approach stored as an Asset.

*   **States:** A generic container. Can hold a *Clip*, a *Blend Tree*, or another *Sub-StateMachine*.
*   **Transitions:** Rules to switch states.
    *   `condition: Speed > 0.1` -> Transition `Idle` to `Walk`.
    *   `duration: 0.2s` (Crossfading).
*   **Layering:** Support multiple layers (e.g., "Legs" doing walking, "Arms" doing shooting). Masking for 3D (AvatarMask).

### 2.2 Blend Trees (Unreal Style)
Essential for 3D, useful for 2D (e.g., 8-direction movement).
*   **1D Blend:** Input `Speed`. Nodes: `Idle (0)`, `Walk (1)`, `Run (5)`. System calculates weights (e.g., Speed 3 = 50% Walk, 50% Run).
*   **2D Blend (Cartesian):** Input `X, Y`. Used for Strafe movement.

### 2.3 3D Specifics: Skeletal Skinning
*   **Bone Hierarchy:** Defined in `Skeleton` asset.
*   **Skinning:**
    *   **CPU:** Interpolate Keyframes -> Compute Local Matrix -> Multiply Parent Matrix -> Compute Inverse Bind Pose -> Output `FinalMatrices`.
    *   **GPU:** Vertex Shader receives `FinalMatrices` array. Weights/Indices are in Vertex Buffer.

---

## 3. ECS Integration Strategy

We avoid the "OO Heavy" update. We utilize **Parallel Systems**.

### 3.1 Components
```rust
// The Controller (Logic)
#[derive(Component)]
struct Animator {
    pub controller_asset: Handle<GraphAsset>,
    pub parameters: HashMap<String, ParameterValue>,
    pub current_state: StateHandle,
    pub active_transitions: Vec<TransitionState>,
}

// 3D Skeleton
#[derive(Component)]
struct SkinnedMesh {
    pub skeleton: Handle<Skeleton>,
    pub bone_entities: Vec<Entity>, // Map BoneID -> EntityID
}

// 2D Animation
#[derive(Component)]
struct SpriteAnimator {
    // Usually just uses the generic Animator, 
    // but might need specific sprite flipping logic
    pub flip_x: bool,
}
```

### 3.2 Systems Pipeline
1.  **`animator_logic_system`**:
    *   Reads `Animator` parameters and `DeltaTime`.
    *   Advances State Machine (Check transitions, update timers).
    *   Outputs a `AnimationEvaluation` list (e.g., `[(ClipA, 0.5s, 0.8), (ClipB, 1.2s, 0.2)]`).
2.  **`animation_sample_3d_system`** (Parallel):
    *   Reads `AnimationEvaluation`.
    *   Samples curves from `SkeletalClips`.
    *   Blends results (LERP/SLERP).
    *   Writes to `Transform` components of Bones.
3.  **`animation_sample_2d_system`** (Parallel):
    *   Reads `AnimationEvaluation`.
    *   Samples `SpriteClips`.
    *   Writes to `Sprite` component.

---

## 4. Comparison & Choice

| Feature | Unity (Mecanim) | Unreal (Persona) | **XS Engine (Proposed)** |
| :--- | :--- | :--- | :--- |
| **Logic** | State Machine | Blueprint Graph | **State Machine** (Clean, Deterministic) |
| **Blending** | Blend Tree | Blend Space | **Blend Tree** (1D/2D) |
| **Control** | Parameters | Variables | **Parameters** (ECS Component) |
| **Perf** | C++ overhead | Fast | **Rust ECS** (Parallel Sampling) |

**Verdict:** We choose the **Unity-like State Machine** model because it is easier to visualize and debug for gameplay logic, but we implement the **Blending Math** akin to Unreal for high-quality motion.
