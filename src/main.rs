use std::sync::Arc;

use winit::{application::ApplicationHandler, error::EventLoopError, event::{DeviceEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::Window};
use yhwh::{common::constants::{WINDOW_HEIGHT, WINDOW_WIDTH}, engine::Engine};

pub struct App {
    engine: Option<Engine>,
}

impl App {
    pub fn new() -> Self {
        return Self {
            engine: None,
        };
    }
}

impl ApplicationHandler<Engine> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes().with_title("yhwh").with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.engine = Some(pollster::block_on(Engine::new(window)));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Engine) {
        self.engine = Some(event);
    }
    
    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        let engine = self.engine.as_mut().unwrap();

        engine.handle_device_events(&event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        #[allow(unused_mut)]
        let mut engine = self.engine.as_mut().unwrap();

        // general events
        engine.handle_window_events(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                engine.update();
            }
            _ => {}
        }
    }
}

impl App {
    pub fn run() -> Result<(), EventLoopError> {
      env_logger::init();

      let event_loop = EventLoop::with_user_event().build()?;

      let mut app = App::new();
      event_loop.run_app(&mut app)?;

      Ok(())
   }
}

fn main() {
    App::run().unwrap();
}
