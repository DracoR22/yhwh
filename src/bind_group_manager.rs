use wgpu::{BindGroupEntry, BindGroupLayoutEntry};

use crate::texture;

/// TextureLayout
pub enum TL {
    Depth,
    DepthMultisampled,
    DepthArray,
    Float,
    NonfilterableFloat,
    NonfilterableFloatMultisampled,
    Cube,
    UInt,
    SInt,
}

pub struct BindGroupManager;

impl BindGroupManager {
    pub fn create_uniform_bind_group_layout(device: &wgpu::Device, visibility: wgpu::ShaderStages, label: Option<&str>) -> anyhow::Result<wgpu::BindGroupLayout> {
       let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label,
         });

         Ok(bind_group_layout)
    }

    pub fn bindgroup_layout_entries(binding_offset: u32, it: impl Iterator<Item = TL>) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> {
        it.enumerate().flat_map(move |(i, bgtype)| {
            std::iter::once(wgpu::BindGroupLayoutEntry {
                binding: binding_offset + (i * 2) as u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                   multisampled: matches!(
                        bgtype,
                        TL::NonfilterableFloatMultisampled | TL::DepthMultisampled
                    ),
                    view_dimension: match bgtype {
                        TL::Cube => wgpu::TextureViewDimension::Cube,
                        TL::DepthArray => wgpu::TextureViewDimension::D2Array,
                        _ => wgpu::TextureViewDimension::D2,
                    },
                    sample_type: match bgtype {
                        TL::Depth | TL::DepthMultisampled | TL::DepthArray => {
                            wgpu::TextureSampleType::Depth
                        },
                        TL::UInt => wgpu::TextureSampleType::Uint,
                        TL::SInt => wgpu::TextureSampleType::Sint,
                         _ => wgpu::TextureSampleType::Float {
                            filterable: !matches!(
                                bgtype,
                                TL::NonfilterableFloat | TL::NonfilterableFloatMultisampled
                            ),
                        },
                    }
                },
                count: None
            }).chain(std::iter::once(wgpu::BindGroupLayoutEntry {
              binding: binding_offset + (i * 2 + 1) as u32,
              visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
              ty: wgpu::BindingType::Sampler(
                    if matches!(bgtype, TL::Depth | TL::DepthMultisampled | TL::DepthArray) {
                        wgpu::SamplerBindingType::Comparison
                    } else {
                        wgpu::SamplerBindingType::Filtering
                    },
                ),
                count: None,
            }))
        })
    }

    pub fn create_texture_bind_group_layout(device: &wgpu::Device, it: impl IntoIterator<Item = TL>) -> anyhow::Result<wgpu::BindGroupLayout> {
        let entries: Vec<BindGroupLayoutEntry> = Self::bindgroup_layout_entries(0, it.into_iter()).collect();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &entries,
                label: Some("Texture_Bind_Group_Layout"),
            });

        Ok(bind_group_layout)
    }

    pub fn create_uniform_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer, label: Option<&str>) -> anyhow::Result<wgpu::BindGroup> {
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label,
        });

        Ok(bind_group)
    }

    pub fn create_texture_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, texture: &texture::Texture) -> anyhow::Result<wgpu::BindGroup> {
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("Texture_Bind_Group"),
        });

        Ok(bind_group)
    }

    pub fn multi_bindgroup_entries<'a>(binding_offset: u32, texs: &'a [&texture::Texture]) -> impl Iterator<Item = BindGroupEntry<'a>> {
        texs.iter().enumerate().flat_map(move |(i, tex)| {
            std::iter::once(BindGroupEntry {
                binding: binding_offset + (i * 2) as u32,
                resource: wgpu::BindingResource::TextureView(&tex.view),
            })
            .chain(std::iter::once(BindGroupEntry {
                binding: binding_offset + (i * 2 + 1) as u32,
                resource: wgpu::BindingResource::Sampler(&tex.sampler),
            }))
        })
    }
//     pub fn multi_bindgroup_entries<'a>(
//     binding_offset: u32,
//     texs: &'a [&texture::Texture],
// ) -> Vec<wgpu::BindGroupEntry<'a>> {
//     let mut entries = Vec::new();

//     for (i, tex) in texs.iter().enumerate() {
//         entries.push(wgpu::BindGroupEntry {
//             binding: binding_offset + (i as u32 * 2),
//             resource: wgpu::BindingResource::TextureView(&tex.view),
//         });

//         entries.push(wgpu::BindGroupEntry {
//             binding: binding_offset + (i as u32 * 2 + 1),
//             resource: wgpu::BindingResource::Sampler(&tex.sampler),
//         });
//     }

//     entries
// }


     pub fn create_multi_texture_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, texs: &[&texture::Texture]) -> anyhow::Result<wgpu::BindGroup> {
      let entries = Self::multi_bindgroup_entries(0, texs).collect::<Vec<_>>();
       //let entries = Self::multi_bindgroup_entries(0, texs);
      let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: None,
        });

        Ok(bind_group)
    }
}