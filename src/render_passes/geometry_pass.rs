use yhwh_core::common::constants::SCR_RESOLUTION;

use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_core::render_data_manager::RenderDataManager, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct GBufferTextures {
    pub base_color: Texture,
    pub normal: Texture,
    pub rma: Texture,
    pub world_position: Texture,
    pub depth: Texture, 
}

pub struct GeometryPass {
    pub textures: GBufferTextures,
    pipeline: wgpu::RenderPipeline
}

impl GeometryPass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let base_color_texture = Texture::create_fbo(
            &ctx.device, SCR_RESOLUTION,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );
        let normal_texture = Texture::create_fbo(
            &ctx.device, SCR_RESOLUTION,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );
        let rma_texture = Texture::create_fbo(
            &ctx.device, SCR_RESOLUTION,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );
        let world_position_texture = Texture::create_fbo(
            &ctx.device, SCR_RESOLUTION,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );
        let depth_texture = Texture::create_depth_texture(
            &ctx.device, SCR_RESOLUTION,
            wgpu::TextureFormat::Depth32Float
        );

        let shader_code = std::fs::read_to_string("res/shaders/gbuffer.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gbuffer shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let material_bind_group_layout = &asset_manager.default_material().unwrap().bind_group_layout;

        let pipeline = PipelineBuilder::new(
          "gbuffer pipeline",
          &[&material_bind_group_layout, &uniforms.bind_group_layout, &uniforms.bind_group_layout],
          &[Vertex::desc()],
          &shader_module,
          [
            wgpu::TextureFormat::Rgba8UnormSrgb, // base color
            wgpu::TextureFormat::Rgba16Float, // normal
            wgpu::TextureFormat::Rgba8Unorm, // rma
            wgpu::TextureFormat::Rgba16Float // world position
          ]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .with_cull_mode(wgpu::Face::Back)
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        Self {
            textures: GBufferTextures {
                base_color: base_color_texture,
                normal: normal_texture,
                rma: rma_texture,
                world_position: world_position_texture,
                depth: depth_texture
            },
            pipeline
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData, render_data: &RenderDataManager) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("geometry pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.textures.base_color.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             }),
              Some(wgpu::RenderPassColorAttachment {
                view: &self.textures.normal.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             }),
             Some(wgpu::RenderPassColorAttachment {
                view: &self.textures.rma.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             }),
             Some(wgpu::RenderPassColorAttachment {
                view: &self.textures.world_position.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             })
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.textures.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Default::default(),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
       
        for render_item in render_data.render_items_pbr().iter() {
            let model_uniform = uniforms.models.get(&render_item.object_id).unwrap();
            let material = game_data.asset_manager.material_by_index(render_item.material_index).unwrap();
            let mesh = game_data.asset_manager.mesh_by_index(render_item.mesh_index).unwrap();

            pass.set_bind_group(0, &material.bind_group, &[]);
            pass.set_bind_group(2, &model_uniform.bind_group, &[]);
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }
}