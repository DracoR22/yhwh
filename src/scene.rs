use crate::{asset_manager::AssetManager, common::create_info::GameObjectCreateInfo, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject}, utils::json::load_level};

pub struct Scene {
    pub game_objects: Vec<GameObject>,
    pub animated_game_objects: Vec<AnimatedGameObject>
}

impl Scene {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let mut game_objects: Vec<GameObject> = Vec::new();
        let mut animated_game_objects: Vec<AnimatedGameObject> = Vec::new();  

        let level = load_level().expect("Could not load level!!");

        for create_info in level.game_objects {
            game_objects.push(GameObject::new(&create_info, &asset_manager));
        }

        // todo: serialize animated game objects into json instead
         let glock_create_info = GameObjectCreateInfo {
            model_name: "glock".to_string(),
            position: [10.0, 2.0, 0.0],
            rotation: [1.0, 1.0, 1.0],
            size: [1.5, 1.5, 1.5],
            tex_scale: [1.0, 1.0],
            mesh_rendering_info: vec![]
        };

        animated_game_objects.push(AnimatedGameObject::new(&glock_create_info, &asset_manager));

        Self {
            game_objects,
            animated_game_objects
        }
    }

    pub fn add_game_object(&mut self, create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) {
        self.game_objects.push(GameObject::new(&create_info, asset_manager));
    }

    pub fn remove_game_object_by_id(&mut self, id: usize) {
        self.game_objects.retain(|g| g.id != id);
    }
}