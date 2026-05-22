use cgmath::Vector3;

use crate::{common::types::Transform, mesh_nodes::MeshNodes};

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
    pub fn new() {
        
    }
}