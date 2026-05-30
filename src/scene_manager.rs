use std::{collections::{HashMap, HashSet}, fs};

use crate::{asset_manager::AssetManager, common::create_info::SceneCreateInfo, scene::Scene};

pub struct SceneManager {
    loaded_scenes: HashMap<String, Scene>,
    active_scenes: HashSet<String>
}

impl SceneManager {
    pub fn new() -> Self {
        Self {
            loaded_scenes: HashMap::new(),
            active_scenes: HashSet::new()
        }
    }

    pub fn load_scene(&mut self, asset_manager: &AssetManager, scene_file_name: &str) {
        if self.loaded_scenes.contains_key(&scene_file_name.to_string()) {
            println!("Scene {} was already loaded!", scene_file_name);
            return;
        }

        let path = format!("res/scenes/{}", scene_file_name);

        match fs::read_to_string(&path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(create_info) => {
                         let scene = Scene::new(&create_info, asset_manager);
                         self.loaded_scenes.insert(scene_file_name.to_string(), scene);
                    },
                    Err(err) => {
                        println!("Could not deserialize scene {}: {}", path, err);
                    }
                }
            },
            Err(err) => {
                println!("Failed to load scene {}: {}", path, err);
            }
        }
    }

    pub fn activate_scene(&mut self, file_name: &str) {
        if self.loaded_scenes.contains_key(file_name) {
            self.active_scenes.insert(file_name.to_string());
        } else {
            println!("SceneManager::make_active() error: {} does not exist!", file_name)
        }
    }

    pub fn deactivate_scene(&mut self, file_name: &str) {
        self.active_scenes.remove(file_name);
    }

    pub fn active_scenes(&self) -> impl Iterator<Item = &Scene> {
        self.active_scenes
            .iter()
            .filter_map(|id| {
                self.loaded_scenes.get(id)
            })
    }

    pub fn active_scene_ids(&self) -> impl Iterator<Item = &String> {
        self.active_scenes.iter()
    }

    pub fn get_scene_by_file_name(&self, file_name: &str) -> Option<&Scene> {
        self.loaded_scenes.get(file_name)
    }

    pub fn get_scene_by_file_name_mut(&mut self, file_name: &str) -> Option<&mut Scene> {
        self.loaded_scenes.get_mut(file_name)
    }
}