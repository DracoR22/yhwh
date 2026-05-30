use std::collections::HashMap;

use cgmath::Matrix;
use cgmath::SquareMatrix;
use cgmath::Vector2;

use crate::asset_manager::AssetManager;
use crate::bind_group_manager::BindGroupManager;
use crate::common::constants::MAX_LIGHTS;
use crate::game::game_data::GameData;
use crate::scene::Scene;
use crate::ssbo::SSBO;
use crate::texture::Texture;
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
    pub normal_matrix: [[f32; 4]; 4],
    pub tex_scale: [f32; 2],
    _padding_0: [f32; 2]
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
              [0.0,        0.0,        0.0,        1.0],

          ],
        tex_scale: [1.0, 1.0],
        _padding_0: [0.0, 0.0],
      }
    }

    pub fn update(&mut self, matrix: &cgmath::Matrix4<f32>, tex_scale: &cgmath::Vector2<f32>) {
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
            [0.0,             0.0,             0.0,          1.0],

        ];
       } else {
        //println!("NORMAL MATRIX FUCKED UP");
        self.normal_matrix = [
          [1.0, 0.0, 0.0, 0.0],
          [0.0, 1.0, 0.0, 0.0],
          [0.0, 0.0, 1.0, 0.0],
          [0.0, 0.0, 0.0, 1.0],

        ];
       }

       self.tex_scale = (*tex_scale).into();
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
   pub _pad0: u32,
   pub color: [f32; 3],
   pub _pad1: u32,
   pub strength: f32,
   pub radius: f32,
   pub _pad2: u32,
   pub _pad3: u32
}

