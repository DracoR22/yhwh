use crate::{bind_group_manager::{BindGroupManager, TL}, pipeline_builder::PipelineBuilder, render_passes::geometry_pass::GBufferTextures, renderer_common::{QUAD_VERTEX_BUFFER_LAYOUT, QUAD_VERTICES}, texture::Texture, uniform_manager::UniformManager, wgpu_context::WgpuContext};
use wgpu::util::DeviceExt;
use yhwh_core::common::constants::SCR_RESOLUTION;

pub struct LightingPass {
    pub texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer
}

impl LightingPass {
    pub fn new(ctx: &WgpuContext, gbuffer_textures: &GBufferTextures, ssao_texture: &Texture, uniforms: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/d_lighting.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lighting shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(
            &ctx.device,
            [
                TL::Float, // basecolor
                TL::Float, // normal
                TL::Float, // rma
                TL::Float ,// world position
                TL::Float // ssao
            ]
        );

        let bind_group = BindGroupManager::create_multi_texture_bind_group(
            &ctx.device,
            &bind_group_layout,
            &[
               &gbuffer_textures.base_color,
               &gbuffer_textures.normal,
               &gbuffer_textures.rma,
               &gbuffer_textures.world_position,
               &ssao_texture
            ],
        );

        let pipeline = PipelineBuilder::new(
            "lighting pipeline",
            &[
                &bind_group_layout, // textures
                &uniforms.bind_group_layout, // camera
                &uniforms.lights_ssbo.bind_group_layout // lights + shadows
            ],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("lighting vertex buffer"),
          contents: bytemuck::cast_slice(&QUAD_VERTICES),
          usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            texture,
            bind_group_layout,
            bind_group,
            pipeline,
            vertex_buffer
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &UniformManager, gbuffer_textures: &GBufferTextures) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("lighting pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        pass.set_bind_group(2, &uniforms.lights_ssbo.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));  
        pass.draw(0..6, 0..1); 
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
        let shader_code = std::fs::read_to_string("res/shaders/d_lighting.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lighting shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline = PipelineBuilder::new(
            "lighting pipeline",
            &[
             &self.bind_group_layout, // textures
             &uniforms.bind_group_layout, // camera
             &uniforms.lights_ssbo.bind_group_layout // lights + shadows
            ],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
         )
         .build(&ctx.device);

         self.pipeline = pipeline;
    }
}

