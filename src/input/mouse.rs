use std::collections::HashMap;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use crate::input::yhwh_keys::YHWHMouseButton;

pub struct Mouse {
    pub delta_x: f64,
    pub delta_y: f64,
    scroll_dx: f32,
    scroll_dy: f32,
    buttons: HashMap<YHWHMouseButton, bool>,
    buttons_changed: HashMap<YHWHMouseButton, bool>,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            scroll_dx: 0.0,
            scroll_dy: 0.0,
            buttons: HashMap::new(),
            buttons_changed: HashMap::new()
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let mapped_button = map_button(*button);
                let is_pressed = state == &ElementState::Pressed;
                let prev_state = self.buttons.get(&mapped_button).cloned().unwrap_or(false);

                self.buttons.insert(mapped_button.clone(), is_pressed);
                self.buttons_changed.insert(mapped_button, prev_state != is_pressed);
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
          match event {
            DeviceEvent::MouseMotion { delta } => {
                self.delta_x = delta.0;
                self.delta_y = delta.1;
            }
            _ => {}
        }
    }

    pub fn button_pressed(&self, button: &YHWHMouseButton) -> bool {
        self.buttons.get(&button).cloned().unwrap_or(false)
    }

    pub fn button_just_pressed(&self, button: &YHWHMouseButton) -> bool {
        self.button_pressed(&button) && self.buttons_changed.get(&button).cloned().unwrap_or(false)
    }

    pub fn button_just_released(&self, button: &YHWHMouseButton) -> bool {
        !self.button_pressed(&button) && self.buttons_changed.get(&button).cloned().unwrap_or(false)
    }

    pub fn button_changed(&self, button: &YHWHMouseButton) -> bool {
        self.buttons_changed.get(&button).cloned().unwrap_or(false)
    }

    pub fn end_frame(&mut self) {
        self.buttons_changed.clear();
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }
}

fn map_button(button: MouseButton) -> YHWHMouseButton {
    match button {
        MouseButton::Left => YHWHMouseButton::Left,
        MouseButton::Right => YHWHMouseButton::Right,
        MouseButton::Middle => YHWHMouseButton::Middle,
        MouseButton::Back => YHWHMouseButton::Back,
        MouseButton::Forward => YHWHMouseButton::Forward,
        MouseButton::Other(b) => YHWHMouseButton::Other(b),
    }
}