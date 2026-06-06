use cgmath::{Deg, Matrix4, Quaternion, Vector2, Vector3};
use cgmath::Rotation3;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;
use yhwh_core::math::aabb::Aabb;

use crate::camera::Camera;
use crate::common::create_info::DoorObjectCreateInfo;
use crate::common::types::RenderItem;
use crate::input::input::Input;
use crate::utils::ray_cast::ray_intersects_aabb;
use crate::{asset_manager::AssetManager, common::{create_info::MeshNodeCreateInfo, types::Transform}, mesh_nodes::MeshNodes, utils::unique_id};

pub enum DoorState {
    Opened,
    Closed,
    // Opening,
    // Closing
}

pub struct DoorObject {
    pub id: usize,
    pub mesh_nodes: MeshNodes,
    pub transform: Transform,
    pub model_name: String,
    pub state: DoorState,
    pub current_angle: f32,
    pub target_angle: f32,
    pub is_selected: bool,
    pub interacted: bool,
    pub closed_angle: f32,
    render_items: Vec<RenderItem>
}

impl DoorObject {
   pub fn new(create_info: &DoorObjectCreateInfo, asset_manager: &AssetManager) -> Self {   
        let model_name = "door2";

        let transform = Transform {
            position: Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            rotation: Vector3::new(create_info.rotation[0], create_info.rotation[1], create_info.rotation[2]),
            size: Vector3::new(create_info.size[0], create_info.size[1], create_info.size[2])
        };

        let mesh_nodes = MeshNodes::new(model_name, &create_info.mesh_rendering_info, asset_manager);

        // let hinge_matrix = mesh_nodes.get_mesh_node_by_mesh_name("door_f_0").unwrap().transform_matrix.clone();        

        Self {
            id: unique_id::next_id(),
            mesh_nodes,
            closed_angle: transform.rotation.y,
            transform,
            model_name: String::from(model_name),
            state: DoorState::Closed,
            current_angle: 0.0,
            target_angle: 0.0,
            is_selected: false,
            interacted: false,
            render_items: Vec::new()
        }
    }

    pub fn update(&mut self, asset_manager: &AssetManager, delta_time: f32) {
         // update render items
        self.render_items.clear();
        let model = asset_manager.model_by_name(&self.model_name).expect(&format!("GameObject error: no model for {}", self.model_name.clone()));
        for node in self.mesh_nodes.get_nodes().iter() {
            let render_item = RenderItem {
                rendering_mode: node.rendering_mode,
                texture_scale: Vector2::new(1.0, 1.0),
                mesh_index: node.mesh_index,
                material_index: node.material_index,
                model_matrix: self.model_matrix(),
                object_id: self.id,
                aabb: model.aabb
            };

            self.render_items.push(render_item);
        }

        // open/closed
        self.target_angle = match self.state {
            DoorState::Opened => self.closed_angle + 90.0,
            DoorState::Closed => self.closed_angle,
        };

        let speed = 120.0;
        let diff = self.target_angle - self.transform.rotation.y;

        if diff.abs() > 0.01 {
            let step = speed * delta_time;
            self.transform.rotation.y += diff.clamp(-step, step);
        } else {
            self.transform.rotation.y = self.target_angle;
        }
    }

    pub fn model_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.transform.position);
        let rotation = Matrix4::from(
            Quaternion::from_angle_x(Deg(self.transform.rotation.x))
            * Quaternion::from_angle_y(Deg(self.transform.rotation.y))
            * Quaternion::from_angle_z(Deg(self.transform.rotation.z))
        );
        let scale = Matrix4::from_nonuniform_scale(self.transform.size.x, self.transform.size.z, self.transform.size.y);

        translation * rotation * scale
    }

    pub fn toggle_state(&mut self, audio_manager: &mut AudioManager, input: &Input) {
        if input.keyboard.key_just_pressed(KeyCode::KeyG) {
            self.state = match self.state {
                DoorState::Opened => {
                    let _ = audio_manager.play_audio("door1_stop.wav", 1.0, 1.0);
                    DoorState::Closed
                },
                DoorState::Closed => {
                    let _ = audio_manager.play_audio("door1_move.wav", 1.0, 1.0);
                    DoorState::Opened
                }
            } 
        }
    }

    pub fn state_to_string(&self) -> String {
        match self.state {
            DoorState::Opened => {
                String::from("Opened")
            },
            DoorState::Closed => {
                String::from("Closed")
            }
        }
    }   

    pub fn interact(&self, asset_manager: &AssetManager, camera: &Camera) -> bool {
        match asset_manager.model_by_name(&self.model_name) {
            Some(model) => {
                if let Some(aabb) = model.aabb {
                    let model_matrix = self.model_matrix();
                    let world_aabb = aabb.transform(model_matrix);

                    let ray_origin = Vector3::<f32>::new(camera.position.x, camera.position.y, camera.position.z);
                    let ray_dir = camera.forward();

                    let max_distance = 10.0;

                    if let Some(distance) = ray_intersects_aabb(ray_origin, ray_dir, &world_aabb) {
                        return distance < max_distance;
                    }
                }
            },
            _ => ()
        };

        return false
    }
}

// Getters
impl DoorObject {
    pub fn get_create_info(&self, asset_manager: &AssetManager) -> DoorObjectCreateInfo {
        let mut mesh_nodes_create_infos: Vec<MeshNodeCreateInfo> = Vec::new();

        for mesh_node in self.mesh_nodes().get_nodes() {
          if let Some((mesh, material)) = asset_manager.mesh_by_index(mesh_node.mesh_index).zip(asset_manager.material_by_index(mesh_node.material_index)) {
            let create_info = MeshNodeCreateInfo {
                material_name: material.name.clone(),
                mesh_name: mesh.name.clone(),
                rendering_mode: mesh_node.rendering_mode.clone()
                // emissive: mesh_node.emissive,
                // glass: mesh_node.glass
            };

            mesh_nodes_create_infos.push(create_info);
          }
          
        }

        let create_info = DoorObjectCreateInfo {
            position: [self.transform.position.x, self.transform.position.y, self.transform.position.z],
            rotation: [self.transform.rotation.x, self.transform.rotation.y, self.transform.rotation.z],
            size: [self.transform.size.x, self.transform.size.y, self.transform.size.z],
            mesh_rendering_info: mesh_nodes_create_infos
        };

        create_info
    }

    pub fn mesh_nodes(&self) -> &MeshNodes {
        &self.mesh_nodes
    }

    pub fn mesh_nodes_mut(&mut self) -> &mut MeshNodes {
        &mut self.mesh_nodes
    }

    pub fn render_items(&self) -> &Vec<RenderItem> {
        &self.render_items
    }
}