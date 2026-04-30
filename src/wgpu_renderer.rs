use std::{sync::Arc};

use winit::{window::Window};

use crate::{common::enums::GameState, egui_renderer::{egui_renderer::EguiRenderer, ui_manager::UiManager}, game::game_data::GameData,  render_passes::{animation_pass::AnimationPass, emissive_pass::EmissivePass, glass_pass::GlassPass, outline_pass::OutlinePass, postprocess_pass::PostProcessPass, scene_pass::ScenePass, shadow_pass::ShadowPass, skybox_pass::SkyboxPass, ui_pass::UiPass}, uniform_manager::UniformManager, wgpu_context::WgpuContext};

pub struct WgpuRenderer {
    pub egui_renderer: EguiRenderer,
    pub wgpu_context: WgpuContext,
    postprocess_pass: PostProcessPass,
    scene_pass: ScenePass,
    animation_pass: AnimationPass,
    skybox_pass: SkyboxPass,
    outline_pass: OutlinePass,
    emissive_pass: EmissivePass,
    shadow_pass: ShadowPass,
    glass_pass: GlassPass,
    ui_pass: UiPass,
    uniform_manager: UniformManager,
    ui_manager: UiManager,
}

impl WgpuRenderer {
    pub async fn create_context(window: &Arc<Window>) -> WgpuContext {
        let context = WgpuContext::new(&window).await.unwrap();
        context
    }

    pub fn new(window: &Arc<Window>, context: WgpuContext, game_data: &GameData) -> Self {
        // init wgpu
        let config = context.get_surface_config();

        // init egui
        let mut egui_renderer = EguiRenderer::new(&context, &window);
        let mut ui_manager = UiManager::new();
        ui_manager.register_textures(&context, &mut egui_renderer.renderer, &game_data.asset_manager);

        // load uniforms
        let shadow_pass = ShadowPass::new(&context, &game_data);
        let wgpu_uniforms = UniformManager::new(&context, &game_data.scene, &shadow_pass.shadow_cube_map_array.texture);

        // load render groups
        let scene_pass = ScenePass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let animation_pass = AnimationPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let skybox_pass = SkyboxPass::new(&context, &game_data.asset_manager, &wgpu_uniforms);
        let outline_pass = OutlinePass::new(&context, &wgpu_uniforms);
        let emissive_pass = EmissivePass::new(&context, &wgpu_uniforms, &scene_pass.emissive_texture);
        let glass_pass = GlassPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let postprocess_pass = PostProcessPass::new(&context, &config, &scene_pass.pbr_texture, &emissive_pass.get_final_texture(), &outline_pass.get_outline_texture(), &glass_pass.texture);
        let ui_pass = UiPass::new(&context, &game_data, &wgpu_uniforms);

        return Self {
            wgpu_context: context,
            egui_renderer,
            shadow_pass,
            scene_pass,
            postprocess_pass,
            animation_pass,
            skybox_pass,
            outline_pass,
            emissive_pass,
            glass_pass,
            ui_pass,
            uniform_manager: wgpu_uniforms,
            ui_manager
        };
    }

    pub fn render(&mut self, window: &Window, game_data: &mut GameData) -> Result<(), wgpu::SurfaceError> {
        // submit uniforms
        self.uniform_manager.submit_animation_uniforms(&self.wgpu_context, &mut game_data.asset_manager, game_data.delta_time);
        self.uniform_manager.submit_model_uniforms(&self.wgpu_context, &game_data.scene);
        self.uniform_manager.submit_camera_uniforms(&self.wgpu_context, &game_data.active_camera());
        self.uniform_manager.submit_light_uniforms(&self.wgpu_context, &game_data.scene, &self.shadow_pass.shadow_cube_map_array.texture);
        
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
            label: Some("render encoder"),
        });

       self.shadow_pass.render(&mut encoder, &self.wgpu_context, &mut self.uniform_manager, &game_data);
       self.scene_pass.render(&mut encoder, &self.uniform_manager, &game_data);
       self.animation_pass.render(&mut encoder, &self.uniform_manager, &game_data, &self.scene_pass.pbr_texture, &self.scene_pass.depth_texture);
       self.skybox_pass.render(&mut encoder, &self.uniform_manager, &self.scene_pass.pbr_texture, &self.scene_pass.depth_texture);
       self.outline_pass.render(&mut encoder, &self.uniform_manager, &game_data);
       self.emissive_pass.render(&mut encoder, &self.wgpu_context, &mut self.uniform_manager);
       self.glass_pass.render(&mut encoder, &self.uniform_manager, &self.scene_pass.depth_texture, &game_data);
       self.postprocess_pass.render(&mut encoder, &swapchain_view, &self.wgpu_context, &self.scene_pass.pbr_texture, &self.emissive_pass.get_final_texture(), self.outline_pass.get_outline_texture(), &self.glass_pass.texture);
       self.ui_pass.render(&self.wgpu_context, &mut encoder, &game_data, (window.inner_size().width as f32, window.inner_size().height as f32), &swapchain_view);

       if game_data.game_state == GameState::Editor {
        self.egui_renderer.draw(&self.wgpu_context, &mut encoder, &window, swapchain_view, |ui| {
          self.ui_manager.scene_hierarchy_window.draw(ui, &self.ui_manager.materials, game_data, (window.inner_size().width, window.inner_size().height));
        });
       }

       queue.submit(std::iter::once(encoder.finish()));
       swapchain_fbo.present();

       Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
          return;
        }

        self.wgpu_context.resize(width, height);
        self.postprocess_pass.resize(&self.wgpu_context.get_device(), width, height, &self.emissive_pass.get_final_texture());
    }

    pub fn resize_ctx(&mut self, width: u32, height: u32) {
        self.wgpu_context.resize(width, height);
    }

    pub fn hot_load_shaders(&mut self) {
         self.outline_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.postprocess_pass.hotload_shader(&self.wgpu_context);
         self.scene_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.emissive_pass.hotload_shader(&self.wgpu_context.device, &self.uniform_manager);
         self.glass_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         println!("Hot-Loaded shaders!");
    }
}