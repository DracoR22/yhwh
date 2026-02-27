use crate::{asset_manager::AssetManager, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, objects::game_object::GameObject, pipeline_builder::PipelineBuilder, pipeline_manager::PipelineManager, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct LightingPass {
    stencil_pipeline: wgpu::RenderPipeline,
    pipeline: wgpu::RenderPipeline,
    texture_bg_layout: wgpu::BindGroupLayout
}

impl LightingPass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = &asset_manager.get_material_by_name("Barrel_RED").unwrap().bind_group_layout;

         let pipeline = PipelineBuilder::new(
            "lighting pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);

        let write_stencil = true;
        let stencil_pipeline = PipelineBuilder::new(
            "lighting stencil pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .with_stencil_state(write_stencil)
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

     Self {
        stencil_pipeline,
        pipeline,
        texture_bg_layout: texture_bind_group_layout.clone()
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
                if game_object.get_mesh_nodes().get_mesh_rendering_info_by_mesh_name(&mesh.name).emissive {
                  continue;
                }

                let mesh_material_index = game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
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

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
      let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
      let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
       });

       let pipeline = PipelineBuilder::new(
            "lighting pipeline",
            &[
              &self.texture_bg_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);

      let write_stencil = true;
      let stencil_pipeline = PipelineBuilder::new(
            "lighting stencil pipeline",
            &[
              &self.texture_bg_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .with_stencil_state(write_stencil)
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

      self.pipeline = pipeline;
      self.stencil_pipeline = stencil_pipeline;
    }
}