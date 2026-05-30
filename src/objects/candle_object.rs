use cgmath::Vector3;

use crate::{asset_manager::AssetManager, common::{create_info::CandleObjectCreateInfo, types::Transform}, mesh_nodes::MeshNodes, utils::unique_id};

pub struct CandleObject {
    pub id: usize,
    pub transform: Transform,
    pub model_name: String,
    pub is_selected: bool,
    pub extinguished: bool,
    pub flame_color: Vector3<f32>,
    pub mesh_nodes: MeshNodes
}

impl CandleObject {
    pub fn new(create_info: CandleObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        let transform = Transform {
            position: create_info.position.into(),
            size: create_info.size.into(),
            rotation: create_info.rotation.into()
        };
        Self {
            transform,
            model_name: create_info.model_name.clone(),
            extinguished: create_info.extinguished,
            flame_color: create_info.flame_color.into(),
            mesh_nodes: MeshNodes::new(&create_info.model_name.clone(), &create_info.mesh_rendering_info, asset_manager),
            is_selected: false,
            id: unique_id::next_id()
        }
    }
}