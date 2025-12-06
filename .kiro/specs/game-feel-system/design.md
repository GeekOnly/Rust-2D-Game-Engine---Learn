# Game Feel System Design Document

## Overview

The Game Feel System is a comprehensive module designed to enhance player experience through synchronized visual, audio, and haptic feedback. The system provides a unified API for triggering effects like screen shake, hit stop, particle effects, camera manipulation, audio playback, and haptic feedback across 2D, 2.5D, and 3D game environments.

### Key Design Goals

1. **Ease of Use**: Simple API for triggering complex multi-modal feedback with minimal code
2. **Performance**: Efficient resource management with object pooling and performance budgeting
3. **Flexibility**: Support for custom easing functions, animation curves, and effect sequencing
4. **Cross-Dimensional**: Consistent behavior across 2D, 2.5D, and 3D rendering modes
5. **Composability**: Ability to combine multiple effects into reusable presets
6. **Editor Integration**: Real-time preview and testing capabilities

## Architecture

The Game Feel System follows a component-based architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                  GameFeelManager                        │
│  (Central coordinator, effect scheduling, time control) │
└────────────┬────────────────────────────────────────────┘
             │
    ┌────────┴────────┬──────────┬──────────┬──────────┐
    │                 │          │          │          │
┌───▼────┐  ┌────────▼───┐  ┌───▼────┐  ┌──▼─────┐  ┌▼────────┐
│ Camera │  │   Tween    │  │Particle│  │ Audio  │  │ Haptic  │
│ Effects│  │   System   │  │ System │  │ System │  │ System  │
└────────┘  └────────────┘  └────────┘  └────────┘  └─────────┘
```

### Core Components

1. **GameFeelManager**: Central hub that coordinates all game feel effects, manages time scale, and handles effect scheduling
2. **CameraEffectSystem**: Handles screen shake, zoom, rotation, and camera offset effects
3. **TweenSystem**: Provides interpolation for any value type with customizable easing functions
4. **ParticleSystem**: Manages particle spawning, lifecycle, and rendering
5. **AudioSystem**: Controls sound playback, audio mixer parameters, and playlists
6. **HapticSystem**: Manages haptic feedback for supported platforms
7. **EffectSequencer**: Orchestrates complex multi-effect sequences with timing control
8. **PerformanceMonitor**: Tracks performance metrics and enforces budgets

## Components and Interfaces

### GameFeelManager

The central manager that coordinates all game feel effects.

```rust
pub struct GameFeelManager {
    camera_effects: CameraEffectSystem,
    tween_system: TweenSystem,
    particle_system: ParticleSystem,
    audio_system: AudioSystem,
    haptic_system: HapticSystem,
    sequencer: EffectSequencer,
    performance_monitor: PerformanceMonitor,
    
    time_scale: f32,
    target_time_scale: f32,
    time_scale_transition: Option<TweenHandle>,
    
    dimension_mode: DimensionMode,
    effect_presets: HashMap<String, EffectPreset>,
}

impl GameFeelManager {
    pub fn new() -> Self;
    pub fn update(&mut self, delta_time: f32);
    
    // Time control
    pub fn set_time_scale(&mut self, scale: f32, transition_duration: Option<f32>);
    pub fn get_scaled_delta_time(&self, delta_time: f32) -> f32;
    
    // Effect triggering
    pub fn trigger_screen_shake(&mut self, intensity: f32, duration: f32, direction: Option<Vec3>);
    pub fn trigger_hit_stop(&mut self, duration: f32, selective: Option<Vec<EntityId>>);
    pub fn trigger_impact(&mut self, preset: &str, position: Vec3, intensity: f32, direction: Vec3);
    
    // Preset management
    pub fn register_preset(&mut self, name: String, preset: EffectPreset);
    pub fn trigger_preset(&mut self, name: &str, params: EffectParams);
    
    // Dimension mode
    pub fn set_dimension_mode(&mut self, mode: DimensionMode);
}
```

### CameraEffectSystem

Manages all camera-related effects including shake, zoom, rotation, and offset.

```rust
pub struct CameraEffectSystem {
    trauma: f32,
    trauma_decay_rate: f32,
    max_trauma: f32,
    
    shake_frequency: f32,
    shake_amplitude: f32,
    shake_rotation_amplitude: f32,
    
