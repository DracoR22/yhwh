use std::collections::HashMap;

use cgmath::Vector2;
use rapier3d::na::Vector;
use yhwh_audio::audio_manager::AudioManager;

use crate::{asset_manager::AssetManager, camera::{Camera, CameraController}, common::enums::GameState, game::{player::Player, ui::{UiElement, UiElementKind}}, input::input::Input, scene::Scene};

pub struct GameData {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub asset_manager: AssetManager,
    pub scene: Scene,
    pub delta_time: std::time::Duration,
    pub last_redraw: std::time::Instant,
    pub fps_accum: Vec<f64>,
    pub avg_fps: f64,
    pub game_state: GameState,
    pub player: Player,
    pub ui_map: HashMap<UiElementKind, UiElement>
}

impl GameData {
    pub fn new(asset_manager: AssetManager, (w_width, w_height): (f32, f32)) -> Self {
        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = CameraController::new(8.0, 0.4);

        let mut scene = Scene::new(&asset_manager);

        let mut ui_map = HashMap::<UiElementKind, UiElement>::new();
        let crosshair_pos = Vector2::new(w_width * 0.5, w_height * 0.5);
        let crosshair_scale = 50.0;
        ui_map.insert(UiElementKind::Crosshair, UiElement::new(crosshair_pos, crosshair_scale, "crosshair179.png"));

        let key_g_pos = Vector2::new(crosshair_pos.x + 60.0, crosshair_pos.y);
        let key_g_scale = 50.0;
        ui_map.insert(UiElementKind::KeyG, UiElement::new(key_g_pos, key_g_scale, "Key_G.png"));

        let player = Player::new(&mut scene, &asset_manager);

        Self {
            asset_manager,
            scene,
            camera,
            camera_controller,
            avg_fps: 0.0,
            fps_accum: Default::default(),
            delta_time: std::time::Duration::new(0, 0),
            last_redraw: std::time::Instant::now(),
            game_state: GameState::Playing,
            player,
            ui_map
        }
    }

    pub fn update(&mut self, input: &Input, audio_manager: &mut AudioManager) {
        self.update_fps();
        
        for light in self.scene.lights.iter_mut() {
            light.update();
        }

        for door in self.scene.door_objects.iter_mut() {
            door.update(self.delta_time.as_secs_f32());
            if door.interact(&self.asset_manager, &self.player.camera) {
                door.toggle_state(audio_manager, &input);
                self.player.can_interact = true;
            } else {
                self.player.can_interact = false;
            }
        }

        if let Some(crosshair) = self.ui_map.get_mut(&UiElementKind::Crosshair) {
            crosshair.visible = self.player.can_interact;
        }
        
        if let Some(interact_key) = self.ui_map.get_mut(&UiElementKind::KeyG) {
            interact_key.visible = self.player.can_interact;
        }
    
        match self.game_state {
            GameState::Playing => {
                self.player.update(&input, self.delta_time, audio_manager, &mut self.scene);
            },
            GameState::Editor => {
                self.camera_controller.update_movement_editor(&input);
                self.camera_controller.update_camera(&mut self.camera, self.delta_time);
            }
        }

        let projection = self.active_camera().get_projection().calc_matrix();
        let view = self.active_camera().calc_matrix();

        self.active_camera_mut().frustum.update(&(projection * view));
    }

    pub fn update_fps(&mut self) {
        let now = std::time::Instant::now();
        self.delta_time = now - self.last_redraw;
        self.last_redraw = now;

        let fps = 1.0 / self.delta_time.as_secs_f64();
        self.fps_accum.push(fps);
            if self.fps_accum.len() > 100 {
            self.fps_accum.remove(0);
        }

        self.avg_fps = self.fps_accum.iter().sum::<f64>() / self.fps_accum.len() as f64;
    }

    pub fn active_camera(&self) -> &Camera {
        match self.game_state {
            GameState::Playing => &self.player.camera,
            GameState::Editor => &self.camera
        }
    }

    pub fn active_camera_mut(&mut self) -> &mut Camera {
        match self.game_state {
            GameState::Playing => &mut self.player.camera,
            GameState::Editor => &mut self.camera
        }
    }
}
