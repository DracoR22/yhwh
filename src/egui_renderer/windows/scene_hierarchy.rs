use std::collections::{HashMap, HashSet};

use egui::{Align, Align2, Context, Sense, TextureId, Ui, Vec2, load::SizedTexture};

use crate::{
    common::{create_info::{GameObjectCreateInfo, LightObjectCreateInfo}, enums::LightType},
    egui_renderer::ui_manager::EguiMaterial,
    engine::GameData,
    objects::{
        animated_game_object::{self, AnimatedGameObject},
        game_object::GameObject,
    },
    utils::json::save_level,
};

pub struct SceneHierarchyWindow {
    selected_game_object_id: isize,
    selected_light_id: isize,
    selected_mesh_index: HashMap<usize, usize>,

    add_game_object_selected: bool,
    selected_model_index: usize,
    selected_material_index: usize,

    objects_marked_for_removal: HashSet<usize>,
    scale_uniform: bool
}

impl SceneHierarchyWindow {
    pub fn new() -> Self {
        Self {
            selected_game_object_id: -1,
            selected_light_id: -1,
            selected_mesh_index: HashMap::new(),
            add_game_object_selected: false,
            selected_model_index: 0,
            selected_material_index: 0,
            objects_marked_for_removal: HashSet::new(),
            scale_uniform: true
        }
    }

    pub fn draw(
        &mut self,
        ui: &egui::Context,
        materials: &Vec<EguiMaterial>,
        game_data: &mut GameData,
        (window_width, window_height): (u32, u32)
    ) {
        egui::SidePanel::right("Transforms")
           .resizable(true)
           .default_width(250.0)
           .width_range(250.0..=300.0)
            .show(&ui, |ui| {
                ui.separator();
                if self.selected_game_object_id != -1 {
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
                                let selected_index =  self.selected_mesh_index.entry(game_object.id).or_insert(0);

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

                                ui.label("Material");
                                egui::ScrollArea::vertical().max_height(10.0).show(ui, |ui| {
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
                                        game_object.get_mesh_nodes_mut().set_mesh_material_by_mesh_index(
                                                &game_data.asset_manager,
                                                &model.meshes[*selected_index].name,
                                                &material.material_name,
                                        );

                                       // self.selected_material_index = material.material_index;
                                    }
                                }
                                });
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
               
                self.process_marked_for_removal(game_data);

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
                        rotation: [1.0, 1.0, 1.0],
                        size: [1.0, 1.0, 1.0],
                        tex_scale: [1.0, 1.0],
                        mesh_rendering_info: vec![],
                    };

                    if ui.button("Add").clicked() {
                        game_data.scene.add_game_object(&create_info, &game_data.asset_manager);
                    }
                }

