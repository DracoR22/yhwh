use wgpu::CommandEncoder;

use crate::{asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, texture::Texture, uniform::Uniform, uniform_manager::UniformManager, utils::unique_id, vertex::Vertex, wgpu_context::WgpuContext};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FireUniform {
    scroll_speeds: [f32; 4],
    scales: [f32; 4],

    distortion1: [f32; 2],
    distortion2: [f32; 2],
    distortion3: [f32; 2],

    distortion_scale: f32,
    distortion_bias: f32,
    time: f32,
    _pad0: f32,

    _pad1: f32,
    _pad2: f32
}

impl FireUniform {
    pub fn new() -> Self { 
        Self {
            scroll_speeds: [1.3, 2.1, 2.3, 0.0],
            scales: [1.0, 2.0, 3.0, 0.0],
            distortion1: [0.1, 0.2],
            distortion2: [0.1, 0.3],
            distortion3: [0.1, 0.1],
            distortion_scale: 0.8,
            distortion_bias: 0.5,
            time: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0
        }
    }

    pub fn update(&mut self) {
        self.time += 0.01;
        if self.time > 1000.0 {
            self.time = 0.0
        }
    }
}

pub struct FirePass {
    pipeline: wgpu::RenderPipeline,
    textures_bind_group_layout: wgpu::BindGroupLayout,
    textures_bind_group: wgpu::BindGroup,
    model_id: usize,
    uniform: Uniform<FireUniform>
}

impl FirePass {
    pub fn new(ctx: &WgpuContext, asset_manager: &AssetManager, uniforms: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/fire.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fire shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let color_texture = asset_manager.texture_by_name("Fire_COLOR.png").unwrap();
        let noise_texture = asset_manager.texture_by_name("Fire_NOISE.png").unwrap();
        let alpha_texture = asset_manager.texture_by_name("Fire_ALPHA.png").unwrap();

        let textures_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float, TL::Float, TL::Float]);
        let textures_bind_group = BindGroupManager::create_multi_texture_bind_group(
            &ctx.device,
            &textures_bind_group_layout, 
            &[color_texture, noise_texture, alpha_texture]
        );

        let pipeline = PipelineBuilder::new(
            "fire pipeline",
            &[
                &textures_bind_group_layout, // color, noise and alpha textures
                &uniforms.bind_group_layout, // camera
                &uniforms.bind_group_layout, // model
                &uniforms.bind_group_layout // globals
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        let model_id = unique_id::next_id();

        Self {
            pipeline,
            textures_bind_group_layout,
            textures_bind_group,
            model_id,
            uniform: Uniform::new(FireUniform::new(), &ctx.device)
        }
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, ctx: &WgpuContext, uniforms: &UniformManager, game_data: &GameData, out_color: &Texture, out_depth: &Texture) {
        self.uniform.value_mut().update();
        self.uniform.update(&ctx.queue);

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
        pass.set_bind_group(0, &self.textures_bind_group, &[]);
        pass.set_bind_group(1, &uniforms.camera.bind_group, &[]);
        pass.set_bind_group(2, &self.uniform.bind_group, &[]);

        game_data.world.for_each_chunk(|chunk| {
            for game_object in chunk.game_objects.iter() {
                if game_object.transform.position.x == 3.0686445 {
                    pass.set_bind_group(3, &uniforms.models.get(&game_object.id).unwrap().bind_group, &[]);
                    let model = game_data.asset_manager.model_by_name(&game_object.model_name).unwrap();
                    for mesh in model.meshes.iter() {
                        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
                    }
                }
            }
        });
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext, uniforms: &UniformManager) {
        let shader_code = std::fs::read_to_string("res/shaders/fire.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fire shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline = PipelineBuilder::new(
            "fire pipeline",
            &[
                &self.textures_bind_group_layout, // color, noise and alpha textures
                &uniforms.bind_group_layout, // camera
                &uniforms.bind_group_layout, // model
                &uniforms.bind_group_layout // globals
            ],
            &[Vertex::desc()],
            &shader_module,
            [wgpu::TextureFormat::Rgba16Float]
        )
        .with_depth(wgpu::TextureFormat::Depth32Float)
        .with_depth_write()
        .build(&ctx.device);

        self.pipeline = pipeline;
    }
}