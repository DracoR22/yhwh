use crate::{bind_group_manager::{BindGroupManager, TL}, pipeline_manager::PipelineManager, texture, wgpu_context::WgpuContext};

pub struct PostProcessPass {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    texture: texture::Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl PostProcessPass {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, outline_texture: &texture::Texture) -> Self {
       let format = wgpu::TextureFormat::Rgba16Float;
 
       let width = config.width;
       let height = config.height;

       let hdr_texture = texture::Texture::create_fbo(&device, (width, height), format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

       let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Float, TL::Float]).unwrap();
       // let bind_group = BindGroupManager::create_texture_bind_group(&device, &bind_group_layout, &texture).unwrap();
        let bind_group = BindGroupManager::create_multi_texture_bind_group(
         &device,
         &bind_group_layout,
         &[&hdr_texture, &outline_texture])
         .unwrap();

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/postprocess.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Post_Process_Pipeline_Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
        });

        let pipeline = PipelineManager::create_pipeline(&device, &pipeline_layout, config.format.add_srgb_suffix(), None, &shader_module, &[], Some("4")).unwrap();

        Self {
            bind_group,
            format,
            height, 
            bind_group_layout, 
            pipeline,
            texture: hdr_texture,
            width,
            pipeline_layout
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
    //    self.texture = texture::Texture::create_fbo(&device, (width, height),  wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

    //    match BindGroupManager::create_texture_bind_group(&device, &self.bind_group_layout, &self.texture) {
    //     Ok(result) => self.bind_group = result,
    //     Err(e) => {
    //         println!("PostProcessGroup::resize() error: failed to resize texture bind group!! {e}")
    //     }
    //    }

    //    self.width = width;
    //    self.height = height;
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
       return &self.texture.view
    }

    pub fn get_format(&self) -> wgpu::TextureFormat {
        return self.format
    }

    pub fn hotload_shader(&mut self, ctx: &WgpuContext) {
      let shader_code = std::fs::read_to_string("res/shaders/postprocess.wgsl").unwrap();
      let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default_Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
       });

      let new_pipeline = PipelineManager::create_pipeline(&ctx.device, &self.pipeline_layout, ctx.config.format.add_srgb_suffix(), None, &shader_module, &[], Some("4"));

      match new_pipeline {
          Ok(res) => self.pipeline = res,
          Err(err) => println!("PostProcessPass::hotload_shader() error: {err}")
      }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, swapchain_view: &wgpu::TextureView) {
       let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post_Process::render()"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &swapchain_view,
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