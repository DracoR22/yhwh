use std::sync::Arc;

use egui::ahash::HashMap;

use crate::{asset_manager::AssetManager, model::Model};

pub struct MeshRenderingInfo {
   pub mesh_index: u32,
   pub material_index: u32
}

pub struct AnimatedObject {
    model: Arc<Model>,
    mesh_rendering_info: HashMap<String, MeshRenderingInfo>
}

impl AnimatedObject {
    pub fn new(model: &Model, asset_manager: &AssetManager) {
        // for mesh in &model.meshes {
        //     let mesh_index = asset_manager.
        // }
    }
}