use std::collections::{HashMap};

use egui::{Vec2, load::SizedTexture};

use crate::{
    egui_renderer::{editor::{door_objects::DoorObjects, game_objects::GameObjects, lights::Lights}, ui_manager::EguiMaterial}, engine::GameData, utils::json::save_level
};

pub struct EditorLayout {
    selected_mesh_index: HashMap<usize, usize>,
    game_objects_panel: GameObjects,
    door_objects_panel: DoorObjects,
    lights_panel: Lights,
}

impl EditorLayout {
    pub fn new() -> Self {
        Self {
            selected_mesh_index: HashMap::new(),
            game_objects_panel: GameObjects::new(),
            door_objects_panel: DoorObjects::new(),
            lights_panel: Lights::new()
        }
    }

    pub fn draw(
        &mut self,
        ui: &egui::Context,
        materials: &Vec<EguiMaterial>,
        game_data: &mut GameData,
        (window_width, window_height): (u32, u32)
    ) {
        egui::SidePanel::right("Right Panel")
           .resizable(true)
           .default_width(250.0)
           .width_range(250.0..=300.0)
            .show(&ui, |ui| {
                self.game_objects_panel.update(ui, game_data, materials, (window_width, window_height));
                self.game_objects_panel.add_new(ui, game_data);
                self.game_objects_panel.process_marked_for_removal(game_data);

                self.door_objects_panel.update(ui, game_data, materials, (window_width, window_height));
                self.lights_panel.update(ui, game_data);
            });

            egui::SidePanel::left("right_panel")
               .resizable(true)
               .default_width(250.0)
               .width_range(250.0..=300.0)
               .show(&ui, |ui| {
                self.game_objects_panel.list(ui, game_data);

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

                self.door_objects_panel.list(ui, game_data);
                self.lights_panel.list(ui, game_data);

                ui.collapsing("File", |ui| {
                    if ui.button("Save Level").clicked() {
                        save_level(game_data);
                    }
                })
            });
    }
}
