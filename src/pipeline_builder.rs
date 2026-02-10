pub struct PipelineBuilder<'a> {
    pub label: &'static str,
    pub layouts: &'a [&'a wgpu::BindGroupLayout],
    pub shader_module: &'a wgpu::ShaderModule,
    pub target_formats: Vec<wgpu::TextureFormat>,
    pub vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
    pub depth_format: Option<wgpu::TextureFormat>,
    pub depth_write: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub cull_mode: Option<wgpu::Face>,
    pub blend: wgpu::BlendState,
    pub stencil_state: wgpu::StencilState
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(
        label: &'static str,
        layouts: &'a [&'a wgpu::BindGroupLayout],
        vertex_buffers: &'a [wgpu::VertexBufferLayout<'a>],
        shader_module: &'a wgpu::ShaderModule,
        target_formats: impl IntoIterator<Item = wgpu::TextureFormat>
    ) -> Self {
        Self {
            label,
            layouts,
            vertex_buffers,
            shader_module,
            target_formats: target_formats.into_iter().collect(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            depth_format: None,
            cull_mode: None,
            stencil_state: Default::default()
        }
    }

    pub fn with_depth(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(format);
        self
    }

    pub fn with_depth_write(mut self) -> Self {
        self.depth_write = true;
        self.depth_compare = wgpu::CompareFunction::Less;
        self
    }

    pub fn with_cull_mode(mut self, face: wgpu::Face) -> Self {
        self.cull_mode = Some(face);
        self
    }

    pub fn with_blend(mut self, blend: wgpu::BlendState) -> Self {
        self.blend = blend;
        self
    }

    pub fn with_stencil_state(mut self, write: bool) -> Self  {
        self.stencil_state.read_mask = 0xFF;
        if write {
            self.stencil_state.front = wgpu::StencilFaceState {
              compare: wgpu::CompareFunction::Always,
              fail_op: wgpu::StencilOperation::Keep,
              depth_fail_op: wgpu::StencilOperation::Keep,
              pass_op: wgpu::StencilOperation::Replace,
            };
            self.stencil_state.back = wgpu::StencilFaceState {
              compare: wgpu::CompareFunction::Always,
              fail_op: wgpu::StencilOperation::Keep,
              depth_fail_op: wgpu::StencilOperation::Keep,
              pass_op: wgpu::StencilOperation::Replace,
            };
            self.stencil_state.write_mask = 0xFF;
            self.depth_compare = wgpu::CompareFunction::Less;
        } else {
            self.stencil_state.front = wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::NotEqual,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            };
            self.stencil_state.back = wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::NotEqual,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Keep,
            };
            self.stencil_state.write_mask = 0x00;
            self.depth_compare = wgpu::CompareFunction::Always;
        }

        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        let targets: Vec<Option<wgpu::ColorTargetState>> = self.target_formats.into_iter().map(|format| {
                Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(self.blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })
            }).collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(self.label),
            bind_group_layouts: self.layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: self.shader_module,
                entry_point: Some("vs_main"),
                buffers: self.vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: self.shader_module,
                entry_point: Some("fs_main"),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: self.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: self.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: self.depth_write,
                depth_compare: self.depth_compare,
                stencil: self.stencil_state,
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }
}