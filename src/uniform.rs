use std::sync::atomic::{AtomicBool, Ordering};

use wgpu::util::DeviceExt;

use crate::{bind_group_manager::BindGroupManager, u8slice::ToU8Slice};


pub struct Uniform<T> {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    value: T,
    pub changed: AtomicBool
}

impl<T> Uniform<T>
where T: ToU8Slice {
    pub fn new(value: T, device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: value.cast_slice(),
            usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
            &device,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            Some(format!("bind_group_layout for {}", std::any::type_name::<T>()).as_ref()))
        .unwrap();

        let bind_group = BindGroupManager::create_uniform_bind_group(
            &device,
            &bind_group_layout,
            &buffer,
            Some(format!("bind_group for {}", std::any::type_name::<T>()).as_ref()))
        .unwrap();

    Self {
        bind_group,
        bind_group_layout, 
        buffer,
        changed: AtomicBool::from(true),
        value
    }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        *self.changed.get_mut() = true;
        &mut self.value
    }

    pub fn set_value(&mut self, new_value: T) {
        self.value = new_value;
        self.changed.store(true, Ordering::SeqCst);
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        if self.changed.swap(false, Ordering::SeqCst) {
            queue.write_buffer(
                &self.buffer,
                0,
                self.value.cast_slice(),
            );
        }
    }

    pub fn update_direct(&self, queue: &wgpu::Queue, value: &T) {
        let data = value.cast_slice();
        queue.write_buffer(&self.buffer, 0, data);
        self.changed.store(false, Ordering::SeqCst);
    }
}