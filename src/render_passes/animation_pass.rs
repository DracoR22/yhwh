use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, bind_group_manager::BindGroupManager, common::constants::HDR_TEX_FORMAT, model::Model, pipeline_manager::PipelineManager, uniform_manager::UniformManager, vertex::Vertex};

pub struct AnimationPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl AnimationPass {
    pub fn new(device: &wgpu::Device, wgpu_uniforms: &UniformManager) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/animation.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("animation_pipeline_layout"),
            bind_group_layouts: &[
                &wgpu_uniforms.camera.bind_group_layout,
                &wgpu_uniforms.bind_group_layout,
                &wgpu_uniforms.animation.bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_pipeline(
            &device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(wgpu::TextureFormat::Depth32Float),
            &shader_module,
            &[Vertex::desc()],
            Some("animation_pipeline"))
        .unwrap();

        Self {
          bind_group: wgpu_uniforms.animation.bind_group.clone(),
          pipeline
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, uniforms: &UniformManager,  asset_manager: &AssetManager, object_id: usize) {
        let Some(model_uniform) = uniforms.models.get(&object_id) else {
            println!("No model bind group for object {:?}, skipping draw", object_id);
            return
        };

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);
        render_pass.set_bind_group(2, &uniforms.animation.bind_group, &[]);

       if let Some(model) = asset_manager.get_model_by_name("glock") {
        for mesh in &model.meshes {
         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
       }
    }
}