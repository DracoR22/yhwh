use yhwh_core::common::constants::{MAX_LIGHTS, SHADOW_MAP_RES_SIZE};

use crate::{bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, shadow_cube_map_array::ShadowCubeMapArray, uniform::Uniform, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowCubeMapUniform {
  pub light_matrix: [[f32; 4]; 4],
  pub light_pos_x: f32,
  pub light_pos_y: f32,
  pub light_pos_z: f32,
  pub far_plane: f32
}

impl ShadowCubeMapUniform {
  pub fn new() -> Self {
    Self {
       light_matrix: Default::default(),
       light_pos_x: 0.0,
       light_pos_y: 0.0,
       light_pos_z: 0.0,
       far_plane: 0.0
    }
  }

  pub fn update(&mut self, light_matrix: cgmath::Matrix4<f32>, light_pos: cgmath::Vector3<f32>, far_plane: f32) {
    self.light_matrix = light_matrix.into();
    self.light_pos_x = light_pos.x;
    self.light_pos_y = light_pos.y;
    self.light_pos_z = light_pos.z;
    self.far_plane = far_plane;
  }
}

pub struct ShadowPass {
    pub shadow_cube_map_array: ShadowCubeMapArray,
    shadow_cube_map_uniforms: Vec<Uniform<ShadowCubeMapUniform>>,
    pipeline: wgpu::RenderPipeline,
}

impl ShadowPass {
    pub fn new(ctx: &WgpuContext, game_data: &GameData) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/shadow_cube_map.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow cube map shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let shadow_cube_map_array = ShadowCubeMapArray::new(ctx, SHADOW_MAP_RES_SIZE as u32, MAX_LIGHTS as u32);

        let mut shadow_cube_map_uniforms: Vec<Uniform<ShadowCubeMapUniform>> = Vec::new();
        for _ in 0..MAX_LIGHTS {
            for _ in 0..6 {
                shadow_cube_map_uniforms.push(Uniform::new(ShadowCubeMapUniform::new(), &ctx.device));
            }
        }

        let pipeline = PipelineBuilder::new(
            "shadow cube map pipeline",
            &[
                &shadow_cube_map_uniforms[0].bind_group_layout, // shadow
                &shadow_cube_map_uniforms[0].bind_group_layout // model
            ],
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
            shadow_cube_map_uniforms
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager, game_data: &GameData) {
       let mut global_index = 0; 

       game_data.world.for_each_chunk(|chunk| {
            for light in chunk.lights.iter() {
                for face in 0..6 {
                    let index = global_index * 6 + face;

                    self.shadow_cube_map_uniforms[index].value_mut().update(light.projection_transforms[face], light.position, light.radius);
                    self.shadow_cube_map_uniforms[index].update(&ctx.queue);

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
                    pass.set_bind_group(0, &self.shadow_cube_map_uniforms[index].bind_group, &[]);

                    // draw scene
                    for game_object in chunk.game_objects.iter() {
                        // skip distant objects TODO: compare also bounding boxes
                        if (cgmath::MetricSpace::distance2(game_object.transform.position, light.position)) > light.radius * light.radius {
                            continue;
                        }

                        if !game_object.shadows {
                            continue;
                        }

                        let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                            println!("No model bind group for object {:?}, skipping draw", game_object.id);
                            continue;
                        };

                        if let Some(model) = game_data.asset_manager.model_by_name(&game_object.model_name()) {
                            pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                            for mesh in model.meshes.iter() {
                                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                            }
                        }
                    }
                }

                global_index += 1;
            }
       });
    }
}