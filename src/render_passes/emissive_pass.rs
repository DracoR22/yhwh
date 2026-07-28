use std::collections::HashMap;

use crate::{
    bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_common::{QUAD_VERTEX_BUFFER_LAYOUT, QUAD_VERTICES}, renderer_core::render_data_manager::RenderDataManager, texture::Texture, uniform::Uniform, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext
};
use cgmath::Vector4;
use wgpu::util::DeviceExt;
use yhwh_core::common::constants::{BLUR_PASSES, SCR_RESOLUTION};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurUniform {
    pub direction: [f32; 2],
    pub sample_distance: f32,
    _pad: f32
}

impl BlurUniform {
  pub fn new() -> Self {
    Self {
      direction: [1.0, 0.0],
      sample_distance: 1.0,
      _pad: 0.0
    }
  }

  pub fn update(&mut self, direction: [f32; 2], sample_distance: f32) {
    self.direction[0] = direction[0];
    self.direction[1] = direction[1];
    self.sample_distance = sample_distance;
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaskUniform {
    pub color: [f32; 4],
    pub emissive: [f32; 4],
}

impl MaskUniform {
    pub fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: [1.0, 0.5, 0.0, 1.0]
        }
    }

    pub fn update(&mut self, color: Vector4<f32>, emissive: Vector4<f32>) {
        self.color = color.into();
        self.emissive = emissive.into();
    }
}

pub struct EmissivePass {
    pub mask_texture: Texture,
    pub ping_texture: Texture,
    pub pong_texture: Texture,
    blur_uniforms: Vec<Uniform<BlurUniform>>,
    blur_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    ping_bind_group: wgpu::BindGroup,
    pong_bind_group: wgpu::BindGroup,
    mask_uniforms: HashMap<usize, Uniform<MaskUniform>>,
    mask_pipeline: wgpu::RenderPipeline,
    mask_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    final_is_ping: bool,
}

impl EmissivePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/blur.wgsl").unwrap();
        let shader_module = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("blur shader"),
                source: wgpu::ShaderSource::Wgsl(shader_code.into()),
            });

        let mask_shader_code = std::fs::read_to_string("res/shaders/bloom.wgsl").unwrap();
        let mask_shader_module = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("mask bloom shader"),
                source: wgpu::ShaderSource::Wgsl(mask_shader_code.into()),
            });

        let vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let width = 1920 / 2;
        let height = 1080 / 2;

        let mask_texture = Texture::create_fbo(
            &ctx.device,
            SCR_RESOLUTION,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let ping_texture = Texture::create_fbo(
            &ctx.device,
            (width, height),
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let pong_texture = Texture::create_fbo(
            &ctx.device,
            (width, height),
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float]);

        let ping_bind_group = BindGroupManager::create_texture_bind_group(
            &ctx.device,
            &bind_group_layout,
            &ping_texture,
        );
        let pong_bind_group = BindGroupManager::create_texture_bind_group(
            &ctx.device,
            &bind_group_layout,
            &pong_texture,
        );
        let mask_bind_group = BindGroupManager::create_texture_bind_group(
            &ctx.device,
            &bind_group_layout,
            &mask_texture,
        );

        let mut blur_uniforms = Vec::new();
        for _ in 0..BLUR_PASSES {
            blur_uniforms.push(Uniform::new(BlurUniform::new(), &ctx.device));
        }

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&bind_group_layout, &uniforms.bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        let mask_uniform = Uniform::new(MaskUniform::new(), &ctx.device);

        let mask_pipeline = PipelineBuilder::new(
            "emissive mask pipeline",
            &[
              &uniforms.bind_group_layout, // mask
              &uniforms.bind_group_layout, // camera
              &uniforms.bind_group_layout // model
            ],
            &[Vertex::desc()],
            &mask_shader_module,
            [
                wgpu::TextureFormat::Rgba16Float, // color
                wgpu::TextureFormat::Rgba16Float, // emissive color
            ],
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        Self {
            mask_pipeline,
            mask_uniforms: HashMap::new(),
            blur_uniforms,
            blur_pipeline,
            mask_texture,
            ping_texture,
            pong_texture,
            bind_group_layout,
            ping_bind_group,
            pong_bind_group,
            mask_bind_group,
            vertex_buffer,
            final_is_ping: true,
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager) {
        let mut horizontal = true;
        let mut first_iteration = true;

        let sample_distance = 1.0;

        for i in 0..BLUR_PASSES {
            // if sample_distance < 8.0 {
            //     sample_distance = sample_distance + 1.0
            // }

            let direction = if horizontal { [1.0, 0.0] } else { [0.0, 1.0] };

            self.blur_uniforms[i].value_mut().update(direction, sample_distance);
            self.blur_uniforms[i].update(&ctx.queue);

            let target_texture = if horizontal {
                &self.pong_texture
            } else {
                &self.ping_texture
            };

            let target_bind_group = if first_iteration {
                &self.mask_bind_group
            } else {
                if horizontal {
                    &self.ping_bind_group
                } else {
                    &self.pong_bind_group
                }
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.blur_pipeline);
            pass.set_bind_group(0, target_bind_group, &[]);
            pass.set_bind_group(1, &self.blur_uniforms[i].bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);

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
            label: Some("blur shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&self.bind_group_layout, &uniforms.bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .build(&device);

        self.blur_pipeline = blur_pipeline;
    }

    pub fn get_final_texture(&self) -> &Texture {
        if self.final_is_ping {
            &self.ping_texture
        } else {
            &self.pong_texture
        }
    }

    pub fn render_mask(&mut self, ctx: &WgpuContext, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData, render_data: &RenderDataManager, out_color_texture: &Texture, out_depth_texture: &Texture) {
        // self.mask_uniform.update(&ctx.queue);
        // self.mask_uniform.value_mut().update(color, emissive);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("emissive mask pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &out_color_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.mask_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &out_depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Default::default(),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.mask_pipeline);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

        for render_item in render_data.render_items_emissive().iter() {
            let model_uniform = uniforms.models.get(&render_item.object_id).unwrap();
            let mesh = game_data.asset_manager.mesh_by_index(render_item.mesh_index).unwrap();

            if !self.mask_uniforms.contains_key(&render_item.object_id) {
                self.create_mask_uniform(ctx, render_item.object_id);
            }

            if let Some(mask_uniform) = self.mask_uniforms.get_mut(&render_item.object_id) {
                mask_uniform.value_mut().update(Vector4::new(1.0, 1.0, 1.0, 1.0), render_item.emissive_color);
                mask_uniform.update(&ctx.queue);

                pass.set_bind_group(0, &mask_uniform.bind_group, &[]);
            }

            pass.set_bind_group(2, &model_uniform.bind_group, &[]);

            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..),  wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }

    pub fn create_mask_uniform(&mut self, ctx: &WgpuContext, id: usize) {
        self.mask_uniforms.insert(id, Uniform::new(MaskUniform::new(), &ctx.device));
    }
}
