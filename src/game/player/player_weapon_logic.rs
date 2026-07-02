use cgmath::{Angle, Bounded, Euler, InnerSpace, Matrix3, Matrix4, Quaternion, Rad, Vector3};
use rand::Rng;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;

use crate::{animation::animation::PlaybackMode, game::{player::player::Player, weapon::WeaponAction}, input::{input::Input, yhwh_keys::YHWHMouseButton}};

impl<'a> Player {
    pub fn update_weapon_logic(&mut self, input: &Input, audio_manager: &mut AudioManager) {
        let moving = self.moving();

        let weapon = &mut self.weapon_animated_game_object;
        let desert_eagle = &mut self.weapon_manager.desert_eagle_info;
        let current_anim_state = &weapon.get_animation_playback_state().unwrap();

        let current_is_idle = current_anim_state.current == desert_eagle.animations.idle;
        let current_is_sprint_end = current_anim_state.current == desert_eagle.animations.sprint_end;
         let current_is_sprint_start = current_anim_state.current == desert_eagle.animations.sprint_start;
        let current_is_fire = current_anim_state.current == desert_eagle.animations.fire;
        let current_is_ads_fire = current_anim_state.current == desert_eagle.animations.ads_fire;
        let current_is_draw = current_anim_state.current == desert_eagle.animations.draw;
        let current_is_initial_draw = current_anim_state.current == desert_eagle.animations.initial_draw;
        let current_anim_finished = current_anim_state.time >= current_anim_state.total_time;

        let pressing_fire = input.mouse.button_pressed(&YHWHMouseButton::Left);
        let pressing_ads = input.mouse.button_pressed(&YHWHMouseButton::Right);
        let draw_pressed = input.keyboard.key_just_pressed(KeyCode::KeyT);

        let fire_audios = [
            "deagle_fire_1.wav",
            "deagle_fire_2.wav",
            "deagle_fire_3.wav",
        ];

        if !desert_eagle.has && draw_pressed {
            weapon.toggle_animation();
            if current_anim_state.paused {
                let _ = audio_manager.play_audio("deagle_initial_draw.wav", 1.0, 1.0);

                weapon.set_animation_playback_mode(PlaybackMode::Once);
                weapon.set_current_animation(desert_eagle.animations.initial_draw);
                desert_eagle.has = true;
                desert_eagle.equipped = true;
                self.weapon_action = WeaponAction::InitialDraw;
            }
        } else if desert_eagle.has && desert_eagle.equipped && draw_pressed {
            let _ = audio_manager.play_audio("deagle_holster.wav", 1.0, 1.0);
            weapon.set_animation_playback_mode(PlaybackMode::Once);
            weapon.set_current_animation(desert_eagle.animations.holster);
            desert_eagle.equipped = false;
        } else if desert_eagle.has && !desert_eagle.equipped && draw_pressed {
            let _ = audio_manager.play_audio("deagle_draw.wav", 1.0, 1.0);

            weapon.set_animation_playback_mode(PlaybackMode::Once);
            weapon.set_current_animation(desert_eagle.animations.draw);
            self.weapon_action = WeaponAction::Draw;
            desert_eagle.equipped = true;
        }

        if desert_eagle.has {
            if desert_eagle.equipped {
                if (self.weapon_action == WeaponAction::Idle || self.weapon_action == WeaponAction::AdsToIdle) && pressing_ads {
                    let _ = audio_manager.play_audio("deagle_ads_1.wav", 1.0, 1.0);

                    weapon.set_animation_playback_mode(PlaybackMode::Once);
                    weapon.set_current_animation(desert_eagle.animations.idle_to_ads);
                    self.weapon_action = WeaponAction::IdleToAds;
                }

                if pressing_fire && pressing_ads && self.weapon_action != WeaponAction::AdsFire {
                    let mut rng = rand::thread_rng();
                    let rand_audio = rng.gen_range(0..2);
                    let _ = audio_manager.play_audio(&fire_audios[rand_audio], 1.0, 1.0);

                    weapon.set_animation_playback_mode(PlaybackMode::Once);
                    weapon.set_current_animation(desert_eagle.animations.ads_fire);
                    self.weapon_action = WeaponAction::AdsFire;
                }

                if pressing_fire && !pressing_ads && self.weapon_action != WeaponAction::Fire {
                    let mut rng = rand::thread_rng();
                    let rand_audio = rng.gen_range(0..2);
                    let _ = audio_manager.play_audio(&fire_audios[rand_audio], 1.0, 1.0);

                    weapon.set_animation_playback_mode(PlaybackMode::Once);
                    weapon.set_current_animation(desert_eagle.animations.fire);
                    self.weapon_action = WeaponAction::Fire;
                }

                if moving && self.weapon_action != WeaponAction::SprintStart
                    && self.weapon_action != WeaponAction::Sprint
                    && !pressing_fire
                    && !current_is_fire
                    && !current_is_initial_draw 
                {
                    weapon.set_animation_playback_mode(PlaybackMode::Once);
                    weapon.set_current_animation(desert_eagle.animations.sprint_start);
                    self.weapon_action = WeaponAction::SprintStart;
                }

                match self.weapon_action {
                    // ads state machine
                    WeaponAction::IdleToAds => {
                        if current_anim_finished && pressing_ads {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.ads_idle);
                            self.weapon_action = WeaponAction::AdsIdle;
                        } else if !pressing_ads {
                            weapon.set_animation_playback_mode(PlaybackMode::Once);
                            weapon.set_current_animation(desert_eagle.animations.ads_to_idle);
                            self.weapon_action = WeaponAction::AdsToIdle;
                        }
                    }
                    WeaponAction::AdsFire => {
                        if pressing_ads && current_is_ads_fire && current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.ads_idle);
                            self.weapon_action = WeaponAction::AdsIdle;
                        } else if !pressing_ads && current_is_ads_fire {
                            weapon.set_animation_playback_mode(PlaybackMode::Once);
                            weapon.set_current_animation(desert_eagle.animations.ads_to_idle);
                            self.weapon_action = WeaponAction::AdsToIdle;
                        }
                    }
                    WeaponAction::AdsIdle => {
                       if !pressing_ads {
                        weapon.set_animation_playback_mode(PlaybackMode::Once);
                        weapon.set_current_animation(desert_eagle.animations.ads_to_idle);
                        self.weapon_action = WeaponAction::AdsToIdle;
                       }
                    }
                    WeaponAction::AdsToIdle => {
                        if current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }

                    // state machine
                    WeaponAction::InitialDraw => {
                        if current_is_initial_draw && current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }

                    WeaponAction::Draw => {
                         if current_is_draw && current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }
                    WeaponAction::Fire => {
                        if current_is_fire && current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }
                    WeaponAction::SprintStart => {
                        if current_is_sprint_start && current_anim_state.time > 0.239 {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.sprint);
                            self.weapon_action = WeaponAction::Sprint;
                        } else if !moving {
                            weapon.set_animation_playback_mode(PlaybackMode::Once);
                            weapon.set_current_animation(desert_eagle.animations.sprint_end);
                            self.weapon_action = WeaponAction::SprintEnd;
                        }
                    }
                    WeaponAction::Sprint => {
                        if !moving {
                            weapon.set_animation_playback_mode(PlaybackMode::Once);
                            weapon.set_current_animation(desert_eagle.animations.sprint_end);
                            self.weapon_action = WeaponAction::SprintEnd;
                        }
                    }
                    WeaponAction::SprintEnd => {
                        if current_is_sprint_end && current_anim_finished {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }
                    WeaponAction::Walk => {
                        if !moving {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }
                    _ => {
                        if !moving {
                            weapon.set_animation_playback_mode(PlaybackMode::Loop);
                            weapon.set_current_animation(desert_eagle.animations.idle);
                            self.weapon_action = WeaponAction::Idle;
                        }
                    }
                }
            }
        } else {
            weapon.stop_animation();
        }


        // place_weapon_in_fps_view //
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

        weapon.model_matrix = model;
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