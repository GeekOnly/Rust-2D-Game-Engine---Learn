// Input System - Unified input handling for keyboard, mouse, gamepad, and touch
// Supports multiple input devices with priority system

use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// KEYBOARD INPUT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // Arrow Keys
    Up, Down, Left, Right,

    // Function Keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Special Keys
    Space, Enter, Escape, Tab, Backspace, Delete,
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt,

    // Other
    Minus, Equals, LeftBracket, RightBracket,
    Semicolon, Quote, Comma, Period, Slash, Backslash,
}

impl Key {
    /// Convert from winit key code name
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "KeyA" | "A" => Some(Key::A),
            "KeyB" | "B" => Some(Key::B),
            "KeyC" | "C" => Some(Key::C),
            "KeyD" | "D" => Some(Key::D),
            "KeyE" | "E" => Some(Key::E),
            "KeyF" | "F" => Some(Key::F),
            "KeyG" | "G" => Some(Key::G),
            "KeyH" | "H" => Some(Key::H),
            "KeyI" | "I" => Some(Key::I),
            "KeyJ" | "J" => Some(Key::J),
            "KeyK" | "K" => Some(Key::K),
            "KeyL" | "L" => Some(Key::L),
            "KeyM" | "M" => Some(Key::M),
            "KeyN" | "N" => Some(Key::N),
            "KeyO" | "O" => Some(Key::O),
            "KeyP" | "P" => Some(Key::P),
            "KeyQ" | "Q" => Some(Key::Q),
            "KeyR" | "R" => Some(Key::R),
            "KeyS" | "S" => Some(Key::S),
            "KeyT" | "T" => Some(Key::T),
            "KeyU" | "U" => Some(Key::U),
            "KeyV" | "V" => Some(Key::V),
            "KeyW" | "W" => Some(Key::W),
            "KeyX" | "X" => Some(Key::X),
            "KeyY" | "Y" => Some(Key::Y),
            "KeyZ" | "Z" => Some(Key::Z),

            "Digit0" | "0" => Some(Key::Num0),
            "Digit1" | "1" => Some(Key::Num1),
            "Digit2" | "2" => Some(Key::Num2),
            "Digit3" | "3" => Some(Key::Num3),
            "Digit4" | "4" => Some(Key::Num4),
            "Digit5" | "5" => Some(Key::Num5),
            "Digit6" | "6" => Some(Key::Num6),
            "Digit7" | "7" => Some(Key::Num7),
            "Digit8" | "8" => Some(Key::Num8),
            "Digit9" | "9" => Some(Key::Num9),

            "ArrowUp" | "Up" => Some(Key::Up),
            "ArrowDown" | "Down" => Some(Key::Down),
            "ArrowLeft" | "Left" => Some(Key::Left),
            "ArrowRight" | "Right" => Some(Key::Right),

            "F1" => Some(Key::F1),
            "F2" => Some(Key::F2),
            "F3" => Some(Key::F3),
            "F4" => Some(Key::F4),
            "F5" => Some(Key::F5),
            "F6" => Some(Key::F6),
            "F7" => Some(Key::F7),
            "F8" => Some(Key::F8),
            "F9" => Some(Key::F9),
            "F10" => Some(Key::F10),
            "F11" => Some(Key::F11),
            "F12" => Some(Key::F12),

            "Space" => Some(Key::Space),
            "Enter" => Some(Key::Enter),
            "Escape" => Some(Key::Escape),
            "Tab" => Some(Key::Tab),
            "Backspace" => Some(Key::Backspace),
            "Delete" => Some(Key::Delete),

            "ShiftLeft" | "LShift" => Some(Key::LShift),
            "ShiftRight" | "RShift" => Some(Key::RShift),
            "ControlLeft" | "LCtrl" => Some(Key::LCtrl),
            "ControlRight" | "RCtrl" => Some(Key::RCtrl),
            "AltLeft" | "LAlt" => Some(Key::LAlt),
            "AltRight" | "RAlt" => Some(Key::RAlt),

            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Key::A => "A", Key::B => "B", Key::C => "C", Key::D => "D",
            Key::E => "E", Key::F => "F", Key::G => "G", Key::H => "H",
            Key::I => "I", Key::J => "J", Key::K => "K", Key::L => "L",
            Key::M => "M", Key::N => "N", Key::O => "O", Key::P => "P",
            Key::Q => "Q", Key::R => "R", Key::S => "S", Key::T => "T",
            Key::U => "U", Key::V => "V", Key::W => "W", Key::X => "X",
            Key::Y => "Y", Key::Z => "Z",
            Key::Num0 => "0", Key::Num1 => "1", Key::Num2 => "2",
            Key::Num3 => "3", Key::Num4 => "4", Key::Num5 => "5",
            Key::Num6 => "6", Key::Num7 => "7", Key::Num8 => "8", Key::Num9 => "9",
            Key::Up => "Up", Key::Down => "Down",
            Key::Left => "Left", Key::Right => "Right",
            Key::Space => "Space", Key::Enter => "Enter",
            Key::Escape => "Escape", Key::Tab => "Tab",
            Key::Backspace => "Backspace", Key::Delete => "Delete",
            Key::LShift => "LShift", Key::RShift => "RShift",
            Key::LCtrl => "LCtrl", Key::RCtrl => "RCtrl",
            Key::LAlt => "LAlt", Key::RAlt => "RAlt",
            _ => "Unknown",
        }
    }
}

