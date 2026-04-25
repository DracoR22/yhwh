use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::{DEPTH_TEXTURE_FORMAT, DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, engine::GameData, pipeline_builder::PipelineBuilder, renderer_common::{QUAD_VERTEX_BUFFER_LAYOUT, QUAD_VERTICES}, texture::{self, Texture}, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};
use wgpu::util::DeviceExt;

pub struct EmissivePass {
    pipeline: wgpu::RenderPipeline,
    ping_texture: Texture,
    pong_texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    ping_bind_group: wgpu::BindGroup,
    pong_bind_group: wgpu::BindGroup,
    source_bind_group: wgpu::BindGroup,   
    quad_vertex_buffer: wgpu::Buffer,
    final_is_ping: bool,
}

impl EmissivePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, source_texture: &Texture) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/blur.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blur shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let quad_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("quad vertex buffer"),
          contents: bytemuck::cast_slice(&QUAD_VERTICES),
          usage: wgpu::BufferUsages::VERTEX,
        });

        let width = 1920 / 2;
        let height = 1080 / 2;

        let ping_texture = Texture::create_fbo(&ctx.device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let pong_texture = Texture::create_fbo(&ctx.device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float]).unwrap();

        let ping_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &ping_texture).unwrap();
        let pong_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &pong_texture).unwrap();
        let source_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &source_texture).unwrap();

        let pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&bind_group_layout, &uniforms.bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [HDR_TEX_FORMAT]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        Self {
            pipeline,
            ping_texture, 
            pong_texture,
            bind_group_layout,
            ping_bind_group,
            pong_bind_group,
            source_bind_group,
            quad_vertex_buffer,
            final_is_ping: true
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager) {
        let mut horizontal = true;
        let mut first_iteration = true;

        let blur_passes = 4;
        let mut sample_distance = 4.0;

        for i in 0..blur_passes {
            if sample_distance < 8.0 {
                sample_distance = sample_distance + 1.0
            }

            let direction = if horizontal {
                [1.0, 0.0]
            } else {
                [0.0, 1.0]
            };

            uniforms.blurs[i].value_mut().update(direction, sample_distance);
            uniforms.blurs[i].update(&ctx.queue);
    
            let target_texture = if horizontal {
                &self.pong_texture
            } else {
                &self.ping_texture
            };

            let target_bind_group = if first_iteration {
                &self.source_bind_group
            } else {
                if horizontal {
                    &self.ping_bind_group
                } else {
                    &self.pong_bind_group
                }
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("blur pass {}", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, target_bind_group, &[]);
            render_pass.set_bind_group(1, &uniforms.blurs[i].bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));  
            render_pass.draw(0..6, 0..1);    

            self.final_is_ping = !horizontal;

            horizontal = !horizontal;

            if first_iteration {
                first_iteration = false;
            } 
        }
    }

    pub fn hotload_shader(&mut self, device: &wgpu::Device, uniforms: &UniformManager) {
        let shader_code = std::fs::read_to_string("res/shaders/blur.wgsl").unwrap();
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&self.bind_group_layout, &uniforms.bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [HDR_TEX_FORMAT]
        )
        .build(&device);

        self.pipeline = blur_pipeline;
    }

    pub fn get_final_texture(&self) -> &Texture {
        if self.final_is_ping {
        &self.ping_texture
    } else {
        &self.pong_texture
    }
    }
}