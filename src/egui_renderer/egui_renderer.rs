use wgpu::rwh::HasDisplayHandle;
use winit::{event::WindowEvent, window::Window};

use crate::{wgpu_context::WgpuContext};

pub struct EguiRenderer {
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
    pub show_cursor: bool
}

impl EguiRenderer {
    pub fn new(ctx: &WgpuContext, window: &Window) -> Self {
        let egui_state = egui_winit::State::new(
            egui::Context::default(),
            egui::ViewportId::ROOT,
            &window.display_handle().unwrap(),
            Some(window.scale_factor() as f32),
            window.theme(),
            None
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &ctx.device,
            ctx.config.format,
            None,
            1,
            true
        );

        Self {
            context: egui_state.egui_ctx().clone(),
            renderer: egui_renderer,
            state: egui_state,
            show_cursor: false
        }
    }

    pub fn set_cursor_visible(&mut self, is_visible: bool) {
       self.show_cursor = is_visible;
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);

        if let WindowEvent::ScaleFactorChanged { scale_factor, .. } = event {
          self.context.set_pixels_per_point(*scale_factor as f32);
        }
    }

    pub fn draw(
        &mut self,
        ctx: &WgpuContext,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        // show_cursor: bool,
        surface_view: wgpu::TextureView,
        mut run_ui: impl FnMut(&egui::Context)
       ) {
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, {
        let context = &self.context;
        move |_ui| {
            run_ui(context);
          }
        });

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
          size_in_pixels: [ctx.config.width, ctx.config.height],
          pixels_per_point: window.scale_factor() as f32,
        };

        self.state.handle_platform_output(&window, full_output.platform_output);

        window.set_cursor_visible(self.show_cursor);

        for (id, image_delta) in &full_output.textures_delta.set {
           self.renderer.update_texture(&ctx.device, &ctx.queue, *id, image_delta);
        }
        
        let tris = self.context.tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        self.renderer.update_buffers(&ctx.device, &ctx.queue, encoder, &tris, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.renderer.render(&mut render_pass.forget_lifetime(), &tris, &screen_descriptor);
       
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}