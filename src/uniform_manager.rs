use std::collections::HashMap;

use cgmath::Matrix;
use cgmath::SquareMatrix;
use yhwh_core::common::constants::MAX_LIGHTS;

use crate::bind_group_manager::BindGroupManager;
use crate::game::game_data::GameData;
use crate::renderer_core::render_data_manager::RenderDataManager;
use crate::ssbo::SSBO;
use crate::texture::Texture;
use crate::{camera::{Camera}, uniform::Uniform, wgpu_context::WgpuContext};

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

pub struct UniformManager {
    pub camera: Uniform<CameraUniform>,
    pub models: HashMap<usize, Uniform<ModelUniform>>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub lights_ssbo: SSBO
}

impl UniformManager {
    pub fn new(ctx: &WgpuContext, game_data: &GameData, shadow_texture: &Texture) -> Self {
      let mut model_uniforms: HashMap<usize, Uniform<ModelUniform>> = HashMap::new();

      game_data.world.for_each_chunk(|chunk| {
        // for game_object in chunk.game_objects.iter() {
        //   model_uniforms.insert(game_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
        // }

        for animated_game_object in chunk.animated_game_objects.iter() {
          model_uniforms.insert(animated_game_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
          //animation_uniforms.insert(animated_game_object.id, Uniform::new(AnimationUniform::new(), &ctx.device));
        }

        // for door_object in chunk.door_objects.iter() {
        //   model_uniforms.insert(door_object.id, Uniform::new(ModelUniform::new(), &ctx.device));
        // }
      }); 

      // let mut shadow_cube_maps: Vec<Uniform<ShadowCubeMapUniform>> = Vec::new();
      // for _ in 0..MAX_LIGHTS {
      //    for _ in 0..6 {
      //      shadow_cube_maps.push(Uniform::new(ShadowCubeMapUniform::new(), &ctx.device));
      //    }
      // }

      let lights_ssbo = SSBO::new((std::mem::size_of::<LightUniform>() * MAX_LIGHTS as usize) as u64, &ctx.device, shadow_texture);

      // let blur_passes = 8;
      // let mut blurs = Vec::new();
      // for _ in 0..blur_passes {
      //   blurs.push(Uniform::new(BlurUniform::new(), &ctx.device));
      // }

      let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
        &ctx.device,
        wgpu::ShaderStages::VERTEX_FRAGMENT,
        Some("uniform bind group layout"));

      Self {
        models: model_uniforms,
        //animations: animation_uniforms,
        camera: Uniform::new(CameraUniform::new(), &ctx.device),
        // blurs,
        bind_group_layout,
        lights_ssbo,
        // shadow_cube_maps
      }
    }

    pub fn create_model(&mut self, ctx: &WgpuContext, id: usize) {
      self.models.insert(id, Uniform::new(ModelUniform::new(), &ctx.device));
    }

    pub fn submit_model_uniforms(&mut self, ctx: &WgpuContext, render_data: &RenderDataManager) {
      for model_intance in render_data.model_instances().iter() {
        if !self.models.contains_key(&model_intance.object_id) {
          self.create_model(&ctx, model_intance.object_id);
        }

        if let Some(model_uniform) = self.models.get_mut(&model_intance.object_id) {
          model_uniform.value_mut().update(&model_intance.model_matrix, &model_intance.texture_scale);
          model_uniform.update(&ctx.queue);  
        }
      }
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