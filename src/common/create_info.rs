use std::fmt;
use serde::{Deserialize, Serialize};
use crate::common::enums::{LightType, MeshRenderingMode};

#[derive(Serialize, Deserialize, Debug)]
pub struct MeshNodeCreateInfo {
    pub mesh_name: String,
    pub material_name: String,
    pub rendering_mode: MeshRenderingMode
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameObjectCreateInfo {
    pub model_name: String,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
    pub tex_scale: [f32; 2],
    pub shadows: bool,
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimatedGameObjectCreateInfo {
    pub model_name: String,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
    pub tex_scale: [f32; 2],
    pub loop_anim: bool,
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightObjectCreateInfo {
    pub color: [f32; 3],
    pub position: [f32; 3],
    pub strength: f32,
    pub radius: f32,
    pub light_type: LightType,
    pub shadows: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DoorObjectCreateInfo {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CandleObjectCreateInfo {
    pub model_name: String,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 3],
    pub flame_color: [f32; 3],
    pub extinguished: bool,
    pub mesh_rendering_info: Vec<MeshNodeCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneCreateInfo {
    pub name: String,
    pub game_objects: Vec<GameObjectCreateInfo>,
    pub door_objects: Vec<DoorObjectCreateInfo>,
    pub lights: Vec<LightObjectCreateInfo>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MapCreateInfo {
    pub name: String,
    pub chunks: Vec<String>
}

impl fmt::Display for MeshRenderingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshRenderingMode::Pbr => write!(f, "Pbr"),
            MeshRenderingMode::Emissive => write!(f, "Emissive"),
            MeshRenderingMode::Glass => write!(f, "Glass"),
            MeshRenderingMode::Flame => write!(f, "Flame"),
        }
    }
}