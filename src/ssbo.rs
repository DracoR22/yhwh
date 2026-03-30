use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::util::DeviceExt;

use crate::{texture::Texture, u8slice::ToU8SliceArray, wgpu_context::WgpuContext};

pub struct SSBO {
    pub value_buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer_size: u64
}

impl SSBO {
    pub fn new(size: u64, device: &wgpu::Device, shadow_texture: &Texture) -> Self {
        let value_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSBO Value Buffer"),
            size,
            usage:  wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSBO Count Buffer"),
            contents: bytemuck::bytes_of(&0u32),
            usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }, wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // TODO!! GET THIS OUT OF HERE. RIGHT NOW IS USED TO STORE SHADOW VALUES ON LIGHTS SSBO DUE TO WGPU LIMIT OF 4 GROUPS PER SHADER
                    wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                view_dimension: wgpu::TextureViewDimension::CubeArray,
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                    wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Sampler(
                                wgpu::SamplerBindingType::Comparison
                            ),
                            count: None,
                    },
                ],
                label: Some("SSBO bind group layout"),
         });

         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                   binding: 0,
                   resource: value_buffer.as_entire_binding(),
             }, wgpu::BindGroupEntry { 
                   binding: 1,
                   resource: count_buffer.as_entire_binding() 
             },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&shadow_texture.sampler),
                },
            ],
            label: Some("SSBO uniform bind group"),
         });

         Self {
            bind_group,
            bind_group_layout,
            value_buffer,
            count_buffer,
            buffer_size: size
         }
    }

    pub fn update<T: bytemuck::Pod>(&mut self, ctx: &WgpuContext, size: u64, data: &Vec<T>, shadow_texture: &Texture) {
        self.ensure_capacity(&ctx.device, size, shadow_texture);

        ctx.queue.write_buffer(&self.value_buffer, 0, data.as_slice().cast_slice());

        let count = data.len() as u32;
        ctx.queue.write_buffer(&self.count_buffer, 0, bytemuck::bytes_of(&count));
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, size: u64, shadow_texture: &Texture) {
        if size == 0 || self.buffer_size >= size {
          return;
        }

        self.buffer_size = size;

        self.value_buffer = device.create_buffer(&wgpu::BufferDescriptor {
          label: Some("SSBO value buffer (resized)"),
          size: size,
          usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
          mapped_at_creation: false,
        });

        // self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
        //     layout: &self.bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //          binding: 0,
        //          resource: self.value_buffer.as_entire_binding(),
        //      }, wgpu::BindGroupEntry { 
        //         binding: 1,
        //         resource: self.count_buffer.as_entire_binding() 
        //       },
        //        wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 3,
        //             resource: wgpu::BindingResource::Sampler(&shadow_texture.sampler),
        //         },
        //     ],
        //     label: Some("SSBO uniform bind group"),
        // });

    }

    pub fn rebuild_bind_group(&mut self, device: &wgpu::Device, texture: &Texture) {
         self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                 binding: 0,
                 resource: self.value_buffer.as_entire_binding(),
             }, wgpu::BindGroupEntry { 
                binding: 1,
                resource: self.count_buffer.as_entire_binding() 
              },
               wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("SSBO uniform bind group"),
        });
    }
}
