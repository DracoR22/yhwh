use cgmath::{Matrix4, Vector2};

use crate::objects::{animated_game_object::AnimatedGameObject, door_object::DoorObject, game_object::GameObject};

pub struct ModelInstance {
    pub object_id: usize,
    pub model_matrix: Matrix4<f32>,
    pub texture_scale: Vector2<f32>
}

pub trait ModelInstanceSource {
    fn id(&self) -> usize;
    fn model_matrix(&self) -> Matrix4<f32>;
    fn texture_scale(&self) -> Vector2<f32>;
}

impl ModelInstanceSource for GameObject {
    fn model_matrix(&self) -> Matrix4<f32> {
        self.model_matrix()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn texture_scale(&self) -> Vector2<f32> {
        self.tex_scale
    }
}

impl ModelInstanceSource for AnimatedGameObject {
    fn model_matrix(&self) -> Matrix4<f32> {
        self.model_matrix()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn texture_scale(&self) -> Vector2<f32> {
        self.tex_scale
    }
}

impl ModelInstanceSource for DoorObject {
    fn model_matrix(&self) -> Matrix4<f32> {
        self.model_matrix()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn texture_scale(&self) -> Vector2<f32> {
        Vector2::new(1.0, 1.0)
    }
}