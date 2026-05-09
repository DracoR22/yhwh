use crate::{
    bind_group_manager::{BindGroupManager, TL},
    common::constants::SCR_RESOLUTION,
    game::game_data::GameData,
    pipeline_builder::PipelineBuilder,
    renderer_common::{QUAD_VERTEX_BUFFER_LAYOUT, QUAD_VERTICES},
    texture::{Texture},
    uniform_manager::UniformManager,
    vertex::Vertex,
    wgpu_context::WgpuContext,
};
use wgpu::util::DeviceExt;

pub struct EmissivePass {
    mask_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    mask_texture: Texture,
    ping_texture: Texture,
    pong_texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    ping_bind_group: wgpu::BindGroup,
    pong_bind_group: wgpu::BindGroup,
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

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&bind_group_layout, &uniforms.bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        let mask_pipeline = PipelineBuilder::new(
            "emissive mask pipeline",
            &[&uniforms.bind_group_layout, &uniforms.bind_group_layout],
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

        let blur_passes = 4;
        let sample_distance = 4.0;

        for i in 0..blur_passes {
            // if sample_distance < 8.0 {
            //     sample_distance = sample_distance + 1.0
            // }

            let direction = if horizontal { [1.0, 0.0] } else { [0.0, 1.0] };

            uniforms.blurs[i]
                .value_mut()
                .update(direction, sample_distance);
            uniforms.blurs[i].update(&ctx.queue);

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
            pass.set_bind_group(1, &uniforms.blurs[i].bind_group, &[]);
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

    pub fn render_mask(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData, out_color_texture: &Texture, out_depth_texture: &Texture,) {
        // extract emissive meshes
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
        pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);

        for game_object in game_data.scene.game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                println!(
                    "No model bind group for object {:?}, skipping draw",
                    game_object.id
                );
                continue;
            };

            if let Some(model) = game_data
                .asset_manager
                .get_model_by_name(&game_object.get_model_name())
            {
                pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                for mesh in model.meshes.iter() {
                    match game_object
                        .get_mesh_nodes()
                        .get_mesh_node_by_mesh_name(&mesh.name)
                    {
                        Some(mesh_node) => {
                            if mesh_node.emissive {
                                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                pass.set_index_buffer(
                                    mesh.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                            }
                        }
                        None => (),
                    }
                }
            }
        }
    }
}
