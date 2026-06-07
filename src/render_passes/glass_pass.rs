use yhwh_core::common::constants::SCR_RESOLUTION;

use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_core::render_data_manager::RenderDataManager, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct GlassPass {
    pub texture: Texture,
    pipeline: wgpu::RenderPipeline,
    texture_bg_layout: wgpu::BindGroupLayout
}

impl GlassPass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/glass.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glass shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let glass_material = asset_manager.material_by_name("WindowsGlass").unwrap();

        let pipeline = PipelineBuilder::new(
            "glass pipeline",
            &[&glass_material.bind_group_layout, &uniforms.camera.bind_group_layout, &uniforms.bind_group_layout, &uniforms.lights_ssbo.bind_group_layout],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        Self {
            pipeline,
            texture,
            texture_bg_layout: glass_material.bind_group_layout.clone()
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager,  game_data: &GameData, render_data: &RenderDataManager, out_depth_texture: &Texture) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.texture.view,
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
            stencil_ops: Default::default()
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let glass_material = game_data.asset_manager.material_by_name("WindowsGlass").unwrap();

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &glass_material.bind_group, &[]);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        pass.set_bind_group(3, &uniforms.lights_ssbo.bind_group, &[]);

        for render_item in render_data.render_items_glass().iter() {
            let model_uniform = uniforms.models.get(&render_item.object_id).unwrap();
            let mesh = game_data.asset_manager.mesh_by_index(render_item.mesh_index).unwrap();

            pass.set_bind_group(2, &model_uniform.bind_group, &[]);

            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
        let shader_code = std::fs::read_to_string("res/shaders/glass.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glass shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        // let glass_material = asset_manager.get_material_by_name("WindowsGlass").unwrap();

        let pipeline = PipelineBuilder::new(
            "glass pipeline",
            &[&self.texture_bg_layout, &uniforms.camera.bind_group_layout, &uniforms.bind_group_layout, &uniforms.lights_ssbo.bind_group_layout],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

    self.pipeline = pipeline;
    }
}