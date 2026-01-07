use std::{collections::HashMap, path::Path, sync::mpsc, thread};

use crate::{material::Material, model::{self, Mesh, Model}, texture::{Texture, TextureData}, wgpu_context::WgpuContext};

pub struct AssetManager {
    models: Vec<Model>,
    model_index_map: HashMap<String, usize>,
    textures: HashMap<String, Texture>,
    material_index_map: HashMap<String, usize>,
    materials: Vec<Material>,
    mesh_index_map: HashMap<String, usize>,
    meshes: Vec<Mesh>
}

impl AssetManager {
    pub fn new(ctx: &WgpuContext) -> Self {
        let textures = Self::load_all_textures(&ctx);
        let (models, model_index_map) = Self::load_models(&ctx);

        let mut meshes: Vec<Mesh> = Vec::new();
        let mut mesh_index_map: HashMap<String, usize> = HashMap::new();
        for model in models.iter() {
            for mesh in &model.meshes {
                meshes.push(mesh.clone());
                mesh_index_map.insert(mesh.name.clone(), meshes.len() - 1);
            }
        }

        Self {
            textures,
            models,
            model_index_map,
            meshes,
            mesh_index_map,
            materials: Default::default(),
            material_index_map: Default::default()
        }
    }

    pub fn load_all_textures(ctx: &WgpuContext) -> HashMap<String, Texture> {
        let now = std::time::SystemTime::now();

        let (sender, receiver) = mpsc::channel::<TextureData>();
        for entry in std::fs::read_dir("res/textures").unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();

            println!("LOADING TEXTURE: {}", entry.path().to_str().unwrap());

            let sender = sender.clone();
            thread::spawn(move || {
                let data = TextureData { 
                    image: Texture::decode_texture_from_path(&entry.path().to_str().unwrap()),
                    name: file_name.to_str().unwrap_or("Default_ALB.png") .to_string()
                };
                sender.send(data).unwrap();
            });
        }

        drop(sender);

        let mut texture_map: HashMap<String, Texture> = HashMap::new();
        for data in receiver {
             let is_normal_map = data.name.contains("_NRM");
               let texture = Texture::allocate_gpu_from_image(&ctx.device, &ctx.queue, &data.image, is_normal_map);
               texture_map.insert(data.name, texture);
        }

        let duration = now.elapsed();
        println!("Loded all textures in: {:.3?}", duration.unwrap());

        texture_map
    }

    pub fn get_texture_by_name(&self, name: &str) -> Option<&Texture> {
        if self.textures.contains_key(name) {
            self.textures.get(name)
        } else {
            println!("AssetManager::get_texture_by_name() error: texture {name} not found!");
            None
        }
    }

    pub fn build_materials(&mut self, device: &wgpu::Device) {
        for (key, _texture) in &self.textures {
            if key.contains("_ALB") {
                let material_name = Self::get_texture_material_name(key);

                let material = Material::new(&material_name, &device, [
                    self.get_texture_by_name(&format!("{material_name}_ALB.png")).unwrap_or(self.get_texture_by_name("Default_ALB.png").unwrap()),
                    self.get_texture_by_name(&format!("{material_name}_NRM.png")).unwrap_or(self.get_texture_by_name("Default_NRM.png").unwrap()),
                    self.get_texture_by_name(&format!("{material_name}_RMA.png")).unwrap_or(self.get_texture_by_name("Default_RMA.png").unwrap())
                ]);

                self.materials.push(material);
                self.material_index_map.insert(material_name, self.materials.len() - 1);
            }
        }
    }

    pub fn get_material_by_name(&self, name: &str) -> Option<&Material> {
        if let Some(&index) = self.material_index_map.get(name) {
             Some(&self.materials[index])
        } else {
            println!("AssetManager::get_material_by_name() error: material {name} not found!");
             None
        }
    }

    pub fn get_material_by_index(&self, index: usize) -> Option<&Material> {
        if let Some(material) = self.materials.get(index) {
            Some(material)
        } else {
             println!("AssetManager::get_material_by_index() error: material with index {index} not found!");
             None
        }
    }

    pub fn get_material_index_by_name(&self, name: &str) -> usize {
        if let Some(&index) = self.material_index_map.get(name) {
            index
        } else {
            println!("AssetManager::get_material_index_by_name() error: material {name} not found!");
            0
        }
    }

    fn get_texture_material_name(name: &str) -> String {
       let path = Path::new(name);
       let mut material_name = String::new();
       let suffix = "_ALB";

       if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(base) = stem.strip_suffix(suffix) {
                material_name = base.to_string();
            }
        }

        material_name
    }

    pub fn get_all_materials(&self) -> &Vec<Material> {
        &self.materials
    }

     pub fn get_phong_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
      if !self.materials.is_empty() {
        Some(&self.materials[0].bind_group_layout)
      } else {
        None
      }
    }
}

