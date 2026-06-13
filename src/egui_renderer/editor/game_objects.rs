use std::collections::{HashMap, HashSet};

use egui::{Id, Sense, Ui, Vec2, load::SizedTexture};
use yhwh_core::common::{create_info::GameObjectCreateInfo, enums::MeshRenderingMode};

use crate::{asset_manager::AssetManager, egui_renderer::{editor::common::ChunkEditorState, ui_manager::EguiMaterial}, game::game_data::GameData, world::chunk::Chunk};

pub struct GameObjects {
    //selected_id: i32,
    // selected_mesh_index_map: HashMap<usize, usize>,
    // objects_marked_for_removal: HashSet<usize>,
    // add_game_object_selected: bool,
    // selected_model_index: usize,
    chunk_states: HashMap<String, ChunkEditorState>,
    scale_uniform: bool,
    material_window_open: bool,
    pub should_reset_other_states: bool
}

impl GameObjects {
    pub fn new() -> Self {
        Self {
            // add_game_object_selected: false,
            // add_game_object_chunk: None,
            // selected_id: -1,
            // selected_mesh_index_map: HashMap::new(),
            // objects_marked_for_removal: HashSet::new(),
            scale_uniform: true,
            material_window_open: false,
            // selected_model_index: 0,
            should_reset_other_states: false,
            chunk_states: HashMap::new()
        }
    }

    pub fn update_chunk_state(&mut self, chunk_name: &str) {
        if !self.chunk_states.contains_key(chunk_name) {
            self.chunk_states.insert(chunk_name.to_string(), ChunkEditorState { 
                selected_id: -1,
                selected_model_index: 0,
                add_game_object_selected: false,
                selected_mesh_index_map: HashMap::new(),
                objects_marked_for_removal: HashSet::new(),
            });
        }
    }

