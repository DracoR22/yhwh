use cgmath::{Deg, Matrix4, Quaternion, Vector3};
use cgmath::Rotation3;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;
use yhwh_core::math::aabb::Aabb;

use crate::camera::Camera;
use crate::common::create_info::DoorObjectCreateInfo;
use crate::engine::GameData;
use crate::input::input::Input;
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
    pub interacted: bool
}

impl DoorObject {
   pub fn new(create_info: &DoorObjectCreateInfo, asset_manager: &AssetManager) -> Self {   
        let model_name = "door";

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
            transform,
            model_name: String::from(model_name),
            state: DoorState::Closed,
            current_angle: 0.0,
            target_angle: 0.0,
            is_selected: false,
            interacted: false
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.target_angle = match self.state {
            DoorState::Opened => 90.0,
            DoorState::Closed => 0.0,
        };

        let speed = 120.0;
        let diff = self.target_angle - self.current_angle;

        if diff.abs() > 0.01 {
            let step = speed * delta_time;
            self.current_angle += diff.clamp(-step, step);
        }
       
        self.transform.rotation.y = self.current_angle;


    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(self.transform.position);
        let rotation = Matrix4::from(
            Quaternion::from_angle_x(Deg(self.transform.rotation.x))
            * Quaternion::from_angle_y(Deg(self.transform.rotation.y))
            * Quaternion::from_angle_z(Deg(self.transform.rotation.z))
        );
        let scale = Matrix4::from_nonuniform_scale(self.transform.size.x, self.transform.size.z, self.transform.size.y);

        translation * rotation * scale
    }

    pub fn get_mesh_nodes(&self) -> &MeshNodes {
        &self.mesh_nodes
    }

    pub fn get_mesh_nodes_mut(&mut self) -> &mut MeshNodes {
        &mut self.mesh_nodes
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
        match asset_manager.get_model_by_name(&self.model_name) {
            Some(model) => {
                if let Some(aabb) = model.aabb {
                    let model_matrix = self.get_model_matrix();
                    let world_aabb = aabb.transform(model_matrix);

                    let ray_origin = Vector3::<f32>::new(camera.position.x, camera.position.y, camera.position.z);
                    let ray_dir = camera.forward();

                    let max_distance = 30.0;

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

impl DoorObject {
    pub fn get_create_info(&self, asset_manager: &AssetManager) -> DoorObjectCreateInfo {
        let mut mesh_nodes_create_infos: Vec<MeshNodeCreateInfo> = Vec::new();

        for mesh_node in self.get_mesh_nodes().get_mesh_nodes() {
          if let Some((mesh, material)) = asset_manager.get_mesh_by_index(mesh_node.mesh_index).zip(asset_manager.get_material_by_index(mesh_node.material_index)) {
            let create_info = MeshNodeCreateInfo {
                material_name: material.name.clone(),
                mesh_name: mesh.name.clone(),
                emissive: mesh_node.emissive,
                glass: mesh_node.glass
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
}

pub fn ray_intersects_aabb(origin: Vector3<f32>, dir: Vector3<f32>, aabb: &Aabb<f32>) -> Option<f32> {
    let mut tmin = (aabb.min.x - origin.x) / dir.x;
    let mut tmax = (aabb.max.x - origin.x) / dir.x;

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
    }

    let mut tymin = (aabb.min.y - origin.y) / dir.y;
    let mut tymax = (aabb.max.y - origin.y) / dir.y;

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
    }

    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }

    tmin = tmin.max(tymin);
    tmax = tmax.min(tymax);

    let mut tzmin = (aabb.min.z - origin.z) / dir.z;
    let mut tzmax = (aabb.max.z - origin.z) / dir.z;

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
    }

    if (tmin > tzmax) || (tzmin > tmax) {
        return None;
    }

    tmin = tmin.max(tzmin);
    tmax = tmax.min(tzmax);

    if tmax < 0.0 {
        return None; // box is behind you
    }

    Some(tmin.max(0.0))
}