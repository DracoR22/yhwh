use std::collections::HashMap;

use crate::{asset_manager::AssetManager, objects::animated_game_object::AnimatedGameObject};

pub struct SceneHierarchyWindow {
    selected_mesh_index: HashMap<usize, usize>,
    material_previews: HashMap<String, egui::TextureId>,
}

impl SceneHierarchyWindow {
    pub fn new() -> Self {
        Self {
            selected_mesh_index: HashMap::new(),
            material_previews: HashMap::new(),
        }
    }

    pub fn draw(
        &mut self,
        ui: &egui::Context,
        animated_game_objects: &Vec<AnimatedGameObject>,
        asset_manager: &AssetManager,
    ) {
        egui::Window::new("Settings")
            .resizable(true)
            .vscroll(true)
            .default_open(true)
            .show(&ui, |ui| {
                ui.label("Window!");

                ui.collapsing("Animated Game Objects", |ui| {
                    for animated_game_object in animated_game_objects.iter() {
                        ui.label(animated_game_object.get_name());

                        if let Some(model) = asset_manager.get_model_by_name(animated_game_object.get_model_name()) {
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

                let some_button = ui.button("TEST BUTTON");
                if some_button.clicked() {
                    println!("BUTTON CLICKED!")
                }
            });
    }
}
