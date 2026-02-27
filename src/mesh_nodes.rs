use std::collections::HashMap;

use crate::{asset_manager::AssetManager, common::{create_info::MeshNodeCreateInfo, types::MeshRenderingInfo}};

pub struct MeshNodes {
    model_name: String,
    mesh_rendering_info: Vec<MeshRenderingInfo>,
    mesh_rendering_info_index_map: HashMap<String, usize>,
}

impl MeshNodes {
    pub fn new(model_name: &str, create_info: &Vec<MeshNodeCreateInfo>, asset_manager: &AssetManager) -> Self {
        let mut mesh_rendering_info: Vec<MeshRenderingInfo> = Vec::new();
        let mut mesh_rendering_info_index_map: HashMap<String, usize> = HashMap::new();

        if create_info.is_empty() {
            if let Some(model) = asset_manager.get_model_by_name(model_name) {
            for mesh in &model.meshes {
                 let mesh_index = asset_manager.get_mesh_index_by_name(&mesh.name);
                 let material_index = asset_manager.get_material_index_by_name("Default");
                 mesh_rendering_info.push(MeshRenderingInfo {
                 mesh_index,
                 material_index,
                 emissive: false
                });
                mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
            }
          }
        } else {
            for info in create_info.iter() {
            let mesh = asset_manager.get_mesh_by_name(&info.mesh_name).expect("GameObject::new() error: Mesh {info.name} not found in model meshes");
            
            let mesh_index = asset_manager.get_mesh_index_by_name(&info.mesh_name);
            let material_index = asset_manager.get_material_index_by_name(&info.material_name);

            mesh_rendering_info.push(MeshRenderingInfo {
                mesh_index,
                material_index,
                emissive: info.emissive
            });
            mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
           }
        }

        Self {
            model_name: model_name.to_string(),
            mesh_rendering_info,
            mesh_rendering_info_index_map
        }
    }

    pub fn get_model_name(&self) -> &String {
        &self.model_name
    }

    pub fn get_mesh_material_index_by_mesh_name(&self, mesh_name: &str) -> usize {
        if let Some(&index) = self.mesh_rendering_info_index_map.get(mesh_name) {
          self.mesh_rendering_info[index].material_index
        } else {
          println!("MeshNodes::get_mesh_material_index_by_mesh_name() error: mesh {mesh_name} not found!");
          0
        }
    }

    pub fn set_mesh_material(&mut self, asset_manager: &AssetManager, mesh_name: &str, material_name: &str) {
        let mesh_index = asset_manager.get_mesh_index_by_name(mesh_name);
        let material_index = asset_manager.get_material_index_by_name(material_name);

        for info in self.mesh_rendering_info.iter_mut() {
            if info.mesh_index == mesh_index {
                info.material_index = material_index;
                return
            }
        }
    }

    pub fn get_mesh_rendering_info_by_mesh_name(&self, mesh_name: &str) -> &MeshRenderingInfo {
        if let Some(&index) = self.mesh_rendering_info_index_map.get(mesh_name) {
            &self.mesh_rendering_info[index]
        } else {
            println!("MeshNodes::get_mesh_rendering_info_by_mesh_name() error: mesh {mesh_name} not found!");
            &MeshRenderingInfo { 
                mesh_index: 0,
                material_index: 0,
                emissive: false
            }
        }
    }

    pub fn get_mesh_rendering_info_by_mesh_name_mut(&mut self, mesh_name: &str) -> Option<&mut MeshRenderingInfo> {
        if let Some(&index) = self.mesh_rendering_info_index_map.get(mesh_name) {
            Some (&mut self.mesh_rendering_info[index])
        } else {
           None
        }
    }

    pub fn get_mesh_rendering_infos(&self) -> &Vec<MeshRenderingInfo> {
        &self.mesh_rendering_info
    }
}
