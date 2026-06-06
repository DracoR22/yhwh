use std::{arch::naked_asm, collections::HashMap, fs::{self, File}};
use std::io::Write;

use cgmath::{Matrix4, SquareMatrix};

use crate::{animation::skin::MAX_JOINTS_PER_MESH, asset_manager::AssetManager, common::{create_info::{AnimatedGameObjectCreateInfo, DoorObjectCreateInfo, GameObjectCreateInfo, LightObjectCreateInfo, MapCreateInfo, SceneCreateInfo}, types::AnimatedRenderData}, frustum::Frustum, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject, light_object::LightObject}, render_core::render_data_manager::RenderDataManager, scene::Scene};

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

        // hard coded!!
        let create_info = AnimatedGameObjectCreateInfo {
            model_name: "untitled2".to_string(),
            position: [10.0, 2.0, 10.0],
            rotation: [0.0, 0.0, 0.0],
            size: [0.08, 0.08, 0.08],
            tex_scale: [1.0, 1.0],
            loop_anim: true,
            mesh_rendering_info: vec![]
        };

        chunks.get_mut("test3.json").unwrap().add_animated_game_object(&create_info, asset_manager);

        Self {
            chunks
        }
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

    pub fn submit_render_items(&self, render_data_manager: &mut RenderDataManager, frustum: &Frustum) {
        render_data_manager.clear();
        
        self.for_each_chunk(|chunk| {
            for game_object in chunk.game_objects.iter() {
                render_data_manager.submit_render_items(game_object.render_items(), frustum);

                if game_object.is_selected {
                    render_data_manager.submit_outlined_render_items(game_object.render_items());
                }
            }

            for door_object in chunk.door_objects.iter() {
                render_data_manager.submit_render_items(door_object.render_items(), frustum);

                if door_object.is_selected {
                    render_data_manager.submit_outlined_render_items(door_object.render_items());
                }
            }
  
            for animated_object in chunk.animated_game_objects.iter() {
                let mut joint_matrices = [Matrix4::identity(); MAX_JOINTS_PER_MESH];

                if let Some(skin) = animated_object.skins.get(0) {
                    for (i, joint) in skin.joints().iter().enumerate() {
                        if i >= MAX_JOINTS_PER_MESH {
                            break;
                        }

                        joint_matrices[i] = joint.matrix();
                    }
                }

                render_data_manager.submit_animated_render_data(AnimatedRenderData {
                    object_id: animated_object.id,
                    joint_matrices
                });
                render_data_manager.submit_animated_render_items(animated_object.render_items());
            }
        }); 
    }
}

// iterators
impl World {
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

    pub fn for_each_light<F: FnMut(&LightObject)>(&self, mut f: F) {
        for chunk in self.chunks.values() {
            for light in chunk.lights.iter() {
                f(light)
            }
        }
    }

    pub fn light_count(&self) -> usize {
        let mut count = 0;
        for chunk in self.chunks.values() {
           count += chunk.lights.len();
        }

        count
    }
}

// getters
impl World {
    pub fn animated_game_object(&self, id: usize) -> Option<&AnimatedGameObject> {
        for chunk in self.chunks.values() {
            for object in chunk.animated_game_objects.iter() {
                if object.id == id {
                    return Some(object)
                }
            }
        };

        None
    }
}