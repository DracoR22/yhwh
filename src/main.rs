use cgmath::{prelude::*, Rad};

use std::{path::PathBuf, sync::{mpsc, Arc}, thread, time::Instant};

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
use yhwh::{animation::skin::MAX_JOINTS_PER_MESH, bind_group_manager::{BindGroupManager, TL}, camera::{CameraController, Projection}, cube_map::CubeMap, input::keyboard::Keyboard, instance::{Instance, InstanceUniform}, model::{self, Mesh, Model}, pipeline_manager::PipelineManager, render_passes::{animation_pass::AnimationPass, postprocess_pass::{self, PostProcessPass}}, renderer_common::SKYBOX_VERTICES, texture::{self, Texture}, uniform::Uniform, uniform_types::{AnimationUniform, CameraUniform, LightUniform, ModelUniform, WgpuUniforms}, utils::file, wgpu_context::WgpuContext};
use yhwh::{
    camera::Camera,
    vertex::Vertex,
};

pub struct GameObject {
    pub model_name: String,
    pub name: String
}

pub struct State {
    window: Arc<Window>,
    wgpu_context: WgpuContext,
    render_pipeline: wgpu::RenderPipeline,
    projection: Projection,
    camera: Camera,
    camera_controller: CameraController,
    animation_pipeline: wgpu::RenderPipeline,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    instance_render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    obj_model: Model,
    cube_model: Model,
    plane_model: Model,
    glb_model: Model,
    game_objects: Vec<GameObject>,
    debug_render_pipeline: wgpu::RenderPipeline,
    barrel_texture_bind_group: wgpu::BindGroup,
    cubemap_bind_group: wgpu::BindGroup,
    cubemap_render_pipeline: wgpu::RenderPipeline,
    cubemap_vertex_buffer: wgpu::Buffer,

    postprocess_pass: PostProcessPass,
    animation_pass: AnimationPass,
    wgpu_uniforms: WgpuUniforms,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        // init wgpu
        let context = WgpuContext::new(&window).await.unwrap();
        let config = context.get_surface_config();
        let device = context.get_device();
        let queue = context.get_queue();

        // load camera
        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        let game_objects_size = 2;

        let mut model_uniforms: Vec<Uniform<ModelUniform>> = Vec::new();
        for _g in 0..game_objects_size {
            let model_uniform = Uniform::new(ModelUniform::new(), &device);

            model_uniforms.push(model_uniform);
        }
        // load uniforms
        let wgpu_uniforms = WgpuUniforms { 
            camera: Uniform::new(CameraUniform::new(), &device),
            models: model_uniforms,
            animation: Uniform::new(AnimationUniform::new(), &device),
            light: Uniform::new(LightUniform::new(), &device)
        };

        // render groups
        let postprocess_pass = PostProcessPass::new(&device, &config);
        let animation_pass = AnimationPass::new(&device, &wgpu_uniforms);

        // load textures
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let diffuse_bytes = include_bytes!("../res/textures/phantom.png");
        let diffuse_texture = Texture::new_from_bytes(&device, &queue, diffuse_bytes, false);

        let dude_bytes = include_bytes!("../res/textures/dude.jpg");
        let dude_texture = Texture::new_from_bytes(&device, &queue, dude_bytes, false);

        let barrel_bytes = include_bytes!("../res/textures/barrel_BLUE_Albedo.png");
        let barrel_texture = Texture::new_from_bytes(&device, &queue, barrel_bytes, false);

        let barrel_nrm_bytes = include_bytes!("../res/textures/barrel_BLUE_Normal.png");
        let barrel_nrm_texture = Texture::new_from_bytes(&device, &queue, barrel_nrm_bytes, true);

        // MULTITHREAD TEXTURE LOADING!!!
        let now = std::time::SystemTime::now();

        // let (sender, receiver) = mpsc::channel::<image::DynamicImage>();

        // for entry in std::fs::read_dir("res/textures").unwrap() {
        //     let entry = entry.unwrap();

        //     println!("LOADING TEXTURE: {}", entry.path().to_str().unwrap());

        //     let sender = sender.clone();
        //     thread::spawn(move || {
        //         let data = Texture::decode_texture_from_path(&entry.path().to_str().unwrap());
        //         sender.send(data).unwrap();
        //     });
        // }

        // drop(sender);

        // let mut skybox_textures: Vec<image::DynamicImage> = Vec::new();
        // for data in receiver {
        //   // let tex = Texture::new_from_bytes(&device, &queue, &data.pixels, false);
        //   //println!("loaded texture: {}", data.name);

        //   skybox_textures.push(data);
        // }

        
        // let cubemap_texture = CubeMap::new_from_image(&device, &queue, [
        //    &skybox_textures[4].fliph(),
        //    &skybox_textures[5],
        //    &skybox_textures[6],
        //    &skybox_textures[7],
        //    &skybox_textures[8],
        //    &skybox_textures[9]
        // ]);