// ============================================================================
// MOUSE INPUT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Default)]
pub struct MouseState {
    pub position: Vec2,           // Screen position
    pub delta: Vec2,              // Movement since last frame
    pub scroll_delta: Vec2,       // Scroll wheel delta
    pub buttons: HashSet<MouseButton>,
    pub buttons_pressed: HashSet<MouseButton>,   // Just pressed this frame
    pub buttons_released: HashSet<MouseButton>,  // Just released this frame
}

// ============================================================================
// GAMEPAD INPUT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South,      // A on Xbox, X on PlayStation
    East,       // B on Xbox, O on PlayStation
    North,      // Y on Xbox, Triangle on PlayStation
    West,       // X on Xbox, Square on PlayStation
    L1, R1,     // Shoulder buttons
    L2, R2,     // Triggers (as buttons)
    L3, R3,     // Stick clicks
    Start,
    Select,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Debug, Clone, Default)]
pub struct GamepadState {
    pub connected: bool,
    pub buttons: HashSet<GamepadButton>,
    pub buttons_pressed: HashSet<GamepadButton>,
    pub buttons_released: HashSet<GamepadButton>,
    pub axes: HashMap<GamepadAxis, f32>,
    pub left_stick: Vec2,   // -1.0 to 1.0
    pub right_stick: Vec2,  // -1.0 to 1.0
}

// ============================================================================
// TOUCH INPUT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    pub id: TouchId,
    pub position: Vec2,
    pub phase: TouchPhase,
}

#[derive(Debug, Clone, Default)]
pub struct TouchState {
    pub touches: HashMap<TouchId, Touch>,
    pub started_this_frame: Vec<Touch>,
    pub ended_this_frame: Vec<Touch>,
}

// ============================================================================
// UNIFIED INPUT SYSTEM
// ============================================================================

#[derive(Debug, Default)]
pub struct InputSystem {
    // Keyboard
    keys: HashSet<Key>,
    keys_pressed: HashSet<Key>,   // Just pressed this frame
    keys_released: HashSet<Key>,  // Just released this frame

    // Mouse
    pub mouse: MouseState,

    // Gamepads (support up to 4 controllers)
    pub gamepads: [GamepadState; 4],

    // Touch
    pub touch: TouchState,

    // Gilrs context for gamepad support
    gilrs: Option<gilrs::Gilrs>,
}

impl InputSystem {
    pub fn new() -> Self {
        let gilrs = gilrs::Gilrs::new().ok();

        Self {
            keys: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse: MouseState::default(),
            gamepads: Default::default(),
            touch: TouchState::default(),
            gilrs,
        }
    }

    // ========================================================================
    // KEYBOARD METHODS
    // ========================================================================

