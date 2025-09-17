use crate::{asset_manager::AssetManager, common::constants::HDR_TEX_FORMAT, pipeline_manager::PipelineManager, uniform_types::WgpuUniforms, vertex::Vertex, wgpu_context::WgpuContext};

pub struct LightingPass {
    pipeline: wgpu::RenderPipeline
}

impl LightingPass {
    pub fn new(ctx: &WgpuContext, uniforms: &WgpuUniforms, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = &asset_manager.get_material_by_name("barrel_BLUE").unwrap().bind_group_layout;

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lighting_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniforms.camera.bind_group_layout,
                &uniforms.models[0].bind_group_layout,
                &uniforms.light.bind_group_layout
            ],
            push_constant_ranges: &[],
        });

          let pipeline = PipelineManager::create_pipeline(
            &ctx.device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(wgpu::TextureFormat::Depth32Float),
            &shader_module,
            &[Vertex::desc()],
            Some("lighting_pipeline"))
        .unwrap();

     Self {
         pipeline
     }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, uniforms: &WgpuUniforms, asset_manager: &AssetManager) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &asset_manager.get_material_by_name("barrel_BLUE").unwrap().bind_group, &[]);
        render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(2, &uniforms.models[2].bind_group, &[]);
        render_pass.set_bind_group(3, &uniforms.light.bind_group, &[]);

        if let Some(model) = asset_manager.get_model_by_name("Barrel") {
            for mesh in &model.meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &asset_manager.get_material_by_name("barrel_BLUE").unwrap().bind_group, &[]);
        render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(2, &uniforms.models[1].bind_group, &[]);
        render_pass.set_bind_group(3, &uniforms.light.bind_group, &[]);

        if let Some(plane_model) = asset_manager.get_model_by_name("Plane") { 
           for mesh in &plane_model.meshes {
             render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
             render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
             render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }
    }
}