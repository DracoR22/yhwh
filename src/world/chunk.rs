use yhwh_core::common::create_info::{AnimatedGameObjectCreateInfo, DoorObjectCreateInfo, GameObjectCreateInfo, LightObjectCreateInfo, ChunkCreateInfo};

use crate::{asset_manager::AssetManager, objects::{animated_game_object::AnimatedGameObject, door_object::DoorObject, game_object::GameObject, light_object::LightObject}};

pub struct Chunk {
    pub file_name: String,
    pub name: String,
    pub game_objects: Vec<GameObject>,
    pub animated_game_objects: Vec<AnimatedGameObject>,
    pub door_objects: Vec<DoorObject>,
    pub lights: Vec<LightObject>
}

impl Chunk {
    pub fn new(create_info: &ChunkCreateInfo, asset_manager: &AssetManager) -> Self {
        let mut game_objects = Vec::<GameObject>::new();
        let mut animated_game_objects = Vec::<AnimatedGameObject>::new();  
        let mut door_objects = Vec::<DoorObject>::new();
        let mut lights = Vec::<LightObject>::new();

        for game_object_create_info in create_info.game_objects.iter() {
            game_objects.push(GameObject::new(&game_object_create_info, &asset_manager));
        }

        for door_create_info in create_info.door_objects.iter() {
            door_objects.push(DoorObject::new(&door_create_info, asset_manager));
        }

        for light_create_info in create_info.lights.iter() {
            lights.push(LightObject::new(&light_create_info));
        }

        let file_name = format!("{}{}", create_info.name, ".json");

        Self {
            file_name,
            name: create_info.name.clone(),
            game_objects,
            animated_game_objects,
            lights,
            door_objects
        }
    }

    pub fn add_game_object(&mut self, create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) {
        self.game_objects.push(GameObject::new(&create_info, asset_manager));
    }

    pub fn remove_game_object_by_id(&mut self, id: usize) {
        self.game_objects.retain(|g| g.id != id);
    }

    pub fn add_animated_game_object(&mut self, create_info: &AnimatedGameObjectCreateInfo, asset_manager: &AssetManager) -> usize {
        let animated_game_object = AnimatedGameObject::new(create_info, asset_manager);
        let id = animated_game_object.id;
        self.animated_game_objects.push(animated_game_object);

        id
    }

    pub fn get_create_info(&self, asset_manager: &AssetManager) -> ChunkCreateInfo {
        ChunkCreateInfo {
            name: self.name.clone(),
            game_objects: self.game_objects.iter().map(|o| o.create_info(asset_manager)).collect(),
            door_objects: self.door_objects.iter().map(|o| o.get_create_info(asset_manager)).collect(),
            lights: self.lights.iter().map(|o| o.get_create_info()).collect(),
        }
    }
}

// Lights
impl Chunk {
    pub fn add_light(&mut self, create_info: &LightObjectCreateInfo) {
        self.lights.push(LightObject::new(&create_info));
    }
}

// doors
impl Chunk {
    pub fn add_door_object(&mut self, create_info: &DoorObjectCreateInfo, asset_manager: &AssetManager) {
        self.door_objects.push(DoorObject::new(create_info, asset_manager));
    }
}