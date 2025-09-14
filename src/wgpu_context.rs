use std::sync::Arc;

use winit::window::Window;

pub struct WgpuContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
}

#[derive(Debug)]
pub enum ContextError {
    RequestDeviceError(wgpu::RequestDeviceError),
    NoAdapterFound
}

impl WgpuContext {
    pub async fn new(window: &Arc<Window>) -> Result<Self, ContextError> {
       let size = window.inner_size();

       let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(ContextError::NoAdapterFound)?;

        let info = adapter.get_info();
        println!("Using Backend: {:?}", info.backend); 

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(ContextError::RequestDeviceError)?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self { 
            config,
            device, 
            is_surface_configured: true,
            queue,
            surface
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
         if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn get_device(&self) -> &wgpu::Device {
        return &self.device;
    }

    pub fn get_surface(&self) -> &wgpu::Surface<'static> {
        return &self.surface;
    }

    pub fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration {
       return &self.config;
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        return &self.queue;
    }

    pub fn is_surface_configured(&self) -> bool {
        return self.is_surface_configured;
    }
}