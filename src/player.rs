use std::time::Duration;

use rand::Rng;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;

use crate::{camera::{Camera, CameraController}, engine::GameData, input::input::Input};

const WOOD_FOOTSTEPS: [&str; 4] = [
    "wood1.wav",
    "wood2.wav",
    "wood3.wav",
    "wood4.wav",
];

pub struct Player {
    position: cgmath::Vector3<f32>,
    camera_controller: CameraController,
    step_timer: f32,
    pub camera: Camera,
}

impl Player {
    pub fn new() -> Self {
        let pos = cgmath::Vector3::new(4.0, 6.0, 20.0);
        let speed = 4.0;
        let sensitivity = 0.4;

        Self {
            position: pos,
            camera: Camera::new(cgmath::Point3::new(pos.x, pos.y, pos.z), cgmath::Deg(-90.0), cgmath::Deg(-20.0)),
            camera_controller: CameraController::new(speed, sensitivity),
            step_timer: 0.0
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: Duration, audio_manager: &mut AudioManager) {
        self.camera_controller.update_movement_player(input);
        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.update_audio(audio_manager, delta_time);
    }

    pub fn update_audio(&mut self, audio_manager: &mut AudioManager, delta_time: Duration) {
        let interval = 0.6;

        if self.moving() {
          self.step_timer -= delta_time.as_secs_f32();

          if self.step_timer <= 0.0 {
            let random_index = rand::thread_rng().gen_range(0..WOOD_FOOTSTEPS.len());
            audio_manager.play_audio(WOOD_FOOTSTEPS[random_index], 1.0, 0.5);
            self.step_timer = interval;
          }
        }
    }

    pub fn moving(&self) -> bool {
        self.camera_controller.moving
    }
}