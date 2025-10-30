use crate::{instance::InstanceUniform, texture, vertex::Vertex};

pub struct PipelineManager;

impl PipelineManager {
   pub fn create_pipeline(device: &wgpu::Device, pipeline_layout: &wgpu::PipelineLayout, texture_format: wgpu::TextureFormat, depth_format: Option<wgpu::TextureFormat>, shader_module: &wgpu::ShaderModule, vertex_layouts: &[wgpu::VertexBufferLayout], label: Option<&str>) -> anyhow::Result<wgpu::RenderPipeline> {
       let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,//Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
           }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(render_pipeline)
    }

    pub fn create_stencil_pipeline(device: &wgpu::Device, pipeline_layout: &wgpu::PipelineLayout, texture_format: wgpu::TextureFormat, depth_format: Option<wgpu::TextureFormat>, shader_module: &wgpu::ShaderModule, vertex_layouts: &[wgpu::VertexBufferLayout], label: Option<&str>) -> anyhow::Result<wgpu::RenderPipeline> {
         let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,//Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState { 
                    compare: wgpu::CompareFunction::Always,
                    fail_op: wgpu::StencilOperation::Replace,
                    depth_fail_op: wgpu::StencilOperation::Replace,
                    pass_op: wgpu::StencilOperation::Replace,
                 },
                back: wgpu::StencilFaceState::default(),
                read_mask: 0xFF,
                write_mask: 0xFF, 
            },
            bias: wgpu::DepthBiasState::default(),
           }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(render_pipeline)
    }

     pub fn create_cubemap_pipeline(device: &wgpu::Device, pipeline_layout: &wgpu::PipelineLayout, texture_format: wgpu::TextureFormat, shader_module: &wgpu::ShaderModule) -> anyhow::Result<wgpu::RenderPipeline> {
       let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cube_Map_Render_Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                array_stride:  3 * std::mem::size_of::<f32>() as wgpu::BufferAddress, 
                step_mode: wgpu::VertexStepMode::Vertex,                         
                attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                 }
              ]
           }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
             depth_stencil: Some(wgpu::DepthStencilState {
             format: texture::Texture::DEPTH_FORMAT,
             depth_write_enabled: false,
             depth_compare: wgpu::CompareFunction::LessEqual, // 1.
             stencil: wgpu::StencilState::default(), // 2.
            bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(render_pipeline)
    }
}