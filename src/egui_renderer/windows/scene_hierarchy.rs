use std::collections::HashMap;

use crate::{asset_manager::AssetManager, engine::GameData, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject}, utils::json::{self, save_level}};

pub struct SceneHierarchyWindow {
    selected_game_object_id: isize,
    selected_mesh_index: HashMap<usize, usize>,
    material_previews: HashMap<String, egui::TextureId>,
}

impl SceneHierarchyWindow {
    pub fn new() -> Self {
        Self {
            selected_game_object_id: -1,
            selected_mesh_index: HashMap::new(),
            material_previews: HashMap::new(),
        }
    }

    pub fn draw(&mut self, ui: &egui::Context, game_data: &mut GameData) {
        egui::Window::new("Transforms")
             .resizable(true)
             .vscroll(true)
             .default_open(true)
             .show(&ui, |ui| {
                for game_object in game_data.game_objects.iter_mut() {
                    if game_object.is_selected {
                        ui.label("Position X");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().x));

                        ui.label("Position Y");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().y));

                        ui.label("Position Z");
                        ui.add(egui::DragValue::new(&mut game_object.get_position_mut().z));

                        ui.label("Size");
                        let mut size = game_object.get_size().x;
                        let slider = ui.add(egui::Slider::new(&mut size, 0.0..=100.0));
                        if slider.changed() {
                            game_object.set_size(cgmath::Vector3::new(size, size, size));
                        }

                        ui.label("Rotation X");
                        let mut rotation = game_object.get_rotation();
                        let slider_rot_x = ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0).suffix("°"));
                        let slider_rot_y = ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0).suffix("°"));
                        let slider_rot_z = ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0).suffix("°"));

                        if slider_rot_x.changed() || slider_rot_y.changed() || slider_rot_z.changed() {
                            game_object.set_rotation(rotation);
                        }

                        ui.label("Texture Scale");
                        //let mut texture_scale = game_object.tex_scale;
                        ui.add(egui::Slider::new(&mut game_object.tex_scale.x, 1.0..=10.0).suffix(" X"));
                        ui.add(egui::Slider::new(&mut game_object.tex_scale.y, 1.0..=10.0).suffix(" Y"));
                    }
                }
             });
        egui::Window::new("Settings")
            .resizable(true)
            .vscroll(true)
            .default_open(true)
            .show(&ui, |ui| {
                ui.collapsing("Game Objects", |ui| {
                    for game_object in game_data.game_objects.iter_mut() {
                        let button = ui.button(game_object.get_name());

                        if button.clicked() {
                            self.selected_game_object_id = game_object.object_id as isize;
                        }

                        if game_object.object_id as isize == self.selected_game_object_id {
                            game_object.set_selected(true);
                        } else {
                            game_object.set_selected(false);
                        }
                    }
                });

                ui.collapsing("Animated Game Objects", |ui| {
                    for animated_game_object in game_data.animated_game_objects.iter() {
                        ui.label(animated_game_object.get_name());

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
                                            if ui.selectable_label(i == *selected_index, &mesh.name).clicked() {
                                                *selected_index = i;
                                            }
                                        }
                                    });

                                // let materials = asset_manager.get_all_materials();
                                // for material in materials.iter() {
                                //     let material_name: &str = &material.name;
                                //     if let Some(texture) = asset_manager.get_texture_by_name(&format!("{material_name}_ALB.png")) {
                                //         let tex_id = self
                                //             .material_previews
                                //             .entry(material_name.to_string())
                                //             .or_insert_with(|| {
                                //                 let color_image =
                                //                     egui::ColorImage::from_rgba_unmultiplied(
                                //                         [
                                //                             texture.dimensions.0 as usize,
                                //                             texture.dimensions.1 as usize,
                                //                         ],
                                //                         &texture.pixel_data,
                                //                     );
                                //                 ui.ctx()
                                //                     .load_texture(
                                //                         &format!("preview_{material_name}"),
                                //                         color_image,
                                //                         egui::TextureOptions::default(),
                                //                     )
                                //                     .id()
                                //             });

                                //         ui.image((*tex_id, egui::vec2(128.0, 128.0)));
                                //     }
                                // }
                            } else {
                                egui::ComboBox::from_label("Meshes")
                                    .selected_text("No Meshes")
                                    .show_ui(ui, |_| {});
                            }
                        }
                    }
                });

                ui.collapsing("File", |ui| {
                    if ui.button("Save Level").clicked() {
                        save_level(game_data);
                    }
                })
            });
    }
}
