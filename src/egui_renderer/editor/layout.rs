use std::fs;

use cgmath::{Vector2, Vector3, Vector4, Zero};
use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use egui::{ComboBox, Id, Modal, Ui};

use crate::asset_manager::AssetManager;
use crate::common::create_info::SceneCreateInfo;
use crate::egui_renderer::editor::animated_game_objects::AnimatedGameObjects;
use crate::input::input::Input;
use crate::input::yhwh_keys::YHWHMouseButton;
use crate::utils::ray_cast::ray_intersects_aabb;
use crate::{
    egui_renderer::{editor::{door_objects::DoorObjects, game_objects::GameObjects, lights::Lights}, ui_manager::EguiMaterial}, game::game_data::GameData
};
enum RayCastHit {
    GameObject(i32),
    DoorObject(i32)
}

pub struct EditorLayout {
    game_objects_panel: GameObjects,
    animated_game_objects: AnimatedGameObjects,
    door_objects_panel: DoorObjects,
    lights_panel: Lights,
    ray_direction: Vector3<f32>,
    ray_origin: Vector3<f32>,

    save_map_modal: bool,
    map_list: Vec<String>,
    selected_map: String,

    add_chunk_modal: bool,
    new_chunk_name: String
}

impl EditorLayout {
    pub fn new() -> Self {
        Self {
            game_objects_panel: GameObjects::new(),
            animated_game_objects: AnimatedGameObjects::new(),
            door_objects_panel: DoorObjects::new(),
            lights_panel: Lights::new(),
            ray_direction: Vector3::zero(),
            ray_origin: Vector3::zero(),
            save_map_modal: false,
            map_list: Vec::new(),
            selected_map: "".to_string(),
            add_chunk_modal: false,
            new_chunk_name: "untitled".to_string()
        }
    }

