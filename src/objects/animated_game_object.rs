use std::collections::HashMap;

use crate::{asset_manager::AssetManager, common::{create_info::GameObjectCreateInfo, types::MeshRenderingInfo}, utils::unique_id};

pub struct AnimatedGameObject {
    pub object_id: usize,
    name: String,
    model_name: String,
    position: cgmath::Vector3<f32>,
    size: cgmath::Vector3<f32>,
    rotation: cgmath::Matrix4<f32>,
    mesh_rendering_info: Vec<MeshRenderingInfo>,
    mesh_rendering_info_index_map: HashMap<String, usize>,
}

impl AnimatedGameObject {
    pub fn new(create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        let model = asset_manager.get_model_by_name(&create_info.model_name).unwrap();

        let mut mesh_rendering_info: Vec<MeshRenderingInfo> = Vec::new();
        let mut mesh_rendering_info_index_map: HashMap<String, usize> = HashMap::new();

        if create_info.mesh_rendering_info.is_empty() {
            for mesh in &model.meshes {
            let mesh_index = asset_manager.get_mesh_index_by_name(&mesh.name);
            let material_index = asset_manager.get_material_index_by_name("Default");

            mesh_rendering_info.push(MeshRenderingInfo {
                mesh_index,
                material_index
            });
            mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
            }
        } else {
            for info in &create_info.mesh_rendering_info {
            let mesh = asset_manager.get_mesh_by_name(&info.mesh_name).expect("GameObject::new() error: Mesh {info.name} not found in model meshes");
            
            let mesh_index = asset_manager.get_mesh_index_by_name(&info.mesh_name);
            let material_index = asset_manager.get_material_index_by_name(&info.material_name);

            mesh_rendering_info.push(MeshRenderingInfo {
                mesh_index,
                material_index
            });
            mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
           }
        }

        Self { 
            model_name: create_info.model_name.clone(),
            name: create_info.name.clone(),
            position: create_info.position,
            rotation: create_info.rotation,
            size: create_info.size,
            object_id: unique_id::next_id(),
            mesh_rendering_info,
            mesh_rendering_info_index_map
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

    pub fn get_mesh_material_index(&self, mesh_name: &str) -> usize {
        if let Some(&index) = self.mesh_rendering_info_index_map.get(mesh_name) {
          self.mesh_rendering_info[index].material_index
        } else {
          println!("GameObject::get_mesh_material_index() error: mesh {mesh_name} not found!");
          0
        }
    }

    pub fn set_mesh_material(&mut self, asset_manager: &AssetManager, mesh_name: &str, material_name: &str) {
        let mesh_index = asset_manager.get_mesh_index_by_name(mesh_name);
        let material_index = asset_manager.get_mesh_index_by_name(material_name);

        for info in self.mesh_rendering_info.iter_mut() {
            if info.mesh_index == mesh_index {
                info.material_index = material_index;
                return
            }
        }
    } 
}