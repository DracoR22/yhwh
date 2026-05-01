use std::collections::HashMap;

use egui::{Ui, Vec2, load::SizedTexture};

use crate::{egui_renderer::ui_manager::EguiMaterial, game::game_data::GameData};

pub struct AnimatedGameObjects {
    selected_mesh_index_map: HashMap<usize, usize>,
}

impl AnimatedGameObjects {
    pub fn new() -> Self {
        Self {
            selected_mesh_index_map: HashMap::new(),
        }
    }

    pub fn list(&mut self, ui: &mut Ui, game_data: &mut GameData, materials: &Vec<EguiMaterial>) {
          ui.collapsing("Animated Game Objects", |ui| {
                for (index, animated_game_object) in game_data.scene.animated_game_objects.iter().enumerate() {
                        ui.label(
                            animated_game_object.get_model_name().to_string() + &index.to_string(),
                        );

                        if let Some(model) = game_data.asset_manager.get_model_by_name(animated_game_object.get_model_name()) {
                            if !model.meshes.is_empty() {
                                let selected_index = self
                                    .selected_mesh_index_map
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
    }
}