use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, objects::game_object::GameObject, pipeline_manager::PipelineManager, texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct OutlinePass {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl OutlinePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/outline.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Outline_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outline_Pipeline_Layout"),
                bind_group_layouts: &[
                    &uniforms.camera.bind_group_layout,
                    &uniforms.bind_group_layout
                ],
                push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_stencil_pipeline(
            &ctx.device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            false
        ).unwrap();

        Self {
            pipeline_layout,
            pipeline,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, out_texture_view: &wgpu::TextureView, depth_texture_view: &wgpu::TextureView, uniforms: &UniformManager, game_objects: &Vec<GameObject>, asset_manager: &AssetManager) {
         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Outline_Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &out_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
              view: &depth_texture_view,
              depth_ops: None,
              stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
              }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);

        for game_object in game_objects.iter() {
           if game_object.is_selected {
            let Some(model_uniform) = uniforms.models.get(&game_object.object_id) else {
             println!("No model bind group for object {:?}, skipping draw", game_object.object_id);
             continue;
            };

            render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);

            if let Some(model) = asset_manager.get_model_by_name(&game_object.get_model_name()) {
             for mesh in model.meshes.iter() {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_stencil_reference(1);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
             }
            }
           }
        }
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext) {
      let shader_code = std::fs::read_to_string("res/shaders/outline.wgsl").unwrap();
      let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Outline_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
       });

      let new_pipeline = PipelineManager::create_stencil_pipeline(
        &ctx.device,
        &self.pipeline_layout,
        HDR_TEX_FORMAT,
        Some(DEPTH_TEXTURE_STENCIL_FORMAT),
        &shader_module,
        &[Vertex::desc()],
        false
      );

      match new_pipeline {
          Ok(res) => self.pipeline = res,
          Err(err) => println!("PostProcessPass::hotload_shader() error: {err}")
      }
    }
}