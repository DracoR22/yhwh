use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, bind_group_manager::BindGroupManager, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, model::Model, objects::animated_game_object::AnimatedGameObject, pipeline_manager::PipelineManager, uniform_manager::UniformManager, vertex::Vertex};

pub struct AnimationPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl AnimationPass {
    pub fn new(device: &wgpu::Device, wgpu_uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/animation.wgsl").into()),
        });

        let texture_bind_group_layout = asset_manager.get_phong_bind_group_layout().expect("No bind group layout for Phong!");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("animation_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
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
            Some(DEPTH_TEXTURE_STENCIL_FORMAT),
            &shader_module,
            &[Vertex::desc()],
            Some("animation_pipeline"))
        .unwrap();

        Self {
          bind_group: wgpu_uniforms.animation.bind_group.clone(),
          pipeline
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, uniforms: &UniformManager, asset_manager: &AssetManager, animated_game_objects: &Vec<AnimatedGameObject>) {
        render_pass.set_pipeline(&self.pipeline);

        for animated_game_object in animated_game_objects.iter() {
          let Some(model_uniform) = uniforms.models.get(&animated_game_object.object_id) else {
            println!("No model bind group for object {:?}, skipping draw", &animated_game_object.object_id);
            return
          };
          render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
          render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);
          render_pass.set_bind_group(3, &uniforms.animation.bind_group, &[]);

          if let Some(model) = asset_manager.get_model_by_name(&animated_game_object.get_model_name()) {
           for mesh in &model.meshes {
             let mesh_material_index = animated_game_object.get_mesh_nodes().get_mesh_material_index(&mesh.name);
             let mesh_material = asset_manager.get_material_by_index(mesh_material_index);

             render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

             render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
             render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
             render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
          }
        }
    }
}