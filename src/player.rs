use std::time::Duration;

use winit::keyboard::KeyCode;

use crate::{camera::{Camera, CameraController}, engine::GameData, input::input::Input};

pub struct Player {
    position: cgmath::Vector3<f32>,
    camera_controller: CameraController,
    pub camera: Camera,
}

impl Player {
    pub fn new() -> Self {
        let pos = cgmath::Vector3::new(4.0, 7.0, 20.0);
        let speed = 4.0;
        let sensitivity = 0.4;

        Self {
            position: pos,
            camera: Camera::new(cgmath::Point3::new(pos.x, pos.y, pos.z), cgmath::Deg(-90.0), cgmath::Deg(-20.0)),
            camera_controller: CameraController::new(speed, sensitivity)
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: Duration) {
        self.camera_controller.update_movement_player(input);
        self.camera_controller.update_camera(&mut self.camera, delta_time);
    }

    pub fn moving(&self) -> bool {
        self.camera_controller.moving
    }
}