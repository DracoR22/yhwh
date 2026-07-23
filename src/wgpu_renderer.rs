use std::{sync::Arc};

use winit::{window::Window};
use yhwh_core::common::enums::GameState;

use crate::{egui_renderer::{egui_renderer::EguiRenderer, ui_manager::UiManager}, game::game_data::GameData, input::input::Input, render_passes::{animation_pass::AnimationPass, emissive_pass::EmissivePass, fire_pass::FirePass, flame_pass::FlamePass, geometry_pass::GeometryPass, glass_pass::GlassPass, lighting_pass::LightingPass, outline_pass::OutlinePass, postprocess_pass::PostProcessPass, scene_pass::ScenePass, shadow_pass::ShadowPass, skybox_pass::SkyboxPass, ssao_pass::SSAOPass, ui_pass::UiPass}, renderer_core::render_data_manager::RenderDataManager, renderer_utils::final_texture, texture::Texture, uniform_manager::UniformManager, wgpu_context::WgpuContext};

struct RenderPasses {
    shadow_pass: ShadowPass,
    geometry_pass: GeometryPass,
    ssao_pass: SSAOPass,
    lighting_pass: LightingPass,
    animation_pass: AnimationPass,
    skybox_pass: SkyboxPass,
    outline_pass: OutlinePass,
    emissive_pass: EmissivePass,
    glass_pass: GlassPass,
    flame_pass: FlamePass,
    fire_pass: FirePass,
    postprocess_pass: PostProcessPass,
    ui_pass: UiPass,
}

pub enum FinalTexture {
    Lighting,
    Albedo,
    Normal,
    Ssao
}

pub struct WgpuRenderer {
    pub egui_renderer: EguiRenderer,
    pub wgpu_context: WgpuContext,
    uniform_manager: UniformManager,
    ui_manager: UiManager,
    render_passes: RenderPasses,
    final_texture: FinalTexture,
}

pub enum EditorCommand {
    SetFinalTexture(FinalTexture)
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
        let mut wgpu_uniforms = UniformManager::new(&context, &game_data, &shadow_pass.shadow_cube_map_array.texture);

        // load render groups
        let geometry_pass = GeometryPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let ssao_pass = SSAOPass::new(&context, &geometry_pass.textures, &wgpu_uniforms);
        let lighting_pass = LightingPass::new(&context, &geometry_pass.textures, &ssao_pass.blur_texture, &wgpu_uniforms);
        let scene_pass = ScenePass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let animation_pass = AnimationPass::new(&context, &mut wgpu_uniforms, &game_data);
        let skybox_pass = SkyboxPass::new(&context, &game_data.asset_manager, &wgpu_uniforms);
        let outline_pass = OutlinePass::new(&context, &wgpu_uniforms);
        let emissive_pass = EmissivePass::new(&context, &wgpu_uniforms);
        let glass_pass = GlassPass::new(&context, &wgpu_uniforms, &game_data.asset_manager);
        let flame_pass = FlamePass::new(&context);
        let fire_pass = FirePass::new(&context, &game_data.asset_manager, &wgpu_uniforms);
        let postprocess_pass = PostProcessPass::new(&context, &config, &scene_pass.pbr_texture, &emissive_pass.get_final_texture(), &outline_pass.outline_texture(), &glass_pass.texture);
        let ui_pass = UiPass::new(&context, &game_data, &wgpu_uniforms);

