# AI-Ready Profiler System Upgrade Plan

## Objective
Upgrade the existing `ProfilerSystem` to not just *display* performance data, but to *capture, analyze, and export* it in a structured format (JSON) that an AI Agent can consume to track issues, detect regressions, and provide specific optimization advice.

## Core Features

### 1. Structured Telemetry (AI Readable)
Instead of just internal `VecDeque`, we will implement a `TelemetryFrame` struct that can be serialized to JSON.

```rust
#[derive(Serialize)]
pub struct TelemetryReport {
    pub session_id: String,
    pub timestamp: String,
    pub device_info: DeviceInfo,
    pub frames: Vec<TelemetryFrame>, // Only capture frames of interest (e.g. spikes) or sampled
    pub summary: SessionSummary,
}

#[derive(Serialize)]
pub struct TelemetryFrame {
    pub frame_number: u64,
    pub duration_ms: f32,
    pub scopes: Vec<ScopeData>, // Flattened hierarchy
    pub batch_stats: BatchPerformanceStats,
    pub memory_delta: i64,
    pub associated_events: Vec<String>, // e.g. "AssetLoaded: Player.png"
}
```

### 2. Automatic Anomaly Detection
The profiler will have configurable "Triggers". When a trigger is met, it will:
1.  **Mark** the frame as "Anomalous".
2.  **Dump** the full frame data (Scope hierarchy, Batch stats).
3.  **Emit** a warning log for the AI.

**Triggers:**
*   `FrameSpike`: Frame time > 33.3ms (30 FPS) or custom threshold.
*   `MemorySpike`: Allocation delta > 10 MB/frame.
*   `DrawCallSpike`: Draw calls > 1000/frame.

### 3. "Black Box" Context Recording
AI needs context. "Why did FPS drop?"
The profiler will accept "Markers" or "Tags" from other systems.
*   `profiler.tag("LoadingLevel", "Level1")`
*   `profiler.tag("SpawnEnemy", "Boss")`
These tags are attached to the `TelemetryFrame`. If a spike happens, we see: "Spike at Frame 500 (Duration: 100ms) | Tags: [LoadingLevel: Level1]".

### 4. Integration
*   **Editor:** Add "Export Profiler Report" button.
*   **Runtime:** Auto-save `profile_capture_{timestamp}.json` on exit or crash (optional).
*   **AI Access:** The AI Agent can read `profile_capture_*.json` to answer questions like "Why does it lag when I spawn the boss?".

---

## Implementation Steps

1.  **Dependencies:** Add `serde`, `serde_json`, `chrono` to `engine/Cargo.toml` (already present in workspace).
2.  **Telemetry Structs:** Define `TelemetryReport`, `TelemetryFrame` in `profiler.rs`.
3.  **Capture Logic:** Modify `ProfilerSystem` to hold a buffer of `TelemetryFrame`.
    *   Ring buffer for last 600 frames (10 seconds).
    *   "Snapshot" features to save the buffer to disk.
4.  **Anomaly Logic:** Check thresholds in `end_frame()`. If trigger -> Save Snapshot.
5.  **Tag API:** Add `add_frame_tag(key, value)`.

This system turns the Profiler from a passive UI into an active Debugging Assistant.