    zoom_tweens: Vec<TweenHandle>,
    rotation_tweens: Vec<TweenHandle>,
    offset_tweens: Vec<TweenHandle>,
    
    base_position: Vec3,
    base_rotation: Quat,
    base_fov: f32,
    
    dimension_mode: DimensionMode,
}

impl CameraEffectSystem {
    pub fn add_trauma(&mut self, amount: f32);
    pub fn update(&mut self, delta_time: f32, camera: &mut Camera);
    
    pub fn trigger_shake(&mut self, intensity: f32, duration: f32, direction: Option<Vec3>);
    pub fn trigger_zoom(&mut self, target_fov: f32, duration: f32, easing: EasingFunction);
    pub fn trigger_rotation(&mut self, axis: Vec3, angle: f32, duration: f32, easing: EasingFunction);
    pub fn trigger_offset(&mut self, offset: Vec3, duration: f32, easing: EasingFunction);
    
    fn calculate_shake_offset(&self, time: f32) -> Vec3;
    fn calculate_shake_rotation(&self, time: f32) -> Quat;
}
```

### TweenSystem

Provides flexible interpolation for any value type with support for easing functions and chaining.

```rust
pub struct TweenSystem {
    active_tweens: Vec<Tween>,
    next_handle: TweenHandle,
}

pub struct Tween {
    handle: TweenHandle,
    start_value: TweenValue,
    end_value: TweenValue,
    current_time: f32,
    duration: f32,
    easing: EasingFunction,
    on_update: Box<dyn FnMut(TweenValue)>,
    on_complete: Option<Box<dyn FnOnce()>>,
    chain: Option<TweenHandle>,
    parallel: Vec<TweenHandle>,
}

pub enum TweenValue {
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Color(Color),
    Quat(Quat),
}

pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Elastic,
    Bounce,
    Back,
    Custom(Box<dyn Fn(f32) -> f32>),
    Curve(AnimationCurve),
}

impl TweenSystem {
    pub fn create_tween(&mut self, start: TweenValue, end: TweenValue, duration: f32) -> TweenHandle;
    pub fn update(&mut self, delta_time: f32);
    pub fn cancel_tween(&mut self, handle: TweenHandle, snap_to_end: bool);
    pub fn chain_tween(&mut self, first: TweenHandle, second: TweenHandle);
    pub fn parallel_tween(&mut self, parent: TweenHandle, child: TweenHandle);
}
```

### ParticleSystem

Manages particle effects with efficient pooling and lifecycle management.

```rust
pub struct ParticleSystem {
    particles: Vec<Particle>,
    emitters: HashMap<EmitterId, ParticleEmitter>,
    particle_pool: Vec<Particle>,
    max_particles: usize,
    dimension_mode: DimensionMode,
}

pub struct Particle {
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    color: Color,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    active: bool,
}

pub struct ParticleEmitter {
    id: EmitterId,
    position: Vec3,
    attached_entity: Option<EntityId>,
    emission_rate: f32,
    particle_config: ParticleConfig,
    active: bool,
}

pub struct ParticleConfig {
    initial_velocity: Vec3,
    velocity_randomness: Vec3,
    lifetime: f32,
    lifetime_randomness: f32,
    color_gradient: Vec<(f32, Color)>,
    size_over_lifetime: AnimationCurve,
    gravity: Vec3,
}

impl ParticleSystem {
    pub fn spawn_particles(&mut self, position: Vec3, count: usize, config: ParticleConfig);
    pub fn create_emitter(&mut self, config: ParticleEmitter) -> EmitterId;
    pub fn update(&mut self, delta_time: f32);
    pub fn render(&self, renderer: &mut Renderer);
    
    fn recycle_particle(&mut self, index: usize);
    fn prioritize_particles(&mut self, camera_position: Vec3);
}
```

### AudioSystem

Handles sound playback, audio mixer control, and playlist management.

```rust
pub struct AudioSystem {
    audio_sources: Vec<AudioSource>,
    audio_pool: Vec<AudioSource>,
    mixer: AudioMixer,
    playlists: HashMap<String, Playlist>,
    active_sounds: HashMap<SoundId, ActiveSound>,
    next_sound_id: SoundId,
}

pub struct AudioSource {
    clip: Option<AudioClip>,
    volume: f32,
    pitch: f32,
    position: Option<Vec3>,
    spatial: bool,
    playing: bool,
}