        let sky_right_texture = Texture::decode_texture_from_path("res/textures/SkyRight.jpg");
        let sky_left_texture = Texture::decode_texture_from_path("res/textures/SkyLeft.jpg");
        let sky_bottom_texture = Texture::decode_texture_from_path("res/textures/SkyBottom.jpg");
        let sky_top_texture = Texture::decode_texture_from_path("res/textures/SkyTop.jpg");
        let sky_front_texture = Texture::decode_texture_from_path("res/textures/SkyFront.jpg");
        let sky_back_texture = Texture::decode_texture_from_path("res/textures/SkyBack.jpg");

        let cubemap_texture = CubeMap::new_from_image(&device, &queue, [
           &sky_right_texture.fliph(),
           &sky_left_texture,
           &sky_top_texture,
           &sky_bottom_texture,
           &sky_front_texture,
           &sky_back_texture
        ]);

        let duration = now.elapsed();
        println!("Loded all textures in: {:.3?}", duration.unwrap());
 
        // load shaders
        let default_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/simple.wgsl").into()),
        });

         let debug_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/debug.wgsl").into()),
        });

        let instance_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/instance.wgsl").into()),
        });

        let cubemap_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/cube_map.wgsl").into()),
        });

        let animation_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instance_Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../res/shaders/animation.wgsl").into()),
        });

        let cubemap_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube_Vertex_Buffer"),
            contents: bytemuck::cast_slice(SKYBOX_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instances = Instance::get_instances();
        let instance_data = Instance::get_instance_data();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
         label: Some("Instance Buffer"),
         contents: bytemuck::cast_slice(&instance_data),
         usage: wgpu::BufferUsages::VERTEX,
        });

        let texture_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Float, TL::Float]).unwrap();
        let cubemap_bind_group_layout = BindGroupManager::create_texture_bind_group_layout(&device, [TL::Cube]).unwrap();
        
        let diffuse_bind_group = BindGroupManager::create_multi_texture_bind_group(&device, &texture_bind_group_layout, &[&diffuse_texture, &barrel_nrm_texture]).unwrap();
        let dude_bind_group = BindGroupManager::create_multi_texture_bind_group(&device, &texture_bind_group_layout, &[&dude_texture, &barrel_nrm_texture]).unwrap();
        let barrel_texture_bind_group = BindGroupManager::create_multi_texture_bind_group(&device, &texture_bind_group_layout, &[&barrel_texture, &barrel_nrm_texture]).unwrap();

        let cube_tex = texture::Texture { sampler: cubemap_texture.sampler, view: cubemap_texture.view, texture: cubemap_texture.texture };
        let cubemap_bind_group = BindGroupManager::create_texture_bind_group(&device, &cubemap_bind_group_layout, &cube_tex).unwrap();
 
        // pipeline layouts
        let animation_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Animation_Pipeline_Layout"),
                bind_group_layouts: &[&wgpu_uniforms.camera.bind_group_layout, &wgpu_uniforms.models[0].bind_group_layout, &wgpu_uniforms.animation.bind_group_layout],
                push_constant_ranges: &[],
        });

        let cubemap_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cube_Map_Pipeline_Layout"),
                bind_group_layouts: &[&cubemap_bind_group_layout, &wgpu_uniforms.camera.bind_group_layout],
                push_constant_ranges: &[],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render_Pipeline_Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &wgpu_uniforms.camera.bind_group_layout, &wgpu_uniforms.models[0].bind_group_layout, &wgpu_uniforms.light.bind_group_layout],
                push_constant_ranges: &[],
        });

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
        let cubemap_render_pipeline = PipelineManager::create_cubemap_pipeline(&device, &cubemap_pipeline_layout, postprocess_pass.get_format(), &cubemap_shader_module).unwrap();
        let animation_pipeline = PipelineManager::create_pipeline(&device, &animation_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &animation_shader_module, &[Vertex::desc()], Some("animation pipeline")).unwrap();
        let render_pipeline = PipelineManager::create_pipeline(&device, &render_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &default_shader_module, &[Vertex::desc()], Some("1")).unwrap();
        let debug_render_pipeline = PipelineManager::create_pipeline(&device, &debug_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &debug_shader_module, &[Vertex::desc()], Some("2")).unwrap();
        let instance_render_pipeline = PipelineManager::create_pipeline(&device, &instance_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &instance_shader_module, &[Vertex::desc(), InstanceUniform::desc()], Some("3")).unwrap();

        let obj_model = model::load_obj_model("Barrel.obj", &device, "Barrel").await.unwrap();
        let cube_model = model::load_cube(&device, "Cube").unwrap();
        let plane_model = model::load_plane(&device, "Plane").unwrap();
        let glb_model = model::load_glb_model(&device).unwrap();

        let game_object1 = GameObject { name: "Cube1".to_string(), model_name: cube_model.name.clone() };
        let game_object2 = GameObject { name: "Plane1".to_string(), model_name: plane_model.name.clone() };

        let mut game_objects: Vec<GameObject> = Vec::new();
        game_objects.push(game_object1);
        game_objects.push(game_object2);

        let time = Instant::now();

        return Self {
            window,
            wgpu_context: context,
            render_pipeline,
            camera,
            projection,
            camera_controller,
            animation_pipeline,
            instance_buffer,
            instances,
            instance_render_pipeline,
            depth_texture,
            obj_model,
            cube_model,
            glb_model,
            game_objects,
            plane_model,
            debug_render_pipeline,
            barrel_texture_bind_group,
            cubemap_bind_group,
            cubemap_render_pipeline,
            cubemap_vertex_buffer,
            postprocess_pass,
            animation_pass,
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

      // model uniform (0)
      self.glb_model.update(delta_time);
      let skin_uniform = self.wgpu_uniforms.animation.value_mut();
      if let Some(skin) = self.glb_model.skins.get(0) {
        for (i, joint) in skin.joints().iter().enumerate() {
         if i >= MAX_JOINTS_PER_MESH {
            break; 
         }

         // Convert cgmath::Matrix4 to [[f32; 4]; 4]
         skin_uniform.joint_matrices[i] = joint.matrix().into();
        }
      }

      self.wgpu_uniforms.animation.update(&queue);

      let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(10.0, 2.0, 0.0));
      let rotation = cgmath::Matrix4::from_angle_x(cgmath::Rad(0.0));
      let scale = cgmath::Matrix4::from_scale(1.5);
      let model_matrix = translation * rotation * scale;

      let model_uniform = self.wgpu_uniforms.models[0].value_mut();
      model_uniform.update(&model_matrix);
      self.wgpu_uniforms.models[0].update(&queue); 

      // model uniform (1)
      let p_translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0));
      let p_rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0));
      let p_scale = cgmath::Matrix4::from_scale(100.0);
      let p_model_matrix = p_translation * p_rotation * p_scale;

    //   self.model_uniform.update_model_matrix(&p_model_matrix);
    let mut updated_model2 = ModelUniform::new();
    updated_model2.update(&p_model_matrix);

    self.wgpu_uniforms.models[1].update_direct(&queue, &updated_model2);

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
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        // default pass
        render_pass.set_pipeline(&self.render_pipeline);

    //     // uniforms
    //     render_pass.set_bind_group(0, &self.barrel_texture_bind_group, &[]);
    //     render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
    //     render_pass.set_bind_group(2, &self.model_bind_groups[0], &[]);
    //     render_pass.set_bind_group(3, &self.light_bind_group, &[]);

    //    for mesh in &self.glb_model.meshes {
    //      render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    //      render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    //      render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
    //    }

         // uniforms
        render_pass.set_bind_group(0, &self.barrel_texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.wgpu_uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(2, &self.wgpu_uniforms.models[1].bind_group, &[]);
        render_pass.set_bind_group(3, &self.wgpu_uniforms.light.bind_group, &[]);

       for mesh in &self.plane_model.meshes {
         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
       }

        // animation
       self.animation_pass.render(&self.wgpu_uniforms, &mut render_pass, &self.glb_model);

        // instance pass
        render_pass.set_pipeline(&self.instance_render_pipeline);

        render_pass.set_bind_group(0, &self.barrel_texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.wgpu_uniforms.camera.bind_group, &[]);
        render_pass.set_bind_group(2, &self.wgpu_uniforms.light.bind_group, &[]);

       for mesh in &self.obj_model.meshes {
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as _);
       }

       // debug pass
       render_pass.set_pipeline(&self.debug_render_pipeline);

       render_pass.set_bind_group(0, &self.wgpu_uniforms.camera.bind_group, &[]);
       render_pass.set_bind_group(1, &self.wgpu_uniforms.light.bind_group, &[]);
       
       for mesh in &self.cube_model.meshes {
         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
       }

       // skybox
       render_pass.set_pipeline(&self.cubemap_render_pipeline);

       render_pass.set_bind_group(0, &self.cubemap_bind_group, &[]);
       render_pass.set_bind_group(1, &self.wgpu_uniforms.camera.bind_group, &[]);

       render_pass.set_vertex_buffer(0, self.cubemap_vertex_buffer.slice(..));
       render_pass.draw(0..(SKYBOX_VERTICES.len() / 3) as u32, 0..1);

       drop(render_pass);

       // post process
       self.postprocess_pass.render(&mut encoder, &view);

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
                state.camera_controller.handle_mouse(delta.0, delta.1);
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

                let anim_len = state.glb_model.animations.as_ref().unwrap().animations().len();
                 if self.keyboard.key_just_pressed(KeyCode::KeyR) {
                   let play_back_state = state.glb_model.get_animation_playback_state().unwrap();
                   let mut current_anim = play_back_state.current;

                  if current_anim + 1 < anim_len {
                   current_anim += 1;
                  } else {
                   current_anim = 0;
                  }

                  state.glb_model.set_current_animation(current_anim);
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
