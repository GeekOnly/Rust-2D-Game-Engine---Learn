# AAA Audio System Design ("Symphony")

## Overview
This document outlines the design for a **High-Fidelity Audio System** capable of delivering immersive 3D spatial audio and dynamic soundtracks.
The goal is to move beyond "playing sounds" to "wait-free, graph-based audio processing" similar to Wwise or FMOD, but implemented natively in Rust.

---

## 1. Core Architecture: The Audio Graph

Audio must run on a separate, high-priority thread to prevent "pops" or "crackles" when the game FPS drops.

### 1.1 The Graph Model
Instead of "fire and forget", we build a signal processing graph:
*   **Sources:** Wav/Ogg files, Oscillators.
*   **Nodes:** Filters (LowPass), Effects (Reverb), Faders (Volume).
*   **Buses:** Logical grouping (Master -> SFX -> Weapons).

### 1.2 The Technology Stack
*   **Recommendation:** Use **`kira`** (Rust Crate) as the low-level backend. It handles the DSP graph and timing natively.
*   **Abstraction:** We wrap `kira` in an ECS-friendly `AudioSystem`.

---

## 2. Key Features

### 2.1 3D Spatial Audio (HRTF)
For "AAA" immersion, sounds must have location.
*   **Attenuation:** Volume drops with distance (Logarithmic/Linear curves).
*   **Panning:** Stereo panning based on Angle to Listener.
*   **Doppler Effect:** Pitch shift based on relative velocity (e.g., a car passing by).
*   **Occlusion (Advanced):** Raycast from Listener to Source. If hit Wall -> Apply LowPass Filter (Muffled sound).

### 2.2 Dynamic Mixing (Ducking)
*   **Buses:**
    *   `Master`
    *   `Music`
    *   `SFX` (Explosions, Footsteps)
    *   `Voice` (Dialogue)
    *   `UI`
*   **Sidechain Compression (Ducking):**
    *   *Rule:* When `Voice` volume > 0.1, reduce `Music` volume by 50% automatically.
    *   *Result:* Dialogue is always clear without manual scripting.

### 2.3 Interactive Music System (IMS)
Linked to the **AI Game Director**.
*   **Technique: Vertical Layering (Stems)**
    *   Music Track has 4 layers: *Drum / Bass / Melody / Choir*.
    *   **Calm State:** Play only *Melody*.
    *   **Combat State:** Fade in *Drum + Bass*.
    *   **Boss State:** Fade in *Choir*.
*   **Technique: Horizontal Resequencing**
    *   Transition tracks (Bridge) that play when switching from "Explore" to "Combat" to ensure musical timing (on beat).

### 2.4 Environmental Audio (Reverb Zones)
*   **Components:** `ReverbZone` (Box/Sphere).
*   **Logic:** When Player enters "Cave Zone", set the Global Reverb Mix to 100% "Cave Preset".
*   **DSP:** Uses a "Convolver" or "Freeverb" algorithm to simulate accurate room acoustics.

---

## 3. Advanced Topic: Ray Traced Audio (Propagation)

**Question:** "Should we use Ray Traced Audio (like Steam Audio)?"
**Answer for Mobile:** **NO** (for Real-time), but **YES** (for Baked).

### 3.1 The Problem
Real-time Audio Ray Tracing requires shooting 1000s of rays *per sound source* every frame to calculate reflections (Echoes) and diffraction (Sound going around corners).
*   **Desktop:** Possible on High-end CPU (Cyberpunk 2077 uses spatial audio heavily).
*   **Mobile:** Burns CPU battery instantly. Not viable for real-time.

### 3.2 The Mobile Solution: Baked Acoustic Probes
We emulate the quality of Ray Tracing without the runtime cost. Strategy similar to **Reference: Microsoft Project Acoustics**.

1.  **Editor Time (Baking):**
    *   We place "Acoustic Probes" (voxels) every 2-5 meters in the scene.
    *   The Engine fires thousands of rays to calculate:
        *   *Wetness:* How much reverb is here?
        *   *Portals:* Where can sound enter this room from? (Windows/Doors).
    *   Save this data into a lightweight 3D Texture or Grid.

