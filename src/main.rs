use cgmath::{prelude::*, Matrix4, Rad};

use std::{collections::HashMap, path::PathBuf, sync::{mpsc, Arc}, thread, time::Instant};

use image::GenericImageView;
use wgpu::{core::device, util::DeviceExt};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};
use yhwh::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, bind_group_manager::{BindGroupManager, TL}, camera::{CameraController, Projection}, cube_map::CubeMap, egui_renderer::egui_renderer::EguiRenderer, input::keyboard::Keyboard, instance::{Instance, InstanceUniform}, model::{self, Mesh, Model}, objects::game_object::{GameObject, GameObjectCreateInfo, MeshReneringCreateInfo}, pipeline_manager::PipelineManager, render_passes::{animation_pass::AnimationPass, lighting_pass::LightingPass, postprocess_pass::{self, PostProcessPass}, skybox_pass::SkyboxPass}, renderer_common::SKYBOX_VERTICES, texture::{self, Texture, TextureHelpers}, uniform::Uniform, uniform_types::{AnimationUniform, CameraUniform, LightUniform, ModelUniform, WgpuUniforms}, utils::{file, unique_id}, wgpu_context::WgpuContext};
use yhwh::{
    camera::Camera,
    vertex::Vertex,
};

pub struct State {
    window: Arc<Window>,
    wgpu_context: WgpuContext,
    asset_manager: AssetManager,
    projection: Projection,
    camera: Camera,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    instance_render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    debug_render_pipeline: wgpu::RenderPipeline,
    egui_renderer: EguiRenderer,
    game_objects: Vec<GameObject>,
    animated_game_object_id: usize,

    postprocess_pass: PostProcessPass,
    lighting_pass: LightingPass,
    animation_pass: AnimationPass,
    skybox_pass: SkyboxPass,
    wgpu_uniforms: WgpuUniforms,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        // init wgpu
        let context = WgpuContext::new(&window).await.unwrap();
        let config = context.get_surface_config();
        let device = context.get_device();
        let queue = context.get_queue();

        let egui_renderer = EguiRenderer::new(&context, &window);

        // load textures
        let mut asset_manager = AssetManager::new(&context);
        asset_manager.build_materials(&device);


        // init game objects
        let mut game_objects: Vec<GameObject> = Vec::new();

