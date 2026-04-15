use crate::{asset_manager::AssetManager, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, engine::GameData, objects::game_object::GameObject, pipeline_builder::PipelineBuilder, pipeline_manager::PipelineManager, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct ScenePass {
    pbr_pipeline: wgpu::RenderPipeline,
    emissive_pipeline: wgpu::RenderPipeline,
    texture_bg_layout: wgpu::BindGroupLayout,
    pub pbr_texture: Texture,
    pub emissive_texture: Texture,
    pub depth_texture: Texture
}

impl ScenePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager) -> Self {
        let pbr_shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
        let pbr_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lighting shader"),
            source: wgpu::ShaderSource::Wgsl(pbr_shader_code.into()),
        });

        let emissive_shader_code = std::fs::read_to_string("res/shaders/bloom.wgsl").unwrap();
        let emissive_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom shader"),
            source: wgpu::ShaderSource::Wgsl(emissive_shader_code.into()),
        });

        let texture_bind_group_layout = &asset_manager.get_default_material().unwrap().bind_group_layout;

        let pbr_texture = Texture::create_fbo(&ctx.device, (1920, 1080), wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let emissive_texture = Texture::create_fbo(&ctx.device, (1920, 1080), wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let depth_texture = Texture::create_depth_texture(&ctx.device, "scene depth texture", DEPTH_TEXTURE_STENCIL_FORMAT);

        let pbr_pipeline = PipelineBuilder::new(
            "scene::pbr pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &pbr_shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .with_cull_mode(wgpu::Face::Back)
        .build(&ctx.device);

        let emissive_pipeline = PipelineBuilder::new(
            "scene::emissive pipeline",
            &[
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
            ],
            &[Vertex::desc()],
            &emissive_shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);

     Self {
        emissive_pipeline,
        pbr_pipeline,
        pbr_texture,
        depth_texture,
        emissive_texture,
        texture_bg_layout: texture_bind_group_layout.clone()
     }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, uniforms: &UniformManager, game_data: &GameData) {
       let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("scene pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.pbr_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             }),
              Some(wgpu::RenderPassColorAttachment {
                view: &self.emissive_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             })
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
              load: wgpu::LoadOp::Clear(1.0),
              store: wgpu::StoreOp::Store,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: wgpu::StoreOp::Store,
            }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        let frustum = &game_data.active_camera().frustum;

        // draw pbr objects
        render_pass.set_pipeline(&self.pbr_pipeline);
        render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(3, &uniforms.lights_ssbo.bind_group, &[]);

        for game_object in game_data.scene.game_objects.iter() {
          let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.id);
            continue;
          };

          render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);

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
                let mesh_node = game_object.get_mesh_nodes().get_mesh_rendering_info_by_mesh_name(&mesh.name);

                if mesh_node.emissive || mesh_node.glass {
                  continue;
                }

                let mesh_material_index = game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
                let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

                render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_stencil_reference(1);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
          }
        }

        // extract emissive meshes
        render_pass.set_pipeline(&self.emissive_pipeline);
        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);
        
        for game_object in game_data.scene.game_objects.iter() {
          let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.id);
            continue;
          };

          if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
              render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                for mesh in model.meshes.iter() {
                    if game_object.get_mesh_nodes().get_mesh_rendering_info_by_mesh_name(&mesh.name).emissive {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
              }
          }
        }
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
      let shader_code = std::fs::read_to_string("res/shaders/lighting.wgsl").unwrap();
      let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lighting_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
       });

       let pipeline = PipelineBuilder::new(
            "lighting pipeline",
            &[
              &self.texture_bg_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
              &uniforms.lights_ssbo.bind_group_layout
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .with_cull_mode(wgpu::Face::Back)
        .build(&ctx.device);

      self.pbr_pipeline = pipeline;
    }
}