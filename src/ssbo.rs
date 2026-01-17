use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::util::DeviceExt;

use crate::{u8slice::ToU8SliceArray, wgpu_context::WgpuContext};

pub struct SSBO {
    pub value_buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer_size: u64
}

impl SSBO
 {
    pub fn new(size: u64, device: &wgpu::Device) -> Self {
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
                }],
                label: Some("SSBO bind group layout"),
         });

         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                 binding: 0,
                 resource: value_buffer.as_entire_binding(),
             }, wgpu::BindGroupEntry { 
                binding: 1,
                resource: count_buffer.as_entire_binding() 
              }
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

    pub fn update<T: bytemuck::Pod>(&mut self, ctx: &WgpuContext, size: u64, data: &Vec<T>) {
        self.ensure_capacity(&ctx.device, size);

        ctx.queue.write_buffer(&self.value_buffer, 0, data.as_slice().cast_slice());

        let count = data.len() as u32;
        ctx.queue.write_buffer(&self.count_buffer, 0, bytemuck::bytes_of(&count));
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, size: u64) {
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

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                 binding: 0,
                 resource: self.value_buffer.as_entire_binding(),
             }, wgpu::BindGroupEntry { 
                binding: 1,
                resource: self.count_buffer.as_entire_binding() 
              }
            ],
            label: Some("SSBO uniform bind group"),
        });

    }
}
