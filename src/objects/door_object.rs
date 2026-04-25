use cgmath::{Deg, Matrix4, Quaternion, Vector3};
use cgmath::Rotation3;
use winit::keyboard::KeyCode;
use yhwh_audio::audio_manager::AudioManager;

use crate::common::create_info::DoorObjectCreateInfo;
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
    pub is_selected: bool
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
            is_selected: false
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

}