    pub fn draw(
        &mut self,
        ui: &egui::Context,
        materials: &Vec<EguiMaterial>,
        game_data: &mut GameData,
        input: &Input,
        (window_width, window_height): (u32, u32)
    ) { 
        // game_data.world.for_each_chunk(|chunk| {
        //     self.game_objects_panel.update_chunk_state(&chunk.name);
        // });   

        egui::TopBottomPanel::top("top bar").show(ui, |ui| {
            self.save_map_modal(ui, game_data);
            self.add_chunk_modal(ui, game_data);
            ui.scope(|ui| {
                egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.set_min_width(180.0);

                    if ui.button("Save Map").clicked() {
                        self.save_map_modal = true;
                    }

                    if ui.button("Save Chunk").clicked() {
                        
                    }

                    if ui.button("Add Chunk to Active Map").clicked() {
                        self.add_chunk_modal = true;
                    }
                });
            });
            }); 
        });
        egui::SidePanel::right("Right Panel")
           .resizable(true)
           .default_width(250.0)
           .width_range(250.0..=300.0)
           .show(&ui, |ui| {
                game_data.world.for_each_chunk_mut(|chunk| {
                    self.game_objects_panel.update(ui, chunk, &game_data.asset_manager, materials, (window_width, window_height));
                    self.door_objects_panel.update(ui, chunk, &game_data.asset_manager, materials, (window_width, window_height));
                    self.animated_game_objects.update(ui, chunk, materials, (window_width, window_height));
                    self.lights_panel.update(ui, chunk);

                    // reset states
                    if self.door_objects_panel.should_reset_other_states {
                        self.game_objects_panel.reset_other_chunks_states(chunk);
                        self.game_objects_panel.reset_states(chunk);
                        self.door_objects_panel.should_reset_other_states = false;
                    }

                    if self.game_objects_panel.should_reset_other_states {
                        self.door_objects_panel.reset_states();
                        self.game_objects_panel.should_reset_other_states = false;
                    }
                });
            });

        egui::SidePanel::left("left panel")
               .resizable(true)
               .default_width(250.0)
               .width_range(250.0..=300.0)
               .show(&ui, |ui| {

                ui.collapsing("World", |ui| {
                    game_data.world.for_each_chunk_mut(|chunk| {
                        ui.collapsing(chunk.file_name.clone(), |ui| {
                            self.game_objects_panel.list(ui, chunk);
                            self.door_objects_panel.list(ui, chunk);
                            self.animated_game_objects.list(ui, chunk, materials);
                            self.lights_panel.list(ui, chunk);
                        });
                    });
                });    

                // ui.collapsing("File", |ui| {
                //     if ui.button("Save Level").clicked() {
                //         save_level(game_data);
                //     }
                // })
            });

            game_data.world.for_each_chunk_mut(|chunk| {
                self.game_objects_panel.apply_selection(chunk);
                self.door_objects_panel.apply_selection(chunk);
                 self.animated_game_objects.apply_selection(chunk);
            });

            // mouse picking stuff
            if !ui.wants_pointer_input() && !ui.is_pointer_over_area() && input.mouse.button_just_pressed(&YHWHMouseButton::Left) {
                let mut closest_distance = f32::INFINITY;
                let mut closest_hit: Option<RayCastHit> = None;

                game_data.world.for_each_chunk_mut(|chunk| {
                    for game_object in chunk.game_objects.iter_mut() {
                        if let Some(model) = game_data.asset_manager.model_by_name(game_object.model_name()) {
                            if let Some(aabb) = model.aabb {
                                let model_matrix = game_object.model_matrix();
                                let world_aabb = aabb.transform(model_matrix);

                                if let Some(distance) = ray_intersects_aabb(self.ray_origin, self.ray_direction, &world_aabb) {
                                    if distance < closest_distance {
                                        closest_distance = distance;
                                        closest_hit = Some(RayCastHit::GameObject(game_object.id as i32));
                                    }
                                }
                            }
                        }
                    }

                    for door_object in chunk.door_objects.iter_mut() {
                        if let Some(model) = game_data.asset_manager.model_by_name(&door_object.model_name) {
                            if let Some(aabb) = model.aabb {
                                let model_matrix = door_object.model_matrix();
                                let world_aabb = aabb.transform(model_matrix);

                                if let Some(distance) = ray_intersects_aabb(self.ray_origin, self.ray_direction, &world_aabb) {
                                    if distance < closest_distance {
                                        closest_distance = distance;
                                        closest_hit = Some(RayCastHit::DoorObject(door_object.id as i32));
                                    }
                                }
                            }
                        }
                    }
                });

               game_data.world.for_each_chunk(|chunk| {
                 match closest_hit {
                    Some(RayCastHit::GameObject(id)) => {
                        self.game_objects_panel.set_selected_id(id, chunk);
                    }
                    Some(RayCastHit::DoorObject(id)) => {
                        self.door_objects_panel.set_selected_id(id);
                    }
                    None => {}
                }
               });

                //  if let Some(id) = closest_game_object_id {
                //     self.game_objects_panel.set_selected_id(id);
                //  }

                //  if let Some(id) = closest_door_object_id {
                //     self.door_objects_panel.set_selected_id(id);
                //  }
            }

            self.update_mouse_rays((window_width as f32, window_height as f32), game_data, input);
    }

    fn save_map_modal(&mut self, ui: &mut Ui, game_data: &GameData) {
        if self.save_map_modal {
            let modal = Modal::new(Id::new("Save Map")).show(ui.ctx(), |ui| {
                let maps_path = "res/maps";

                for entry in fs::read_dir(maps_path).expect("res/maps is missing!!") {
                    self.map_list.clear();

                    let entry = entry.unwrap();
                    let path = entry.path();

                    if path.is_file() && path.file_name().is_some() {
                        self.map_list.push(path.file_name().unwrap().to_str().unwrap().to_string());
                        if self.selected_map == "" {
                            self.selected_map = path.file_name().unwrap().to_str().unwrap().to_string();
                        }
                    }
                }

                egui::ComboBox::from_label("")
                    .selected_text(&self.selected_map)
                    .show_ui(ui, |ui| {
                        for map_name in self.map_list.iter() {
                            if ui.selectable_label(self.selected_map == map_name.to_string(), map_name).clicked() {
                                self.selected_map = map_name.to_string();
                            }
                        }

                        // if no maps are loaded
                        if ui.selectable_label(self.selected_map == "untitled_map", "untitled_map").clicked() {
                            self.selected_map = "untitled_map".to_string();
                         }
                    });

                 ui.add(egui::TextEdit::singleline(&mut self.selected_map).hint_text(""));

                 if ui.button("Save").clicked() {
                    game_data.world.save_map(&self.selected_map, &game_data.asset_manager);
                    self.save_map_modal = false;
                 }
            });

            if modal.should_close() {
                self.save_map_modal = false;
            }
        }
    }

    fn add_chunk_modal(&mut self, ui: &mut Ui, game_data: &mut GameData) {
        if self.add_chunk_modal {
            let modal = Modal::new(Id::new("Add Chunk")).show(ui.ctx(), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_chunk_name).hint_text(""));

                if ui.button("Add").clicked() {
                    let create_info = SceneCreateInfo {
                        name: self.new_chunk_name.clone(),
                        door_objects: Vec::new(),
                        game_objects: Vec::new(),
                        lights: Vec::new()
                    };
                    game_data.world.add_chunk(&create_info, &game_data.asset_manager);

                    self.add_chunk_modal = false;
                }
            });

            if modal.should_close() {
                self.add_chunk_modal = false;
            }
        }
    }
}

impl EditorLayout {
    pub fn update_mouse_rays(&mut self, (w_width, w_height): (f32, f32), game_data: &GameData, input: &Input) {
        let camera = &game_data.camera;
        let viewport_size = Vector2::<f32>::new(w_width, w_height);

        let mouse_x = input.mouse.position.0 as f32;
        let mouse_y = input.mouse.position.1 as f32;

        let x = (2.0 * mouse_x) / viewport_size.x - 1.0;
        let y = 1.0 - (2.0 * mouse_y) / viewport_size.y;
        let z = 1.0;
        let ray_nds = Vector3::new(x, y, z);

        let projection_matrix = camera.get_projection().calc_matrix();
        let view_matrix = camera.calc_matrix();

        let ray_clip = Vector4::new(ray_nds.x, ray_nds.y, -1.0, 1.0);
        if let Some((inverse_projection, inverse_view)) = projection_matrix.invert().zip(view_matrix.invert()) {
            let mut ray_eye = inverse_projection * ray_clip;
            ray_eye = Vector4::new(ray_eye.x, ray_eye.y, -1.0, 0.0);

    
            let mut ray_world = (inverse_view * ray_eye).truncate();
            ray_world = ray_world.normalize();

            self.ray_direction = ray_world;
            self.ray_origin = Vector3::new(inverse_view.w.x, inverse_view.w.y, inverse_view.w.z);
        }
    }
}