        return Self {
            wgpu_context: context,
            egui_renderer,
            uniform_manager: wgpu_uniforms,
            ui_manager,
            final_texture: FinalTexture::Lighting,
            render_passes: RenderPasses { 
                lighting_pass,
                shadow_pass,
                geometry_pass,
                ssao_pass,
                postprocess_pass,
                animation_pass,
                skybox_pass,
                outline_pass,
                emissive_pass,
                glass_pass,
                flame_pass,
                fire_pass,
                ui_pass,
             }
        };
    }

    pub fn render(&mut self, window: &Window, game_data: &mut GameData, render_data_manager: &mut RenderDataManager, input: &Input) -> Result<(), wgpu::SurfaceError> {
        let RenderPasses { 
            shadow_pass,
            geometry_pass,
            ssao_pass,
            lighting_pass,
            animation_pass,
            skybox_pass,
            outline_pass,
            emissive_pass,
            glass_pass,
            flame_pass,
            fire_pass,
            postprocess_pass,
            ui_pass 
        } = &mut self.render_passes;

        // submit uniforms
        self.uniform_manager.submit_model_uniforms(&self.wgpu_context, &render_data_manager);
        self.uniform_manager.submit_camera_uniforms(&self.wgpu_context, &game_data.active_camera());
        self.uniform_manager.submit_light_uniforms(&self.wgpu_context, &game_data, &shadow_pass.shadow_cube_map_array.texture);
        
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

        shadow_pass.render(&mut encoder, &self.wgpu_context, &mut self.uniform_manager, &game_data);
        geometry_pass.render(&mut encoder, &self.uniform_manager, &game_data, render_data_manager);
        animation_pass.render(&mut encoder, &self.wgpu_context, &mut self.uniform_manager, game_data, render_data_manager, &geometry_pass.textures, &lighting_pass.texture, &geometry_pass.textures.depth, &outline_pass.mask_texture());
        ssao_pass.render(&mut encoder, &self.wgpu_context, &self.uniform_manager);
        lighting_pass.render(&mut encoder, &self.wgpu_context, &self.uniform_manager, &geometry_pass.textures);

        emissive_pass.render_mask(&mut encoder, &self.uniform_manager, &game_data, &render_data_manager, &lighting_pass.texture, &geometry_pass.textures.depth);
        flame_pass.render(&mut encoder, &self.wgpu_context, &self.uniform_manager, game_data, render_data_manager, &lighting_pass.texture, &emissive_pass.mask_texture, &geometry_pass.textures.depth);
        fire_pass.render(&mut encoder, &self.wgpu_context, &self.uniform_manager, &render_data_manager, game_data, &lighting_pass.texture, &geometry_pass.textures.depth);
        skybox_pass.render(&mut encoder, &self.uniform_manager, &lighting_pass.texture, &geometry_pass.textures.depth);
        outline_pass.render(&mut encoder, &self.uniform_manager, &game_data, &render_data_manager);
        emissive_pass.render(&mut encoder, &self.wgpu_context, &mut self.uniform_manager);
        glass_pass.render(&mut encoder, &self.uniform_manager, &game_data, &render_data_manager, &geometry_pass.textures.depth);

        let final_texture = final_texture(&self.final_texture, geometry_pass, lighting_pass, ssao_pass);

        postprocess_pass.render(&mut encoder, &swapchain_view, &self.wgpu_context, &final_texture, &emissive_pass.get_final_texture(), outline_pass.outline_texture(), &glass_pass.texture);
        ui_pass.render(&self.wgpu_context, &mut encoder, &game_data, (window.inner_size().width as f32, window.inner_size().height as f32), &swapchain_view);

        // egui
        let mut commands = Vec::<EditorCommand>::new();

        if game_data.game_state == GameState::Editor {
            self.egui_renderer.draw(&self.wgpu_context, &mut encoder, &window, swapchain_view, |ui| {
               self.ui_manager.editor_layout.draw(ui, &mut commands, &self.ui_manager.materials, game_data, input, (window.inner_size().width, window.inner_size().height));
            });
        }

        for command in commands.into_iter() {
                match command {
                    EditorCommand::SetFinalTexture(texture) => {
                        self.final_texture = texture;
                    }
                }
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
        self.render_passes.postprocess_pass.resize(&self.wgpu_context.get_device(), width, height, &self.render_passes.emissive_pass.get_final_texture());
    }

    pub fn resize_ctx(&mut self, width: u32, height: u32) {
        self.wgpu_context.resize(width, height);
    }

    pub fn set_final_texture(&mut self, texture: FinalTexture) {
        self.final_texture = texture;
    }

    pub fn hot_load_shaders(&mut self) {
         self.render_passes.outline_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.render_passes.postprocess_pass.hotload_shader(&self.wgpu_context);
         self.render_passes.emissive_pass.hotload_shader(&self.wgpu_context.device, &self.uniform_manager);
         self.render_passes.glass_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.render_passes.lighting_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.render_passes.ssao_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         self.render_passes.flame_pass.hotload_shader(&self.wgpu_context);
         self.render_passes.fire_pass.hotload_shader(&self.wgpu_context, &self.uniform_manager);
         println!("Hot-Loaded shaders!");
    }
}