impl LightUniform {
    pub fn new() -> Self {
        Self {
          position: [2.0, 2.0, 2.0],
          _pad0: 0,
          color: [1.0, 1.0, 1.0],
          _pad1: 0,
          strength: 50.0,
          radius: 5.0,
          _pad2: 0,
          _pad3: 0
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniform {
    pub direction: [f32; 2],
    pub sample_distance: f32,
    _pad: f32
}

impl BlurUniform {
  pub fn new() -> Self {
    Self {
      direction: [1.0, 0.0],
      sample_distance: 1.0,
      _pad: 0.0
    }
  }

  pub fn update(&mut self, direction: [f32; 2], sample_distance: f32) {
    self.direction[0] = direction[0];
    self.direction[1] = direction[1];
    self.sample_distance = sample_distance;
  }
}

pub struct UniformManager {
    pub camera: Uniform<CameraUniform>,
    pub models: HashMap<usize, Uniform<ModelUniform>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub animations: HashMap<usize, Uniform<AnimationUniform>>,
    pub blurs: Vec<Uniform<BlurUniform>>,
    pub lights_ssbo: SSBO
}

impl UniformManager {
    pub fn new(ctx: &WgpuContext, game_data: &GameData, shadow_texture: &Texture) -> Self {
      let mut model_uniforms: HashMap<usize, Uniform<ModelUniform>> = HashMap::new();
      let mut animation_uniforms: HashMap<usize, Uniform<AnimationUniform>> = HashMap::new();

      game_data.world.for_each_chunk(|chunk| {
        for game_object in chunk.game_objects.iter() {
          model_uniforms.insert(game_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
        }

        for animated_game_object in chunk.animated_game_objects.iter() {
          model_uniforms.insert(animated_game_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
          animation_uniforms.insert(animated_game_object.id, Uniform::new(AnimationUniform::new(), &ctx.device));
        }

        for door_object in chunk.door_objects.iter() {
          model_uniforms.insert(door_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
        }
      }); 

      // let mut shadow_cube_maps: Vec<Uniform<ShadowCubeMapUniform>> = Vec::new();
      // for _ in 0..MAX_LIGHTS {
      //    for _ in 0..6 {
      //      shadow_cube_maps.push(Uniform::new(ShadowCubeMapUniform::new(), &ctx.device));
      //    }
      // }

      let lights_ssbo = SSBO::new((std::mem::size_of::<LightUniform>() * MAX_LIGHTS as usize) as u64, &ctx.device, shadow_texture);

      let blur_passes = 4;
      let mut blurs = Vec::new();
      for _ in 0..blur_passes {
        blurs.push(Uniform::new(BlurUniform::new(), &ctx.device));
      }

      let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
        &ctx.device,
        wgpu::ShaderStages::VERTEX_FRAGMENT,
        Some("uniform bind group layout"));

      Self {
        models: model_uniforms,
        animations: animation_uniforms,
        camera: Uniform::new(CameraUniform::new(), &ctx.device),
        blurs,
        bind_group_layout,
        lights_ssbo,
        // shadow_cube_maps
      }
    }

    pub fn create_model(&mut self, ctx: &WgpuContext, id: usize) {
      self.models.insert(id, Uniform::new(ModelUniform::new(), &ctx.device));
    }

    pub fn submit_model_uniforms(&mut self, ctx: &WgpuContext, game_data: &GameData) {
      game_data.world.for_each_chunk(|chunk| {
        for animated_game_object in chunk.animated_game_objects.iter() {
          if !self.models.contains_key(&animated_game_object.id) {
           self.create_model(&ctx, animated_game_object.id);
          }
          if let Some(model_uniform) = self.models.get_mut(&animated_game_object.id) {
            model_uniform.value_mut().update(&animated_game_object.get_model_matrix(), &animated_game_object.tex_scale);
            model_uniform.update(&ctx.queue);
          }
        }

        for game_object in chunk.game_objects.iter() {
          if !self.models.contains_key(&game_object.id) {
            self.create_model(&ctx, game_object.id);
          }

          if let Some(model_uniform) = self.models.get_mut(&game_object.id) {
            model_uniform.value_mut().update(&game_object.get_model_matrix(), &game_object.tex_scale);
            model_uniform.update(&ctx.queue);  
          }
        }

        let door_tex_scale = Vector2::new(1.0, 1.0);
        for door_object in chunk.door_objects.iter() {
          if !self.models.contains_key(&door_object.id) {
            self.create_model(&ctx, door_object.id);
          }

          if let Some(model_uniform) = self.models.get_mut(&door_object.id) {
            model_uniform.value_mut().update(&door_object.get_model_matrix(), &door_tex_scale);
            model_uniform.update(&ctx.queue);  
          }
        }
      });
    }

    pub fn submit_animation_uniforms(&mut self, ctx: &WgpuContext, game_data: &mut GameData) {
        // TODO! CREATE A NEW UNIFORM IN RUNTIME LIKE WITH MODELS
      game_data.world.for_each_chunk_mut(|chunk| {
          for animated_game_object in chunk.animated_game_objects.iter_mut() {
            if let Some(animation_uniform) = self.animations.get_mut(&animated_game_object.id) {
              animated_game_object.update(&game_data.asset_manager, game_data.delta_time.as_secs_f32());
              let skin_uniform = animation_uniform.value_mut();

              if let Some(skin) = animated_game_object.skins.get(0) {
                  for (i, joint) in skin.joints().iter().enumerate() {
                        if i >= MAX_JOINTS_PER_MESH {
                           break; 
                        }

                      skin_uniform.joint_matrices[i] = joint.matrix().into();
                  }
              }

              animation_uniform.update(&ctx.queue);
           }
        }
      });

      // self.animations.update(&ctx.queue);
    }

    pub fn create_animation(&mut self, ctx: &WgpuContext, id: usize) {
      self.animations.insert(id, Uniform::new(AnimationUniform::new(), &ctx.device));
    }

    pub fn submit_light_uniforms(&mut self, ctx: &WgpuContext, game_data: &GameData, shadow_texture: &Texture) {
      let light_count = game_data.world.light_count();
      let mut light_uniforms: Vec<LightUniform> = Vec::with_capacity(light_count);

      game_data.world.for_each_chunk(|chunk| {
         for light in chunk.lights.iter() {
            let light_uniform = LightUniform {
              position: light.position.into(),
              _pad0: 0,
              color: light.color.into(),
              _pad1: 0,
              strength: light.strength,
              radius: light.radius,
              _pad2: 0,
              _pad3: 0
            };

            light_uniforms.push(light_uniform);
          }
      });

      self.lights_ssbo.update(&ctx, (light_uniforms.len() * std::mem::size_of::<LightUniform>()) as u64, &light_uniforms, shadow_texture);
    }

    pub fn submit_camera_uniforms(&mut self, ctx: &WgpuContext, camera: &Camera) {
      self.camera.value_mut().update(&camera);
      self.camera.update(&ctx.queue);
    }
}