use crate::{bind_group_manager::{BindGroupManager, TL}, common::constants::{DEPTH_TEXTURE_STENCIL_FORMAT, MAX_LIGHTS, SHADOW_MAP_RES_SIZE}, engine::GameData, pipeline_builder::PipelineBuilder, shadow_cube_map_array::ShadowCubeMapArray, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

pub struct ShadowPass {
    pub shadow_cube_map_array: ShadowCubeMapArray,
    pipeline: wgpu::RenderPipeline,
    // bind_group_layout: wgpu::BindGroupLayout,
    // bind_group: wgpu::BindGroup
}

impl ShadowPass {
    pub fn new(ctx: &WgpuContext, game_data: &GameData) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/shadow_cube_map.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow cube map shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let shadow_cube_map_array = ShadowCubeMapArray::new(ctx, SHADOW_MAP_RES_SIZE as u32, game_data.scene.lights.len() as u32);

        let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
            &ctx.device,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            Some("shadow uniforms bind group layout"))
        .unwrap();

        let pipeline = PipelineBuilder::new(
            "shadow cube map pipeline",
            &[&bind_group_layout, &bind_group_layout],
            &[Vertex::desc()],
            &shader_module,
            []
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        Self {
            pipeline,
            shadow_cube_map_array,
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &GameData) {
        let active_lights = game_data.scene.lights.len().min(MAX_LIGHTS as usize);

        if active_lights > self.shadow_cube_map_array.capacity {
            self.shadow_cube_map_array = ShadowCubeMapArray::new(ctx, SHADOW_MAP_RES_SIZE as u32, active_lights as u32);
            uniforms.lights_ssbo.rebuild_bind_group(&ctx.device, &self.shadow_cube_map_array.texture);
        }

        for light_index in 0..active_lights {
            let light = &game_data.scene.lights[light_index];

            if !light.shadows {
                continue;
            }

            for face in 0..6 {
                let index = light_index * 6 + face;

                uniforms.shadow_cube_maps[index].value_mut().update(light.projection_transforms[face], light.position, light.radius);
                uniforms.shadow_cube_maps[index].update(&ctx.queue);

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("shadow cube pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.shadow_cube_map_array.face_views[index],
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }
                    ),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &uniforms.shadow_cube_maps[index].bind_group, &[]);

                // draw scene
                for game_object in game_data.scene.game_objects.iter() {
                    // skip distant objects TODO: compare also bounding boxes
                    if (cgmath::MetricSpace::distance2(game_object.get_position(), light.position)) > light.radius * light.radius {
                        continue;
                    }

                    if !game_object.shadows {
                        continue;
                    }

                    let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                        println!("No model bind group for object {:?}, skipping draw", game_object.id);
                        continue;
                    };

                    if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
                        pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                        for mesh in model.meshes.iter() {
                            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                        }
                    }
                }
            }
        }
    }

    // pub fn get_bind_group(&self) -> &wgpu::BindGroup {
    //     &self.bind_group
    // }

    // fn update_bind_group(&mut self, ctx: &WgpuContext) {
    //     match BindGroupManager::create_texture_bind_group(&ctx.device, &self.bind_group_layout, &self.shadow_cube_map_array.texture) {
    //         Ok(bind_group) => {
    //             self.bind_group = bind_group
    //         },
    //         Err(_err) => {}
    //     }
    // }
}