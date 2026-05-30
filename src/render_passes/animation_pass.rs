use crate::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, game::game_data::GameData, objects::animated_game_object, pipeline_builder::PipelineBuilder, texture::Texture, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct AnimationPass {
    pipeline: wgpu::RenderPipeline,
    created_player_uniform: bool
}

impl AnimationPass {
    pub fn new(ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &GameData) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/animation.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let texture_bind_group_layout = game_data.asset_manager.get_phong_bind_group_layout().expect("No bind group layout for Phong!");

        let pipeline = PipelineBuilder::new(
            "animation pipeline",
            &[
              &texture_bind_group_layout,
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout, // model
              &uniforms.bind_group_layout // animation
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float],
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        let animated_game_object = game_data.player.weapon_animated_game_object();
        uniforms.create_model(ctx, animated_game_object.id);
        uniforms.create_animation(ctx, animated_game_object.id);

        Self {
          pipeline,
          created_player_uniform: false
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &mut GameData, output_texture: &Texture, output_depth: &Texture) {
       let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("animation pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &output_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
             }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &output_depth.view,
            depth_ops: Some(wgpu::Operations {
              load: wgpu::LoadOp::Load,
              store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

        game_data.world.for_each_chunk(|chunk| {
           for animated_game_object in chunk.animated_game_objects.iter() {
              let Some(model_uniform) = uniforms.models.get(&animated_game_object.id) else {
                println!("No model bind group for object {:?}, skipping draw", &animated_game_object.id);
                return
              };

              let Some(animation_uniform) = uniforms.animations.get(&animated_game_object.id) else {
                println!("No animation bind group for object {:?}, skipping draw", &animated_game_object.id);
                return
              };

              render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);
              render_pass.set_bind_group(3, &animation_uniform.bind_group, &[]);

              if let Some(model) = game_data.asset_manager.get_model_by_name(&animated_game_object.get_model_name()) {
              for mesh in &model.meshes {
                let mesh_material_index = animated_game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
                let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

                render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                }
              }
            }
        });

        // draw player weapons
        let has_weapon = game_data.player.weapon_manager.desert_eagle_info.has;
        let animated_game_object = game_data.player.weapon_animated_game_object_mut();

        let Some(model_uniform) = uniforms.models.get_mut(&animated_game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", &animated_game_object.id);
            return
        };

        let Some(animation_uniform) = uniforms.animations.get_mut(&animated_game_object.id) else {
            println!("No animation bind group for object {:?}, skipping draw", &animated_game_object.id);
            return
        };

        model_uniform.value_mut().update(&animated_game_object.model_matrix, &animated_game_object.tex_scale);
        model_uniform.update(&ctx.queue);

        animated_game_object.update(&game_data.asset_manager, game_data.delta_time.as_secs_f32());
              let skin_uniform = animation_uniform.value_mut();

              if let Some(skin) = animated_game_object.skins.get(0) {
                  for (i, joint) in skin.joints().iter().enumerate() {
                        if i >= MAX_JOINTS_PER_MESH {
                           break; 
                        }

                      skin_uniform.joint_matrices[i] = joint.matrix().into();
                  }
        }
        animation_uniform.update(&ctx.queue);

        render_pass.set_bind_group(2, &model_uniform.bind_group, &[]);
        render_pass.set_bind_group(3, &animation_uniform.bind_group, &[]);

        // if !has_weapon {
        //    return;
        // }

        if let Some(model) = game_data.asset_manager.get_model_by_name(&animated_game_object.get_model_name()) {
           for mesh in &model.meshes {
             let mesh_material_index = animated_game_object.get_mesh_nodes().get_mesh_material_index_by_mesh_name(&mesh.name);
             let mesh_material = game_data.asset_manager.get_material_by_index(mesh_material_index);

             render_pass.set_bind_group(0, &mesh_material.unwrap().bind_group, &[]);

             render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
             render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

             render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
        }
    }
}