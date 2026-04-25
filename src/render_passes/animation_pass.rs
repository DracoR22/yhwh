use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, engine::GameData, model::Model, objects::animated_game_object::AnimatedGameObject, pipeline_builder::PipelineBuilder, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct AnimationPass {
    pipeline: wgpu::RenderPipeline,
}

impl AnimationPass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/animation.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = asset_manager.get_phong_bind_group_layout().expect("No bind group layout for Phong!");

        let pipeline = PipelineBuilder::new(
            "animation pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.animation.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);


        Self {
          pipeline
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData, output_texture: &Texture, output_depth: &Texture) {
       let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        
        render_pass.set_pipeline(&self.pipeline);

        for animated_game_object in game_data.scene.animated_game_objects.iter() {
          let Some(model_uniform) = uniforms.models.get(&animated_game_object.object_id) else {
            println!("No model bind group for object {:?}, skipping draw", &animated_game_object.object_id);
            return
          };
          render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
          render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);
          render_pass.set_bind_group(3, &uniforms.animation.bind_group, &[]);

          if let Some(model) = game_data.asset_manager.get_model_by_name(&animated_game_object.get_model_name()) {
           for mesh in &model.meshes {
             let mesh_material_index = animated_game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
             let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

             render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

             render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
             render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
             render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
          }
        }
    }
}