                if self.selected_light_id > 0 {
                    for light in game_data.scene.lights.iter_mut() {
                        if light.id as isize == self.selected_light_id {
                            ui.label("Position X");
                            ui.add(egui::DragValue::new(&mut light.position.x));

                            ui.label("Position Y");
                            ui.add(egui::DragValue::new(&mut light.position.y));

                            ui.label("Position Z");
                            ui.add(egui::DragValue::new(&mut light.position.z));

                            let mut color = [
                                light.color.x,
                                light.color.y,
                                light.color.z,
                            ];

                            ui.label("Color");
                            if ui.color_edit_button_rgb(&mut color).changed() {
                                light.color.x = color[0];
                                light.color.y = color[1];
                                light.color.z = color[2];
                            }

                            ui.label("Strength");
                            ui.add(egui::Slider::new(&mut light.strength, 0.0..=100.0));

                            ui.label("Radius");
                            ui.add(egui::Slider::new(&mut light.radius, 0.0..=100.0));
                        }
                    }
                }
            });

            egui::SidePanel::left("right_panel")
               .resizable(true)
               .default_width(250.0)
               .width_range(250.0..=300.0)
               .show(&ui, |ui| {
                    //ui.set_min_width(200.0);
                    ui.separator();
                    ui.collapsing("Game Objects", |ui| {
                    for (index, game_object) in game_data.scene.game_objects.iter_mut().enumerate() {
                        let button = ui.button(game_object.get_model_name().to_string() + " (" + &index.to_string() + ")");

                        if button.clicked() {
                            self.selected_game_object_id = game_object.id as isize;
                            self.add_game_object_selected = false;
                        }

                        if game_object.id as isize == self.selected_game_object_id {
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
                        self.selected_game_object_id = -1;
                        self.add_game_object_selected = true;
                        self.selected_light_id = -1;
                    }
                });

                ui.collapsing("Animated Game Objects", |ui| {
                    for (index, animated_game_object) in game_data.scene.animated_game_objects.iter().enumerate() {
                        ui.label(
                            animated_game_object.get_model_name().to_string() + &index.to_string(),
                        );

                        if let Some(model) = game_data.asset_manager.get_model_by_name(animated_game_object.get_model_name()) {
                            if !model.meshes.is_empty() {
                                let selected_index = self
                                    .selected_mesh_index
                                    .entry(animated_game_object.object_id)
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

                                for material in materials.iter() {
                                    ui.add(egui::Image::from_texture(SizedTexture::new(
                                        material.texture_id,
                                        Vec2::new(100.0, 100.0),
                                    )));
                                }
                            } else {
                                egui::ComboBox::from_label("Meshes")
                                    .selected_text("No Meshes")
                                    .show_ui(ui, |_| {});
                            }
                        }
                    }
                });

                ui.collapsing("Lights", |ui| {
                    for (index, light) in game_data.scene.lights.iter().enumerate() {
                        let button = ui.button("Light (".to_string() + &index.to_string() + ")");

                        if button.clicked() {
                            self.selected_light_id = light.id as isize;
                            self.add_game_object_selected = false;
                            self.selected_game_object_id = -1;
                        }
                    }

                    ui.separator();

                    if ui.button("Add Light").clicked() {
                        let create_info = LightObjectCreateInfo {
                                    color: [1.0, 1.0, 1.0],
                                    position: [2.0, 2.0, 2.0],
                                    radius: 10.0,
                                    strength: 50.0,
                                    light_type: LightType::Point
                                };

                        game_data.scene.add_light(&create_info);
                    }
                });

                ui.collapsing("File", |ui| {
                    if ui.button("Save Level").clicked() {
                        save_level(game_data);
                    }
                })
            });
    }

    pub fn process_marked_for_removal(&mut self, game_data: &mut GameData) {
        for id in self.objects_marked_for_removal.drain() {
            game_data.scene.remove_game_object_by_id(id);
        }
    }

    fn draw_meshes(
        &mut self,
        game_object: &mut GameObject,
        game_data: &GameData,
        materials: &Vec<EguiMaterial>,
        ui: &mut Ui,
    ) {
        if let Some(model) = game_data
            .asset_manager
            .get_model_by_name(game_object.get_model_name())
        {
            if !model.meshes.is_empty() {
                let selected_index = self.selected_mesh_index.entry(game_object.id).or_insert(0);

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

                ui.label("Material");
                for material in materials.iter() {
                    let button = ui.add(egui::Image::from_texture(SizedTexture::new(
                        material.texture_id,
                        Vec2::new(100.0, 100.0),
                    )));

                    if button.hovered() {
                        ui.painter().rect_stroke(
                            button.rect,
                            4.0,
                            egui::Stroke::new(1.5, egui::Color32::WHITE),
                            egui::StrokeKind::Middle,
                        );
                    }

                    if button.clicked() {
                        game_object
                            .get_mesh_nodes_mut()
                            .set_mesh_material_by_mesh_index(
                                &game_data.asset_manager,
                                &model.meshes[*selected_index].name,
                                &material.material_name,
                            );
                    }
                }
            } else {
                egui::ComboBox::from_label("Meshes")
                    .selected_text("No Meshes")
                    .show_ui(ui, |_| {});
            }
        }
    }
}
