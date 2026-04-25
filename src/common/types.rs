use cgmath::Vector3;

pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub size: Vector3<f32>
}

impl Transform  {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, size: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            size
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            size: Vector3::new(1.0, 1.0, 1.0)
        }
    }
}