use crate::{bind_group_manager::{BindGroupManager, TL}, common::constants::HDR_TEX_FORMAT, pipeline_builder::PipelineBuilder, pipeline_manager::PipelineManager, texture::{self, Texture}, wgpu_context::WgpuContext};

pub struct PostProcessPass {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    texture: Texture,
    emissive_texture: Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl PostProcessPass {
    pub fn new(ctx: &WgpuContext, config: &wgpu::SurfaceConfiguration) -> Self {
       let format = wgpu::TextureFormat::Rgba16Float;
 
       let width = config.width;
       let height = config.height;

       let hdr_texture = Texture::create_fbo(&ctx.device, (1920, 1080), format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
       let emissive_texture = Texture::create_fbo(&ctx.device, (1920, 1080), format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&ctx.device, [TL::Float, TL::Float]).unwrap();
        let bind_group = BindGroupManager::create_multi_texture_bind_group(&ctx.device, &bind_group_layout, &[&hdr_texture, &emissive_texture]).unwrap();

       let shader_module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/postprocess.wgsl").into()),
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post_Process_Pipeline_Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        //let pipeline = PipelineManager::create_pipeline(&device, &pipeline_layout, config.format.add_srgb_suffix(), None, &shader_module, &[], Some("4")).unwrap();

         let pipeline = PipelineBuilder::new(
            "postprocess pipeline",
            &[&bind_group_layout],
            &[],
            &shader_module,
            [config.format.add_srgb_suffix()],
        )
        .build(&ctx.device);

        Self {
            bind_group,
            format,
            height, 
            bind_group_layout, 
            pipeline,
            texture: hdr_texture,
            emissive_texture,
            width,
            pipeline_layout
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, final_blur_texture: &Texture) {
       //self.texture = texture::Texture::create_fbo(&device, (width, height),  wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);
      // self.emissive_texture = texture::Texture::create_fbo(&device, (width, height),  wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT);

    //    match BindGroupManager::create_multi_texture_bind_group(&device, &self.bind_group_layout, &[&self.texture, &final_blur_texture]) {
    //     Ok(result) => self.bind_group = result,
    //     Err(e) => {
    //         println!("PostProcessGroup::resize() error: failed to resize texture bind group!! {e}")
    //     }
    //    }

       self.width = width;
       self.height = height;
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
       return &self.texture.view
    }

    pub fn get_emissive_view(&self) -> &wgpu::TextureView {
        return &self.emissive_texture.view
    }

    pub fn get_hdr_texture(&self) -> &Texture {
        return &self.texture
    }

    pub fn get_emmisive_texture(&self) -> &Texture {
        return &self.emissive_texture
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

       let pipeline = PipelineBuilder::new(
            "postprocess pipeline",
            &[&self.bind_group_layout],
            &[],
            &shader_module,
            [ctx.config.format.add_srgb_suffix()],
        )
        .build(&ctx.device);

       self.pipeline = pipeline;
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, swapchain_view: &wgpu::TextureView, ctx: &WgpuContext, final_blur_texture: &Texture) {

self.bind_group = BindGroupManager::create_multi_texture_bind_group(
    &ctx.device,
    &self.bind_group_layout,
    &[&self.texture, &final_blur_texture],
).unwrap();
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