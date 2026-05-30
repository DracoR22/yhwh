use std::collections::HashMap;

use egui::{load::SizedTexture, Ui, Vec2};

use crate::{
    animation::animation::PlaybackMode, asset_manager::AssetManager, egui_renderer::ui_manager::EguiMaterial, game::game_data::GameData, objects::animated_game_object::AnimatedGameObject, scene::Scene
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum EditMode {
    Transform,
    Animation,
}

pub struct AnimatedGameObjects {
    selected_mesh_index_map: HashMap<usize, usize>,
    selected_id: i32,
    scale_uniform: bool,
    selected_anim_map: HashMap<usize, usize>,
    edit_mode: EditMode,
    pub should_reset_other_states: bool,
}

impl AnimatedGameObjects {
    pub fn new() -> Self {
        Self {
            selected_mesh_index_map: HashMap::new(),
            selected_id: -1,
            scale_uniform: true,
            should_reset_other_states: false,
            selected_anim_map: HashMap::new(),
            edit_mode: EditMode::Transform,
        }
    }

    pub fn apply_selection(&mut self, chunk: &mut Scene) {
        for animated_game_object in chunk.animated_game_objects.iter_mut() {
            if animated_game_object.id as i32 == self.selected_id {
                animated_game_object.is_selected = true;
            } else {
                animated_game_object.is_selected = false;
            }
        }
    }

    pub fn list(&mut self, ui: &mut Ui, chunk: &mut Scene, materials: &Vec<EguiMaterial>) {
        ui.collapsing("Animated Game Objects", |ui| {
            for (index, animated_game_object) in
                chunk.animated_game_objects.iter().enumerate()
            {
                let button = ui.button(
                    animated_game_object.get_model_name().to_string()
                        + " ("
                        + &index.to_string()
                        + ")",
                );

                if button.clicked() {
                    self.selected_id = animated_game_object.id as i32;
                    self.should_reset_other_states = true;
                }
            }
        });
    }

    pub fn update(
        &mut self,
        ui: &mut Ui,
        chunk: &mut Scene,
        materials: &Vec<EguiMaterial>,
        (window_width, window_height): (u32, u32),
    ) {
        if self.selected_id != -1 {
            for animated_game_object in chunk.animated_game_objects.iter_mut() {
                if animated_game_object.id as i32 == self.selected_id {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.edit_mode, EditMode::Transform, "Transform");
                        ui.selectable_value(&mut self.edit_mode, EditMode::Animation, "Animation");
                    });

                    match self.edit_mode {
                        EditMode::Transform => self.transform_ui(ui, animated_game_object),
                        EditMode::Animation => self.animation_ui(ui, animated_game_object),
                    };
                }
            }
        }
    }

    fn transform_ui(&mut self, ui: &mut Ui, animated_game_object: &mut AnimatedGameObject) {
            ui.add_space(3.0);

            ui.label("Position X");
            ui.add(egui::DragValue::new(
                &mut animated_game_object.transform.position.x,
            ));

            ui.label("Position Y");
            ui.add(egui::DragValue::new(
                &mut animated_game_object.transform.position.y,
            ));

            ui.label("Position Z");
            ui.add(egui::DragValue::new(
                &mut animated_game_object.transform.position.z,
            ));

            ui.checkbox(&mut self.scale_uniform, "Scale Uniform");

            let mut size = animated_game_object.transform.size;

            let changed_x = ui
                .add(egui::Slider::new(&mut size.x, 0.0..=100.0).text("Size X"))
                .changed();

            let changed_y = ui
                .add(egui::Slider::new(&mut size.y, 0.0..=100.0).text("Size Y"))
                .changed();

            let changed_z = ui
                .add(egui::Slider::new(&mut size.z, 0.0..=100.0).text("Size Z"))
                .changed();

            if changed_x || changed_y || changed_z {
                if self.scale_uniform {
                    let new_value = if changed_x {
                        size.x
                    } else if changed_y {
                        size.y
                    } else {
                        size.z
                    };

                    animated_game_object.transform.size =
                        cgmath::Vector3::new(new_value, new_value, new_value);
                } else {
                    animated_game_object.transform.size = size;
                }
            }

            ui.label("Rotation X");
            let mut rotation = animated_game_object.transform.rotation;
            let slider_rot_x = ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0).suffix("°"));
            ui.label("Rotation Y");
            let slider_rot_y = ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0).suffix("°"));
            ui.label("Rotation Z");
            let slider_rot_z = ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0).suffix("°"));

            if slider_rot_x.changed()
                || slider_rot_y.changed()
                || slider_rot_z.changed()
                || slider_rot_x.changed()
            {
                animated_game_object.transform.rotation = rotation;
            }
    }

    fn animation_ui(&mut self, ui: &mut Ui, animated_game_object: &mut AnimatedGameObject) {
       ui.add_space(3.0);

        let selected_anim_index = self
                .selected_anim_map
                .entry(animated_game_object.id)
                .or_insert(0);
            let selected_anim = &animated_game_object
                .animations
                .as_ref()
                .unwrap()
                .animations()[*selected_anim_index];

        let mut value_changed = false;
        egui::ComboBox::from_label("")
                .selected_text(selected_anim.get_name())
                .show_ui(ui, |ui| {
                    for (i, anim) in animated_game_object
                        .animations
                        .as_ref()
                        .unwrap()
                        .animations()
                        .iter()
                        .enumerate()
                    {
                        value_changed |= ui
                            .selectable_value(selected_anim_index, i, anim.get_name())
                            .changed();
                    }
                });

            if value_changed {
                animated_game_object.set_current_animation(*selected_anim_index);
            }

            let play_back_state = animated_game_object.get_animation_playback_state().unwrap();
            ui.add(
                egui::ProgressBar::new(play_back_state.progress()).text(format!(
                    "{:.2}s / {:.2}s",
                    play_back_state.time, play_back_state.total_time
                )),
            );

            let mut loop_anim = matches!(play_back_state.playback_mode, PlaybackMode::Loop);

            if ui.checkbox(&mut loop_anim, "Loop").changed() {
                let mode = if loop_anim {
                    PlaybackMode::Loop
                } else {
                    PlaybackMode::Once
                };

                animated_game_object.set_animation_playback_mode(mode);
            }

            let mut anim_paused = matches!(play_back_state.paused, true);
            if ui.checkbox(&mut anim_paused, "Paused").changed() {
                animated_game_object.toggle_animation();
            }
    }
}