pub struct AudioMixer {
    groups: HashMap<String, MixerGroup>,
    snapshots: HashMap<String, MixerSnapshot>,
    current_snapshot: String,
}

pub struct MixerGroup {
    volume: f32,
    filters: Vec<AudioFilter>,
}

pub struct Playlist {
    tracks: Vec<AudioClip>,
    current_index: usize,
    mode: PlaylistMode,
    crossfade_duration: f32,
}

pub enum PlaylistMode {
    Sequential,
    Loop,
    Shuffle,
}

impl AudioSystem {
    pub fn play_sound(&mut self, clip: AudioClip, volume: f32, pitch: f32, position: Option<Vec3>) -> SoundId;
    pub fn play_sound_randomized(&mut self, clip: AudioClip, volume_range: (f32, f32), pitch_range: (f32, f32)) -> SoundId;
    pub fn play_layered_sound(&mut self, layers: Vec<SoundLayer>) -> Vec<SoundId>;
    
    pub fn tween_mixer_parameter(&mut self, group: &str, parameter: &str, target: f32, duration: f32);
    pub fn transition_to_snapshot(&mut self, snapshot: &str, duration: f32);
    
    pub fn create_playlist(&mut self, name: String, tracks: Vec<AudioClip>, mode: PlaylistMode);
    pub fn play_playlist(&mut self, name: &str);
    pub fn skip_track(&mut self, name: &str);
    
    pub fn update(&mut self, delta_time: f32);
}
```

### HapticSystem

Manages haptic feedback for supported platforms.

```rust
pub struct HapticSystem {
    platform_support: bool,
    active_haptics: Vec<ActiveHaptic>,
}

pub struct ActiveHaptic {
    pattern: HapticPattern,
    intensity: f32,
    elapsed_time: f32,
    duration: f32,
}

pub enum HapticPattern {
    Impulse { intensity: f32 },
    Continuous { amplitude: f32, frequency: f32 },
    Pattern { data: Vec<(f32, f32)> }, // (time, intensity) pairs
    Preset(HapticPreset),
}

pub enum HapticPreset {
    LightImpact,
    MediumImpact,
    HeavyImpact,
    Selection,
    Warning,
    Success,
    Failure,
}

impl HapticSystem {
    pub fn trigger_haptic(&mut self, pattern: HapticPattern, intensity: f32);
    pub fn trigger_preset(&mut self, preset: HapticPreset);
    pub fn update(&mut self, delta_time: f32);
    pub fn is_supported(&self) -> bool;
}
```

### EffectSequencer

Orchestrates complex sequences of effects with precise timing control.

```rust
pub struct EffectSequencer {
    sequences: HashMap<String, EffectSequence>,
    active_sequences: Vec<ActiveSequence>,
}

pub struct EffectSequence {
    name: String,
    steps: Vec<SequenceStep>,
    loop_mode: bool,
}

pub struct SequenceStep {
    delay: f32,
    effect: EffectCommand,
}

pub enum EffectCommand {
    ScreenShake { intensity: f32, duration: f32 },
    HitStop { duration: f32 },
    PlaySound { clip: AudioClip, volume: f32, pitch: f32 },
    SpawnParticles { position: Vec3, config: ParticleConfig },
    TriggerHaptic { pattern: HapticPattern },
    CameraZoom { target_fov: f32, duration: f32 },
    Custom(Box<dyn FnOnce(&mut GameFeelManager)>),
}

pub struct ActiveSequence {
    sequence: EffectSequence,
    current_step: usize,
    elapsed_time: f32,
}

impl EffectSequencer {
    pub fn register_sequence(&mut self, sequence: EffectSequence);
    pub fn trigger_sequence(&mut self, name: &str, manager: &mut GameFeelManager);
    pub fn update(&mut self, delta_time: f32, manager: &mut GameFeelManager);
    pub fn interrupt_sequence(&mut self, name: &str);
}
```

### PerformanceMonitor

Tracks performance metrics and enforces budgets to maintain frame rate.

```rust
pub struct PerformanceMonitor {
    frame_time_budget: f32,
    current_frame_time: f32,
    
    effect_timings: HashMap<String, f32>,
    particle_count: usize,
    active_tweens: usize,
    active_sounds: usize,
    
    profiling_enabled: bool,
}

