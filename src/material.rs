use crate::{bind_group_manager::{BindGroupManager, TL}, texture::Texture};

pub struct Material {
   pub name: String,
   pub bind_group_layout: wgpu::BindGroupLayout,
   pub bind_group: wgpu::BindGroup
}

impl Material {
    pub fn new(name: &str, device: &wgpu::Device, textures: [&Texture; 3]) -> Self {
       let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Float, TL::Float]).unwrap();

       let bind_group = BindGroupManager::create_multi_texture_bind_group(
        &device,
        &bind_group_layout,
        &[&textures[0], &textures[1]])
       .unwrap();

       Self {
        name: name.to_string(),
        bind_group,
        bind_group_layout, 
       }
    }
}