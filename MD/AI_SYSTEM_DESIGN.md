# Network-Ready AI System Design

## Overview
This document outlines an AI architecture designed specifically to integrate with the **Online Subsystem AAA Design (2026)**.
The core requirement is that AI must be **Server-Authoritative** but **Client-Predictable** to minimize bandwidth. Syncing 100 zombies by sending their `Transform` every frame is inefficient; we must sync their **Intent**.

---

## 1. Core Architecture: The "Hybrid Brain"

To support both the **Game Director** (Dynamic Difficulty) and standard mob behavior, we use a two-layer system.

### 1.1 Strategy Layer (Utility AI)
**"What should I do?"**
*   Evaluates high-level needs based on World State (e.g., Health, Squad Morale, Game Director commands).
*   **Output:** A Goal (e.g., `Goal::FlankPlayer`, `Goal::Retreat`).
*   **Frequency:** Low (every 1-2 seconds).
*   **Integration:** The **Game Director** can override this layer directly (e.g., force all mobs to `Goal::Rush`).

### 1.2 Tactics Layer (Behavior Tree)
**"How do I do it?"**
*   executes the concrete actions to achieve the Strategy Goal.
*   **Nodes:** `MoveTo(Target)`, `PlayAnim(Attack)`, `Wait(2s)`.
*   **Frequency:** Medium (10-20Hz).

---

## 2. Online Subsystem Integration

### 2.1 The "Intent Sync" Protocol
Instead of syncing `(x,y,z)` every tick, we sync the **AI Command**.

**Component:** `AiNetworkState`
```rust
#[derive(Component, Serialize, Deserialize)]
struct AiNetworkState {
    pub current_action: ActionType, // e.g., Moving, Attacking, Stunned
    pub target_entity: Option<EntityId>,
    pub target_position: Vec3,
    pub movement_speed: f32,
    pub server_tick: u32,
}
```

**Replication Logic:**
1.  **Server:** Decides "Zombie A attacks Player B".
    *   Sends Packet: `[ID: 101, Action: MoveTo, Target: PlayerB, Speed: 4.5]`
2.  **Client:** Receives packet.
    *   Finds Entity 101.
    *   Triggers local pathfinding: `NavMesh.Path(ZombiePos, PlayerPos)`.
    *   Plays "Run" animation.
3.  **Correction:**
    *   Server sends actual Position every 1-2 seconds (or if error > threshold) to fix drift.

### 2.2 Determinism & Navigation
For "Intent Sync" to work, Client and Server need the same **Navigation Mesh**.
*   **Deterministic Pathfinding:** Use a seed-based approach or standardized `Detour/Recast` library usage so both sides calculate roughly the same path.
*   **Client prediction:** If the server says "Move to X", the client can simulate the movement instantly (Client-side Prediction) so the zombie doesn't "skate".

---

## 3. Performance & Scalability (LOD)

To support 100+ AIs online, we implementation **AI LOD (Level of Detail)**.

### 3.1 LOD Buckets
*   **LOD 0 (Close to Player < 20m):**
    *   Tick Logic: Every Frame.
    *   Anim: Full Skeleton.
    *   Sync: High Priority (Intent + Transforms).
*   **LOD 1 (Medium Range 20m - 50m):**
    *   Tick Logic: 10Hz.
    *   Anim: Simplified (Root motion only).
    *   Sync: Medium (Intent updates mostly).
*   **LOD 2 (Far Range > 50m):**
    *   Tick Logic: 1Hz (Utility Only).
    *   Anim: None (Capsule slide).
    *   Sync: Low (Only status changes).

### 3.2 Time-Sliced Processing
Don't update all 100 AIs in the same frame.
*   Frame 1: Update AIs 0-9.
*   Frame 2: Update AIs 10-19.
*   ...
This keeps the Server MSPT (Milliseconds per Tick) stable.

---

## 4. ECS Implementation Strategy

```rust
// The Brain
#[derive(Component)]
struct AiAgent {
    pub config: Handle<AiConfig>, // Stats, BehaviorTree Asset
    pub brain_state: UtilityState, // Hunger, Aggro, etc.
    pub current_goal: AiGoal,
}

// Logic System
fn ai_decision_system(
    mut query: Query<(&mut AiAgent, &Transform)>,
    time: Res<Time>,
    lod_map: Res<AiLodMap>
) {
    for (agent, transform) in query.iter_mut() {
        if !should_update(agent, lod_map) { continue; }
        
        // 1. Utility Check (Strategy)
        // 2. Behavior Tree Tick (Tactics)
        // 3. Update NetworkState component
    }
}
```

## 5. Comparison: Why this Design?

| Feature | Standard AI | **Network-Ready (This Design)** |
| :--- | :--- | :--- |
| **Sync** | Sync Transform | Sync **Intent/Goal** |
| **Bandwidth** | High (Position spam) | **Low** (Events only) |
| **Smoothness** | Jittery on lag | **Smooth** (Client simulates path) |
| **Scale** | ~20 Agents | **100+ Agents** (via LOD/Time-Slice) |

This design perfectly matches the `ONLINE_SUBSYSTEM_AAA_DESIGN_2026.md` goal of "Bandwidth Management" and "Mobile First".