    pub fn apply_selection(&mut self, chunk: &mut Chunk) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        for game_object in chunk.game_objects.iter_mut() {
            if game_object.id as i32 == state.selected_id {
                game_object.set_selected(true);
            } else {
                game_object.set_selected(false);
            }
        }
    }

    pub fn list(&mut self, ui: &mut Ui, chunk: &mut Chunk) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        ui.collapsing("Game Objects", |ui| {
                for (index, game_object) in chunk.game_objects.iter_mut().enumerate() {
                    let button = ui.button(game_object.model_name().to_string() + " (" + &index.to_string() + ")");

                    if button.clicked() {
                        state.selected_id = game_object.id as i32;
                        state.add_game_object_selected = false;
                        self.should_reset_other_states = true;
                    }
                }

                ui.separator();
                if ui.button("New Game Object").clicked() {
                    for game_object in chunk.game_objects.iter_mut() {
                        game_object.set_selected(false);
                    }
                    state.selected_id = -1;
                    state.add_game_object_selected = true;
                    self.should_reset_other_states = true;
                    // self.selected_light_id = -1;
                }
            });
    }

    pub fn update(&mut self, ui: &mut Ui, chunk: &mut Chunk, asset_manager: &AssetManager, materials: &Vec<EguiMaterial>, (window_width, window_height): (u32, u32)) {
            let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
                ChunkEditorState::new()
            });

            if state.selected_id != -1 {
                for game_object in chunk.game_objects.iter_mut() {
                    if game_object.is_selected {
                        ui.label("Position X");
                        ui.add(egui::DragValue::new(&mut game_object.transform.position.x));

                        ui.label("Position Y");
                        ui.add(egui::DragValue::new(&mut game_object.transform.position.y));

                        ui.label("Position Z");
                        ui.add(egui::DragValue::new(&mut game_object.transform.position.z));

                        ui.checkbox(&mut self.scale_uniform, "Scale Uniform");

                        let mut size = game_object.transform.size;

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

                                game_object.transform.size = cgmath::Vector3::new(new_value, new_value, new_value);
                            } else {
                                game_object.transform.size = size;
                            }
                        }
                        ui.label("Rotation X");
                        let mut rotation = game_object.transform.rotation;
                        let slider_rot_x = ui.add(egui::Slider::new(&mut rotation.x, 0.0..=360.0).suffix("°"));
                        ui.label("Rotation Y");
                        let slider_rot_y = ui.add(egui::Slider::new(&mut rotation.y, 0.0..=360.0).suffix("°"));
                        ui.label("Rotation Z");
                        let slider_rot_z = ui.add(egui::Slider::new(&mut rotation.z, 0.0..=360.0).suffix("°"));

                        if slider_rot_x.changed()
                            || slider_rot_y.changed()
                            || slider_rot_z.changed()
                            || slider_rot_x.changed() {
                            game_object.transform.rotation = rotation;
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
                        if let Some(model) = asset_manager.model_by_name(game_object.model_name()) {
                            if !model.meshes.is_empty() {
                                let selected_index =  state.selected_mesh_index_map.entry(game_object.id).or_insert(0);

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
                                                // game_object.mesh_nodes_mut().set_mesh_material(
                                                //     &asset_manager,
                                                //     &model.name,
                                                //     &model.meshes[*selected_index].name,
                                                //     &material.material_name,
                                                // );
                                                 game_object.mesh_nodes_mut().set_mesh_material_by_index(
                                                    &asset_manager,
                                                    model.meshes[*selected_index].global_index,
                                                    &material.material_name,
                                                );
                                             }
                                            }
                                            
                                        // });
                                        });
                                    });


                                }

                                ui.label("Shadows");
                                ui.checkbox(&mut game_object.shadows, "");

                                ui.label("Rendering Mode");
                                if let Some(node) = game_object.mesh_nodes_mut().node_mut(model.meshes[*selected_index].global_index) {
                                    egui::ComboBox::from_label("")
                                        .selected_text(node.rendering_mode.to_string())
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_label(node.rendering_mode == MeshRenderingMode::Pbr, "Pbr").clicked() {
                                                node.rendering_mode = MeshRenderingMode::Pbr;
                                            }
                                            if ui.selectable_label(node.rendering_mode == MeshRenderingMode::Emissive, "Emissive").clicked() {
                                                node.rendering_mode = MeshRenderingMode::Emissive;
                                            }
                                            if ui.selectable_label(node.rendering_mode == MeshRenderingMode::Glass, "Glass").clicked() {
                                                node.rendering_mode = MeshRenderingMode::Glass;
                                            }
                                            if ui.selectable_label(node.rendering_mode == MeshRenderingMode::Flame, "Flame").clicked() {
                                                node.rendering_mode = MeshRenderingMode::Flame;
                                            }
                                    });
                                }
                            
                            //     match game_object.mesh_nodes_mut().get_mesh_node_by_mesh_name_mut(&model.meshes[*selected_index].name) {
                            //         Some(mesh_node) => {
                            //             ui.label("Rendering Mode");
                            //             egui::ComboBox::from_label("")
                            //                 .selected_text(mesh_node.rendering_mode.to_string())
                            //                 .show_ui(ui, |ui| {
                            //                     if ui.selectable_label(mesh_node.rendering_mode == MeshRenderingMode::Pbr, "Pbr").clicked() {
                            //                         mesh_node.rendering_mode = MeshRenderingMode::Pbr;
                            //                     }
                            //                     if ui.selectable_label(mesh_node.rendering_mode == MeshRenderingMode::Emissive, "Emissive").clicked() {
                            //                         mesh_node.rendering_mode = MeshRenderingMode::Emissive;
                            //                     }
                            //                     if ui.selectable_label(mesh_node.rendering_mode == MeshRenderingMode::Glass, "Glass").clicked() {
                            //                         mesh_node.rendering_mode = MeshRenderingMode::Glass;
                            //                     }
                            //                     if ui.selectable_label(mesh_node.rendering_mode == MeshRenderingMode::Flame, "Flame").clicked() {
                            //                          mesh_node.rendering_mode = MeshRenderingMode::Flame;
                            //                     }
                            //                 });
                            //         },
                            //         _ => {}
                            //     }

                            } else {
                                egui::ComboBox::from_label("Meshes")
                                    .selected_text("No Meshes")
                                    .show_ui(ui, |_| {});
                            }
                        }

                        ui.separator();
                        if ui.button("Delete").clicked() {
                           state.objects_marked_for_removal.insert(game_object.id);
                        }
                    }
                }
            }

            self.add_new(ui, chunk, asset_manager);
            self.process_marked_for_removal(chunk);
    }

    pub fn process_marked_for_removal(&mut self, chunk: &mut Chunk) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        for id in state.objects_marked_for_removal.drain() {
            chunk.remove_game_object_by_id(id);
        }
    }

    pub fn add_new(&mut self, ui: &mut Ui, chunk: &mut Chunk, asset_manager: &AssetManager) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        if state.add_game_object_selected {    
            let models = asset_manager.models();
                    egui::ComboBox::from_label("Select Model")
                        .selected_text(&models[state.selected_model_index].name)
                        .show_ui(ui, |ui| {
                            for (index, model) in models.iter().enumerate() {
                                ui.selectable_value(
                                    &mut state.selected_model_index,
                                    index,
                                    &model.name,
                                );
                            }
                        });

            let create_info = GameObjectCreateInfo {
                        model_name: models[state.selected_model_index].name.to_string(),
                        position: [1.0, 5.0, 1.0],
                        rotation: [0.0, 0.0, 0.0],
                        size: [1.0, 1.0, 1.0],
                        tex_scale: [1.0, 1.0],
                        shadows: false,
                        mesh_rendering_info: vec![],
                    };

                    if ui.button("Add").clicked() {
                        chunk.add_game_object(&create_info, &asset_manager);

                        state.selected_id = -1;
                        state.selected_model_index = 0;
                        state.add_game_object_selected = false;
                    }
        }
    }

    pub fn set_selected_id(&mut self, id: i32, chunk: &Chunk) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        state.selected_id = id;
        self.should_reset_other_states = true;
    }

    pub fn reset_states(&mut self, chunk: &mut Chunk) {
        let state = self.chunk_states.entry(chunk.name.clone()).or_insert_with(|| {
            ChunkEditorState::new()
        });

        state.selected_id = -1;
        state.selected_model_index = 0;
        state.add_game_object_selected = false;
        self.material_window_open = false;
        self.scale_uniform = true;
    }

    pub fn reset_other_chunks_states(&mut self, chunk: &mut Chunk) {
        for data in self.chunk_states.iter_mut() {
            let chunk_name = data.0;
            let state = data.1;

            if chunk_name != &chunk.name {
                state.selected_id = -1;
                state.selected_model_index = 0;
                state.add_game_object_selected = false;
            }
        }
    }
}