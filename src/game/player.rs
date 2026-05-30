use std::time::Duration;

use cgmath::{Angle, Array, Basis3, EuclideanSpace, Euler, InnerSpace, Matrix3, Matrix4, MetricSpace, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector2, Vector3, num_traits::bounds::LowerBounded};
use rand::Rng;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;

use crate::{animation::animation::PlaybackMode, asset_manager::AssetManager, camera::{Camera, CameraController}, common::create_info::{AnimatedGameObjectCreateInfo, GameObjectCreateInfo}, game::{game_data::GameData, weapon::{WeaponAction, WeaponAnimations, WeaponInfo, WeaponManager}}, input::{input::Input, yhwh_keys::YHWHMouseButton}, objects::animated_game_object::AnimatedGameObject, scene::Scene};

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
    weapon_animated_game_object: AnimatedGameObject,
    weapon_action: WeaponAction,
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
            mesh_rendering_info: vec![]
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

        // let candle_sound_distance = 20.0;
        // for game_object in scene.game_objects.iter() {
        //     if game_object.model_name == "candles" {
        //         let player_pos = &self.camera.position;

        //         if player_pos.distance2(Point3::from_vec(game_object.transform.position)) < candle_sound_distance {
        //             audio_manager.play_audio("fire_small.wav", 1.0, 0.3);
        //         }
        //     }
        // }
        
        // weapon
        let moving = self.moving();

        let weapon = &mut self.weapon_animated_game_object;
        let desert_eagle = &mut self.weapon_manager.desert_eagle_info;
        let current_anim_state = &weapon.get_animation_playback_state().unwrap();

        let current_is_fire = current_anim_state.current == desert_eagle.animations.fire;
        let current_is_holster = current_anim_state.current == desert_eagle.animations.holster;
        let current_is_draw = current_anim_state.current == desert_eagle.animations.draw;
        let current_is_initial_draw = current_anim_state.current == desert_eagle.animations.initial_draw;
        let current_anim_finished = current_anim_state.time >= current_anim_state.total_time;

        let fire_pressed = input.mouse.button_just_pressed(&YHWHMouseButton::Left);
        let draw_pressed = input.keyboard.key_just_pressed(KeyCode::KeyT);

        if !desert_eagle.has && draw_pressed {
            weapon.toggle_animation();
            if current_anim_state.paused {
            weapon.set_animation_playback_mode(PlaybackMode::Once);
            weapon.set_current_animation(desert_eagle.animations.initial_draw);
            desert_eagle.has = true;
            desert_eagle.equipped = true;
            self.weapon_action = WeaponAction::InitialDraw;
            }
        } else if desert_eagle.has && desert_eagle.equipped && draw_pressed {
            weapon.set_animation_playback_mode(PlaybackMode::Once);
            weapon.set_current_animation(desert_eagle.animations.holster);
            desert_eagle.equipped = false;
        } else if desert_eagle.has && !desert_eagle.equipped && draw_pressed {
            weapon.set_animation_playback_mode(PlaybackMode::Once);
            weapon.set_current_animation(desert_eagle.animations.draw);
            desert_eagle.equipped = true;
        }

        if desert_eagle.has {
            if desert_eagle.equipped {
                if fire_pressed {
                weapon.set_animation_playback_mode(PlaybackMode::Once);
                weapon.set_current_animation(desert_eagle.animations.fire);
                self.weapon_action = WeaponAction::Fire;
            } 

            if moving {
                weapon.set_animation_playback_mode(PlaybackMode::Loop);
                weapon.set_current_animation(desert_eagle.animations.walk);
                self.weapon_action = WeaponAction::Walk;
            } 
            
            if !moving && 
            (!fire_pressed  && !current_is_fire) && 
            (!draw_pressed && !current_is_initial_draw) && 
            (!draw_pressed && !current_is_holster) &&
            (!draw_pressed && !current_is_draw) {
                weapon.set_animation_playback_mode(PlaybackMode::Loop);
                weapon.set_current_animation(desert_eagle.animations.idle);
                self.weapon_action = WeaponAction::Idle;
             }
            }
        } else {
            weapon.stop_animation();
        }

        let camera = &self.camera;
        let forward = camera.forward();
        let right = Vector3::unit_y().cross(forward).normalize();
        let up = forward.cross(right).normalize();
        let world_up = Vector3::new(0.0, 1.0, 0.0);

        let forward_offset = if desert_eagle.has && !current_anim_state.paused { 0.0 } else { -5.0 }; // hack to hide it
        let offset = forward * forward_offset + right * -0.3 - up * 2.6;

        let pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z) + offset;
        let translation_matrix = Matrix4::from_translation(pos);

        let mut theta = Rad::min_value();
        theta = Rad::acos(world_up.dot(forward) / up.magnitude() / forward.magnitude());
        let rotation = Matrix4::from_axis_angle(right, theta - Rad::turn_div_4());

        let rotation_angle = Rad(forward.z.atan2(forward.x));
        let rotation_matrix = Matrix4::from_axis_angle(
            world_up,
            -rotation_angle
        );

        let scale_matrix = Matrix4::from_nonuniform_scale(weapon.transform.size.x, weapon.transform.size.y, weapon.transform.size.z);

        let combined_rotation = rotation * rotation_matrix;
        let model = translation_matrix * combined_rotation * scale_matrix;

        let (p, r, s) = decompose_matrix(model);

        // weapon.transform.position = p;
        // weapon.transform.rotation = r;
        // weapon.transform.size = s;
        weapon.model_matrix = model;
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

fn decompose_matrix(
    model: Matrix4<f32>,
) -> (
    Vector3<f32>,
    Vector3<f32>,
    Vector3<f32>,
) {
    // Position
    let position = model.w.truncate();

    // Scale
    let scale_x = model.x.truncate().magnitude();
    let scale_y = model.y.truncate().magnitude();
    let scale_z = model.z.truncate().magnitude();

    let scale = Vector3::new(
        scale_x,
        scale_y,
        scale_z,
    );

    // Remove scale
    let right =
        model.x.truncate() / scale_x;

    let up =
        model.y.truncate() / scale_y;

    let forward =
        model.z.truncate() / scale_z;

    // Rotation matrix
    let rotation_matrix =
        Matrix3::from_cols(
            right,
            up,
            forward,
        );

    // Quaternion
    let quat =
        Quaternion::from(rotation_matrix);

    // Euler
    let euler: Euler<Rad<f32>> =
        Euler::from(quat);

    let rotation = Vector3::new(
        euler.x.0.to_degrees(),
        euler.y.0.to_degrees(),
        euler.z.0.to_degrees(),
    );

    (
        position,
        rotation,
        scale,
    )
}