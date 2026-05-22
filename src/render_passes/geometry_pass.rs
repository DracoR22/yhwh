use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::SCR_RESOLUTION, game::game_data::GameData, pipeline_builder::PipelineBuilder, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

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
        let base_color_texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba8UnormSrgb, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let normal_texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let rma_texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let world_position_texture = Texture::create_fbo(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let depth_texture = Texture::create_depth_texture(&ctx.device, SCR_RESOLUTION, wgpu::TextureFormat::Depth32Float);

        let shader_code = std::fs::read_to_string("res/shaders/gbuffer.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gbuffer shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let material_bind_group_layout = &asset_manager.get_default_material().unwrap().bind_group_layout;

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

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData) {
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

        let frustum = &game_data.active_camera().frustum;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

        // game objects
        for game_object in game_data.scene.game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
             println!("No model bind group for object {:?}, skipping draw", game_object.id);
             continue;
            };

           pass.set_bind_group(2, &model_uniform.bind_group, &[]);
            if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
                    // frustum culling
                    if let Some(model_aabb) = model.aabb {
                        let model_matrix = game_object.get_model_matrix();
                        let world_aabb = model_aabb.transform(model_matrix);

                        if !frustum.intersects_aabb(&world_aabb) {
                            continue;
                        }
                    }

                    for mesh in &model.meshes {
                        match game_object.get_mesh_nodes().get_mesh_node_by_mesh_name(&mesh.name) {
                        Some(mesh_node) => {
                            if mesh_node.emissive || mesh_node.glass || mesh_node.candle_flame {
                            continue;
                            }
                        },
                        None => ()
                        }

                        let mesh_material_index = game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
                        let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

                        pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
            }
        }

        // door objects
        for door_object in game_data.scene.door_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&door_object.id) else {
             println!("No model bind group for object {:?}, skipping draw", door_object.id);
             continue;
            };

           pass.set_bind_group(2, &model_uniform.bind_group, &[]);
            if let Some(model) = game_data.asset_manager.get_model_by_name(&door_object.model_name) {
                    // frustum culling
                    if let Some(model_aabb) = model.aabb {
                        let model_matrix = door_object.get_model_matrix();
                        let world_aabb = model_aabb.transform(model_matrix);

                        if !frustum.intersects_aabb(&world_aabb) {
                            continue;
                        }
                    }

                    for mesh in &model.meshes {
                        match door_object.get_mesh_nodes().get_mesh_node_by_mesh_name(&mesh.name) {
                        Some(mesh_node) => {
                            if mesh_node.emissive || mesh_node.glass {
                            continue;
                            }
                        },
                        None => ()
                        }

                        let mesh_material_index = door_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
                        let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

                        pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
            }
        }
    }
}