use std::{collections::HashMap, path::Path, sync::mpsc, thread};

use crate::{cube_map::CubeMap, material::Material, texture::{Texture, TextureData}, wgpu_context::WgpuContext};

pub struct AssetManager {
    textures: HashMap<String, Texture>,
    materials: HashMap<String, Material>
}

impl AssetManager {
    pub fn new(ctx: &WgpuContext) -> Self {
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

        Self {
            textures: texture_map,
            materials: Default::default()
        }
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
                    self.get_texture_by_name(&format!("{material_name}_ALB.png")).unwrap(),
                    self.get_texture_by_name(&format!("{material_name}_NRM.png")).unwrap(),
                    self.get_texture_by_name(&format!("{material_name}_ALB.png")).unwrap()
                ]);

                self.materials.insert(material_name, material);
            }
        }
    }

    pub fn get_material_by_name(&self, name: &str) -> Option<&Material> {
        if self.materials.contains_key(name) {
            self.materials.get(name)
        } else {
             println!("AssetManager::get_material_by_name() error: material {name} not found!");
            None
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
}