use std::{collections::HashMap, sync::Arc};
use winit::{event::{DeviceEvent, WindowEvent}, keyboard::KeyCode, window::{CursorGrabMode, Window}};
use yhwh_audio::audio_manager::AudioManager;
use yhwh_core::common::enums::GameState;

use crate::{asset_manager::AssetManager, camera::{Camera, CameraController}, egui_renderer::egui_renderer::EguiRenderer, game::game_data::GameData, input::input::Input, physics::physics::Physics, renderer_core::render_data_manager::RenderDataManager, wgpu_renderer::{FinalTexture, WgpuRenderer}};


pub struct Engine {
    window: Arc<Window>,
    wgpu_renderer: WgpuRenderer,
    render_data_manager: RenderDataManager,
    physics: Physics,
    game_data: GameData,
    input: Input,
    show_cursor: bool,
    audio_manager: AudioManager
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> Self {
        // window config
        let show_cursor = false;
        window.set_cursor_visible(show_cursor);
        let _res = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked));

        // load resources
        let wgpu_context = WgpuRenderer::create_context(&window).await;
        let mut asset_manager = AssetManager::new(&wgpu_context);
        asset_manager.build_materials(&wgpu_context.device);
        
        let game_data = GameData::new(asset_manager, (window.inner_size().width as f32, window.inner_size().height as f32));

        // load wgpu
        let wgpu_renderer = WgpuRenderer::new(&window, wgpu_context, &game_data);

        let mut audio_manager = AudioManager::new();
        audio_manager.load_audios("res/audio");

        Self {
            physics: Physics::new(),
            wgpu_renderer,
            render_data_manager: RenderDataManager::new(),
            window,
            input: Input::new(),
            show_cursor,
            game_data,
            audio_manager
        }
    }

    pub fn update(&mut self) {
        // update physics
        //self.physics.step_simulation(self.game_data.delta_time);

        // update game
        self.game_data.update(&self.input, &mut self.audio_manager, &mut self.render_data_manager);

        self.window.set_title(&format!("FPS: {:.1}", self.game_data.avg_fps));
        self.toggle_cursor();

        self.handle_dev_tools();

        // update wgpu renderer
        match self.wgpu_renderer.render(&self.window, &mut self.game_data, &mut self.render_data_manager, &self.input) {
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.wgpu_renderer.resize_ctx(size.width, size.height);
            }
            Err(e) => {
                println!("Engine::update() error: Unable to render {}", e);
            }
        }

        self.input.keyboard.end_frame();
        self.input.mouse.end_frame();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.game_data.camera.get_projection_mut().resize(width, height);
        self.wgpu_renderer.resize(width, height);
    }

    pub fn handle_window_events(&mut self, event: &WindowEvent) {
        self.input.keyboard.handle_event(&event);
        self.input.mouse.handle_window_event(&event);
        self.wgpu_renderer.egui_renderer.handle_input(&self.window, &event);
        self.wgpu_renderer.egui_renderer.set_cursor_visible(self.show_cursor);
    }

    pub fn handle_device_events(&mut self, event: &DeviceEvent) {
        self.input.mouse.handle_device_event(&event);
    }

    pub fn toggle_cursor(&mut self) {
         if self.input.keyboard.key_just_pressed(KeyCode::F1) {
            self.show_cursor = !self.show_cursor;
            self.window.set_cursor_visible(self.show_cursor);

            if self.show_cursor {
                self.game_data.game_state = GameState::Editor;
                let _res = self.window.set_cursor_grab(CursorGrabMode::None);
            } else {
                self.game_data.game_state = GameState::Playing;
                let _res = self.window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_e| self.window.set_cursor_grab(CursorGrabMode::Locked));
            }
        }
    }

    pub fn handle_dev_tools(&mut self) {
        if self.input.keyboard.key_just_pressed(KeyCode::F11) {
          self.wgpu_renderer.hot_load_shaders();
        }

        // if self.game_data.game_state == GameState::Editor {
        //     if self.input.keyboard.key_just_pressed(KeyCode::Digit1) {
        //         self.wgpu_renderer.set_final_texture(FinalTexture::Lighting);
        //     } else if self.input.keyboard.key_just_pressed(KeyCode::Digit2) {
        //         self.wgpu_renderer.set_final_texture(FinalTexture::Albedo);
        //     } else if self.input.keyboard.key_just_pressed(KeyCode::Digit3) {
        //         self.wgpu_renderer.set_final_texture(FinalTexture::Normal);
        //     }
        // }
    }
}