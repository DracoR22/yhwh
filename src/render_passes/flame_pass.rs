use std::collections::HashMap;

use crate::{common::enums::MeshRenderingMode, game::game_data::GameData, pipeline_builder::PipelineBuilder, texture::Texture, uniform::Uniform, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FlameUniform {
    pub time: f32,
    pub pad_0: f32,
    pub pad_1: f32,
    pub pad_2: f32
}

impl FlameUniform {
    pub fn new() -> Self {
        Self { 
            time: 0.0,
            pad_0: 0.0,
            pad_1: 0.0,
            pad_2: 0.0
         }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlameParamsUniform {
    pub random_seed: f32,
    pub pad_0: f32,
    pub pad_1: f32,
    pub pad_2: f32
}

impl FlameParamsUniform {
    pub fn new() -> Self {
        Self {
            random_seed: rand::random::<f32>() * 1000.0,
            pad_0: 0.0,
            pad_1: 0.0,
            pad_2: 0.0
        }
    }
}

pub struct FlamePass {
    pipeline: wgpu::RenderPipeline,
    global_uniform: Uniform<FlameUniform>,
    flame_uniforms: HashMap<usize, Uniform<FlameParamsUniform>>
}

impl FlamePass {
    pub fn new(ctx: &WgpuContext, game_data: &GameData) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/flame.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("flames shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let global_uniform = Uniform::new(FlameUniform::new(), &ctx.device);

        // store a random seed per flame mesh node
        let mut flame_uniforms = HashMap::<usize, Uniform<FlameParamsUniform>>::new();
        game_data.world.for_each_chunk(|chunk| {
            for game_object in chunk.game_objects.iter() {
                if let Some(model) = game_data.asset_manager.model_by_name(&game_object.model_name) {
                    for mesh in model.meshes.iter() {
                        match game_object.mesh_nodes().get_mesh_node_by_mesh_name(&mesh.name) {
                            Some(mesh_node) => {
                                // if mesh_node.flame {
                                //     flame_uniforms.insert(mesh_node.id, Uniform::new(FlameParamsUniform::new(), &ctx.device));
                                // }

                                match mesh_node.rendering_mode {
                                    MeshRenderingMode::Flame => {
                                        flame_uniforms.insert(mesh_node.id, Uniform::new(FlameParamsUniform::new(), &ctx.device));
                                    },
                                    _ => ()
                                }
                            },
                            None => ()
                        }
                    }
                }
           }
        });

        let pipeline = PipelineBuilder::new(
            "flame pipeline",
            &[
                &global_uniform.bind_group_layout, // global
                &global_uniform.bind_group_layout, // camera
                &global_uniform.bind_group_layout, // model
                &global_uniform.bind_group_layout // flame
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float, wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        Self {
            pipeline,
            global_uniform,
            flame_uniforms
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &UniformManager, game_data: &GameData, out_color: &Texture, out_emissive: &Texture, out_depth: &Texture) { 
        self.global_uniform.value_mut().time += game_data.delta_time.as_secs_f32();
        self.global_uniform.update(&ctx.queue);

        for data in self.flame_uniforms.iter_mut() {
            let uniform = data.1;
            
            uniform.update(&ctx.queue);
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("flame pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &out_color.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
             }),
             Some(wgpu::RenderPassColorAttachment {
                view: &out_emissive.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
             }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &out_depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Default::default(),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.global_uniform.bind_group, &[]);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);

        game_data.world.for_each_chunk(|chunk| {
            for game_object in chunk.game_objects.iter() {
                let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
                println!("No model bind group for object {:?}, skipping draw", game_object.id);
                continue;
                };

                pass.set_bind_group(2, &model_uniform.bind_group, &[]);

                if let Some(model) = game_data.asset_manager.model_by_name(&game_object.model_name) {
                    for mesh in model.meshes.iter() {
                        match game_object.mesh_nodes().get_mesh_node_by_mesh_name(&mesh.name) {
                            Some(mesh_node) => {
                                if mesh_node.rendering_mode != MeshRenderingMode::Flame {
                                continue;
                                }

                                // TODO: MOVE OUT OF HERE!! 
                                if !self.flame_uniforms.contains_key(&mesh_node.id) {
                                    self.flame_uniforms.insert(mesh_node.id, Uniform::new(FlameParamsUniform::new(), &ctx.device));
                                }

                                let Some(flame_uniform) = self.flame_uniforms.get(&mesh_node.id) else {
                                    println!("No flame bind group for model {:?}, skipping draw", game_object.id);
                                    continue;
                                };

                                pass.set_bind_group(3, &flame_uniform.bind_group, &[]);
                            },
                            None => ()
                        } 
                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
                }
            }
        });
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext) {
        let shader_code = std::fs::read_to_string("res/shaders/flame.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("flame shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline = PipelineBuilder::new(
            "flame pipeline",
            &[
                &self.global_uniform.bind_group_layout, // global
                &self.global_uniform.bind_group_layout, // camera
                &self.global_uniform.bind_group_layout, // model
                &self.global_uniform.bind_group_layout // flames
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float, wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);
        
        self.pipeline = pipeline;
    }
}