//! World-Space UI Components
//! 
//! UI elements that exist in world space and follow entities
//! (e.g., health bars above enemies, damage numbers, interaction prompts)

use serde::{Serialize, Deserialize};

/// World-space UI component that renders UI elements in 3D/2D world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldUI {
    pub ui_type: WorldUIType,
    /// Offset from entity position (in world units)
    pub offset: [f32; 2],
    /// Always face camera (billboard)
    pub billboard: bool,
    /// Scale factor (1.0 = normal size)
    pub scale: f32,
}

impl Default for WorldUI {
    fn default() -> Self {
        Self {
            ui_type: WorldUIType::HealthBar { current: 100.0, max: 100.0 },
            offset: [0.0, 50.0],
            billboard: true,
            scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorldUIType {
    /// Health bar above entity
    HealthBar {
        current: f32,
        max: f32,
    },
    /// Floating damage number
    DamageNumber {
        value: i32,
        lifetime: f32,
        /// Velocity for floating animation
        velocity: [f32; 2],
    },
    /// Interaction prompt (e.g., "Press E to interact")
    InteractionPrompt {
        text: String,
        key: String,
    },
    /// Quest marker
    QuestMarker {
        marker_type: QuestMarkerType,
    },
    /// Custom text label
    TextLabel {
        text: String,
        color: [f32; 4],
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum QuestMarkerType {
    Objective,
    TurnIn,
    Available,
}

impl WorldUI {
    /// Create a health bar
    pub fn health_bar(current: f32, max: f32) -> Self {
        Self {
            ui_type: WorldUIType::HealthBar { current, max },
            offset: [0.0, 50.0],
            billboard: true,
            scale: 1.0,
        }
    }
    
    /// Create a damage number
    pub fn damage_number(value: i32) -> Self {
        Self {
            ui_type: WorldUIType::DamageNumber {
                value,
                lifetime: 1.0,
                velocity: [0.0, 50.0], // Float upward
            },
            offset: [0.0, 0.0],
            billboard: true,
            scale: 1.0,
        }
    }
    
    /// Create an interaction prompt
    pub fn interaction_prompt(text: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            ui_type: WorldUIType::InteractionPrompt {
                text: text.into(),
                key: key.into(),
            },
            offset: [0.0, 30.0],
            billboard: true,
            scale: 1.0,
        }
    }
    
    /// Create a quest marker
    pub fn quest_marker(marker_type: QuestMarkerType) -> Self {
        Self {
            ui_type: WorldUIType::QuestMarker { marker_type },
            offset: [0.0, 80.0],
            billboard: true,
            scale: 1.0,
        }
    }
    
    /// Update health bar values
    pub fn update_health(&mut self, current: f32, max: f32) {
        if let WorldUIType::HealthBar { current: c, max: m } = &mut self.ui_type {
            *c = current;
            *m = max;
        }
    }
    
    /// Update damage number lifetime
    pub fn update_damage_number(&mut self, dt: f32) -> bool {
        if let WorldUIType::DamageNumber { lifetime, velocity, .. } = &mut self.ui_type {
            *lifetime -= dt;
            // Update offset based on velocity
            self.offset[0] += velocity[0] * dt;
            self.offset[1] += velocity[1] * dt;
            // Fade out
            *lifetime > 0.0
        } else {
            true
        }
    }
}
