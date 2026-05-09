use crate::{texture::Texture, wgpu_context::WgpuContext};

pub struct ShadowCubeMapArray {
    pub texture: Texture,
    pub face_views: Vec<wgpu::TextureView>,
    pub size: usize,
    pub capacity: usize,
}

impl ShadowCubeMapArray {
    pub fn new(ctx: &WgpuContext, size: u32, light_count: u32) -> Self {
        let layer_count = light_count * 6;

        let cube_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow cube map array texture"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // 6 views per light
        let mut face_views = Vec::new();
        for i in 0..layer_count {
            let view = cube_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("shadow cube face view"),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: i,
                array_layer_count: Some(1),
                ..Default::default()
            });
            face_views.push(view);
        }

        // sampling stuff
        let cube_array_view = cube_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow cube map array view"),
            dimension: Some(wgpu::TextureViewDimension::CubeArray),
            base_array_layer: 0,
            array_layer_count: Some(light_count * 6),
            ..Default::default()
        });

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            face_views,
            capacity: light_count.next_power_of_two() as usize,
            size: size as usize,
            texture: Texture {
                gpu_texture: cube_texture,
                sampler,
                view: cube_array_view,
                dimensions: Default::default(),
                pixel_data: Default::default()
            }
        }
    }
}
