use crate::{asset_manager::AssetManager, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, objects::game_object::GameObject, pipeline_manager::PipelineManager, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct LightingPass {
    pipeline_layout: wgpu::PipelineLayout,
    stencil_pipeline: wgpu::RenderPipeline,
    pipeline: wgpu::RenderPipeline
}

impl LightingPass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = &asset_manager.get_material_by_name("Barrel_RED").unwrap().bind_group_layout;

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lighting_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniforms.camera.bind_group_layout,
                &uniforms.bind_group_layout,
                &uniforms.lights_ssbo.bind_group_layout
            ],
            push_constant_ranges: &[],
        });

         let pipeline = PipelineManager::create_pipeline(
            &ctx.device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            Some("Lighting_Pipeline")
        )
        .unwrap();

        let stencil_pipeline = PipelineManager::create_stencil_pipeline(
            &ctx.device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            true
        )
        .unwrap();

     Self {
        pipeline_layout,
        stencil_pipeline,
        pipeline
     }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, uniforms: &UniformManager, asset_manager: &AssetManager, game_objects: &Vec<GameObject>) {
        for game_object in game_objects.iter() {
          let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.id);
            continue;
          };

          if game_object.is_selected {
            render_pass.set_pipeline(&self.stencil_pipeline);
          } else {
            render_pass.set_pipeline(&self.pipeline);
          }

          render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
          render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);
          render_pass.set_bind_group(3, &uniforms.lights_ssbo.bind_group, &[]);

          if let Some(model) = asset_manager.get_model_by_name(&game_object.get_model_name()) {
            for mesh in &model.meshes {
                let mesh_material_index = game_object.get_mesh_nodes().get_mesh_material_index(&mesh.name);
                let mesh_material = asset_manager.get_material_by_index(mesh_material_index);

                render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_stencil_reference(1);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
          }
        }
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext) {
      let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
      let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
       });

      let pipeline = PipelineManager::create_pipeline(
            &ctx.device,
            &self.pipeline_layout,
            HDR_TEX_FORMAT,
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            Some("Lighting_Pipeline")
        );

      let stencil_pipeline = PipelineManager::create_stencil_pipeline(
            &ctx.device,
            &self.pipeline_layout,
            HDR_TEX_FORMAT,
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            true
        );

      match (pipeline, stencil_pipeline) {
        (Ok(pipeline), Ok(stencil_pipeline)) => {
          self.pipeline = pipeline;
          self.stencil_pipeline = stencil_pipeline;
        },

        (Err(e), _) | (_, Err(e)) => {
          println!("LightingPass::hotload_shader() error: {e}");
        }
      }
    }
}