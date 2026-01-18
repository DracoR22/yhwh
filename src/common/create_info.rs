use serde::{Deserialize, Serialize};

use crate::common::enums::LightType;

#[derive(Serialize, Deserialize, Debug)]
pub struct MeshNodeCreateInfo {
    pub mesh_name: String,
    pub material_name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameObjectCreateInfo {
    pub model_name: String,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
    pub tex_scale: [f32; 2],
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightObjectCreateInfo {
    pub color: [f32; 3],
    pub position: [f32; 3],
    pub strength: f32,
    pub radius: f32,
    pub light_type: LightType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelCreateInfo {
    pub name: String,
    pub game_objects: Vec<GameObjectCreateInfo>,
    pub lights: Vec<LightObjectCreateInfo>
}