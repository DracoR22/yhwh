use std::collections::{HashMap, HashSet};

use egui::{Id, Sense, Ui, Vec2, load::SizedTexture};

use crate::{common::create_info::GameObjectCreateInfo, egui_renderer::ui_manager::EguiMaterial, game::game_data::GameData};

pub struct GameObjects {
    selected_id: i32,
    selected_mesh_index_map: HashMap<usize, usize>,
    objects_marked_for_removal: HashSet<usize>,
    add_game_object_selected: bool,
    scale_uniform: bool,
    material_window_open: bool,
    selected_model_index: usize,
    pub should_reset_other_states: bool
}

impl GameObjects {
    pub fn new() -> Self {
        Self {
            add_game_object_selected: false,
            selected_id: -1,
            selected_mesh_index_map: HashMap::new(),
            objects_marked_for_removal: HashSet::new(),
            scale_uniform: true,
            material_window_open: false,
            selected_model_index: 0,
            should_reset_other_states: false
        }
    }

    pub fn list(&mut self, ui: &mut Ui, game_data: &mut GameData) {
        ui.separator();
        ui.collapsing("Game Objects", |ui| {
                for (index, game_object) in game_data.scene.game_objects.iter_mut().enumerate() {
                    let button = ui.button(game_object.get_model_name().to_string() + " (" + &index.to_string() + ")");

                    if button.clicked() {
                        self.selected_id = game_object.id as i32;
                        self.add_game_object_selected = false;
                    }

                    if game_object.id as i32 == self.selected_id {
                        game_object.set_selected(true);
                    } else {
                        game_object.set_selected(false);
                    }
                }

                ui.separator();
                if ui.button("New Game Object").clicked() {
                    for game_object in game_data.scene.game_objects.iter_mut() {
                        game_object.set_selected(false);
                    }
                    self.selected_id = -1;
                    self.add_game_object_selected = true;
                    self.should_reset_other_states = true;
                    // self.selected_light_id = -1;
                }
            });
    }

    pub fn update(&mut self, ui: &mut Ui, game_data: &mut GameData, materials: &Vec<EguiMaterial>, (window_width, window_height): (u32, u32)) {
         ui.separator();
            if self.selected_id != -1 {
                for game_object in game_data.scene.game_objects.iter_mut() {
                    if game_object.is_selected {
                        ui.label("Position X");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().x));

                        ui.label("Position Y");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().y));

                        ui.label("Position Z");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().z));

                        ui.checkbox(&mut self.scale_uniform, "Scale Uniform");

                        let mut size = game_object.get_size();

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

                                game_object.set_size(cgmath::Vector3::new(new_value, new_value, new_value));
                            } else {
                                game_object.set_size(size);
                            }
                        }
                        ui.label("Rotation X");
                        let mut rotation = game_object.get_rotation();
                        let slider_rot_x = ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0).suffix("°"));
                        ui.label("Rotation Y");
                        let slider_rot_y = ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0).suffix("°"));
                        ui.label("Rotation Z");
                        let slider_rot_z = ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0).suffix("°"));

                        if slider_rot_x.changed()
                            || slider_rot_y.changed()
                            || slider_rot_z.changed()
                            || slider_rot_x.changed() {
                            game_object.set_rotation(rotation);
                        }

                        ui.label("Texture Scale");
                        ui.add(
                            egui::Slider::new(&mut game_object.tex_scale.x, 1.0..=10.0)
                                .suffix(" X"),
                        );
                        ui.add(
                            egui::Slider::new(&mut game_object.tex_scale.y, 1.0..=10.0)
                                .suffix(" Y"),
                        );

                        //self.draw_meshes(game_object, game_data, &materials, &mut ui);
                        if let Some(model) = game_data.asset_manager.get_model_by_name(game_object.get_model_name()) {
                            if !model.meshes.is_empty() {
                                let selected_index =  self.selected_mesh_index_map.entry(game_object.id).or_insert(0);

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
                                                let button = ui.add(egui::Image::from_texture(SizedTexture::new(
                                                    material.texture_id,
                                                    Vec2::new(100.0, 100.0),
                                            )).sense(Sense::click()));

                                            if button.hovered() {
                                                ui.painter().rect_stroke(
                                                    button.rect,
                                                    4.0,
                                                    egui::Stroke::new(1.5, egui::Color32::WHITE),
                                                    egui::StrokeKind::Middle,
                                                );
                                            }

                                            if button.clicked() {
                                                game_object.get_mesh_nodes_mut().set_mesh_material(
                                                &game_data.asset_manager,
                                                &model.meshes[*selected_index].name,
                                                &material.material_name,
                                                );
                                             }
                                            }
                                            
                                        // });
                                        });
                                    });


                                }

                                ui.label("Emissive");
                                match game_object.get_mesh_nodes_mut().get_mesh_node_by_mesh_name_mut(&model.meshes[*selected_index].name) {
                                    Some(mesh_node) => {
                                        ui.checkbox(&mut mesh_node.emissive, "");
                                    }
                                    _ => {}
                                }

                                ui.label("Shadows");
                                ui.checkbox(&mut game_object.shadows, "");

                                ui.label("Glass");
                                match game_object.get_mesh_nodes_mut().get_mesh_node_by_mesh_name_mut(&model.meshes[*selected_index].name) {
                                    Some(mesh_node) => {
                                        ui.checkbox(&mut mesh_node.glass, "");
                                    },
                                    _ => {}
                                }
                            } else {
                                egui::ComboBox::from_label("Meshes")
                                    .selected_text("No Meshes")
                                    .show_ui(ui, |_| {});
                            }
                        }

                        ui.separator();
                        if ui.button("Delete").clicked() {
                           self.objects_marked_for_removal.insert(game_object.id);
                        }
                    }
                }
            }

            self.add_new(ui, game_data);
            self.process_marked_for_removal(game_data);
    }

    pub fn process_marked_for_removal(&mut self, game_data: &mut GameData) {
        for id in self.objects_marked_for_removal.drain() {
            game_data.scene.remove_game_object_by_id(id);
        }
    }

    pub fn add_new(&mut self, ui: &mut Ui, game_data: &mut GameData) {
        if self.add_game_object_selected {
            let models = game_data.asset_manager.get_models();
                    egui::ComboBox::from_label("Select Model")
                        .selected_text(&models[self.selected_model_index].name)
                        .show_ui(ui, |ui| {
                            for (index, model) in models.iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.selected_model_index,
                                    index,
                                    &model.name,
                                );
                            }
                        });

            let create_info = GameObjectCreateInfo {
                        model_name: models[self.selected_model_index].name.to_string(),
                        position: [1.0, 5.0, 1.0],
                        rotation: [0.0, 0.0, 0.0],
                        size: [1.0, 1.0, 1.0],
                        tex_scale: [1.0, 1.0],
                        shadows: false,
                        mesh_rendering_info: vec![],
                    };

                    if ui.button("Add").clicked() {
                        game_data.scene.add_game_object(&create_info, &game_data.asset_manager);
                }
        }
    }

    pub fn reset_states(&mut self) {
        self.selected_id = -1;
        self.add_game_object_selected = false;
        self.material_window_open = false;
        self.scale_uniform = false;
    }
}