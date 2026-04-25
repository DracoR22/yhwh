use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::SCR_RESOLUTION, engine::GameData, pipeline_builder::PipelineBuilder, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

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

        let texture = Texture::create_fbo(&ctx.device, (SCR_RESOLUTION[0], SCR_RESOLUTION[1]), wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let glass_material = asset_manager.get_material_by_name("WindowsGlass").unwrap();

        // let texture_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float, TL::Float, TL::Float]).unwrap();
        // let texture_bind_group = BindGroupManager::create_multi_texture_bind_group(
        //     &ctx.device,
        //     &scene_bind_group_layout,
        //     &[glass_material.bind_group_layout])
        // .unwrap();

        let pipeline = PipelineBuilder::new(
            "glass pipeline",
            &[&glass_material.bind_group_layout, &uniforms.camera.bind_group_layout, &uniforms.bind_group_layout, &uniforms.lights_ssbo.bind_group_layout],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32FloatStencil8)
        .with_depth_write()
        .build(&ctx.device);

        Self {
            pipeline,
            texture,
            texture_bg_layout: glass_material.bind_group_layout.clone()
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, out_depth_texture: &Texture, game_data: &GameData) {
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

        let glass_material = game_data.asset_manager.get_material_by_name("WindowsGlass").unwrap();

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &glass_material.bind_group, &[]);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        pass.set_bind_group(3, &uniforms.lights_ssbo.bind_group, &[]);

        for game_object in game_data.scene.game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                println!("No model bind group for object {:?}, skipping draw", game_object.id);
                continue;
            };

            pass.set_bind_group(2, &model_uniform.bind_group, &[]);

            if let Some(model) = game_data.asset_manager.get_model_by_name(game_object.get_model_name()) {
                for mesh in model.meshes.iter() {
                    match game_object.get_mesh_nodes().get_mesh_node_by_mesh_name(&mesh.name) {
                        Some(mesh_node)=> {
                            if mesh_node.glass {
                                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                            }
                        },
                        None => ()
                    }
                }
            }
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
        .with_depth(wgpu::TextureFormat::Depth32FloatStencil8)
        .with_depth_write()
        .build(&ctx.device);

    self.pipeline = pipeline;
    }
}