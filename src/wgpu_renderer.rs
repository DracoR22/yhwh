use std::{sync::Arc};

use winit::{window::Window};

use crate::{common::{constants::DEPTH_TEXTURE_STENCIL_FORMAT, create_info::{GameObjectCreateInfo, MeshNodeCreateInfo}}, egui_renderer::{egui_renderer::EguiRenderer, ui_manager::UiManager, windows::scene_hierarchy::SceneHierarchyWindow}, engine::GameData, input::keyboard::Keyboard, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject}, pipeline_manager::PipelineManager, render_passes::{animation_pass::AnimationPass, emissive_pass::EmissivePass, lighting_pass::LightingPass, outline_pass::OutlinePass, postprocess_pass::PostProcessPass, skybox_pass::SkyboxPass}, texture, uniform::Uniform, uniform_manager::{AnimationUniform, CameraUniform, LightUniform, ModelUniform, UniformManager}, utils::unique_id, vertex::Vertex, wgpu_context::{self, WgpuContext}};

pub struct WgpuRenderer {
    pub egui_renderer: EguiRenderer,
    pub wgpu_context: WgpuContext,
    depth_texture: texture::Texture,
    postprocess_pass: PostProcessPass,
    lighting_pass: LightingPass,
    animation_pass: AnimationPass,
    skybox_pass: SkyboxPass,
    outline_pass: OutlinePass,
    emissive_pass: EmissivePass,
    uniform_manager: UniformManager,
    ui_manager: UiManager
}

impl WgpuRenderer {
    pub async fn create_context(window: &Arc<Window>) -> WgpuContext {
        let context = WgpuContext::new(&window).await.unwrap();
        context
    }

    pub fn new(window: &Arc<Window>, context: WgpuContext, game_data: &GameData) -> Self {
        // init wgpu
        let config = context.get_surface_config();
        let device = context.get_device();

        // init egui
        let mut egui_renderer = EguiRenderer::new(&context, &window);
        let mut ui_manager = UiManager::new();
        ui_manager.register_textures(&context, &mut egui_renderer.renderer, &game_data.asset_manager);

        // load uniforms
        let wgpu_uniforms = UniformManager::new(&context, &game_data.scene);

        // load fbos
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture", DEPTH_TEXTURE_STENCIL_FORMAT);

        // load render groups
        let lighting_pass = LightingPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let animation_pass = AnimationPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let skybox_pass = SkyboxPass::new(&context, &game_data.asset_manager, &wgpu_uniforms);
        let outline_pass = OutlinePass::new(&context, &wgpu_uniforms);
        let postprocess_pass = PostProcessPass::new(&context, &config);
        let emissive_pass = EmissivePass::new(&context, &wgpu_uniforms, &game_data.asset_manager, &postprocess_pass.get_emmisive_texture());

        return Self {
            wgpu_context: context,
            depth_texture,
            egui_renderer,
            lighting_pass,
            postprocess_pass,
            animation_pass,
            skybox_pass,
            outline_pass,
            emissive_pass,
            uniform_manager: wgpu_uniforms,
            ui_manager
        };
    }

    pub fn render(&mut self, window: &Window, game_data: &mut GameData) -> Result<(), wgpu::SurfaceError> {
        // submit uniforms
        self.uniform_manager.submit_animation_uniforms(&self.wgpu_context, &mut game_data.asset_manager, game_data.delta_time);
        self.uniform_manager.submit_model_uniforms(&self.wgpu_context, &game_data.scene);
        self.uniform_manager.submit_camera_uniforms(&self.wgpu_context, &game_data.camera);
        self.uniform_manager.submit_light_uniforms(&self.wgpu_context, &game_data.scene);
        
        window.request_redraw();

        let device = self.wgpu_context.get_device();
        let surface = self.wgpu_context.get_surface();
        let queue = self.wgpu_context.get_queue();

        if !self.wgpu_context.is_surface_configured() {
            return Ok(());
        }

        let swapchain_fbo = surface.get_current_texture()?;
        let swapchain_view = swapchain_fbo.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("First_Pass"),
            color_attachments: &[
              Some(wgpu::RenderPassColorAttachment {
                view: &self.postprocess_pass.get_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             }),
              Some(wgpu::RenderPassColorAttachment {
                view: &self.postprocess_pass.get_emissive_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
             })
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
              load: wgpu::LoadOp::Clear(1.0),
              store: wgpu::StoreOp::Store,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: wgpu::StoreOp::Store,
            }),
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

       self.lighting_pass.render(&mut render_pass, &self.uniform_manager, &game_data.asset_manager, &game_data.scene.game_objects);
       self.animation_pass.render(&mut render_pass, &self.uniform_manager, &game_data.asset_manager, &game_data.scene.animated_game_objects);
       self.emissive_pass.render(&mut render_pass, &game_data, &self.uniform_manager, self.postprocess_pass.get_view(), self.postprocess_pass.get_emissive_view(), &self.depth_texture.view);

       // skybox
       self.skybox_pass.render(&mut render_pass, &self.uniform_manager);

       drop(render_pass);

       // post process
       self.outline_pass.render(&mut encoder, &self.postprocess_pass.get_view(), &self.depth_texture.view, &self.uniform_manager, &game_data.scene.game_objects, &game_data.asset_manager);
       self.emissive_pass.render_blur(&mut encoder, &self.wgpu_context, &mut self.uniform_manager, self.postprocess_pass.get_emmisive_texture());
       self.postprocess_pass.render(&mut encoder, &swapchain_view, &self.wgpu_context, &self.emissive_pass.get_final_texture());

       self.egui_renderer.draw(&self.wgpu_context, &mut encoder, &window, swapchain_view, |ui| {
          self.ui_manager.scene_hierarchy_window.draw(ui, &self.ui_manager.materials, game_data, (window.inner_size().width, window.inner_size().height));
       });

       queue.submit(std::iter::once(encoder.finish()));
       swapchain_fbo.present();

       Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
          return;
        }

        self.wgpu_context.resize(width, height);
       // self.depth_texture = texture::Texture::create_depth_texture(&self.wgpu_context.get_device(), &self.wgpu_context.get_surface_config(), "depth_texture", DEPTH_TEXTURE_STENCIL_FORMAT);
        self.emissive_pass.resize(&self.wgpu_context.device, width, height, &self.postprocess_pass.get_emmisive_texture());
        self.postprocess_pass.resize(&self.wgpu_context.get_device(), width, height, &self.emissive_pass.get_final_texture());
    }

    pub fn resize_ctx(&mut self, width: u32, height: u32) {
        self.wgpu_context.resize(width, height);
    }

    pub fn hot_load_shaders(&mut self) {
         self.outline_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.postprocess_pass.hotload_shader(&self.wgpu_context);
         self.lighting_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.emissive_pass.hotload_shader(&self.wgpu_context.device, &self.uniform_manager);
         println!("Hot-Loaded shaders!");
    }
}