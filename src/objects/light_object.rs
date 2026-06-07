use cgmath::{SquareMatrix, Transform};
use yhwh_core::common::{constants::{SHADOW_MAP_NEAR_PLANE, SHADOW_MAP_RES_SIZE}, create_info::LightObjectCreateInfo, enums::LightType};

use crate::{utils::unique_id};

pub struct LightObject {
    pub color: cgmath::Vector3<f32>,
    pub position: cgmath::Vector3<f32>,
    pub strength: f32,
    pub radius: f32,
    pub light_type: LightType,
    pub id: usize,
    pub view_matrices: [cgmath::Matrix4<f32>; 6],
    pub projection_transforms: [cgmath::Matrix4<f32>; 6],
    pub shadows: bool
}

const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0,-1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl LightObject {
    pub fn new(create_info: &LightObjectCreateInfo) -> Self {
        Self {
            color: cgmath::Vector3::new(create_info.color[0], create_info.color[1], create_info.color[2]),
            position: cgmath::Vector3::new(create_info.position[0], create_info.position[1], create_info.position[2]),
            radius: create_info.radius,
            strength: create_info.strength,
            light_type: create_info.light_type.clone(),
            id: unique_id::next_id(),
            view_matrices: [cgmath::Matrix4::identity(); 6],
            projection_transforms: [cgmath::Matrix4::identity(); 6],
            shadows: create_info.shadows
        }
    }

     pub fn get_create_info(&self) -> LightObjectCreateInfo {
        let create_info = LightObjectCreateInfo { 
            position: [self.position.x, self.position.y, self.position.z],
            color: [self.color.x, self.color.y, self.color.z],
            radius: self.radius,
            strength: self.strength,
            light_type: self.light_type.clone(),
            shadows: self.shadows
        };

        create_info
    }

    pub fn update(&mut self) {
        self.update_matrices();
    } 

    pub fn update_matrices(&mut self) {
        let point3_pos = cgmath::point3(self.position.x, self.position.y, self.position.z);

        let projection_matrix = cgmath::perspective(cgmath::Rad(90.0f32.to_radians()), SHADOW_MAP_RES_SIZE as f32 / SHADOW_MAP_RES_SIZE as f32, SHADOW_MAP_NEAR_PLANE, self.radius);

        self.view_matrices[0] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(1.0, 0.0, 0.0), cgmath::vec3(0.0, -1.0, 0.0));
        self.view_matrices[1] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(-1.0, 0.0, 0.0), cgmath::vec3(0.0, -1.0, 0.0));
        self.view_matrices[2] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(0.0, 1.0, 0.0), cgmath::vec3(0.0, 0.0, 1.0));
        self.view_matrices[3] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(0.0, -1.0, 0.0), cgmath::vec3(0.0 ,0.0, -1.0));
        self.view_matrices[4] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(0.0, 0.0, 1.0), cgmath::vec3(0.0, -1.0, 0.0));
        self.view_matrices[5] = cgmath::Matrix4::look_at_rh(point3_pos, point3_pos + cgmath::vec3(0.0,0.0,-1.0), cgmath::vec3(0.0,-1.0,0.0));

        self.projection_transforms[0] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[0];
        self.projection_transforms[1] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[1];
        self.projection_transforms[2] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[2];
        self.projection_transforms[3] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[3];
        self.projection_transforms[4] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[4];
        self.projection_transforms[5] = OPENGL_TO_WGPU_MATRIX * projection_matrix * self.view_matrices[5];
    }
}

