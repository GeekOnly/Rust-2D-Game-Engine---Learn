# AAA Online Subsystem Design (2026 Vision)

## Overview
This document outlines the architecture for a "AAA Mobile-First" Online Subsystem, designed to rival the capabilities of Unreal Engine 5's networking and Roblox's massive scalability.
The core goal is **Seamless Synchronization** of complex, dynamic worlds (Voxels, Physics, AI) over potentially unstable mobile networks.

---

## 1. Core Architecture: "Authoritative Server with Client Prediction"

To prevent cheating and ensure consistency, the Server is the single source of truth. However, to mask latency (Lag), Clients must "Predict" the future.

### 1.1 The ECS Network Tick
Unlike traditional OOP networking, we utilize the ECS data layout for cache-efficient replication.

*   **Fast Path (Unreliable/UDP):** Physics, Transform, Input (60Hz or higher).
*   **State Path (Reliable/TCP/Fragmented):** Inventory, Chat, Terrain Modifications, AI State changes.

### 1.2 Replication Graph (The Scalability Secret)
Inspired by Unreal Engine, we implement a **Replication Graph**. Instead of iterating every entity for every client ($O(N \times M)$), we pre-sort entities into "Nodes":
*   **Spatial Node:** Quadtree/Octree of entities. If a player is far, we don't even check these entities.
*   **Always Relevant Node:** Key game states (Score, Time).
*   **Team Node:** Entities only visible to teammates.

---

## 2. Advanced System Synchronization

### 2.1 Physics & Rigidbodies (Rocket League Style)
Physics is the hardest to sync due to "The Butterfly Effect".
*   **Technique:** **buffer-based Snapshot Interpolation** for remote objects, **Prediction + Rollback** for the local player.
*   **Optimization:**
    *   **Quantization:** Compress Position (Vec3 f32) to smaller integers relative to a "Base" position.
    *   **Orientation:** Use "Smallest Three" Quaternions compression (drop the largest component, reconstruct it).
    *   **Sleeping:** If a body stops moving, send a "Sleep" packet and stop replicating until it wakes up.

### 2.2 Voxel World & Dynamic Terrain
Syncing 1,000,000 modified blocks is impossible if done naively.
*   **Technique:** **Chunk-based Delta Compression**.
*   **Protocol:**
    1.  **Initial Load:** Stream compressed Chunks (RLE/LZ4) via a separate reliable channel.
    2.  **Modification:** When a player digs/builds, send an **Action** (e.g., `Explosion(Pos, Radius)`) instead of sending 500 changed block IDs. Both Server and Client execute the logic.
    3.  **Hash Check:** Periodically, Server sends `Hash(ChunkID)`. If Client's hash differs, re-request that specific chunk (Correction).

### 2.3 AI & NPCs
Syncing 100 zombies moving is bandwidth heavy.
*   **Deterministic AI (Preferred):** Sync the "State" and "Target", not the Transform every frame.
    *   *Server Sends:* `ZombieID: State=Chase, Target=PlayerID, Speed=1.2`
    *   *Client:* Runs pathfinding locally. The zombie moves smoothly.
    *   *Correction:* Occasionally sync actual Position to prevent drift.
*   **LOD (Level of Detail) Network:**
    *   **Near:** Full Transform Sync (20Hz).
    *   **Far:** Low Frequency (2Hz) + Interpolation.
    *   **Very Far:** Stop syncing transforms, just show dots on map or hide.

### 2.4 Destructible Environments
For "Battlefield-style" destruction (Mesh Fracturing):
*   **Seed Synchronization:**
    *   Do not sync 50 pieces of debris.
    *   Server sends: `Event: Break, EntID: 55, Seed: 12345, Force: (10, 2, 0)`.
    *   Client uses `Seed: 12345` to fracture the mesh locally. The debris flies in the *exact same way* on all clients because the math is deterministic.

---

## 3. Bandwidth Management & "Mobile First"

Mobile data is unstable (Packet Loss, Jitter).
*   **Priority Accumulator:** Every replicated entity has `Priority`. Every frame, `Priority += DeltaTime`. When `Priority >= Threshold`, send update and reset `Priority`.
    *   *Result:* Closer enemies update faster (60Hz). Far enemies update slower (5Hz).
