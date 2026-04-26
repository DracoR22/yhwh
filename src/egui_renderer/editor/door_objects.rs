use std::collections::HashMap;

use egui::{Id, Sense, Ui, Vec2, load::SizedTexture};

use crate::{common::create_info::DoorObjectCreateInfo, egui_renderer::ui_manager::EguiMaterial, engine::GameData};

pub struct DoorObjects {
    selected_id: i32,
    scale_uniform: bool,
    selected_mesh_index_map: HashMap<usize, usize>,
    material_window_open: bool,
    add_object_selected: bool,
    pub should_reset_other_states: bool
}

impl DoorObjects {
    pub fn new() -> Self {
        Self {
            selected_id: -1,
            scale_uniform: true,
            selected_mesh_index_map: HashMap::new(),
            material_window_open: false,
            add_object_selected: false,
            should_reset_other_states: false
        }
    }

    pub fn list(&mut self, ui: &mut Ui, game_data: &mut GameData) {
        ui.collapsing("Door Objects", |ui| {
            for (index, door_object) in game_data.scene.door_objects.iter_mut().enumerate() {
                let button = ui.button(door_object.model_name.to_string() + " (" + &index.to_string() + ")");

                    if button.clicked() {
                        self.selected_id = door_object.id as i32;
                        self.add_object_selected = false;
                    }

                    if door_object.id as i32 == self.selected_id {
                        door_object.is_selected = true;
                    } else {
                        door_object.is_selected = false;
                    }
                }

                ui.separator();
                if ui.button("New Door Object").clicked() {
                    for game_object in game_data.scene.game_objects.iter_mut() {
                        game_object.set_selected(false);
                    }
                    self.selected_id = -1;
                    self.add_object_selected = true;
                    self.should_reset_other_states = true;
                    // self.selected_light_id = -1;
                }
        });
    }

    pub fn update(&mut self, ui: &mut Ui, game_data: &mut GameData, materials: &Vec<EguiMaterial>, (window_width, window_height): (u32, u32)) {
        for door_object in game_data.scene.door_objects.iter_mut() {
            if door_object.is_selected {
                ui.label("Position X");
                ui.add(egui::DragValue::new(&mut door_object.transform.position.x));

                ui.label("Position Y");
                ui.add(egui::DragValue::new(&mut door_object.transform.position.y));

                ui.label("Position Z");
                ui.add(egui::DragValue::new(&mut door_object.transform.position.z));

                ui.checkbox(&mut self.scale_uniform, "Scale Uniform");

                let mut size = door_object.transform.size;

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

                        door_object.transform.size =
                            cgmath::Vector3::new(new_value, new_value, new_value);
                    } else {
                        door_object.transform.size = size;
                    }
                }

                ui.label("Rotation X");
                let mut rotation = door_object.transform.rotation;

                let slider_rot_x =
                    ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0).suffix("°"));
                ui.label("Rotation Y");
                let slider_rot_y =
                    ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0).suffix("°"));
                ui.label("Rotation Z");
                let slider_rot_z =
                    ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0).suffix("°"));

                if slider_rot_x.changed()
                    || slider_rot_y.changed()
                    || slider_rot_z.changed()
                    || slider_rot_x.changed()
                {
                    door_object.transform.rotation = rotation;
                }

                if let Some(model) = game_data.asset_manager.get_model_by_name(&door_object.model_name) {
                    if !model.meshes.is_empty() {
                        let selected_index = self
                            .selected_mesh_index_map
                            .entry(door_object.id)
                            .or_insert(0);

                        if *selected_index >= model.meshes.len() {
                            *selected_index = 0;
                        }

                        egui::ComboBox::from_label("Meshes")
                            .selected_text(&model.meshes[*selected_index].name)
                            .show_ui(ui, |ui| {
                                for (i, mesh) in model.meshes.iter().enumerate() {
                                    if ui
                                        .selectable_label(i == *selected_index, &mesh.name)
                                        .clicked()
                                    {
                                        *selected_index = i;
                                    }
                                }
                            });

                        if ui.button("Browse Materials").clicked() {
                            self.material_window_open = true;
                        }

                        if self.material_window_open {
                            egui::Window::new("Materials")
                                .id(Id::new("material_window_id"))
                                .resizable(true)
                                .min_size([(window_width / 2) as f32, (window_height / 2) as f32])
                                .open(&mut self.material_window_open)
                                .show(&ui.ctx(), |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for material in materials.iter() {
                                            let button = ui.add(
                                                egui::Image::from_texture(SizedTexture::new(
                                                    material.texture_id,
                                                    Vec2::new(100.0, 100.0),
                                                ))
                                                .sense(Sense::click()),
                                            );

                                            if button.hovered() {
                                                ui.painter().rect_stroke(
                                                    button.rect,
                                                    4.0,
                                                    egui::Stroke::new(1.5, egui::Color32::WHITE),
                                                    egui::StrokeKind::Middle,
                                                );
                                            }

                                            if button.clicked() {
                                                door_object.get_mesh_nodes_mut().set_mesh_material(
                                                    &game_data.asset_manager,
                                                    &model.meshes[*selected_index].name,
                                                    &material.material_name,
                                                );
                                            }
                                        }
                                    });
                                });
                        }
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Door State: ");
                    ui.label(door_object.state_to_string());
                });
            }
        }

        self.add_new(ui, game_data);
    }

    pub fn add_new(&mut self, ui: &mut Ui, game_data: &mut GameData) {
        if self.add_object_selected {
            let create_info = DoorObjectCreateInfo {
                    position: [1.0, 5.0, 1.0],
                    rotation: [0.0, 0.0, 0.0],
                    size: [1.0, 1.0, 1.0],
                    mesh_rendering_info: vec![],
                };

            if ui.button("Add").clicked() {
                game_data.scene.add_door_object(&create_info, &game_data.asset_manager);
            }
        }
    }

    pub fn reset_states(&mut self) {
        self.selected_id = -1;
        self.material_window_open = false;
        self.add_object_selected = false;
    }
}
