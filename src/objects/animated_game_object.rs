use std::collections::HashMap;

use crate::{animation::{animation::{AnimationState, Animations, PlaybackMode, load_animations}, node::Nodes, skin::{Skin, create_skins_from_gltf}}, asset_manager::AssetManager, common::{create_info::{AnimatedGameObjectCreateInfo, GameObjectCreateInfo}, types::Transform}, mesh_nodes::MeshNodes, utils::unique_id};
use cgmath::{Matrix4, Rotation3, SquareMatrix, Vector3};

pub struct AnimatedGameObject {
    pub id: usize,
    pub model_name: String,
    // pub position: cgmath::Vector3<f32>,
    // pub size: cgmath::Vector3<f32>,
    // pub euler_rotation: cgmath::Vector3<f32>,
    pub transform: Transform,
    pub tex_scale: cgmath::Vector2<f32>,
    pub is_selected: bool,

    pub mesh_nodes: MeshNodes,

    pub animations: Option<Animations>,
    pub nodes: Nodes,
    pub skins: Vec<Skin>,
    pub model_matrix: Matrix4<f32>
}

#[derive(Debug)]
pub enum AnimatedGameObjectError {
    InvalidModel,
    NoNodes,
    Error
}

impl AnimatedGameObject {
    pub fn new(create_info: &AnimatedGameObjectCreateInfo, asset_manager: &AssetManager) -> Self {
        let model = asset_manager.get_model_by_name(&create_info.model_name).unwrap();
        let gltf = model.gltf.as_ref().unwrap();
        let buffers = model.gltf_buffers.as_ref().unwrap();
        let model_transform = &model.global_transform;

        // load animations
        let animations = load_animations(gltf.animations(), &buffers);

        // load skins
        let mut skins = create_skins_from_gltf(gltf.skins(), &buffers);

        // load nodes
        //let mut nodes = Nodes::from_gltf_nodes(gltf.nodes(), &gltf.default_scene().ok_or(AnimatedGameObjectError::NoNodes)?);
        let mut nodes = Nodes::from_gltf_nodes(gltf.nodes(), &gltf.default_scene().unwrap());

        nodes.transform(Some(*model_transform));
        nodes
        .get_skins_transform()
        .iter()
        .for_each(|(index, model_transform)| {
            let skin = &mut skins[*index];
            skin.compute_joints_matrices(*model_transform, nodes.nodes());
        });

        let transform = Transform {
            position: Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            rotation: Vector3::new(create_info.rotation[0], create_info.rotation[1], create_info.rotation[2]),
            size: Vector3::new(create_info.size[0], create_info.size[1], create_info.size[2])
        };

        // if let Some(animations) = &animations {
        //     for (i, a) in animations.animations().iter().enumerate() {
        //         println!("ANIN NAME: {} INDEX: {}", a.get_name(), i);
        //     }
        // }

        Self { 
            model_name: create_info.model_name.clone(),
            transform,
            tex_scale: cgmath::Vector2::new(create_info.tex_scale[0], create_info.tex_scale[1]),
            id: unique_id::next_id(),
            is_selected: false,
            mesh_nodes: MeshNodes::new(&create_info.model_name, &create_info.mesh_rendering_info, asset_manager),
            animations,
            nodes, 
            skins,
            model_matrix: Matrix4::identity()
        }
    }

    pub fn update(&mut self, asset_manager: &AssetManager, delta_time: f32) -> bool {
        let model = asset_manager.get_model_by_name(&self.model_name);
        if model.is_none() {
            return false
        }

        let updated = if let Some(animations) = self.animations.as_mut() {
            animations.update(&mut self.nodes, delta_time)
        } else {
            false
        };

        if updated {
            self.nodes.transform(Some(model.unwrap().global_transform));
            self.nodes
                .get_skins_transform()
                .iter()
                .for_each(|(index, transform)| {
                    let skin = &mut self.skins[*index];
                    skin.compute_joints_matrices(*transform, self.nodes.nodes());
                });
        }

        updated
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
}

impl AnimatedGameObject {
    pub fn get_animation_playback_state(&self) -> Option<AnimationState> {
        self.animations
            .as_ref()
            .map(Animations::get_playback_state)
            .copied()
    }

    pub fn set_current_animation(&mut self, animation_index: usize) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_current(animation_index);
        }
    }

    pub fn restart_current_animation(&mut self, animation_index: usize) {
        if let Some(animations) = self.animations.as_mut() {
            animations.restart_current(animation_index);
        }
    }

    pub fn set_animation_playback_mode(&mut self, playback_mode: PlaybackMode) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_playback_mode(playback_mode);
        }
    }

    pub fn toggle_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.toggle();
        }
    }

    pub fn stop_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.stop();
        }
    }

    pub fn reset_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.reset();
        }
    }
}