2.  **Runtime (Mobile):**
    *   **Lookup:** When a sound plays at Position A and listener is at Position B.
    *   **Query:** `AcousticData.Sample(PosA, PosB)`.
    *   **Result:** The system returns pre-calculated Reverb & Filter values instantly.

---

## 4. Physics-Driven Audio (Integrations)

**Goal:** Objects should "sound" right when they collide, slide, or roll, based on what they are made of.

### 4.1 Physical Materials Logic
We don't hardcode "MetalHitSound". We use a lookup table (Matrix).
*   **Materials:** `Wood`, `Metal`, `Stone`, `Flesh`, `Glass`.
*   **Interaction Matrix:**
    *   `Wood` x `Stone` -> Play `WoodHitStone.wav`
    *   `Metal` x `Metal` -> Play `MetalClank.wav`
    *   `Flesh` x `Metal` -> Play `SwordHitBody.wav`

### 4.2 Impact Sounds (Impulse)
Listen to Rapier Physics `ContactEvents`.
*   **Threshold:** Only play if `RelativeVelocity > 1.0 m/s`. (Prevents "micro-jitter" noise).
*   **Volume:** Map `RelativeVelocity` (Force) to `Volume` (0.0 to 1.0).
*   **Optimization:** Limit max 3 impact sounds per frame per object (to avoid spam).

### 4.3 Continuous Audio (Sliding/Rolling)
For objects dragging along the ground.
*   **Logic:**
    *   If `IsInContact` AND `TangentialVelocity > 0.1` -> Start Looping Sound.
    *   **Pitch/Volume Modulation:**
        *   Faster Slide = Higher Pitch + Louder Volume.
        *   Heavier Object = Lower Pitch.
*   **Example:** A stone crate pushed across a concrete floor.

---

## 5. ECS Integration

How the user interacts with it in code.

### 5.1 Components
```rust
#[derive(Component)]
struct AudioListener; // Put this on the Camera

#[derive(Component)]
struct AudioSource {
    pub clip: Handle<AudioClip>,
    pub config: PlaybackConfig (Loop, Volume, Pitch),
    pub spatial: SpatialConfig (MinDist, MaxDist),
    pub bus: String, // "SFX", "Voice"
}

#[derive(Component)]
struct PhysicalAudio {
    pub material: AudioMaterialType, // Wood, Metal...
    pub slide_loop: Option<Handle<AudioClip>>,
    pub roll_loop: Option<Handle<AudioClip>>,
}
```

### 5.2 The System Loop
1.  **`audio_physics_system`**:
    *   Read `CollisionEvent` queue from Physics Engine.
    *   Lookup `PhysicalAudio` components of Entity A and Entity B.
    *   Calculate Impact Volume.
    *   Spawn a "One-Shot" Audio Source at the Contact Point.
2.  **`audio_slide_system`**:
    *   Query moving rigidbodies in contact.
    *   Update parameters of the Looping Audio Source.

---

## 6. Comparison & Performance

| Feature | Simple (Rodio/Bevy Default) | **AAA Design (Proposed)** |
| :--- | :--- | :--- |
| **Mixing** | Hard global volume | Bus Graph + Sidechaining |
| **Spatial** | Simple Panning | 3D + Occlusion + Doppler |
| **Physics** | Manual scripting | **Auto-Matrix (Material x Material)** |
| **Propagation** | None | **Baked Acoustic Probes** |
| **Perf** | Can stutter on main thread | Dedicated DSP Thread |

---

## 7. Online Voice Chat Integration (VoIP)

For an "Online Subsystem", Voice Chat is critical.

### 7.1 Architecture
*   **Do NOT** process raw VoIP packets through the main Audio Graph (Latency risk).
*   **Use External SDK:** Integrate dedicated wrappers like **Vivox** or **Agora** SDKs.
*   **Spatial Injection:**
    *   Valid VoIP packets from "Teammate A" are decoded to a PCM stream.
    *   This PCM stream is fed into a **Custom AudioSource** in our engine.
    *   **Result:** We can apply **HRTF/Occlusion** to the Voice Chat. (You hear Teammate A coming from the *Left room*).

### 7.2 Push-to-Talk Logic
*   Handled by `InputSystem`, not `AudioSystem`.
*   Input -> Toggle Network Stream.

This completes the ecosystem, linking Online Logic to Audio Presentation.
