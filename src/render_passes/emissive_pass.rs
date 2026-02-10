use crate::{asset_manager::AssetManager, common::constants::{DEPTH_TEXTURE_FORMAT, DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, engine::GameData, pipeline_builder::PipelineBuilder, pipeline_manager::PipelineManager, texture::{self, Texture}, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct EmissivePass {
    pipeline: wgpu::RenderPipeline
}

impl EmissivePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/bloom.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        // let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("bloom_pipeline_layout"),
        //     bind_group_layouts: &[
        //         //&texture_bind_group_layout,
        //         &uniform_manager.camera.bind_group_layout,
        //         &uniform_manager.bind_group_layout,
        //     ],
        //     push_constant_ranges: &[],
        // });

       //  let texture_bind_group_layout = asset_manager.get_phong_bind_group_layout().expect("No bind group layout for Phong!");

        let pipeline = PipelineBuilder::new(
            "emissive pipeline",
            &[
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);

        Self {
            pipeline
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, game_data: &GameData, uniforms: &UniformManager, hdr_texture_view: &wgpu::TextureView, emissive_texture_view: &wgpu::TextureView, depth_texture_view: &wgpu::TextureView) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);
        
        for game_object in game_data.scene.game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.id);
            continue;
          };

          if game_object.get_model_name() == "candles" {
            if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
                render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                for mesh in model.meshes.iter() {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                }
            }
          }
        }
    }
}