// models
impl AssetManager {
     pub fn load_models(ctx: &WgpuContext) -> (Vec<Model>, HashMap<String, usize>) {
        let mut models: Vec<Model> = Vec::new();
        let mut model_index_map: HashMap<String, usize> = HashMap::new();
        for entry in std::fs::read_dir("res/models").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if extension.eq_ignore_ascii_case("glb") {
                    match model::load_glb_model(&ctx.device, &entry.path().to_str().unwrap()) {
                        Ok(res) => { 
                           let model_name = res.name.clone();
                           models.push(res);
                           model_index_map.insert(model_name, models.len() -1);
                         },
                        Err(err) => {
                            println!("AssetManager::load_models() error with file {:?} {}", path.file_name().unwrap(), err);
                        }
                    }
                }

                if extension.eq_ignore_ascii_case("obj") {
                    match model::load_obj_model_sync(&ctx.device, &entry.path().to_str().unwrap()) {
                        Ok(res) => {
                           let model_name = res.name.clone();
                           models.push(res);
                           model_index_map.insert(model_name, models.len() -1);
                        },
                        Err(err) => {
                            println!("AssetManager::load_models() error with file {:?} {}", path.file_name().unwrap(), err)
                        }
                    }
                }
            }
        }

        let cube_model = model::load_cube(&ctx.device, "Cube").unwrap();
        let cube_model_name = cube_model.name.clone();
        models.push(cube_model);
        model_index_map.insert(cube_model_name, models.len() -1);

        let plane_model = model::load_plane(&ctx.device, "Plane").unwrap();
        let plane_model_name = plane_model.name.clone();
        models.push(plane_model);
        model_index_map.insert(plane_model_name, models.len() - 1);

        (models, model_index_map)
    }

    pub fn get_model_by_name(&self, name: &str) -> Option<&Model> {
        if let Some(index) = self.model_index_map.get(name) {
            self.models.get(*index)
        } else {
            println!("AssetManager::get_model_by_name() error: model {name} not found!");
            None
        }
    }

    pub fn get_model_by_name_mut(&mut self, name: &str) -> Option<&mut Model> {
        if let Some(index) = self.model_index_map.get(name) {
            self.models.get_mut(*index)
        } else {
            println!("AssetManager::get_model_by_name() error: model {name} not found!");
            None
        }
    }

    pub fn get_models(&self) -> &Vec<Model> {
        &self.models
    }
}

// meshes
impl AssetManager {
    pub fn get_mesh_by_name(&self, name: &str) -> Option<&Mesh> {
        for mesh in &self.meshes {
           if mesh.name == name {
            return Some(mesh)
           }
        }

        return None
    }

    pub fn get_mesh_by_index(&self, index: usize) -> Option<&Mesh> {
        if index >= 0 && index < self.meshes.len() {
            return Some(&self.meshes[index])
        } 

        return  None
    }

    pub fn get_mesh_index_by_name(&self, name: &str) -> usize {
          if let Some(&index) = self.mesh_index_map.get(name) {
            index
        } else {
            println!("AssetManager::get_mesh_index_by_name() error: mesh {name} not found!");
            0
        }
    }
}