impl PerformanceMonitor {
    pub fn begin_frame(&mut self);
    pub fn end_frame(&mut self);
    pub fn record_effect_time(&mut self, effect_name: &str, duration: f32);
    
    pub fn is_budget_exceeded(&self) -> bool;
    pub fn should_skip_effect(&self, priority: EffectPriority) -> bool;
    
    pub fn get_report(&self) -> PerformanceReport;
}

pub struct PerformanceReport {
    pub total_frame_time: f32,
    pub effect_breakdown: HashMap<String, f32>,
    pub particle_count: usize,
    pub active_tweens: usize,
    pub active_sounds: usize,
}
```

## Data Models

### Core Data Structures

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TweenHandle(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmitterId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

#[derive(Debug, Clone, Copy)]
pub enum DimensionMode {
    TwoD,
    TwoPointFiveD,
    ThreeD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}
```

### Effect Presets

```rust
pub struct EffectPreset {
    pub name: String,
    pub screen_shake: Option<ScreenShakeConfig>,
    pub hit_stop: Option<HitStopConfig>,
    pub particles: Vec<ParticleConfig>,
    pub sounds: Vec<SoundConfig>,
    pub haptics: Option<HapticPattern>,
    pub camera_effects: Vec<CameraEffectConfig>,
    pub priority: EffectPriority,
}

pub struct ScreenShakeConfig {
    pub intensity: f32,
    pub duration: f32,
    pub directional: bool,
}

pub struct HitStopConfig {
    pub duration: f32,
    pub selective_entities: Vec<EntityId>,
}

pub struct SoundConfig {
    pub clip: AudioClip,
    pub volume: (f32, f32), // min, max for randomization
    pub pitch: (f32, f32),  // min, max for randomization
    pub spatial: bool,
    pub delay: f32,
}

pub struct CameraEffectConfig {
    pub effect_type: CameraEffectType,
    pub duration: f32,
    pub easing: EasingFunction,
}

pub enum CameraEffectType {
    Zoom { target_fov: f32 },
    Rotation { axis: Vec3, angle: f32 },
    Offset { offset: Vec3 },
}
```

### Animation Curves

```rust
pub struct AnimationCurve {
    keyframes: Vec<Keyframe>,
    interpolation_mode: InterpolationMode,
    wrap_mode: WrapMode,
}

pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
}

pub enum InterpolationMode {
    Linear,
    Smooth,
    Constant,
    Cubic,
}

pub enum WrapMode {
    Clamp,
    Loop,
    PingPong,
}

impl AnimationCurve {
    pub fn new() -> Self;
    pub fn add_keyframe(&mut self, keyframe: Keyframe);
    pub fn evaluate(&self, time: f32) -> f32;
    pub fn from_preset(preset: CurvePreset) -> Self;
}

pub enum CurvePreset {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}
```

### Squash and Stretch

```rust
pub struct SquashAndStretchEffect {
    pub entity: EntityId,
    pub original_scale: Vec3,
    pub squash_axis: Vec3,
    pub intensity: f32,
    pub duration: f32,
    pub preserve_volume: bool,
    pub elapsed_time: f32,
}

impl SquashAndStretchEffect {
    pub fn calculate_scale(&self, t: f32) -> Vec3;
    pub fn update(&mut self, delta_time: f32) -> bool; // returns true if complete
}
```

### Recoil System

```rust
pub struct RecoilEffect {
    pub entity: EntityId,
    pub spring: SpringDamper,
    pub pattern: Option<RecoilPattern>,
    pub camera_recoil: bool,
}

pub struct SpringDamper {
    pub position: Vec3,
    pub velocity: Vec3,
    pub target: Vec3,
    pub stiffness: f32,
    pub damping: f32,
}

pub struct RecoilPattern {
    pub offsets: Vec<Vec3>,
    pub current_index: usize,
}

impl SpringDamper {
    pub fn apply_impulse(&mut self, force: Vec3);
    pub fn update(&mut self, delta_time: f32);
}
```

### Sound Layers

```rust
pub struct SoundLayer {
    pub clip: AudioClip,
    pub volume: f32,
    pub pitch: f32,
    pub delay: f32,
    pub priority: EffectPriority,
    pub variations: Vec<AudioClip>, // for random selection
}

impl SoundLayer {
    pub fn select_clip(&self) -> AudioClip;
}
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*
