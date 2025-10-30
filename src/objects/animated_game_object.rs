use std::collections::HashMap;

use crate::{asset_manager::AssetManager, common::{create_info::GameObjectCreateInfo, types::MeshRenderingInfo}, mesh_nodes::MeshNodes, utils::unique_id};

pub struct AnimatedGameObject {
    pub object_id: usize,
    name: String,
    model_name: String,
    position: cgmath::Vector3<f32>,
    size: cgmath::Vector3<f32>,
    rotation: cgmath::Matrix4<f32>,
    mesh_nodes: MeshNodes
}

impl AnimatedGameObject {
    pub fn new(create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        Self { 
            model_name: create_info.model_name.clone(),
            name: create_info.name.clone(),
            position: create_info.position,
            rotation: create_info.rotation,
            size: create_info.size,
            object_id: unique_id::next_id(),
            mesh_nodes: MeshNodes::new(&create_info.model_name, &create_info.mesh_rendering_info, asset_manager)
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_model_name(&self) -> &String {
        &self.model_name
    }

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn get_rotation_matrix(&self) -> cgmath::Matrix4<f32> {
        self.rotation
    }

    pub fn get_size(&self) -> cgmath::Vector3<f32> {
        self.size
    }

    pub fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
        let model_matrix = cgmath::Matrix4::from_translation(self.position)
         * self.rotation 
         * cgmath::Matrix4::from_nonuniform_scale(self.size.x, self.size.y, self.size.z);
         
         model_matrix
    }

    pub fn get_mesh_nodes(&self) -> &MeshNodes {
        &self.mesh_nodes
    }

    pub fn get_mesh_nodes_mut(&mut self) -> &mut MeshNodes {
        &mut self.mesh_nodes
    }
}