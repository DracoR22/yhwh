
use std::collections::HashMap;
use winit::{event::{ElementState, KeyEvent,  WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

pub struct Keyboard {
    keys: HashMap<KeyCode, bool>,
    keys_changed: HashMap<KeyCode, bool>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            keys_changed: HashMap::new(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state,
                    ..
                },
            ..
        } = event
        {
            let is_pressed = state == &ElementState::Pressed;
            let prev_state = self.keys.get(code).cloned().unwrap_or(false);
            self.keys.insert(*code, is_pressed);
            self.keys_changed.insert(*code, prev_state != is_pressed);
        }
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys.get(&key).cloned().unwrap_or(false)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.key_pressed(key) && self.keys_changed.get(&key).cloned().unwrap_or(false)
    }

    pub fn key_just_released(&self, key: KeyCode) -> bool {
        !self.key_pressed(key) && self.keys_changed.get(&key).cloned().unwrap_or(false)
    }

    pub fn key_changed(&self, key: KeyCode) -> bool {
        self.keys_changed.get(&key).cloned().unwrap_or(false)
    }

    pub fn end_frame(&mut self) {
        self.keys_changed.clear();
    }
}