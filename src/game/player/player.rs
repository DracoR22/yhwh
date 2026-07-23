use std::time::Duration;

use cgmath::{Angle, Array, Basis3, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4, MetricSpace, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector2, Vector3, num_traits::bounds::LowerBounded};
use rand::Rng;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;
use yhwh_core::common::{create_info::{AnimatedGameObjectCreateInfo, MeshNodeCreateInfo}, enums::MeshRenderingMode};

use crate::{animation::animation::PlaybackMode, asset_manager::AssetManager, camera::{Camera, CameraController}, game::{game_data::GameData, weapon::{WeaponAction, WeaponAnimations, WeaponInfo, WeaponManager}}, input::{input::Input, yhwh_keys::YHWHMouseButton}, objects::animated_game_object::AnimatedGameObject};

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
    pub weapon_action: WeaponAction,
    pub weapon_animated_game_object: AnimatedGameObject,
    pub weapon_manager: WeaponManager,
    pub camera: Camera,
    pub can_interact: bool
}

impl<'a> Player {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let pos = cgmath::Vector3::new(4.0, 6.0, 20.0);
        let speed = 6.0;
        let sensitivity = 0.4;

        let weapon_create_info = AnimatedGameObjectCreateInfo {
            model_name: "untitled2".to_string(),
            position: [10.0, 2.0, 0.0],
            rotation: [1.0, 1.0, 1.0],
            size: [0.1, 0.1, 0.1],
            tex_scale: [1.0, 1.0],
            loop_anim: true,
            mesh_rendering_info: Player::desert_eagle_mesh_nodes()
        };

        //let weapon_object_id = scene.add_animated_game_object(&weapon_create_info, asset_manager);
        let weapon_animated_game_object = AnimatedGameObject::new(&weapon_create_info, asset_manager);
        let weapon_manager = WeaponManager::new(weapon_animated_game_object.id);

        Self {
            position: pos,
            camera: Camera::new(cgmath::Point3::new(pos.x, pos.y, pos.z), cgmath::Deg(-90.0), cgmath::Deg(-20.0)),
            camera_controller: CameraController::new(speed, sensitivity),
            step_timer: 0.0,
            can_interact: false,
            weapon_action: WeaponAction::Idle,
            weapon_animated_game_object,
            weapon_manager
        }
    }

    pub fn update(&mut self, input: &Input, delta_time: Duration, audio_manager: &mut AudioManager) {
        self.camera_controller.update_movement_player(input);
        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.update_audio(audio_manager, delta_time);
        self.update_weapon_logic(input, audio_manager);

        let fire_pos = Point3::new(3.3, 3.98, 71.0);
        let max_fire_distance = 40.0;
        let near_fire_distance = self.camera.position.distance(fire_pos);
        let playing_fire_audio = audio_manager.is_playing("fire_small1.wav");

        // if near_fire_distance <= max_fire_distance && !playing_fire_audio {
        //    let _ = audio_manager.play_audio("fire_small1.wav", 1.0, 0.0);
        // }
    }

    pub fn update_audio(&mut self, audio_manager: &mut AudioManager, delta_time: Duration) {
        let interval = 0.6;

        if self.moving() {
          self.step_timer -= delta_time.as_secs_f32();

          if self.step_timer <= 0.0 {
            let random_index = rand::thread_rng().gen_range(0..WOOD_FOOTSTEPS.len());
            let _ = audio_manager.play_audio(WOOD_FOOTSTEPS[random_index], 1.0, 0.5);
            self.step_timer = interval;
          }
        }
    }

    pub fn desert_eagle_mesh_nodes() -> Vec<MeshNodeCreateInfo> {
        vec![
            MeshNodeCreateInfo {
                mesh_name: String::from("M_Base_Skin_Sleve"),
                material_name: String::from("Default"),
                rendering_mode: MeshRenderingMode::Pbr
            },
            MeshNodeCreateInfo {
                mesh_name: String::from("M_Base_Skin_Mat"),
                material_name: String::from("Arms"),
                rendering_mode: MeshRenderingMode::Pbr
            },
              MeshNodeCreateInfo {
                mesh_name: String::from("M_DFK"),
                material_name: String::from("Arms"),
                rendering_mode: MeshRenderingMode::Pbr
            },

            MeshNodeCreateInfo {
                mesh_name: String::from("M_Deagle_Slide"),
                material_name: String::from("DEagle_Frame"),
                rendering_mode: MeshRenderingMode::Pbr
            },
            MeshNodeCreateInfo {
                mesh_name: String::from("M_Deagle_Frame"),
                material_name: String::from("DEagle_Rec"),
                rendering_mode: MeshRenderingMode::Pbr
            },
            MeshNodeCreateInfo {
                mesh_name: String::from("M_Deagle_Grip"),
                material_name: String::from("DEagle_Grip"),
                rendering_mode: MeshRenderingMode::Pbr
            },

            MeshNodeCreateInfo {
                mesh_name: String::from("Deagle_Mag"),
                material_name: String::from("Default"),
                rendering_mode: MeshRenderingMode::Pbr
            },
            MeshNodeCreateInfo {
                mesh_name: String::from("Deagle_Mag.001"),
                material_name: String::from("Default"),
                rendering_mode: MeshRenderingMode::Pbr
            },
        ]
    }

    pub fn set_weapon_action(&mut self, action: WeaponAction) {
        self.weapon_action = action;
    }

    pub fn weapon_action(&self) -> &WeaponAction {
        &self.weapon_action
    }

    pub fn moving(&self) -> bool {
        self.camera_controller.moving
    }

    pub fn weapon_animated_game_object(&self) -> &AnimatedGameObject {
        &self.weapon_animated_game_object
    }

    pub fn weapon_animated_game_object_mut(&mut self) ->  &mut AnimatedGameObject {
        &mut self.weapon_animated_game_object
    }
}