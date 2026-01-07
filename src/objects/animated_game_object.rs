use std::collections::HashMap;

use crate::{asset_manager::AssetManager, common::{create_info::GameObjectCreateInfo, types::MeshRenderingInfo}, mesh_nodes::MeshNodes, utils::unique_id};
use cgmath::Rotation3;

pub struct AnimatedGameObject {
    pub object_id: usize,
    model_name: String,
    position: cgmath::Vector3<f32>,
    size: cgmath::Vector3<f32>,
    euler_rotation: cgmath::Vector3<f32>,
    pub tex_scale: cgmath::Vector2<f32>,
    mesh_nodes: MeshNodes
}

impl AnimatedGameObject {
    pub fn new(create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        Self { 
            model_name: create_info.model_name.clone(),
            position: cgmath::Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            euler_rotation: cgmath::Vector3::new(create_info.rotation[0], create_info.rotation[1], create_info.rotation[2]),
            size: cgmath::Vector3::new(create_info.size[0], create_info.size[1], create_info.size[2]),
            tex_scale: cgmath::Vector2::new(create_info.tex_scale[0], create_info.tex_scale[1]),
            object_id: unique_id::next_id(),
            mesh_nodes: MeshNodes::new(&create_info.model_name, &create_info.mesh_rendering_info, asset_manager)
        }
    }

    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn get_rotation(&self) -> cgmath::Vector3<f32> {
        self.euler_rotation
    }

    pub fn get_size(&self) -> cgmath::Vector3<f32> {
        self.size
    }

    pub fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
        // let model_matrix = cgmath::Matrix4::from_translation(self.position)
        //  * self.rotation 
        //  * cgmath::Matrix4::from_nonuniform_scale(self.size.x, self.size.y, self.size.z);
         
        //  model_matrix

        let translation = cgmath::Matrix4::from_translation(self.position);
        let rotation = cgmath::Matrix4::from(
            cgmath::Quaternion::from_angle_x(cgmath::Deg(self.euler_rotation.x))
            * cgmath::Quaternion::from_angle_y(cgmath::Deg(self.euler_rotation.y))
            * cgmath::Quaternion::from_angle_z(cgmath::Deg(self.euler_rotation.z))
        );
        let scale = cgmath::Matrix4::from_nonuniform_scale(self.size.x, self.size.z, self.size.y);

        translation * rotation * scale
    }

    pub fn get_mesh_nodes(&self) -> &MeshNodes {
        &self.mesh_nodes
    }

    pub fn get_mesh_nodes_mut(&mut self) -> &mut MeshNodes {
        &mut self.mesh_nodes
    }
}