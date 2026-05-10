use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use crate::{bind_group_manager::{BindGroupManager, TL}, common::constants::SCR_RESOLUTION, pipeline_builder::PipelineBuilder, render_passes::geometry_pass::GBufferTextures, texture::{Texture, TextureBuilder}, uniform::Uniform, uniform_manager::UniformManager, wgpu_context::WgpuContext};

const KERNEL_SIZE: usize = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SSAOUniform {
    kernel_samples:[[f32; 4]; KERNEL_SIZE],
    kernel_size: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32
}

impl SSAOUniform {
    pub fn new() -> Self {
        Self {
            kernel_samples: generate_kernel(),
            kernel_size: KERNEL_SIZE as u32,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0
        }
    }
}


pub struct SSAOPass {
    uniform: Uniform<SSAOUniform>,
    color_pipeline: wgpu::RenderPipeline,
    color_bind_group_layout: wgpu::BindGroupLayout,
    color_bind_group: wgpu::BindGroup,
    pub color_texture: Texture,
    blur_pipeline: wgpu::RenderPipeline,
    blur_bind_group_layout: wgpu::BindGroupLayout,
    blur_bind_group: wgpu::BindGroup,
    pub blur_texture: Texture
}  

impl SSAOPass{
    pub fn new(ctx: &WgpuContext, gbuffer_textures: &GBufferTextures, uniforms: &UniformManager) -> Self {
        let noise = generate_noise();
        let raw_noise: Vec<u8> = bytemuck::cast_slice(&noise).to_vec();
        
        let noise_texture = TextureBuilder::from_raw(
            raw_noise,
            4, 
            4, 
            wgpu::TextureFormat::Rgba32Float
        )
        .build(&ctx.device, &ctx.queue);

        let color_shader_code = std::fs::read_to_string("res/shaders/ssao.wgsl").unwrap();
        let color_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao shader"),
            source: wgpu::ShaderSource::Wgsl(color_shader_code.into())
        });

        let color_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(
            &ctx.device, 
            [
                TL::Float, // position
                TL::Float, // normal
                TL::Float // noise
            ]
        );

        let color_bind_group = BindGroupManager::create_multi_texture_bind_group(
            &ctx.device,
            &color_bind_group_layout,
            &[
                &gbuffer_textures.world_position,
                &gbuffer_textures.normal,
                &noise_texture
            ]
        );

        let color_pipeline = PipelineBuilder::new(
            "ssao color pipeline",
            &[&color_bind_group_layout, &uniforms.bind_group_layout, &uniforms.bind_group_layout],
            &[],
            &color_shader_module,
            [wgpu::TextureFormat::R8Unorm]
        )
        .with_blend(wgpu::BlendState::REPLACE) // alpha blending wont work because we are returning a single float in the fragment shader
        .build(&ctx.device);

        let uniform = Uniform::new(SSAOUniform::new(), &ctx.device);

        let color_texture = Texture::create_fbo(
            &ctx.device,
            SCR_RESOLUTION,
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );

        let blur_texture = Texture::create_fbo(
            &ctx.device,
            SCR_RESOLUTION,
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );

        // blur pass
        let blur_shader_code = std::fs::read_to_string("res/shaders/ssao_blur.wgsl").unwrap();
        let blur_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao blur shader"),
            source: wgpu::ShaderSource::Wgsl(blur_shader_code.into())
        });

        let blur_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(
            &ctx.device,
            [
                TL::Float
            ]
        );
        let blur_bind_group = BindGroupManager::create_texture_bind_group(
            &ctx.device,
            &blur_bind_group_layout,
            &color_texture
        );

        let blur_pipeline = PipelineBuilder::new(
            "ssao blur pipeline",
            &[&blur_bind_group_layout],
            &[],
            &blur_shader_module,
            [wgpu::TextureFormat::R8Unorm]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        Self {
            uniform,
            color_pipeline,
            color_bind_group_layout,
            color_bind_group,
            color_texture,
            blur_pipeline,
            blur_bind_group_layout,
            blur_bind_group,
            blur_texture
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &UniformManager) {
        self.uniform.update(&ctx.queue);

        // color pass
        let mut color_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ssao color pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.color_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        color_pass.set_pipeline(&self.color_pipeline);
        color_pass.set_bind_group(0, &self.color_bind_group, &[]);
        color_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        color_pass.set_bind_group(2, &self.uniform.bind_group, &[]);
        color_pass.draw(0..3, 0..1);

        drop(color_pass);

        // blur pass
        let mut blur_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ssao blur pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.blur_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        blur_pass.set_pipeline(&self.blur_pipeline);
        blur_pass.set_bind_group(0, &self.blur_bind_group, &[]);
        blur_pass.draw(0..3, 0..1);
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
        let color_shader_code = std::fs::read_to_string("res/shaders/ssao.wgsl").unwrap();
        let color_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao shader"),
            source: wgpu::ShaderSource::Wgsl(color_shader_code.into())
        });

        let color_pipeline = PipelineBuilder::new(
            "ssao color pipeline",
            &[&self.color_bind_group_layout, &uniforms.bind_group_layout, &uniforms.bind_group_layout],
            &[],
            &color_shader_module,
            [wgpu::TextureFormat::R8Unorm]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        let blur_shader_code = std::fs::read_to_string("res/shaders/ssao_blur.wgsl").unwrap();
        let blur_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao blur shader"),
            source: wgpu::ShaderSource::Wgsl(blur_shader_code.into())
        });

        let blur_pipeline = PipelineBuilder::new(
            "ssao blur pipeline",
            &[&self.blur_bind_group_layout],
            &[],
            &blur_shader_module,
            [wgpu::TextureFormat::R8Unorm]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        self.color_pipeline = color_pipeline;
        self.blur_pipeline = blur_pipeline;
    }
}

fn generate_noise() -> Vec<[f32; 4]> {
    let mut rng = rand::thread_rng();
    let mut noise = Vec::<[f32; 4]>::new();
    for _ in 0..16 {
        noise.push([
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            0.0,
            0.0,
       ]);
    }

    return noise
}

fn generate_kernel() -> [[f32; 4]; KERNEL_SIZE] {
    let mut rng = rand::thread_rng();
    let mut kernel_samples = [[0.0; 4]; KERNEL_SIZE];

    for i in 0..KERNEL_SIZE {
        let mut sample = Vector3::<f32>::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(0.0..1.0),
        );
        sample = sample.normalize();
        sample *= rng.gen_range(0.0..1.0);
        let mut scale = i as f32 / KERNEL_SIZE as f32;
        scale = lerp(0.1, 1.0, scale * scale);
        sample *= scale;

        kernel_samples[i] = [
            sample.x,
            sample.y,
            sample.z,
            0.0
        ]
    }

    kernel_samples
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    return a + f * (b - a)
}