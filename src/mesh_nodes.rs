use std::collections::HashMap;

use cgmath::{Matrix4};

use crate::{asset_manager::AssetManager, common::create_info::{MeshNodeCreateInfo}, common::enums::MeshRenderingMode, utils::unique_id};

pub struct MeshNode {
   pub id: usize,
   pub mesh_index: usize,
   pub material_index: usize,
   pub rendering_mode: MeshRenderingMode,
   pub transform_matrix: Matrix4<f32>,
}

pub struct MeshNodes {
    model_name: String,
    nodes: Vec<MeshNode>,
    nodes_index_map: HashMap<String, usize>,
}

impl MeshNodes {
    pub fn new(model_name: &str, create_info: &Vec<MeshNodeCreateInfo>, asset_manager: &AssetManager) -> Self {
        let mut mesh_rendering_info: Vec<MeshNode> = Vec::new();
        let mut mesh_rendering_info_index_map: HashMap<String, usize> = HashMap::new();

        if create_info.is_empty() {
            if let Some(model) = asset_manager.model_by_name(model_name) {

            for mesh in &model.meshes {
                //  let mesh_index = asset_manager.mesh_index_by_name(model_name, &mesh.name);
                 let material_index = asset_manager.material_index_by_name("Default");

                 mesh_rendering_info.push(MeshNode {
                 id: unique_id::next_id(),
                 mesh_index: mesh.global_index,
                 material_index,
                 rendering_mode: MeshRenderingMode::Pbr,
                 transform_matrix: mesh.transform_matrix
                });
                mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
            }
          }
        } else {
            let model = asset_manager.model_by_name(model_name).unwrap();
            for (info, mesh) in create_info.iter().zip(model.meshes.iter()) { // Hack!!
                
                //let mesh = asset_manager.mesh_by_name(&info.mesh_name).expect(&format!("MeshNodes::new() error: Mesh {} not found in model meshes", info.mesh_name));
                // let mesh_index = asset_manager.mesh_index_by_name(model_name, &info.mesh_name);
                let material_index = asset_manager.material_index_by_name(&info.material_name);
                
                mesh_rendering_info.push(MeshNode {
                        id: unique_id::next_id(),
                        mesh_index: mesh.global_index,
                        material_index,
                        rendering_mode: info.rendering_mode,
                        transform_matrix: mesh.transform_matrix
                });
                mesh_rendering_info_index_map.insert(mesh.name.clone(), mesh_rendering_info.len() - 1);
            
           }
        }

        Self {
            model_name: model_name.to_string(),
            nodes: mesh_rendering_info,
            nodes_index_map: mesh_rendering_info_index_map
        }
    }

    pub fn get_model_name(&self) -> &String {
        &self.model_name
    }

    pub fn get_mesh_material_index_by_mesh_name(&self, mesh_name: &str) -> usize {
        if let Some(&index) = self.nodes_index_map.get(mesh_name) {
          self.nodes[index].material_index
        } else {
          println!("MeshNodes::get_mesh_material_index_by_mesh_name() error: mesh {mesh_name} not found!");
          0
        }
    }

    pub fn set_mesh_material(&mut self, asset_manager: &AssetManager, model_name: &str, mesh_name: &str, material_name: &str) {
        let mesh_index = asset_manager.mesh_index_by_name(model_name, mesh_name);
        let material_index = asset_manager.material_index_by_name(material_name);

        for info in self.nodes.iter_mut() {
            if info.mesh_index == mesh_index {
                info.material_index = material_index;
                return
            }
        }
    }

    pub fn get_mesh_node_by_mesh_name(&self, mesh_name: &str) -> Option<&MeshNode> {
        if let Some(&index) = self.nodes_index_map.get(mesh_name) {
            Some(&self.nodes[index])
        } else {
            println!("MeshNodes::get_mesh_rendering_info_by_mesh_name() error: mesh {mesh_name} not found!");
            None
        }
    }

    pub fn get_mesh_node_by_mesh_name_mut(&mut self, mesh_name: &str) -> Option<&mut MeshNode> {
        if let Some(&index) = self.nodes_index_map.get(mesh_name) {
            Some (&mut self.nodes[index])
        } else {
           None
        }
    }

    pub fn get_nodes(&self) -> &Vec<MeshNode> {
        &self.nodes
    }
}