*   **Adaptive Frequency:** Detect client RTT/Packet Loss. If connection is bad, automatically reduce update rate of non-critical objects (Particles, Decorations) to save bandwidth for Gameplay.

## 4. Cheat Protection (AAA Standard)
*   **Server-Side Rewind (Lag Compensation):** When a player shoots, Server rewinds "Hitbox" history to where the player *saw* the enemy X ms ago to check the hit.
*   **Movement Validation:** Server checks `Distance(Pos_Current, Pos_Last)`. If > `MaxSpeed * DeltaTime`, Rubberband the player back.

## 5. Integration Roadmap
1.  **Core Transport:** Reliable/Unreliable UDP abstraction (completed plan).
2.   **Replication Graph:** Spatial hashing implementation.
3.   **Prediction System:** Physics rollback queue.
4.   **Specialized Systems:** Voxel/Terrain Delta Sync handlers.

---

## 6. Industry Comparison Analysis (Why this architecture?)

We benchmarked the **XS Engine Online Subsystem** architecture against the industry leaders to ensure we are building a "Best in Class" solution.

| Feature | **XS Engine (Proposed)** | **Unreal Engine 5** | **Unity Netcode (NGO)** | **FishNet (Unity Plugin)** | **Roblox** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Architecture** | **ECS / Data-Oriented** | Actor Model / OOP | OOP (MonoBehaviour) | OOP (Optimized) | Proprietary (Part-based) |
| **Replication Logic** | **Snapshot + Replication Graph** | Replication Graph | NetworkVars (Dirty Check) | Tick-based Sync | Automatic (Low control) |
| **Physics** | **Deterministic / Rollback** | Chaos (Authoritative) | Client Auth / Server Auth | Client Side Prediction (CSP) | Distributed Physics (Complex) |
| **Scalability** | **High** (Target: 100+ players) | High (Fortnite proven) | Low-Medium (CCU limit) | High (Bandwidth optimized) | **Massive** (Dynamic Sharding) |
| **Mobile Focus** | **Native (Packet Loss Aware)** | Heavy (High Bandwidth) | Moderate | Excellent (Bandwidth Saver) | Excellent (Streaming) |
| **Development** | **High Control (Rust)** | High Control (C++) | Easy (C#) | Easy-Medium (C#) | Very Easy (Lua) |

### 6.1 vs Unreal Engine 5
*   **Strengths:** UE5 has the gold standard "Replication Graph" which we are adopting. Its prediction system (CharacterMovementComponent) is battle-tested.
*   **Constraint:** UE5 is heavy. It assumes high-end hardware. Our XS Engine design strips away the bloat (Reflection, Blueprints overhead) to run blazing fast on Mobile via Rust's zero-cost abstractions.

### 6.2 vs Unity Netcode for GameObjects (NGO)
*   **Strengths:** NGO is easy to learn but suffers from performance issues at scale due to GameObject overhead.
*   **XS Advantage:** By using **ECS**, we avoid the "GameObject tax". Network serialization in Rust (`serde` / `bincode`) is significantly faster than C# Reflection used in Unity.

### 6.3 vs FishNet (Best Unity Networking)
*   **Strengths:** FishNet is famous for its "Bandwidth Management" (bits packing) and CSP (Client Side Prediction).
*   **Adoption:** We are implementing FishNet's **"Priority Accumulator"** and **"LOD"** concepts directly into our ECS systems to achieve similar bandwidth efficiency.

### 6.4 vs Roblox
*   **Strengths:** Roblox handles **"Infrastructure"** magically. Developers just press "Play".
*   **XS Goal:** We emulate this via our **Kubernetes + Agones** infrastructure plan. However, we give developers *more control* over the networking logic (e.g., custom rollback) which Roblox hides, allowing for competitive eSports-grade mechanics that Roblox struggles with.

### Conclusion
Our design is a **Hybrid**:
1.  **ECS Performance** of Overmatch/Bevy.
2.  **Scalability Features** (Replication Graph) of **Unreal**.
3.  **Bandwidth Efficiency** of **FishNet**.
4.  **Infrastructure Ease** of **Roblox** (via Agones).
