use crate::{common::{create_info::LightObjectCreateInfo, enums::LightType}, utils::unique_id};

pub struct LightObject {
    pub color: cgmath::Vector3<f32>,
    pub position: cgmath::Vector3<f32>,
    pub strength: f32,
    pub radius: f32,
    pub light_type: LightType,
    pub id: usize
}

impl LightObject {
    pub fn new(create_info: &LightObjectCreateInfo) -> Self {
        Self {
            color: cgmath::Vector3::new(create_info.color[0], create_info.color[1], create_info.color[2]),
            position: cgmath::Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            radius: create_info.radius,
            strength: create_info.strength,
            light_type: create_info.light_type.clone(),
            id: unique_id::next_id()
        }
    }

     pub fn get_create_info(&self) -> LightObjectCreateInfo {
        let create_info = LightObjectCreateInfo { 
            position: [self.position.x, self.position.y, self.position.z],
            color: [self.color.x, self.color.y, self.color.z],
            radius: self.radius,
            strength: self.strength,
            light_type: self.light_type.clone()
        };

        create_info
    }
}

