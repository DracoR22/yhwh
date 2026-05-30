use crate::{asset_manager::AssetManager, common::{create_info::{GameObjectCreateInfo, MeshNodeCreateInfo}, types::Transform}, mesh_nodes::MeshNodes, utils::unique_id};
use cgmath::{Rotation3, Vector3};

pub struct GameObject {
    pub model_name: String,
    pub transform: Transform,
    pub tex_scale: cgmath::Vector2<f32>,
    pub is_selected: bool,
    pub id: usize,
    pub shadows: bool,
    mesh_nodes: MeshNodes,
}

impl GameObject {
    pub fn new(create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        let transform = Transform {
            position: Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            rotation: Vector3::new(create_info.rotation[0], create_info.rotation[1], create_info.rotation[2]),
            size: Vector3::new(create_info.size[0], create_info.size[1], create_info.size[2])
        };

        Self { 
            model_name: create_info.model_name.clone(),
            transform,
            tex_scale: cgmath::Vector2::new(create_info.tex_scale[0], create_info.tex_scale[1]),
            is_selected: false,
            id: unique_id::next_id(),
            mesh_nodes: MeshNodes::new(&create_info.model_name.clone(), &create_info.mesh_rendering_info, asset_manager),
            shadows: create_info.shadows
        }
    }

    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }

    pub fn get_model_matrix(&self) -> cgmath::Matrix4<f32> {
        let translation = cgmath::Matrix4::from_translation(self.transform.position);
        let rotation = cgmath::Matrix4::from(
            cgmath::Quaternion::from_angle_x(cgmath::Deg(self.transform.rotation.x))
            * cgmath::Quaternion::from_angle_y(cgmath::Deg(self.transform.rotation.y))
            * cgmath::Quaternion::from_angle_z(cgmath::Deg(self.transform.rotation.z))
        );
        let scale = cgmath::Matrix4::from_nonuniform_scale(self.transform.size.x, self.transform.size.z, self.transform.size.y);

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

    pub fn set_shadows(&mut self, shadows: bool) {
        self.shadows = shadows;
    }
}

impl GameObject {
    pub fn get_create_info(&self, asset_manager: &AssetManager) -> GameObjectCreateInfo {
        let mut mesh_nodes_create_infos: Vec<MeshNodeCreateInfo> = Vec::new();

        for mesh_node in self.get_mesh_nodes().get_mesh_nodes() {
          if let Some((mesh, material)) = asset_manager.get_mesh_by_index(mesh_node.mesh_index).zip(asset_manager.get_material_by_index(mesh_node.material_index)) {
            let create_info = MeshNodeCreateInfo {
                material_name: material.name.clone(),
                mesh_name: mesh.name.clone(),
                rendering_mode: mesh_node.rendering_mode
                // emissive: mesh_node.emissive,
                // glass: mesh_node.glass
            };

            mesh_nodes_create_infos.push(create_info);
          }
          
        }

        let create_info = GameObjectCreateInfo {
            size: self.transform.size.into(),
            position: self.transform.position.into(),
            rotation: self.transform.rotation.into(),
            tex_scale: self.tex_scale.into(),
            mesh_rendering_info: mesh_nodes_create_infos,
            shadows: self.shadows,
            model_name: self.get_model_name().to_string()
        };

        create_info
    }
}