use std::{collections::HashMap, sync::Arc};

use wgpu::Texture;
use winit::{keyboard::KeyCode, window::Window};

use crate::{asset_manager::AssetManager, bind_group_manager::BindGroupManager, egui_renderer::{egui_renderer::EguiRenderer, windows::scene_hierarchy::SceneHierarchyWindow}, engine::GameData, input::keyboard::Keyboard, objects::game_object::{GameObject, GameObjectCreateInfo, MeshReneringCreateInfo}, pipeline_manager::PipelineManager, render_passes::{animation_pass::AnimationPass, lighting_pass::LightingPass, postprocess_pass::PostProcessPass, skybox_pass::SkyboxPass}, texture, uniform::Uniform, uniform_manager::{AnimationUniform, CameraUniform, LightUniform, ModelUniform, UniformManager}, utils::unique_id, vertex::Vertex, wgpu_context::WgpuContext};

pub struct WgpuRenderer {
    wgpu_context: WgpuContext,
    pub asset_manager: AssetManager,
    depth_texture: texture::Texture,
    debug_render_pipeline: wgpu::RenderPipeline,
    pub egui_renderer: EguiRenderer,
    game_objects: Vec<GameObject>,
    animated_game_object_id: usize,

    postprocess_pass: PostProcessPass,
    lighting_pass: LightingPass,
    animation_pass: AnimationPass,
    skybox_pass: SkyboxPass,
    uniform_manager: UniformManager,
}

impl WgpuRenderer {
    pub async fn new(window: &Arc<Window>) -> Self {
        // init wgpu
        let context = WgpuContext::new(&window).await.unwrap();
        let config = context.get_surface_config();
        let device = context.get_device();

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
            Some("Uniform_Bind_Group_Layout"))
        .unwrap();

        let wgpu_uniforms = UniformManager { 
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
 
        // pipeline layouts
        let debug_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Debug_Pipeline_Layout"),
                bind_group_layouts: &[&wgpu_uniforms.camera.bind_group_layout, &wgpu_uniforms.light.bind_group_layout],
                push_constant_ranges: &[],
        });

        // render pipelines
        let debug_render_pipeline = PipelineManager::create_pipeline(&device, &debug_pipeline_layout, postprocess_pass.get_format(), Some(wgpu::TextureFormat::Depth32Float), &debug_shader_module, &[Vertex::desc()], Some("2")).unwrap();

        return Self {
            wgpu_context: context,
            asset_manager,
            depth_texture,
            debug_render_pipeline,
            egui_renderer,
            game_objects,
            animated_game_object_id,
            lighting_pass,
            postprocess_pass,
            animation_pass,
            skybox_pass,
            uniform_manager: wgpu_uniforms,
        };
    }

    pub fn render(&mut self, window: &Window, game_data: &GameData) -> Result<(), wgpu::SurfaceError> {
        // submit uniforms
        self.uniform_manager.submit_animation_uniforms(&self.wgpu_context, &mut self.asset_manager, game_data.delta_time);
        self.uniform_manager.submit_model_uniforms(&self.wgpu_context, &self.game_objects, self.animated_game_object_id);
        self.uniform_manager.submit_camera_uniforms(&self.wgpu_context, &game_data.camera, &game_data.projection);
        self.uniform_manager.submit_light_uniforms(&self.wgpu_context, game_data.delta_time);
        
        window.request_redraw();

        let device = self.wgpu_context.get_device();
        let surface = self.wgpu_context.get_surface();
        let queue = self.wgpu_context.get_queue();

        if !self.wgpu_context.is_surface_configured() {
            return Ok(());
        }

        let swapchain_fbo = surface.get_current_texture()?;
        let surface_view = swapchain_fbo.texture.create_view(&wgpu::TextureViewDescriptor::default());

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

    
       self.lighting_pass.render(&mut render_pass, &self.uniform_manager, &self.asset_manager, &self.game_objects);
       self.animation_pass.render(&mut render_pass, &self.uniform_manager, &self.asset_manager, self.animated_game_object_id);

       // debug pass
       render_pass.set_pipeline(&self.debug_render_pipeline);

       render_pass.set_bind_group(0, &self.uniform_manager.camera.bind_group, &[]);
       render_pass.set_bind_group(1, &self.uniform_manager.light.bind_group, &[]);
       
       if let Some(debug_cube) = self.asset_manager.get_model_by_name("Cube") {
        for mesh in &debug_cube.meshes {
         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
         render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
       }

       // skybox
       self.skybox_pass.render(&mut render_pass, &self.uniform_manager);

       drop(render_pass);

       // post process
       self.postprocess_pass.render(&mut encoder, &surface_view);

       self.egui_renderer.draw(&self.wgpu_context, &mut encoder, &window, surface_view, |ui| {
           SceneHierarchyWindow::draw(ui);
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
        self.depth_texture = texture::Texture::create_depth_texture(&self.wgpu_context.get_device(), &self.wgpu_context.get_surface_config(), "depth_texture");
        self.postprocess_pass.resize(&self.wgpu_context.get_device(), width, height);
    }

    pub fn resize_ctx(&mut self, width: u32, height: u32) {
        self.wgpu_context.resize(width, height);
    }

    pub fn hot_load_shaders(&mut self) {
         self.postprocess_pass.hotload_shader(&self.wgpu_context);
         println!("Hot-Loaded shaders!");
    }
}