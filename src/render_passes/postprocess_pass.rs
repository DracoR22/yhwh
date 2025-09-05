use crate::{bind_group_manager::{BindGroupManager, TL}, pipeline_manager::PipelineManager, texture};

pub struct PostProcessPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
}

impl PostProcessPass {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
       let format = wgpu::TextureFormat::Rgba16Float;

       let width = config.width;
       let height = config.height;

       let texture = texture::Texture::create_fbo(&device, (width, height), format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

       let layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Float]).unwrap();
       let bind_group = BindGroupManager::create_texture_bind_group(&device, &layout, &texture).unwrap();

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/postprocess.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Post_Process_Pipeline_Layout"),
                bind_group_layouts: &[&layout],
                push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_pipeline(&device, &pipeline_layout, config.format.add_srgb_suffix(), None, &shader_module, &[], Some("4")).unwrap();

        Self {
            bind_group,
            format,
            height, 
            layout, 
            pipeline,
            texture,
            width
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
       self.texture = texture::Texture::create_fbo(&device, (width, height),  wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

       match BindGroupManager::create_texture_bind_group(&device, &self.layout, &self.texture) {
        Ok(result) => self.bind_group = result,
        Err(e) => {
            println!("PostProcessGroup::resize() error: failed to resize texture bind group!! {e}")
        }
       }

       self.width = width;
       self.height = height;
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
       return &self.texture.view
    }

    pub fn get_format(&self) -> wgpu::TextureFormat {
        return self.format
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
       let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post_Process::render()"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}