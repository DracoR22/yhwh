use std::collections::HashMap;

use cgmath::Matrix;
use cgmath::SquareMatrix;
use cgmath::Rotation3;

use crate::asset_manager::AssetManager;
use crate::bind_group_manager::BindGroupManager;
use crate::objects::animated_game_object::AnimatedGameObject;
use crate::{animation::skin::MAX_JOINTS_PER_MESH, camera::{Camera, Projection}, objects::game_object::GameObject, uniform::Uniform, wgpu_context::WgpuContext};

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
    pub model_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3]
}

impl ModelUniform {
    pub fn new() -> Self {
      let normal = cgmath::Matrix3::identity();
      Self {
        model_matrix: cgmath::Matrix4::identity().into(),
        normal_matrix: [
              [normal.x.x, normal.x.y, normal.x.z, 0.0],
              [normal.y.x, normal.y.y, normal.y.z, 0.0],
              [normal.z.x, normal.z.y, normal.z.z, 0.0],
          ]
      }
    }

    pub fn update(&mut self, matrix: &cgmath::Matrix4<f32>) {
       self.model_matrix = (*matrix).into();

       let upper3x3 = cgmath::Matrix3::from_cols(
        matrix.x.truncate(),
        matrix.y.truncate(),
        matrix.z.truncate(),
       );

       if let Some(normal) = upper3x3.invert() {
        let transposed = normal.transpose();
        self.normal_matrix = [
            [transposed.x.x, transposed.x.y, transposed.x.z, 0.0],
            [transposed.y.x, transposed.y.y, transposed.y.z, 0.0],
            [transposed.z.x, transposed.z.y, transposed.z.z, 0.0],
        ];
       }
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

    pub fn update(&mut self, camera: &Camera) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view = camera.calc_matrix().into();
        self.projection = camera.get_projection().calc_matrix().into();
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

pub struct UniformManager {
    pub camera: Uniform<CameraUniform>,
    pub models: HashMap<usize, Uniform<ModelUniform>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub animation: Uniform<AnimationUniform>,
    pub light: Uniform<LightUniform>
}

impl UniformManager {
    pub fn new(ctx: &WgpuContext, game_objects: &Vec<GameObject>, animated_game_objects: &Vec<AnimatedGameObject>) -> Self {
      let mut model_uniforms: HashMap<usize, Uniform<ModelUniform>> = HashMap::new();

      for game_object in game_objects.iter() {
        model_uniforms.insert(game_object.object_id, Uniform::new(ModelUniform::new(), &ctx.device));
      }

      for animated_game_object in animated_game_objects.iter() {
        model_uniforms.insert(animated_game_object.object_id, Uniform::new(ModelUniform::new(), &ctx.device));
      }

      let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
        &ctx.device,
        wgpu::ShaderStages::VERTEX_FRAGMENT,
        Some("Uniform_Bind_Group_Layout"))
      .unwrap();

      Self {
        models: model_uniforms,
        animation: Uniform::new(AnimationUniform::new(), &ctx.device),
        camera: Uniform::new(CameraUniform::new(), &ctx.device),
        light: Uniform::new(LightUniform::new(), &ctx.device),
        bind_group_layout
      }
    }
    pub fn submit_model_uniforms(&mut self, ctx: &WgpuContext, game_objects: &Vec<GameObject>, animated_game_objects: &Vec<AnimatedGameObject>) {
      for animated_game_object in animated_game_objects {
        if let Some(model_uniform) = self.models.get_mut(&animated_game_object.object_id) {
          model_uniform.value_mut().update(&animated_game_object.get_model_matrix());
          model_uniform.update(&ctx.queue);
        }
      }

      for game_object in game_objects {
        if let Some(model_uniform) = self.models.get_mut(&game_object.object_id) {
          model_uniform.value_mut().update(&game_object.get_model_matrix());
          model_uniform.update(&ctx.queue);  
        }

        // if let Some(outlined_uniform) = self.outlined_models.get_mut(&game_object.object_id) {
        //   outlined_uniform.value_mut().update(&game_object.get_model_matrix());
        //   outlined_uniform.update(&ctx.queue);  
        // }
      }
    }

    pub fn submit_animation_uniforms(&mut self, ctx: &WgpuContext, asset_manager: &mut AssetManager, delta_time: std::time::Duration) {
         if let Some(glb_model) = asset_manager.get_model_by_name_mut("glock") {
          glb_model.update(delta_time.as_secs_f32());
          let skin_uniform = self.animation.value_mut();

          if let Some(skin) = glb_model.skins.get(0) {
           for (i, joint) in skin.joints().iter().enumerate() {
            if i >= MAX_JOINTS_PER_MESH {
             break; 
            }

           // Convert cgmath::Matrix4 to [[f32; 4]; 4]
           skin_uniform.joint_matrices[i] = joint.matrix().into();
         }
       }
      }

      self.animation.update(&ctx.queue);
    }

    pub fn submit_light_uniforms(&mut self, ctx: &WgpuContext, dt: std::time::Duration) {
      let light_uniform = self.light.value_mut();
      let old_position: cgmath::Vector3<_> = light_uniform.position.into();
      light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into();
      self.light.update(&ctx.queue);
    }

    pub fn submit_camera_uniforms(&mut self, ctx: &WgpuContext, camera: &Camera) {
      self.camera.value_mut().update(&camera);
      self.camera.update(&ctx.queue);
    }
}