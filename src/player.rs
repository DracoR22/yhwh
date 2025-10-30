use crate::camera::{Camera, CameraController};

pub struct Player {
    position: cgmath::Vector3<f32>,
    camera: Camera,
    camera_controller: CameraController
}

impl Player {
    pub fn new() -> Self {
        let pos = cgmath::Vector3::new(0.0, 1.0, 0.0);
        let speed = 4.0;
        let sensitivity = 0.4;

        Self {
            position: pos,
            camera: Camera::new(cgmath::Point3::new(pos.x, pos.y, pos.z), cgmath::Deg(-90.0), cgmath::Deg(-20.0)),
            camera_controller: CameraController::new(speed, sensitivity)
        }
    }

    pub fn update(&self) {

    }
}