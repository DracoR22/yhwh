use std::collections::HashMap;

use yhwh_core::common::constants::MAX_JOINTS_PER_MESH;
use crate::{game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_core::render_data_manager::RenderDataManager, texture::Texture, uniform::Uniform, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};
use cgmath::SquareMatrix;

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

pub struct AnimationPass {
    pipeline: wgpu::RenderPipeline,
    outline_pipeline: wgpu::RenderPipeline,
    uniforms: HashMap<usize, Uniform<AnimationUniform>>,
    created_player_uniform: bool,
}

impl AnimationPass {
    pub fn new(ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &GameData) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/animation.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("animation shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = game_data.asset_manager.get_phong_bind_group_layout().expect("No bind group layout for Phong!");

        let pipeline = PipelineBuilder::new(
            "animation pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout, // model
              &uniforms.bind_group_layout // animation
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

       let outline_shader_code = std::fs::read_to_string("res/shaders/solid_color_animation.wgsl").unwrap();
       let outline_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("solid color animation shader"),
            source: wgpu::ShaderSource::Wgsl(outline_shader_code.into()),
        });

        let outline_pipeline = PipelineBuilder::new(
              "animation outline pipeline",
              &[
                &uniforms.bind_group_layout, // camera
                &uniforms.bind_group_layout, // model
                &uniforms.bind_group_layout // animation
              ],
              &[Vertex::desc()],
              &outline_shader_module,
              [wgpu::TextureFormat::R8Unorm],
          )
          .build(&ctx.device);

        let animated_game_object = game_data.player.weapon_animated_game_object();
        uniforms.create_model(ctx, animated_game_object.id);

        Self {
          pipeline,
          outline_pipeline,
          uniforms: HashMap::new(),
          created_player_uniform: false
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &mut GameData, render_data: &mut RenderDataManager, output_texture: &Texture, output_depth: &Texture, output_outline: &Texture) {
       self.submit_animation_uniforms(ctx, render_data);
       
       let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("animation pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &output_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
             }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &output_depth.view,
            depth_ops: Some(wgpu::Operations {
              load: wgpu::LoadOp::Load,
              store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

        for render_item in render_data.render_items_animated().iter() {
          let model_uniform = uniforms.models.get(&render_item.object_id).unwrap();
          let animation_uniform = self.uniforms.get(&render_item.object_id).unwrap();
          let material = game_data.asset_manager.material_by_index(render_item.material_index).unwrap();
          let mesh = game_data.asset_manager.mesh_by_index(render_item.mesh_index).unwrap();

          pass.set_bind_group(2, &model_uniform.bind_group, &[]);
          pass.set_bind_group(3, &animation_uniform.bind_group, &[]);
          pass.set_bind_group(0, &material.bind_group, &[]);

          pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
          pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
          pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }

        self.render_player_weapons(game_data, uniforms, ctx, &mut pass);

        drop(pass);

        self.render_outlines(encoder, game_data, render_data, uniforms, output_outline);
    }
    
    fn render_outlines(&mut self, encoder: &mut wgpu::CommandEncoder, game_data: &GameData, render_data: &RenderDataManager, uniforms: &UniformManager, out_texture: &Texture) {
       let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("animation outline pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &out_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.outline_pipeline);
        pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);

        for render_item in render_data.render_items_outlined_animated().iter() {
          let model_uniform = uniforms.models.get(&render_item.object_id).unwrap();
          let animation_uniform = self.uniforms.get(&render_item.object_id).unwrap();
          let mesh = game_data.asset_manager.mesh_by_index(render_item.mesh_index).unwrap();

          pass.set_bind_group(1, &model_uniform.bind_group, &[]);
          pass.set_bind_group(2, &animation_uniform.bind_group, &[]);

          pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
          pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
          pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }

    fn render_player_weapons(&mut self, game_data: &mut GameData, uniforms: &mut UniformManager, ctx: &WgpuContext, pass: &mut wgpu::RenderPass) {
        let has_weapon = game_data.player.weapon_manager.desert_eagle_info.has;
        let animated_game_object = game_data.player.weapon_animated_game_object_mut();

        let Some(model_uniform) = uniforms.models.get_mut(&animated_game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", &animated_game_object.id);
            return
        };

        let Some(animation_uniform) = self.uniforms.get_mut(&animated_game_object.id) else {
            // println!("No animation bind group for object {:?}, skipping draw", &animated_game_object.id);
            // return
            self.create_uniform(ctx, animated_game_object.id);
            return
        };

        model_uniform.value_mut().update(&animated_game_object.model_matrix, &animated_game_object.tex_scale);
        model_uniform.update(&ctx.queue);

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

        pass.set_bind_group(2, &model_uniform.bind_group, &[]);
        pass.set_bind_group(3, &animation_uniform.bind_group, &[]);

        // if !has_weapon {
        //    return;
        // }

        if let Some(model) = game_data.asset_manager.model_by_name(&animated_game_object.get_model_name()) {
           for mesh in &model.meshes {
             let mesh_material_index = animated_game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
             let mesh_material = game_data.asset_manager.material_by_index(mesh_material_index);

             pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

             pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
             pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

             pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }
    }

    fn submit_animation_uniforms(&mut self, ctx: &WgpuContext, render_data: &mut RenderDataManager) {
      for anim_data in render_data.animated_render_data().iter() {
        if !self.uniforms.contains_key(&anim_data.object_id) {
          self.create_uniform(ctx, anim_data.object_id);
        }

        if let Some(animation_uniform) = self.uniforms.get_mut(&anim_data.object_id) {
          let skin_uniform = animation_uniform.value_mut();
          skin_uniform.joint_matrices = anim_data.joint_matrices.map(Into::into);

          animation_uniform.update(&ctx.queue);
        }
      }
    }

    fn create_uniform(&mut self, ctx: &WgpuContext, id: usize) {
      self.uniforms.insert(id, Uniform::new(AnimationUniform::new(), &ctx.device));
    }
}