use std::collections::HashMap;

use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use wgpu::util::DeviceExt;
use crate::{bind_group_manager::{BindGroupManager, TL}, game::game_data::GameData, pipeline_builder::PipelineBuilder, renderer_common::{UI_VERTEX_BUFFER_LAYOUT, UI_VERTICES}, texture::Texture, uniform::Uniform, uniform_manager::UniformManager, wgpu_context::WgpuContext};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiUniform {
    pub model_matrix: [[f32; 4]; 4],
    pub ortho_projection: [[f32; 4]; 4]
}

impl UiUniform {
    pub fn new() -> Self {
        Self {
            model_matrix: Matrix4::identity().into(),
            ortho_projection: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, model_matrix: &Matrix4<f32>, (width, height): (f32, f32)) {
        self.model_matrix = (*model_matrix).into();
        self.ortho_projection = cgmath::ortho(0.0, width, height, 0.0, -1.0, 1.0).into()
    }
}

struct UiRenderData {
    uniform: Uniform<UiUniform>,
    texture_bind_group: wgpu::BindGroup,
}

pub struct UiPass {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    render_data: HashMap<usize, UiRenderData>
}

impl UiPass {
    pub fn new(ctx: &WgpuContext, game_data: &GameData, uniform_manager: &UniformManager) -> Self {
        let shader_code = std::fs::read_to_string("res/shaders/ui.wgsl").unwrap();
        let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let mut render_data = HashMap::<usize, UiRenderData>::new();

        let texture_bg_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float]).unwrap();

        for map_data in game_data.ui_map.iter() {
            let ui_element = map_data.1;
            
            match game_data.asset_manager.get_texture_by_name(&ui_element.texture_name) {
                Some(texture) => {
                    let bind_group = BindGroupManager::create_multi_texture_bind_group(
                        &ctx.device,
                        &texture_bg_layout,
                        &[&texture])
                    .unwrap();

                    render_data.insert(ui_element.id, UiRenderData { 
                        uniform: Uniform::new(UiUniform::new(), &ctx.device),
                        texture_bind_group: bind_group
                    });
                },
                None => {
                    let default_texture = game_data.asset_manager.get_texture_by_name("Default_ALB.png").unwrap();
                    let bind_group = BindGroupManager::create_multi_texture_bind_group(
                        &ctx.device,
                        &texture_bg_layout,
                        &[&default_texture])
                    .unwrap();

                    render_data.insert(ui_element.id, UiRenderData { 
                        uniform: Uniform::new(UiUniform::new(), &ctx.device),
                        texture_bind_group: bind_group
                    });
                }
            }
        }

        let pipeline = PipelineBuilder::new(
            "ui pipeline",
            &[&texture_bg_layout, &uniform_manager.bind_group_layout],
            &[UI_VERTEX_BUFFER_LAYOUT],
            &shader_module,
            [wgpu::TextureFormat::Rgba8UnormSrgb]
        ).build(&ctx.device);

        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("ui quad vertex buffer"),
          contents: bytemuck::cast_slice(&UI_VERTICES),
          usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            render_data
        }
    }

    pub fn render(&mut self, ctx: &WgpuContext, encoder: &mut wgpu::CommandEncoder, game_data: &GameData, (window_width, window_height): (f32, f32), output_view: &wgpu::TextureView) {
        self.update_uniforms(ctx, game_data, (window_width, window_height));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            }),
            ],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);

        for uniform_data in game_data.ui_map.iter() {
            let ui_element = uniform_data.1;
            if !ui_element.visible {
                continue;
            }
            
            if let Some(render_data) = self.render_data.get(&ui_element.id) {
                pass.set_bind_group(0, &render_data.texture_bind_group, &[]);
                pass.set_bind_group(1, &render_data.uniform.bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.draw(0..6, 0..1);
            }
        }
    }

    pub fn update_uniforms(&mut self, ctx: &WgpuContext, game_data: &GameData, (w_width, w_height): (f32, f32)) {
        for map_data in game_data.ui_map.iter() {
            let ui_element = map_data.1;

            if let Some(render_data) = self.render_data.get_mut(&ui_element.id) {
                render_data.uniform.value_mut().update(&ui_element.get_model_matrix(), (w_width, w_height));
                render_data.uniform.update(&ctx.queue);
            }
        }
    }
}