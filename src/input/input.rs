use crate::input::{keyboard::Keyboard, mouse::Mouse};

pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse
}

impl Input {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new()
        }
    }
}