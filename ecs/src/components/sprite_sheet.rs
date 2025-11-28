use serde::{Deserialize, Serialize};

/// Represents a single frame within a sprite sheet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteFrame {
    /// X coordinate in the sprite sheet (pixels)
    pub x: u32,
    /// Y coordinate in the sprite sheet (pixels)
    pub y: u32,
    /// Width of the frame (pixels)
    pub width: u32,
    /// Height of the frame (pixels)
    pub height: u32,
    /// Optional frame name/identifier
    pub name: Option<String>,
}

/// Sprite sheet component for managing sprite atlas data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteSheet {
    /// Path to the sprite sheet texture
    pub texture_path: String,
    /// Texture ID for rendering
    pub texture_id: String,
    /// Width of the entire sprite sheet (pixels)
    pub sheet_width: u32,
    /// Height of the entire sprite sheet (pixels)
    pub sheet_height: u32,
    /// Individual frames within the sprite sheet
    pub frames: Vec<SpriteFrame>,
}

impl SpriteSheet {
    /// Create a new sprite sheet
    pub fn new(texture_path: impl Into<String>, texture_id: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            texture_path: texture_path.into(),
            texture_id: texture_id.into(),
            sheet_width: width,
            sheet_height: height,
            frames: Vec::new(),
        }
    }

    /// Add a frame to the sprite sheet
    pub fn add_frame(&mut self, frame: SpriteFrame) {
        self.frames.push(frame);
    }

    /// Create a grid-based sprite sheet (equal-sized frames)
    pub fn from_grid(
        texture_path: impl Into<String>,
        texture_id: impl Into<String>,
        sheet_width: u32,
        sheet_height: u32,
        frame_width: u32,
        frame_height: u32,
        spacing: u32,
        margin: u32,
    ) -> Self {
        let mut sheet = Self::new(texture_path, texture_id, sheet_width, sheet_height);
        
        let cols = (sheet_width - 2 * margin + spacing) / (frame_width + spacing);
        let rows = (sheet_height - 2 * margin + spacing) / (frame_height + spacing);
        
        for row in 0..rows {
            for col in 0..cols {
                let x = margin + col * (frame_width + spacing);
                let y = margin + row * (frame_height + spacing);
                
                sheet.add_frame(SpriteFrame {
                    x,
                    y,
                    width: frame_width,
                    height: frame_height,
                    name: Some(format!("frame_{}_{}", row, col)),
                });
            }
        }
        
        sheet
    }

    /// Get a frame by index
    pub fn get_frame(&self, index: usize) -> Option<&SpriteFrame> {
        self.frames.get(index)
    }

    /// Get a frame by name
    pub fn get_frame_by_name(&self, name: &str) -> Option<&SpriteFrame> {
        self.frames.iter().find(|f| {
            f.name.as_ref().map(|n| n == name).unwrap_or(false)
        })
    }
}

/// Animation loop mode
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnimationMode {
    /// Play once and stop
    Once,
    /// Loop continuously
    Loop,
    /// Ping-pong (forward then backward)
    PingPong,
}

/// Animated sprite component for frame-based animations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimatedSprite {
    /// Reference to the sprite sheet entity or texture ID
    pub sprite_sheet_id: String,
    /// Current frame index
    pub current_frame: usize,
    /// Frame indices to play (if empty, plays all frames)
    pub frame_sequence: Vec<usize>,
    /// Time per frame (seconds)
    pub frame_duration: f32,
    /// Accumulated time for current frame
    #[serde(skip)]
    pub elapsed_time: f32,
    /// Animation mode
    pub mode: AnimationMode,
    /// Is the animation playing?
    pub playing: bool,
    /// Animation direction (1 = forward, -1 = backward for ping-pong)
    #[serde(skip)]
    pub direction: i32,
}

impl Default for AnimatedSprite {
    fn default() -> Self {
        Self {
            sprite_sheet_id: String::new(),
            current_frame: 0,
            frame_sequence: Vec::new(),
            frame_duration: 0.1, // 10 FPS by default
            elapsed_time: 0.0,
            mode: AnimationMode::Loop,
            playing: true,
            direction: 1,
        }
    }
}

impl AnimatedSprite {
    /// Create a new animated sprite
    pub fn new(sprite_sheet_id: impl Into<String>, frame_duration: f32) -> Self {
        Self {
            sprite_sheet_id: sprite_sheet_id.into(),
            frame_duration,
            ..Default::default()
        }
    }

    /// Update the animation
    pub fn update(&mut self, delta_time: f32, total_frames: usize) {
        if !self.playing || total_frames == 0 {
            return;
        }

        self.elapsed_time += delta_time;

        if self.elapsed_time >= self.frame_duration {
            self.elapsed_time -= self.frame_duration;
            
            let frame_count = if self.frame_sequence.is_empty() {
                total_frames
            } else {
                self.frame_sequence.len()
            };

            match self.mode {
                AnimationMode::Once => {
                    if self.current_frame < frame_count - 1 {
                        self.current_frame += 1;
                    } else {
                        self.playing = false;
                    }
                }
                AnimationMode::Loop => {
                    self.current_frame = (self.current_frame + 1) % frame_count;
                }
                AnimationMode::PingPong => {
                    let next_frame = self.current_frame as i32 + self.direction;
                    
                    if next_frame >= frame_count as i32 {
                        self.direction = -1;
                        self.current_frame = frame_count.saturating_sub(2);
                    } else if next_frame < 0 {
                        self.direction = 1;
                        self.current_frame = 1.min(frame_count - 1);
                    } else {
                        self.current_frame = next_frame as usize;
                    }
                }
            }
        }
    }

    /// Get the actual frame index to render
    pub fn get_frame_index(&self) -> usize {
        if self.frame_sequence.is_empty() {
            self.current_frame
        } else {
            self.frame_sequence.get(self.current_frame).copied().unwrap_or(0)
        }
    }

    /// Play the animation
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_frame = 0;
        self.elapsed_time = 0.0;
        self.direction = 1;
    }

    /// Set frame sequence
    pub fn set_sequence(&mut self, sequence: Vec<usize>) {
        self.frame_sequence = sequence;
        self.current_frame = 0;
    }
}
