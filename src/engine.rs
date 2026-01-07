use std::sync::Arc;

use winit::{event::{DeviceEvent, WindowEvent}, keyboard::KeyCode, window::{CursorGrabMode, Window}};

use crate::{asset_manager::AssetManager, camera::{Camera, CameraController, Projection}, common::{constants::{WINDOW_HEIGHT, WINDOW_WIDTH}, create_info::{GameObjectCreateInfo, MeshNodeCreateInfo}, enums::GameState}, input::{input::Input, keyboard::Keyboard, mouse::Mouse}, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject}, physics::physics::Physics, scene::Scene, utils::json::load_level, wgpu_renderer::WgpuRenderer};

pub struct GameData {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub asset_manager: AssetManager,
    pub scene: Scene,
    pub delta_time: std::time::Duration,
    pub last_redraw: std::time::Instant,
    pub fps_accum: Vec<f64>,
    pub avg_fps: f64,
    pub game_state: GameState
}

pub struct Engine {
    window: Arc<Window>,
    wgpu_renderer: WgpuRenderer,
    physics: Physics,
    game_data: GameData,
    input: Input,
    show_cursor: bool,
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> Self {
        // window config
        let show_cursor = false;
        window.set_cursor_visible(show_cursor);
        let _res = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked));

        // load camera
        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = CameraController::new(8.0, 0.4);

        // load physics

        // load resources
        let wgpu_context = WgpuRenderer::create_context(&window).await;
        let mut asset_manager = AssetManager::new(&wgpu_context);
        asset_manager.build_materials(&wgpu_context.device);
    
        // load scene
        let scene = Scene::new(&asset_manager);

        let game_data = GameData {
            asset_manager,
            scene,
            camera,
            camera_controller,
            avg_fps: 0.0,
            fps_accum: Default::default(),
            delta_time: std::time::Duration::new(0, 0),
            last_redraw: std::time::Instant::now(),
            game_state: GameState::Playing
        };

        // load wgpu
        let wgpu_renderer = WgpuRenderer::new(&window, wgpu_context, &game_data);

        Self {
            physics: Physics::new(),
            wgpu_renderer,
            window,
            input: Input::new(),
            show_cursor,
            game_data,
        }
    }

    pub fn update(&mut self) {
        // update physics
        //self.physics.step_simulation(self.game_data.delta_time);

        // update game
        self.game_data.update_fps();
        self.game_data.camera_controller.update_camera(&mut self.game_data.camera, self.game_data.delta_time);

        self.window.set_title(&format!("FPS: {:.1}", self.game_data.avg_fps));
        self.toggle_cursor();

        self.handle_dev_tools();
        self.update_object_position();

        // update wgpu renderer
        match self.wgpu_renderer.render(&self.window, &mut self.game_data) {
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
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.game_data.camera.get_projection_mut().resize(width, height);
        self.wgpu_renderer.resize(width, height);
    }

    pub fn handle_window_events(&mut self, event: &WindowEvent) {
        self.game_data.camera_controller.handle_keyboard(&event);
        self.input.keyboard.handle_event(&event);
        self.wgpu_renderer.egui_renderer.handle_input(&self.window, &event);
        self.wgpu_renderer.egui_renderer.set_cursor_visible(self.show_cursor);
    }

    pub fn handle_device_events(&mut self, event: &DeviceEvent) {
         match event {
            DeviceEvent::MouseMotion { delta } => {
               self.input.mouse.handle_mouse_motion(delta.0, delta.1);
               if !self.show_cursor {
                 self.game_data.camera_controller.handle_mouse(delta.0, delta.1);
               }
            }
            _ => {}
        }
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
        if self.input.keyboard.key_just_pressed(KeyCode::Digit2) {
          self.wgpu_renderer.hot_load_shaders();
        }

        if let Some(glb_model) = self.game_data.asset_manager.get_model_by_name_mut("glock") {
            let anim_len = glb_model.animations.as_ref().unwrap().animations().len();
            if self.input.keyboard.key_just_pressed(KeyCode::KeyR) {
                let play_back_state = glb_model.get_animation_playback_state().unwrap();
                let mut current_anim = play_back_state.current;

                if current_anim + 1 < anim_len {
                  current_anim += 1;
                } else {
                  current_anim = 0;
                }

                glb_model.set_current_animation(current_anim);
            }
        }
    }

     // TODO: MOVE OUT OF HERE!!
    pub fn update_object_position(&mut self) {
        for game_object in self.game_data.scene.game_objects.iter_mut() {
            if self.game_data.game_state == GameState::Editor && game_object.is_selected {
              if self.input.keyboard.key_pressed(KeyCode::KeyR) {
                let sensitivity = 0.05;
                game_object.get_position_mut().x += self.input.mouse.delta_x as f32 * sensitivity;
             }
           }  
        }
    }
}

impl GameData {
    pub fn update_fps(&mut self) {
        let now = std::time::Instant::now();
        self.delta_time = now - self.last_redraw;
        self.last_redraw = now;

        let fps = 1.0 / self.delta_time.as_secs_f64();
        self.fps_accum.push(fps);
            if self.fps_accum.len() > 100 {
            self.fps_accum.remove(0);
        }

        self.avg_fps = self.fps_accum.iter().sum::<f64>() / self.fps_accum.len() as f64;
    }
}
