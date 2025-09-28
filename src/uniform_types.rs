use std::collections::HashMap;

use cgmath::SquareMatrix;

use crate::{animation::skin::MAX_JOINTS_PER_MESH, camera::{Camera, Projection}, uniform::Uniform};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnimationUniform {
    pub joint_matrices: [[[f32; 4]; 4]; MAX_JOINTS_PER_MESH],
}

impl AnimationUniform {
    pub fn new() -> Self {
      Self {
        joint_matrices: [cgmath::Matrix4::<f32>::identity().into(); MAX_JOINTS_PER_MESH]
      }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model_matrix: [[f32; 4]; 4]
}

impl ModelUniform {
    pub fn new() -> Self {
      Self {
        model_matrix: cgmath::Matrix4::identity().into()
      }
    }

    pub fn update(&mut self, matrix: &cgmath::Matrix4<f32>) {
       self.model_matrix = (*matrix).into();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view: cgmath::Matrix4::identity().into(),
            projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view = camera.calc_matrix().into();
        self.projection = projection.calc_matrix().into();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
   pub position: [f32; 3],
   pub _padding: u32,
   pub color: [f32; 3],
   pub _padding2: u32,
}

impl LightUniform {
    pub fn new() -> Self {
        Self {
          position: [2.0, 2.0, 2.0],
           _padding: 0,
           color: [1.0, 1.0, 1.0],
          _padding2: 0,
        }
    }
}

pub struct WgpuUniforms {
    pub camera: Uniform<CameraUniform>,
    pub models: HashMap<usize, Uniform<ModelUniform>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub animation: Uniform<AnimationUniform>,
    pub light: Uniform<LightUniform>
}