use crate::bind_group_manager::{BindGroupManager, TL};
use crate::cube_map::CubeMap;
use crate::pipeline_manager::PipelineManager;
use crate::renderer_common::SKYBOX_VERTICES;
use crate::uniform_manager::UniformManager;
use crate::{asset_manager::AssetManager, wgpu_context::WgpuContext};
use crate::texture::{Texture, TextureHelpers};

pub struct SkyboxPass {
    pipeline: wgpu::RenderPipeline,
    cubemap: CubeMap
}

impl SkyboxPass {
    pub fn new(ctx: &WgpuContext, asset_manager: &AssetManager, uniforms: &UniformManager) -> Self {
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cubemap_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/cube_map.wgsl").into()),
        });
        
        let flipped_right = asset_manager.get_texture_by_name("SkyRight.jpg").unwrap().flip_horizontal();
        let cubemap = CubeMap::new(&ctx.device, &ctx.queue, asset_manager.get_texture_by_name("SkyRight.jpg").unwrap().dimensions, [
            &flipped_right.pixel_data,
            &asset_manager.get_texture_by_name("SkyLeft.jpg").unwrap().pixel_data,
            &asset_manager.get_texture_by_name("SkyTop.jpg").unwrap().pixel_data,
            &asset_manager.get_texture_by_name("SkyBottom.jpg").unwrap().pixel_data,
            &asset_manager.get_texture_by_name("SkyFront.jpg").unwrap().pixel_data,
            &asset_manager.get_texture_by_name("SkyBack.jpg").unwrap().pixel_data,
        ]);

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Skybox_Map_Pipeline_Layout"),
                bind_group_layouts: &[&cubemap.texture_bind_group_layout, &uniforms.camera.bind_group_layout],
                push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_cubemap_pipeline(
            &ctx.device,
            &pipeline_layout,
            wgpu::TextureFormat::Rgba16Float,
            &shader_module)
        .unwrap();

        Self {
            cubemap,
            pipeline
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, uniforms: &UniformManager, ) {
       render_pass.set_pipeline(&self.pipeline);

       render_pass.set_bind_group(0, &self.cubemap.texture_bind_group, &[]);
       render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

       render_pass.set_vertex_buffer(0, self.cubemap.vertex_buffer.slice(..));
       render_pass.draw(0..(SKYBOX_VERTICES.len() / 3) as u32, 0..1);
    }
}