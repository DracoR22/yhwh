use std::{default, path::PathBuf};

use image::{DynamicImage, GenericImageView};

use crate::{bind_group_manager::{self, BindGroupManager, TL}, wgpu_context::WgpuContext};

#[derive(Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub dimensions: (u32, u32),
    pub pixel_data: Vec<u8>
}

impl Texture {
    pub fn decode_texture_from_path(path: &str) -> DynamicImage {
        let bytes = std::fs::read(path).expect(&format!("Could not open texture with path: {path}"));
        let image = image::load_from_memory(&bytes).unwrap();

        return image
    }

    pub fn decode_texture_from_bytes(bytes: &[u8]) -> DynamicImage {
        let image = image::load_from_memory(bytes).unwrap();

        return image
    }

    pub fn allocate_gpu_from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], is_normal_map: bool) -> Self {
        let image = image::load_from_memory(bytes).unwrap();
        return Self::allocate_gpu_from_image(device, queue, &image, is_normal_map);
    }

    pub fn allocate_gpu_from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: &image::DynamicImage, is_normal_map: bool) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

         let format = if is_normal_map {
           wgpu::TextureFormat::Rgba8Unorm
        } else {
           wgpu::TextureFormat::Rgba8UnormSrgb
        };

         let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let mip_count = (dimensions.0.max(dimensions.1) as f32).log2().floor() as u32 + 1;

         let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: mip_count, // 1
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("2D_Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
        
        generate_mips(device, queue, &texture, format, mip_count);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self { 
            texture,
            view: texture_view,
            sampler: texture_sampler,
            dimensions: img.dimensions(),
            pixel_data: rgba.into_raw()
            }
    }
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str, format: wgpu::TextureFormat) -> Self {
         let width = 1920;
        let height = 1080;
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { 
            texture,
            view,
            sampler,
            dimensions: Default::default(),
            pixel_data: Default::default()
         }
    }

    pub fn create_fbo(device: &wgpu::Device, (width, height): (u32, u32), format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> Self {
         let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

         let texture = device.create_texture(&wgpu::TextureDescriptor {
            format,
            usage,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            label: Some("FBO_Texture"),
            view_formats: &[]
         });

         let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            sampler,
            texture, 
            view,
            dimensions: Default::default(),
            pixel_data: Default::default()
        }
    }
}

// helpers
pub trait TextureHelpers {
    fn flip_horizontal(&self) -> Self;
}

impl TextureHelpers for Texture {
    fn flip_horizontal(&self) -> Self {
        let mut new = self.clone();
        let bpp = 4; // RGBA8
        let width = new.dimensions.0 as usize;
        let height = new.dimensions.1 as usize;
        let row_stride = (width * bpp) as usize;

        for y in 0..height {
            let row_start = y as usize * row_stride;
            let row = &mut new.pixel_data[row_start..row_start + row_stride];

            for x in 0..(width / 2) {
                let left = (x as usize) * bpp;
                let right = ((width - 1 - x) as usize) * bpp;

                for i in 0..bpp {
                    row.swap(left + i, right + i);
                }
            }
        }

        new
    }
}

pub struct TextureData {
    pub image: DynamicImage,
    pub name: String
}

fn generate_mips(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, format: wgpu::TextureFormat, mip_count: u32) {
    let shader_code = std::fs::read_to_string("res/shaders/mipmap.wgsl").expect("res/shaders/mipmap.wgsl was not found!");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mip shader"),
        source: wgpu::ShaderSource::Wgsl(shader_code.into())
    });

    let bglayout = BindGroupManager::create_texture_bind_group_layout(device, [TL::Float]).unwrap();

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bglayout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mip pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview: None
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        min_filter: wgpu::FilterMode::Linear,
        mag_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("mip encoder"),
     });

    for mip in 1..mip_count {
        let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: mip - 1,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
            base_mip_level: mip,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    queue.submit(Some(encoder.finish()));
}