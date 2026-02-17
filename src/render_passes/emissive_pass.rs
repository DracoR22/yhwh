use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, common::constants::{DEPTH_TEXTURE_FORMAT, DEPTH_TEXTURE_STENCIL_FORMAT, HDR_TEX_FORMAT}, engine::GameData, pipeline_builder::PipelineBuilder, pipeline_manager::PipelineManager, renderer_common::QUAD_VERTICES, texture::{self, Texture}, uniform_manager::UniformManager, vertex::Vertex, wgpu_context::WgpuContext};
use wgpu::util::DeviceExt;

pub struct EmissivePass {
    pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    ping_texture: Texture,
    pong_texture: Texture,
    bind_group_layout: wgpu::BindGroupLayout,
    ping_bind_group: wgpu::BindGroup,
    pong_bind_group: wgpu::BindGroup,
    source_bind_group: wgpu::BindGroup,   
    quad_vertex_buffer: wgpu::Buffer,
    final_blur_texture: Texture,
    final_is_ping: bool,
}

 const quad_vertex_buffer_layout: wgpu::VertexBufferLayout = wgpu::VertexBufferLayout {
            array_stride: (4 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,
                1 => Float32x2
            ]
    };

impl EmissivePass {
    pub fn new(ctx: &WgpuContext, uniforms: &UniformManager, asset_manager: &AssetManager, source_texture: &Texture) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/bloom.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline = PipelineBuilder::new(
            "emissive pipeline",
            &[
              &uniforms.camera.bind_group_layout,
              &uniforms.bind_group_layout,
            ],
            &[Vertex::desc()],
            &shader_module,
            [HDR_TEX_FORMAT, HDR_TEX_FORMAT],
        )
        .with_depth(DEPTH_TEXTURE_STENCIL_FORMAT)
        .with_depth_write()
        .build(&ctx.device);