        let barrel_go_create_info = GameObjectCreateInfo {
            model_name: "Barrel".to_string(),
            name: "Barrel1".to_string(),
            position: cgmath::Vector3::new(0.0, 2.0, 0.0),
            size: cgmath::Vector3::new(5.0, 5.0, 5.0),
            rotation: cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0)),
            mesh_rendering_info: vec![MeshReneringCreateInfo {
                material_name: "barrel_BLUE".to_string(),
                mesh_name: "barrel_YELLOW".to_string()
            }]
        };

        let barrel_go: GameObject = GameObject::new(&barrel_go_create_info, &asset_manager);

         let plane_go_create_info = GameObjectCreateInfo {
            model_name: "Plane".to_string(),
            name: "Plane1".to_string(),
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            size: cgmath::Vector3::new(100.0, 100.0, 100.0),
            rotation: cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0)),
            mesh_rendering_info: vec![]
        };

        let plane_go: GameObject = GameObject::new(&plane_go_create_info, &asset_manager);

        game_objects.push(barrel_go);
        game_objects.push(plane_go);

        // load camera
        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        //let game_objects_size = 3;

        // let mut model_uniforms: Vec<Uniform<ModelUniform>> = Vec::new();
        // for _g in 0..game_objects_size {
        //     let model_uniform = Uniform::new(ModelUniform::new(), &device);

        //     model_uniforms.push(model_uniform);
        // }

        let mut model_uniforms: HashMap<usize, Uniform<ModelUniform>> = HashMap::new();
        for game_object in game_objects.iter() {
            model_uniforms.insert(game_object.object_id, Uniform::new(ModelUniform::new(), &device));
        }

        let animated_game_object_id = unique_id::next_id();
        model_uniforms.insert(animated_game_object_id, Uniform::new(ModelUniform::new(), &device));

        // load uniforms
        let bind_group_layout = BindGroupManager::create_uniform_bind_group_layout(
            &device,
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            Some(format!("bind_group_layout for {}", std::any::type_name::<ModelUniform>()).as_ref()))
        .unwrap();

        let wgpu_uniforms = WgpuUniforms { 
            camera: Uniform::new(CameraUniform::new(), &device),
            models: model_uniforms,
            animation: Uniform::new(AnimationUniform::new(), &device),
            light: Uniform::new(LightUniform::new(), &device),
            bind_group_layout
        };

        // load fbos
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // render groups
        let lighting_pass = LightingPass::new(&context, &wgpu_uniforms, &asset_manager);
        let postprocess_pass = PostProcessPass::new(&device, &config);
        let animation_pass = AnimationPass::new(&device, &wgpu_uniforms);
        let skybox_pass = SkyboxPass::new(&context, &asset_manager, &wgpu_uniforms);
 
        // load shaders
        let debug_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/debug.wgsl").into()),
        });

        let instance_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/instance.wgsl").into()),
        });

        let instances = Instance::get_instances();
        let instance_data = Instance::get_instance_data();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Instance Buffer"),
         contents: bytemuck::cast_slice(&instance_data),
         usage: wgpu::BufferUsages::VERTEX,
        });

        let texture_bind_group_layout = &asset_manager.get_material_by_name("barrel_BLUE").unwrap().bind_group_layout;
 
        // pipeline layouts

        let instance_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render_Pipeline_Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &wgpu_uniforms.camera.bind_group_layout, &wgpu_uniforms.light.bind_group_layout],
                push_constant_ranges: &[],
        });

        let debug_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug_Pipeline_Layout"),
                bind_group_layouts: &[&wgpu_uniforms.camera.bind_group_layout, &wgpu_uniforms.light.bind_group_layout],
                push_constant_ranges: &[],
        });

        // render pipelines
        let debug_render_pipeline = PipelineManager::create_pipeline(&device, &debug_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &debug_shader_module, &[Vertex::desc()], Some("2")).unwrap();
        let instance_render_pipeline = PipelineManager::create_pipeline(&device, &instance_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &instance_shader_module, &[Vertex::desc(), InstanceUniform::desc()], Some("3")).unwrap();

        return Self {
            window,
            wgpu_context: context,
            asset_manager,
            camera,
            projection,
            camera_controller,
            instance_buffer,
            instances,
            instance_render_pipeline,
            depth_texture,
            debug_render_pipeline,
            egui_renderer,
            game_objects,
            animated_game_object_id,
            lighting_pass,
            postprocess_pass,
            animation_pass,
            skybox_pass,
            wgpu_uniforms,
        };
    }

    pub fn update(&mut self, dt: std::time::Duration) {
      let delta_time = dt.as_secs_f32();
      let queue = self.wgpu_context.get_queue();

      // camera uniform
      self.camera_controller.update_camera(&mut self.camera, dt);
      let mut updated_camera = CameraUniform::new();
      updated_camera.update(&self.camera, &self.projection);

      self.wgpu_uniforms.camera.update_direct(&queue, &updated_camera);

      // model uniform (0) glb model
      if let Some(glb_model) = self.asset_manager.get_model_by_name_mut("glock") {
          glb_model.update(delta_time);
          let skin_uniform = self.wgpu_uniforms.animation.value_mut();

          if let Some(skin) = glb_model.skins.get(0) {
           for (i, joint) in skin.joints().iter().enumerate() {
            if i >= MAX_JOINTS_PER_MESH {
             break; 
            }

           // Convert cgmath::Matrix4 to [[f32; 4]; 4]
           skin_uniform.joint_matrices[i] = joint.matrix().into();
         }
       }
      }

      self.wgpu_uniforms.animation.update(&queue);

      let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(10.0, 2.0, 0.0));
      let rotation = cgmath::Matrix4::from_angle_x(cgmath::Rad(0.0));
      let scale = cgmath::Matrix4::from_scale(1.5);
      let model_matrix = translation * rotation * scale;

    //   let model_uniform = self.wgpu_uniforms.models[0].value_mut();
    //   model_uniform.update(&model_matrix);
    //   self.wgpu_uniforms.models[0].update(&queue); 

      if let Some(model_uniform) = self.wgpu_uniforms.models.get_mut(&self.animated_game_object_id) {
        model_uniform.value_mut().update(&model_matrix);
        model_uniform.update(&queue);
      }

    //   // model uniform (1) plane model
    //   let p_translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0));
    //   let p_rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0));
    //   let p_scale = cgmath::Matrix4::from_scale(100.0);
    //   let p_model_matrix = p_translation * p_rotation * p_scale;

    //   let mut updated_model2 = ModelUniform::new();
    //   updated_model2.update(&p_model_matrix);

    //   self.wgpu_uniforms.models[1].update_direct(&queue, &updated_model2);

    //    // model uniform (2) barrel model
    //   let f_translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 2.0, 0.0));
    //   let f_rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0));
    //   let f_scale = cgmath::Matrix4::from_scale(5.0);
    //   let f_model_matrix = f_translation * f_rotation * f_scale;

    //   let mut updated_model3 = ModelUniform::new();
    //   updated_model3.update(&f_model_matrix);

    //   self.wgpu_uniforms.models[2].update_direct(&queue, &updated_model3);

    for game_object in &self.game_objects {
        if let Some(model_uniform) = self.wgpu_uniforms.models.get_mut(&game_object.object_id) {
            model_uniform.value_mut().update(&game_object.get_model_matrix());
            model_uniform.update(&queue);
        }
    }

      // update light position
      let light_uniform = self.wgpu_uniforms.light.value_mut();
      let old_position: cgmath::Vector3<_> = light_uniform.position.into();
      light_uniform.position = (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_position).into();
      self.wgpu_uniforms.light.update(&queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        let device = self.wgpu_context.get_device();
        let surface = self.wgpu_context.get_surface();
        let queue = self.wgpu_context.get_queue();
        let config = self.wgpu_context.get_surface_config();

        if !self.wgpu_context.is_surface_configured() {
            return Ok(());
        }

        // get the texture to render to
        let output = surface.get_current_texture()?;
        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("First_Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.postprocess_pass.get_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

    
       self.lighting_pass.render(&mut render_pass, &self.wgpu_uniforms, &self.asset_manager, &self.game_objects);
       self.animation_pass.render(&mut render_pass, &self.wgpu_uniforms, &self.asset_manager, self.animated_game_object_id);

        // instance pass
    //     render_pass.set_pipeline(&self.instance_render_pipeline);

    //     render_pass.set_bind_group(0, &self.asset_manager.get_material_by_name("barrel_BLUE").unwrap().bind_group, &[]);
    //     render_pass.set_bind_group(1, &self.wgpu_uniforms.camera.bind_group, &[]);
    //     render_pass.set_bind_group(2, &self.wgpu_uniforms.light.bind_group, &[]);

    //    for mesh in &self.obj_model.meshes {
    //     render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    //     render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    //     render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //     render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as _);
    //    }

       // debug pass
       render_pass.set_pipeline(&self.debug_render_pipeline);

       render_pass.set_bind_group(0, &self.wgpu_uniforms.camera.bind_group, &[]);
       render_pass.set_bind_group(1, &self.wgpu_uniforms.light.bind_group, &[]);
       
       if let Some(debug_cube) = self.asset_manager.get_model_by_name("Cube") {
        for mesh in &debug_cube.meshes {
         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
       }

       // skybox
       self.skybox_pass.render(&mut render_pass, &self.wgpu_uniforms);

       drop(render_pass);

       // post process
       self.postprocess_pass.render(&mut encoder, &surface_view);

       self.egui_renderer.draw(&self.wgpu_context, &mut encoder, &self.window, surface_view, |ui| {
                        egui::Window::new("Settings")
                            .resizable(true)
                            .vscroll(true)
                            .default_open(true)
                            .show(&ui, |mut ui| {
                                ui.label("Window!");
                                ui.button("SOME BUTTON");
                                ui.label("Window!");
                                ui.label("Window!");
                                ui.label("Window!");

                                //proto_scene.egui(ui);
                            });
        });

       // submit will accept anything that implements IntoIter
       queue.submit(std::iter::once(encoder.finish()));
       output.present();

       Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.wgpu_context.get_device(), &self.wgpu_context.get_surface_config(), "depth_texture");
        self.postprocess_pass.resize(&self.wgpu_context.get_device(), width, height);
    }
}

