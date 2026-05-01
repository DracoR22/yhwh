use cgmath::{Vector2, Vector3, Vector4, Zero};
use cgmath::SquareMatrix;
use cgmath::InnerSpace;

use crate::egui_renderer::editor::animated_game_objects::AnimatedGameObjects;
use crate::input::input::Input;
use crate::input::yhwh_keys::YHWHMouseButton;
use crate::utils::ray_cast::ray_intersects_aabb;
use crate::{
    egui_renderer::{editor::{door_objects::DoorObjects, game_objects::GameObjects, lights::Lights}, ui_manager::EguiMaterial}, game::game_data::GameData, utils::json::save_level
};

pub struct EditorLayout {
    game_objects_panel: GameObjects,
    animated_game_objects: AnimatedGameObjects,
    door_objects_panel: DoorObjects,
    lights_panel: Lights,
    ray_direction: Vector3<f32>,
    ray_origin: Vector3<f32>
}

impl EditorLayout {
    pub fn new() -> Self {
        Self {
            game_objects_panel: GameObjects::new(),
            animated_game_objects: AnimatedGameObjects::new(),
            door_objects_panel: DoorObjects::new(),
            lights_panel: Lights::new(),
            ray_direction: Vector3::zero(),
            ray_origin: Vector3::zero()
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
        egui::SidePanel::right("Right Panel")
           .resizable(true)
           .default_width(250.0)
           .width_range(250.0..=300.0)
           .show(&ui, |ui| {
                self.game_objects_panel.update(ui, game_data, materials, (window_width, window_height));
                self.door_objects_panel.update(ui, game_data, materials, (window_width, window_height));
                self.lights_panel.update(ui, game_data);

                // reset states
                if self.door_objects_panel.should_reset_other_states {
                    self.game_objects_panel.reset_states();
                    self.door_objects_panel.should_reset_other_states = false;
                }

                if self.game_objects_panel.should_reset_other_states {
                    self.door_objects_panel.reset_states();
                    self.game_objects_panel.should_reset_other_states = false;
                }
            });

        egui::SidePanel::left("left panel")
               .resizable(true)
               .default_width(250.0)
               .width_range(250.0..=300.0)
               .show(&ui, |ui| {
                self.game_objects_panel.list(ui, game_data);
                self.animated_game_objects.list(ui, game_data, materials);
                self.door_objects_panel.list(ui, game_data);
                self.lights_panel.list(ui, game_data);

                ui.collapsing("File", |ui| {
                    if ui.button("Save Level").clicked() {
                        save_level(game_data);
                    }
                })
            });

            self.game_objects_panel.apply_selection(game_data);

            if !ui.wants_pointer_input() && !ui.is_pointer_over_area() && input.mouse.button_just_pressed(&YHWHMouseButton::Left) {
                let mut closest_distance = f32::INFINITY;
                let mut closest_id: Option<i32> = None;

                for game_object in game_data.scene.game_objects.iter_mut() {
                    if let Some(model) = game_data.asset_manager.get_model_by_name(game_object.get_model_name()) {
                        if let Some(aabb) = model.aabb {
                            let model_matrix = game_object.get_model_matrix();
                            let world_aabb = aabb.transform(model_matrix);

                            if let Some(distance) = ray_intersects_aabb(self.ray_origin, self.ray_direction, &world_aabb) {
                                if distance < closest_distance {
                                    closest_distance = distance;
                                    closest_id = Some(game_object.id as i32);
                                }
                            }
                         }
                    }
                }

                 if let Some(id) = closest_id {
                        self.game_objects_panel.set_selected_id(id);
                 }
            }

            self.update_mouse_rays((window_width as f32, window_height as f32), game_data, input);
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
