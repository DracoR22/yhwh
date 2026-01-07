use crate::{asset_manager::AssetManager, common::{create_info::{GameObjectCreateInfo, MeshNodeCreateInfo}, types::MeshRenderingInfo}, mesh_nodes::MeshNodes, utils::unique_id};
use cgmath::Rotation3;

pub struct GameObject {
    model_name: String,
    position: cgmath::Vector3<f32>,
    size: cgmath::Vector3<f32>,
    euler_rotation: cgmath::Vector3<f32>,
    pub tex_scale: cgmath::Vector2<f32>,
    pub is_selected: bool,
    pub id: usize,
    mesh_nodes: MeshNodes,
}

impl GameObject {
    pub fn new(create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        Self { 
            model_name: create_info.model_name.clone(),
            position: cgmath::Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            euler_rotation: cgmath::Vector3::new(create_info.rotation[0], create_info.rotation[1], create_info.rotation[2]),
            size: cgmath::Vector3::new(create_info.size[0], create_info.size[1], create_info.size[2]),
            tex_scale: cgmath::Vector2::new(create_info.tex_scale[0], create_info.tex_scale[1]),
            is_selected: false,
            id: unique_id::next_id(),
            mesh_nodes: MeshNodes::new(&create_info.model_name.clone(), &create_info.mesh_rendering_info, asset_manager),
        }
    }

    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn get_position_mut(&mut self) -> &mut cgmath::Vector3<f32> {
        &mut self.position
    }

    pub fn get_rotation(&self) -> cgmath::Vector3<f32> {
        self.euler_rotation
    }

    pub fn get_size(&self) -> cgmath::Vector3<f32> {
        self.size
    }

    pub fn get_size_mut(&mut self) -> &mut cgmath::Vector3<f32> {
        &mut self.size
    }

    pub fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
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

    pub fn set_selected(&mut self, value: bool) {
        self.is_selected = value;
    }

    pub fn set_position(&mut self, position: cgmath::Vector3<f32>) {
        self.position = position;
    }

    pub fn set_size(&mut self, size: cgmath::Vector3<f32>) {
        self.size = size;
    }

    pub fn set_rotation(&mut self, rotation: cgmath::Vector3<f32>) {
        self.euler_rotation = rotation;
    }
}

impl GameObject {
    pub fn get_create_info(&self, asset_manager: &AssetManager) -> GameObjectCreateInfo {
        let mut mesh_nodes_create_infos: Vec<MeshNodeCreateInfo> = Vec::new();

        for mesh_node in self.get_mesh_nodes().get_mesh_rendering_info() {
          if let Some((mesh, material)) = asset_manager.get_mesh_by_index(mesh_node.mesh_index).zip(asset_manager.get_material_by_index(mesh_node.material_index)) {
            let create_info = MeshNodeCreateInfo {
                material_name: material.name.clone(),
                mesh_name: mesh.name.clone()
            };

            mesh_nodes_create_infos.push(create_info);
          }
          
        }

        let create_info = GameObjectCreateInfo {
            size: self.get_size().into(),
            position: self.get_position().into(),
            rotation: self.get_rotation().into(),
            tex_scale: self.tex_scale.into(),
            mesh_rendering_info: mesh_nodes_create_infos,
            model_name: self.get_model_name().to_string()
        };

        create_info
    }
}