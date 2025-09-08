use image::GenericImageView;

use crate::{bind_group_manager::{BindGroupManager, TL}, texture::Texture};

pub struct CubeMap {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout
}

impl CubeMap {
    pub fn new_from_image(device: &wgpu::Device, queue: &wgpu::Queue, image_faces: [&image::DynamicImage; 6]) -> Self {
       let dimensions = image_faces[0].dimensions();

       let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 6,
        };

         let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Cube_Map_Texture"),
            view_formats: &[],
        });

        for (i, image) in image_faces.iter().enumerate() {
             queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                 origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &image.to_rgba8(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
             wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
          );
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let cube_tex = Texture { 
            sampler: texture_sampler.clone(),
            view: texture_view.clone(),
            texture: texture.clone(),
            dimensions: Default::default(),
            pixel_data: Default::default()
        };

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Cube]).unwrap();
        let bind_group = BindGroupManager::create_texture_bind_group(&device, &bind_group_layout, &cube_tex).unwrap();

        Self { 
            texture,
            view: texture_view,
            sampler: texture_sampler,
            texture_bind_group: bind_group,
            texture_bind_group_layout: bind_group_layout
        }
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, dimensions: (u32, u32), face_bytes: [&[u8]; 6]) -> Self {
         let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 6,
        };

         let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Cube_Map_Texture"),
            view_formats: &[],
        });

        for (layer, data) in face_bytes.iter().enumerate() {
            queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                 origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            *data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
             wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );
        }

         let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

         let cube_tex = Texture { 
            sampler: texture_sampler.clone(),
            view: texture_view.clone(),
            texture: texture.clone(),
            dimensions: Default::default(),
            pixel_data: Default::default()
        };

        let bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Cube]).unwrap();
        let bind_group = BindGroupManager::create_texture_bind_group(&device, &bind_group_layout, &cube_tex).unwrap();

        Self { 
            texture,
            view: texture_view,
            sampler: texture_sampler,
            texture_bind_group: bind_group,
            texture_bind_group_layout: bind_group_layout
         }
    }

}