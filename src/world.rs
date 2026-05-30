use std::{collections::HashMap, fs::{self, File}};
use std::io::Write;

use crate::{asset_manager::AssetManager, common::create_info::{DoorObjectCreateInfo, GameObjectCreateInfo, LightObjectCreateInfo, MapCreateInfo, SceneCreateInfo}, objects::game_object::GameObject, scene::Scene};

pub struct World {
    chunks: HashMap<String, Scene>
}

impl World {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let mut chunks = HashMap::<String, Scene>::new();

        let map_path = "res/maps/house.json";
        let map_json = fs::read_to_string(map_path).expect("Could not load map");
        let map_create_info: MapCreateInfo = serde_json::from_str(&map_json).expect("Could not deserialize map");

        for chunk_file in map_create_info.chunks.iter() {
            let chunk_path = format!("res/scenes/{}", chunk_file);
            let chunk_json = fs::read_to_string(chunk_path).expect("Could not load chunk");
            let chunk_create_info: SceneCreateInfo = serde_json::from_str(&chunk_json).expect("Could not deserialize chunk");
            let chunk = Scene::new(&chunk_create_info, asset_manager);

            chunks.insert(chunk_file.to_string(), chunk);
        }

        Self {
            chunks
        }
    }

    pub fn for_each_chunk_mut<F: FnMut(&mut Scene)>(&mut self, mut f: F) {
        for chunk in self.chunks.values_mut() {
           f(chunk)
        }
    }

    pub fn for_each_chunk<F: FnMut(&Scene)>(&self, mut f: F) {
        for chunk in self.chunks.values() {
            f(chunk)
        }
    }

    pub fn light_count(&self) -> usize {
        let mut count = 0;
        for chunk in self.chunks.values() {
           count += chunk.lights.len();
        }

        count
    }

    pub fn add_chunk(&mut self, create_info: &SceneCreateInfo, asset_manager: &AssetManager) {
        let chunk = Scene::new(&create_info, asset_manager);
        self.chunks.insert(chunk.file_name.clone(), chunk);
    }

    pub fn save_map(&self, map_file_name: &str, asset_manager: &AssetManager) {
        let mut chunk_create_infos = Vec::<String>::new();
        self.for_each_chunk(|chunk| {
            let chunk_create_info = chunk.get_create_info(asset_manager);
            self.save_chunk(&chunk_create_info);
            chunk_create_infos.push(chunk.file_name.clone());
        });

        let map_create_info = MapCreateInfo {
            chunks: chunk_create_infos,
            name: map_file_name.to_string().replace(".json", "")
        };

        let json = serde_json::to_string_pretty(&map_create_info).unwrap();
        match File::create(String::from("res/maps/") + &map_create_info.name + ".json").unwrap().write_all(json.as_bytes()) {
            Ok(_msg) => { println!("Map saved!") },
            Err(err) => { println!("Could not save map. Error: {}", err) }
        }
    }

    pub fn save_chunk(&self, chunk_create_info: &SceneCreateInfo) {
        let json = serde_json::to_string_pretty(&chunk_create_info).unwrap();

        match File::create(String::from("res/scenes/") + &chunk_create_info.name + ".json").unwrap().write_all(json.as_bytes()) {
            Ok(_msg) => { println!("Chunk saved!") },
            Err(err) => { println!("Could not save chunk. Error: {}", err) }
        }
    }
}