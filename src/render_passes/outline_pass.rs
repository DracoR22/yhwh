use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::HDR_TEX_FORMAT, objects::game_object::GameObject, pipeline_manager::PipelineManager, texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct OutlinePass {
    fbo_tex: texture::Texture,
    pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture
    //bind_group: wgpu::BindGroup
}

impl OutlinePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/outline.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Outline_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let format = wgpu::TextureFormat::Rgba16Float;
        let fbo_tex = texture::Texture::create_fbo(&ctx.device, (ctx.config.width, ctx.config.height), format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let depth_texture = texture::Texture::create_depth_texture(&ctx.device, &ctx.config, "depth_texture2");

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outline_Pipeline_Layout"),
                bind_group_layouts: &[
                    &uniforms.camera.bind_group_layout,
                    &uniforms.bind_group_layout
                ],
                push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_pipeline(
            &ctx.device,
            &pipeline_layout,
            HDR_TEX_FORMAT,
            Some(wgpu::TextureFormat::Depth32Float),
            &shader_module,
            &[Vertex::desc()],
            Some("Outline_Pipeline")
        ).unwrap();

        Self {
            fbo_tex,
            pipeline,
            depth_texture
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_objects: &Vec<GameObject>, asset_manager: &AssetManager) {
         let mut outline_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("OutlinePass::render()"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.fbo_tex.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None
        });

        outline_pass.set_pipeline(&self.pipeline);
        outline_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);

        for game_object in game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.object_id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.object_id);
            continue;
           };

           outline_pass.set_bind_group(1, &model_uniform.bind_group, &[]);

           if let Some(model) = asset_manager.get_model_by_name(&game_object.get_model_name()) {
             for mesh in model.meshes.iter() {
                outline_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                outline_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                outline_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
             }
           }
        }
    }

    pub fn get_texture(&self) -> &texture::Texture {
        &self.fbo_tex
    }
}