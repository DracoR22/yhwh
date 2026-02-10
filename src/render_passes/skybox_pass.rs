use crate::bind_group_manager::{BindGroupManager, TL};
use crate::common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT};
use crate::cube_map::CubeMap;
use crate::pipeline_builder::PipelineBuilder;
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

        let cubemap_buffers = [wgpu::VertexBufferLayout {
                array_stride:  3 * std::mem::size_of::<f32>() as wgpu::BufferAddress, 
                step_mode: wgpu::VertexStepMode::Vertex,                         
                attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                 }
              ]
        }];

        let pipeline = PipelineBuilder::new(
            "skybox pipeline",
            &[&cubemap.texture_bind_group_layout, &uniforms.camera.bind_group_layout],
            &cubemap_buffers,
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT]
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_cull_mode(wgpu::Face::Back)
        .build(&ctx.device);

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