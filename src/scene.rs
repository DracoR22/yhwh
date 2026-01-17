use crate::{asset_manager::AssetManager, common::{create_info::{GameObjectCreateInfo, LightObjectCreateInfo}, enums::LightType}, objects::{animated_game_object::AnimatedGameObject, game_object::GameObject, light_object::LightObject}, utils::json::load_level};

pub struct Scene {
    pub game_objects: Vec<GameObject>,
    pub animated_game_objects: Vec<AnimatedGameObject>,
    pub lights: Vec<LightObject>
}

impl Scene {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let mut game_objects: Vec<GameObject> = Vec::new();
        let mut animated_game_objects: Vec<AnimatedGameObject> = Vec::new();  
        let mut lights: Vec<LightObject> = Vec::new();

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

        // TODO REMOVE HARDCODED!!!!
        lights.push(LightObject::new(&LightObjectCreateInfo {
            color: [1.0, 1.0, 1.0],
            position: [2.0, 2.0, 2.0],
            radius: 10.0,
            strength: 50.0,
            light_type: LightType::Point
        }));

        //  lights.push(LightObject::new(&LightObjectCreateInfo {
        //     color: [1.0, 0.0, 0.0],
        //     position: [10.0, 1.0, 0.0],
        //     radius: 10.0,
        //     strength: 50.0,
        //     light_type: LightType::Point
        // }));

        Self {
            game_objects,
            animated_game_objects,
            lights
        }
    }

    pub fn add_game_object(&mut self, create_info: &GameObjectCreateInfo, asset_manager: &AssetManager) {
        self.game_objects.push(GameObject::new(&create_info, asset_manager));
    }

    pub fn remove_game_object_by_id(&mut self, id: usize) {
        self.game_objects.retain(|g| g.id != id);
    }
}

// Lights
// impl Scene {
//     pub fn get_lights(&self) {

//     }
// }