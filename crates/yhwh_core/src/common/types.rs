use cgmath::{Matrix4, Vector2, Vector3, Vector4};

use crate::{common::constants::MAX_JOINTS_PER_MESH, common::enums::MeshRenderingMode, math::aabb::Aabb};

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

#[derive(Clone)]
pub struct RenderItem {
    pub model_matrix: Matrix4<f32>,

    pub emissive_color: Vector4<f32>,
    
    pub object_id: usize,
    pub mesh_node_id: usize,
    pub mesh_index: usize,
    pub material_index: usize,

    pub aabb: Option<Aabb<f32>>,

    pub rendering_mode: MeshRenderingMode,
    pub texture_scale: Vector2<f32>
}

#[derive(Clone)]
pub struct AnimatedRenderData {
    pub object_id: usize,
    pub joint_matrices: [Matrix4<f32>; MAX_JOINTS_PER_MESH],
}