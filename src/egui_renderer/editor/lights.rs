use egui::Ui;

use crate::{common::{create_info::LightObjectCreateInfo, enums::LightType}, game::game_data::GameData, scene::Scene};

pub struct Lights {
    selected_id: i32,

}

impl Lights {
    pub fn new() -> Self {
        Self {
            selected_id: -1
        }
    }
    pub fn list(&mut self, ui: &mut Ui, chunk: &mut Scene) {
        ui.collapsing("Lights", |ui| {
            for (index, light) in chunk.lights.iter().enumerate() {
                let button = ui.button("Light (".to_string() + &index.to_string() + ")");

                if button.clicked() {
                    self.selected_id = light.id as i32;
                    // self.add_game_object_selected = false;
                    // self.selected_game_object_id = -1;
                }
            }

            ui.separator();

            if ui.button("Add Light").clicked() {
                let create_info = LightObjectCreateInfo {
                    color: [1.0, 1.0, 1.0],
                    position: [2.0, 2.0, 2.0],
                    radius: 10.0,
                    strength: 50.0,
                    light_type: LightType::Point,
                    shadows: true
                };

               chunk.add_light(&create_info);
            } 
        });
    }
    pub fn update(&mut self, ui: &mut Ui, chunk: &mut Scene) {
        if self.selected_id > 0 {
            for light in chunk.lights.iter_mut() {
                if light.id as i32 == self.selected_id {
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
                    ui.add(egui::Slider::new(&mut light.strength, 1.0..=100.0));

                    ui.label("Radius");
                    ui.add(egui::Slider::new(&mut light.radius, 1.0..=100.0));

                    ui.label("Shadows");
                    ui.checkbox(&mut light.shadows, "");
                }
            }
        }
    }
}