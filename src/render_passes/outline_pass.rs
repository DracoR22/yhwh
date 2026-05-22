use crate::{bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_common::{QUAD_VERTEX_BUFFER_LAYOUT, QUAD_VERTICES}, texture::{self, Texture}, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};
use wgpu::util::DeviceExt;

pub struct OutlinePass {
    mask_pipeline: wgpu::RenderPipeline,
    mask_texture: Texture,
    mask_bind_group: wgpu::BindGroup,
    mask_bind_group_layout: wgpu::BindGroupLayout,
    outline_pipeline: wgpu::RenderPipeline,
    outline_texture: Texture,
    vertex_buffer: wgpu::Buffer,
}

impl OutlinePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager) -> Self {
        let mask_shader_code = std::fs::read_to_string("res/shaders/solid_color.wgsl").unwrap();
        let mask_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mask solid color shader"),
            source: wgpu::ShaderSource::Wgsl(mask_shader_code.into()),
        });

        let outline_shader_code = std::fs::read_to_string("res/shaders/outline.wgsl").unwrap();
        let outline_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("outline shader"),
            source: wgpu::ShaderSource::Wgsl(outline_shader_code.into()),
        });

        let mask_pipeline = PipelineBuilder::new(
            "outline mask pipeline",
            &[&uniforms.camera.bind_group_layout, &uniforms.bind_group_layout],
            &[Vertex::desc()],
            &mask_shader_module,
            [wgpu::TextureFormat::R8Unorm],
        )
        .build(&ctx.device);

        let mask_texture = Texture::create_fbo(
            &ctx.device, (1920, 1080),
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );

        let mask_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float]);
        let mask_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &mask_bind_group_layout, &mask_texture);

        let outline_pipeline = PipelineBuilder::new(
            "outline pipeline",
            &[&mask_bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &outline_shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        //.with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .build(&ctx.device);

        let outline_texture = Texture::create_fbo(
            &ctx.device, (1920, 1080),
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        );

        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("outline quad vertex buffer"),
          contents: bytemuck::cast_slice(&QUAD_VERTICES),
          usage: wgpu::BufferUsages::VERTEX,
        });

        Self {  
            mask_pipeline,
            mask_texture,
            mask_bind_group,
            outline_pipeline,
            outline_texture,
            vertex_buffer,
            mask_bind_group_layout
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("outline mask pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.mask_texture.view,
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

        render_pass.set_pipeline(&self.mask_pipeline);
        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);

        for game_object in game_data.scene.game_objects.iter() {
            // game objects
           if game_object.is_selected {
               let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                    println!("No model bind group for object {:?}, skipping draw", game_object.id);
                    continue;
                };

                render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);

                if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
                    for mesh in model.meshes.iter() {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.set_stencil_reference(1);
                        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
                }
            }
        }

         // doors
        for door_object in game_data.scene.door_objects.iter() {
            if door_object.is_selected {
                let Some(model_uniform) = uniforms.models.get(&door_object.id) else {
                    println!("No model bind group for object {:?}, skipping draw", door_object.id);
                    continue;
               };

                render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);

                if let Some(model) = game_data.asset_manager.get_model_by_name(&door_object.model_name) {
                    for mesh in model.meshes.iter() {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.set_stencil_reference(1);
                        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
                }
            }
        }

        // animated game objects
        // for animated_game_object in game_data.scene.animated_game_objects.iter() {
        //    if animated_game_object.is_selected {
        //        let Some(model_uniform) = uniforms.models.get(&animated_game_object.id) else {
        //             println!("No model bind group for object {:?}, skipping draw", animated_game_object.id);
        //             continue;
        //         };

        //         render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);

        //         if let Some(model) = game_data.asset_manager.get_model_by_name(&animated_game_object.get_model_name()) {
        //             for mesh in model.meshes.iter() {
        //                 render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        //                 render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //                 render_pass.set_stencil_reference(1);
        //                 render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        //             }
        //         }
        //     }
        // }

        drop(render_pass);

        let mut outline_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("outline pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.outline_texture.view,
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

        outline_pass.set_pipeline(&self.outline_pipeline);
        outline_pass.set_bind_group(0, &self.mask_bind_group, &[]);
        outline_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        outline_pass.draw(0..6, 0..1);
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
        let mask_shader_code = std::fs::read_to_string("res/shaders/solid_color.wgsl").unwrap();
        let mask_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mask solid color shader"),
            source: wgpu::ShaderSource::Wgsl(mask_shader_code.into()),
        });

        let outline_shader_code = std::fs::read_to_string("res/shaders/outline.wgsl").unwrap();
        let outline_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("outline shader"),
            source: wgpu::ShaderSource::Wgsl(outline_shader_code.into()),
        });

        let mask_pipeline = PipelineBuilder::new(
                "outline mask pipeline",
                &[&uniforms.camera.bind_group_layout, &uniforms.bind_group_layout],
                &[Vertex::desc()],
                &mask_shader_module,
                [wgpu::TextureFormat::R8Unorm],
            )
            .build(&ctx.device);

        let outline_pipeline = PipelineBuilder::new(
            "outline pipeline",
            &[&self.mask_bind_group_layout],
            &[QUAD_VERTEX_BUFFER_LAYOUT],
            &outline_shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .build(&ctx.device);

        self.mask_pipeline = mask_pipeline;
        self.outline_pipeline = outline_pipeline;
    }

    pub fn get_outline_texture(&self) -> &Texture {
        &self.outline_texture
    }
}