        let blur_shader_code = std::fs::read_to_string("res/shaders/blur.wgsl").unwrap();
        let blur_shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur_Shader"),
            source: wgpu::ShaderSource::Wgsl(blur_shader_code.into())
        });

        let quad_vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("Vertex_Buffer"),
          contents: bytemuck::cast_slice(&QUAD_VERTICES),
          usage: wgpu::BufferUsages::VERTEX,
        });

        let width = 1920;
        let height = 1080;

        let ping_texture = Texture::create_fbo(&ctx.device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let pong_texture = Texture::create_fbo(&ctx.device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        let final_blur_texture = Texture::create_fbo(&ctx.device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float]).unwrap();

        let ping_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &ping_texture).unwrap();
        let pong_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &pong_texture).unwrap();
        let source_bind_group = BindGroupManager::create_texture_bind_group(&ctx.device, &bind_group_layout, &source_texture).unwrap();

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&bind_group_layout, &uniforms.blur.bind_group_layout],
            &[quad_vertex_buffer_layout],
            &blur_shader_module,
            [HDR_TEX_FORMAT]
        )
        .with_blend(wgpu::BlendState::REPLACE)
        .build(&ctx.device);

        Self {
            pipeline,
            blur_pipeline,
            ping_texture, 
            pong_texture,
            bind_group_layout,
            ping_bind_group,
            pong_bind_group,
            source_bind_group,
            quad_vertex_buffer,
            final_blur_texture,
            final_is_ping: true
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, game_data: &GameData, uniforms: &UniformManager, hdr_texture_view: &wgpu::TextureView, emissive_texture_view: &wgpu::TextureView, depth_texture_view: &wgpu::TextureView) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &uniforms.camera.bind_group, &[]);
        
        for game_object in game_data.scene.game_objects.iter() {
            let Some(model_uniform) = uniforms.models.get(&game_object.id) else {
            println!("No model bind group for object {:?}, skipping draw", game_object.id);
            continue;
          };

          if game_object.get_model_name() == "candles" ||  game_object.get_model_name() == "Cube" {
            if let Some(model) = game_data.asset_manager.get_model_by_name(&game_object.get_model_name()) {
                render_pass.set_bind_group(1, &model_uniform.bind_group, &[]);
                for mesh in model.meshes.iter() {
                    if mesh.name == "Object_0" || mesh.name == "Cube_Mesh"  {
                         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
                }
            }
          }
        }
    }

    pub fn render_blur(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &WgpuContext, uniforms: &mut UniformManager, source_texture: &Texture) {
        let mut horizontal = true;
        let mut first_iteration = true;

        let blur_passes = 4;
        let mut sample_distance = 4.0;

        for i in 0..blur_passes {
            if sample_distance < 8.0 {
                sample_distance = sample_distance + 1.0
            }

            let direction = if horizontal {
                [1.0, 0.0]
            } else {
                [0.0, 1.0]
            };

            uniforms.blurs[i].value_mut().update(direction, sample_distance);
            uniforms.blurs[i].update(&ctx.queue);
    
            let target_texture = if horizontal {
                &self.pong_texture
            } else {
                &self.ping_texture
            };

            let target_bind_group = if first_iteration {
                &self.source_bind_group
            } else {
                if horizontal {
                    &self.ping_bind_group
                } else {
                    &self.pong_bind_group
                }
            };

            // let texture_write = if first_iteration {
            //     &source_texture
            // } else {
            //     if horizontal {
            //         &self.ping_texture
            //     } else {
            //         &self.pong_texture
            //     }
            // };

            // println!("TEX VIEW WRITE: {:?}", &target_texture.view);
            // print!("TEX VIEW READ: {:?}", &texture_write.view);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("blur pass {}", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None
            });

            render_pass.set_pipeline(&self.blur_pipeline);
            render_pass.set_bind_group(0, target_bind_group, &[]);
            render_pass.set_bind_group(1, &uniforms.blurs[i].bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));  
            render_pass.draw(0..6, 0..1);    

            //self.final_blur_texture = target_texture.clone();
            self.final_is_ping = !horizontal;

            horizontal = !horizontal;

            if first_iteration {
                first_iteration = false;
            } 
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, source_texture: &Texture) {
        // self.ping_texture = Texture::create_fbo(&device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
        // self.pong_texture = Texture::create_fbo(&device, (width, height), HDR_TEX_FORMAT, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        // match BindGroupManager::create_texture_bind_group(&device, &self.bind_group_layout, &self.ping_texture) {
        //     Ok(result) => { self.ping_bind_group = result },
        //     Err(e) => { println!("EmissivePass::resize() error: failed to resize PING texture bind group!! {e}")}
        // }
        // match BindGroupManager::create_texture_bind_group(&device, &self.bind_group_layout, &self.pong_texture) {
        //     Ok(result) => { self.pong_bind_group = result },
        //     Err(e) => { println!("EmissivePass::resize() error: failed to resize PONG texture bind group!! {e}")}
        // }
        // match BindGroupManager::create_texture_bind_group(&device, &self.bind_group_layout, &source_texture) { 
        //     Ok(result) => { self.source_bind_group = result },
        //     Err(e) => { println!("EmissivePass::resize() error: failed to resize SOURCE texture bind group!! {e}")}
        // }
    }

    pub fn hotload_shader(&mut self, device: &wgpu::Device, uniforms: &UniformManager) {
        let shader_code = std::fs::read_to_string("res/shaders/blur.wgsl").unwrap();
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into())
        });

        let blur_pipeline = PipelineBuilder::new(
            "blur pipeline",
            &[&self.bind_group_layout, &uniforms.bind_group_layout],
            &[quad_vertex_buffer_layout],
            &shader_module,
            [HDR_TEX_FORMAT]
        )
        .build(&device);

        self.blur_pipeline = blur_pipeline;
    }

    pub fn get_final_texture(&self) -> &Texture {
        if self.final_is_ping {
        &self.ping_texture
    } else {
        &self.pong_texture
    }
    }
}