    /// Check if a key is currently pressed
    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }

    /// Press a key
    pub fn press_key(&mut self, key: Key) {
        if !self.keys.contains(&key) {
            self.keys_pressed.insert(key);
        }
        self.keys.insert(key);
    }

    /// Release a key
    pub fn release_key(&mut self, key: Key) {
        if self.keys.contains(&key) {
            self.keys_released.insert(key);
        }
        self.keys.remove(&key);
    }

    // ========================================================================
    // MOUSE METHODS
    // ========================================================================

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse.buttons.contains(&button)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse.buttons_pressed.contains(&button)
    }

    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse.buttons_released.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse.position
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse.delta
    }

    pub fn mouse_scroll_delta(&self) -> Vec2 {
        self.mouse.scroll_delta
    }

    pub fn press_mouse_button(&mut self, button: MouseButton) {
        if !self.mouse.buttons.contains(&button) {
            self.mouse.buttons_pressed.insert(button);
        }
        self.mouse.buttons.insert(button);
    }

    pub fn release_mouse_button(&mut self, button: MouseButton) {
        if self.mouse.buttons.contains(&button) {
            self.mouse.buttons_released.insert(button);
        }
        self.mouse.buttons.remove(&button);
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        let new_pos = Vec2::new(x, y);
        self.mouse.delta = new_pos - self.mouse.position;
        self.mouse.position = new_pos;
    }

    pub fn set_mouse_scroll(&mut self, x: f32, y: f32) {
        self.mouse.scroll_delta = Vec2::new(x, y);
    }

    // ========================================================================
    // GAMEPAD METHODS
    // ========================================================================

    pub fn is_gamepad_button_down(&self, gamepad_id: usize, button: GamepadButton) -> bool {
        if gamepad_id >= 4 { return false; }
        self.gamepads[gamepad_id].buttons.contains(&button)
    }

    pub fn is_gamepad_button_pressed(&self, gamepad_id: usize, button: GamepadButton) -> bool {
        if gamepad_id >= 4 { return false; }
        self.gamepads[gamepad_id].buttons_pressed.contains(&button)
    }

    pub fn is_gamepad_button_released(&self, gamepad_id: usize, button: GamepadButton) -> bool {
        if gamepad_id >= 4 { return false; }
        self.gamepads[gamepad_id].buttons_released.contains(&button)
    }

    pub fn gamepad_axis(&self, gamepad_id: usize, axis: GamepadAxis) -> f32 {
        if gamepad_id >= 4 { return 0.0; }
        *self.gamepads[gamepad_id].axes.get(&axis).unwrap_or(&0.0)
    }

    pub fn gamepad_left_stick(&self, gamepad_id: usize) -> Vec2 {
        if gamepad_id >= 4 { return Vec2::ZERO; }
        self.gamepads[gamepad_id].left_stick
    }

    pub fn gamepad_right_stick(&self, gamepad_id: usize) -> Vec2 {
        if gamepad_id >= 4 { return Vec2::ZERO; }
        self.gamepads[gamepad_id].right_stick
    }

    pub fn is_gamepad_connected(&self, gamepad_id: usize) -> bool {
        if gamepad_id >= 4 { return false; }
        self.gamepads[gamepad_id].connected
    }

    // ========================================================================
    // TOUCH METHODS
    // ========================================================================

    pub fn touches(&self) -> impl Iterator<Item = &Touch> {
        self.touch.touches.values()
    }

    pub fn touch_count(&self) -> usize {
        self.touch.touches.len()
    }

    pub fn get_touch(&self, id: TouchId) -> Option<&Touch> {
        self.touch.touches.get(&id)
    }

    pub fn touches_started_this_frame(&self) -> &[Touch] {
        &self.touch.started_this_frame
    }

    pub fn touches_ended_this_frame(&self) -> &[Touch] {
        &self.touch.ended_this_frame
    }

    pub fn add_touch(&mut self, id: u64, x: f32, y: f32, phase: TouchPhase) {
        let touch = Touch {
            id: TouchId(id),
            position: Vec2::new(x, y),
            phase,
        };

        match phase {
            TouchPhase::Started => {
                self.touch.touches.insert(TouchId(id), touch);
                self.touch.started_this_frame.push(touch);
            }
            TouchPhase::Moved => {
                self.touch.touches.insert(TouchId(id), touch);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.touch.touches.remove(&TouchId(id));
                self.touch.ended_this_frame.push(touch);
            }
        }
    }

    // ========================================================================
    // VIRTUAL INPUT (for flexible control schemes)
    // ========================================================================

    /// Get movement vector from WASD/Arrow keys OR gamepad left stick OR touch virtual joystick
    pub fn get_movement_input(&self, gamepad_id: usize) -> Vec2 {
        let mut input = Vec2::ZERO;

        // Keyboard input (priority 1)
        if self.is_key_down(Key::W) || self.is_key_down(Key::Up) {
            input.y -= 1.0;
        }
        if self.is_key_down(Key::S) || self.is_key_down(Key::Down) {
            input.y += 1.0;
        }
        if self.is_key_down(Key::A) || self.is_key_down(Key::Left) {
            input.x -= 1.0;
        }
        if self.is_key_down(Key::D) || self.is_key_down(Key::Right) {
            input.x += 1.0;
        }

        // If no keyboard input, try gamepad
        if input == Vec2::ZERO && gamepad_id < 4 && self.gamepads[gamepad_id].connected {
            input = self.gamepads[gamepad_id].left_stick;
        }

        // Normalize diagonal movement
        if input.length_squared() > 1.0 {
            input = input.normalize();
        }

        input
    }

    /// Get action button state (Space/Enter OR gamepad South button OR touch tap)
    pub fn get_action_button(&self, gamepad_id: usize) -> bool {
        self.is_key_down(Key::Space) ||
        self.is_key_down(Key::Enter) ||
        self.is_gamepad_button_down(gamepad_id, GamepadButton::South) ||
        self.touch_count() > 0
    }

    /// Get action button just pressed
    pub fn get_action_button_pressed(&self, gamepad_id: usize) -> bool {
        self.is_key_pressed(Key::Space) ||
        self.is_key_pressed(Key::Enter) ||
        self.is_gamepad_button_pressed(gamepad_id, GamepadButton::South) ||
        !self.touch.started_this_frame.is_empty()
    }

    // ========================================================================
    // FRAME UPDATE
    // ========================================================================

    /// Call this at the beginning of each frame to clear per-frame state
    pub fn begin_frame(&mut self) {
        // Clear keyboard frame state
        self.keys_pressed.clear();
        self.keys_released.clear();

        // Clear mouse frame state
        self.mouse.buttons_pressed.clear();
        self.mouse.buttons_released.clear();
        self.mouse.delta = Vec2::ZERO;
        self.mouse.scroll_delta = Vec2::ZERO;

        // Clear gamepad frame state
        for gamepad in &mut self.gamepads {
            gamepad.buttons_pressed.clear();
            gamepad.buttons_released.clear();
        }

        // Clear touch frame state
        self.touch.started_this_frame.clear();
        self.touch.ended_this_frame.clear();
    }

    /// Update gamepad state from gilrs
    pub fn update_gamepads(&mut self) {
        if let Some(ref mut gilrs) = self.gilrs {
            while let Some(event) = gilrs.next_event() {
                let gamepad_id: usize = event.id.into();
                if gamepad_id >= 4 { continue; }

                match event.event {
                    gilrs::EventType::Connected => {
                        self.gamepads[gamepad_id].connected = true;
                    }
                    gilrs::EventType::Disconnected => {
                        self.gamepads[gamepad_id].connected = false;
                    }
                    gilrs::EventType::ButtonPressed(button, _) => {
                        if let Some(mapped) = Self::map_gilrs_button(button) {
                            let gamepad = &mut self.gamepads[gamepad_id];
                            if !gamepad.buttons.contains(&mapped) {
                                gamepad.buttons_pressed.insert(mapped);
                            }
                            gamepad.buttons.insert(mapped);
                        }
                    }
                    gilrs::EventType::ButtonReleased(button, _) => {
                        if let Some(mapped) = Self::map_gilrs_button(button) {
                            let gamepad = &mut self.gamepads[gamepad_id];
                            if gamepad.buttons.contains(&mapped) {
                                gamepad.buttons_released.insert(mapped);
                            }
                            gamepad.buttons.remove(&mapped);
                        }
                    }
                    gilrs::EventType::AxisChanged(axis, value, _) => {
                        let gamepad = &mut self.gamepads[gamepad_id];

                        // Apply deadzone
                        let value = if value.abs() < 0.15 { 0.0 } else { value };

                        match axis {
                            gilrs::Axis::LeftStickX => {
                                gamepad.axes.insert(GamepadAxis::LeftStickX, value);
                                gamepad.left_stick.x = value;
                            }
                            gilrs::Axis::LeftStickY => {
                                gamepad.axes.insert(GamepadAxis::LeftStickY, -value); // Invert Y
                                gamepad.left_stick.y = -value;
                            }
                            gilrs::Axis::RightStickX => {
                                gamepad.axes.insert(GamepadAxis::RightStickX, value);
                                gamepad.right_stick.x = value;
                            }
                            gilrs::Axis::RightStickY => {
                                gamepad.axes.insert(GamepadAxis::RightStickY, -value); // Invert Y
                                gamepad.right_stick.y = -value;
                            }
                            gilrs::Axis::LeftZ => {
                                gamepad.axes.insert(GamepadAxis::LeftTrigger, value);
                            }
                            gilrs::Axis::RightZ => {
                                gamepad.axes.insert(GamepadAxis::RightTrigger, value);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn map_gilrs_button(button: gilrs::Button) -> Option<GamepadButton> {
        match button {
            gilrs::Button::South => Some(GamepadButton::South),
            gilrs::Button::East => Some(GamepadButton::East),
            gilrs::Button::North => Some(GamepadButton::North),
            gilrs::Button::West => Some(GamepadButton::West),
            gilrs::Button::LeftTrigger => Some(GamepadButton::L1),
            gilrs::Button::RightTrigger => Some(GamepadButton::R1),
            gilrs::Button::LeftTrigger2 => Some(GamepadButton::L2),
            gilrs::Button::RightTrigger2 => Some(GamepadButton::R2),
            gilrs::Button::LeftThumb => Some(GamepadButton::L3),
            gilrs::Button::RightThumb => Some(GamepadButton::R3),
            gilrs::Button::Start => Some(GamepadButton::Start),
            gilrs::Button::Select => Some(GamepadButton::Select),
            gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
            gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
            gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
            gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
            _ => None,
        }
    }
}
