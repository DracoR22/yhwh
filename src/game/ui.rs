use cgmath::{Matrix4, Vector2, Vector3};

use crate::utils::unique_id;

#[derive(Eq, Hash, PartialEq)]
pub enum UiElementKind {
    Crosshair,
    KeyG
}

pub struct UiElement {
    pub position: Vector2<f32>,
    pub scale: f32,
    pub visible: bool,
    pub texture_name: String,
    pub id: usize
}

impl UiElement {
    pub fn new(position: Vector2<f32>, scale: f32, texture_name: &str) -> Self {
        Self {
            texture_name: texture_name.to_string(),
            position,
            scale,
            visible: true,
            id: unique_id::next_id()
        }
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, 0.0));
        let scale = Matrix4::from_nonuniform_scale(self.scale, self.scale, 1.0);

        let model_matrix = translation * scale;

        model_matrix
    }
}