pub struct App {
    state: Option<State>,
    last_redraw: std::time::Instant,
    window_width: f64,
    window_height: f64,
    keyboard: Keyboard,
    cursor_visible: bool,
    fps_accum: Vec<f64>,
}

impl App {
    pub fn new() -> Self {
        return Self {
            state: None,
            window_width: 1280.0,
            window_height: 720.0,
            last_redraw: std::time::Instant::now(),
            keyboard: Keyboard::new(),
            cursor_visible: false,
            fps_accum: Vec::new()
        };
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("yhwh")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.window_width,
                self.window_height,
            ));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window.set_cursor_visible(self.cursor_visible);

        let _res = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked));

        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        let state = self.state.as_mut().unwrap();
        match event {
            DeviceEvent::MouseMotion { delta } => {
               if !self.cursor_visible {
                 state.camera_controller.handle_mouse(delta.0, delta.1);
               }
            }

            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let mut state = self.state.as_mut().unwrap();

        state.camera_controller.handle_keyboard(&event);
        self.keyboard.handle_event(&event);
        state.egui_renderer.handle_input(&state.window, &event);
        match event {
            
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.window_height = size.height as f64;
                self.window_width = size.width as f64;
                state.wgpu_context.resize(size.width, size.height);
                state.resize(size.width, size.height);
            }
            
            WindowEvent::RedrawRequested => {
                if self.keyboard.key_just_pressed(KeyCode::F1) {
                    self.cursor_visible = !self.cursor_visible;
                    state.window.set_cursor_visible(self.cursor_visible);

                    if self.cursor_visible {
                      let _res = state.window.set_cursor_grab(CursorGrabMode::None);
                    } else {
                      let _res = state.window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_e| state.window.set_cursor_grab(CursorGrabMode::Locked));
                    }
                }

                if self.keyboard.key_just_pressed(KeyCode::F2) {
                    state.postprocess_pass.hotload_shader(&state.wgpu_context);
                }

                if let Some(glb_model) = state.asset_manager.get_model_by_name_mut("glock") {
                  let anim_len = glb_model.animations.as_ref().unwrap().animations().len();
                  if self.keyboard.key_just_pressed(KeyCode::KeyR) {
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

                let now = std::time::Instant::now();
                let dt = now - self.last_redraw;
                self.last_redraw = now;
         
                // fps counter
                let fps = 1.0 / dt.as_secs_f64();
                self.fps_accum.push(fps);
                if self.fps_accum.len() > 100 {
                   self.fps_accum.remove(0);
                }

                let avg_fps = self.fps_accum.iter().sum::<f64>() / self.fps_accum.len() as f64;
                state.window.set_title(&format!("FPS: {:.1}", avg_fps));

                state.egui_renderer.set_cursor_visible(self.cursor_visible);

                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.wgpu_context.resize(size.width, size.height);
                    }
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }

                self.keyboard.end_frame();
            }

            // input stuff
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: keyboard_state,
                        ..
                    },
                ..
            } => match (code, keyboard_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },

             WindowEvent::MouseWheel { delta, .. } => {
               state.camera_controller.handle_mouse_scroll(&delta);
            }

            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                // let new_red = position.x / self.window_width;
                // let new_green = position.y / self.window_height;

                // state.clear_green = new_green;
                // state.clear_red = new_red;
            }

            _ => {}
        }
    }
}

pub fn run() -> Result<(), EventLoopError> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn main() {
    run().unwrap();
}
